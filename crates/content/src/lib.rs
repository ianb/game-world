//! Town content: zones, buildings, villagers, as serde data loaded from JSON, validated, and
//! compiled into tile grids the sim walks on.
//!
//! Source coordinates are tiles. Buildings are authored in their own local coordinates and
//! placed in a zone by `at`. See `docs/design/world-layout.md`.

pub mod compile;
pub mod load;
pub mod model;
pub mod nav;
pub mod render;
pub mod validate;

pub use compile::{Area, Cell, CompiledDoor, CompiledObject, Place, PlaceKind, World};
pub use load::{LoadError, load_dir, load_str};
pub use model::*;
pub use nav::{Leg, Nav, Route, Spot, WALK_TICKS_PER_TILE};
pub use render::{list_doors, list_objects, render_place};
pub use validate::{ContentError, validate};
