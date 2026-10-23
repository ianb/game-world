//! Inputs are the only way anything outside the sim affects it. Each carries provenance and
//! is applied at the tick its owner assigned.
use crate::ids::{PlaceId, VillagerId};
use crate::time::Tick;
use serde::{Deserialize, Serialize};

/// Who caused an input. Players and external actors (a villager brain backed by a model, a
/// drama director) go through the same door with different permissions.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Source {
    System,
    Player(u16),
    Actor(String),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum InputKind {
    /// Does nothing. Useful for tests and as a heartbeat in replays.
    Noop,
    /// A human-readable marker that shows up in traces.
    Marker(String),
    /// Debug: move a villager instantly to a passable tile. Rejected for non-system sources
    /// and for impassable targets.
    Teleport {
        who: VillagerId,
        place: PlaceId,
        x: u16,
        y: u16,
    },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct Input {
    pub tick: Tick,
    pub source: Source,
    pub kind: InputKind,
    /// Assigned by the sim when the input is accepted; 0 when submitted. Orders inputs
    /// for the same tick and makes the log replayable.
    #[serde(default)]
    pub seq: u32,
}
