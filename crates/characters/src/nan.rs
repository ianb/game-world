//! Nan, Gid's mother, deaf. Hails everyone who passes the mill bench; keeps only what is said to her face.
//! Description and design notes: `docs/characters/nan.md`.
use crate::Character;
use crate::toolbox::activity_of;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Caught, Gesture};
use sim::social::{Heard, Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nan {
    pub mind: Memory,
}

impl Character for Nan {
    const ID: &'static str = "nan";
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // What others say to each other does not reach her.
            Trigger::Overheard { .. } => Vec::new(),
            // From the bench she hails everyone who goes by.
            Trigger::Passing { other } | Trigger::Encounter { other }
                if activity_of(ctx, "nan") == "sit" =>
            {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Nan {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 2,
            skill: 3,
            rejection_sensitivity: 2,
            charity: 2,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Nan: thanks and a kind word. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank | G::Compliment => 2,
            G::Leave | G::Rebuff => -1,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }

    /// She keeps only what is said to her face.
    fn hear(&mut self, h: Heard) {
        if h.caught == Caught::Addressed {
            self.mind.hear(h);
        }
    }
}
