//! Boundary types: what crosses from the sim to the viewer and back. Integer-only, no maps,
//! no `u64`/`usize`, no `serde(with)`. Bump `PROTOCOL_VERSION` on any change.
use crate::brain::CharacterSet;
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::sim::Sim;
use crate::time::{Calendar, Tick, TimeOfDay};
use content::{Cell, PlaceKind, Pos, Rect};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 13;

/// Events the sim emits for viewers and external actors.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
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
        cause: Cause,
    },
    /// A villager started doing something where it stands.
    Activity {
        tick: Tick,
        who: VillagerId,
        activity: String,
        cause: Cause,
    },
    /// A villager could not get where it wanted.
    Stuck {
        tick: Tick,
        who: VillagerId,
        reason: String,
        cause: Cause,
    },
    /// The day's plan for `who`, made at midnight. `dropped` lists schedule items the
    /// character's code left out.
    Planned {
        tick: Tick,
        who: VillagerId,
        items: Vec<PlanItem>,
        dropped: Vec<PlanItem>,
    },
    /// A note character code left about its own reasons.
    Noted {
        tick: Tick,
        who: VillagerId,
        text: String,
        cause: Cause,
    },
    /// A character's brain made a choice: the day plan, a reaction, or a note it left.
    Decided {
        tick: Tick,
        who: VillagerId,
        text: String,
    },
    /// `who` greeted or started talking with `with`. `minutes` is 0 for a greeting.
    Talked {
        tick: Tick,
        who: VillagerId,
        with: VillagerId,
        topic: String,
        minutes: u32,
        cause: Cause,
    },
    /// `who` cleared growth on `tiles` tiles around (x, y) in `place`.
    Cleared {
        tick: Tick,
        who: VillagerId,
        place: PlaceId,
        x: u16,
        y: u16,
        tiles: u16,
        cause: Cause,
    },
    /// Money moved. `from` or `to` is None for the world outside the town. `amount` is
    /// what actually moved, which is less than asked when the payer was short.
    Paid {
        tick: Tick,
        who: VillagerId,
        from: Option<VillagerId>,
        to: Option<VillagerId>,
        amount: u32,
        what: String,
        /// What the payer has left afterwards; 0 for the world outside.
        payer_left: u32,
        cause: Cause,
    },
    /// Goods on an object changed by `delta`; `now` is the count after.
    Stocked {
        tick: Tick,
        who: VillagerId,
        object: ObjectId,
        good: String,
        delta: i32,
        now: u16,
        cause: Cause,
    },
    /// A need of `who` got pressing or urgent.
    Need {
        tick: Tick,
        who: VillagerId,
        need: Need,
        level: u8,
    },
    /// `who` started a conversation with `with` in `frame`.
    ConversationStarted {
        tick: Tick,
        convo: u32,
        who: VillagerId,
        with: VillagerId,
        place: PlaceId,
        frame: Frame,
        topic: String,
        cause: Cause,
    },
    /// One turn. `meant` is what the speaker's program chose, `did` what came out.
    /// `breach` is true when `did` steps outside the frame.
    Beat {
        tick: Tick,
        convo: u32,
        who: VillagerId,
        frame: Frame,
        stance: Stance,
        meant: Act,
        did: Act,
        breach: bool,
    },
    /// How `who` took `speaker`'s turn. `meant` and `did` are repeated from the beat for
    /// readers of the log; the hearer only ever saw `did`.
    TookUp {
        tick: Tick,
        convo: u32,
        who: VillagerId,
        speaker: VillagerId,
        meant: MoveKind,
        did: MoveKind,
        taken_as: MoveKind,
        warmth: i8,
        attribution: Attribution,
        overheard: bool,
    },
    FrameChanged {
        tick: Tick,
        convo: u32,
        who: VillagerId,
        from: Frame,
        to: Frame,
    },
    /// How it went for each: the most intense moment and the last, from -3 to 3.
    ConversationEnded {
        tick: Tick,
        convo: u32,
        who: VillagerId,
        with: VillagerId,
        frame: Frame,
        my_peak: i8,
        my_end: i8,
        their_peak: i8,
        their_end: i8,
        ended_by: Option<VillagerId>,
    },
}

