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
    /// How many of Tom's fish she has heard about, so a new catch is news.
    pub fish_stories_heard: u16,
}

impl Mara {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32, tom: &Tom) -> Vec<Planned> {
        let me = ctx.villager_id("mara").expect("mara exists");
        let mut plan = Plan::baseline(me, ctx);
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
            Trigger::Arrived => {
                match ctx.villager(me).activity.as_str() {
                    "bake" => self.batches_baked += 1,
                    "bring-pastry" => self.pastries_delivered += 1,
                    _ => {}
                }
                Vec::new()
            }
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
