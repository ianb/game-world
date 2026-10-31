//! Moods: a category of named states, not one number. Each villager is prone to a few
//! moods from this library, each with sources of their own (`content::MoodSpec`). A mood
//! is a count of points on a fixed scale, read as four named levels, with what last moved
//! it. Spirits, the overall reading conversations use, adds the good levels, takes away
//! the bad ones and the body's discomfort. Design: `docs/design/components.md`.
use crate::ids::VillagerId;
use crate::protocol::MoodKind;
use crate::time::Tick;
use content::MoodSpec;
use serde::{Deserialize, Serialize};

/// Points to a level; a mood runs from 0 to `MAX_POINTS`, levels 0 to `MAX_LEVEL`.
pub const POINTS_PER_LEVEL: u8 = 4;
pub const MAX_LEVEL: u8 = 3;
pub const MAX_POINTS: u8 = POINTS_PER_LEVEL * MAX_LEVEL;
/// The level at which a mood is strong enough to act on: the villager is told.
pub const ACTS_AT: u8 = 2;

/// What last moved a mood, for behaviour to read; the text beside it is for people.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Source {
    Alone,
    Idle,
    Noise(VillagerId),
    Crowds,
    Slight(VillagerId),
    Stuck,
    Money,
    Dark,
    Stranger(VillagerId),
    Garden,
    Rain,
    FineDay,
    Company(VillagerId),
    GoodThing,
    Harvest,
    Earning,
    Work,
    Time,
}

/// One mood someone is prone to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mood {
    pub kind: MoodKind,
    pub points: u8,
    pub source: Source,
    pub cause: String,
    pub since: Tick,
    /// When the villager was last told of it (`Trigger::Mood`).
    pub offered: Tick,
}

impl Mood {
    #[must_use]
    pub fn level(&self) -> u8 {
        (self.points / POINTS_PER_LEVEL).min(MAX_LEVEL)
    }
}

/// Everything about one villager's moods: the moods themselves, in content order, and
/// what the hourly rules need to remember.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moods {
    pub moods: Vec<Mood>,
    /// When they last had company: a conversation started or ended.
    pub last_company: Tick,
    /// Idle hours in a row.
    pub idle_hours: u8,
    /// Waking hours in a row without company.
    pub alone_hours: u8,
    /// Hours counted, so slow moods can fade every other hour.
    pub hours: u32,
}

impl Moods {
    #[must_use]
    pub fn new(specs: &[MoodSpec]) -> Moods {
        Moods {
            moods: specs
                .iter()
                .map(|s| Mood {
                    kind: kind_of(s),
                    points: 0,
                    source: Source::Time,
                    cause: String::new(),
                    since: Tick::ZERO,
                    offered: Tick::ZERO,
                })
                .collect(),
            ..Moods::default()
        }
    }

    #[must_use]
    pub fn get(&self, kind: MoodKind) -> Option<&Mood> {
        self.moods.iter().find(|m| m.kind == kind)
    }

    /// The level of a mood they have, 0 for one they are not prone to.
    #[must_use]
    pub fn level(&self, kind: MoodKind) -> u8 {
        self.get(kind).map_or(0, Mood::level)
    }
}

#[must_use]
pub fn kind_of(spec: &MoodSpec) -> MoodKind {
    match spec {
        MoodSpec::Lonely { .. } => MoodKind::Lonely,
        MoodSpec::Restless { .. } => MoodKind::Restless,
        MoodSpec::Irritable { .. } => MoodKind::Irritable,
        MoodSpec::Anxious { .. } => MoodKind::Anxious,
        MoodSpec::Gloomy { .. } => MoodKind::Gloomy,
        MoodSpec::Cheerful { .. } => MoodKind::Cheerful,
        MoodSpec::Proud { .. } => MoodKind::Proud,
    }
}

/// Whether a mood feels good.
#[must_use]
#[deny(clippy::wildcard_enum_match_arm)]
pub fn good(kind: MoodKind) -> bool {
    match kind {
        MoodKind::Cheerful | MoodKind::Proud => true,
        MoodKind::Lonely
        | MoodKind::Restless
        | MoodKind::Irritable
        | MoodKind::Anxious
        | MoodKind::Gloomy => false,
    }
}

/// The names of a mood's levels, 1 to 3. Placeholders, for Ian to rename.
#[must_use]
#[deny(clippy::wildcard_enum_match_arm)]
pub fn level_names(kind: MoodKind) -> [&'static str; 3] {
    match kind {
        MoodKind::Lonely => ["missing company", "lonely", "desolate"],
        MoodKind::Restless => ["fidgety", "restless", "stir-crazy"],
        MoodKind::Irritable => ["ruffled", "irritable", "furious"],
        MoodKind::Anxious => ["uneasy", "anxious", "frightened"],
        MoodKind::Gloomy => ["flat", "gloomy", "low"],
        MoodKind::Cheerful => ["pleased", "cheerful", "elated"],
        MoodKind::Proud => ["satisfied", "proud", "puffed up"],
    }
}

