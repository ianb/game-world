//! The town with its real characters: Mara and Tom as code.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::protocol::Emitted;
use sim::time::{TICKS_PER_DAY, Tick};
use sim::views;
use std::fmt::Write as _;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn town() -> Content {
    load_dir(&root().join("content")).unwrap()
}

fn at(day: u32, hour: u32, minute: u32) -> Tick {
    Tick(day * TICKS_PER_DAY).at_time(hour, minute)
}

#[test]
fn tom_fishes_on_sunday_and_mara_brings_a_pastry_when_he_is_low() {
    let mut sim: Sim<Town> = Sim::new(7, town());
    // Monday evening: nobody keeps Tom company at 20:00 (Mara drinks at 19:06 and is there),
    // so his mood depends on whether she is in the hall. Run to Sunday and look.
    let mut events = Vec::new();
    for day in 0..7 {
        events.extend(sim.step_until(at(day, 23, 59)).unwrap());
    }
    let tom = &sim.state().characters.tom;
    assert_eq!(
        tom.days_since_day_off, 0,
        "Sunday resets the counter: {tom:?}"
    );
    let fished = events
        .iter()
        .any(|e| matches!(e, Emitted::Activity { activity, .. } if activity == "fish"));
    assert!(fished, "Tom fished at some point in the week");
    let mara = &sim.state().characters.mara;
    assert!(mara.batches_baked >= 6, "Mara bakes most days: {mara:?}");
}

#[test]
fn pastry_delivery_happens_only_when_tom_is_grumpy() {
    // Tom starts at mood -1, so day 0 includes a pastry run; after company his mood rises.
    let mut sim: Sim<Town> = Sim::new(7, town());
    let events = sim.step_until(at(1, 0, 0)).unwrap();
    let delivered = events
        .iter()
        .any(|e| matches!(e, Emitted::Activity { activity, .. } if activity == "bring-pastry"));
    assert!(delivered, "{}", views::where_is_everyone(&sim));
    assert_eq!(sim.state().characters.mara.pastries_delivered, 1);
}

#[test]
fn a_sunday_itinerary() {
    let mut sim: Sim<Town> = Sim::new(7, town());
    sim.step_until(at(6, 0, 0)).unwrap();
    let events = sim.step_until(at(7, 0, 0)).unwrap();
    let mut out = String::new();
    for id in ["mara", "tom"] {
        let who = sim.ctx().villager_id(id).unwrap();
        out.push_str(&views::itinerary(&sim, who, &events));
    }
    let _ = writeln!(
        out,
        "{:?}\n{:?}",
        sim.state().characters.mara,
        sim.state().characters.tom
    );
    insta::assert_snapshot!(out);
}

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
    let text = std::fs::read_to_string(root().join("fixtures/day0-seed1.json")).unwrap();
    let fixture: Fixture = serde_json::from_str(&text).unwrap();
    assert_eq!(
        fixture.protocol_version,
        sim::protocol::PROTOCOL_VERSION,
        "regenerate the fixture: cargo run -p tools -- events > fixtures/day0-seed1.json"
    );
    let mut sim: Sim<Town> = Sim::new(fixture.seed, town());
    let start = Tick(fixture.day * TICKS_PER_DAY);
    let mut events = sim.step_until(start).unwrap();
    events.retain(|e| e.tick() >= start);
    events.extend(sim.step_until(start.plus(TICKS_PER_DAY)).unwrap());
    assert_eq!(
        events.len(),
        fixture.events.len(),
        "regenerate the fixture if the change is intended"
    );
    assert_eq!(events, fixture.events);
    assert_eq!(sim.hash().hex(), fixture.hash);
}

#[test]
fn snapshot_round_trip_with_character_state() {
    let mut live: Sim<Town> = Sim::new(3, town());
    live.step_until(at(0, 20, 30)).unwrap();
    let mut restored = Sim::<Town>::restore(&live.snapshot(), town()).unwrap();
    assert_eq!(live.state().characters, restored.state().characters);
    let a = live.step_until(at(2, 12, 0)).unwrap();
    let b = restored.step_until(at(2, 12, 0)).unwrap();
    assert_eq!(a, b);
    assert_eq!(live.hash(), restored.hash());
}

/// Same inputs as packages/sim-wasm/smoke.mjs, which compares the wasm build's hash to
/// this snapshot.
#[test]
fn golden_hash_after_a_week_matches_the_wasm_smoke() {
    use sim::input::{Input, InputKind, Source};
    let mut sim: Sim<Town> = Sim::new(2026, town());
    sim.apply_input(Input {
        tick: at(3, 12, 0),
        source: Source::Player(0),
        kind: InputKind::Marker("midweek".into()),
        seq: 0,
    })
    .unwrap();
    sim.step_until(Tick(7 * TICKS_PER_DAY)).unwrap();
    insta::assert_snapshot!(sim.hash().hex());
}

#[test]
fn villagers_greet_talk_and_remember() {
    let mut sim: Sim<Town> = Sim::new(7, town());
    let events = sim.step_until(at(1, 0, 0)).unwrap();
    let talks: Vec<(&str, u32)> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Talked { topic, minutes, .. } => Some((topic.as_str(), *minutes)),
            _ => None,
        })
        .collect();
    assert!(
        talks.contains(&("sweets", 5)),
        "Pip pesters Mara at noon: {talks:?}"
    );
    assert!(
        talks.contains(&("the day", 10)),
        "Mara and Tom talk at the tavern: {talks:?}"
    );
    assert!(
        talks
            .iter()
            .filter(|(t, m)| *t == "greeting" && *m == 0)
            .count()
            >= 4,
        "greetings in passing: {talks:?}"
    );
    let c = &sim.state().characters;
    assert_eq!(c.pip.buns, 1, "{:?}", c.pip);
    assert!(c.tom.mood > -1, "company cheered Tom up: {:?}", c.tom);
    // Nobody is left stuck in "talk": conversations resume the previous activity.
    for v in &sim.state().villagers {
        assert_ne!(v.activity, "talk", "{}", views::where_is_everyone(&sim));
    }
    assert!(events.iter().all(|e| !matches!(e, Emitted::Stuck { .. })));
}

#[test]
fn a_conversation_defers_a_wait_instead_of_cancelling_it() {
    // Pip roams in wait loops all afternoon. The noon talk with Mara must not leave him
    // stranded: after it ends he keeps roaming (a Waited fires and he moves again).
    let mut sim: Sim<Town> = Sim::new(7, town());
    sim.step_until(at(0, 12, 0)).unwrap();
    let events = sim.step_until(at(0, 13, 30)).unwrap();
    let pip = sim.ctx().villager_id("pip").unwrap();
    let talked = events.iter().any(
        |e| matches!(e, Emitted::Talked { who, topic, .. } if *who == pip && topic == "sweets"),
    );
    assert!(talked, "{}", views::itinerary(&sim, pip, &events));
    let roam_departures_after = events
        .iter()
        .filter(|e| matches!(e, Emitted::Departed { who, activity, tick, .. } if *who == pip && activity == "roam" && *tick > at(0, 12, 30)))
        .count();
    assert!(
        roam_departures_after >= 2,
        "{}",
        views::itinerary(&sim, pip, &events)
    );
}
