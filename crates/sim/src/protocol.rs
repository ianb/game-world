//! Boundary types: what crosses from the sim to the viewer and back. Integer-only, no maps,
//! no `u64`/`usize`, no `serde(with)`. Bump `PROTOCOL_VERSION` on any change.
use crate::brain::CharacterSet;
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::sim::Sim;
use crate::time::{Calendar, Tick, TimeOfDay};
use content::{Cell, PlaceKind, Pos, Rect};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 30;

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
        topic: Topic,
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
        meant: Gesture,
        did: Gesture,
        taken_as: Gesture,
        warmth: i8,
        attribution: Attribution,
        caught: Caught,
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
    /// Something happened to `who`'s body.
    Mishap {
        tick: Tick,
        who: VillagerId,
        what: Mishap,
        place: PlaceId,
        x: u16,
        y: u16,
    },
    /// A hen of `owner`'s laid; `now` is how many eggs are in the coop.
    Laid {
        tick: Tick,
        flock: String,
        owner: VillagerId,
        object: ObjectId,
        now: u16,
    },
    /// `who` left goods on the platform for the next train.
    Consigned {
        tick: Tick,
        who: VillagerId,
        good: String,
        count: u16,
        cause: Cause,
    },
    /// A train stopped at the platform and left `delivered` in the goods shed.
    TrainArrived {
        tick: Tick,
        delivered: Vec<GoodsLine>,
    },
    /// The train left with what was on the platform, and paid for it.
    TrainLeft {
        tick: Tick,
        shipped: Vec<Shipment>,
    },
    /// A train went through without stopping.
    TrainPassed {
        tick: Tick,
    },
    /// One of `who`'s moods changed level, `was` to `level` (0 to 3), with what moved it.
    MoodShifted {
        tick: Tick,
        who: VillagerId,
        mood: MoodKind,
        was: u8,
        level: u8,
        cause: String,
    },
    /// It rained in the night: every field is soaked.
    Rained {
        tick: Tick,
    },
    /// Overnight, `tiles` of a plot's crop ripened, or spoiled past saving.
    CropsRipened {
        tick: Tick,
        plot: String,
        tiles: u16,
    },
    CropsSpoiled {
        tick: Tick,
        plot: String,
        tiles: u16,
    },
    /// `who` worked one tile of a field. A harvest that made up a whole unit of the good
    /// says what went into the store.
    Tended {
        tick: Tick,
        who: VillagerId,
        plot: String,
        x: u16,
        y: u16,
        work: FieldWork,
        stored: Option<GoodsLine>,
        cause: Cause,
    },
    /// What `who` told themselves about something.
    Thought {
        tick: Tick,
        who: VillagerId,
        thought: Thought,
    },
    /// `who` took one `good` from `object`: ate, chewed, drank or breathed it. For a
    /// declared substance, `batch` says which batch it came from, `lift` what it does to
    /// their spirits while it lasts, and `odd` whether it was not what they expected.
    Consumed {
        tick: Tick,
        who: VillagerId,
        object: ObjectId,
        good: String,
        batch: Option<u32>,
        lift: i8,
        odd: Odd,
        cause: Cause,
    },
    /// `who`'s craving for `good` reached `level`: 1 wanting, 2 desperate.
    Craved {
        tick: Tick,
        who: VillagerId,
        good: String,
        level: u8,
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

/// What a person does with a turn in a conversation. A small fixed set: every character
/// says what each one means to them (`Interlocutor::warmth_of`). Some usually only come
/// out of a misfire or a reading (`Jab`, `Awkward`, `Rebuff`, `Pry`), though a character
/// can mean one.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Gesture {
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

/// One turn's content: a gesture about a topic, a bid for another frame, or an answer to a
/// bid.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Act {
    Gesture { gesture: Gesture, topic: Topic },
    Bid { frame: Frame, topic: Topic },
    Answer { answer: BidAnswer },
}

/// What a conversation or a turn is about: a topic declared in the content
/// (`content/topics/`), the person it concerns, if any ("Mara's money"), and a second
/// person for topics between two ("Hal sweet on Mara": about Hal, with Mara).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct Topic {
    pub key: String,
    pub about: Option<VillagerId>,
    pub with: Option<VillagerId>,
}

