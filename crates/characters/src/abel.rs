//! Abel, the woodcutter. Keeps to himself in the woods; on Saturdays he brings firewood to the tavern and to Nell.
//! Description and design notes: `docs/characters/abel.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, go, pay, stock, stock_of, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;

const MAX_WOOD: u16 = 20;
const WOOD_PRICE: u32 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Abel {
    pub mind: Memory,
}

impl Character for Abel {
    const ID: &'static str = "abel";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => {
                crate::Fondness::Indifferent
            }
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "abel");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Sat {
            plan.at(10, 0, go(ctx, "bar_tavern", "deliver-wood"));
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
            // A spell of chopping adds to the pile.
            Trigger::Arrived
                if activity_of(ctx, "abel") == "chop"
                    && stock_of(ctx, "woodpile_abel", "wood") < MAX_WOOD =>
            {
                vec![stock(ctx, "woodpile_abel", "wood", 2)]
            }
            // Saturday: wood for the tavern, then for Nell.
            Trigger::Arrived if activity_of(ctx, "abel") == "deliver-wood" => {
                let mut cmds = Vec::new();
                if stock_of(ctx, "woodpile_abel", "wood") >= 2 {
                    cmds.push(stock(ctx, "woodpile_abel", "wood", -2));
                    cmds.push(pay(ctx, "tom", "abel", 2 * WOOD_PRICE, "firewood"));
                }
                cmds.push(go(ctx, "table_nell", "deliver-wood-to-nell"));
                cmds
            }
            Trigger::Arrived if activity_of(ctx, "abel") == "deliver-wood-to-nell" => {
                let mut cmds = Vec::new();
                if stock_of(ctx, "woodpile_abel", "wood") >= 1
                    && ctx.coins(villager(ctx, "nell")) >= WOOD_PRICE
                {
                    cmds.push(stock(ctx, "woodpile_abel", "wood", -1));
                    cmds.push(pay(ctx, "nell", "abel", WOOD_PRICE, "firewood"));
                }
                cmds.push(go(ctx, "table_abel", "rest"));
                cmds
            }
            // A nod for anyone he meets; he does not stop.
            Trigger::Encounter { other } | Trigger::Passing { other } => {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Abel {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -2,
            warmth: 0,
            skill: 3,
            rejection_sensitivity: 3,
            charity: 1,
            discretion: 8,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Abel: few words, plainly meant; questions are an intrusion. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank | G::Agree => 1,
            G::Ask | G::Tease => -1,
            G::Pry => -2,
            G::Tell
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
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
