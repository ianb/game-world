//! Wynn, the herbalist. Gathers along the wood's edge and offers a remedy to anyone she knows has come to grief.
//! Description and design notes: `docs/characters/wynn.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, chat, go_area, pay, stock, stock_of, villager, weekday, work_area,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Emitted, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::{TICKS_PER_DAY, Weekday};

const MAX_REMEDIES: u16 = 6;
const REMEDY_PRICE: u32 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wynn {
    pub mind: Memory,
}

impl Character for Wynn {
    const ID: &'static str = "wynn";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Fond,
            content::AnimalKind::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "wynn");
        let mut plan = Plan::baseline(me, ctx);
        if matches!(weekday(ctx), Weekday::Mon | Weekday::Wed | Weekday::Fri)
            && let Some(go) = go_area(ctx, "wood-edge", "gather-herbs", rng)
        {
            plan.at(9, 0, go);
        }
        plan.build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            // Gathering: a handful at each spot, then on along the wood's edge.
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "wynn") == "gather-herbs" => {
                let mut cmds = work_area(
                    ctx,
                    "wynn",
                    trigger,
                    "wood-edge",
                    "gather-herbs",
                    25,
                    0,
                    rng,
                );
                if matches!(trigger, Trigger::Arrived)
                    && stock_of(ctx, "shelf_wynn", "remedy") < MAX_REMEDIES
                {
                    cmds.push(stock(ctx, "shelf_wynn", "remedy", 1));
                }
                cmds
            }
            // Someone she saw or heard come to grief lately: she offers something for it.
            Trigger::Encounter { other }
                if stock_of(ctx, "shelf_wynn", "remedy") > 0 && Wynn::ailing(ctx, *other) =>
            {
                let me = villager(ctx, "wynn");
                vec![chat(ctx, me, *other, Frame::Transaction, "remedy", 10)?]
            }
            Trigger::TalkEnded { other, topic } if topic.is("remedy") => {
                if stock_of(ctx, "shelf_wynn", "remedy") > 0 && ctx.coins(*other) >= REMEDY_PRICE {
                    vec![
                        stock(ctx, "shelf_wynn", "remedy", -1),
                        pay(
                            ctx,
                            &ctx.villager(*other).id,
                            "wynn",
                            REMEDY_PRICE,
                            "a remedy",
                        ),
                    ]
                } else {
                    vec![Command::Note("they would not take it".into())]
                }
            }
            _ => return None,
        })
    }
}

impl Wynn {
    /// Whether she saw or heard `other` come to grief in the last day.
    fn ailing(ctx: &Ctx<'_>, other: VillagerId) -> bool {
        let me = villager(ctx, "wynn");
        ctx.recall(me, ctx.tick.minus(TICKS_PER_DAY))
            .any(|(r, _)| matches!(&r.event, Emitted::Mishap { who, .. } if *who == other))
    }
}

impl Interlocutor for Wynn {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 2,
            skill: 6,
            rejection_sensitivity: 3,
            charity: 2,
            discretion: 8,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Wynn: thanks warm her most; she likes being asked. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank => 3,
            G::Ask | G::Sympathise => 1,
            G::Jab | G::Rebuff => -2,
            G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
