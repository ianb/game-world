//! Conversations as a short run of beats. The sim keeps the record and the clock; every
//! choice in a beat (what to do, how it comes out, how it is taken) belongs to the
//! participants' programs, through `Interlocutor`.
use super::Sim;
use crate::brain::{CharacterSet, Ctx, Trigger};
use crate::event::Event;
use crate::ids::VillagerId;
use crate::protocol::{Act, BidAnswer, Caught, Emitted, Frame, Gesture, Topic};
use crate::social::{
    BeatRecord, Carry, Conversation, Episode, Felt, Heard, Interlocutor, Plain, Situation,
    act_carry, act_gesture,
};
use crate::time::{TICKS_PER_MINUTE, Tick};
use rand_pcg::Pcg32;

/// How long a conversation runs on after both agree to go into a new frame.
const EXTENSION_MINUTES: u32 = 12;
/// Time between turns.
const BEAT_TICKS: u32 = 2 * TICKS_PER_MINUTE;

impl<C: CharacterSet> Sim<C> {
    /// Starts the record and the first beat. The caller has already checked the two can
    /// talk and set them both talking until `ends`.
    pub(super) fn open_conversation(
        &mut self,
        who: VillagerId,
        with: VillagerId,
        frame: Frame,
        topic: &Topic,
        ends: Tick,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let id = self.state.next_conversation;
        self.state.next_conversation += 1;
        self.state.conversations.insert(
            id,
            Conversation {
                id,
                a: who,
                b: with,
                frame,
                topic: topic.clone(),
                booked: topic.clone(),
                started: tick,
                ends,
                next: who,
                turns: [0, 0],
                beats: Vec::new(),
                bid: None,
                breach_reply: None,
                felt: [Felt::default(), Felt::default()],
            },
        );
        out.push(Emitted::ConversationStarted {
            tick,
            convo: id,
            who,
            with,
            place: self.state.villagers[who.index()].place,
            frame,
            topic: topic.clone(),
            cause: self.cause.clone(),
        });
        self.state
            .queue
            .schedule(tick.plus(TICKS_PER_MINUTE), Event::Beat { convo: id });
    }

    /// Everyone in earshot of `a` and `b` who is awake and not in a conversation.
    fn bystanders(&self, a: VillagerId, b: VillagerId) -> Vec<VillagerId> {
        let va = &self.state.villagers[a.index()];
        self.state
            .villagers
            .iter()
            .enumerate()
            .filter(|(i, v)| {
                *i != a.index()
                    && *i != b.index()
                    && !matches!(v.activity.as_str(), "sleep" | "talk")
                    && self.together(va, v)
            })
            .map(|(i, _)| VillagerId(u16::try_from(i).expect("few villagers")))
            .collect()
    }

