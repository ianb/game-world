//! Bess, the sexton. Tends the graves by day and will not be out after dark.
//! Description and design notes: `docs/characters/bess.md`.
use crate::Character;
use crate::toolbox::{activity_of, between, go, in_building, think, villager, work_area};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bess {
    pub mind: Memory,
}

impl Character for Bess {
    const ID: &'static str = "bess";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => {
                crate::Fondness::Indifferent
            }
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            // The graves: tidy round one, then another.
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "bess") == "tend-graves" => {
                work_area(ctx, "bess", trigger, "graveyard", "tend-graves", 20, 1, rng)
            }
            // After dark she will not be out: home, and she frets about it.
            Trigger::Look { .. }
                if !between(ctx, (6, 0), (19, 30))
                    && !in_building(ctx, villager(ctx, "bess"), "sexton")
                    && activity_of(ctx, "bess") != "walking" =>
            {
                let me = villager(ctx, "bess");
                vec![
                    think(
                        ctx.topic("the-dark").about(me),
                        Blame::World,
                        true,
                        false,
                        -2,
                    ),
                    go(ctx, "chair_bess", "rest"),
                ]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Bess {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: 1,
            skill: 4,
            rejection_sensitivity: 6,
            charity: 1,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Bess: kindness matters to her and teasing frightens her. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Sympathise => 2,
            G::Joke | G::Leave => -1,
            G::Tease | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }
}
