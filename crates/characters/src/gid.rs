//! Gid, the miller. Will not mill on Fridays, and a fall means the day is against him.
//! Description and design notes: `docs/characters/gid.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, counter_talk, go, pay, stock, stock_of, think_aloud, villager, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture, Stance};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_stance};
use sim::time::Weekday;

const FLOUR_PRICE: u32 = 2;
const MAX_FLOUR: u16 = 12;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gid {
    /// Whether he has given up on today.
    pub given_up: bool,
    pub mind: Memory,
}

impl Character for Gid {
    const ID: &'static str = "gid";
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "gid");
        self.given_up = false;
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Thu {
            plan.at(10, 0, go(ctx, "counter_bakery", "deliver-flour"));
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
            // Not after a fall: home again.
            Trigger::Arrived if self.given_up && activity_of(ctx, "gid") == "mill" => {
                vec![go(ctx, "table_mill", "rest")]
            }
            // A turn at the stones grinds flour.
            Trigger::Arrived
                if activity_of(ctx, "gid") == "mill"
                    && stock_of(ctx, "millstone", "flour") < MAX_FLOUR =>
            {
                vec![stock(ctx, "millstone", "flour", 2)]
            }
            // Thursday: flour for the bakery.
            Trigger::Arrived if activity_of(ctx, "gid") == "deliver-flour" => {
                let n = stock_of(ctx, "millstone", "flour").min(4);
                if n == 0 {
                    vec![Command::Note("no flour to take".into())]
                } else {
                    let mut cmds = vec![
                        stock(ctx, "millstone", "flour", -i32::from(n)),
                        stock(ctx, "pantry_bakery", "flour", i32::from(n)),
                        pay(ctx, "mara", "gid", u32::from(n) * FLOUR_PRICE, "flour"),
                    ];
                    cmds.extend(counter_talk(ctx, "gid", "mara", "flour-run"));
                    cmds
                }
            }
            // A fall: the day is against him, and he goes home.
            Trigger::Mishap { .. } if !self.given_up => {
                self.given_up = true;
                let me = villager(ctx, "gid");
                vec![
                    think_aloud(ctx.topic("bad-luck").about(me), Blame::Luck, true, true, -2),
                    go(ctx, "table_mill", "rest"),
                ]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Gid {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 1,
            skill: 5,
            rejection_sensitivity: 5,
            charity: 0,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Gid: agreement and thanks; teasing about luck is no joke. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank | G::Agree => 1,
            G::Complain => -1,
            G::Tease | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Joke
            | G::Compliment
            | G::Request
            | G::Offer
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

    /// Guarded with everyone on a Friday.
    fn stance(&self, s: &Situation<'_>) -> Stance {
        if s.ctx.calendar.weekday == Weekday::Fri {
            Stance::Guarded
        } else {
            default_stance(self, s)
        }
    }
}
