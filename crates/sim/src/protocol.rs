//! Boundary types: what crosses from the sim to the viewer and back. Integer-only, no maps,
//! no `u64`/`usize`, no `serde(with)`. Bump `PROTOCOL_VERSION` on any change.
use crate::ids::VillagerId;
use crate::time::{Calendar, Tick};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

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
    Moved {
        tick: Tick,
        who: VillagerId,
        x: u16,
        y: u16,
    },
}

impl Emitted {
    #[must_use]
    pub fn tick(&self) -> Tick {
        match self {
            Emitted::DayStarted { tick, .. }
            | Emitted::InputApplied { tick, .. }
            | Emitted::InputRejected { tick, .. }
            | Emitted::Moved { tick, .. } => *tick,
        }
    }
}
