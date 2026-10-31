//! The Welling's scrap zone as a whole: its content, its golden hash, and the trades that
//! move goods between the vent, the Mire, the stall and the fishers' tables.
use content::validate::Severity;
use sim::Sim;
use sim::protocol::{Emitted, Odd};
use sim::time::{TICKS_PER_DAY, Tick};
use welling::City;
use welling::scene::{Scene, at, city_content};

#[test]
fn the_content_has_no_errors() {
    let problems = content::validate(&city_content());
    let errors: Vec<String> = problems
        .iter()
        .filter(|p| p.severity == Severity::Error)
        .map(ToString::to_string)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn golden_hash_after_a_week_matches_the_wasm_smoke() {
    use sim::input::{Input, InputKind, Source};
    let mut sim: Sim<City> = Sim::new(2026, city_content());
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
fn everyone_is_in_bed_at_three_in_the_morning() {
    let mut s = Scene::new();
    s.run_until(at(1, 3, 0));
    for v in &s.sim.content().villagers {
        assert_eq!(s.activity(&v.id), "sleep", "{} is up", v.id);
    }
}

#[test]
fn sab_captures_raw_gas_at_the_hood() {
    let mut s = Scene::new();
    s.run_until(at(0, 5, 59));
    let before = s.stock("casks_mire", "raw-gas");
    s.run_until(at(0, 10, 0));
    assert_eq!(s.activity("sab"), "capture");
    assert!(
        s.stock("casks_mire", "raw-gas") > before,
        "the casks did not fill: {before} then {}",
        s.stock("casks_mire", "raw-gas")
    );
}

#[test]
fn the_refiners_turn_raw_gas_into_refined() {
    let mut s = Scene::new();
    s.run_until(at(0, 13, 0));
    let ora = s.id("ora");
    let by_ora = |good: &str, delta: i32| {
        s.events
            .iter()
            .filter(|e| {
                matches!(e, Emitted::Stocked { who, object, good: g, delta: d, .. }
                    if *who == ora && *d == delta && g == good
                        && s.sim.world().objects[object.index()].id == "casks_mire")
            })
            .count()
    };
    let spells = by_ora("refined-gas", 1);
    assert!(spells > 0, "Ora refined nothing by 1pm");
    assert_eq!(by_ora("raw-gas", -2), spells, "two raw for each refined");
}

#[test]
fn tully_brings_refined_gas_up_to_the_stall_and_pays_ora() {
    let mut s = Scene::new();
    s.run_until(at(0, 8, 29));
    let (tully, ora) = (
        s.sim.ctx().coins(s.id("tully")),
        s.sim.ctx().coins(s.id("ora")),
    );
    let stall = s.stock("stall_gas", "refined-gas");
    s.run_until(at(0, 9, 30));
    let moved = s.stock("stall_gas", "refined-gas") - stall;
    assert!(moved > 0, "nothing reached the stall");
    let paid = u32::from(moved) * welling::toolbox::GAS_TRADE_PRICE;
    assert!(s.sim.ctx().coins(s.id("tully")) <= tully - paid);
    assert!(s.sim.ctx().coins(s.id("ora")) >= ora + paid);
    assert_eq!(s.activity("tully"), "sell");
}

#[test]
fn nessa_cooks_her_catch_and_grue_eats_his_raw() {
    let mut s = Scene::new();
    s.run_until(at(1, 0, 0));
    let nessa = s.id("nessa");
    let cooked = s.events.iter().any(|e| {
        matches!(e, Emitted::Stocked { who, good, delta, .. }
            if *who == nessa && good == "cooked-fish" && *delta > 0)
    });
    assert!(cooked, "Nessa cooked nothing on day 0");
    assert!(
        took(&s, "cooked-fish").any(|t| t.0 == "nessa"),
        "Nessa ate none"
    );
    assert!(
        took(&s, "fish").any(|t| t.0 == "grue"),
        "Grue ate no raw fish"
    );
    assert!(
        !took(&s, "fish").any(|t| t.0 == "nessa"),
        "Nessa ate raw fish"
    );
}

/// Who took `good`, with the batch and how it went, in order.
fn took<'a>(s: &'a Scene, good: &'a str) -> impl Iterator<Item = (String, Option<u32>, Odd)> + 'a {
    s.events.iter().filter_map(move |e| match e {
        Emitted::Consumed {
            who,
            good: g,
            batch,
            odd,
            ..
        } if g == good => Some((
            s.sim.state().villagers[who.index()].id.clone(),
            *batch,
            *odd,
        )),
        _ => None,
    })
}

#[test]
fn everyone_chews_kelp() {
    let mut s = Scene::new();
    s.run_until(at(1, 0, 0));
    let chewers: std::collections::BTreeSet<String> = took(&s, "kelp").map(|t| t.0).collect();
    assert!(chewers.len() >= 6, "only {chewers:?} chewed kelp on day 0");
}

#[test]
fn pike_buys_gas_at_the_stall_and_the_batch_goes_back_to_a_refiner() {
    let mut s = Scene::new();
    s.run_until(at(0, 18, 0));
    let (_, batch, _) = took(&s, "refined-gas")
        .find(|t| t.0 == "pike")
        .expect("Pike took no gas by 6pm");
    let paid = s.events.iter().any(|e| {
        matches!(e, Emitted::Paid { from, to, amount, .. }
            if *from == Some(s.id("pike")) && *to == Some(s.id("tully"))
                && *amount == welling::toolbox::GAS_PRICE)
    });
    assert!(paid, "Pike did not pay Tully for it");
    // The gas the stall started with was made by nobody; whatever Pike took, its batch is
    // known, so it can be traced.
    let batch = batch.expect("the dose has a batch");
    let made = &s.sim.state().batches.made[batch as usize];
    let by = made
        .by
        .map(|v| s.sim.state().villagers[v.index()].id.clone());
    assert!(
        by.is_none() || matches!(by.as_deref(), Some("ora" | "sab")),
        "batch {batch} made by {by:?}"
    );
}

#[test]
fn a_bad_batch_of_raw_fish_reaches_whoever_eats_it() {
    let mut s = Scene::new();
    s.force_for(sim::substances::BAD_BATCH, "grue", true);
    s.run_until(at(2, 0, 0));
    let (_, batch, odd) = took(&s, "fish")
        .find(|t| t.0 == "grue" && t.2 == Odd::BadBatch)
        .expect("Grue ate no fish from a bad batch");
    let made = &s.sim.state().batches.made[batch.expect("a batch") as usize];
    assert!(made.bad);
    assert_eq!(made.by, Some(s.id("grue")), "his own catch");
    assert_eq!(odd, Odd::BadBatch);
    let lowered = s.events.iter().any(|e| {
        matches!(e, Emitted::Consumed { who, good, lift, .. }
            if *who == s.id("grue") && good == "fish" && *lift < 0)
    });
    assert!(lowered, "the bad fish did not lower his spirits");
}

#[test]
fn cooked_fish_has_no_surprises() {
    let mut s = Scene::new();
    s.force(sim::substances::ODD_PORTION, true);
    s.run_until(at(1, 0, 0));
    assert!(took(&s, "cooked-fish").all(|t| t.2 == Odd::No));
}

#[test]
fn the_boat_takes_refined_gas_and_brings_kelp_for_the_kettle() {
    let mut s = Scene::new();
    // Day 0 is a Monday; the boat calls on Tuesday at ten.
    s.run_until(at(1, 15, 0));
    let came = s.events.iter().any(|e| {
        matches!(e, Emitted::TrainArrived { delivered, .. }
            if delivered.iter().any(|g| g.good == "kelp"))
    });
    assert!(came, "no boat with kelp on Tuesday");
    let paid = s.events.iter().any(|e| {
        matches!(e, Emitted::Paid { to, what, .. }
            if *to == Some(s.id("ora")) && what.ends_with("refined-gas by boat"))
    });
    assert!(paid, "the boat paid Ora nothing for gas");
    let lark = s.id("lark");
    let restocked = s.events.iter().any(|e| {
        matches!(e, Emitted::Stocked { who, good, delta, object, .. }
            if *who == lark && good == "kelp" && *delta > 0
                && s.sim.world().objects[object.index()].id == "counter_kettle")
    });
    assert!(restocked, "Lark did not restock the Kettle's kelp");
}
