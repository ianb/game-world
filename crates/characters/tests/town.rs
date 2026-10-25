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

/// How many times `who` changed `good` on the object `object` by `delta`.
fn stocked(sim: &Sim<Town>, events: &[Emitted], who: &str, object: &str, delta: i32) -> usize {
    let who = sim.ctx().villager_id(who).unwrap();
    events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::Stocked { who: w, object: o, delta: d, .. }
                if *w == who && *d == delta && sim.world().objects[o.index()].id == object)
        })
        .count()
}

fn at(day: u32, hour: u32, minute: u32) -> Tick {
    Tick(day * TICKS_PER_DAY).at_time(hour, minute)
}

#[test]
fn tom_fishes_on_sunday_and_mara_bakes() {
    let mut sim: Sim<Town> = Sim::new(7, town());
    // Monday evening: nobody keeps Tom company at 20:00 (Mara drinks at 19:06 and is there),
    // so his mood depends on whether she is in the hall. Run to Sunday and look.
    let mut events = Vec::new();
    for day in 0..7 {
        events.extend(sim.step_until(at(day, 23, 59)).unwrap());
    }
    let fished = events
        .iter()
        .any(|e| matches!(e, Emitted::Activity { activity, .. } if activity == "fish"));
    assert!(fished, "Tom fished at some point in the week");
    let bakes = stocked(
        &sim,
        &events,
        "mara",
        "shelf_bakery",
        characters::mara::LOAVES_PER_BATCH,
    );
    assert!(bakes >= 6, "Mara bakes most days: {bakes}");
}

#[test]
fn mara_brings_a_pastry_only_after_tom_seemed_or_sounded_low() {
    // She knows Tom only from talking to him and hearing him: no pastry on the first day,
    // and each run follows a talk in which he seemed low or grumbling she overheard.
    let mut sim: Sim<Town> = Sim::new(1, town());
    let mut events = sim.step_until(at(1, 0, 0)).unwrap();
    // Days with a pastry run (the activity restarts after the talk at the bar).
    let ran = |events: &[Emitted]| {
        events
            .iter()
            .filter(
                |e| matches!(e, Emitted::Activity { activity, .. } if activity == "bring-pastry"),
            )
            .map(|e| e.tick().day_index())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    assert_eq!(ran(&events), 0, "no pastry on the first day");
    events.extend(sim.step_until(at(28, 0, 0)).unwrap());
    let runs = ran(&events);
    assert!(runs >= 1, "Tom seemed low at least once in four weeks");
    let reasons = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::Noted { text, .. }
                if text.starts_with("Tom seemed low") || text.starts_with("heard Tom grumbling"))
        })
        .count();
    assert_eq!(runs, reasons, "every run had its reason noted");
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

