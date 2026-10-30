//! The committed town, run through the sim with the baseline data-driven characters.
use content::{Content, load_dir};
use proptest::prelude::*;
use sim::input::{Input, InputKind, Source};
use sim::protocol::Emitted;
use sim::time::{TICKS_PER_DAY, Tick};
use sim::views;
use sim::{DataCharacters, Sim, SimError};
use std::path::PathBuf;

fn town() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap()
}

fn new_sim(seed: u64) -> Sim<DataCharacters> {
    Sim::new(seed, town())
}

fn at(day: u32, hour: u32, minute: u32) -> Tick {
    Tick(day * TICKS_PER_DAY).at_time(hour, minute)
}

fn villager(sim: &Sim<DataCharacters>, id: &str) -> sim::ids::VillagerId {
    sim.ctx().villager_id(id).unwrap()
}

fn marker(tick: Tick, text: &str) -> Input {
    Input {
        tick,
        source: Source::Player(0),
        kind: InputKind::Marker(text.into()),
        seq: 0,
    }
}

fn teleport(
    tick: Tick,
    source: Source,
    who: sim::ids::VillagerId,
    place: u16,
    x: u16,
    y: u16,
) -> Input {
    Input {
        tick,
        source,
        kind: InputKind::Teleport {
            who,
            place: sim::ids::PlaceId(place),
            x,
            y,
        },
        seq: 0,
    }
}

#[test]
fn villagers_start_asleep_by_their_beds() {
    let sim = new_sim(1);
    let text = views::where_is_everyone(&sim);
    assert!(text.contains("house/upstairs"), "{text}");
    assert!(sim.state().villagers.iter().all(|v| v.activity == "sleep"));
}

#[test]
fn mara_reaches_the_bakery_counter_and_everyone_sleeps_at_night() {
    let mut sim = new_sim(1);
    let events = sim.step_until(at(0, 9, 30)).unwrap();
    let mara = villager(&sim, "mara");
    let m = &sim.state().villagers[mara.index()];
    assert_eq!(
        m.activity,
        "tend-shop",
        "{}",
        views::where_is_everyone(&sim)
    );
    let counter = sim
        .world()
        .objects
        .iter()
        .find(|o| o.id == "counter_bakery")
        .unwrap();
    assert_eq!(m.place.index(), counter.place);
    assert_eq!(
        m.pos.x.abs_diff(counter.at.x) + m.pos.y.abs_diff(counter.at.y),
        1,
        "stands next to the counter"
    );
    assert!(
        events.iter().all(|e| !matches!(e, Emitted::Stuck { .. })),
        "nobody got stuck"
    );

    // Half past midnight: the latest bedtime is half past eleven.
    sim.step_until(at(1, 0, 30)).unwrap();
    for v in &sim.state().villagers {
        assert_eq!(v.activity, "sleep", "{}", views::where_is_everyone(&sim));
        // Everyone is in the place their bed is in.
        let bed = sim
            .world()
            .objects
            .iter()
            .find(|o| o.kind == "bed" && o.owner.as_deref() == Some(v.id.as_str()))
            .unwrap_or_else(|| panic!("{} has no bed", v.name));
        assert_eq!(v.place.index(), bed.place, "{} sleeps by their bed", v.name);
    }
}

#[test]
fn every_step_is_to_an_adjacent_passable_tile_or_through_a_door() {
    let mut sim = new_sim(3);
    let events = sim.step_until(at(1, 0, 0)).unwrap();
    let world = sim.world();
    let mut last: Vec<Option<(u16, u16, u16)>> = vec![None; sim.state().villagers.len()];
    for e in &events {
        if let Emitted::Moved {
            who, place, x, y, ..
        } = e
        {
            assert!(
                world.passable(place.index(), *x, *y),
                "moved onto an impassable tile"
            );
            if let Some((lp, lx, ly)) = last[who.index()] {
                if lp == place.0 {
                    let d = lx.abs_diff(*x) + ly.abs_diff(*y);
                    assert!(
                        d <= 1,
                        "jumped {d} tiles inside {}",
                        world.places[place.index()].id
                    );
                } else {
                    let crossed = world.doors.iter().any(|dr| {
                        (dr.from_place == usize::from(lp)
                            && dr.from.x == lx
                            && dr.from.y == ly
                            && dr.to_place == place.index()
                            && dr.to.x == *x
                            && dr.to.y == *y)
                            || (dr.to_place == usize::from(lp)
                                && dr.to.x == lx
                                && dr.to.y == ly
                                && dr.from_place == place.index()
                                && dr.from.x == *x
                                && dr.from.y == *y)
                    });
                    assert!(crossed, "changed place without a door");
                }
            }
            last[who.index()] = Some((place.0, *x, *y));
        }
    }
}

