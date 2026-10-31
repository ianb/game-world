//! Scenes: the town set up at a moment so that one behavior has what it needs, with the
//! chances that would decide it forced. Each states its preconditions instead of relying
//! on a seed that happens to produce them.
use sim::needs::Need;
use sim::protocol::{Act, BidAnswer, Cause, Emitted, Frame, Gesture};
use sim::social::chances;
use sim::time::TICKS_PER_MINUTE;
use valley::scene::{Scene, at};
use valley::{benny, kit, penn};

/// Nell, short of money, confides it to Elias in her cottage; he takes the bid, hears it,
/// and helps her when the talk ends, citing what she said.
#[test]
fn a_worry_confided_in_private_is_taken_up_and_acted_on() {
    let mut s = Scene::new();
    s.run_until(at(0, 14, 0));
    s.force_for(chances::CONFIDE, "nell", true)
        .force(chances::MISFIRE, false)
        .force(chances::READ_WORSE, false)
        .set_coins("nell", 1)
        .acquaint("nell", "elias", 3)
        .acquaint("elias", "nell", 3)
        .put_by("nell", "chair_nell")
        .put_beside("elias", "nell")
        .talk("nell", "elias", Frame::SmallTalk, "day", 20);
    let events = s.run_until(at(0, 15, 0)).to_vec();
    let (nell, elias) = (s.id("nell"), s.id("elias"));

    let bid = events.iter().find_map(|e| match e {
        Emitted::Beat {
            tick,
            who,
            did:
                Act::Bid {
                    frame: Frame::Confiding,
                    topic,
                },
            ..
        } if *who == nell => Some((*tick, topic.clone())),
        _ => None,
    });
    let (bid_at, worry) = bid.expect("Nell bids to confide");
    assert!(worry.is("money") && worry.about == Some(nell));
    assert!(events.iter().any(|e| matches!(e,
        Emitted::Beat { tick, who, did: Act::Answer { answer: BidAnswer::Toward }, .. }
            if *who == elias && *tick > bid_at)));
    assert!(
        events.iter().any(|e| matches!(e,
            Emitted::Beat { who, did: Act::Gesture { gesture: Gesture::Disclose, topic }, .. }
                if *who == nell && topic.is("money"))),
        "Nell tells him"
    );
    // Every time she said it, by gesture or bid.
    let said: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Beat {
                tick,
                who,
                did: Act::Gesture { topic, .. } | Act::Bid { topic, .. },
                ..
            } if *who == nell && topic.is("money") => Some(*tick),
            _ => None,
        })
        .collect();
    let help = events.iter().find_map(|e| match e {
        Emitted::Paid {
            who,
            to: Some(to),
            cause: Cause::Knew { from, heard_at, .. },
            ..
        } if *who == elias && *to == nell => Some((*from, *heard_at)),
        _ => None,
    });
    let (from, heard_at) = help.expect("Elias helps her");
    assert_eq!(from, nell);
    assert!(
        said.contains(&heard_at),
        "the help cites something she said"
    );
}

/// Without the chance, the same set-up has Nell keep it to herself.
#[test]
fn without_the_chance_she_keeps_it_to_herself() {
    let mut s = Scene::new();
    s.run_until(at(0, 14, 0));
    s.force_for(chances::CONFIDE, "nell", false)
        .set_coins("nell", 1)
        .acquaint("nell", "elias", 3)
        .acquaint("elias", "nell", 3)
        .put_by("nell", "chair_nell")
        .put_beside("elias", "nell")
        .talk("nell", "elias", Frame::SmallTalk, "day", 20);
    let nell = s.id("nell");
    let events = s.run_until(at(0, 15, 0));
    assert!(!events.iter().any(|e| matches!(e,
        Emitted::Beat { who, did: Act::Bid { frame: Frame::Confiding, .. }, .. } if *who == nell)));
}

