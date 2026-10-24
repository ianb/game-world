//! Per-villager simulation state: where they are, where they are going, what they are doing.
use crate::brain::Command;
use crate::brain::Target;
use crate::ids::PlaceId;
use crate::needs::Needs;
use crate::protocol::Cause;
use crate::time::Tick;
use content::Pos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A door crossing at the end of a leg: door index, and whether the crossing goes from the
/// door's `to` end back to its `from` end.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Crossing {
    pub door: u16,
    pub reverse: bool,
}

/// One leg of a planned walk still to come.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PlannedLeg {
    pub place: PlaceId,
    pub to: Pos,
    pub crossing: Option<Crossing>,
}

/// A planned walk: legs across places, the current leg's remaining tiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Movement {
    /// Remaining legs after the current one.
    pub legs: Vec<PlannedLeg>,
    /// Tiles still to step through in the current leg, in order.
    pub path: Vec<Pos>,
    /// Door to cross when the current leg ends.
    pub door_after: Option<Crossing>,
    /// What to do on arrival.
    pub then: String,
    /// True when this is the walk back from an errand: arrival resumes rather than starts.
    pub resuming: bool,
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
    /// Today's remaining planned commands, by tick, in the order they were planned, each
    /// with why it is in the plan.
    pub plan: BTreeMap<Tick, Vec<(Cause, Command)>>,
    /// Tick of the last change of activity, for views.
    pub since: Tick,
    /// Activity to return to when a conversation ends.
    pub resume: Option<String>,
    /// When the current conversation ends, if talking.
    pub talk_until: Option<Tick>,
    /// When the current `Wait` elapses, if waiting. A conversation defers it.
    pub wait_until: Option<Tick>,
    pub coins: u32,
    pub needs: Needs,
    /// A short errand (a drink, the privy) in the middle of something else: where to go
    /// back to when it is done.
    pub interlude: Option<Interlude>,
    /// Ticks of an interrupted wait to re-arm once the walk back ends.
    pub resume_wait: Option<u32>,
    /// Why the current intention (walk, wait, activity) was started.
    pub cause: Cause,
    /// The cause to restore when the walk back from an errand ends.
    pub resume_cause: Option<Cause>,
}

/// An errand that returns the villager to where they were.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interlude {
    pub activity: String,
    pub minutes: u32,
    pub back_place: PlaceId,
    pub back_pos: Pos,
    pub back_activity: String,
    /// Ticks left on a `Wait` that the errand interrupted, re-armed on return.
    pub back_wait: Option<u32>,
    pub back_cause: Cause,
    /// Further errands asked for while this one was under way, done in order before
    /// going back.
    pub queued: Vec<Errand>,
}

/// An errand waiting its turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Errand {
    pub target: Target,
    pub activity: String,
    pub minutes: u32,
    pub cause: Cause,
}
