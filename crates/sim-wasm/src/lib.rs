//! The wasm facade: one `Sim` handle, typed events out, typed inputs in, bytes for
//! snapshots. The TypeScript side never reaches past this file.
use wasm_bindgen::prelude::*;

/// Protocol version asserted by the TypeScript wrapper at construction.
#[wasm_bindgen]
#[must_use]
pub fn protocol_version() -> u32 {
    0
}
