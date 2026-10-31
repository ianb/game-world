//! The authored data model. Mirrors `docs/design/world-layout.md`.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
#[cfg(feature = "schema")]
use ts_rs::TS;

/// A rectangle in tiles (`x`, `y`, `w`, `h`). For a room, the interior floor tiles; walls
/// are derived around it.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    #[must_use]
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }

    #[must_use]
    pub fn right(&self) -> u16 {
        self.x + self.w
    }

    #[must_use]
    pub fn bottom(&self) -> u16 {
        self.y + self.h
    }

    #[must_use]
    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }

    /// The rect grown by one tile on every side: a room plus its walls.
    #[must_use]
    pub fn with_walls(&self) -> Rect {
        Rect {
            x: self.x.saturating_sub(1),
            y: self.y.saturating_sub(1),
            w: self.w + 2,
            h: self.h + 2,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

/// An outdoor place. One continuous grid at the same scale as interiors.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Zone {
    pub id: String,
    pub width: u16,
    pub height: u16,
    /// Impassable terrain (trees, rock, cliffs).
    #[serde(default)]
    pub blocked: Vec<Rect>,
    /// Water: impassable, drawn as water.
    #[serde(default)]
    pub water: Vec<Rect>,
    /// Made paths and roads: cheap to walk, so routes prefer them.
    #[serde(default)]
    pub paths: Vec<Rect>,
    /// Things standing outdoors (a well, a privy, a bench), in zone coordinates.
    #[serde(default)]
    pub objects: Vec<ObjectSpec>,
    /// Named stretches of ground that character code refers to: a yard to weed, a field to
    /// work, a bank to fish from. In zone coordinates.
    #[serde(default)]
    pub areas: Vec<AreaSpec>,
    /// Tilled ground and what grows in it. Only the views read it.
    #[serde(default)]
    pub plots: Vec<PlotSpec>,
    /// Animals kept in the zone.
    #[serde(default)]
    pub flocks: Vec<FlockSpec>,
    /// The railway through the zone, if there is one.
    #[serde(default)]
    pub railway: Option<RailwaySpec>,
    /// Where a debug warp lands.
    pub arrival: Pos,
}

/// A bed of tilled ground with one crop.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PlotSpec {
    pub id: String,
    pub rect: Rect,
    pub crop: Crop,
    /// The object the harvest goes into. Without one, what is grown is eaten at home.
    #[serde(default)]
    pub store: Option<String>,
    /// The building whose people keep this plot as their garden, in turns, in the early
    /// evening.
    #[serde(default)]
    pub household: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum Crop {
    Wheat,
    Cabbages,
    Carrots,
    Potatoes,
    Beans,
    Corn,
    Flowers,
}

/// A kind of animal. Every character says how they feel about each kind.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum AnimalKind {
    Chicken,
    Sheep,
}

/// Animals of one kind that belong to someone and keep to one area. Chickens roost and
/// lay in their `coop`; animals without one stay out.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct FlockSpec {
    pub id: String,
    pub kind: AnimalKind,
    pub count: u8,
    /// The villager they belong to.
    pub owner: String,
    /// The area they keep to.
    pub area: String,
    #[serde(default)]
    pub coop: Option<String>,
}

/// A railway line across the zone: goods only. The track is a polyline of zone tiles
/// joined by straight runs. Trains stop at the tile beside `platform`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RailwaySpec {
    pub track: Vec<Pos>,
    /// Where goods for sending are left, and where the train stops.
    pub platform: String,
    /// Where delivered goods are left.
    pub shed: String,
    pub runs: Vec<TrainRun>,
    /// What the shed holds after a stopping train: each train tops it up to these counts.
    #[serde(default)]
    pub delivers: Vec<GoodsCount>,
    /// What the railway pays for goods left on the platform, each.
    #[serde(default)]
    pub buys: Vec<GoodsPrice>,
}

/// One train on the days given: it reaches the platform at `arrive` and leaves at `depart`.
/// When `stops` is false it passes through at `arrive` without stopping.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct TrainRun {
    pub days: DayPattern,
    pub arrive: String,
    #[serde(default)]
    pub depart: Option<String>,
    /// Which end of the track it comes from.
    pub from: TrackEnd,
    #[serde(default = "yes")]
    pub stops: bool,
}