/// Nell confides in Hester, who keeps nothing to herself; Hester tells Elias; Elias, who
/// never heard it from Nell, helps her when he sees her, citing Hester.
#[test]
fn news_passed_on_reaches_someone_who_acts_on_it() {
    let mut s = Scene::new();
    s.run_until(at(0, 14, 0));
    s.force_for(chances::CONFIDE, "nell", true)
        .force_for(chances::GOSSIP, "hester", true)
        .force(chances::MISFIRE, false)
        .force(chances::READ_WORSE, false)
        .set_coins("nell", 1)
        .acquaint("nell", "hester", 3)
        .acquaint("hester", "nell", 3)
        .acquaint("hester", "elias", 3)
        .acquaint("elias", "hester", 3)
        .acquaint("elias", "nell", 3)
        .put_by("nell", "chair_nell")
        .put_beside("hester", "nell")
        .talk("nell", "hester", Frame::SmallTalk, "day", 20);
    s.run_until(at(0, 14, 30));
    s.put_by("hester", "altar_chapel")
        .put_beside("elias", "hester")
        .talk("hester", "elias", Frame::SmallTalk, "day", 20);
    s.run_until(at(0, 15, 0));
    s.put_by("nell", "table_vicarage")
        .put_beside("elias", "nell")
        .talk("elias", "nell", Frame::SmallTalk, "day", 10);
    s.run_until(at(0, 15, 30));
    let (nell, hester, elias) = (s.id("nell"), s.id("hester"), s.id("elias"));

    let passed_on: Vec<_> = s
        .events
        .iter()
        .filter_map(|e| match e {
            Emitted::Beat {
                tick,
                who,
                did:
                    Act::Gesture {
                        gesture: Gesture::Tell,
                        topic,
                    },
                ..
            } if *who == hester && topic.about == Some(nell) && *tick >= at(0, 14, 30) => {
                Some(*tick)
            }
            _ => None,
        })
        .collect();
    assert!(!passed_on.is_empty(), "Hester passes it on");
    let help = s.events.iter().find_map(|e| match e {
        Emitted::Paid {
            who,
            to: Some(to),
            cause: Cause::Knew { from, heard_at, .. },
            ..
        } if *who == elias && *to == nell => Some((*from, *heard_at)),
        _ => None,
    });
    let (from, heard_at) = help.expect("Elias helps her");
    assert_eq!(from, hester, "citing Hester");
    assert!(passed_on.contains(&heard_at), "citing what Hester told him");
}

