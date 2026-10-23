//! Per-villager simulation state: where they are, where they are going, what they are doing.
use crate::brain::Command;
use crate::ids::PlaceId;
use crate::time::Tick;
use content::Pos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A planned walk: legs across places, the current leg's remaining tiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Movement {
    /// Remaining legs after the current one: (place, to, door to cross after).
    pub legs: Vec<(PlaceId, Pos, Option<u16>)>,
    /// Tiles still to step through in the current leg, in order.
    pub path: Vec<Pos>,
    /// Door to cross when the current leg ends.
    pub door_after: Option<u16>,
    /// What to do on arrival.
    pub then: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct VillagerState {
    pub id: String,
    pub name: String,
    pub place: PlaceId,
    pub pos: Pos,
    /// Bumped whenever the current movement or wait changes; stale `Arrive`/`Waited` events
    /// carry an old value and are dropped.
    pub generation: u32,
    /// Bumped whenever the day plan is replaced; stale `WakeUp` events are dropped.
    pub plan_generation: u32,
    pub activity: String,
    pub moving: Option<Movement>,
    /// Today's remaining planned commands, by tick.
    pub plan: BTreeMap<Tick, Command>,
    /// Tick of the last change of activity, for views.
    pub since: Tick,
    /// Activity to return to when a conversation ends.
    pub resume: Option<String>,
}
