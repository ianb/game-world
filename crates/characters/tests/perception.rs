//! What villagers perceive, what they tell themselves, and what it builds to.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::protocol::{Act, Blame, Emitted, MoveKind};
use sim::time::{TICKS_PER_DAY, Tick};
use std::path::PathBuf;

fn town() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap()
}

#[test]
fn the_town_keeps_what_is_worth_knowing_with_who_could_perceive_it() {
    let mut sim: Sim<Town> = Sim::new(1, town());
    sim.step_until(Tick(3 * TICKS_PER_DAY)).unwrap();
    let record = &sim.state().record;
    assert!(!record.entries.is_empty());
    assert!(
        record
            .entries
            .iter()
            .all(|r| !matches!(r.event, Emitted::Moved { .. } | Emitted::Decided { .. })),
        "footsteps and reasons are not kept"
    );
    assert!(
        record.entries.iter().any(|r| !r.saw.is_empty()),
        "somebody saw something"
    );
    // Day-long things from the first day are gone by the third.
    assert!(
        record.entries.iter().all(|r| {
            r.keep == sim::recall::Keep::Month || r.event.tick() >= Tick(TICKS_PER_DAY)
        })
    );
    // Thoughts are private.
    assert!(
        record
            .entries
            .iter()
            .filter(|r| matches!(r.event, Emitted::Thought { .. }))
            .all(|r| r.saw.is_empty() && r.heard.is_empty())
    );
}

#[test]
fn tom_blames_the_town_and_it_boils_over() {
    let mut sim: Sim<Town> = Sim::new(2, town());
    let events = sim.step_until(Tick(28 * TICKS_PER_DAY)).unwrap();
    let tom = sim.ctx().villager_id("tom").unwrap();
    let blamed_world = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::Thought { who, thought, .. }
                if *who == tom && thought.blame == Blame::World && thought.lasting && thought.everywhere)
        })
        .count();
    assert!(blamed_world > 0);
    // An outburst is a dig he meant, not one that slipped out.
    let digs = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::Beat { who, meant: Act::Move { kind: MoveKind::Jab, .. }, did, .. }
                if *who == tom && matches!(did, Act::Move { kind: MoveKind::Jab, .. }))
        })
        .count();
    assert!(digs >= 1);
    assert_eq!(usize::from(sim.state().characters.tom.outbursts), digs);
}