/// The name of a mood at a level, or "" at 0.
#[must_use]
pub fn level_name(kind: MoodKind, level: u8) -> &'static str {
    match level {
        0 => "",
        l => level_names(kind)[usize::from(l.min(MAX_LEVEL) - 1)],
    }
}

/// Spirits, -3 to 3: the good moods' levels, less the bad ones', plus `body` (0, -1 or
/// -2 from pressing needs).
#[must_use]
pub fn spirits(moods: &Moods, body: i8) -> i8 {
    let sum: i32 = moods
        .moods
        .iter()
        .map(|m| {
            let l = i32::from(m.level());
            if good(m.kind) { l } else { -l }
        })
        .sum();
    i8::try_from((sum + i32::from(body)).clamp(-3, 3)).unwrap_or(0)
}

/// The name of a spirits reading. Placeholders, for Ian to rename.
#[must_use]
pub fn spirits_name(spirits: i8) -> &'static str {
    match spirits {
        i8::MIN..=-3 => "wretched",
        -2 => "low",
        -1 => "out of sorts",
        0 => "even",
        1 => "content",
        2 => "in good spirits",
        _ => "bright",
    }
}

/// A change of points to one mood, with what caused it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Push {
    pub kind: MoodKind,
    /// Added (or taken, if negative) points.
    pub by: i8,
    /// Instead of adding, raise the mood to at least this many points.
    pub floor: Option<u8>,
    pub source: Source,
    pub cause: String,
}

impl Push {
    #[must_use]
    pub fn by(kind: MoodKind, by: i8, source: Source, cause: impl Into<String>) -> Push {
        Push {
            kind,
            by,
            floor: None,
            source,
            cause: cause.into(),
        }
    }
}

/// Applies a push to the mood it names, if they have it. Returns the level before and
/// after when the level changed.
pub fn apply(moods: &mut Moods, push: &Push, now: Tick) -> Option<(MoodKind, u8, u8)> {
    let m = moods.moods.iter_mut().find(|m| m.kind == push.kind)?;
    let was = m.level();
    let mut points = i16::from(m.points) + i16::from(push.by);
    if let Some(floor) = push.floor {
        points = points.max(i16::from(floor));
    }
    let points = u8::try_from(points.clamp(0, i16::from(MAX_POINTS))).unwrap_or(0);
    // A floor that holds the mood where it is becomes its cause.
    if push.floor.is_some_and(|f| f >= m.points) {
        m.source = push.source;
        m.cause.clone_from(&push.cause);
    }
    if points == m.points {
        return None;
    }
    // A rise names its cause; a fall keeps the cause of what is left.
    if points > m.points {
        m.source = push.source;
        m.cause.clone_from(&push.cause);
    }
    m.points = points;
    let now_level = m.level();
    if now_level == was {
        None
    } else {
        m.since = now;
        Some((m.kind, was, now_level))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::{Delight, Irritant};

    fn moods() -> Moods {
        Moods::new(&[
            MoodSpec::Lonely { after_hours: 4 },
            MoodSpec::Irritable {
                by: vec![Irritant::Noise],
            },
            MoodSpec::Cheerful {
                by: vec![Delight::Rain],
            },
        ])
    }

    #[test]
    fn points_read_as_named_levels() {
        let mut m = moods();
        let push = Push::by(MoodKind::Lonely, 9, Source::Alone, "no company");
        assert_eq!(
            apply(&mut m, &push, Tick::ZERO),
            Some((MoodKind::Lonely, 0, 2))
        );
        assert_eq!(m.level(MoodKind::Lonely), 2);
        assert_eq!(level_name(MoodKind::Lonely, 2), "lonely");
        assert_eq!(m.level(MoodKind::Proud), 0, "not prone to it");
        assert_eq!(
            apply(
                &mut m,
                &Push::by(MoodKind::Proud, 8, Source::Work, "x"),
                Tick::ZERO
            ),
            None
        );
    }

    #[test]
    fn spirits_weigh_good_against_bad_and_the_body() {
        let mut m = moods();
        apply(
            &mut m,
            &Push::by(MoodKind::Cheerful, 12, Source::Rain, "rain"),
            Tick::ZERO,
        );
        assert_eq!(spirits(&m, 0), 3);
        apply(
            &mut m,
            &Push::by(MoodKind::Irritable, 8, Source::Crowds, "x"),
            Tick::ZERO,
        );
        assert_eq!(spirits(&m, 0), 1);
        assert_eq!(spirits(&m, -2), -1);
        assert_eq!(spirits_name(-1), "out of sorts");
    }

    #[test]
    fn a_floor_raises_but_never_lowers() {
        let mut m = Moods::new(&[MoodSpec::Anxious {
            about: vec![content::Worry::Money(30)],
        }]);
        let floor = |f| Push {
            kind: MoodKind::Anxious,
            by: 0,
            floor: Some(f),
            source: Source::Money,
            cause: "money".into(),
        };
        apply(&mut m, &floor(8), Tick::ZERO);
        assert_eq!(m.level(MoodKind::Anxious), 2);
        apply(&mut m, &floor(4), Tick::ZERO);
        assert_eq!(m.level(MoodKind::Anxious), 2);
    }
}
