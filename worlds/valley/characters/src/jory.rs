//! Jory Hollins, Will's son. Sweet on Lottie: calls on her on Sundays and always finds something nice to say.
//! Description and design notes: `docs/characters/jory.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, chat, go, seek, tend_fields, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Act, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_next_act};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jory {
    pub mind: Memory,
}

impl Character for Jory {
    const ID: &'static str = "jory";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken | crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "jory");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Sun {
            plan.at(14, 0, go(ctx, "bench_square", "call-on-lottie"));
        }
        plan.build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "jory") == "farm" => {
                tend_fields(ctx, "jory", trigger, "hollins-fields", "farm", rng)
            }
            // On Sunday he goes to find her, wherever she is this side of the river.
            Trigger::Arrived if activity_of(ctx, "jory") == "call-on-lottie" => {
                let me = villager(ctx, "jory");
                let lottie = villager(ctx, "lottie");
                match seek(ctx, me, lottie, Frame::SmallTalk, ctx.topic("day"), 20) {
                    Some(talk) => vec![talk],
                    None => vec![Command::Note("no sign of Lottie".into())],
                }
            }
            // Meeting her by chance, he stops.
            Trigger::Encounter { other } if *other == villager(ctx, "lottie") => {
                let me = villager(ctx, "jory");
                vec![chat(ctx, me, *other, Frame::SmallTalk, "day", 15)?]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Jory {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 1,
            skill: 3,
            rejection_sensitivity: 6,
            charity: 1,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Jory: a kind word is everything; being rebuffed is the worst. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Compliment => 2,
            G::Joke => 1,
            G::Tease | G::Leave => -2,
            G::Rebuff => -3,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }

    /// With Lottie he finds something nice to say, once a conversation.
    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        if s.other == villager(s.ctx, "lottie")
            && s.frame == Frame::SmallTalk
            && !s.i_did(Gesture::Compliment)
        {
            Act::Gesture {
                gesture: Gesture::Compliment,
                topic: s.topic.clone(),
            }
        } else {
            default_next_act(self, s, rng)
        }
    }
}
