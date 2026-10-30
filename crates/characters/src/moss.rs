//! Moss, the shepherd. Talks to his sheep and to nobody else until he has had a drink.
//! Description and design notes: `docs/characters/moss.md`.
use crate::Character;
use crate::toolbox::{
    activity_of, around, buy_ale_nearest, day_number, think_aloud, villager, work_area,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{BidAnswer, Blame, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_answer_bid};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moss {
    /// The day he last had a drink, so he knows whether he has had one tonight.
    pub drank_on: Option<u32>,
    pub mind: Memory,
}

impl Character for Moss {
    const ID: &'static str = "moss";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Indifferent,
            content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            // With the sheep, alone, he talks to them.
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "moss") == "shepherd" => {
                let me = villager(ctx, "moss");
                let mut cmds = work_area(ctx, "moss", trigger, "pasture", "shepherd", 45, 0, rng);
                if matches!(trigger, Trigger::Arrived) && around(ctx, me, 8).is_empty() {
                    cmds.push(think_aloud(
                        ctx.topic("sheep").about(me),
                        Blame::World,
                        false,
                        false,
                        1,
                    ));
                }
                cmds
            }
            Trigger::Arrived if activity_of(ctx, "moss") == "drink" => {
                match buy_ale_nearest(ctx, "moss") {
                    Some(cmds) => {
                        self.drank_on = Some(day_number(ctx));
                        cmds
                    }
                    None => vec![Command::Note("no drink to be had".into())],
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Moss {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -3,
            warmth: 0,
            skill: 2,
            rejection_sensitivity: 3,
            charity: 1,
            discretion: 9,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Moss: being left in peace is a kindness; questions are not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Leave | G::Sympathise => 1,
            G::Ask => -1,
            G::Pry => -3,
            G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }

    /// Until he has had a drink today he will not be drawn.
    fn answer_bid(&mut self, s: &Situation<'_>, frame: Frame, rng: &mut Pcg32) -> BidAnswer {
        if self.drank_on == Some(day_number(s.ctx)) {
            default_answer_bid(self, s, frame, rng)
        } else {
            BidAnswer::Away { account: false }
        }
    }
}
