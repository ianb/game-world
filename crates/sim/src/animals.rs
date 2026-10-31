//! Animals: flocks declared in content, each animal a position in the state. They wander
//! inside their flock's area on their own clock, and chickens roost in their coop at night
//! and lay there in the morning. Animals have no minds; what they mean to people is each
//! character's business (`Trigger::Animals`).
use crate::ids::{AnimalKindId, ObjectId, PlaceId, VillagerId};
use content::{Pos, Rect};
use serde::{Deserialize, Serialize};

/// One animal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Animal {
    /// Index into the sim's flocks.
    pub flock: u16,
    pub kind: AnimalKindId,
    pub place: PlaceId,
    pub pos: Pos,
    /// In the coop for the night, out of sight.
    pub roosting: bool,
}

/// A flock with its references resolved. Derived from content, not part of the state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flock {
    pub id: String,
    pub kind: AnimalKindId,
    pub owner: VillagerId,
    pub place: PlaceId,
    pub area: Rect,
    pub coop: Option<ObjectId>,
    /// From the kind's declaration: when they are up and about, as seconds into the day.
    pub wakes: u32,
    pub sleeps: u32,
    /// Seconds between one step and the next, as a range to draw from.
    pub step: (u32, u32),
    /// The good one may lay in the coop in the morning.
    pub lays: Option<String>,
}

/// The chance label for a hen laying when she comes out in the morning.
pub const LAYS: &str = "hen-lays";
/// Chance in a hundred that a hen lays.
pub const LAY_CHANCE: i16 = 60;
/// Hours before the same villager notices the same flock again.
pub const NOTICE_HOURS: u32 = 4;
