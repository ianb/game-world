//! The character interface. A character set owns every villager's typed private state,
//! reads the world through `Ctx`, and writes by returning commands. The sim applies them.
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::needs::{Need, Needs};
use crate::protocol::{Blame, Caught, Frame, Gesture, Mishap, Thought, Topic};
use crate::recall::{Perceived, Record, Recorded};
use crate::social::Interlocutor;
use crate::terrain::Terrain;
use crate::time::{Calendar, Tick, TimeOfDay};
use crate::villager::VillagerState;
use content::{Content, Nav, Pos, Rect, World};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::fmt::Debug;

/// Read-only view of the sim for character code.
#[derive(Debug)]
pub struct Ctx<'a> {
    pub tick: Tick,
    pub calendar: Calendar,
    pub content: &'a Content,
    pub world: &'a World,
    pub nav: &'a Nav,
    pub villagers: &'a [VillagerState],
    pub terrain: &'a Terrain,
    /// Goods on objects, by object index then good.
    pub stock: &'a BTreeMap<u16, BTreeMap<String, u16>>,
    /// What the town keeps of recent events, with who could perceive each.
    pub record: &'a Record,
}

impl Ctx<'_> {
    /// The topic `key`, which must be declared in the content (`content/topics/`). Panics
    /// on an undeclared key, so a typo fails the first test that reaches it.
    #[must_use]
    pub fn topic(&self, key: &str) -> Topic {
        assert!(
            self.content.topic(key).is_some(),
            "topic {key:?} is not declared in content/topics"
        );
        Topic::new(key)
    }

    /// Kept events `who` did, saw or heard since `since`, oldest first. What they make of
    /// them, and whether they noticed at all, is their own code's business.
    pub fn recall(
        &self,
        who: VillagerId,
        since: Tick,
    ) -> impl Iterator<Item = (&Recorded, Perceived)> {
        self.record.recall(who, since)
    }

    #[must_use]
    pub fn villager(&self, who: VillagerId) -> &VillagerState {
        &self.villagers[who.index()]
    }

    #[must_use]
    pub fn villager_id(&self, id: &str) -> Option<VillagerId> {
        self.villagers
            .iter()
            .position(|v| v.id == id)
            .map(|i| VillagerId(u16::try_from(i).expect("few villagers")))
    }

    #[must_use]
    pub fn object_id(&self, id: &str) -> Option<ObjectId> {
        self.world
            .objects
            .iter()
            .position(|o| o.id == id)
            .map(|i| ObjectId(u16::try_from(i).expect("few objects")))
    }

    #[must_use]
    pub fn place_id(&self, id: &str) -> Option<PlaceId> {
        self.world
            .place_index
            .get(id)
            .map(|&i| PlaceId(u16::try_from(i).expect("few places")))
    }

    /// A random passable tile of the given cell kind in a place, for wandering.
    #[must_use]
    pub fn random_tile(&self, place: PlaceId, kind: content::Cell, rng: &mut Pcg32) -> Option<Pos> {
        use rand::RngExt;
        let p = &self.world.places[place.index()];
        let candidates: Vec<Pos> = (0..p.height)
            .flat_map(|y| (0..p.width).map(move |x| (x, y)))
            .filter(|&(x, y)| p.cell(x, y) == kind && self.world.passable(place.index(), x, y))
            .map(|(x, y)| Pos { x, y })
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let n = u32::try_from(candidates.len()).expect("small grid");
        Some(candidates[rng.random_range(0..n) as usize])
    }

    #[must_use]
    pub fn coins(&self, who: VillagerId) -> u32 {
        self.villager(who).coins
    }

    #[must_use]
    pub fn needs(&self, who: VillagerId) -> &Needs {
        &self.villager(who).needs
    }

    /// Goods of one kind on an object.
    #[must_use]
    pub fn stock(&self, object: ObjectId, good: &str) -> u16 {
        self.stock
            .get(&object.0)
            .and_then(|goods| goods.get(good))
            .copied()
            .unwrap_or(0)
    }

    /// Objects of a kind, in content order.
    #[must_use]
    pub fn objects_of_kind(&self, kind: &str) -> Vec<ObjectId> {
        self.world
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.kind == kind)
            .map(|(i, _)| ObjectId(u16::try_from(i).expect("few objects")))
            .collect()
    }

    /// The object of a kind nearest to `who` by travel time, if any is reachable.
    #[must_use]
    pub fn nearest_object(&self, who: VillagerId, kind: &str) -> Option<ObjectId> {
        let v = self.villager(who);
        let here = content::Spot::new(v.place.index(), v.pos.x, v.pos.y);
        self.objects_of_kind(kind)
            .into_iter()
            .filter_map(|id| {
                let o = &self.world.objects[id.index()];
                let access = content::validate::neighbours(o.at.x, o.at.y)
                    .into_iter()
                    .find(|&(x, y)| self.world.passable(o.place, x, y))?;
                let there = content::Spot::new(o.place, access.0, access.1);
                let cost = self.nav.route(self.world, here, there)?.cost;
                Some((cost, id))
            })
            .min_by_key(|(cost, id)| (*cost, id.0))
            .map(|(_, id)| id)
    }

    /// Outdoor tiles in `place` that are tall, weedy or overgrown, row-major.
    #[must_use]
    pub fn overgrown_tiles(&self, place: PlaceId) -> Vec<Pos> {
        self.terrain.overgrown_tiles(self.world, place.index())
    }

    /// The overgrown tile inside `area` nearest to `from` (Manhattan), if any.
    #[must_use]
    pub fn nearest_overgrown(&self, place: PlaceId, area: Rect, from: Pos) -> Option<Pos> {
        self.overgrown_tiles(place)
            .into_iter()
            .filter(|p| area.contains(p.x, p.y))
            .min_by_key(|p| from.x.abs_diff(p.x) + from.y.abs_diff(p.y))
    }

    /// Villagers in the same place as `who`, excluding `who`.
    #[must_use]
    pub fn same_place(&self, who: VillagerId) -> Vec<VillagerId> {
        let here = self.villager(who).place;
        self.villagers
            .iter()
            .enumerate()
            .filter(|(i, v)| *i != who.index() && v.place == here)
            .map(|(i, _)| VillagerId(u16::try_from(i).expect("few villagers")))
            .collect()
    }
}