#[test]
fn itinerary_and_afternoon_snapshot() {
    let mut sim = new_sim(1);
    let mut events = sim.step_until(at(0, 14, 0)).unwrap();
    let mut out = String::new();
    out.push_str(&views::where_is_everyone(&sim));
    out.push('\n');
    for i in 0..sim.world().places.len() {
        out.push_str(&views::render_place_now(&sim, i));
    }
    events.extend(sim.step_until(at(1, 0, 0)).unwrap());
    for id in ["mara", "tom"] {
        let who = villager(&sim, id);
        out.push('\n');
        out.push_str(&views::itinerary(&sim, who, &events));
        out.push_str(&views::place_changes(&sim, who, &events));
    }
    insta::assert_snapshot!(out);
}

#[test]
fn weekend_entries_override_daily_ones() {
    // Day index 5 is Saturday. Mara wanders instead of tending the shop at 14:00.
    let mut sim = new_sim(1);
    sim.step_until(at(5, 14, 30)).unwrap();
    let mara = villager(&sim, "mara");
    let m = &sim.state().villagers[mara.index()];
    assert!(
        m.activity == "wander" || m.activity == "walking",
        "{}",
        views::where_is_everyone(&sim)
    );
}

#[test]
fn inputs_apply_in_order_and_bad_ones_are_rejected() {
    let mut sim = new_sim(1);
    sim.apply_input(marker(Tick(100), "b")).unwrap();
    sim.apply_input(marker(Tick(50), "a")).unwrap();
    let events = sim.step_until(Tick(100)).unwrap();
    let applied: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::InputApplied { summary, .. } => Some(summary.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(applied, vec!["marker \"a\"", "marker \"b\""]);
    assert_eq!(
        sim.apply_input(marker(Tick(5), "late")),
        Err(SimError::InputInPast {
            input: Tick(5),
            current: Tick(100)
        })
    );
    let tom = villager(&sim, "tom");
    sim.apply_input(teleport(Tick(101), Source::Player(3), tom, 0, 20, 20))
        .unwrap();
    let events = sim.step_until(Tick(101)).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Emitted::InputRejected { .. }))
    );
}

#[test]
fn snapshot_round_trip_and_content_check() {
    let mut live = new_sim(9);
    live.step_until(at(0, 8, 0)).unwrap();
    let bytes = live.snapshot();
    let mut restored = Sim::<DataCharacters>::restore(&bytes, town()).unwrap();
    assert_eq!(live.hash(), restored.hash());
    let a = live.step_until(at(0, 20, 0)).unwrap();
    let b = restored.step_until(at(0, 20, 0)).unwrap();
    assert_eq!(a, b);
    assert_eq!(live.hash(), restored.hash());

    let mut other = town();
    other.villagers[0].name = "Marguerite".into();
    assert_eq!(
        Sim::<DataCharacters>::restore(&bytes, other).err(),
        Some(SimError::ContentMismatch)
    );
}

#[test]
fn golden_hash_after_a_week() {
    let mut sim = new_sim(2026);
    sim.apply_input(marker(at(3, 12, 0), "midweek")).unwrap();
    sim.step_until(Tick(7 * TICKS_PER_DAY)).unwrap();
    insta::assert_snapshot!(sim.hash().hex());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn same_seed_gives_same_hash(seed in any::<u64>(), stop_minutes in 0u32..(2 * 24 * 60)) {
        let run = || {
            let mut sim = new_sim(seed);
            sim.step_until(Tick(stop_minutes * 60)).unwrap();
            sim.hash()
        };
        prop_assert_eq!(run(), run());
    }

    #[test]
    fn snapshot_then_resume_matches_straight_run(seed in any::<u64>(), t in 0u32..(24 * 60), k in 0u32..(24 * 60)) {
        let mut straight = new_sim(seed);
        straight.step_until(Tick((t + k) * 60)).unwrap();
        let mut paused = new_sim(seed);
        paused.step_until(Tick(t * 60)).unwrap();
        let mut resumed = Sim::<DataCharacters>::restore(&paused.snapshot(), town()).unwrap();
        resumed.step_until(Tick((t + k) * 60)).unwrap();
        prop_assert_eq!(straight.hash(), resumed.hash());
    }
}

#[test]
fn teleport_onto_a_wall_is_rejected_and_onto_ground_moves() {
    let mut sim = new_sim(1);
    let tom = villager(&sim, "tom");
    sim.apply_input(teleport(Tick(10), Source::System, tom, 0, 4, 4))
        .unwrap(); // a wall tile
    sim.apply_input(teleport(Tick(10), Source::System, tom, 0, 20, 20))
        .unwrap();
    let events = sim.step_until(Tick(10)).unwrap();
    assert!(events.iter().any(|e| matches!(e, Emitted::InputRejected { reason, .. } if reason.contains("not a passable tile"))));
    let t = &sim.state().villagers[tom.index()];
    assert_eq!((t.place.0, t.pos.x, t.pos.y), (0, 20, 20));
}