/// Two needs pressing at the same moment each get their errand in turn, rather than the
/// second replacing the first.
#[test]
fn errands_queue_rather_than_replace_each_other() {
    let mut s = Scene::new();
    s.run_until(at(0, 9, 30));
    let elias = s.id("elias");
    let needs = s.needs("elias");
    needs.bladder = sim::needs::PRESSING - 1;
    needs.thirst = sim::needs::PRESSING - 1;
    let events = s.run_until(at(0, 11, 0));
    let pressed = events
        .iter()
        .find_map(|e| match e {
            Emitted::Need {
                tick,
                who,
                need: Need::Bladder,
                ..
            } if *who == elias => Some(*tick),
            _ => None,
        })
        .expect("his bladder presses");
    assert!(events.iter().any(|e| matches!(e,
        Emitted::Need { tick, who, need: Need::Thirst, .. } if *who == elias && *tick == pressed)));
    let within = pressed.plus(45 * TICKS_PER_MINUTE);
    let done: Vec<&str> = events
        .iter()
        .filter(|e| e.tick() >= pressed && e.tick() < within)
        .filter_map(|e| match e {
            Emitted::Activity { who, activity, .. } if *who == elias => Some(activity.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        done.contains(&"relieve") && done.contains(&"drink-water"),
        "{done:?}"
    );
}

/// Kit looks for Pip before school two mornings running; Agnes notes him missing at half
/// past nine both days, and tells his mother when she next meets her.
#[test]
fn agnes_tells_a_parent_when_a_pupil_keeps_coming_late() {
    let mut s = Scene::new();
    s.force_for(kit::DAWDLES, "kit", true);
    let (kit, agnes, jenna) = (s.id("kit"), s.id("agnes"), s.id("jenna"));
    s.run_until(at(2, 17, 0));
    let mut looked: Vec<u32> = s
        .events
        .iter()
        .filter_map(|e| match e {
            Emitted::Activity {
                tick,
                who,
                activity,
                ..
            } if *who == kit
                && activity == "look-for-pip"
                && *tick < at(tick.day_index(), 9, 30) =>
            {
                Some(tick.day_index())
            }
            _ => None,
        })
        .collect();
    looked.dedup();
    assert_eq!(looked, [1, 2], "Kit dawdles both mornings");
    let noted = s
        .events
        .iter()
        .filter(|e| matches!(e, Emitted::Noted { who, text, .. } if *who == agnes && text == "Kit is not at their desk"))
        .count();
    assert_eq!(noted, 2, "Agnes notes it both mornings");

    s.put_by("jenna", "table_cooper")
        .put_beside("agnes", "jenna");
    let events = s.run_until(at(2, 17, 30));
    assert!(
        events.iter().any(|e| matches!(e,
            Emitted::Beat { who, did: Act::Gesture { topic, .. } | Act::Bid { topic, .. }, .. }
                if *who == agnes && topic.is("being-late") && topic.about == Some(kit))
            || matches!(e, Emitted::Talked { who, with, topic, .. }
                if *who == agnes && *with == jenna && topic.starts_with("being-late"))),
        "Agnes tells Jenna about Kit"
    );
}

/// Without the chance he goes straight to school, and Agnes has nothing to note.
#[test]
fn kit_on_time_gives_agnes_nothing_to_note() {
    let mut s = Scene::new();
    s.force_for(kit::DAWDLES, "kit", false);
    let agnes = s.id("agnes");
    let events = s.run_until(at(1, 12, 0));
    assert!(!events.iter().any(|e| matches!(e,
        Emitted::Noted { who, text, .. } if *who == agnes && text == "Kit is not at their desk")));
}

/// Benny hides at bedtime; after seven Rosa goes looking, finds him, and he goes to bed.
#[test]
fn rosa_finds_benny_when_he_hides_at_bedtime() {
    let mut s = Scene::new();
    s.force_for(benny::HIDES, "benny", true);
    let (benny, rosa) = (s.id("benny"), s.id("rosa"));
    let events = s.run_until(at(0, 23, 0));
    let hid = events
        .iter()
        .find_map(|e| match e {
            Emitted::Activity {
                tick,
                who,
                activity,
                ..
            } if *who == benny && activity == "hiding" => Some(*tick),
            _ => None,
        })
        .expect("Benny hides");
    let found = events
        .iter()
        .find_map(|e| match e {
            Emitted::Beat {
                tick,
                who,
                did: Act::Gesture { topic, .. } | Act::Bid { topic, .. },
                ..
            } if *who == rosa && topic.is("hiding") && *tick > hid => Some(*tick),
            _ => None,
        })
        .expect("Rosa finds him");
    assert!(events.iter().any(|e| matches!(e,
        Emitted::Activity { tick, who, activity, .. } if *who == benny && activity == "sleep" && *tick > found)));
}

/// Penn forgets at the kiln: he blames himself and goes back to the wheel, and no pots
/// leave the wheel.
#[test]
fn penn_forgets_at_the_kiln() {
    let mut s = Scene::new();
    s.force_for(penn::FORGETS, "penn", true);
    let penn = s.id("penn");
    let events = s.run_until(at(1, 0, 0));
    assert!(events.iter().any(|e| matches!(e,
        Emitted::Thought { who, thought, .. } if *who == penn && thought.about.is("forgetting"))));
    assert!(!events.iter().any(|e| matches!(e,
        Emitted::Stocked { who, delta, .. } if *who == penn && *delta < 0)));
}

/// Tuesday's train: the hens have laid, Ada takes eggs and cheese to the platform before it
/// comes, it leaves deliveries in the shed and pays her when it goes, and Greta fetches
/// the tea for the store.
#[test]
fn the_tuesday_train_takes_eggs_and_brings_tea() {
    let mut s = Scene::new();
    s.force(sim::animals::LAYS, true);
    let (ada, greta) = (s.id("ada"), s.id("greta"));
    let events = s.run_until(at(1, 12, 0)).to_vec();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Emitted::Laid { flock, .. } if flock == "hens_hollins"))
    );
    let sent: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Consigned { who, good, .. } if *who == ada => Some(good.as_str()),
            _ => None,
        })
        .collect();
    assert!(sent.contains(&"eggs"), "Ada sends eggs: {sent:?}");
    assert!(events.iter().any(|e| matches!(e, Emitted::TrainArrived { delivered, .. } if delivered.iter().any(|d| d.good == "tea"))));
    let paid = events.iter().find_map(|e| match e {
        Emitted::TrainLeft { shipped, .. } => Some(
            shipped
                .iter()
                .filter(|s| s.who == ada)
                .map(|s| s.paid)
                .sum::<u32>(),
        ),
        _ => None,
    });
    assert!(paid.is_some_and(|p| p > 0), "the railway pays Ada");
    assert!(events.iter().any(|e| matches!(e,
        Emitted::Paid { who, to: None, what, .. } if *who == greta && what == "tea")));
    assert!(
        s.sim
            .ctx()
            .stock(s.sim.ctx().object_id("shelf_store").unwrap(), "tea")
            > 0
    );
}

