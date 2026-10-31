//! Rob, the cooper. Sells a barrel to the tavern on Fridays; a slight from the same person twice in a week, he says out loud.
//! Description and design notes: `docs/characters/rob.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, counter_talk, go, hour, pay, stock, stock_of, think, think_aloud, villager,
    weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;
use std::collections::BTreeMap;

const BARREL_PRICE: u32 = 6;
const MAX_BARRELS: u16 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rob {
    /// Slights overheard this week, by who gave them.
    pub sore: BTreeMap<VillagerId, u8>,
    pub mind: Memory,
}

impl Character for Rob {
    const ID: &'static str = "rob";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken | crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "rob");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Mon {
            self.sore.clear();
        }
        if weekday(ctx) == Weekday::Fri {
            plan.at(16, 0, go(ctx, "bar_tavern", "deliver-barrel"));
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
            // A barrel for Tom, if one is ready.
            Trigger::Arrived if activity_of(ctx, "rob") == "deliver-barrel" => {
                if stock_of(ctx, "workbench_cooper", "barrel") == 0 {
                    vec![Command::Note("no barrel ready".into())]
                } else {
                    let mut cmds = vec![
                        stock(ctx, "workbench_cooper", "barrel", -1),
                        pay(ctx, "tom", "rob", BARREL_PRICE, "a barrel"),
                    ];
                    cmds.extend(counter_talk(ctx, "rob", "tom", "barrels"));
                    cmds
                }
            }
            // An afternoon at the bench makes a barrel.
            Trigger::Arrived
                if activity_of(ctx, "rob") == "make-barrels"
                    && hour(ctx) >= 13
                    && stock_of(ctx, "workbench_cooper", "barrel") < MAX_BARRELS =>
            {
                vec![stock(ctx, "workbench_cooper", "barrel", 1)]
            }
            // A slight sticks; the second from the same person in a week, he says aloud.
            Trigger::Overheard {
                speaker,
                about_me: true,
                ..
            } => {
                let me = villager(ctx, "rob");
                let n = self.sore.entry(*speaker).or_default();
                *n += 1;
                let topic = ctx.topic("what-was-said").about(me).with(*speaker);
                if *n >= 2 {
                    vec![think_aloud(topic, Blame::Them(*speaker), true, false, -2)]
                } else {
                    vec![think(topic, Blame::Them(*speaker), false, false, -1)]
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Rob {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 0,
            skill: 4,
            rejection_sensitivity: 6,
            charity: -2,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Rob: he likes a joke, and a dig or a rebuff goes straight in. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 1,
            G::Complain => -1,
            G::Tease | G::Pry => -2,
            G::Jab | G::Rebuff => -3,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Compliment
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward => gesture.warmth(),
        }
    }
}
