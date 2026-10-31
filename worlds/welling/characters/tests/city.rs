//! The Welling's scrap zone as a whole: its content, its golden hash, and the trades that
//! move goods between the vent, the Mire, the stall and the fishers' tables.
use content::validate::Severity;
use sim::Sim;
use sim::protocol::Emitted;
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
    assert!(
        s.stock("table_nessa", "cooked-fish") > 0,
        "Nessa cooked nothing on day 0"
    );
    assert_eq!(
        s.stock("table_grue", "cooked-fish"),
        0,
        "Grue does not cook"
    );
}
