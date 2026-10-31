//! Pike, who hangs about the gas stall. Placeholder: short of money, low in the rain, never still for long.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pike {
    pub mind: Memory,
}

impl Character for Pike {
    const ID: &'static str = "pike";

    fn react(
        &mut self,
        trigger: &Trigger,
        _ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Encounter { other } => vec![Command::Greet { with: *other }],
            _ => return None,
        })
    }
}

impl Interlocutor for Pike {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: -1,
            skill: 3,
            rejection_sensitivity: 8,
            charity: -1,
            discretion: 2,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to them: hears a slight in most things. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Offer => 2,
            G::Decline | G::Rebuff | G::Tease => -3,
            G::Thank
            | G::Agree
            | G::Ask
            | G::Pry
            | G::Tell
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward => gesture.warmth(),
        }
    }
}
