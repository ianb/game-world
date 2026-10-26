//! Hal is sweet on Mara, Mara is fond of Tom, and Tom does not notice. What each of them
//! does comes from what they saw and heard.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::ids::VillagerId;
use sim::protocol::{Act, BidAnswer, Emitted, Frame, Gesture};
use sim::recall::Record;
use sim::time::{TICKS_PER_HOUR, Tick};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn town() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap()
}

#[test]
fn the_triangle_runs_on_what_each_of_them_perceives() {
    // On seed 1, Mara learns in the second week, from Hal muttering at the tavern.
    let mut sim: Sim<Town> = Sim::new(1, town());
    let ctx = sim.ctx();
    let [hal, mara, tom] = ["hal", "mara", "tom"].map(|v| ctx.villager_id(v).unwrap());
    // Stepped an hour at a time so each thought is checked against the record while what
    // led to it is still remembered.
    let mut events = Vec::new();
    let mut rivals = Vec::new();
    for hour in 1..=21 * 24 {
        let step = sim.step_until(Tick(hour * TICKS_PER_HOUR)).unwrap();
        rivals.extend(hal_is_jealous_of_what_he_saw(
            &step,
            &sim.state().record,
            hal,
            mara,
        ));
        events.extend(step);
    }
    assert!(
        rivals.iter().any(|aloud| *aloud),
        "said aloud at least once"
    );
    let pairs = mara_seeks_out_tom(&events, mara, tom);
    mara_keeps_hal_at_a_distance(&events, &pairs, hal, mara);

    // Tom hears himself muttered about and holds it against Hal, without knowing why.
    let stung = events.iter().any(|e| {
        matches!(e, Emitted::Thought { who, thought, .. }
            if *who == tom && thought.about.is("what-was-said") && thought.about.about == Some(hal))
    });
    assert!(stung);
}

type Pairs = BTreeMap<u32, (VillagerId, VillagerId)>;

/// Mara seeks Tom out, beyond buying her ale, and pays him compliments. Returns who started
/// each conversation with whom.
fn mara_seeks_out_tom(events: &[Emitted], mara: VillagerId, tom: VillagerId) -> Pairs {
    let mut pairs = BTreeMap::new();
    let mut sought = 0;
    let mut compliments = 0;
    for e in events {
        match e {
            Emitted::ConversationStarted {
                convo,
                who,
                with,
                frame,
                ..
            } => {
                pairs.insert(*convo, (*who, *with));
                sought += usize::from(*who == mara && *with == tom && *frame != Frame::Transaction);
            }
            Emitted::Beat {
                convo,
                who,
                did:
                    Act::Gesture {
                        gesture: Gesture::Compliment,
                        ..
                    },
                ..
            } if *who == mara && pairs.get(convo) == Some(&(mara, tom)) => compliments += 1,
            _ => {}
        }
    }
    assert!(
        sought > 0 && compliments > 0,
        "sought {sought}, compliments {compliments}"
    );
    pairs
}

/// Every rival thought of Hal's is about someone he saw Mara seek out shortly before.
/// Returns whether each was said aloud.
fn hal_is_jealous_of_what_he_saw(
    events: &[Emitted],
    record: &Record,
    hal: VillagerId,
    mara: VillagerId,
) -> Vec<bool> {
    let rival_thoughts: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Emitted::Thought {
                tick, who, thought, ..
            } if *who == hal && thought.about.is("rival") => {
                Some((*tick, thought.about.about, thought.aloud))
            }
            _ => None,
        })
        .collect();
    for (tick, rival, _) in &rival_thoughts {
        let saw = record.recall(hal, tick.minus(30 * 60)).any(|(r, _)| {
            matches!(&r.event, Emitted::ConversationStarted { who, with, .. }
                if *who == mara && Some(*with) == *rival)
        });
        assert!(saw, "rival thought at {tick:?} with nothing seen");
    }
    rival_thoughts.iter().map(|t| t.2).collect()
}

/// Once Mara knows, she keeps Hal at a distance.
fn mara_keeps_hal_at_a_distance(
    events: &[Emitted],
    pairs: &Pairs,
    hal: VillagerId,
    mara: VillagerId,
) {
    let knows = events
        .iter()
        .find_map(|e| match e {
            Emitted::Thought {
                tick, who, thought, ..
            } if *who == mara && thought.about.is("sweet-on") => Some(*tick),
            _ => None,
        })
        .expect("Mara learns Hal is sweet on her");
    for e in events {
        let Emitted::Beat {
            tick,
            convo,
            who,
            did,
            ..
        } = e
        else {
            continue;
        };
        if *tick <= knows
            || *who != mara
            || !pairs.get(convo).is_some_and(|p| p.0 == hal || p.1 == hal)
        {
            continue;
        }
        match did {
            Act::Gesture { gesture, .. } => assert!(
                matches!(
                    gesture,
                    Gesture::Offer
                        | Gesture::Thank
                        | Gesture::Leave
                        | Gesture::Ask
                        | Gesture::Request
                ),
                "Mara to Hal after knowing: {did:?}"
            ),
            Act::Answer { answer } => assert_ne!(*answer, BidAnswer::Toward),
            Act::Bid { .. } => panic!("Mara bids with Hal after knowing: {did:?}"),
        }
    }
}
