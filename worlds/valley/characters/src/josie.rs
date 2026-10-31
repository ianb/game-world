//! Josie, the maid at the manor. On Saturday nights, once Pell is asleep, she slips out to the alehouse.
//! Description and design notes: `docs/characters/josie.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, buy_ale_nearest, go, think, villager, wait_minutes, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Josie {
    pub mind: Memory,
}

impl Character for Josie {
    const ID: &'static str = "josie";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Fond,
            crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "josie");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Sat {
            plan.at(22, 15, go(ctx, "stove_manor", "listen-for-pell"));
        }
        plan.build()
    }

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived if activity_of(ctx, "josie") == "listen-for-pell" => {
                if activity_of(ctx, "pell") == "sleep" {
                    let me = villager(ctx, "josie");
                    vec![
                        think(
                            ctx.topic("nights-out").about(me),
                            Blame::Me,
                            false,
                            false,
                            2,
                        ),
                        go(ctx, "table_alehouse_1", "night-out"),
                    ]
                } else {
                    vec![go(ctx, "bed_josie", "sleep")]
                }
            }
            Trigger::Arrived if activity_of(ctx, "josie") == "night-out" => {
                let mut cmds = buy_ale_nearest(ctx, "josie").unwrap_or_default();
                cmds.push(wait_minutes(75, "night-out"));
                cmds
            }
            Trigger::Waited if activity_of(ctx, "josie") == "night-out" => {
                vec![go(ctx, "bed_josie", "sleep")]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Josie {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 2,
            skill: 4,
            rejection_sensitivity: 4,
            charity: 2,
            discretion: 3,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Josie: a joke and a little teasing are fun; prying is dangerous. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 2,
            G::Compliment | G::Tease => 1,
            G::Complain => -1,
            G::Pry | G::Rebuff => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward => gesture.warmth(),
        }
    }
}
