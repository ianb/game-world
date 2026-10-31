//! Ora, a refiner at the Mire. Placeholder: steady at the bench, proud of clean gas, short with crowds.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use crate::toolbox::{activity_of, refine};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ora {
    pub mind: Memory,
}

impl Character for Ora {
    const ID: &'static str = "ora";

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "ora") == "refine" => {
                refine(ctx)
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Ora {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 0,
            skill: 6,
            rejection_sensitivity: 4,
            charity: 0,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to them: plain dealing counts; teasing does not. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank | G::Agree => 1,
            G::Tease | G::Jab => -2,
            G::Ask
            | G::Pry
            | G::Tell
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
