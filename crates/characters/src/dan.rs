//! Dan, Ashby's hired hand, new to the valley. Introduces himself to everyone he has not met.
//! Description and design notes: `docs/characters/dan.md`.
use crate::Character;
use crate::toolbox::{activity_of, chat, tend_fields, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dan {
    /// Everyone he has introduced himself to.
    pub met: BTreeSet<VillagerId>,
    pub mind: Memory,
}

impl Character for Dan {
    const ID: &'static str = "dan";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Indifferent,
            content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "dan") == "farm" => {
                tend_fields(ctx, "dan", trigger, "ashby-fields", "farm", rng)
            }
            // Anyone he has not met yet: he stops and gives his name, or at least a word.
            Trigger::Encounter { other } | Trigger::Passing { other }
                if !self.met.contains(other) && activity_of(ctx, "dan") != "sleep" =>
            {
                let me = villager(ctx, "dan");
                match chat(ctx, me, *other, Frame::Greeting, "names", 5) {
                    Some(talk) => {
                        self.met.insert(*other);
                        vec![talk]
                    }
                    None => vec![Command::Greet { with: *other }],
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Dan {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 1,
            skill: 4,
            rejection_sensitivity: 5,
            charity: 1,
            discretion: 4,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Dan: being asked about himself means he is welcome; being turned away means he is not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Ask => 2,
            G::Tell | G::Compliment => 1,
            G::Decline | G::Leave => -1,
            G::Rebuff => -3,
            G::Agree
            | G::Joke
            | G::Tease
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
