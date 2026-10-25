//! Nell, a pensioner. Money comes to her from outside the town once a week and leaves
//! through the bakery and the tavern, which is most of what makes the economy a loop.
//! She is shy: anxious with anyone she does not know well or with others listening, at
//! ease only with people she has come to like. When something goes wrong she blames
//! herself and expects it to keep happening.
use crate::toolbox::{Plan, activity_of, buy_ale, buy_bread, counter_talk, pay, think, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Mishap, Stance};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_stance};
use sim::time::Weekday;

pub const PENSION: u32 = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nell {
    pub pensions_received: u16,
    pub loaves_bought: u16,
    pub days_without_bread: u8,
    pub mind: Memory,
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

    pub fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived if activity_of(ctx, "nell") == "shop" => {
                if let Some(mut cmds) = buy_bread(ctx, "nell") {
                    self.loaves_bought += 1;
                    self.days_without_bread = 0;
                    cmds.extend(counter_talk(ctx, "nell", "mara", "bread"));
                    cmds
                } else {
                    self.days_without_bread = self.days_without_bread.saturating_add(1);
                    vec![Command::Note("no bread to be had".into())]
                }
            }
            Trigger::Arrived if activity_of(ctx, "nell") == "drink" => {
                let mut cmds = buy_ale(ctx, "nell")
                    .unwrap_or_else(|| vec![Command::Note("no ale tonight".into())]);
                cmds.extend(counter_talk(ctx, "nell", "tom", "ale"));
                cmds
            }
            Trigger::Mishap {
                what: Mishap::Tripped,
            } => vec![think("tripped", Blame::Me, true, false, -1)],
            Trigger::Passing { other } if activity_of(ctx, "nell") != "sleep" => {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Nell {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -2,
            warmth: 1,
            skill: 4,
            rejection_sensitivity: 7,
            charity: 0,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<String> {
        if self.days_without_bread >= 1 {
            Some("going without".into())
        } else if ctx.coins(me) < 3 {
            Some("money".into())
        } else {
            None
        }
    }

    fn stance(&self, s: &Situation<'_>) -> Stance {
        let r = self.mind.regard(s.other);
        if r.feeling >= 1 && r.familiarity >= 4 {
            default_stance(self, s)
        } else if r.familiarity < 4 || !s.private() {
            Stance::Anxious
        } else {
            default_stance(self, s)
        }
    }
}