/// On Monday the afternoon train goes through without stopping, and nobody goes to meet it.
#[test]
fn the_monday_train_passes_through() {
    let mut s = Scene::new();
    let events = s.run_until(at(0, 16, 0));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Emitted::TrainPassed { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Emitted::Consigned { .. } | Emitted::TrainArrived { .. }))
    );
}

/// The same hens, three people: Pip is delighted and says so, Tom is annoyed and lays it at
/// Ada's door aloud, and Rob does not care.
#[test]
fn people_take_to_the_hens_in_their_own_ways() {
    let mut s = Scene::new();
    s.run_until(at(0, 10, 0));
    let ada = s.id("ada");
    let mut felt = |who: &str| {
        s.put_by_flock(who, "hens_hollins");
        let id = s.id(who);
        s.events.iter().rev().find_map(|e| match e {
            Emitted::Thought { who, thought, .. } if *who == id && thought.about.is("hens") => {
                Some((thought.feeling, thought.aloud, thought.about.about))
            }
            _ => None,
        })
    };
    assert_eq!(felt("pip"), Some((2, true, Some(ada))));
    assert_eq!(felt("tom"), Some((-1, true, Some(ada))));
    assert_eq!(felt("rob"), None);
}

/// Pip, delighted by Ada's hens, runs after them for a few minutes and then goes back to
/// what he was doing; the hens keep out of his reach.
#[test]
fn pip_chases_the_hens_and_they_run() {
    let mut s = Scene::new();
    s.run_until(at(0, 10, 0));
    let pip = s.id("pip");
    s.put_by_flock("pip", "hens_hollins");
    let events = s.run_until(at(0, 10, 20)).to_vec();
    let chased = events.iter().find_map(|e| match e {
        Emitted::Activity {
            tick,
            who,
            activity,
            ..
        } if *who == pip && activity == "chase-hens" => Some(*tick),
        _ => None,
    });
    let chased = chased.expect("Pip chases the hens");
    assert!(events.iter().any(|e| matches!(e,
        Emitted::Activity { tick, who, activity, .. } if *who == pip && activity != "chase-hens" && activity != "walking" && *tick > chased)),
        "and then goes back to what he was doing");
    // While he chases, no hen lets him stand next to it for long: at the end, none is
    // beside him.
    let p = s.sim.state().villagers[pip.index()].pos;
    let flock = s
        .sim
        .flocks()
        .iter()
        .position(|f| f.id == "hens_hollins")
        .unwrap();
    assert!(
        s.sim
            .state()
            .animals
            .iter()
            .filter(|a| usize::from(a.flock) == flock)
            .all(|a| a.pos != p)
    );
}

/// On a day the train stops Lottie is on the platform before it comes, stays until it has
/// gone, and then says aloud that she wants to leave.
#[test]
fn lottie_watches_the_train_go() {
    let mut s = Scene::new();
    s.force_for(valley::lottie::SAYS_SO, "lottie", true);
    let lottie = s.id("lottie");
    let events = s.run_until(at(1, 11, 0)).to_vec();
    let watching = events.iter().find_map(|e| match e {
        Emitted::Activity {
            tick,
            who,
            activity,
            ..
        } if *who == lottie && activity == "watch-train" => Some(*tick),
        _ => None,
    });
    assert!(
        watching.is_some_and(|t| t < at(1, 10, 0)),
        "on the platform before ten"
    );
    let left = events.iter().find_map(|e| match e {
        Emitted::TrainLeft { tick, .. } => Some(*tick),
        _ => None,
    });
    let thought = events.iter().find_map(|e| match e {
        Emitted::Thought { tick, who, thought }
            if *who == lottie && thought.about.is("leaving") =>
        {
            Some((*tick, thought.aloud))
        }
        _ => None,
    });
    let (when, aloud) = thought.expect("she thinks about leaving");
    assert!(
        aloud && left.is_some_and(|l| when > l),
        "aloud, after the train has gone"
    );
}