/// The kind of exchange two people have settled on. Shared conventions: which moves make
/// sense, what they mean, and which frames you can bid into from here.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Frame {
    Greeting,
    Transaction,
    SmallTalk,
    Confiding,
}

/// Something a person does in a conversation. Some kinds only ever come out of a misfire or
/// a reading (`Jab`, `Awkward`, `Rebuff`, `Pry`): nobody means to do them.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum MoveKind {
    Ask,
    Tell,
    Agree,
    Joke,
    Tease,
    Compliment,
    Complain,
    Request,
    Offer,
    Thank,
    Sympathise,
    Disclose,
    Decline,
    Invite,
    Leave,
    Jab,
    Awkward,
    Rebuff,
    Pry,
}

/// An answer to a bid to change frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum BidAnswer {
    /// Engage: the frame shifts.
    Toward,
    /// Deflect; `account` is whether they gave a reason.
    Away { account: bool },
    /// Rebuff.
    Against,
}

/// One turn's content: a move, a bid for another frame, or an answer to a bid.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Act {
    Move { kind: MoveKind, subject: String },
    Bid { frame: Frame, subject: String },
    Answer { answer: BidAnswer },
}

/// How someone comes into a turn (Goffman's footing).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Stance {
    Open,
    Guarded,
    Anxious,
    Cold,
}

/// Whether a hearer put a move down to the speaker's character or to circumstances.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Attribution {
    Character,
    Situation,
}

/// What a summary line is about.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum SummaryKind {
    /// Character code changed the day's plan from the schedule.
    PlanChange,
    /// A planned activity never started.
    Skipped,
    /// Something done that was not in the day's plan.
    Unplanned,
    /// A conversation.
    Talk,
    /// A villager's note about their own reasons.
    Note,
    /// Money in and out for one villager.
    Money,
    /// Goods ran out.
    Shortage,
    /// A need went urgent.
    Need,
    /// Errands of the body: privy, well, extra meals.
    Body,
    /// Couldn't get somewhere.
    Stuck,
    /// Something that usually happens did not.
    Absent,
    /// Everything else worth a count: greetings, weeding.
    Other,
}

/// One line of a day summary, pointing at the moment it happened so it can be replayed.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct SummaryItem {
    pub tick: Tick,
    pub who: Option<VillagerId>,
    pub kind: SummaryKind,
    pub text: String,
    pub cause: Option<Cause>,
    /// On how many of the previous seven days the same thing happened.
    pub seen_days: u8,
}

/// What happened on one day: deviations from routine first, routine after.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct DaySummary {
    pub day: u32,
    pub calendar: Calendar,
    pub news: Vec<SummaryItem>,
    pub routine: Vec<SummaryItem>,
}

/// Why something happened, carried on the events whose reason cannot be recovered later.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
// Externally tagged (serde's default): it is stored in the snapshot, and postcard cannot
// decode internally tagged enums.
pub enum Cause {
    /// A day-plan item exactly as the content schedule wrote it.
    Schedule,
    /// A day-plan item that character code added or changed.
    Plan,
    /// Character code reacting to something; `trigger` names it ("waited", "hungry (72)").
    Reaction { trigger: String },
    /// The walk back from an errand.
    Return,
    /// An outside input, by sequence number.
    Input { seq: u32 },
    /// The sim's own rules: the start of the day, someone else starting a conversation.
    #[default]
    Sim,
}

/// One item of a day plan.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct PlanItem {
    pub at: TimeOfDay,
    /// The command, summarised.
    pub text: String,
    /// The activity it starts, if it starts one.
    pub activity: Option<String>,
    /// True when it came from the content schedule unchanged.
    pub schedule: bool,
}

/// A bodily need.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Need {
    Hunger,
    Thirst,
    Sleep,
    Bladder,
}

