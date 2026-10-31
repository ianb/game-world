//! Nessa, a fisher who cooks what she catches on the cutter. Placeholder: cheered by fine days, flattened by rain.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use crate::toolbox::{activity_of, object, stock, stock_of, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

const TABLE: &str = "table_nessa";
/// The most fish kept aboard.
const KEEPS: u16 = 6;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nessa {
    pub mind: Memory,
}

impl Character for Nessa {
    const ID: &'static str = "nessa";

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Resumed | Trigger::Waited
                if activity_of(ctx, "nessa") == "fish" =>
            {
                let mut cmds = Vec::new();
                if stock_of(ctx, TABLE, "fish") < KEEPS {
                    cmds.push(stock(ctx, TABLE, "fish", 1));
                }
                cmds.push(wait_minutes(60, "fish"));
                cmds
            }
            // Two at a time on the stove; cooking takes out what the seep put in.
            Trigger::Arrived | Trigger::Resumed if activity_of(ctx, "nessa") == "cook" => {
                let n = stock_of(ctx, TABLE, "fish").min(2);
                if n == 0 {
                    return Some(Vec::new());
                }
                vec![
                    stock(ctx, TABLE, "fish", -i32::from(n)),
                    stock(ctx, TABLE, "cooked-fish", i32::from(n)),
                ]
            }
            Trigger::Arrived | Trigger::Resumed
                if activity_of(ctx, "nessa") == "eat"
                    && stock_of(ctx, TABLE, "cooked-fish") > 0 =>
            {
                vec![Command::Consume {
                    object: object(ctx, TABLE),
                    good: "cooked-fish".into(),
                }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Nessa {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 2,
            skill: 6,
            rejection_sensitivity: 3,
            charity: 2,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to them: warm to warmth, cool to complaint. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke | G::Compliment | G::Thank => 2,
            G::Complain => -1,
            G::Agree
            | G::Ask
            | G::Tease
            | G::Pry
            | G::Tell
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