fn yes() -> bool {
    true
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum TrackEnd {
    /// The first tile of the track.
    Start,
    /// The last tile.
    End,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GoodsCount {
    pub good: String,
    pub count: u16,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GoodsPrice {
    pub good: String,
    pub price: u32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
/// A named stretch of ground in a zone.
pub struct AreaSpec {
    pub id: String,
    pub rect: Rect,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Room {
    pub id: String,
    pub rect: Rect,
    #[serde(default)]
    pub flooring: Flooring,
}

/// What a room's floor is made of. Only the views read it.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum Flooring {
    /// Light boards: houses and shops.
    #[default]
    Planks,
    /// Dark boards: halls, the manor, the tavern.
    DarkPlanks,
    /// Large stone slabs: the chapel, the town hall.
    Flagstone,
    /// Rough stones: workshops, the smithy, barns.
    Cobbles,
    /// Black and white tiles: the manor's best rooms.
    Checker,
    /// Beaten earth: huts and sheds.
    Earth,
}

/// What a building's outside walls are made of. Only the views read it.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema, TS))]
#[cfg_attr(feature = "schema", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum Walling {
    /// Half-timbered plaster.
    #[default]
    Timber,
    Stone,
    Brick,
    /// Rough boards: huts, barns, sheds.
    Boards,
}

/// A walkable gap in a wall between two rooms of the same floor, or between a ground-floor
/// room and the outside (`between` then names one room and the zone is implied).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Opening {
    pub at: Pos,
    pub between: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ObjectSpec {
    pub id: String,
    /// bed, counter, oven, table, chair, stove, shelf, privy, well, cart...
    pub kind: String,
    /// The room, for a building object. Empty for an object placed in a zone.
    #[serde(default)]
    pub room: String,
    pub at: Pos,
    #[serde(default)]
    pub owner: Option<String>,
    /// Declared only where contention is wanted. See decisions.md.
    #[serde(default)]
    pub capacity: Option<u8>,
    #[serde(default)]
    pub activity: Option<String>,
    /// True for things you can stand on or walk through (rugs, doormats). Default blocks.
    #[serde(default)]
    pub walkable: bool,
    /// Goods on it when the town starts, by name.
    #[serde(default)]
    pub stock: BTreeMap<String, u16>,
}

/// One floor of a building. Inline floors are composited into the zone grid at footprint
/// scale; detached floors are their own place, reached by a door (stairs).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Floor {
    pub id: String,
    /// 0 is the ground floor, negative is below ground.
    pub level: i8,
    /// The stated reason this floor is not inline. Required for a detached floor; the
    /// validator warns on a detached ground floor without one.
    #[serde(default)]
    pub detached: Option<String>,
    pub rooms: Vec<Room>,
    #[serde(default)]
    pub openings: Vec<Opening>,
    #[serde(default)]
    pub objects: Vec<ObjectSpec>,
    /// Where a debug warp lands. Must be inside a room and off any door tile.
    pub arrival: Pos,
}

/// One end of a door record, in building-local coordinates when `floor` is set, zone
/// coordinates when `zone` is set.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DoorEnd {
    #[serde(default)]
    pub floor: Option<String>,
    #[serde(default)]
    pub zone: Option<String>,
    pub at: Pos,
}

/// A link between two places (stairs, a cellar hatch, a back gate). Both directions unless
/// `one_way`. Doors inside one inline floor, or from an inline floor to its zone, are
/// `Opening`s instead.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DoorSpec {
    pub id: String,
    pub from: DoorEnd,
    pub to: DoorEnd,
    #[serde(default)]
    pub one_way: bool,
    /// "06:00-20:00". Villagers who are not the owner may pass only inside these hours.
    #[serde(default)]
    pub hours: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Building {
    pub id: String,
    pub zone: String,
    /// Where building-local (0, 0) lands in the zone.
    pub at: Pos,
    pub floors: Vec<Floor>,
    #[serde(default)]
    pub doors: Vec<DoorSpec>,
    #[serde(default)]
    pub walls: Walling,
}

/// Which days a schedule entry applies to. More specific patterns win over `daily`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum DayPattern {
    Daily,
    Weekday,
    Weekend,
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl DayPattern {
    /// Higher is more specific.
    #[must_use]
    pub fn specificity(&self) -> u8 {
        match self {
            DayPattern::Daily => 0,
            DayPattern::Weekday | DayPattern::Weekend => 1,
            _ => 2,
        }
    }
}

/// Where a schedule entry sends the villager: next to an object, or a place's arrival tile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ScheduleTarget {
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub place: Option<String>,
}

/// One baseline schedule entry: at `at`, leave for the target and do `activity` there.
/// The time is a departure time; travel is computed.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ScheduleEntry {
    pub days: DayPattern,
    /// "HH:MM"
    pub at: String,
    #[serde(flatten)]
    pub target: ScheduleTarget,
    pub activity: String,
}

/// Money that reaches a villager from outside the town on a schedule, such as a pension.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Income {
    pub days: DayPattern,
    /// "HH:MM" or "9:00am"
    pub at: String,
    pub amount: u32,
    /// What the payment is, as the event log shows it.
    pub what: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Villager {
    pub id: String,
    pub name: String,
    pub home: String,
    /// Object id of this villager's bed.
    pub bed: String,
    /// Baseline schedule. Character code may extend or override it.
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
    /// Money to start with.
    #[serde(default)]
    pub coins: u32,
    /// Money paid in from outside on a schedule.
    #[serde(default)]
    pub income: Vec<Income>,
    /// Goods they send by rail when a train stops: taken from `object` to the platform.
    #[serde(default)]
    pub sends: Vec<Sends>,
    /// Goods they collect from the goods shed when a train has stopped, paying for them.
    #[serde(default)]
    pub fetches: Vec<Fetches>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Sends {
    pub object: String,
    pub good: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Fetches {
    pub good: String,
    /// Where they put it.
    pub to: String,
    /// What they pay the railway for each.
    pub price: u32,
    /// They fetch only enough to bring what is at `to` up to this.
    pub keep: u16,
}

/// What a topic says about the person it concerns, when it says they are in trouble.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Trouble {
    /// Short of money.
    Money,
    /// Going without food.
    Food,
}

/// Something villagers can talk about. Code refers to it by `id`; `gloss` is how views
/// show it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct TopicDef {
    pub id: String,
    pub gloss: String,
    /// Hearing this topic about someone means they are in this kind of trouble.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trouble: Option<Trouble>,
}

/// Everything loaded from the content directory.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Content {
    pub zones: Vec<Zone>,
    pub buildings: Vec<Building>,
    pub villagers: Vec<Villager>,
    /// The topics conversations can be about. A topic that code uses must be listed here.
    #[serde(default)]
    pub topics: Vec<TopicDef>,
}

impl Content {
    #[must_use]
    pub fn topic(&self, id: &str) -> Option<&TopicDef> {
        self.topics.iter().find(|t| t.id == id)
    }
}
