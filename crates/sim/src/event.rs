//! Internal scheduled events. A closed enum so it can derive `Ord`, `Serialize` and enter the
//! state hash. Never boxed closures.
use crate::ids::{ObjectId, VillagerId};
use crate::input::Input;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Event {
    /// Midnight. Schedules are resolved for the new day and the next rollover is queued.
    DayRollover,
    /// A villager's brain asked to be woken. Stale if `generation` no longer matches.
    WakeUp { who: VillagerId, generation: u32 },
    /// A villager reaches the next tile of its path. Stale if `generation` no longer matches.
    Arrive { who: VillagerId, generation: u32 },
    /// An object's occupant released it.
    ObjectFree { object: ObjectId },
    /// A player or external actor input, applied at its tick in `(tick, seq)` order.
    Input(Input),
}
