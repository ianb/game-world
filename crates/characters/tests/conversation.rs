//! Conversations between the real characters: what four weeks of them contain, and that a
//! conversation in progress survives a snapshot.
use characters::Town;
use content::{Content, load_dir};
use sim::Sim;
use sim::protocol::{Act, BidAnswer, Caught, Emitted, Frame};
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
    // Seed 2: on seed 1 nobody's bid to confide is taken up in these four weeks.
    let mut sim: Sim<Town> = Sim::new(2, town());
    let events = sim.step_until(at(28, 0, 0)).unwrap();
    let pip = sim.ctx().villager_id("pip").unwrap();
    let misfires = events
        .iter()
        .filter(|e| matches!(e, Emitted::Beat { meant, did, .. } if meant != did))
        .count();
    let misreadings = events
        .iter()
        .filter(|e| {
            matches!(e, Emitted::TookUp { did, taken_as, caught: Caught::Addressed, .. } if did != taken_as)
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
                    caught: Caught::Close | Caught::Gist,
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
                    caught: Caught::Addressed,
                    ..
                }
            )
        })
        .count();
    assert!(misreadings * 5 < turns, "{misreadings} of {turns} misread");
}

#[test]
fn how_much_is_overheard_follows_the_gesture_and_the_distance() {
    let mut sim: Sim<Town> = Sim::new(1, town());
    sim.step_until(at(28, 0, 0)).unwrap();
    let c = &sim.state().characters;
    let heard = [
        &c.mara.mind,
        &c.tom.mind,
        &c.pip.mind,
        &c.nell.mind,
        &c.hal.mind,
    ];
    let all: Vec<&sim::social::Heard> = heard.iter().flat_map(|m| m.heard.iter()).collect();
    assert!(all.iter().any(|h| h.caught == Caught::Close));
    assert!(all.iter().any(|h| h.caught == Caught::Gist));
    // Words only reach those close enough; the gist and raised voices carry no topic.
    for h in &all {
        match h.caught {
            Caught::Addressed | Caught::Close => {}
            Caught::Gist | Caught::Noise => assert!(h.topic.is_none(), "{h:?}"),
        }
        if h.caught == Caught::Noise {
            assert!(h.to.is_none());
            assert!(
                h.gesture.carry().reach().2 > 0,
                "only carrying gestures reach that far"
            );
        }
    }
}

#[test]
fn conversations_start_within_reach() {
    // Positions come from the movement events; everyone starts the day in bed, so the
    // first move puts each villager somewhere known.
    let mut sim: Sim<Town> = Sim::new(1, town());
    let start: Vec<_> = sim
        .state()
        .villagers
        .iter()
        .map(|v| (v.place, v.pos.x, v.pos.y))
        .collect();
    let events = sim.step_until(at(14, 0, 0)).unwrap();
    let mut at_ = start;
    let mut started = 0;
    for e in &events {
        match e {
            Emitted::Moved {
                who, place, x, y, ..
            } => at_[usize::from(who.0)] = (*place, *x, *y),
            Emitted::ConversationStarted {
                tick, who, with, ..
            } => {
                let (a, b) = (at_[usize::from(who.0)], at_[usize::from(with.0)]);
                let d = a.1.abs_diff(b.1).max(a.2.abs_diff(b.2));
                assert!(
                    a.0 == b.0 && d <= sim::brain::TALK_REACH,
                    "{tick:?}: {who:?} and {with:?} {d} tiles apart"
                );
                started += 1;
            }
            _ => {}
        }
    }
    assert!(started > 100, "{started} conversations");
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