impl Topic {
    /// A topic that is about nobody in particular. Character code should go through
    /// `Ctx::topic`, which checks the key is declared.
    #[must_use]
    pub fn new(key: &str) -> Topic {
        Topic {
            key: key.to_string(),
            about: None,
            with: None,
        }
    }

    #[must_use]
    pub fn about(mut self, who: VillagerId) -> Topic {
        self.about = Some(who);
        self
    }

    #[must_use]
    pub fn with(mut self, who: VillagerId) -> Topic {
        self.with = Some(who);
        self
    }

    /// Whether `who` is either person the topic concerns.
    #[must_use]
    pub fn names(&self, who: VillagerId) -> bool {
        self.about == Some(who) || self.with == Some(who)
    }

    #[must_use]
    pub fn is(&self, key: &str) -> bool {
        self.key == key
    }

    /// The topic's gloss from `content`, and whose it is by `name`: "money (Mara)",
    /// "sweet on (Hal, Mara)".
    pub fn label_with(
        &self,
        content: &content::Content,
        name: impl Fn(VillagerId) -> String,
    ) -> String {
        let gloss = content
            .topic(&self.key)
            .map_or(self.key.as_str(), |t| t.gloss.as_str());
        match (self.about, self.with) {
            (Some(a), Some(w)) => format!("{gloss} ({}, {})", name(a), name(w)),
            (Some(v), None) | (None, Some(v)) => format!("{gloss} ({})", name(v)),
            (None, None) => gloss.to_string(),
        }
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.about, self.with) {
            (Some(a), Some(w)) => write!(f, "{} of #{} with #{}", self.key, a.0, w.0),
            (Some(v), None) => write!(f, "{} of #{}", self.key, v.0),
            (None, Some(w)) => write!(f, "{} with #{}", self.key, w.0),
            (None, None) => write!(f, "{}", self.key),
        }
    }
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

/// How much of a turn reached someone: they were spoken to, they were close enough for the
/// words, close enough for the tone and any name in it, or only heard that voices were
/// raised.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Caught {
    Addressed,
    Close,
    Gist,
    Noise,
}

/// Whether a hearer put a move down to the speaker's character or to circumstances.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Attribution {
    Character,
    Situation,
}

/// A small accident to the body.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Mishap {
    /// Caught a foot in long grass or weeds.
    Tripped,
}

/// Who or what a villager holds responsible, in their own account of an event.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Blame {
    Me,
    Them(VillagerId),
    World,
    Luck,
}

/// A villager's account of something to themselves: what it was, whose doing, whether it
/// will last and how far it reaches (explanatory style, after Seligman), and how it felt.
/// Said `aloud`, anyone in earshot can overhear it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct Thought {
    pub about: Topic,
    pub blame: Blame,
    pub lasting: bool,
    pub everywhere: bool,
    /// -3 to 3.
    pub feeling: i8,
    pub aloud: bool,
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
    /// What a villager told themselves about something.
    Thought,
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
    /// Trains, and goods sent and delivered by rail.
    Railway,
    /// Animals: eggs laid.
    Animals,
    /// A mood strong enough to act on.
    Mood,
    /// Something taken that was not what it should be, or a craving gone desperate.
    Substance,
    /// Fields: harvests, crops ripening and spoiling.
    Fields,
    /// Rain.
    Weather,
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
    /// Acting on something they heard: `from` said it at `heard_at`, about `topic`.
    Knew {
        from: VillagerId,
        topic: Topic,
        heard_at: Tick,
    },
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
            | Emitted::ConversationEnded { tick, .. }
            | Emitted::Mishap { tick, .. }
            | Emitted::Laid { tick, .. }
            | Emitted::Consigned { tick, .. }
            | Emitted::TrainArrived { tick, .. }
            | Emitted::TrainLeft { tick, .. }
            | Emitted::TrainPassed { tick }
            | Emitted::Rained { tick }
            | Emitted::MoodShifted { tick, .. }
            | Emitted::Consumed { tick, .. }
            | Emitted::Craved { tick, .. }
            | Emitted::CropsRipened { tick, .. }
            | Emitted::CropsSpoiled { tick, .. }
            | Emitted::Tended { tick, .. }
            | Emitted::Thought { tick, .. } => *tick,
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
            | Emitted::ConversationEnded { who, .. }
            | Emitted::Mishap { who, .. }
            | Emitted::Consigned { who, .. }
            | Emitted::Tended { who, .. }
            | Emitted::MoodShifted { who, .. }
            | Emitted::Consumed { who, .. }
            | Emitted::Craved { who, .. }
            | Emitted::Thought { who, .. } => Some(*who),
            _ => None,
        }
    }
}

