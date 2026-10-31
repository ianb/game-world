//! Ottilie, Fen's wife. Mends nets by the shack, and on Friday nights at the alehouse tells a story to anyone she meets.
//! Description and design notes: `docs/characters/ottilie.md`.
use crate::Character;
use crate::toolbox::{activity_of, chat, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Act, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_next_act};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ottilie {
    pub mind: Memory,
}

impl Character for Ottilie {
    const ID: &'static str = "ottilie";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Fond,
            crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // At the alehouse, whoever she meets gets a story.
            Trigger::Encounter { other } if activity_of(ctx, "ottilie") == "drink" => {
                let me = villager(ctx, "ottilie");
                vec![chat(ctx, me, *other, Frame::SmallTalk, "a-story", 20)?]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Ottilie {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 2,
            skill: 7,
            rejection_sensitivity: 2,
            charity: 2,
            discretion: 3,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Ottilie: a laugh is the best answer; walking out on a story is the worst. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 3,
            G::Agree | G::Compliment => 1,
            G::Leave | G::Rebuff => -2,
            G::Ask
            | G::Tell
            | G::Tease
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

    /// A story has a joke in it.
    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        if s.frame == Frame::SmallTalk && s.topic.is("a-story") && !s.i_did(Gesture::Joke) {
            Act::Gesture {
                gesture: Gesture::Joke,
                topic: s.topic.clone(),
            }
        } else {
            default_next_act(self, s, rng)
        }
    }
}
