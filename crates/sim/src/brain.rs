//! The character interface. A character set owns every villager's typed private state,
//! reads the world through `Ctx`, and writes by returning commands. The sim applies them.
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::time::{Calendar, Tick, TimeOfDay};
use crate::villager::VillagerState;
use content::{Content, Nav, Pos, World};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
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
}

impl Ctx<'_> {
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
}

/// A command scheduled for a time of day.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Planned {
    pub at: TimeOfDay,
    pub command: Command,
}

/// Why the brain was asked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trigger {
    /// Reached the target of a `GoTo`.
    Arrived,
    /// A `Wait` elapsed.
    Waited,
    /// Could not reach the target.
    Stuck,
}

/// All characters in a town. Implemented once per town by the `characters` crate with a
/// typed struct per villager; the sim stays generic over it.
pub trait CharacterSet: Clone + Debug + Serialize + DeserializeOwned {
    /// Initial private state, built from content.
    fn new(content: &Content, world: &World) -> Self;

    /// Called at the start of every day (and once at sim start) for each villager.
    /// Returns the day's intentions in any order; the sim sorts them by time.
    fn plan_day(&mut self, who: VillagerId, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned>;

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
    fn new(_content: &Content, _world: &World) -> Self {
        DataCharacters
    }

    fn plan_day(&mut self, who: VillagerId, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        baseline_plan(who, ctx)
    }
}

/// The content schedule for `who` on the current day: most specific day pattern wins per
/// departure time.
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
    by_time.into_values().map(|(_, p)| p).collect()
}