/// Where to go.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Target {
    /// Stand on an access tile next to the object.
    Object(ObjectId),
    /// The place's arrival tile.
    Place(PlaceId),
    Tile {
        place: PlaceId,
        pos: Pos,
    },
}

/// What a character asks the sim to do. Applied in order.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Walk to the target, then start `activity` there.
    GoTo { target: Target, activity: String },
    /// Stay put doing `activity`; wake the brain again after `ticks`.
    Wait { ticks: u32, activity: String },
    /// Stop moving and do nothing until the next planned item.
    Idle,
    /// Does nothing to the world; emits a `Decided` line so a reader can see why the
    /// character chose what it chose. Character code should say its reasons.
    Note(String),
    /// A word in passing. Works whether or not either party is walking; changes nothing
    /// but emits `Talked` and gives the other party a `TalkedTo` trigger.
    Greet { with: VillagerId },
    /// A conversation. Both must be in the same place and standing still. Both switch to
    /// the `talk` activity for `minutes`, then resume what they were doing and get a
    /// `TalkEnded` trigger. Ignored if either is already talking.
    Talk {
        with: VillagerId,
        frame: Frame,
        topic: Topic,
        minutes: u32,
    },
    /// Clear grass and weeds within `radius` tiles of where you stand. Instant; pair it
    /// with a `Wait` for the time the work takes. Emits `Cleared`.
    Clear { radius: u16 },
    /// Move money. Pays what the payer has if that is less than `amount`. Emits `Paid`.
    Pay {
        from: Party,
        to: Party,
        amount: u32,
        what: String,
    },
    /// Change the goods on an object by `delta` (never below zero). Emits `Stocked`.
    Stock {
        object: ObjectId,
        good: String,
        delta: i32,
    },
    /// Go to `target`, do `activity` there for `minutes`, then walk back to where you were
    /// and resume what you were doing. A second interlude while one is running keeps the
    /// first return point.
    Interlude {
        target: Target,
        activity: String,
        minutes: u32,
    },
    /// The commands after this one in the same reaction act on something heard: `from`
    /// said it at `heard_at`, about `topic`. Their events carry it as their cause.
    Because {
        from: VillagerId,
        topic: Topic,
        heard_at: Tick,
    },
    /// Tell yourself something about what happened. Changes nothing; emits `Thought`, which
    /// the town keeps for a month. Only the thinker perceives it, unless it is said aloud:
    /// then it carries like a complaint (if it felt bad) or a remark, and anyone in earshot
    /// may overhear it.
    Think(Thought),
}

