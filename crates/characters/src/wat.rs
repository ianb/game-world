//! Wat, Dot's husband. Helps himself from the keg when Dot is not keeping the bar.
//! Description and design notes: `docs/characters/wat.md`.
use crate::Character;
use crate::toolbox::{activity_of, buy_ale_nearest, stock, stock_of, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wat {
    pub mind: Memory,
}

impl Character for Wat {
    const ID: &'static str = "wat";
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // A drink: free while Dot is not at the bar, paid for when she is.
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "wat") == "drink" => {
                let mut cmds = if activity_of(ctx, "dot") != "keep-bar"
                    && stock_of(ctx, "bar_alehouse", "ale") > 0
                {
                    vec![
                        stock(ctx, "bar_alehouse", "ale", -1),
                        Command::Note("a drink on the house".into()),
                    ]
                } else {
                    buy_ale_nearest(ctx, "wat").unwrap_or_default()
                };
                cmds.push(wait_minutes(60, "drink"));
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Wat {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 1,
            skill: 3,
            rejection_sensitivity: 4,
            charity: 1,
            discretion: 3,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Wat: a drink offered and a joke; complaint and prying are not welcome. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Offer | G::Joke => 2,
            G::Sympathise => 1,
            G::Complain | G::Pry | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Tease
            | G::Compliment
            | G::Request
            | G::Thank
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
