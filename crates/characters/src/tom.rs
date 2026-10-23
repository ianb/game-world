//! Tom, who keeps the tavern. The eclectic one: a mood only he tracks, Sundays off at the
//! pond, late openings when he is low, and an evening check on whether anyone came.
use crate::mara::Mara;
use crate::toolbox::{Plan, activity_of, go_tile, pick, together, wait_minutes, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::time::Weekday;

/// Fishing spots on the pond's west bank, in building-free ground.
const FISHING_SPOTS: [(u16, u16); 3] = [(29, 21), (29, 22), (29, 23)];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tom {
    /// Negative is grumpy. Nudged by how the evening went.
    pub mood: i8,
    pub days_since_day_off: u8,
    pub fish_caught: u16,
}

impl Default for Tom {
    fn default() -> Self {
        Tom {
            mood: -1,
            days_since_day_off: 0,
            fish_caught: 0,
        }
    }
}

impl Tom {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("tom").expect("tom exists");
        let mut plan = Plan::baseline(me, ctx);

        if weekday(ctx) == Weekday::Sun {
            // Day off: no bar. Fish from late morning, then home for a long evening.
            let worked = self.days_since_day_off;
            self.days_since_day_off = 0;
            let (x, y) = *pick(rng, &FISHING_SPOTS);
            plan.drop_activity("tend-bar").drop_activity("close-up");
            plan.at(
                0,
                1,
                Command::Note(format!("Sunday off after {worked} days")),
            );
            plan.at(10, 30, go_tile(ctx, "town", x, y, "fish"));
            plan.at(16, 0, crate::toolbox::go(ctx, "table_house", "rest"));
            return plan.build();
        }
        self.days_since_day_off = self.days_since_day_off.saturating_add(1);

        if self.mood <= -3 {
            plan.shift((10, 0), (11, 0));
            plan.at(
                0,
                1,
                Command::Note(format!("opening late, mood {}", self.mood)),
            );
        }
        // Mid-evening check on the room.
        plan.at(19, 30, wait_minutes(30, "tend-bar"));
        plan.build()
    }

    pub fn react(
        &mut self,
        trigger: Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
        mara: &Mara,
    ) -> Vec<Command> {
        match trigger {
            Trigger::Arrived if activity_of(ctx, "tom") == "fish" => {
                // A fishing afternoon: wait a while, maybe catch something, repeat.
                vec![wait_minutes(*pick(rng, &[40u32, 55, 70]), "fish")]
            }
            Trigger::Waited if activity_of(ctx, "tom") == "fish" => {
                if *pick(rng, &[true, false, false]) {
                    self.fish_caught += 1;
                }
                self.mood = self.mood.saturating_add(1).min(5);
                vec![wait_minutes(*pick(rng, &[40u32, 55, 70]), "fish")]
            }
            Trigger::Waited => {
                // 20:00 on a working day: is anyone here?
                let company = together(ctx, "tom", "mara") && activity_of(ctx, "mara") == "drink";
                let delivered_today = mara.pastries_delivered > 0;
                let before = self.mood;
                self.mood = if company || delivered_today {
                    self.mood.saturating_add(2).min(5)
                } else {
                    self.mood.saturating_sub(1).max(-5)
                };
                let why = match (company, delivered_today) {
                    (true, _) => "Mara is here drinking",
                    (false, true) => "nobody here, but a pastry came by",
                    (false, false) => "nobody came",
                };
                vec![
                    Command::Note(format!("mood {before} -> {} ({why})", self.mood)),
                    wait_minutes(120, "tend-bar"),
                ]
            }
            _ => Vec::new(),
        }
    }
}
