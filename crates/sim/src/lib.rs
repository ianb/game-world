//! The deterministic simulation core.
//!
//! Rules: no wall clock, no ambient randomness, no floats, no hash maps. State is a pure
//! function of (content, seed, inputs). See `docs/decisions.md`.
#![deny(clippy::float_arithmetic)]

pub mod brain;
pub mod event;
pub mod hash;
pub mod ids;
pub mod input;
pub mod protocol;
pub mod queue;
pub mod rng;
pub mod sim;
pub mod snapshot;
pub mod terrain;
pub mod time;
pub mod views;
pub mod villager;

pub use brain::{CharacterSet, Command, Ctx, DataCharacters, Planned, Target, Trigger};
pub use sim::{Sim, SimError, SimState};
pub use time::Tick;
