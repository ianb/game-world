//! Greta, Bram's wife, keeps the hamlet store. Gives credit, keeps the slate, and asks for what is owed wherever she meets the debtor.
//! Description and design notes: `docs/characters/greta.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, chat, in_building, pay, seek, stock, stock_of, villager, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;
use std::collections::BTreeMap;

const CANDLE_PRICE: u32 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Greta {
    /// What each customer owes, in coins.
    pub owed: BTreeMap<VillagerId, u32>,
    pub mind: Memory,
}

impl Character for Greta {
    const ID: &'static str = "greta";
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "greta");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Mon {
            plan.at(
                7,
                45,
                pay(ctx, "greta", "outside", 8, "stock for the store"),
            );
            plan.at(7, 45, stock(ctx, "shelf_store", "candles", 4));
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
            // Someone in the shop while she keeps it: a sale.
            Trigger::Encounter { other }
                if activity_of(ctx, "greta") == "keep-shop"
                    && in_building(ctx, *other, "store") =>
            {
                let me = villager(ctx, "greta");
                vec![chat(ctx, me, *other, Frame::Transaction, "candles", 8)?]
            }
            Trigger::TalkEnded { other, topic } if topic.is("candles") => {
                let who = ctx.villager(*other).id.clone();
                if stock_of(ctx, "shelf_store", "candles") == 0 {
                    vec![Command::Note("no candles left".into())]
                } else if ctx.coins(*other) >= CANDLE_PRICE {
                    vec![
                        stock(ctx, "shelf_store", "candles", -1),
                        pay(ctx, &who, "greta", CANDLE_PRICE, "candles"),
                    ]
                } else {
                    *self.owed.entry(*other).or_default() += CANDLE_PRICE;
                    vec![
                        stock(ctx, "shelf_store", "candles", -1),
                        Command::Note(format!("candles for {who} on the slate")),
                    ]
                }
            }
            // Meeting a debtor anywhere: she asks for it, and takes it if they have it.
            Trigger::Encounter { other } if self.owed.get(other).is_some_and(|n| *n > 0) => {
                let me = villager(ctx, "greta");
                let owed = self.owed.get(other).copied().unwrap_or(0);
                if ctx.coins(*other) >= owed {
                    self.owed.remove(other);
                    vec![pay(
                        ctx,
                        &ctx.villager(*other).id,
                        "greta",
                        owed,
                        "what was owed",
                    )]
                } else {
                    let topic = ctx.topic("what-is-owed").about(*other);
                    vec![seek(ctx, me, *other, Frame::SmallTalk, topic, 8)?]
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Greta {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 1,
            skill: 6,
            rejection_sensitivity: 3,
            charity: 0,
            discretion: 4,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Greta: thanks and fair dealing; being put off is not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank => 2,
            G::Agree => 1,
            G::Decline => -1,
            G::Rebuff | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
