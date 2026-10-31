//! Lark, who runs the drinks at the Kettle. Placeholder: brightest with a full bar, lonely when it empties.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lark {
    pub mind: Memory,
}

impl Character for Lark {
    const ID: &'static str = "lark";

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

impl Interlocutor for Lark {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 3,
            warmth: 2,
            skill: 8,
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

    /// What each gesture means to them: all talk is welcome talk, except leaving. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke | G::Tell | G::Invite => 2,
            G::Leave => -1,
            G::Thank
            | G::Agree
            | G::Ask
            | G::Tease
            | G::Pry
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