/// Celia, with money to spare, buys tea, sugar and tobacco at the store on Tuesday
/// afternoon, after the train has brought them, with at most half of what she has above
/// what she keeps by her. The shelf goes down by what she took.
#[test]
fn money_to_spare_goes_on_luxuries_at_the_store() {
    use valley::toolbox::{COMFORT_COINS, LUXURIES};
    let mut s = Scene::new();
    s.run_until(at(1, 13, 50));
    let counter = s.sim.ctx().object_id("counter_store").unwrap();
    let shelf = s.sim.ctx().object_id("shelf_store").unwrap();
    let before: u16 = LUXURIES
        .iter()
        .map(|(g, _)| s.sim.ctx().stock(shelf, g))
        .sum();
    s.set_coins("celia", 100)
        .put_by("greta", "counter_store")
        .put_by("celia", "shelf_store")
        .cue(
            "celia",
            at(1, 14, 0),
            sim::brain::Command::GoTo {
                target: sim::brain::Target::Object(counter),
                activity: "buy-luxuries".into(),
            },
        );
    let events = s.run_until(at(1, 14, 30)).to_vec();
    let (celia, greta) = (s.id("celia"), s.id("greta"));
    let spent: u32 = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Paid {
                who,
                to: Some(to),
                what,
                amount,
                ..
            } if *who == celia && *to == greta && LUXURIES.iter().any(|(g, _)| g == what) => {
                Some(*amount)
            }
            _ => None,
        })
        .sum();
    assert!(spent > 0, "Celia buys something");
    assert!(spent <= (100 - COMFORT_COINS) / 2, "spent {spent}");
    let after: u16 = LUXURIES
        .iter()
        .map(|(g, _)| s.sim.ctx().stock(shelf, g))
        .sum();
    assert!(after < before, "the shelf went down: {before} to {after}");
}

/// The trip to the store goes into the day's plan only for someone with more than they
/// keep by them.
#[test]
fn only_money_to_spare_sends_someone_shopping() {
    use valley::toolbox::COMFORT_COINS;
    let plans_trip = |coins: u32| {
        let mut s = Scene::new();
        let celia = s.id("celia");
        // A weekday in the second week that is Celia's for the store.
        let day = 7 + u32::from(celia.0) % 5;
        s.run_until(at(day - 1, 23, 0));
        s.set_coins("celia", coins);
        s.run_until(at(day, 0, 30)).iter().any(|e| {
            matches!(e, Emitted::Planned { who, items, .. } if *who == celia
                && items.iter().any(|i| i.activity.as_deref() == Some("buy-luxuries")))
        })
    };
    assert!(plans_trip(COMFORT_COINS + 30));
    assert!(!plans_trip(COMFORT_COINS));
}

/// Will, at his farm work with the wheat ripe, harvests it tile by tile, and the grain
/// goes into the Hollins sacks.
#[test]
fn will_brings_in_ripe_wheat() {
    use sim::protocol::{FieldStage, FieldWork};
    let mut s = Scene::new();
    s.run_until(at(0, 7, 0));
    {
        let plot = s
            .sim
            .state_mut()
            .fields
            .plots
            .iter_mut()
            .find(|p| p.id == "wheat_hollins")
            .expect("wheat field");
        for t in &mut plot.tiles {
            t.stage = FieldStage::Ripe;
            t.growth = 24;
        }
    }
    let sacks = s.sim.ctx().object_id("sacks_loft_hollins").unwrap();
    let before = s.sim.ctx().stock(sacks, "wheat");
    let events = s.run_until(at(0, 16, 0)).to_vec();
    let will = s.id("will");
    let cut = events
        .iter()
        .filter(
            |e| matches!(e, Emitted::Tended { who, work: FieldWork::Harvest, .. } if *who == will),
        )
        .count();
    assert!(cut >= 4, "Will cut {cut} tiles");
    assert!(
        s.sim.ctx().stock(sacks, "wheat") > before,
        "grain went into the sacks"
    );
}

