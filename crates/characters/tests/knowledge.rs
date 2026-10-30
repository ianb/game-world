//! Knowledge turning into action: a gift cites who said what. The chains that lead to
//! gifts are set up as scenes in `tests/scenes.rs`.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::protocol::{Act, Caught, Cause, Emitted};
use sim::time::{TICKS_PER_DAY, Tick};
use std::path::PathBuf;

fn town() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap()
}

fn weeks(seed: u64, weeks: u32) -> Vec<Emitted> {
    let mut sim: Sim<Town> = Sim::new(seed, town());
    sim.step_until(Tick(weeks * 7 * TICKS_PER_DAY)).unwrap()
}

#[test]
fn every_gift_cites_words_the_giver_caught() {
    // Whether gifts happen at all is the scenes' business (`tests/scenes.rs`); this checks
    // that any gift in an ordinary month is backed by what was said.
    gifts_on(1);
}

/// Checks every gift in four weeks of `seed`.
fn gifts_on(seed: u64) {
    let events = weeks(seed, 4);
    let gifts: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Paid {
                tick,
                who,
                to: Some(to),
                cause:
                    Cause::Knew {
                        from,
                        topic,
                        heard_at,
                    },
                ..
            } => Some((*tick, *who, *to, *from, topic.clone(), *heard_at)),
            _ => None,
        })
        .collect();
    for (tick, giver, to, from, topic, heard_at) in &gifts {
        assert!(heard_at < tick);
        assert_eq!(
            topic.about,
            Some(*to),
            "the gift goes to who the news was about"
        );
        // The speaker said it at that tick, as a gesture or as a bid (an overheard bid to
        // confide counts), and the giver caught the words.
        let said = events.iter().position(|e| {
            matches!(e, Emitted::Beat {
                tick: t, who, did: Act::Gesture { topic: said, .. } | Act::Bid { topic: said, .. }, ..
            } if t == heard_at && who == from && said == topic)
        });
        let caught = events.iter().any(|e| {
            matches!(e, Emitted::TookUp {
                tick: t, who, speaker, caught: Caught::Addressed | Caught::Close, ..
            } if t == heard_at && who == giver && speaker == from)
        });
        assert!(
            said.is_some() && caught,
            "seed {seed}: gift at {tick:?} cites {from:?} at {heard_at:?}"
        );
    }
    // Each giver helps each person once for each piece of news.
    let mut once: Vec<_> = gifts.iter().map(|g| (g.1, g.2, g.5)).collect();
    once.sort();
    let n = once.len();
    once.dedup();
    assert_eq!(once.len(), n, "seed {seed}");
}