/// Whether a portion did what it usually does.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Odd {
    No,
    /// Stronger than usual.
    Better,
    /// Off: it lowered their spirits.
    Worse,
    /// From a bad batch, as every portion of that batch is.
    BadBatch,
}

/// A dose working on someone.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct DoseView {
    pub good: String,
    pub batch: Option<u32>,
    pub lift: i8,
    pub hours_left: u8,
}

/// Someone's habit: how hooked, out of 100, and how much they want a dose now.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct HabitView {
    pub good: String,
    pub hooked: u8,
    pub craving: u8,
}

/// A count of one good.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct GoodsLine {
    pub good: String,
    pub count: u16,
}

/// Goods a train took, whose they were, and what the railway paid.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct Shipment {
    pub who: VillagerId,
    pub good: String,
    pub count: u16,
    pub paid: u32,
}

/// An animal as the viewer sees it. Roosting animals are out of sight.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct AnimalView {
    pub id: u16,
    /// The kind's id, as the world declares it ("chicken").
    pub kind: String,
    pub flock: String,
    pub place: PlaceId,
    pub pos: Pos,
    pub roosting: bool,
}

/// A train on the track: its front, in sixteenths of a tile along `RailView::tiles`, and
/// which way it is going.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct TrainView {
    pub head: i32,
    pub toward_end: bool,
    /// Tiles the train covers, engine first.
    pub length: u16,
    pub stopped: bool,
}

/// A train on the timetable: when it comes onto the track, reaches the stop, leaves it,
/// and is gone. A train that does not stop departs as it arrives.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct TrainTimeView {
    pub enter: Tick,
    pub arrive: Tick,
    pub depart: Tick,
    pub exit: Tick,
    pub stops: bool,
}

/// The railway's track, for drawing.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct RailView {
    pub place: PlaceId,
    pub tiles: Vec<Pos>,
    pub stop: u16,
    pub vehicle: content::Vehicle,
}

/// A mood from the shared library. Each villager is prone to a few of them.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum MoodKind {
    Lonely,
    Restless,
    Irritable,
    Anxious,
    Gloomy,
    Cheerful,
    Proud,
}

/// One of someone's moods now: its level (0 to `max`) and that level's name, whether it
/// feels good, and what last moved it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct MoodView {
    pub mood: MoodKind,
    pub level: u8,
    pub max: u8,
    pub name: String,
    pub good: bool,
    pub cause: String,
    pub since: Tick,
}

/// Where a field tile is in its year: fallow ground, tilled, sown, growing, ripe, spoiling
/// if left too long, and stubble after the harvest.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum FieldStage {
    Fallow,
    Tilled,
    Sown,
    Sprouting,
    Growing,
    Ripe,
    Spoiling,
    Stubble,
}

/// A farmer's work on one field tile.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum FieldWork {
    Till,
    Sow,
    Water,
    Weed,
    Harvest,
}

/// One field tile now. Water runs 0 (dry) to `water_full`, weeds 0 to `weeds_max`,
/// fertility 0 to `fertility_max`, and growth 0 to the plot's `grow` (ripe).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct FieldTileView {
    pub stage: FieldStage,
    pub growth: u8,
    pub water: u8,
    pub weeds: u8,
    pub fertility: u8,
}

/// A plot and the state of every tile, row by row across `rect`, with the scales the
/// numbers are on.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct FieldView {
    pub id: String,
    pub place: PlaceId,
    pub rect: Rect,
    /// The crop's id, as the world declares it ("wheat").
    pub crop: String,
    pub grow: u8,
    pub water_full: u8,
    pub weeds_max: u8,
    pub fertility_max: u8,
    pub tiles: Vec<FieldTileView>,
}

/// A bed of tilled ground and its crop.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct PlotView {
    pub place: PlaceId,
    pub rect: Rect,
    /// The crop's id, as the world declares it.
    pub crop: String,
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
    /// The villager it belongs to, by content id.
    pub owner: Option<String>,
    /// What villagers do at it, when it is somewhere to do something.
    pub activity: Option<String>,
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
    pub walls: content::Walling,
}

