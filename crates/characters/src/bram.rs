//! Bram, the smith. Loud at the anvil, so anyone near hears him at it; burns Rook's charcoal.
//! Description and design notes: `docs/characters/bram.md`.
use crate::Character;
use crate::toolbox::{activity_of, stock, stock_of, think_aloud, villager, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bram {
    pub mind: Memory,
}

impl Character for Bram {
    const ID: &'static str = "bram";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => {
                crate::Fondness::Indifferent
            }
        }
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // Hammering, an hour at a time: charcoal goes on the fire, and he can be heard.
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "bram") == "smith" => {
                let me = villager(ctx, "bram");
                if stock_of(ctx, "forge_smithy", "charcoal") == 0 {
                    vec![
                        think_aloud(
                            ctx.topic("charcoal").about(me),
                            Blame::World,
                            false,
                            false,
                            -1,
                        ),
                        wait_minutes(60, "smith"),
                    ]
                } else {
                    vec![
                        stock(ctx, "forge_smithy", "charcoal", -1),
                        think_aloud(ctx.topic("ironwork").about(me), Blame::Me, false, false, 1),
                        wait_minutes(60, "smith"),
                    ]
                }
            }
            // A shout to anyone passing.
            Trigger::Passing { other } => vec![Command::Greet { with: *other }],
            _ => return None,
        })
    }
}

impl Interlocutor for Bram {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 3,
            warmth: 2,
            skill: 4,
            rejection_sensitivity: 1,
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

    /// What each gesture means to Bram: he likes a joke and a ribbing; very little offends him. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 2,
            G::Compliment | G::Agree | G::Tease => 1,
            G::Jab | G::Rebuff => -1,
            G::Ask
            | G::Tell
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
