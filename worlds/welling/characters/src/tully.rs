//! Tully, who keeps the gas stall on the yard. Placeholder: counts the take, worries when it is thin.
//! Notes: `docs/worlds/welling/people.md`.
use crate::Character;
use crate::toolbox::{
    CASKS, GAS_TRADE_PRICE, Plan, REFINED_GAS, STALL, activity_of, go, pay, stock, stock_of,
    villager,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

/// What the stall holds at most.
const STALL_HOLDS: u16 = 6;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tully {
    pub mind: Memory,
}

impl Character for Tully {
    const ID: &'static str = "tully";

    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let mut plan = Plan::baseline(villager(ctx, "tully"), ctx);
        // Before the stall opens: down to the Mire's hold for the night's refined gas.
        plan.at(8, 30, go(ctx, CASKS, "fetch-gas"));
        plan.build()
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Resumed if activity_of(ctx, "tully") == "fetch-gas" => {
                let room = STALL_HOLDS.saturating_sub(stock_of(ctx, STALL, REFINED_GAS));
                let afford = ctx.coins(villager(ctx, "tully")) / GAS_TRADE_PRICE;
                let n = stock_of(ctx, CASKS, REFINED_GAS)
                    .min(room)
                    .min(u16::try_from(afford).unwrap_or(u16::MAX));
                let mut cmds = Vec::new();
                if n > 0 {
                    cmds.push(stock(ctx, CASKS, REFINED_GAS, -i32::from(n)));
                    cmds.push(stock(ctx, STALL, REFINED_GAS, i32::from(n)));
                    cmds.push(pay(
                        ctx,
                        "tully",
                        "ora",
                        u32::from(n) * GAS_TRADE_PRICE,
                        "refined gas",
                    ));
                }
                cmds.push(go(ctx, STALL, "sell"));
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Tully {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 1,
            skill: 7,
            rejection_sensitivity: 3,
            charity: 0,
            discretion: 3,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to them: custom is custom; a jab is remembered. Every gesture is listed, so a new one
    /// makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Compliment | G::Request => 1,
            G::Jab | G::Rebuff => -2,
            G::Thank
            | G::Agree
            | G::Ask
            | G::Tease
            | G::Pry
            | G::Tell
            | G::Joke
            | G::Complain
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward => gesture.warmth(),
        }
    }
}