/// Who pays or is paid. `Outside` is the world beyond the town: pensions come from it,
/// supplies are bought from it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Party {
    Villager(VillagerId),
    Outside,
}

impl Command {
    /// One-line summary for traces.
    #[must_use]
    pub fn summary(&self, ctx: &Ctx<'_>) -> String {
        match self {
            Command::GoTo { target, activity } => format!("{activity} @ {}", target.summary(ctx)),
            Command::Wait { ticks, activity } => format!("{activity} for {}m", ticks / 60),
            Command::Idle => "idle".to_string(),
            Command::Note(text) => format!("note: {text}"),
            Command::Think(t) => format!("thinks: {}", t.describe(ctx)),
            Command::Because { from, topic, .. } => format!(
                "because {} said {}",
                ctx.villager(*from).name,
                topic.label(ctx)
            ),
            Command::Greet { with } => format!("greet {}", ctx.villager(*with).name),
            Command::Talk {
                with,
                frame,
                topic,
                minutes,
            } => {
                format!(
                    "{} with {} about {} for {minutes}m",
                    frame.name(),
                    ctx.villager(*with).name,
                    topic.label(ctx)
                )
            }
            Command::Clear { radius } => format!("clear within {radius}"),
            Command::Pay {
                from,
                to,
                amount,
                what,
            } => {
                let name = |p: &Party| match p {
                    Party::Villager(v) => ctx.villager(*v).name.clone(),
                    Party::Outside => "outside".to_string(),
                };
                format!("{} pays {} {amount} for {what}", name(from), name(to))
            }
            Command::Stock {
                object,
                good,
                delta,
            } => {
                let key = ctx
                    .world
                    .objects
                    .get(object.index())
                    .map_or_else(|| format!("object#{}", object.0), |o| o.id.clone());
                format!("{good} {delta:+} at {key}")
            }
            Command::Interlude {
                target,
                activity,
                minutes,
            } => format!(
                "{activity} for {minutes}m @ {} then back",
                target.summary(ctx)
            ),
        }
    }
}

impl Target {
    #[must_use]
    pub fn summary(&self, ctx: &Ctx<'_>) -> String {
        match self {
            Target::Object(o) => ctx
                .world
                .objects
                .get(o.index())
                .map_or_else(|| format!("object#{}", o.0), |x| x.id.clone()),
            Target::Place(p) => ctx
                .world
                .places
                .get(p.index())
                .map_or_else(|| format!("place#{}", p.0), |x| x.id.clone()),
            Target::Tile { place, pos } => {
                let name = ctx
                    .world
                    .places
                    .get(place.index())
                    .map_or_else(|| format!("place#{}", place.0), |x| x.id.clone());
                format!("{name} ({}, {})", pos.x, pos.y)
            }
        }
    }
}

/// A command scheduled for a time of day.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Planned {
    pub at: TimeOfDay,
    pub command: Command,
}