/// Need levels, 0 (met) to 100.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct NeedsView {
    pub hunger: u8,
    pub thirst: u8,
    pub sleep: u8,
    pub bladder: u8,
}

/// The numbers behind the needs, so a viewer can explain a value without hardcoding
/// the sim's constants.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct RulesView {
    /// Needs run from 0 (met) to this.
    pub need_max: u8,
    /// A need at this level asks the character to act; at `need_urgent` it keeps asking.
    pub need_pressing: u8,
    pub need_urgent: u8,
    /// Base rise per hour awake, before the hourly roll of 0 to 5 less 2.
    pub hunger_per_hour: u8,
    pub thirst_per_hour: u8,
    pub sleep_per_hour: u8,
    pub bladder_per_hour: u8,
    /// Sleep repaid per hour asleep; the other needs rise at half speed asleep.
    pub sleep_recovery_per_hour: u8,
}

#[must_use]
pub fn rules_view() -> RulesView {
    use crate::needs;
    RulesView {
        need_max: 100,
        need_pressing: needs::PRESSING,
        need_urgent: needs::URGENT,
        hunger_per_hour: needs::RATE_HUNGER,
        thirst_per_hour: needs::RATE_THIRST,
        sleep_per_hour: needs::RATE_SLEEP,
        bladder_per_hour: needs::RATE_BLADDER,
        sleep_recovery_per_hour: needs::SLEEP_RECOVERY,
    }
}

/// Goods on one object.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct StockView {
    pub object: ObjectId,
    pub key: String,
    pub good: String,
    pub count: u16,
}

/// What an outdoor tile looks like once wear and growth are applied to its cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Surface {
    Grass,
    TallGrass,
    Brush,
    Track,
    Cowpath,
    Path,
    WeedyPath,
    OvergrownPath,
}

/// The living ground of one place, row-major; None for cells that neither wear nor grow.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct TerrainView {
    pub place: PlaceId,
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Option<Surface>>,
    /// Footfall and growth counters per tile, and the step cost (0 where impassable).
    pub wear: Vec<u8>,
    pub growth: Vec<u8>,
    pub cost: Vec<u16>,
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
            | Emitted::Stuck { tick, .. }
            | Emitted::Planned { tick, .. }
            | Emitted::Noted { tick, .. }
            | Emitted::Decided { tick, .. }
            | Emitted::Talked { tick, .. }
            | Emitted::Cleared { tick, .. }
            | Emitted::Paid { tick, .. }
            | Emitted::Stocked { tick, .. }
            | Emitted::Need { tick, .. }
            | Emitted::ConversationStarted { tick, .. }
            | Emitted::Beat { tick, .. }
            | Emitted::TookUp { tick, .. }
            | Emitted::FrameChanged { tick, .. }
            | Emitted::ConversationEnded { tick, .. } => *tick,
        }
    }

    #[must_use]
    pub fn who(&self) -> Option<VillagerId> {
        match self {
            Emitted::Moved { who, .. }
            | Emitted::Departed { who, .. }
            | Emitted::Activity { who, .. }
            | Emitted::Stuck { who, .. }
            | Emitted::Planned { who, .. }
            | Emitted::Noted { who, .. }
            | Emitted::Decided { who, .. }
            | Emitted::Talked { who, .. }
            | Emitted::Cleared { who, .. }
            | Emitted::Paid { who, .. }
            | Emitted::Stocked { who, .. }
            | Emitted::Need { who, .. }
            | Emitted::ConversationStarted { who, .. }
            | Emitted::Beat { who, .. }
            | Emitted::TookUp { who, .. }
            | Emitted::FrameChanged { who, .. }
            | Emitted::ConversationEnded { who, .. } => Some(*who),
            _ => None,
        }
    }
}

