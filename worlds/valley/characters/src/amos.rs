//! Amos, an old soldier. Raised voices take him back to the war, and he goes home to sit alone; at the alehouse he tells war stories.
//! Description and design notes: `docs/characters/amos.md`.
use crate::Character;
use crate::toolbox::{activity_of, go, seek, think, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Caught, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amos {
    pub mind: Memory,
}

impl Character for Amos {
    const ID: &'static str = "amos";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Fond,
            content::AnimalKind::Sheep => crate::Fondness::Wary,
        }
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // Raised voices, or a dig he overhears: home, alone, and the war on his mind.
            Trigger::Overheard {
                caught, gesture, ..
            } if (*caught == Caught::Noise
                || matches!(gesture, Gesture::Jab | Gesture::Rebuff))
                && activity_of(ctx, "amos") != "sleep" =>
            {
                let me = villager(ctx, "amos");
                vec![
                    think(ctx.topic("the-war").about(me), Blame::World, true, true, -2),
                    go(ctx, "chair_amos", "sit-alone"),
                ]
            }
            // At the alehouse, whoever he meets hears about the war.
            Trigger::Encounter { other } if activity_of(ctx, "amos") == "drink" => {
                let me = villager(ctx, "amos");
                let topic = ctx.topic("the-war").about(me);
                vec![seek(ctx, me, *other, Frame::SmallTalk, topic, 15)?]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Amos {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 1,
            skill: 5,
            rejection_sensitivity: 6,
            charity: 2,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Amos: being asked about the old days warms him; a dig does not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Ask | G::Thank => 2,
            G::Sympathise => 1,
            G::Tease => -1,
            G::Jab | G::Rebuff => -2,
            G::Tell
            | G::Agree
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