/// Why the brain was asked.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Trigger {
    /// Reached the target of a `GoTo`.
    Arrived,
    /// A `Wait` elapsed.
    Waited,
    /// Could not reach the target.
    Stuck,
    /// You and `other` are now in the same place and both standing still (one of you just
    /// arrived). Fired to both, at most once per pair per `ENCOUNTER_COOLDOWN`.
    Encounter { other: VillagerId },
    /// You passed within a tile of `other` while at least one of you was walking. Fired to
    /// both, with the same cooldown.
    Passing { other: VillagerId },
    /// `other` greeted you or started a conversation with you.
    TalkedTo { other: VillagerId, topic: Topic },
    /// A conversation with `other` finished. Your previous activity is already restored.
    TalkEnded { other: VillagerId, topic: Topic },
    /// A need of yours got pressing (70) or urgent (90). `level` is its value now.
    Need { need: Need, level: u8 },
    /// Back from an errand (an `Interlude`), at the spot and activity you left.
    Resumed,
    /// A moment to take in your surroundings, every quarter hour while awake. `since` is
    /// the last one. What you notice is up to you: `Ctx::recall` has what you could have
    /// perceived, and the state of anything in front of you is in `Ctx`.
    Look { since: Tick },
    /// Something happened to your body. Intrusive: it reaches you whatever you attend to.
    Mishap { what: Mishap },
    /// Someone nearby, talking to someone else, said something loud or something about
    /// you. Intrusive. `gesture` is how you took it; what else you caught is in your
    /// `Memory::heard`.
    Overheard {
        speaker: VillagerId,
        gesture: Gesture,
        about_me: bool,
        caught: Caught,
    },
}

impl Trigger {
    /// Short label with names, for traces.
    #[must_use]
    pub fn label(&self, ctx: &Ctx<'_>) -> String {
        let name = |v: &VillagerId| ctx.villager(*v).name.clone();
        match self {
            Trigger::Arrived => "arrived".into(),
            Trigger::Waited => "waited".into(),
            Trigger::Stuck => "stuck".into(),
            Trigger::Encounter { other } => format!("encounter {}", name(other)),
            Trigger::Passing { other } => format!("passing {}", name(other)),
            Trigger::TalkedTo { other, topic } => {
                format!("{} talked about {}", name(other), topic.label(ctx))
            }
            Trigger::TalkEnded { other, topic } => {
                format!("finished {} with {}", topic.label(ctx), name(other))
            }
            Trigger::Need { need, level } => format!("{} ({level})", need.name()),
            Trigger::Resumed => "resumed".into(),
            Trigger::Look { .. } => "looked around".into(),
            Trigger::Mishap { what } => format!("{what:?}").to_lowercase(),
            Trigger::Overheard {
                speaker,
                gesture,
                about_me,
                ..
            } => format!(
                "overheard {} ({}){}",
                name(speaker),
                gesture.name(),
                if *about_me { ", about me" } else { "" }
            ),
        }
    }
}

impl Topic {
    /// The topic's gloss from the content, and whose it is: "money (Mara)".
    #[must_use]
    pub fn label(&self, ctx: &Ctx<'_>) -> String {
        self.label_with(ctx.content, |v| ctx.villager(v).name.clone())
    }
}

impl Thought {
    /// "blames the world (lasting, everywhere): tripped on weeds".
    #[must_use]
    pub fn describe(&self, ctx: &Ctx<'_>) -> String {
        let blame = match self.blame {
            Blame::Me => "blames themself".to_string(),
            Blame::Them(v) => format!("blames {}", ctx.villager(v).name),
            Blame::World => "blames the world".into(),
            Blame::Luck => "puts it down to luck".into(),
        };
        let span = match (self.lasting, self.everywhere) {
            (true, true) => " (lasting, everywhere)",
            (true, false) => " (lasting)",
            (false, true) => " (everywhere)",
            (false, false) => "",
        };
        let aloud = if self.aloud { ", aloud" } else { "" };
        format!(
            "{blame}{span}: {} ({:+}){aloud}",
            self.about.label(ctx),
            self.feeling
        )
    }
}

/// Minimum ticks between proximity triggers of one kind for the same pair.
pub const ENCOUNTER_COOLDOWN: u32 = 20 * 60;

/// How close two villagers outdoors must be to count as meeting.
pub const ENCOUNTER_RANGE: u16 = 6;

/// All characters in a town. Implemented once per town by the `characters` crate with a
/// typed struct per villager; the sim stays generic over it.
pub trait CharacterSet: Clone + Debug + Serialize + DeserializeOwned {
    /// Bump whenever any character struct changes shape, so old snapshots are refused.
    const LAYOUT_VERSION: u32;

    /// Fingerprint of the code that decides behavior. A snapshot made under different
    /// rules is refused, since continuing it would not replay what happened. Zero opts out.
    const BEHAVIOR: u64 = 0;

