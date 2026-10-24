//! Conversations between the real characters: what four weeks of them contain, and that a
//! conversation in progress survives a snapshot.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::protocol::{Act, BidAnswer, Emitted, Frame};
use sim::time::{TICKS_PER_DAY, Tick};
use std::path::PathBuf;

fn town() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap()
}

fn at(day: u32, hour: u32, minute: u32) -> Tick {
    Tick(day * TICKS_PER_DAY).at_time(hour, minute)
}

#[test]
fn four_weeks_of_talk_hold_misfires_misreadings_breaches_and_refusals() {
    let mut sim: Sim<Town> = Sim::new(1, town());
    let events = sim.step_until(at(28, 0, 0)).unwrap();
    let pip = sim.ctx().villager_id("pip").unwrap();
    let misfires = events
        .iter()
        .filter(|e| matches!(e, Emitted::Beat { meant, did, .. } if meant != did))
        .count();
    let misreadings = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::TookUp { did, taken_as, overheard: false, .. } if did != taken_as)
        })
        .count();
    let pip_breaches = events
        .iter()
        .filter(|e| matches!(e, Emitted::Beat { who, breach: true, .. } if *who == pip))
        .count();
    let refusals = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::Beat { did: Act::Answer { answer }, .. } if *answer != BidAnswer::Toward)
        })
        .count();
    let overheard = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Emitted::TookUp {
                    overheard: true,
                    ..
                }
            )
        })
        .count();
    let confidings = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Emitted::FrameChanged {
                    to: Frame::Confiding,
                    ..
                }
            )
        })
        .count();
    let counts = format!(
        "misfires {misfires}, misreadings {misreadings}, Pip's breaches {pip_breaches}, \
         refusals {refusals}, overheard {overheard}, confidings {confidings}"
    );
    assert!(misfires > 0 && misreadings > 0, "{counts}");
    assert!(pip_breaches > 0 && refusals > 0, "{counts}");
    assert!(overheard > 0 && confidings > 0, "{counts}");
    // The bulk of conversations still go as meant and as taken.
    let turns = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Emitted::TookUp {
                    overheard: false,
                    ..
                }
            )
        })
        .count();
    assert!(misreadings * 5 < turns, "{misreadings} of {turns} misread");
}

#[test]
fn a_breach_gets_an_answer() {
    // Whoever Pip steps outside the frame with responds to it on their next turn.
    let mut sim: Sim<Town> = Sim::new(1, town());
    let events = sim.step_until(at(28, 0, 0)).unwrap();
    let mut answered = 0;
    for (i, e) in events.iter().enumerate() {
        let Emitted::Beat {
            convo,
            who,
            breach: true,
            ..
        } = e
        else {
            continue;
        };
        let next = events[i + 1..].iter().find_map(|n| match n {
            Emitted::Beat { convo: c, who, .. } if c == convo => Some(*who),
            Emitted::ConversationEnded { convo: c, .. } if c == convo => Some(*who),
            _ => None,
        });
        if next.is_some_and(|n| n != *who) {
            answered += 1;
        }
    }
    assert!(answered > 0);
}

#[test]
fn a_conversation_in_progress_survives_a_snapshot() {
    // Nell buys bread from Mara at 10:34 on day 0; stop in the middle of it.
    let mut live: Sim<Town> = Sim::new(1, town());
    live.step_until(at(0, 10, 38)).unwrap();
    assert!(!live.state().conversations.is_empty());
    let mut restored = Sim::<Town>::restore(&live.snapshot(), town()).unwrap();
    assert_eq!(live.state().conversations, restored.state().conversations);
    assert_eq!(live.state().characters, restored.state().characters);
    let a = live.step_until(at(1, 12, 0)).unwrap();
    let b = restored.step_until(at(1, 12, 0)).unwrap();
    assert_eq!(a, b);
    assert_eq!(live.hash(), restored.hash());
}
