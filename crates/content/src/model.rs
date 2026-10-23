//! The authored data model. Mirrors `docs/design/world-layout.md`.
use serde::{Deserialize, Serialize};

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
    /// Impassable terrain (water, trees, cliffs).
    #[serde(default)]
    pub blocked: Vec<Rect>,
    /// Where a debug warp lands.
    pub arrival: Pos,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Room {
    pub id: String,
    pub rect: Rect,
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
    /// bed, counter, oven, table, chair, stove, shelf...
    pub kind: String,
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
}

/// Everything loaded from the content directory.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Content {
    pub zones: Vec<Zone>,
    pub buildings: Vec<Building>,
    pub villagers: Vec<Villager>,
}
