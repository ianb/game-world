//! Living ground. Every outdoor tile carries two counters: `wear`, the footfall that
//! turns grass into a track and then a cowpath, and `growth`, the days since the tile was
//! last trampled or cleared. Growth turns grass tall and then to brush, and laid paths
//! weedy and then overgrown. Both are in the state and the hash, so cowpaths and
//! overgrowth replay.
use crate::ids::PlaceId;
use crate::protocol::TerrainView;
use content::{Cell, PlaceKind, Pos, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Footfall at which ground starts to read as a track, and as a cowpath.
pub const WEAR_TRACK: u8 = 24;
pub const WEAR_PATH: u8 = 80;
/// Footfall lost per day of disuse.
pub const WEAR_FADE_PER_DAY: u8 = 2;
/// Growth gained per day on every tile that grows.
pub const GROWTH_PER_DAY: u8 = 1;
/// Growth lost per step onto a tile.
pub const TRAMPLE: u8 = 4;
/// Growth at which grass is tall and a path weedy, and at which both are overgrown.
pub const GROWTH_TALL: u8 = 7;
pub const GROWTH_OVERGROWN: u8 = 28;
/// Growth never counts above this, so a long-wild tile clears as quickly as any other.
pub const GROWTH_MAX: u8 = 60;

/// Step costs. Indoor cells and clean paths are 1, grass 4, and they rise with growth.
pub const COST_GRASS: u32 = 4;
pub const COST_TALL_GRASS: u32 = 6;
pub const COST_BRUSH: u32 = 9;
pub const COST_WEEDY_PATH: u32 = 2;
pub const COST_OVERGROWN_PATH: u32 = 4;
pub const COST_TRACK: u32 = 2;
pub const COST_COWPATH: u32 = 1;

pub use crate::protocol::Surface;

impl Surface {
    #[must_use]
    pub fn glyph(self) -> char {
        match self {
            Surface::Grass => '·',
            Surface::TallGrass => '"',
            Surface::Brush | Surface::OvergrownPath => '♠',
            Surface::Track => '∙',
            Surface::Cowpath => '▒',
            Surface::Path => '░',
            Surface::WeedyPath => '┈',
        }
    }

    #[must_use]
    pub fn cost(self) -> u32 {
        match self {
            Surface::Grass => COST_GRASS,
            Surface::TallGrass => COST_TALL_GRASS,
            Surface::Brush => COST_BRUSH,
            Surface::Track => COST_TRACK,
            Surface::Cowpath | Surface::Path => COST_COWPATH,
            Surface::WeedyPath => COST_WEEDY_PATH,
            Surface::OvergrownPath => COST_OVERGROWN_PATH,
        }
    }

    /// Needs clearing: tall or worse.
    #[must_use]
    pub fn overgrown(self) -> bool {
        matches!(
            self,
            Surface::TallGrass | Surface::Brush | Surface::WeedyPath | Surface::OvergrownPath
        )
    }
}

/// The counters for one outdoor place, row-major.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceTerrain {
    pub width: u16,
    pub wear: Vec<u8>,
    pub growth: Vec<u8>,
}

impl PlaceTerrain {
    fn index(&self, pos: Pos) -> usize {
        usize::from(pos.y) * usize::from(self.width) + usize::from(pos.x)
    }
}

/// Counters for every zone, keyed by place index. Indoor places have none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Terrain {
    pub places: BTreeMap<u16, PlaceTerrain>,
}

impl Terrain {
    #[must_use]
    pub fn new(world: &World) -> Terrain {
        let mut places = BTreeMap::new();
        for (i, p) in world.places.iter().enumerate() {
            if p.kind != PlaceKind::Zone {
                continue;
            }
            let n = usize::from(p.width) * usize::from(p.height);
            places.insert(
                u16::try_from(i).expect("few places"),
                PlaceTerrain {
                    width: p.width,
                    wear: vec![0; n],
                    growth: vec![0; n],
                },
            );
        }
        Terrain { places }
    }

    #[must_use]
    pub fn wear(&self, place: usize, pos: Pos) -> u8 {
        self.places
            .get(&u16::try_from(place).expect("few places"))
            .map_or(0, |t| t.wear[t.index(pos)])
    }

    #[must_use]
    pub fn growth(&self, place: usize, pos: Pos) -> u8 {
        self.places
            .get(&u16::try_from(place).expect("few places"))
            .map_or(0, |t| t.growth[t.index(pos)])
    }

