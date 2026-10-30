//! Martha Ashby, Silas's sister. Keeps bees and tells them, out loud, whatever she last heard.
//! Description and design notes: `docs/characters/martha.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, counter_talk, go, hour, pay, stock, stock_of, think_aloud, villager,
    wait_minutes, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::{TICKS_PER_DAY, Weekday};

const HIVES: [&str; 3] = ["hive_1", "hive_2", "hive_3"];
const HONEY_PRICE: u32 = 3;
const MAX_HONEY: u16 = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Martha {
    /// Which hive she is at.
    pub hive: u8,
    pub mind: Memory,
}

impl Character for Martha {
    const ID: &'static str = "martha";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "martha");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Fri {
            plan.at(10, 30, go(ctx, "counter_bakery", "sell-honey"));
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
            // At each hive she tells the bees the latest thing she heard said, out loud.
            Trigger::Arrived if activity_of(ctx, "martha") == "tend-bees" => {
                let news = self
                    .mind
                    .heard()
                    .since(ctx.tick.minus(TICKS_PER_DAY))
                    .words()
                    .last()
                    .and_then(|h| h.topic.clone());
                let mut cmds = Vec::new();
                if let Some(topic) = news {
                    cmds.push(think_aloud(topic, Blame::World, false, false, 0));
                }
                cmds.push(wait_minutes(25, "tend-bees"));
                cmds
            }
            Trigger::Waited if activity_of(ctx, "martha") == "tend-bees" && hour(ctx) < 12 => {
                self.hive = (self.hive + 1) % 3;
                vec![go(ctx, HIVES[usize::from(self.hive)], "tend-bees")]
            }
            Trigger::Arrived
                if activity_of(ctx, "martha") == "jar-honey"
                    && stock_of(ctx, "jars_martha", "honey") < MAX_HONEY =>
            {
                vec![stock(ctx, "jars_martha", "honey", 1)]
            }
            // Friday: a jar for the bakery.
            Trigger::Arrived if activity_of(ctx, "martha") == "sell-honey" => {
                if stock_of(ctx, "jars_martha", "honey") == 0 {
                    vec![Command::Note("no honey to sell".into())]
                } else {
                    let mut cmds = vec![
                        stock(ctx, "jars_martha", "honey", -1),
                        stock(ctx, "shelf_bakery", "honey", 1),
                        pay(ctx, "mara", "martha", HONEY_PRICE, "honey"),
                    ];
                    cmds.extend(counter_talk(ctx, "martha", "mara", "honey"));
                    cmds
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Martha {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 1,
            skill: 5,
            rejection_sensitivity: 4,
            charity: 1,
            discretion: 3,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Martha: news, above all. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Tell => 2,
            G::Thank | G::Disclose => 1,
            G::Pry => -1,
            G::Jab => -2,
            G::Ask
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