    /// Who besides the two talking catches `did`, and how much: awake, in the same room (or
    /// both outdoors), and within the gesture's reach of the speaker.
    fn listeners(
        &self,
        speaker: VillagerId,
        hearer: VillagerId,
        did: &Act,
    ) -> Vec<(VillagerId, Caught)> {
        let carry = act_carry(did);
        let vs = &self.state.villagers[speaker.index()];
        let place = vs.place.index();
        let room = self.world.room_at(place, vs.pos.x, vs.pos.y);
        self.state
            .villagers
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let who = VillagerId(u16::try_from(i).expect("few villagers"));
                if who == speaker || who == hearer || v.activity == "sleep" || v.place != vs.place {
                    return None;
                }
                if self.world.room_at(place, v.pos.x, v.pos.y) != room {
                    return None;
                }
                let d = v.pos.x.abs_diff(vs.pos.x).max(v.pos.y.abs_diff(vs.pos.y));
                carry.caught_at(d).map(|c| (who, c))
            })
            .collect()
    }

    /// Raised voices heard from afar: who, roughly how, and nothing of what or to whom.
    fn hear_noise(
        &mut self,
        who: VillagerId,
        speaker: VillagerId,
        did: &Act,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let heard = Heard {
            tick,
            speaker,
            to: None,
            gesture: act_gesture(did),
            topic: None,
            about_me: false,
            caught: Caught::Noise,
        };
        let ctx = Ctx {
            tick,
            calendar: tick.calendar(),
            content: &self.content,
            world: &self.world,
            nav: &self.nav,
            villagers: &self.state.villagers,
            terrain: &self.state.terrain,
            stock: &self.state.stock,
            record: &self.state.record,
        };
        if let Some(m) = self.state.characters.interlocutor(who, &ctx) {
            m.hear(heard);
        }
        self.react(
            who,
            Trigger::Overheard {
                speaker,
                gesture: act_gesture(did),
                about_me: false,
                caught: Caught::Noise,
            },
            out,
        );
    }

    /// Runs `f` with `who`'s program and a view of the conversation from their side, where
    /// `other` is the person they are reacting to.
    fn with_mind<R>(
        &mut self,
        who: VillagerId,
        other: VillagerId,
        id: u32,
        bystanders: &[VillagerId],
        f: impl FnOnce(&mut dyn Interlocutor, &Situation<'_>, &mut Pcg32) -> R,
    ) -> R {
        let tick = self.state.tick;
        let convo = &self.state.conversations[&id];
        let ctx = Ctx {
            tick,
            calendar: tick.calendar(),
            content: &self.content,
            world: &self.world,
            nav: &self.nav,
            villagers: &self.state.villagers,
            terrain: &self.state.terrain,
            stock: &self.state.stock,
            record: &self.state.record,
        };
        let in_it = who == convo.a || who == convo.b;
        let situation = Situation {
            ctx: &ctx,
            me: who,
            other,
            frame: convo.frame,
            topic: &convo.topic,
            beats: &convo.beats,
            bystanders,
            initiator: who == convo.a,
            turn: if in_it {
                convo.turns[convo.side(who)]
            } else {
                0
            },
        };
        let rng = &mut self.state.rng.villagers[who.index()].mind;
        let mut plain = Plain::default();
        let mind: &mut dyn Interlocutor = match self.state.characters.interlocutor(who, &ctx) {
            Some(m) => m,
            None => &mut plain,
        };
        f(mind, &situation, rng)
    }

    /// One turn: the speaker answers a bid, responds to a breach, or acts; it comes out how
    /// it comes out; the other participant and any bystanders take it up.
    pub(super) fn beat(&mut self, id: u32, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let Some(c) = self.state.conversations.get(&id) else {
            return;
        };
        let still = |v: VillagerId| self.state.villagers[v.index()].activity == "talk";
        if tick >= c.ends || !still(c.a) || !still(c.b) {
            self.finish(id, None, out);
            return;
        }
        let speaker = c.next;
        let hearer = c.other(speaker);
        let frame = c.frame;
        let pending_bid = c.bid.clone().filter(|(to, _, _)| *to == speaker);
        let answering_breach = c.breach_reply == Some(speaker);
        let last_did = c.beats.last().map(|b| b.did.clone());
        let bystanders = self.bystanders(c.a, c.b);

        let meant = if let Some((_, to, _)) = pending_bid {
            let answer = self.with_mind(speaker, hearer, id, &bystanders, |m, s, r| {
                m.answer_bid(s, to, r)
            });
            Act::Answer { answer }
        } else if answering_breach && let Some(last) = last_did {
            self.with_mind(speaker, hearer, id, &bystanders, |m, s, r| {
                m.out_of_frame(s, &last, r)
            })
        } else {
            self.with_mind(speaker, hearer, id, &bystanders, |m, s, r| m.next_act(s, r))
        };
        let stance = self.with_mind(speaker, hearer, id, &bystanders, |m, s, _| m.stance(s));
        let did = self.with_mind(speaker, hearer, id, &bystanders, |m, s, r| {
            m.perform(s, &meant, stance, r)
        });
        let breach = match &did {
            Act::Gesture { gesture, .. } => !frame.allows(*gesture),
            Act::Bid { frame: to, .. } => !frame.bids_to(*to),
            Act::Answer { .. } => false,
        };
        out.push(Emitted::Beat {
            tick,
            convo: id,
            who: speaker,
            frame,
            stance,
            meant: meant.clone(),
            did: did.clone(),
            breach,
        });

        self.record_beat(id, speaker, frame, &meant, &did, breach, out);
        let kind_did = act_gesture(&did);
        let kind_meant = act_gesture(&meant);

        // The other participant takes it up.
        if kind_did != Gesture::Leave {
            self.take_up(
                id,
                hearer,
                speaker,
                &did,
                kind_meant,
                Caught::Addressed,
                &bystanders,
                out,
            );
            // Anyone else in earshot catches as much as the gesture carries to them.
            for (b, caught) in self.listeners(speaker, hearer, &did) {
                if caught == Caught::Noise {
                    self.hear_noise(b, speaker, &did, out);
                } else {
                    self.take_up(id, b, speaker, &did, kind_meant, caught, &bystanders, out);
                }
            }
        }
        if kind_did == Gesture::Leave {
            self.finish(id, Some(speaker), out);
            return;
        }
        let c = self
            .state
            .conversations
            .get_mut(&id)
            .expect("checked above");
        c.next = hearer;
        let next = tick.plus(BEAT_TICKS).min(c.ends);
        self.state.queue.schedule(next, Event::Beat { convo: id });
    }

    /// Writes a turn into the record: turn counts, a bid opened or answered (shifting the
    /// frame when accepted), and who must respond to a breach.
    #[allow(clippy::too_many_arguments)]
    fn record_beat(
        &mut self,
        id: u32,
        speaker: VillagerId,
        frame: Frame,
        meant: &Act,
        did: &Act,
        breach: bool,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let c = self
            .state
            .conversations
            .get_mut(&id)
            .expect("checked above");
        c.beats.push(BeatRecord {
            tick,
            who: speaker,
            meant: meant.clone(),
            did: did.clone(),
            breach,
        });
        let hearer = c.other(speaker);
        let side = c.side(speaker);
        c.turns[side] += 1;
        if let Act::Gesture {
            gesture: Gesture::Disclose,
            topic,
        } = did
        {
            c.felt[side].notes.push(format!("told {}", topic.key));
        }
        c.breach_reply = breach.then_some(hearer);
        let mut new_frame = None;
        match did {
            Act::Answer { answer } => {
                if let Some((_, to, topic)) = c.bid.take()
                    && *answer == BidAnswer::Toward
                {
                    new_frame = Some(to);
                    c.topic = topic;
                }
            }
            Act::Bid { frame: to, topic } if !breach => {
                c.bid = Some((hearer, *to, topic.clone()));
            }
            _ => {}
        }
        if let Some(to) = new_frame {
            c.frame = to;
            c.turns = [0, 0];
            let longer = tick.plus(EXTENSION_MINUTES * TICKS_PER_MINUTE);
            if longer > c.ends {
                c.ends = longer;
                let (a, b, booked) = (c.a, c.b, c.booked.clone());
                self.extend_talk(a, b, &booked, longer);
            }
            out.push(Emitted::FrameChanged {
                tick,
                convo: id,
                who: hearer,
                from: frame,
                to,
            });
        }
    }

    /// `who` takes up `speaker`'s turn as much as they caught of it. Everyone keeps it in
    /// memory; a participant's reading also goes into how they feel about the conversation.
    /// A bystander who hears their own name, or something said loud, gets an `Overheard`
    /// trigger.
    #[allow(clippy::too_many_arguments)]
    fn take_up(
        &mut self,
        id: u32,
        who: VillagerId,
        speaker: VillagerId,
        did: &Act,
        meant: Gesture,
        caught: Caught,
        bystanders: &[VillagerId],
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let up = self.with_mind(who, speaker, id, bystanders, |m, s, r| {
            m.take_up(s, speaker, did, r)
        });
        let kind_did = act_gesture(did);
        out.push(Emitted::TookUp {
            tick,
            convo: id,
            who,
            speaker,
            meant,
            did: kind_did,
            taken_as: up.taken_as,
            warmth: up.warmth,
            attribution: up.attribution,
            caught,
        });
        let topic = match did {
            Act::Gesture { topic, .. } | Act::Bid { topic, .. } => Some(topic.clone()),
            Act::Answer { .. } => None,
        };
        let about_me = topic.as_ref().is_some_and(|t| t.about == Some(who));
        let to = self.state.conversations.get(&id).map(|c| c.other(speaker));
        let heard = Heard {
            tick,
            speaker,
            to,
            gesture: up.taken_as,
            topic: topic.filter(|_| matches!(caught, Caught::Addressed | Caught::Close)),
            about_me,
            caught,
        };
        self.with_mind(who, speaker, id, bystanders, |m, _, _| m.hear(heard));
        if caught != Caught::Addressed {
            if about_me || act_carry(did) == Carry::Loud {
                self.react(
                    who,
                    Trigger::Overheard {
                        speaker,
                        gesture: up.taken_as,
                        about_me,
                        caught,
                    },
                    out,
                );
            }
            return;
        }
        let c = self
            .state
            .conversations
            .get_mut(&id)
            .expect("conversation exists");
        let side = c.side(who);
        let telling = !c.frame.scripted(up.taken_as);
        let felt = &mut c.felt[side];
        felt.feel(up.warmth, telling);
        if up.taken_as != kind_did {
            felt.notes.push(format!(
                "took {} as {}",
                kind_did.name(),
                up.taken_as.name()
            ));
        }
        if let Act::Answer { answer } = did
            && *answer != BidAnswer::Toward
        {
            felt.notes.push("was turned away".to_string());
        }
        if let Act::Gesture {
            gesture: Gesture::Disclose,
            topic,
        } = did
        {
            felt.notes.push(format!("was told about {}", topic.key));
        }
    }

    /// Ends a conversation: each participant keeps the episode as they experienced it,
    /// and anyone still talking goes back to what they were doing.
    pub(super) fn finish(&mut self, id: u32, ended_by: Option<VillagerId>, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let Some(c) = self.state.conversations.remove(&id) else {
            return;
        };
        for (me, side) in [(c.a, 0usize), (c.b, 1usize)] {
            let felt = &c.felt[side];
            let theirs = &c.felt[1 - side];
            let episode = Episode {
                tick: c.started,
                with: c.other(me),
                frame: c.frame,
                peak: felt.peak,
                end: felt.last,
                they_seemed: i8::try_from(felt.seemed.clamp(-3, 3)).unwrap_or(0),
                notes: felt.notes.clone(),
            };
            out.push(Emitted::ConversationEnded {
                tick,
                convo: id,
                who: me,
                with: c.other(me),
                frame: c.frame,
                my_peak: felt.peak,
                my_end: felt.last,
                their_peak: theirs.peak,
                their_end: theirs.last,
                ended_by,
            });
            // Re-insert briefly so the hook sees the conversation it is remembering.
            self.state.conversations.insert(id, c.clone());
            self.with_mind(me, c.other(me), id, &[], |m, _, _| m.remember(episode));
            self.state.conversations.remove(&id);
            // Ended early: stop talking now rather than at the planned end.
            let v = &mut self.state.villagers[me.index()];
            if v.activity == "talk" && v.talk_until.is_some_and(|t| t > tick) {
                v.talk_until = Some(tick);
                self.state.queue.schedule(
                    tick,
                    Event::TalkEnd {
                        who: me,
                        other: c.other(me),
                        topic: c.booked.clone(),
                    },
                );
            }
        }
    }

    /// Keeps both talking until `until`; the earlier `TalkEnd`s no longer match and pass.
    fn extend_talk(&mut self, a: VillagerId, b: VillagerId, booked: &Topic, until: Tick) {
        for (me, other) in [(a, b), (b, a)] {
            let v = &mut self.state.villagers[me.index()];
            if v.activity != "talk" {
                continue;
            }
            v.talk_until = Some(until);
            self.state.queue.schedule(
                until,
                Event::TalkEnd {
                    who: me,
                    other,
                    topic: booked.clone(),
                },
            );
        }
    }
}
