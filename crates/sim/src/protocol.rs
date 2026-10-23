//! Boundary types: what crosses from the sim to the viewer and back. Integer-only, no maps,
//! no `u64`/`usize`, no `serde(with)`. Bump `PROTOCOL_VERSION` on any change.
use crate::ids::{PlaceId, VillagerId};
use crate::time::{Calendar, Tick};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 2;

/// Events the sim emits for viewers and external actors.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Emitted {
    DayStarted {
        tick: Tick,
        calendar: Calendar,
    },
    InputApplied {
        tick: Tick,
        summary: String,
    },
    InputRejected {
        tick: Tick,
        reason: String,
    },
    /// A villager stepped to a tile (or appeared in a new place through a door).
    Moved {
        tick: Tick,
        who: VillagerId,
        place: PlaceId,
        x: u16,
        y: u16,
    },
    /// A villager set out for somewhere.
    Departed {
        tick: Tick,
        who: VillagerId,
        activity: String,
        eta: Tick,
    },
    /// A villager started doing something where it stands.
    Activity {
        tick: Tick,
        who: VillagerId,
        activity: String,
    },
    /// A villager could not get where it wanted.
    Stuck {
        tick: Tick,
        who: VillagerId,
        reason: String,
    },
}

impl Emitted {
    #[must_use]
    pub fn tick(&self) -> Tick {
        match self {
            Emitted::DayStarted { tick, .. }
            | Emitted::InputApplied { tick, .. }
            | Emitted::InputRejected { tick, .. }
            | Emitted::Moved { tick, .. }
            | Emitted::Departed { tick, .. }
            | Emitted::Activity { tick, .. }
            | Emitted::Stuck { tick, .. } => *tick,
        }
    }

    #[must_use]
    pub fn who(&self) -> Option<VillagerId> {
        match self {
            Emitted::Moved { who, .. }
            | Emitted::Departed { who, .. }
            | Emitted::Activity { who, .. }
            | Emitted::Stuck { who, .. } => Some(*who),
            _ => None,
        }
    }
}