/// A household's garden is kept by one of them each evening, in turn.
#[test]
fn a_household_takes_turns_in_its_garden() {
    let mut s = Scene::new();
    let household = ["penn", "rosa", "benny"];
    let ids: Vec<_> = household.iter().map(|h| s.id(h)).collect();
    let mut turns = Vec::new();
    // Day 0's plans come out with the first step.
    s.run_until(at(0, 1, 0));
    for day in 1..4 {
        let events = s.run_until(at(day, 1, 0)).to_vec();
        let gardeners: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Emitted::Planned { who, items, .. }
                    if ids.contains(who)
                        && items
                            .iter()
                            .any(|i| i.activity.as_deref() == Some("tend-garden")) =>
                {
                    Some(*who)
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(
            gardeners.len(),
            1,
            "one gardener on day {day}: {gardeners:?}"
        );
        turns.push(gardeners[0]);
    }
    turns.dedup();
    assert_eq!(turns.len(), 3, "a different one each day");
}

/// Rain lifts Wynn, who loves it, and gets Josie down; a fine morning does neither.
#[test]
fn rain_cheers_one_and_dampens_another() {
    use sim::protocol::MoodKind;
    let level = |s: &Scene, who: &str, mood: MoodKind| s.sim.ctx().moods(s.id(who)).level(mood);
    let mut wet = Scene::new();
    wet.force(sim::fields::RAINS, true);
    wet.run_until(at(1, 0, 5));
    assert!(
        level(&wet, "wynn", MoodKind::Cheerful) >= 1,
        "Wynn cheered by rain"
    );
    assert!(
        level(&wet, "josie", MoodKind::Gloomy) >= 1,
        "Josie glum in rain"
    );
    let mut dry = Scene::new();
    dry.force(sim::fields::RAINS, false);
    dry.run_until(at(1, 0, 5));
    assert_eq!(level(&dry, "wynn", MoodKind::Cheerful), 0);
    assert_eq!(level(&dry, "josie", MoodKind::Gloomy), 0);
}

/// Bess, afraid of the dark, grows anxious out of doors at night (from the hour, or from
/// her own fretting before she hurries home), and someone who is not afraid of it does not.
#[test]
fn bess_frets_out_after_dark() {
    use sim::protocol::MoodKind;
    let mut s = Scene::new();
    s.run_until(at(0, 20, 30));
    s.put_by("bess", "well_square")
        .put_by("bram", "well_square");
    s.needs("bess").sleep = 0;
    // Kept out at the well past her bedtime.
    s.cue(
        "bess",
        at(0, 20, 31),
        sim::brain::Command::Wait {
            ticks: 3 * 3600,
            activity: "watch".into(),
        },
    );
    let events = s.run_until(at(0, 23, 30)).to_vec();
    let bess = s.id("bess");
    assert!(
        events.iter().any(
            |e| matches!(e, Emitted::MoodShifted { who, mood: MoodKind::Anxious, cause, .. }
            if *who == bess && (cause == "out after dark" || cause == "the dark"))
        ),
        "Bess grew anxious in the dark: {:#?}",
        events
            .iter()
            .filter(|e| e.who() == Some(bess) && !matches!(e, Emitted::Moved { .. }))
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
    );
    assert_eq!(s.sim.ctx().moods(s.id("bram")).level(MoodKind::Anxious), 0);
}

/// Elias, lonely with nobody about, goes where there usually is company.
#[test]
fn a_lonely_villager_goes_looking_for_company() {
    use sim::protocol::MoodKind;
    let mut s = Scene::new();
    s.run_until(at(0, 13, 0));
    let elias = s.id("elias");
    {
        let m = &mut s.sim.state_mut().moods[elias.index()];
        let lonely = m
            .moods
            .iter_mut()
            .find(|m| m.kind == MoodKind::Lonely)
            .unwrap();
        lonely.points = 7;
        m.alone_hours = 10;
    }
    let events = s.run_until(at(0, 16, 0)).to_vec();
    assert!(
        events.iter().any(|e| matches!(e, Emitted::MoodShifted { who, mood: MoodKind::Lonely, level: 2, .. } if *who == elias)),
        "Elias grew lonely"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Emitted::Activity { who, activity, .. }
            if *who == elias && (activity == "company" || activity == "talk"))),
        "and went for company"
    );
}
