//! Jenna, Rob's wife. On washing days at the pond she talks to anyone who stops, and passes on what she knows.
//! Description and design notes: `docs/characters/jenna.md`.
use crate::Character;
use crate::toolbox::{activity_of, chat, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jenna {
    pub mind: Memory,
}

impl Character for Jenna {
    const ID: &'static str = "jenna";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Fond,
            content::AnimalKind::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // At the washtub she talks to anyone who stops nearby; anyone walking by gets a word.
            Trigger::Encounter { other } | Trigger::Passing { other }
                if activity_of(ctx, "jenna") == "wash" =>
            {
                let me = villager(ctx, "jenna");
                vec![
                    chat(ctx, me, *other, Frame::SmallTalk, "washing", 15)
                        .unwrap_or(Command::Greet { with: *other }),
                ]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Jenna {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 1,
            skill: 5,
            rejection_sensitivity: 3,
            charity: 1,
            discretion: 2,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Jenna: news and confidences are what she likes best. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Tell | G::Disclose => 2,
            G::Joke => 1,
            G::Leave | G::Decline => -1,
            G::Rebuff => -2,
            G::Ask
            | G::Agree
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
