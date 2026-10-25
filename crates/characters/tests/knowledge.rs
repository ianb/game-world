//! Knowledge moving through the town and turning into action: news about someone passed
//! on in conversation, and a gift that cites who said what.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::protocol::{Act, Caught, Cause, Emitted, Gesture};
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
fn news_about_someone_is_passed_on_to_someone_else() {
    let events = weeks(1, 2);
    let passed_on = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::Beat {
                who,
                did: Act::Gesture { gesture: Gesture::Tell, topic },
                ..
            } if topic.about.is_some_and(|a| a != *who))
        })
        .count();
    assert!(passed_on > 0);
}

#[test]
fn a_gift_cites_words_that_were_said_to_the_giver() {
    // On seed 1, Nell gives Mara coins twice in the third week: once after Pip passes on
    // Mara's trouble, once after overhearing Mara try to confide in Tom.
    let seed = 1;
    let events = weeks(seed, 3);
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
    assert!(!gifts.is_empty(), "someone acts on what they heard");
}