/// A villager as the viewer sees it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct VillagerView {
    pub id: VillagerId,
    pub key: String,
    pub name: String,
    pub place: PlaceId,
    pub pos: Pos,
    pub activity: String,
    pub since: Tick,
    /// Activity the villager is walking towards, if walking.
    pub heading_for: Option<String>,
    /// Next tile on the current path, for interpolation.
    pub next_tile: Option<Pos>,
    pub coins: u32,
    pub needs: NeedsView,
    /// Why they are doing what they are doing.
    pub cause: Cause,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct PlaceView {
    pub id: PlaceId,
    pub key: String,
    pub kind: PlaceKind,
    pub width: u16,
    pub height: u16,
    /// Row-major, `width * height` cells.
    pub cells: Vec<Cell>,
    pub arrival: Pos,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct ObjectView {
    pub key: String,
    pub kind: String,
    pub place: PlaceId,
    pub pos: Pos,
    pub walkable: bool,
    pub building: String,
    pub room: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct DoorView {
    pub key: String,
    pub from_place: PlaceId,
    pub from: Pos,
    pub to_place: PlaceId,
    pub to: Pos,
    pub one_way: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct RoofView {
    pub building: String,
    pub place: PlaceId,
    pub rect: Rect,
}

/// The compiled world as the viewer needs it: static for the life of a town.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct WorldView {
    pub places: Vec<PlaceView>,
    pub objects: Vec<ObjectView>,
    pub doors: Vec<DoorView>,
    pub roofs: Vec<RoofView>,
}

/// Snapshot of the clock, for the viewer's header.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct ClockView {
    pub tick: Tick,
    pub calendar: Calendar,
    pub time: TimeOfDay,
}

fn place_id(i: usize) -> PlaceId {
    PlaceId(u16::try_from(i).expect("few places"))
}

#[must_use]
pub fn world_view<C: CharacterSet>(sim: &Sim<C>) -> WorldView {
    let w = sim.world();
    WorldView {
        places: w
            .places
            .iter()
            .enumerate()
            .map(|(i, p)| PlaceView {
                id: place_id(i),
                key: p.id.clone(),
                kind: p.kind,
                width: p.width,
                height: p.height,
                cells: p.cells.clone(),
                arrival: p.arrival,
            })
            .collect(),
        objects: w
            .objects
            .iter()
            .map(|o| ObjectView {
                key: o.id.clone(),
                kind: o.kind.clone(),
                place: place_id(o.place),
                pos: o.at,
                walkable: o.walkable,
                building: o.building.clone(),
                room: o.room.clone(),
            })
            .collect(),
        doors: w
            .doors
            .iter()
            .map(|d| DoorView {
                key: d.id.clone(),
                from_place: place_id(d.from_place),
                from: d.from,
                to_place: place_id(d.to_place),
                to: d.to,
                one_way: d.one_way,
            })
            .collect(),
        roofs: w
            .roofs
            .iter()
            .map(|r| RoofView {
                building: r.building.clone(),
                place: place_id(r.place),
                rect: r.rect,
            })
            .collect(),
    }
}

#[must_use]
pub fn villager_views<C: CharacterSet>(sim: &Sim<C>) -> Vec<VillagerView> {
    sim.state()
        .villagers
        .iter()
        .enumerate()
        .map(|(i, v)| VillagerView {
            id: VillagerId(u16::try_from(i).expect("few villagers")),
            key: v.id.clone(),
            name: v.name.clone(),
            place: v.place,
            pos: v.pos,
            activity: v.activity.clone(),
            since: v.since,
            heading_for: v.moving.as_ref().map(|m| m.then.clone()),
            next_tile: v.moving.as_ref().and_then(|m| m.path.first().copied()),
            coins: v.coins,
            needs: v.needs.view(),
            cause: v.cause.clone(),
        })
        .collect()
}

/// How a villager sees someone, read from their memory.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct RegardView {
    pub with: VillagerId,
    pub familiarity: u16,
    pub feeling: i8,
    pub last_seemed: i8,
}

