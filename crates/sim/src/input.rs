//! Inputs are the only way anything outside the sim affects it. Each carries provenance and
//! is applied at the tick its owner assigned.
use crate::ids::VillagerId;
use crate::time::Tick;
use serde::{Deserialize, Serialize};

/// Who caused an input. Players and external actors (a villager brain backed by a model, a
/// drama director) go through the same door with different permissions.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Source {
    System,
    Player(u16),
    Actor(String),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum InputKind {
    /// Does nothing. Useful for tests and as a heartbeat in replays.
    Noop,
    /// A human-readable marker that shows up in traces.
    Marker(String),
    /// Debug: move a villager instantly. Rejected for non-system sources.
    Teleport { who: VillagerId, x: u16, y: u16 },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Input {
    pub tick: Tick,
    pub source: Source,
    pub kind: InputKind,
}