/// A room, for drawing its floor.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct RoomView {
    pub building: String,
    pub room: String,
    pub place: PlaceId,
    pub rect: Rect,
    pub flooring: content::Flooring,
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
    pub rooms: Vec<RoomView>,
    pub rail: Option<RailView>,
    pub plots: Vec<PlotView>,
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
                owner: o.owner.clone(),
                activity: o.activity.clone(),
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
                walls: r.walls,
            })
            .collect(),
        rooms: w
            .rooms
            .iter()
            .map(|r| RoomView {
                building: r.building.clone(),
                room: r.room.clone(),
                place: place_id(r.place),
                rect: r.rect,
                flooring: r.flooring,
            })
            .collect(),
        rail: w.rail.as_ref().map(|r| RailView {
            place: place_id(r.place),
            tiles: r.tiles.clone(),
            stop: u16::try_from(r.stop).unwrap_or(0),
            vehicle: r.vehicle,
        }),
        plots: sim
            .content()
            .zones
            .iter()
            .flat_map(|z| {
                let place = place_id(w.place_index[&z.id]);
                z.plots.iter().map(move |p| PlotView {
                    place,
                    rect: p.rect,
                    crop: p.crop.clone(),
                })
            })
            .collect(),
    }
}

#[must_use]
pub fn field_views<C: CharacterSet>(sim: &Sim<C>) -> Vec<FieldView> {
    sim.state()
        .fields
        .plots
        .iter()
        .map(|p| FieldView {
            id: p.id.clone(),
            place: p.place,
            rect: p.rect,
            crop: sim.content().crops[p.crop.index()].id.clone(),
            grow: sim.content().crops[p.crop.index()].grow,
            water_full: crate::fields::WATER_FULL,
            weeds_max: crate::fields::WEEDS_MAX,
            fertility_max: crate::fields::FERTILITY_MAX,
            tiles: p
                .tiles
                .iter()
                .map(|t| FieldTileView {
                    stage: t.stage,
                    growth: t.growth,
                    water: t.water,
                    weeds: t.weeds,
                    fertility: t.fertility,
                })
                .collect(),
        })
        .collect()
}

#[must_use]
pub fn animal_views<C: CharacterSet>(sim: &Sim<C>) -> Vec<AnimalView> {
    sim.state()
        .animals
        .iter()
        .enumerate()
        .map(|(i, a)| AnimalView {
            id: u16::try_from(i).unwrap_or(u16::MAX),
            kind: sim.content().animals[a.kind.index()].id.clone(),
            flock: sim.flocks()[usize::from(a.flock)].id.clone(),
            place: a.place,
            pos: a.pos,
            roosting: a.roosting,
        })
        .collect()
}

#[must_use]
pub fn train_view<C: CharacterSet>(sim: &Sim<C>) -> Option<TrainView> {
    sim.train_now().map(|(t, head)| TrainView {
        head,
        toward_end: matches!(t.from, content::TrackEnd::Start),
        length: u16::try_from(crate::railway::TRAIN_TILES).unwrap_or(8),
        stopped: t.stops && sim.tick() >= t.arrive && sim.tick() < t.depart,
    })
}

/// The trains still to come over the next `days` days, in order.
#[must_use]
pub fn upcoming_train_views<C: CharacterSet>(sim: &Sim<C>, days: u32) -> Vec<TrainTimeView> {
    sim.upcoming_trains(days)
        .into_iter()
        .map(|t| TrainTimeView {
            enter: t.enter,
            arrive: t.arrive,
            depart: t.depart,
            exit: t.exit,
            stops: t.stops,
        })
        .collect()
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
    pub speaker: VillagerId,
    /// Who it was said to, when they could tell.
    pub to: Option<VillagerId>,
    pub gesture: Gesture,
    /// As views show it: the topic's gloss, and whose. None when it was too far to catch.
    pub topic: Option<String>,
    pub about_me: bool,
    pub caught: Caught,
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
    pub discretion: u8,
    /// Spirits, -3 to 3: the good moods' levels less the bad ones' and the body's
    /// discomfort, with its name.
    pub mood: i8,
    pub spirits: String,
    /// Each mood they are prone to, in their content's order.
    pub moods: Vec<MoodView>,
    /// Doses working on them, and their habits.
    pub doses: Vec<DoseView>,
    pub habits: Vec<HabitView>,
    pub worry: Option<String>,
    pub regard: Vec<RegardView>,
    /// Most recent last.
    pub episodes: Vec<EpisodeView>,
    pub heard: Vec<HeardView>,
    /// What they told themselves recently, most recent last.
    pub thoughts: Vec<ThoughtView>,
}