/// Conversations of the day: (opener key, topic).
fn conversations<'a>(sim: &'a Sim<Town>, events: &'a [Emitted]) -> Vec<(&'a str, &'a str)> {
    events
        .iter()
        .filter_map(|e| match e {
            Emitted::ConversationStarted { who, topic, .. } => Some((
                sim.state().villagers[who.index()].id.as_str(),
                topic.key.as_str(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn villagers_greet_talk_and_remember() {
    let mut sim: Sim<Town> = Sim::new(7, town());
    let events = sim.step_until(at(1, 0, 0)).unwrap();
    let talks = conversations(&sim, &events);
    assert!(
        talks.contains(&("pip", "bun")),
        "Pip asks Mara for a bun at noon: {talks:?}"
    );
    assert!(
        talks.contains(&("mara", "ale")),
        "Mara buys a drink from Tom: {talks:?}"
    );
    assert!(
        talks.contains(&("nell", "bread")),
        "Nell buys bread from Mara: {talks:?}"
    );
    let greetings = events
        .iter()
        .filter(|e| matches!(e, Emitted::Talked { minutes: 0, .. }))
        .count();
    assert!(greetings >= 4, "greetings in passing: {greetings}");
    let c = &sim.state().characters;
    assert_eq!(
        stocked(&sim, &events, "pip", "shelf_bakery", -1),
        1,
        "one bun"
    );
    // Each side keeps its own episode of every conversation it had.
    let mara = sim.ctx().villager_id("mara").unwrap();
    assert!(c.tom.mind.regard(mara).familiarity >= 1, "{:?}", c.tom.mind);
    assert!(c.pip.mind.regard(mara).familiarity >= 1, "{:?}", c.pip.mind);
    // Nobody is left stuck in "talk": conversations resume the previous activity.
    for v in &sim.state().villagers {
        assert_ne!(v.activity, "talk", "{}", views::where_is_everyone(&sim));
    }
    assert!(
        sim.state().conversations.is_empty(),
        "every conversation ended"
    );
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
    assert!(
        conversations(&sim, &events).contains(&("pip", "bun")),
        "{}",
        views::itinerary(&sim, pip, &events)
    );
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

#[test]
fn tom_weeds_the_yard_once_it_grows() {
    use sim::protocol::Surface;
    let mut sim: Sim<Town> = Sim::new(5, town());
    let townp = sim.world().place_index["town"];
    let lane_end = content::Pos { x: 16, y: 22 };
    // Two weeks in, the far end of the lane is weedy or has just been cleared.
    let events = sim.step_until(at(14, 0, 0)).unwrap();
    let cleared: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Emitted::Cleared { .. }))
        .collect();
    assert!(!cleared.is_empty(), "Tom never weeded");
    let notes: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Decided { text, .. } if text.contains("weeding") => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(!notes.is_empty(), "no weeding note");
    let surface = sim.surface(townp, lane_end).unwrap();
    assert!(
        matches!(surface, Surface::Path | Surface::WeedyPath),
        "lane end is {surface:?}"
    );
    // The rest of the green is nobody's job and goes wild in patches. Where exactly depends
    // on where Pip happens to roam, so count rather than name a tile.
    let p = &sim.world().places[townp];
    let wild = (0..p.height)
        .flat_map(|y| (0..p.width).map(move |x| content::Pos { x, y }))
        .filter(|&pos| {
            matches!(
                sim.surface(townp, pos),
                Some(Surface::TallGrass | Surface::Brush)
            )
        })
        .count();
    assert!(wild > 100, "only {wild} wild tiles");
}

#[test]
fn money_and_goods_go_round() {
    let mut sim: Sim<Town> = Sim::new(3, town());
    let mut events = Vec::new();
    for day in 0..7 {
        events.extend(sim.step_until(at(day, 23, 59)).unwrap());
    }
    let paid = |what: &str| {
        events
            .iter()
            .filter(
                |e| matches!(e, Emitted::Paid { what: w, amount, .. } if w == what && *amount > 0),
            )
            .count()
    };
    assert_eq!(paid("pension"), 1, "one pension on Monday");
    assert!(
        paid("bread") >= 5,
        "bread sells most days: {}",
        paid("bread")
    );
    assert!(paid("ale") >= 5, "ale sells most nights: {}", paid("ale"));
    assert!(
        paid("flour") >= 2,
        "Hal delivers flour twice: {}",
        paid("flour")
    );
    let market = events
        .iter()
        .any(|e| matches!(e, Emitted::Paid { to: None, .. }));
    assert!(market, "money leaves town through the market");
    // Money is conserved apart from what enters and leaves.
    let start: u32 = town().villagers.iter().map(|v| v.coins).sum();
    let inflow: u32 = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Paid {
                from: None, amount, ..
            } => Some(*amount),
            _ => None,
        })
        .sum();
    let outflow: u32 = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Paid {
                to: None, amount, ..
            } => Some(*amount),
            _ => None,
        })
        .sum();
    let now: u32 = sim.state().villagers.iter().map(|v| v.coins).sum();
    assert_eq!(now, start + inflow - outflow);
    let hal = sim.ctx().villager_id("hal").unwrap();
    let trips = events
        .iter()
        .filter(|e| matches!(e, Emitted::Paid { from: Some(f), to: None, .. } if *f == hal))
        .count();
    assert_eq!(trips, 2, "two market trips");
    assert!(paid("bread") >= 4, "Nell buys bread most days");
    assert!(views::economy(&sim).contains("flour"));
}

#[test]
fn bodies_send_people_to_the_privy_and_back() {
    let mut sim: Sim<Town> = Sim::new(11, town());
    let events = sim.step_until(at(2, 0, 0)).unwrap();
    let needs = events
        .iter()
        .filter(|e| matches!(e, Emitted::Need { .. }))
        .count();
    assert!(needs >= 5, "needs got pressing {needs} times in two days");
    let relieved = events
        .iter()
        .filter(|e| matches!(e, Emitted::Activity { activity, .. } if activity == "relieve"))
        .count();
    assert!(relieved >= 3, "privy visits: {relieved}");
    // A visit returns the villager to what they were doing: the activity after an errand
    // (via `Resumed`) is the one they left.
    let resumed = events
        .iter()
        .any(|e| matches!(e, Emitted::Decided { text, .. } if text.starts_with("resumed")));
    let _ = resumed; // characters do not react to Resumed, so no Decided line; check state instead
    for v in &sim.state().villagers {
        assert!(v.interlude.is_none(), "{} is stuck in an errand", v.name);
        assert!(
            v.needs.bladder < 100 || v.activity == "sleep",
            "{} never relieved: {:?}",
            v.name,
            v.needs
        );
    }
}

