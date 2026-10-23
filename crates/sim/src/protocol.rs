//! Boundary types: what crosses from the sim to the viewer and back. Integer-only, no maps,
//! no `u64`/`usize`, no `serde(with)`. Bump `PROTOCOL_VERSION` on any change.
use crate::brain::CharacterSet;
use crate::ids::{PlaceId, VillagerId};
use crate::sim::Sim;
use crate::time::{Calendar, Tick, TimeOfDay};
use content::{Cell, PlaceKind, Pos, Rect};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 6;

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
    },
    /// A villager started doing something where it stands.
    Activity {
        tick: Tick,
        who: VillagerId,
        activity: String,
    },
    /// A villager could not get where it wanted.
    Stuck {
        tick: Tick,
        who: VillagerId,
        reason: String,
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
    },
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
            | Emitted::Decided { tick, .. }
            | Emitted::Talked { tick, .. } => *tick,
        }
    }

    #[must_use]
    pub fn who(&self) -> Option<VillagerId> {
        match self {
            Emitted::Moved { who, .. }
            | Emitted::Departed { who, .. }
            | Emitted::Activity { who, .. }
            | Emitted::Stuck { who, .. }
            | Emitted::Decided { who, .. }
            | Emitted::Talked { who, .. } => Some(*who),
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