/// A thought and when it was had.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct ThoughtView {
    pub tick: Tick,
    pub thought: Thought,
}

/// What a gesture is, for a reader browsing them: how far it carries (in tiles: the words,
/// the gist, raised voices; 0 for none), how warm it conventionally is, and how warm each
/// villager with a program of their own takes it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct GestureView {
    pub gesture: Gesture,
    pub words: u16,
    pub gist: u16,
    pub noise: u16,
    pub warmth: i8,
    pub by: Vec<WarmthView>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct WarmthView {
    pub who: VillagerId,
    pub warmth: i8,
}

#[must_use]
pub fn gesture_views<C: CharacterSet>(sim: &Sim<C>) -> Vec<GestureView> {
    let ctx = sim.ctx();
    Gesture::ALL
        .iter()
        .map(|&g| {
            let (words, gist, noise) = g.carry().reach();
            let by = (0..sim.state().villagers.len())
                .map(|i| VillagerId(u16::try_from(i).expect("few villagers")))
                .filter_map(|who| {
                    let mind = sim.state().characters.interlocutor_ref(who, &ctx)?;
                    Some(WarmthView {
                        who,
                        warmth: mind.warmth_of(g),
                    })
                })
                .collect();
            GestureView {
                gesture: g,
                words,
                gist,
                noise,
                warmth: g.warmth(),
                by,
            }
        })
        .collect()
}

/// How many recent episodes and heard things a `MindView` carries.
const MIND_RECENT: usize = 8;

/// `None` for a villager with no program of their own.
/// Each mood `who` is prone to, now.
#[must_use]
pub fn mood_views<C: CharacterSet>(sim: &Sim<C>, who: VillagerId) -> Vec<MoodView> {
    sim.state().moods[who.index()]
        .moods
        .iter()
        .map(|m| MoodView {
            mood: m.kind,
            level: m.level(),
            max: crate::moods::MAX_LEVEL,
            name: crate::moods::level_name(m.kind, m.level()).to_string(),
            good: crate::moods::good(m.kind),
            cause: m.cause.clone(),
            since: m.since,
        })
        .collect()
}

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
    let thoughts: Vec<ThoughtView> = sim
        .state()
        .record
        .entries
        .iter()
        .filter_map(|r| match &r.event {
            Emitted::Thought {
                tick,
                who: w,
                thought,
            } if *w == who => Some(ThoughtView {
                tick: *tick,
                thought: thought.clone(),
            }),
            _ => None,
        })
        .collect();
    let thoughts = thoughts[recent(thoughts.len())..].to_vec();
    Some(MindView {
        who,
        sociable: p.sociable,
        warmth: p.warmth,
        skill: p.skill,
        rejection_sensitivity: p.rejection_sensitivity,
        charity: p.charity,
        discretion: p.discretion,
        mood: mind.mood(&ctx, who),
        spirits: crate::moods::spirits_name(mind.mood(&ctx, who)).to_string(),
        moods: mood_views(sim, who),
        doses: sim.state().intake[who.index()]
            .doses
            .iter()
            .map(|d| DoseView {
                good: sim.content().substances[usize::from(d.substance)]
                    .id
                    .clone(),
                batch: d.batch,
                lift: d.lift,
                hours_left: d.hours_left,
            })
            .collect(),
        habits: sim.state().intake[who.index()]
            .habits
            .iter()
            .map(|h| HabitView {
                good: sim.content().substances[usize::from(h.substance)]
                    .id
                    .clone(),
                hooked: h.hooked,
                craving: h.craving,
            })
            .collect(),
        worry: mind.worry(&ctx, who).map(|t| t.label(&ctx)),
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
                speaker: h.speaker,
                to: h.to,
                gesture: h.gesture,
                topic: h.topic.as_ref().map(|t| t.label(&ctx)),
                about_me: h.about_me,
                caught: h.caught,
            })
            .collect(),
        thoughts,
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
