//! What the town keeps of what happened, so villagers can recall it. Not a knowledge
//! store: each kept event records who did it and who could have seen or heard it, and
//! character code decides what it noticed and what to keep in its own memory. Footsteps,
//! plans and private notes are not kept at all; most events are kept a day, and the ones
//! that matter longer (how a conversation went, money, accidents, thoughts) a month.
use crate::ids::{PlaceId, VillagerId};
use crate::protocol::Emitted;
use crate::time::{TICKS_PER_DAY, Tick};
use content::Pos;
use serde::{Deserialize, Serialize};

/// How long an event stays recallable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Keep {
    Day,
    Month,
}

impl Keep {
    #[must_use]
    pub fn ticks(self) -> u32 {
        match self {
            Keep::Day => TICKS_PER_DAY,
            Keep::Month => 28 * TICKS_PER_DAY,
        }
    }
}

/// Within this many tiles, in the same place and awake, you see it.
pub const SIGHT: u16 = 8;
/// Within this many, you hear it.
pub const EARSHOT: u16 = 16;

/// Whether an event is worth keeping, and for how long.
#[must_use]
pub fn keep(e: &Emitted) -> Option<Keep> {
    match e {
        Emitted::ConversationEnded { .. }
        | Emitted::Paid { .. }
        | Emitted::Mishap { .. }
        | Emitted::Thought { .. } => Some(Keep::Month),
        Emitted::Activity { activity, .. } if activity != "walking" => Some(Keep::Day),
        Emitted::ConversationStarted { .. }
        | Emitted::Beat { .. }
        | Emitted::TookUp { .. }
        | Emitted::FrameChanged { .. }
        | Emitted::Stocked { .. }
        | Emitted::Cleared { .. }
        | Emitted::Talked { .. } => Some(Keep::Day),
        _ => None,
    }
}

/// Events nobody else can perceive: what someone thinks, how a turn landed for them.
fn private(e: &Emitted) -> bool {
    matches!(e, Emitted::Thought { .. } | Emitted::TookUp { .. })
}

/// One kept event, where it happened, and who was in a position to perceive it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorded {
    #[serde(with = "as_json")]
    pub event: Emitted,
    pub keep: Keep,
    pub place: PlaceId,
    pub pos: Pos,
    /// Awake, in the same place and within `SIGHT`; the actor is not included.
    pub saw: Vec<VillagerId>,
    /// Awake, in the same place and within `EARSHOT` but not `SIGHT`.
    pub heard: Vec<VillagerId>,
}

/// `Emitted` is internally tagged for the viewer, which postcard cannot decode, so in the
/// snapshot a kept event travels as JSON text.
/// Events kept in the snapshot as JSON text: `Emitted` is internally tagged for the
/// viewer, and postcard cannot decode that.
pub(crate) mod as_json {
    use crate::protocol::Emitted;
    use serde::{Deserialize, Deserializer, Serializer, de, ser};

    pub fn serialize<S: Serializer>(e: &Emitted, s: S) -> Result<S::Ok, S::Error> {
        let text = serde_json::to_string(e).map_err(ser::Error::custom)?;
        s.serialize_str(&text)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Emitted, D::Error> {
        let text = String::deserialize(d)?;
        serde_json::from_str(&text).map_err(de::Error::custom)
    }

    /// The same for a list of events.
    pub mod list {
        use super::{Deserialize, Deserializer, Emitted, Serializer, de, ser};

        pub fn serialize<S: Serializer>(e: &[Emitted], s: S) -> Result<S::Ok, S::Error> {
            let text = serde_json::to_string(e).map_err(ser::Error::custom)?;
            s.serialize_str(&text)
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Emitted>, D::Error> {
            let text = String::deserialize(d)?;
            serde_json::from_str(&text).map_err(de::Error::custom)
        }
    }
}

/// How someone came to know of a kept event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Perceived {
    /// It was theirs: they did it, or it happened to them.
    Did,
    Saw,
    Heard,
}

impl Recorded {
    #[must_use]
    pub fn perceived_by(&self, who: VillagerId) -> Option<Perceived> {
        if self.event.who() == Some(who) {
            Some(Perceived::Did)
        } else if self.saw.contains(&who) {
            Some(Perceived::Saw)
        } else if self.heard.contains(&who) {
            Some(Perceived::Heard)
        } else {
            None
        }
    }
}

/// Kept events, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub entries: Vec<Recorded>,
}

