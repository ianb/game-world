//! Mara, the baker. Mostly her content schedule, with two personal touches: she counts
//! what she bakes, and when Tom is in a bad mood she brings him a pastry before her
//! evening drink.
use crate::tom::Tom;
use crate::toolbox::{Plan, buy_ale, go, satisfy, stock, stock_of};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};

pub const FLOUR_PER_BATCH: u16 = 2;
pub const LOAVES_PER_BATCH: i32 = 6;
/// With this many loaves left from yesterday she skips a bake (stale bread goes out
/// first, so this only matters when the first bake of the day already happened).
pub const LOAVES_ENOUGH: u16 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mara {
    /// Baking sessions completed. Nobody else tracks this.
    pub batches_baked: u16,
    pub pastries_delivered: u16,
    /// How many of Tom's fish she has heard about, so a new catch is news.
    pub fish_stories_heard: u16,
}

impl Mara {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32, tom: &Tom) -> Vec<Planned> {
        let me = ctx.villager_id("mara").expect("mara exists");
        let mut plan = Plan::baseline(me, ctx);
        // Yesterday's bread goes out before the first bake.
        let stale = i32::from(stock_of(ctx, "shelf_bakery", "bread"));
        if stale > 0 {
            plan.at(6, 35, stock(ctx, "shelf_bakery", "bread", -stale));
        }
        if tom.mood < 0 {
            plan.at(17, 30, go(ctx, "bar_tavern", "bring-pastry"));
            plan.at(
                0,
                1,
                Command::Note(format!(
                    "Tom seems low (mood {}); pastry at 17:30",
                    tom.mood
                )),
            );
        }
        plan.build()
    }

    pub fn react(&mut self, trigger: Trigger, ctx: &Ctx<'_>, tom: &Tom) -> Vec<Command> {
        let me = ctx.villager_id("mara").expect("mara exists");
        let tom_id = ctx.villager_id("tom");
        match trigger {
            Trigger::Arrived => match ctx.villager(me).activity.as_str() {
                // Two measures of flour make six loaves. No flour, no bread.
                "bake" => {
                    if stock_of(ctx, "shelf_bakery", "bread") >= LOAVES_ENOUGH {
                        vec![Command::Note("shelf still full; no bake today".into())]
                    } else if stock_of(ctx, "pantry_bakery", "flour") >= FLOUR_PER_BATCH {
                        self.batches_baked += 1;
                        vec![
                            stock(ctx, "pantry_bakery", "flour", -i32::from(FLOUR_PER_BATCH)),
                            stock(ctx, "shelf_bakery", "bread", LOAVES_PER_BATCH),
                        ]
                    } else {
                        vec![Command::Note("no flour; nothing to bake".into())]
                    }
                }
                "bring-pastry" => {
                    self.pastries_delivered += 1;
                    Vec::new()
                }
                "drink" => {
                    buy_ale(ctx, "mara").unwrap_or_else(|| vec![Command::Note("no ale".into())])
                }
                _ => Vec::new(),
            },
            Trigger::Need { need, level } => satisfy(ctx, "mara", need, level, "table_house"),
            // Settling in at the tavern with Tom behind the bar: a proper chat.
            Trigger::Encounter { other }
                if Some(other) == tom_id && ctx.villager(me).activity == "drink" =>
            {
                let topic = if tom.fish_caught > self.fish_stories_heard {
                    "fishing"
                } else {
                    "the day"
                };
                vec![
                    Command::Note(format!(
                        "Tom looks {}",
                        if tom.mood < 0 { "glum" } else { "fine" }
                    )),
                    Command::Talk {
                        with: other,
                        topic: topic.into(),
                        minutes: 10,
                    },
                ]
            }
            Trigger::TalkEnded { other, topic } if Some(other) == tom_id && topic == "fishing" => {
                self.fish_stories_heard = tom.fish_caught;
                vec![Command::Note(format!(
                    "heard about the fish ({} total)",
                    tom.fish_caught
                ))]
            }
            // A word for anyone she passes, unless she is in the middle of baking.
            Trigger::Passing { other }
                if !matches!(ctx.villager(me).activity.as_str(), "bake" | "sleep") =>
            {
                vec![Command::Greet { with: other }]
            }
            _ => Vec::new(),
        }
    }
}