    /// Initial private state, built from content.
    fn new(content: &Content, world: &World) -> Self;

    /// Called at the start of every day (and once at sim start) for each villager.
    /// Returns the day's intentions in any order; the sim sorts them by time.
    fn plan_day(&mut self, who: VillagerId, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned>;

    /// The conversation partner behind a villager: their persona, memory and social
    /// program. `None` means the shared defaults with no memory.
    fn interlocutor(&mut self, who: VillagerId, ctx: &Ctx<'_>) -> Option<&mut dyn Interlocutor> {
        let _ = (who, ctx);
        None
    }

    /// The same, read-only, for views.
    fn interlocutor_ref(&self, who: VillagerId, ctx: &Ctx<'_>) -> Option<&dyn Interlocutor> {
        let _ = (who, ctx);
        None
    }

    /// Called when something finished. Return commands to run now, or none to stay put
    /// until the next planned item.
    fn react(
        &mut self,
        who: VillagerId,
        trigger: Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
    ) -> Vec<Command> {
        let _ = (who, trigger, ctx, rng);
        Vec::new()
    }
}

/// The baseline: every villager follows the schedule entries in content. Character code
/// starts from this and overrides.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataCharacters;

impl CharacterSet for DataCharacters {
    const LAYOUT_VERSION: u32 = 1;

    fn new(_content: &Content, _world: &World) -> Self {
        DataCharacters
    }

    fn plan_day(&mut self, who: VillagerId, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        baseline_plan(who, ctx)
    }
}

/// The content schedule for `who` on the current day: most specific day pattern wins per
/// departure time. Scheduled income is paid at its time.
#[must_use]
pub fn baseline_plan(who: VillagerId, ctx: &Ctx<'_>) -> Vec<Planned> {
    use content::DayPattern;
    use std::collections::BTreeMap;
    let weekday = ctx.calendar.weekday;
    let villager = &ctx.content.villagers[who.index()];
    let applies = |d: &DayPattern| match d {
        DayPattern::Daily => true,
        DayPattern::Weekday => !weekday.is_weekend(),
        DayPattern::Weekend => weekday.is_weekend(),
        DayPattern::Mon => weekday == crate::time::Weekday::Mon,
        DayPattern::Tue => weekday == crate::time::Weekday::Tue,
        DayPattern::Wed => weekday == crate::time::Weekday::Wed,
        DayPattern::Thu => weekday == crate::time::Weekday::Thu,
        DayPattern::Fri => weekday == crate::time::Weekday::Fri,
        DayPattern::Sat => weekday == crate::time::Weekday::Sat,
        DayPattern::Sun => weekday == crate::time::Weekday::Sun,
    };
    let mut by_time: BTreeMap<u32, (u8, Planned)> = BTreeMap::new();
    for e in &villager.schedule {
        if !applies(&e.days) {
            continue;
        }
        let Some(secs) = content::validate::parse_time(&e.at) else {
            continue;
        };
        let target = match (&e.target.object, &e.target.place) {
            (Some(o), _) => ctx.object_id(o).map(Target::Object),
            (None, Some(p)) => ctx.place_id(p).map(Target::Place),
            _ => None,
        };
        let Some(target) = target else { continue };
        let planned = Planned {
            at: TimeOfDay::from_seconds(secs),
            command: Command::GoTo {
                target,
                activity: e.activity.clone(),
            },
        };
        let spec = e.days.specificity();
        match by_time.get(&secs) {
            Some((existing, _)) if *existing >= spec => {}
            _ => {
                by_time.insert(secs, (spec, planned));
            }
        }
    }
    let mut plan: Vec<Planned> = by_time.into_values().map(|(_, p)| p).collect();
    for i in villager.income.iter().filter(|i| applies(&i.days)) {
        let Some(secs) = content::validate::parse_time(&i.at) else {
            continue;
        };
        plan.push(Planned {
            at: TimeOfDay::from_seconds(secs),
            command: Command::Pay {
                from: Party::Outside,
                to: Party::Villager(who),
                amount: i.amount,
                what: i.what.clone(),
            },
        });
    }
    plan
}
