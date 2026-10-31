//! Grue, a fisher who eats the catch raw. Placeholder: odd in the way raw eaters are, lonely.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use crate::toolbox::{activity_of, stock, stock_of, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

const TABLE: &str = "table_grue";
/// The most fish kept at home.
const KEEPS: u16 = 6;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grue {
    pub mind: Memory,
}

impl Character for Grue {
    const ID: &'static str = "grue";

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "grue") == "fish" => {
                let mut cmds = Vec::new();
                if stock_of(ctx, TABLE, "fish") < KEEPS {
                    cmds.push(stock(ctx, TABLE, "fish", 1));
                }
                cmds.push(wait_minutes(60, "fish"));
                cmds
            }
            // Raw, at the table.
            Trigger::Arrived
                if activity_of(ctx, "grue") == "eat" && stock_of(ctx, TABLE, "fish") > 0 =>
            {
                vec![stock(ctx, TABLE, "fish", -1)]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Grue {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: 0,
            skill: 3,
            rejection_sensitivity: 6,
            charity: 1,
            discretion: 7,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to them: glad of any kindness, wary of questions. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Sympathise | G::Offer | G::Invite => 2,
            G::Pry => -2,
            G::Thank
            | G::Agree
            | G::Ask
            | G::Tease
            | G::Tell
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Disclose
            | G::Decline
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
