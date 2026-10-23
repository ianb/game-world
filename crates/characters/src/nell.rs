//! Nell, a pensioner. Money comes to her from outside the town once a week and leaves
//! through the bakery and the tavern, which is most of what makes the economy a loop.
use crate::toolbox::{Plan, activity_of, buy_ale, buy_bread, pay, satisfy, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::time::Weekday;

pub const PENSION: u32 = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nell {
    pub pensions_received: u16,
    pub loaves_bought: u16,
    pub days_without_bread: u8,
}

impl Nell {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("nell").expect("nell exists");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Mon {
            plan.at(9, 0, pay(ctx, "outside", "nell", PENSION, "pension"));
            plan.at(9, 0, Command::Note("pension day".into()));
        }
        plan.build()
    }

    pub fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) -> Vec<Command> {
        match trigger {
            Trigger::Arrived if activity_of(ctx, "nell") == "shop" => {
                if let Some(cmds) = buy_bread(ctx, "nell") {
                    self.loaves_bought += 1;
                    self.days_without_bread = 0;
                    cmds
                } else {
                    self.days_without_bread = self.days_without_bread.saturating_add(1);
                    vec![Command::Note("no bread to be had".into())]
                }
            }
            Trigger::Arrived if activity_of(ctx, "nell") == "drink" => {
                buy_ale(ctx, "nell").unwrap_or_else(|| vec![Command::Note("no ale tonight".into())])
            }
            Trigger::Need { need, level } => satisfy(ctx, "nell", *need, *level, "table_nell"),
            Trigger::Passing { other } if activity_of(ctx, "nell") != "sleep" => {
                vec![Command::Greet { with: *other }]
            }
            _ => Vec::new(),
        }
    }
}
