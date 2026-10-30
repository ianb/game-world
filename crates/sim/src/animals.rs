//! Animals: flocks declared in content, each animal a position in the state. They wander
//! inside their flock's area on their own clock, and chickens roost in their coop at night
//! and lay there in the morning. Animals have no minds; what they mean to people is each
//! character's business (`Trigger::Animals`).
use crate::ids::{ObjectId, PlaceId, VillagerId};
use content::{AnimalKind, Pos, Rect};
use serde::{Deserialize, Serialize};

/// One animal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Animal {
    /// Index into the sim's flocks.
    pub flock: u16,
    pub kind: AnimalKind,
    pub place: PlaceId,
    pub pos: Pos,
    /// In the coop for the night, out of sight.
    pub roosting: bool,
}

/// A flock with its references resolved. Derived from content, not part of the state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flock {
    pub id: String,
    pub kind: AnimalKind,
    pub owner: VillagerId,
    pub place: PlaceId,
    pub area: Rect,
    pub coop: Option<ObjectId>,
}

/// When an animal kind is up and about, as seconds into the day.
#[must_use]
pub fn waking_hours(kind: AnimalKind) -> (u32, u32) {
    match kind {
        AnimalKind::Chicken => (6 * 3600, 19 * 3600 + 30 * 60),
        AnimalKind::Sheep => (5 * 3600 + 30 * 60, 21 * 3600),
    }
}

/// Seconds between one step and the next, as a range to draw from.
#[must_use]
pub fn step_gap(kind: AnimalKind) -> (u32, u32) {
    match kind {
        AnimalKind::Chicken => (15, 90),
        AnimalKind::Sheep => (60, 240),
    }
}

/// The chance label for a hen laying when she comes out in the morning.
pub const LAYS: &str = "hen-lays";
/// Chance in a hundred that a hen lays.
pub const LAY_CHANCE: i16 = 60;
/// Hours before the same villager notices the same flock again.
pub const NOTICE_HOURS: u32 = 4;