#[test]
fn input_order_does_not_depend_on_when_inputs_were_submitted() {
    // Early: all inputs queued before any internal event for their ticks exists.
    let mut early = new_sim(5);
    for t in [at(0, 6, 5), at(0, 9, 0), at(0, 13, 1)] {
        early.apply_input(marker(t, "m")).unwrap();
    }
    early.step_until(at(1, 0, 0)).unwrap();
    // Late: each input queued moments before its tick, after the day's events are queued.
    let mut late = new_sim(5);
    for t in [at(0, 6, 5), at(0, 9, 0), at(0, 13, 1)] {
        late.step_until(t.minus(1)).unwrap();
        late.apply_input(marker(t, "m")).unwrap();
    }
    late.step_until(at(1, 0, 0)).unwrap();
    assert_eq!(
        early.hash(),
        late.hash(),
        "{:?}",
        early.hash().first_difference(&late.hash())
    );
    // And a replay of the recorded log from scratch matches too.
    let mut replay = new_sim(5);
    for input in &early.state().inputs {
        replay
            .apply_input(Input {
                seq: 0,
                ..input.clone()
            })
            .unwrap();
    }
    replay.step_until(at(1, 0, 0)).unwrap();
    assert_eq!(early.hash(), replay.hash());
}

#[test]
fn eta_matches_arrival() {
    let mut sim = new_sim(2);
    let events = sim.step_until(at(1, 0, 0)).unwrap();
    let mut pending: std::collections::BTreeMap<u16, Tick> = std::collections::BTreeMap::new();
    for e in &events {
        match e {
            Emitted::Departed { who, eta, .. } => {
                pending.insert(who.0, *eta);
            }
            Emitted::Activity {
                who,
                activity,
                tick,
                ..
            } if activity != "walking" => {
                if let Some(eta) = pending.remove(&who.0) {
                    assert_eq!(
                        *tick,
                        eta,
                        "{} arrived at {tick} but eta was {eta}",
                        sim.state().villagers[who.index()].name
                    );
                }
            }
            _ => {}
        }
    }
    assert!(pending.is_empty(), "every departure arrived");
}

/// Two planned items at the same minute both run.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Collider;

impl sim::CharacterSet for Collider {
    const LAYOUT_VERSION: u32 = 1;
    fn new(_: &Content, _: &content::World) -> Self {
        Collider
    }
    fn plan_day(
        &mut self,
        _who: sim::ids::VillagerId,
        _ctx: &sim::Ctx<'_>,
        _rng: &mut rand_pcg::Pcg32,
    ) -> Vec<sim::Planned> {
        vec![
            sim::Planned {
                at: sim::time::TimeOfDay::new(8, 0, 0),
                command: sim::Command::Note("first".into()),
            },
            sim::Planned {
                at: sim::time::TimeOfDay::new(8, 0, 0),
                command: sim::Command::Wait {
                    ticks: 60,
                    activity: "second".into(),
                },
            },
        ]
    }
}

#[test]
fn planned_items_at_the_same_minute_both_run() {
    let mut sim: Sim<Collider> = Sim::new(1, town());
    let events = sim.step_until(at(0, 8, 1)).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Emitted::Activity { activity, .. } if activity == "second"))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Emitted::Noted { text, .. } if text == "first"))
    );
}

#[test]
fn ground_wears_grows_and_is_cleared() {
    use content::Pos;
    use sim::protocol::Surface;
    use sim::terrain::{GROWTH_OVERGROWN, GROWTH_TALL};
    let mut sim = new_sim(7);
    let townp = sim.world().place_index["town"];
    // A corner nobody visits: grass goes tall after a week and to brush after four.
    let corner = Pos { x: 45, y: 3 };
    assert_eq!(sim.surface(townp, corner), Some(Surface::Grass));
    sim.step_until(Tick(u32::from(GROWTH_TALL) * TICKS_PER_DAY))
        .unwrap();
    assert_eq!(sim.surface(townp, corner), Some(Surface::TallGrass));
    sim.step_until(Tick(u32::from(GROWTH_OVERGROWN) * TICKS_PER_DAY))
        .unwrap();
    assert_eq!(sim.surface(townp, corner), Some(Surface::Brush));
    // The street outside the bakery is walked every day and stays a clean path.
    assert_eq!(
        sim.surface(townp, Pos { x: 62, y: 47 }),
        Some(Surface::Path)
    );
    // Indoors has no surface.
    let upstairs = sim.world().place_index["house/upstairs"];
    assert_eq!(sim.surface(upstairs, Pos { x: 2, y: 2 }), None);
    // Clearing resets growth around a tile and reports how many tiles changed.
    let world = sim.world().clone();
    let mut terrain = sim.terrain().clone();
    let place = sim::ids::PlaceId(u16::try_from(townp).unwrap());
    let changed = terrain.clear(&world, place, corner, 1);
    assert_eq!(changed, 9, "the 3x3 block around the corner is all grass");
    assert_eq!(terrain.surface(&world, townp, corner), Some(Surface::Grass));
}
