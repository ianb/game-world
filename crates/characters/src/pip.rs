//! Pip, a placeholder kid: roams the square all day, greets everyone who passes, and
//! turns up at the bakery at noon to pester Mara for a bun. Eclectic by design, to exercise
//! wandering, greetings and conversations.
use crate::mara::Mara;
use crate::toolbox::{Plan, activity_of, go_tile, pick, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pip {
    pub buns: u16,
    pub greetings_today: u16,
}

impl Pip {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        self.greetings_today = 0;
        let me = ctx.villager_id("pip").expect("pip exists");
        Plan::baseline(me, ctx).build()
    }

    pub fn react(
        &mut self,
        trigger: Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
        mara: &Mara,
    ) -> Vec<Command> {
        let me = ctx.villager_id("pip").expect("pip exists");
        match trigger {
            // Roaming: pick a random ground tile, walk there, pause, repeat.
            Trigger::Arrived if activity_of(ctx, "pip") == "roam" => {
                vec![wait_minutes(*pick(rng, &[3u32, 5, 8]), "roam")]
            }
            Trigger::Waited if activity_of(ctx, "pip") == "roam" => {
                let town = ctx.place_id("town").expect("town exists");
                match ctx.random_tile(town, content::Cell::Ground, rng) {
                    Some(p) => vec![go_tile(ctx, "town", p.x, p.y, "roam")],
                    None => vec![wait_minutes(5, "roam")],
                }
            }
            // At the bakery: ask for a bun.
            Trigger::Arrived if activity_of(ctx, "pip") == "pester" => {
                match ctx.villager_id("mara") {
                    Some(m) if ctx.villager(m).place == ctx.villager(me).place => {
                        vec![Command::Talk {
                            with: m,
                            topic: "sweets".into(),
                            minutes: 5,
                        }]
                    }
                    _ => vec![Command::Note("nobody at the bakery to pester".into())],
                }
            }
            Trigger::TalkEnded { other, topic } if topic == "sweets" => {
                // Mara cannot hand Pip anything; Pip notices the shelf and updates himself.
                if Some(other) == ctx.villager_id("mara") && mara.batches_baked > 0 {
                    self.buns += 1;
                    vec![Command::Note(format!("got a bun ({} so far)", self.buns))]
                } else {
                    vec![Command::Note("no bun today".into())]
                }
            }
            // Greets anyone passing, once per cooldown; the sim handles the pairing.
            Trigger::Passing { other } | Trigger::Encounter { other }
                if activity_of(ctx, "pip") != "sleep" =>
            {
                self.greetings_today += 1;
                vec![Command::Greet { with: other }]
            }
            _ => Vec::new(),
        }
    }
}

#[allow(dead_code)]
fn _id(v: VillagerId) -> u16 {
    v.0
}
