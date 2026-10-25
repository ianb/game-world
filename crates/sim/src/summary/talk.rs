//! Conversation lines: one per conversation, saying what happened in it beyond its business
//! (frames reached, bids turned away, misfires, misreadings, breaches, things overheard).
//! A conversation with none of that is routine by its pair and frames.
use super::{Draft, Summarizer, normalize};
use crate::ids::VillagerId;
use crate::protocol::{Act, Attribution, BidAnswer, Cause, Emitted, Frame, SummaryKind};
use crate::social::act_gesture;
use crate::time::Tick;
use crate::views::describe_act;
use std::fmt::Write as _;

#[derive(Debug)]
pub(super) struct Talk {
    tick: Tick,
    a: VillagerId,
    b: VillagerId,
    frames: Vec<String>,
    cause: Cause,
    bid: Option<(VillagerId, Frame, String)>,
    /// The last thing said, for what a bystander overheard.
    last: Option<Act>,
    clauses: Vec<String>,
}

impl Summarizer {
    pub(super) fn conversation(&mut self, e: &Emitted) {
        match e {
            Emitted::ConversationStarted {
                tick,
                convo,
                who,
                with,
                frame,
                topic,
                cause,
                ..
            } => {
                let about = self.topic(topic);
                let t = Talk {
                    tick: *tick,
                    a: *who,
                    b: *with,
                    frames: vec![format!("{} about {about}", frame.name())],
                    cause: cause.clone(),
                    bid: None,
                    last: None,
                    clauses: Vec::new(),
                };
                self.talks.insert(*convo, t);
            }
            Emitted::Beat {
                convo,
                who,
                meant,
                did,
                breach,
                ..
            } => self.beat(*convo, *who, meant, did, *breach),
            Emitted::TookUp {
                convo,
                who,
                speaker,
                did,
                taken_as,
                warmth,
                attribution,
                overheard,
                ..
            } => {
                let (name, speaker) =
                    (self.name(*who).to_string(), self.name(*speaker).to_string());
                // What a bystander catches is the topic of a bid to confide or a disclosure.
                let caught = self
                    .talks
                    .get(convo)
                    .and_then(|t| t.last.as_ref())
                    .filter(|a| {
                        matches!(
                            act_gesture(a),
                            crate::protocol::Gesture::Invite | crate::protocol::Gesture::Disclose
                        )
                    })
                    .and_then(|a| match a {
                        Act::Bid { topic, .. } | Act::Gesture { topic, .. } => Some(topic),
                        Act::Answer { .. } => None,
                    })
                    .map(|t| self.topic(t));
                let Some(t) = self.talks.get_mut(convo) else {
                    return;
                };
                if *overheard {
                    if let Some(caught) = caught {
                        push(&mut t.clauses, format!("{name} overheard \"{caught}\""));
                    }
                } else if taken_as != did {
                    let mut c = format!(
                        "{name} took {speaker}'s {} as {}",
                        did.name(),
                        taken_as.name()
                    );
                    if *warmth < 0 && *attribution == Attribution::Character {
                        let _ = write!(c, " and blamed {speaker}");
                    }
                    push(&mut t.clauses, c);
                }
            }
            Emitted::ConversationEnded { convo, .. } => self.ended(*convo),
            _ => {}
        }
    }

    fn beat(&mut self, convo: u32, who: VillagerId, meant: &Act, did: &Act, breach: bool) {
        let name = self.name(who).to_string();
        let bidder_name = self
            .talks
            .get(&convo)
            .and_then(|t| t.bid.as_ref())
            .map(|(b, _, _)| self.name(*b).to_string());
        let label = |t: &crate::protocol::Topic| self.topic(t);
        let did_text = describe_act(did, &label);
        let bid_about = match did {
            Act::Bid { topic, .. } => Some(self.topic(topic)),
            _ => None,
        };
        let Some(t) = self.talks.get_mut(&convo) else {
            return;
        };
        match did {
            Act::Bid { frame, .. } => {
                t.bid = Some((who, *frame, bid_about.unwrap_or_default()));
            }
            Act::Answer { answer } => {
                if let Some((_, frame, subject)) = t.bid.take() {
                    let bidder = bidder_name.unwrap_or_default();
                    match answer {
                        BidAnswer::Toward if frame == Frame::Confiding => {
                            push(
                                &mut t.clauses,
                                format!("{bidder} confided in {name} about {subject}"),
                            );
                        }
                        BidAnswer::Toward => {
                            t.frames.push(format!("{} about {subject}", frame.name()));
                        }
                        BidAnswer::Away { account } => push(
                            &mut t.clauses,
                            format!(
                                "{name} turned away {bidder}'s bid for {} about {subject}{}",
                                frame.name(),
                                if *account { ", giving a reason" } else { "" }
                            ),
                        ),
                        BidAnswer::Against => push(
                            &mut t.clauses,
                            format!(
                                "{name} rebuffed {bidder}'s bid for {} about {subject}",
                                frame.name()
                            ),
                        ),
                    }
                }
            }
            Act::Gesture { .. } => {}
        }
        // A dig or a rebuff they meant: someone taking something out on the other.
        if meant == did
            && matches!(
                act_gesture(did),
                crate::protocol::Gesture::Jab | crate::protocol::Gesture::Rebuff
            )
        {
            push(
                &mut t.clauses,
                format!("{name} meant {}", act_gesture(did).name()),
            );
        }
        if meant != did {
            let c = match (meant, did) {
                (Act::Answer { .. }, Act::Answer { .. }) => {
                    format!("{name} meant to give a reason and did not")
                }
                _ => format!(
                    "{name}'s {} came out as {}",
                    act_gesture(meant).name(),
                    act_gesture(did).name()
                ),
            };
            push(&mut t.clauses, c);
        }
        if breach {
            push(
                &mut t.clauses,
                format!("{name} stepped out of the frame ({did_text})"),
            );
        }
        t.last = Some(did.clone());
    }

    fn ended(&mut self, convo: u32) {
        let Some(t) = self.talks.remove(&convo) else {
            return;
        };
        let (a, b) = (self.name(t.a).to_string(), self.name(t.b).to_string());
        let (lo, hi) = if t.a.0 <= t.b.0 { (&a, &b) } else { (&b, &a) };
        let pair = format!("talks:{}:{}", lo.to_lowercase(), hi.to_lowercase());
        self.habits
            .insert(pair.clone(), (Some(t.a), format!("{a} and {b} talk")));
        self.day(t.tick).habit_keys.insert(pair);
        let mut text = format!("{a} and {b}: {}", t.frames.join(", then "));
        // Without incident, the pair and the frames decide routine; with one, the line is
        // news unless the same thing happened before.
        let key = if t.clauses.is_empty() {
            format!(
                "talk:{}:{}:{}",
                lo.to_lowercase(),
                hi.to_lowercase(),
                normalize(&text)
            )
        } else {
            text.push_str(". ");
            text.push_str(&t.clauses.join("; "));
            format!("talk:{}", normalize(&text))
        };
        let tick = t.tick;
        self.draft(
            tick,
            Draft {
                tick,
                who: Some(t.a),
                kind: SummaryKind::Talk,
                text,
                cause: Some(t.cause),
                keys: vec![key],
            },
        );
    }
}

fn push(clauses: &mut Vec<String>, c: String) {
    if !clauses.contains(&c) {
        clauses.push(c);
    }
}
