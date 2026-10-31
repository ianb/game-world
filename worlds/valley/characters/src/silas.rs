//! Silas Ashby, farmer and miser. Counts his coins every night and takes any drop hard.
//! Description and design notes: `docs/characters/silas.md`.
use crate::Character;
use crate::toolbox::{activity_of, tend_fields, think, villager, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Silas {
    /// His coins when he last counted them.
    pub counted: u32,
    pub mind: Memory,
}

impl Character for Silas {
    const ID: &'static str = "silas";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken | crate::Animal::Sheep => crate::Fondness::Annoyed,
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "silas") == "farm" => {
                tend_fields(ctx, "silas", trigger, "ashby-fields", "farm", rng)
            }
            // Counting: less than last night, and it stays with him.
            Trigger::Arrived if activity_of(ctx, "silas") == "count-coins" => {
                let me = villager(ctx, "silas");
                let now = ctx.coins(me);
                let before = std::mem::replace(&mut self.counted, now);
                if now < before {
                    vec![
                        think(ctx.topic("money").about(me), Blame::World, true, true, -2),
                        wait_minutes(30, "count-coins"),
                    ]
                } else {
                    vec![wait_minutes(15, "count-coins")]
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Silas {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -2,
            warmth: -2,
            skill: 4,
            rejection_sensitivity: 5,
            charity: -3,
            discretion: 7,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Silas: being given something is the only warm thing; being asked for anything is not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Offer => 2,
            G::Thank => 1,
            G::Ask | G::Joke => -1,
            G::Request | G::Tease | G::Jab => -2,
            G::Pry => -3,
            G::Tell
            | G::Agree
            | G::Compliment
            | G::Complain
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
