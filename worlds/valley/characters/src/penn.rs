//! Penn, the potter. Sometimes reaches the kiln and cannot think what he came for.
//! Description and design notes: `docs/characters/penn.md`.
use crate::Character;
use crate::toolbox::{activity_of, go, stock, stock_of, think, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

/// Chance in a hundred that he forgets at the kiln.
const FORGET_CHANCE: i16 = 25;
/// The chance label for forgetting at the kiln.
pub const FORGETS: &str = "penn-forgets";
const MAX_POTS: u16 = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Penn {
    pub mind: Memory,
}

impl Character for Penn {
    const ID: &'static str = "penn";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Annoyed,
            crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived
                if activity_of(ctx, "penn") == "throw-pots"
                    && stock_of(ctx, "wheel_dory", "pots") < MAX_POTS =>
            {
                vec![stock(ctx, "wheel_dory", "pots", 1)]
            }
            // At the kiln: fire what is thrown, unless it has gone out of his head.
            Trigger::Arrived if activity_of(ctx, "penn") == "fire-pots" => {
                let me = villager(ctx, "penn");
                if ctx.chance(rng, me, FORGETS, FORGET_CHANCE) {
                    vec![
                        think(
                            ctx.topic("forgetting").about(me),
                            Blame::Me,
                            false,
                            false,
                            -1,
                        ),
                        go(ctx, "wheel_dory", "throw-pots"),
                    ]
                } else {
                    let n = i32::from(stock_of(ctx, "wheel_dory", "pots"));
                    vec![
                        stock(ctx, "wheel_dory", "pots", -n),
                        stock(ctx, "kiln_dory", "pots", n),
                    ]
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Penn {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 2,
            skill: 3,
            rejection_sensitivity: 2,
            charity: 3,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Penn: a joke and a kind word; almost nothing offends him. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 2,
            G::Thank | G::Sympathise => 1,
            G::Jab | G::Pry => -1,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
