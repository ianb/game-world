//! One module per villager, plus the shared toolbox. Character code reads through `Ctx`
//! and writes by emitting commands; it never mutates world state directly.
#![deny(clippy::float_arithmetic)]
