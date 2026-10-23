//! From authored rooms and openings to tile grids. Inline floors are composited into their
//! zone; detached floors become places of their own. Output is what the sim and the views
//! consume.
use crate::model::{Building, Content, Floor, Pos, Rect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum Cell {
    /// Outdoor ground.
    Ground,
    /// Impassable terrain.
    Blocked,
    Wall,
    /// Interior floor.
    Floor,
    /// A walkable gap in a wall.
    Opening,
    /// A tile that carries a door record to another place.
    Door,
    /// Nothing authored here (outside every room on a detached floor).
    Void,
}

impl Cell {
    #[must_use]
    pub fn walkable(self) -> bool {
        matches!(
            self,
            Cell::Ground | Cell::Floor | Cell::Opening | Cell::Door
        )
    }

    #[must_use]
    pub fn glyph(self) -> char {
        match self {
            Cell::Ground => '.',
            Cell::Blocked => '^',
            Cell::Wall => '#',
            Cell::Floor => ',',
            Cell::Opening => '+',
            Cell::Door => '>',
            Cell::Void => ' ',
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", ts(export))]
pub enum PlaceKind {
    Zone,
    /// A detached floor, with its building and floor ids.
    Floor,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CompiledObject {
    pub id: String,
    pub kind: String,
    pub place: usize,
    pub at: Pos,
    pub owner: Option<String>,
    pub capacity: Option<u8>,
    pub activity: Option<String>,
    pub walkable: bool,
    pub building: String,
    pub room: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CompiledDoor {
    pub id: String,
    pub building: String,
    pub from_place: usize,
    pub from: Pos,
    pub to_place: usize,
    pub to: Pos,
    pub one_way: bool,
    pub hours: Option<String>,
    pub owner: Option<String>,
}

/// A compiled room: where it landed, for views and validators.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CompiledRoom {
    pub building: String,
    pub floor: String,
    pub room: String,
    pub place: usize,
    pub rect: Rect,
}

/// The roof over an inline floor: hidden when the player is inside.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Roof {
    pub building: String,
    pub place: usize,
    pub rect: Rect,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub kind: PlaceKind,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
    pub arrival: Pos,
    /// Walkable and not covered by a solid object. Built once at the end of compilation.
    pub passable: Vec<bool>,
}

impl Place {
    #[must_use]
    pub fn in_bounds(&self, x: u16, y: u16) -> bool {
        x < self.width && y < self.height
    }

    #[must_use]
    pub fn cell(&self, x: u16, y: u16) -> Cell {
        if self.in_bounds(x, y) {
            self.cells[usize::from(y) * usize::from(self.width) + usize::from(x)]
        } else {
            Cell::Void
        }
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if self.in_bounds(x, y) {
            self.cells[usize::from(y) * usize::from(self.width) + usize::from(x)] = cell;
        }
    }

    #[must_use]
    pub fn walkable(&self, x: u16, y: u16) -> bool {
        self.cell(x, y).walkable()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct World {
    pub places: Vec<Place>,
    pub rooms: Vec<CompiledRoom>,
    pub objects: Vec<CompiledObject>,
    pub doors: Vec<CompiledDoor>,
    pub roofs: Vec<Roof>,
    /// place id -> index
    pub place_index: BTreeMap<String, usize>,
}

impl World {
    #[must_use]
    pub fn place(&self, id: &str) -> Option<&Place> {
        self.place_index.get(id).map(|&i| &self.places[i])
    }

    /// True if an object blocks the tile.
    #[must_use]
    pub fn blocked_by_object(&self, place: usize, x: u16, y: u16) -> bool {
        self.objects
            .iter()
            .any(|o| o.place == place && !o.walkable && o.at.x == x && o.at.y == y)
    }

    /// Index into `rooms` of the room containing the tile, if any.
    #[must_use]
    pub fn room_at(&self, place: usize, x: u16, y: u16) -> Option<usize> {
        self.rooms
            .iter()
            .position(|r| r.place == place && r.rect.contains(x, y))
    }

    /// Walkable for movement: the cell allows it and no solid object sits there.
    #[must_use]
    pub fn passable(&self, place: usize, x: u16, y: u16) -> bool {
        let p = &self.places[place];
        p.in_bounds(x, y) && p.passable[usize::from(y) * usize::from(p.width) + usize::from(x)]
    }

    /// Recomputes every place's passability grid from cells and objects.
    pub fn finish(&mut self) {
        for pi in 0..self.places.len() {
            let (w, h) = (self.places[pi].width, self.places[pi].height);
            let grid: Vec<bool> = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .map(|(x, y)| self.places[pi].walkable(x, y) && !self.blocked_by_object(pi, x, y))
                .collect();
            self.places[pi].passable = grid;
        }
    }
}

/// Place id for a detached floor.
#[must_use]
pub fn floor_place_id(building: &str, floor: &str) -> String {
    format!("{building}/{floor}")
}

/// Compiles content into a world. Does no validation beyond what it needs to not panic;
/// run `validate` on the content first (or on the result) for diagnostics.
#[must_use]
pub fn compile(content: &Content) -> World {
    let mut world = World::default();

    for z in &content.zones {
        let mut place = Place {
            id: z.id.clone(),
            kind: PlaceKind::Zone,
            width: z.width,
            height: z.height,
            cells: vec![Cell::Ground; usize::from(z.width) * usize::from(z.height)],
            arrival: z.arrival,
            passable: Vec::new(),
        };
        for r in &z.blocked {
            fill(&mut place, *r, Cell::Blocked);
        }
        add_place(&mut world, place);
    }

    for b in &content.buildings {
        for f in &b.floors {
            if f.detached.is_some() || f.level != 0 {
                compile_detached(&mut world, b, f);
            } else {
                compile_inline(&mut world, b, f);
            }
        }
    }

    for b in &content.buildings {
        for d in &b.doors {
            let (from_place, from) = resolve_end(&world, b, &d.from);
            let (to_place, to) = resolve_end(&world, b, &d.to);
            world.doors.push(CompiledDoor {
                id: d.id.clone(),
                building: b.id.clone(),
                from_place,
                from,
                to_place,
                to,
                one_way: d.one_way,
                hours: d.hours.clone(),
                owner: d.owner.clone(),
            });
            // Only mark cells that are already walkable; a door on a wall stays a wall so the
            // validator reports it instead of the compiler hiding it.
            mark_door(&mut world.places[from_place], from);
            if !d.one_way {
                mark_door(&mut world.places[to_place], to);
            }
        }
    }

    world.finish();
    world
}

fn mark_door(place: &mut Place, at: Pos) {
    if place.walkable(at.x, at.y) {
        place.set(at.x, at.y, Cell::Door);
    }
}

fn add_place(world: &mut World, place: Place) -> usize {
    let idx = world.places.len();
    world.place_index.insert(place.id.clone(), idx);
    world.places.push(place);
    idx
}

fn fill(place: &mut Place, r: Rect, cell: Cell) {
    for y in r.y..r.bottom() {
        for x in r.x..r.right() {
            place.set(x, y, cell);
        }
    }
}

/// Paints a floor's rooms, walls, openings and objects into `place`, offset by `off`.
fn paint_floor(world: &mut World, place_idx: usize, b: &Building, f: &Floor, off: Pos) {
    let shift = |p: Pos| Pos {
        x: p.x + off.x,
        y: p.y + off.y,
    };
    let shift_rect = |r: Rect| Rect {
        x: r.x + off.x,
        y: r.y + off.y,
        w: r.w,
        h: r.h,
    };

    // Walls first, so adjacent rooms' floors overwrite shared walls correctly.
    for room in &f.rooms {
        fill(
            &mut world.places[place_idx],
            shift_rect(room.rect.with_walls()),
            Cell::Wall,
        );
    }
    for room in &f.rooms {
        let rect = shift_rect(room.rect);
        fill(&mut world.places[place_idx], rect, Cell::Floor);
        world.rooms.push(CompiledRoom {
            building: b.id.clone(),
            floor: f.id.clone(),
            room: room.id.clone(),
            place: place_idx,
            rect,
        });
    }
    for o in &f.openings {
        let p = shift(o.at);
        world.places[place_idx].set(p.x, p.y, Cell::Opening);
    }
    for o in &f.objects {
        world.objects.push(CompiledObject {
            id: o.id.clone(),
            kind: o.kind.clone(),
            place: place_idx,
            at: shift(o.at),
            owner: o.owner.clone(),
            capacity: o.capacity,
            activity: o.activity.clone(),
            walkable: o.walkable,
            building: b.id.clone(),
            room: o.room.clone(),
        });
    }
}

fn compile_inline(world: &mut World, b: &Building, f: &Floor) {
    let Some(&zone_idx) = world.place_index.get(&b.zone) else {
        return;
    };
    paint_floor(world, zone_idx, b, f, b.at);
    if let Some(bbox) = bbox_with_walls(f) {
        world.roofs.push(Roof {
            building: b.id.clone(),
            place: zone_idx,
            rect: Rect {
                x: bbox.x + b.at.x,
                y: bbox.y + b.at.y,
                w: bbox.w,
                h: bbox.h,
            },
        });
    }
}

fn compile_detached(world: &mut World, b: &Building, f: &Floor) {
    let bbox = bbox_with_walls(f).unwrap_or(Rect {
        x: 0,
        y: 0,
        w: 1,
        h: 1,
    });
    let width = bbox.right();
    let height = bbox.bottom();
    let place = Place {
        id: floor_place_id(&b.id, &f.id),
        kind: PlaceKind::Floor,
        width,
        height,
        cells: vec![Cell::Void; usize::from(width) * usize::from(height)],
        arrival: f.arrival,
        passable: Vec::new(),
    };
    let idx = add_place(world, place);
    paint_floor(world, idx, b, f, Pos { x: 0, y: 0 });
}

/// Bounding box of a floor's rooms including their walls, in building-local coordinates.
#[must_use]
pub fn bbox_with_walls(f: &Floor) -> Option<Rect> {
    let mut it = f.rooms.iter().map(|r| r.rect.with_walls());
    let first = it.next()?;
    Some(it.fold(first, |a, r| {
        let x = a.x.min(r.x);
        let y = a.y.min(r.y);
        let right = a.right().max(r.right());
        let bottom = a.bottom().max(r.bottom());
        Rect {
            x,
            y,
            w: right - x,
            h: bottom - y,
        }
    }))
}

/// Resolves a door end to (place index, place coordinates). Unknown references resolve to
/// place 0 at the given position; the validator reports them.
fn resolve_end(world: &World, b: &Building, end: &crate::model::DoorEnd) -> (usize, Pos) {
    if let Some(floor_id) = &end.floor {
        let floor = b.floors.iter().find(|f| &f.id == floor_id);
        let is_inline = floor.is_some_and(|f| f.detached.is_none() && f.level == 0);
        if is_inline {
            let zone = world.place_index.get(&b.zone).copied().unwrap_or(0);
            return (
                zone,
                Pos {
                    x: end.at.x + b.at.x,
                    y: end.at.y + b.at.y,
                },
            );
        }
        let id = floor_place_id(&b.id, floor_id);
        return (world.place_index.get(&id).copied().unwrap_or(0), end.at);
    }
    if let Some(zone) = &end.zone {
        return (world.place_index.get(zone).copied().unwrap_or(0), end.at);
    }
    (0, end.at)
}
