//! Mara, the baker. Mostly her content schedule, with two personal touches: she counts
//! what she bakes, and when Tom is in a bad mood she brings him a pastry before her
//! evening drink.
use crate::tom::Tom;
use crate::toolbox::{Plan, go};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mara {
    /// Baking sessions completed. Nobody else tracks this.
    pub batches_baked: u16,
    pub pastries_delivered: u16,
}

impl Mara {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32, tom: &Tom) -> Vec<Planned> {
        let me = ctx.villager_id("mara").expect("mara exists");
        let mut plan = Plan::baseline(me, ctx);
        if tom.mood < 0 {
            plan.at(17, 30, go(ctx, "bar_tavern", "bring-pastry"));
        }
        plan.build()
    }

    pub fn react(&mut self, trigger: Trigger, ctx: &Ctx<'_>) -> Vec<Command> {
        let me = ctx.villager_id("mara").expect("mara exists");
        if trigger == Trigger::Arrived {
            match ctx.villager(me).activity.as_str() {
                "bake" => self.batches_baked += 1,
                "bring-pastry" => self.pastries_delivered += 1,
                _ => {}
            }
        }
        Vec::new()
    }
}