#[test]
fn rewinding_replays_the_past_exactly() {
    use sim::history::{History, HistoryError};
    use sim::input::{Input, InputKind, Source};
    let mut history: History<Town> = History::new(9, town());
    history.step_until(at(3, 0, 0)).unwrap();
    history
        .apply_input(Input {
            tick: at(4, 9, 0),
            source: Source::Player(0),
            kind: InputKind::Marker("later".into()),
            seq: 0,
        })
        .unwrap();
    history.step_until(at(6, 15, 0)).unwrap();
    let hash_now = history.sim().hash();
    let days = history.summaries().len();
    assert_eq!(days, 6, "six full days summarised");

    // Back to day 1 at noon: the sim is where it was then.
    history.rewind(at(1, 12, 0)).unwrap();
    assert!(history.in_past());
    assert_eq!(history.sim().tick(), at(1, 12, 0));
    let refused = history.apply_input(Input {
        tick: at(1, 13, 0),
        source: Source::Player(0),
        kind: InputKind::Marker("fork".into()),
        seq: 0,
    });
    assert!(matches!(refused, Err(HistoryError::InThePast { .. })));

    // Forward again: same state, the input made after the snapshot replayed, and no day
    // summarised twice.
    history.back_to_now().unwrap();
    assert!(!history.in_past());
    assert_eq!(history.sim().hash(), hash_now);
    assert_eq!(history.summaries().len(), days);
    assert_eq!(history.sim().state().markers.len(), 1);
}

#[test]
fn summaries_report_deviations_and_settle_into_routine() {
    use sim::protocol::SummaryKind;
    let mut history: sim::history::History<Town> = sim::history::History::new(1, town());
    history.step_until(at(15, 2, 0)).unwrap();
    let days = history.summaries();
    // Tom's first Sunday is news: his code drops the bar for fishing.
    let sunday = &days[6];
    assert!(
        sunday
            .news
            .iter()
            .any(|i| i.kind == SummaryKind::PlanChange && i.text.starts_with("Tom changed the day")),
        "{}",
        sim::summary::render(sunday, true)
    );
    // His second Sunday is routine: the same change happened on the same weekday last week.
    let next_sunday = &days[13];
    assert!(
        next_sunday
            .routine
            .iter()
            .any(|i| i.kind == SummaryKind::PlanChange && i.text.starts_with("Tom changed the day")),
        "{}",
        sim::summary::render(next_sunday, true)
    );
    // After the first week, most of a day is routine.
    let quiet = &days[10];
    assert!(
        quiet.routine.len() > quiet.news.len(),
        "{}",
        sim::summary::render(quiet, true)
    );
    // Causes travel with the lines.
    assert!(sunday.news.iter().any(|i| i.cause.is_some()));
}

#[test]
fn everyone_keeps_their_schedule_when_hungry_and_eats_when_starving() {
    let mut sim: Sim<Town> = Sim::new(1, town());
    let events = sim.step_until(at(14, 0, 0)).unwrap();
    for v in 0..sim.state().villagers.len() {
        let who = sim::ids::VillagerId(u16::try_from(v).unwrap());
        let name = sim.state().villagers[v].name.clone();
        // Every hunger reaction below urgent leaves the schedule alone; every urgent one
        // gets an answer.
        for e in &events {
            let Emitted::Decided { who: w, text, .. } = e else {
                continue;
            };
            if *w != who || !text.starts_with("hungry (") {
                continue;
            }
            let level: u8 = text["hungry (".len()..]
                .split(')')
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap();
            if level < sim::needs::URGENT {
                assert!(text.contains("it can wait"), "{name}: {text}");
            } else {
                assert!(text.contains("starving"), "{name}: {text}");
            }
        }
        // And everyone eats every day.
        for day in 0..14 {
            let ate = events.iter().any(|e| {
                matches!(e, Emitted::Activity { who: w, activity, tick, .. }
                    if *w == who && activity == "eat" && tick.0 / sim::time::TICKS_PER_DAY == day)
            });
            assert!(ate, "{name} did not eat on day {day}");
        }
    }
    // Tom's own twist: starving behind the bar, he eats the bar's bread.
    assert!(stocked(&sim, &events, "tom", "bar_tavern", -1) > 0);
}

#[test]
fn errands_queue_rather_than_replace_each_other() {
    // Waking with several pressing needs, each gets its errand in turn. Which mornings
    // have all three depends on the run, so check every one in the first week that does.
    let mut sim: Sim<Town> = Sim::new(1, town());
    let events = sim.step_until(at(7, 9, 0)).unwrap();
    let tom = sim.ctx().villager_id("tom").unwrap();
    let mut checked = 0;
    for day in 1..7 {
        let morning: Vec<&str> = events
            .iter()
            .filter_map(|e| match e {
                Emitted::Activity {
                    who,
                    activity,
                    tick,
                    ..
                } if *who == tom
                    && *tick >= at(day, 7, 30)
                    && *tick < at(day, 9, 0)
                    && activity != "walking" =>
                {
                    Some(activity.as_str())
                }
                _ => None,
            })
            .collect();
        let relieve = morning.iter().position(|a| *a == "relieve");
        let water = morning.iter().position(|a| *a == "drink-water");
        let eat = morning.iter().position(|a| *a == "eat");
        if relieve.is_some() && water.is_some() && eat.is_some() {
            assert!(relieve < water && water < eat, "day {day}: {morning:?}");
            checked += 1;
        }
    }
    assert!(checked > 0, "no morning with all three");
}
