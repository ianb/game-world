//! Internal scheduled events. A closed enum so it can derive `Ord`, `Serialize` and enter the
//! state hash. Never boxed closures.
use crate::ids::{ObjectId, VillagerId};
use crate::input::Input;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Event {
    /// Midnight. Schedules are resolved for the new day and the next rollover is queued.
    DayRollover,
    /// A planned command's departure time. Stale if `generation` no longer matches.
    WakeUp { who: VillagerId, generation: u32 },
    /// A villager steps to the next tile of its path. Stale if `generation` no longer matches.
    Arrive { who: VillagerId, generation: u32 },
    /// A `Wait` elapsed. Stale if `generation` no longer matches.
    Waited { who: VillagerId, generation: u32 },
    /// A conversation ends for `who`. Stale unless `who.talk_until` is exactly this tick.
    TalkEnd {
        who: VillagerId,
        other: VillagerId,
        topic: crate::protocol::Topic,
    },
    /// An object's occupant released it.
    ObjectFree { object: ObjectId },
    /// An hour of `who`'s body: needs rise, sleep repays.
    Body { who: VillagerId },
    /// `who` just stopped sleeping: needs that got pressing overnight ask now.
    Woke { who: VillagerId },
    /// The next turn of a conversation.
    Beat { convo: u32 },
    /// `who` takes in their surroundings; repeats every quarter hour.
    Look { who: VillagerId },
    /// `who` tripped on the tile they just stepped onto.
    Tripped { who: VillagerId },
    /// A player or external actor input, applied at its tick in `(tick, seq)` order.
    Input(Input),
}
