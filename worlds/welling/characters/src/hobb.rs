//! Hobb, the lasher who keeps the rafts tied. Placeholder: walks the bollards, proud of the knots, short when stuck.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use crate::toolbox::{activity_of, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hobb {
    pub mind: Memory,
}

impl Character for Hobb {
    const ID: &'static str = "hobb";

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Resumed if activity_of(ctx, "hobb") == "lash" => {
                vec![wait_minutes(30, "lash")]
            }
            Trigger::Passing { other } => vec![Command::Greet { with: *other }],
            _ => return None,
        })
    }
}

impl Interlocutor for Hobb {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 0,
            skill: 5,
            rejection_sensitivity: 2,
            charity: 1,
            discretion: 8,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to them: few words; agreement is enough. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Agree | G::Thank => 1,
            G::Awkward => -1,
            G::Ask
            | G::Tease
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
            | G::Jab
            | G::Rebuff => gesture.warmth(),
        }
    }
}
