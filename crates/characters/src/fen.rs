//! Fen, the fisherman. After a good day's catch he tells everyone about it; sells fish to the alehouse on Tuesdays.
//! Description and design notes: `docs/characters/fen.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, buy_ale_nearest, pay, stock, stock_of, think_aloud, villager, work_area,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Act, Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_next_act};

/// Chance in a hundred of a fish for each spell at the water.
const CATCH_CHANCE: i16 = 35;
/// The chance label for a fish on a wait at the river.
pub const CATCHES: &str = "fen-catches";
const FISH_PRICE: u32 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fen {
    /// Fish caught today.
    pub caught: u8,
    pub mind: Memory,
}

impl Character for Fen {
    const ID: &'static str = "fen";
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "fen");
        self.caught = 0;
        Plan::baseline(me, ctx).build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "fen") == "fish" => {
                let me = villager(ctx, "fen");
                let mut cmds = work_area(ctx, "fen", trigger, "riverbank", "fish", 30, 0, rng);
                if matches!(trigger, Trigger::Waited) && ctx.chance(rng, me, CATCHES, CATCH_CHANCE)
                {
                    self.caught += 1;
                    cmds.push(stock(ctx, "creel_fisher", "fish", 1));
                    if self.caught == 3 {
                        cmds.push(think_aloud(
                            ctx.topic("the-catch").about(me),
                            Blame::Me,
                            false,
                            false,
                            2,
                        ));
                    }
                }
                cmds
            }
            // Tuesday evening at the alehouse: the catch for Dot, then a drink.
            Trigger::Arrived if activity_of(ctx, "fen") == "drink" => {
                let n = stock_of(ctx, "creel_fisher", "fish").min(3);
                let mut cmds = Vec::new();
                if n > 0 && ctx.coins(villager(ctx, "dot")) >= u32::from(n) * FISH_PRICE {
                    cmds.push(stock(ctx, "creel_fisher", "fish", -i32::from(n)));
                    cmds.push(pay(ctx, "dot", "fen", u32::from(n) * FISH_PRICE, "fish"));
                }
                cmds.extend(buy_ale_nearest(ctx, "fen").unwrap_or_default());
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Fen {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 1,
            skill: 4,
            rejection_sensitivity: 3,
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

    /// What each gesture means to Fen: being asked or praised about his fishing is the best thing. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Ask | G::Compliment => 2,
            G::Agree => 1,
            G::Tease | G::Jab => -2,
            G::Tell
            | G::Joke
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
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }

    /// After a good day he tells anyone about the catch.
    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        if self.caught >= 3 && !s.i_did(Gesture::Tell) {
            Act::Gesture {
                gesture: Gesture::Tell,
                topic: s.ctx.topic("the-catch").about(s.me),
            }
        } else {
            default_next_act(self, s, rng)
        }
    }
}