    /// The tile's surface, or None for cells that neither wear nor grow.
    #[must_use]
    pub fn surface(&self, world: &World, place: usize, pos: Pos) -> Option<Surface> {
        let cell = world.places[place].cell(pos.x, pos.y);
        let growth = self.growth(place, pos);
        Some(match cell {
            Cell::Ground => {
                if growth >= GROWTH_OVERGROWN {
                    Surface::Brush
                } else if growth >= GROWTH_TALL {
                    Surface::TallGrass
                } else {
                    match self.wear(place, pos) {
                        w if w >= WEAR_PATH => Surface::Cowpath,
                        w if w >= WEAR_TRACK => Surface::Track,
                        _ => Surface::Grass,
                    }
                }
            }
            Cell::Path => {
                if growth >= GROWTH_OVERGROWN {
                    Surface::OvergrownPath
                } else if growth >= GROWTH_TALL {
                    Surface::WeedyPath
                } else {
                    Surface::Path
                }
            }
            _ => return None,
        })
    }

    /// A step onto a tile: adds wear on ground and tramples growth on ground and paths.
    pub fn tread(&mut self, world: &World, place: PlaceId, pos: Pos) {
        let cell = world.places[place.index()].cell(pos.x, pos.y);
        let Some(t) = self.places.get_mut(&place.0) else {
            return;
        };
        let i = t.index(pos);
        match cell {
            Cell::Ground => {
                t.wear[i] = t.wear[i].saturating_add(1);
                t.growth[i] = t.growth[i].saturating_sub(TRAMPLE);
            }
            Cell::Path => t.growth[i] = t.growth[i].saturating_sub(TRAMPLE),
            _ => {}
        }
    }

    /// Clears growth within `radius` (Chebyshev) of `at`. Returns how many tiles changed.
    pub fn clear(&mut self, world: &World, place: PlaceId, at: Pos, radius: u16) -> u16 {
        let p = &world.places[place.index()];
        let Some(t) = self.places.get_mut(&place.0) else {
            return 0;
        };
        let mut changed = 0;
        for y in at.y.saturating_sub(radius)..=(at.y + radius).min(p.height - 1) {
            for x in at.x.saturating_sub(radius)..=(at.x + radius).min(p.width - 1) {
                if !matches!(p.cell(x, y), Cell::Ground | Cell::Path) {
                    continue;
                }
                let i = t.index(Pos { x, y });
                if t.growth[i] > 0 {
                    t.growth[i] = 0;
                    changed += 1;
                }
            }
        }
        changed
    }

    /// A day passes: wear fades, everything that grows grows.
    pub fn new_day(&mut self, world: &World) {
        for (&place, t) in &mut self.places {
            let p = &world.places[usize::from(place)];
            for y in 0..p.height {
                for x in 0..p.width {
                    let i = t.index(Pos { x, y });
                    match p.cell(x, y) {
                        Cell::Ground => {
                            t.wear[i] = t.wear[i].saturating_sub(WEAR_FADE_PER_DAY);
                            t.growth[i] =
                                t.growth[i].saturating_add(GROWTH_PER_DAY).min(GROWTH_MAX);
                        }
                        Cell::Path => {
                            t.growth[i] =
                                t.growth[i].saturating_add(GROWTH_PER_DAY).min(GROWTH_MAX);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Every outdoor tile's surface, row-major, for the viewer.
    #[must_use]
    pub fn view(&self, world: &World, place: usize) -> TerrainView {
        let p = &world.places[place];
        let tiles = (0..p.height)
            .flat_map(|y| (0..p.width).map(move |x| Pos { x, y }))
            .map(|pos| self.surface(world, place, pos))
            .collect();
        TerrainView {
            place: PlaceId(u16::try_from(place).expect("few places")),
            width: p.width,
            height: p.height,
            tiles,
        }
    }

    /// Every tile in a place that needs clearing, row-major.
    #[must_use]
    pub fn overgrown_tiles(&self, world: &World, place: usize) -> Vec<Pos> {
        let p = &world.places[place];
        let mut out = Vec::new();
        for y in 0..p.height {
            for x in 0..p.width {
                let pos = Pos { x, y };
                if self
                    .surface(world, place, pos)
                    .is_some_and(Surface::overgrown)
                {
                    out.push(pos);
                }
            }
        }
        out
    }
}
