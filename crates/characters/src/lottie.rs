//! Lottie, Hester's granddaughter. Twice a week she walks to where the road leaves the valley and looks down it.
//! Description and design notes: `docs/characters/lottie.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, go, go_area, think, villager, wait_minutes, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lottie {
    pub mind: Memory,
}

impl Character for Lottie {
    const ID: &'static str = "lottie";
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "lottie");
        let mut plan = Plan::baseline(me, ctx);
        if matches!(weekday(ctx), Weekday::Wed | Weekday::Sun)
            && let Some(go) = go_area(ctx, "road-out", "look-down-road", rng)
        {
            plan.at(15, 0, go);
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
            Trigger::Arrived if activity_of(ctx, "lottie") == "look-down-road" => {
                let me = villager(ctx, "lottie");
                vec![
                    think(ctx.topic("leaving").about(me), Blame::World, true, true, -1),
                    wait_minutes(40, "look-down-road"),
                ]
            }
            Trigger::Waited if activity_of(ctx, "lottie") == "look-down-road" => {
                vec![go(ctx, "table_pike", "chores")]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Lottie {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 1,
            skill: 5,
            rejection_sensitivity: 5,
            charity: 1,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Lottie: news from anywhere else is welcome; prying and teasing are not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Compliment | G::Tell => 1,
            G::Tease => -1,
            G::Pry => -2,
            G::Ask
            | G::Agree
            | G::Joke
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
