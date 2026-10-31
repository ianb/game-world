//! Substances in the running sim: batches made as goods are stocked, followed through
//! whoever carries them, taken with `Command::Consume`, and an hourly craving that tells
//! people when they want a dose.
use super::Sim;
use crate::brain::{CharacterSet, Trigger};
use crate::ids::{ObjectId, VillagerId};
use crate::moods::{Push, Source};
use crate::protocol::{Emitted, MoodKind, Odd};
use crate::substances::{BAD_BATCH, Batches, Habit, Intake, ODD_FOR_THE_BETTER, ODD_PORTION};
use content::{Content, MoodName};
use std::collections::BTreeMap;

/// The mood a substance names, as the sim knows it.
#[deny(clippy::wildcard_enum_match_arm)]
fn kind(m: MoodName) -> MoodKind {
    match m {
        MoodName::Lonely => MoodKind::Lonely,
        MoodName::Restless => MoodKind::Restless,
        MoodName::Irritable => MoodKind::Irritable,
        MoodName::Anxious => MoodKind::Anxious,
        MoodName::Gloomy => MoodKind::Gloomy,
        MoodName::Cheerful => MoodKind::Cheerful,
        MoodName::Proud => MoodKind::Proud,
    }
}

/// Everyone's habits as content starts them.
pub(super) fn starting_intake(content: &Content) -> Vec<Intake> {
    content
        .villagers
        .iter()
        .map(|v| Intake {
            doses: Vec::new(),
            habits: v
                .habits
                .iter()
                .filter_map(|h| {
                    Some(Habit {
                        substance: u16::try_from(content.substance_index(&h.substance)?).ok()?,
                        hooked: h.hooked,
                        craving: 0,
                        clean_hours: 0,
                    })
                })
                .collect(),
        })
        .collect()
}

/// A batch for every substance on an object when the world starts, made by nobody.
pub(super) fn starting_batches(
    content: &Content,
    stock: &BTreeMap<u16, BTreeMap<String, u16>>,
) -> Batches {
    let mut b = Batches::default();
    for (object, goods) in stock {
        for (good, n) in goods {
            if let Some(s) = content.substance_index(good) {
                let id = b.make(
                    u16::try_from(s).expect("few substances"),
                    crate::time::Tick::ZERO,
                    None,
                    false,
                );
                b.put(*object, good, id, *n);
            }
        }
    }
    b
}

impl<C: CharacterSet> Sim<C> {
    /// Keeps the batches in step with a stock change that has been applied: portions taken
    /// off go into `who`'s hands; portions put on come out of their hands first, and any
    /// more are a new batch of their making, which may be bad.
    pub(super) fn batches_follow(
        &mut self,
        who: VillagerId,
        object: ObjectId,
        good: &str,
        delta: i32,
    ) {
        let Some(s) = self.content.substance_index(good) else {
            return;
        };
        let n = u16::try_from(delta.unsigned_abs()).unwrap_or(u16::MAX);
        if delta < 0 {
            self.state.batches.take_off(object.0, good, n, who);
            return;
        }
        let mut left = n;
        for (batch, k) in self.state.batches.from_hands(who, good, n) {
            self.state.batches.put(object.0, good, batch, k);
            left -= k;
        }
        if left == 0 {
            return;
        }
        let bad_chance = self.content.substances[s]
            .wild
            .as_ref()
            .map_or(0, |w| i16::from(w.bad_batch));
        let bad = bad_chance > 0
            && self.state.odds.chance(
                &mut self.state.rng.villagers[who.index()].body,
                who,
                BAD_BATCH,
                bad_chance,
            );
        let tick = self.state.tick;
        let id = self.state.batches.make(
            u16::try_from(s).expect("few substances"),
            tick,
            Some(who),
            bad,
        );
        self.state.batches.put(object.0, good, id, left);
    }

    /// `who` takes one `good` from `object`: a meal, a chew, a drink, a breath. Nothing
    /// happens if there is none. A declared substance has its effect.
    pub(super) fn consume(
        &mut self,
        who: VillagerId,
        object: ObjectId,
        good: &str,
        out: &mut Vec<Emitted>,
    ) {
        if object.index() >= self.world.objects.len() {
            return;
        }
        let have = self
            .state
            .stock
            .get(&object.0)
            .and_then(|g| g.get(good))
            .copied()
            .unwrap_or(0);
        if have == 0 {
            return;
        }
        self.stock(who, object, good, -1, out);
        let batch = self
            .state
            .batches
            .from_hands(who, good, 1)
            .first()
            .map(|(b, _)| *b);
        let tick = self.state.tick;
        let Some(s) = self.content.substance_index(good) else {
            out.push(Emitted::Consumed {
                tick,
                who,
                object,
                good: good.to_string(),
                batch,
                lift: 0,
                odd: Odd::No,
                cause: self.cause.clone(),
            });
            return;
        };
        let def = self.content.substances[s].clone();
        let mut lift = def.lift;
        let mut hours = def.hours;
        let mut odd = Odd::No;
        if let Some(w) = &def.wild {
            let bad = batch.is_some_and(|b| self.state.batches.made[b as usize].bad);
            let rng = &mut self.state.rng.villagers[who.index()].body;
            if bad {
                odd = Odd::BadBatch;
            } else if w.portion > 0
                && self
                    .state
                    .odds
                    .chance(rng, who, ODD_PORTION, i16::from(w.portion))
            {
                odd = if self.state.odds.chance(rng, who, ODD_FOR_THE_BETTER, 50) {
                    Odd::Better
                } else {
                    Odd::Worse
                };
            }
            match odd {
                Odd::Better => lift = lift.saturating_add(w.lift),
                Odd::Worse | Odd::BadBatch => lift = lift.saturating_sub(w.lift),
                Odd::No => {}
            }
            if odd != Odd::No {
                hours = hours.max(w.hours);
            }
        }
        let index = u16::try_from(s).expect("few substances");
        self.state.intake[who.index()].take(&def, index, batch, lift, hours);
        let needs = &mut self.state.villagers[who.index()].needs;
        needs.sleep = needs.sleep.saturating_sub(def.wakes);
        let label = format!("{} ({})", def.id, odd_words(odd));
        for m in &def.moods {
            let push = Push::by(kind(m.mood), m.points, Source::Took, label.clone());
            self.push_mood(who, &push, out);
        }
        out.push(Emitted::Consumed {
            tick,
            who,
            object,
            good: good.to_string(),
            batch,
            lift,
            odd,
            cause: self.cause.clone(),
        });
    }

    /// An hour of someone's habits: doses wear off, cravings grow, and a craving that
    /// reaches a new level is said, and offered to them when they are awake.
    pub(super) fn substances_hour(
        &mut self,
        who: VillagerId,
        asleep: bool,
        out: &mut Vec<Emitted>,
    ) {
        let crossed = self.state.intake[who.index()].hour(!asleep);
        let tick = self.state.tick;
        for (s, level) in crossed {
            let good = self.content.substances[usize::from(s)].id.clone();
            out.push(Emitted::Craved {
                tick,
                who,
                good: good.clone(),
                level,
            });
            if !asleep {
                self.react(who, Trigger::Craving { good, level }, out);
            }
        }
    }
}

fn odd_words(odd: Odd) -> &'static str {
    match odd {
        Odd::No => "as usual",
        Odd::Better => "stronger than usual",
        Odd::Worse => "off",
        Odd::BadBatch => "a bad batch",
    }
}
