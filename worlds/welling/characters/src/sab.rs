//! Sab, who works the hood over the vent and refines in the afternoons. Placeholder: restless, uneasy after dark.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use crate::toolbox::{CASKS, RAW_GAS, activity_of, refine, stock, stock_of, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

/// The most raw gas the casks take.
const CASKS_HOLD: u16 = 12;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sab {
    pub mind: Memory,
}

impl Character for Sab {
    const ID: &'static str = "sab";

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "sab") == "capture" => {
                let mut cmds = Vec::new();
                if stock_of(ctx, CASKS, RAW_GAS) < CASKS_HOLD {
                    cmds.push(stock(ctx, CASKS, RAW_GAS, 1));
                }
                cmds.push(wait_minutes(45, "capture"));
                cmds
            }
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "sab") == "refine" => {
                refine(ctx)
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Sab {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
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

    /// What each gesture means to them: likes a joke, takes questions badly. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke | G::Invite => 2,
            G::Pry | G::Ask => -1,
            G::Thank
            | G::Agree
            | G::Tease
            | G::Tell
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