/// Where a villager is and whether they can take things in, for working out witnesses.
#[derive(Debug)]
pub struct Observer {
    pub who: VillagerId,
    pub place: PlaceId,
    pub pos: Pos,
    pub awake: bool,
}

impl Record {
    /// Keeps `e` if it is worth keeping, with witnesses from `observers`.
    pub fn note(&mut self, e: &Emitted, observers: &[Observer]) {
        let Some(keep) = keep(e) else {
            return;
        };
        let Some(actor) = e.who() else {
            return;
        };
        let Some(at) = observers.iter().find(|o| o.who == actor) else {
            return;
        };
        let (mut saw, mut heard) = (Vec::new(), Vec::new());
        if !private(e) {
            for o in observers {
                if o.who == actor || !o.awake || o.place != at.place {
                    continue;
                }
                let d = o.pos.x.abs_diff(at.pos.x).max(o.pos.y.abs_diff(at.pos.y));
                if d <= SIGHT {
                    saw.push(o.who);
                } else if d <= EARSHOT {
                    heard.push(o.who);
                }
            }
        }
        self.entries.push(Recorded {
            event: e.clone(),
            keep,
            place: at.place,
            pos: at.pos,
            saw,
            heard,
        });
    }

    /// Drops what has been kept long enough.
    pub fn forget(&mut self, now: Tick) {
        self.entries
            .retain(|r| r.event.tick().plus(r.keep.ticks()) > now);
    }

    /// What `who` perceived since `since`, oldest first.
    pub fn recall(
        &self,
        who: VillagerId,
        since: Tick,
    ) -> impl Iterator<Item = (&Recorded, Perceived)> {
        self.entries
            .iter()
            .filter(move |r| r.event.tick() >= since)
            .filter_map(move |r| r.perceived_by(who).map(|p| (r, p)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(who: u16, x: u16, awake: bool) -> Observer {
        Observer {
            who: VillagerId(who),
            place: PlaceId(0),
            pos: Pos { x, y: 0 },
            awake,
        }
    }

    fn activity(tick: u32, who: u16, activity: &str) -> Emitted {
        Emitted::Activity {
            tick: Tick(tick),
            who: VillagerId(who),
            activity: activity.into(),
            cause: crate::protocol::Cause::Sim,
        }
    }

    #[test]
    fn witnesses_are_awake_and_near_and_footsteps_are_not_kept() {
        let mut r = Record::default();
        let people = [
            at(0, 0, true),
            at(1, 5, true),
            at(2, 12, true),
            at(3, 1, false),
        ];
        r.note(&activity(10, 0, "bake"), &people);
        r.note(&activity(11, 0, "walking"), &people);
        assert_eq!(r.entries.len(), 1);
        let e = &r.entries[0];
        assert_eq!(e.perceived_by(VillagerId(0)), Some(Perceived::Did));
        assert_eq!(e.perceived_by(VillagerId(1)), Some(Perceived::Saw));
        assert_eq!(e.perceived_by(VillagerId(2)), Some(Perceived::Heard));
        assert_eq!(e.perceived_by(VillagerId(3)), None, "asleep");
        assert_eq!(r.recall(VillagerId(1), Tick(0)).count(), 1);
        assert_eq!(r.recall(VillagerId(1), Tick(11)).count(), 0);
    }

    #[test]
    fn day_things_go_after_a_day() {
        let mut r = Record::default();
        r.note(&activity(10, 0, "bake"), &[at(0, 0, true)]);
        r.forget(Tick(10 + TICKS_PER_DAY - 1));
        assert_eq!(r.entries.len(), 1);
        r.forget(Tick(10 + TICKS_PER_DAY));
        assert!(r.entries.is_empty());
    }
}