/// A remembered conversation.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct EpisodeView {
    pub tick: Tick,
    pub with: VillagerId,
    pub frame: Frame,
    pub peak: i8,
    pub end: i8,
    pub they_seemed: i8,
    pub notes: Vec<String>,
}

/// Something said to them or overheard, as they took it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct HeardView {
    pub tick: Tick,
    pub from: VillagerId,
    pub kind: MoveKind,
    pub subject: String,
    pub overheard: bool,
}

/// A villager's inner side, for the inspector: settled tendencies, how they are now, how
/// they see the people they have talked to, and what they recently remember.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct MindView {
    pub who: VillagerId,
    pub sociable: i8,
    pub warmth: i8,
    pub skill: u8,
    pub rejection_sensitivity: u8,
    pub charity: i8,
    pub mood: i8,
    pub worry: Option<String>,
    pub regard: Vec<RegardView>,
    /// Most recent last.
    pub episodes: Vec<EpisodeView>,
    pub heard: Vec<HeardView>,
}

/// How many recent episodes and heard things a `MindView` carries.
const MIND_RECENT: usize = 8;

/// `None` for a villager with no program of their own.
#[must_use]
pub fn mind_view<C: CharacterSet>(sim: &Sim<C>, who: VillagerId) -> Option<MindView> {
    let ctx = sim.ctx();
    let mind = sim.state().characters.interlocutor_ref(who, &ctx)?;
    let p = mind.persona();
    let memory = mind.memory();
    let regard = (0..sim.state().villagers.len())
        .map(|i| VillagerId(u16::try_from(i).expect("few villagers")))
        .filter(|&o| o != who)
        .filter_map(|o| {
            let r = memory.regard(o);
            (r.familiarity > 0).then_some(RegardView {
                with: o,
                familiarity: r.familiarity,
                feeling: r.feeling,
                last_seemed: r.last_seemed,
            })
        })
        .collect();
    let recent = |n: usize| n.saturating_sub(MIND_RECENT);
    Some(MindView {
        who,
        sociable: p.sociable,
        warmth: p.warmth,
        skill: p.skill,
        rejection_sensitivity: p.rejection_sensitivity,
        charity: p.charity,
        mood: mind.mood(&ctx, who),
        worry: mind.worry(&ctx, who),
        regard,
        episodes: memory.episodes[recent(memory.episodes.len())..]
            .iter()
            .map(|e| EpisodeView {
                tick: e.tick,
                with: e.with,
                frame: e.frame,
                peak: e.peak,
                end: e.end,
                they_seemed: e.they_seemed,
                notes: e.notes.clone(),
            })
            .collect(),
        heard: memory.heard[recent(memory.heard.len())..]
            .iter()
            .map(|h| HeardView {
                tick: h.tick,
                from: h.from,
                kind: h.kind,
                subject: h.subject.clone(),
                overheard: h.overheard,
            })
            .collect(),
    })
}

/// Every good on every object, in object order.
#[must_use]
pub fn stock_views<C: CharacterSet>(sim: &Sim<C>) -> Vec<StockView> {
    let world = sim.world();
    sim.state()
        .stock
        .iter()
        .flat_map(|(&object, goods)| {
            goods.iter().map(move |(good, &count)| StockView {
                object: ObjectId(object),
                key: world
                    .objects
                    .get(usize::from(object))
                    .map_or_else(String::new, |o| o.id.clone()),
                good: good.clone(),
                count,
            })
        })
        .collect()
}

#[must_use]
pub fn clock_view<C: CharacterSet>(sim: &Sim<C>) -> ClockView {
    let tick = sim.tick();
    let calendar = tick.calendar();
    ClockView {
        tick,
        calendar,
        time: calendar.time,
    }
}

#[cfg(all(test, feature = "schema"))]
mod export {
    //! `cargo test -p sim --features schema export_bindings` writes the TypeScript files.
    //! ts-rs exports on `#[ts(export)]` types when their generated test runs; this module
    //! only exists so the command has a stable name to target.
    #[test]
    fn export_bindings() {}
}
