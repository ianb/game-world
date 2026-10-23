//! The committed event fixture must match a fresh native run. vitest checks the same file
//! against the wasm build, so both sides prove the same bytes.
use content::load_dir;
use sim::protocol::Emitted;
use sim::time::{TICKS_PER_DAY, Tick};
use sim::{DataCharacters, Sim};
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct Fixture {
    seed: u64,
    day: u32,
    protocol_version: u32,
    hash: String,
    events: Vec<Emitted>,
}

#[test]
fn day0_fixture_matches_native_run() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("fixtures/day0-seed1.json")).unwrap();
    let fixture: Fixture = serde_json::from_str(&text).unwrap();
    assert_eq!(
        fixture.protocol_version,
        sim::protocol::PROTOCOL_VERSION,
        "regenerate the fixture: sim-tools events > fixtures/day0-seed1.json"
    );
    let mut sim: Sim<DataCharacters> =
        Sim::new(fixture.seed, load_dir(&root.join("content")).unwrap());
    let start = Tick(fixture.day * TICKS_PER_DAY);
    sim.step_until(start).unwrap();
    let events = sim.step_until(start.plus(TICKS_PER_DAY)).unwrap();
    assert_eq!(events.len(), fixture.events.len());
    assert_eq!(events, fixture.events);
    assert_eq!(sim.hash().hex(), fixture.hash);
}
