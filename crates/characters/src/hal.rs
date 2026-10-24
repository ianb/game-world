//! Hal, the carter. Twice a week he goes out along the road, buys flour and ale from the
//! world beyond the town, and sells them on to the bakery and the tavern. Supplies enter
//! and money leaves through him.
use crate::toolbox::{
    Plan, activity_of, buy_ale, go, go_tile, pay, stock, villager, wait_minutes, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::time::Weekday;

/// The east end of the street, where the road leaves the town.
pub const ROAD_OUT: (u16, u16) = (46, 11);
pub const FLOUR_PER_TRIP: i32 = 8;
pub const ALE_PER_TRIP: i32 = 6;
/// What the market charges him for a load, and what he charges in town.
pub const LOAD_COST: u32 = 10;
pub const FLOUR_PRICE: u32 = 8;
pub const ALE_DELIVERY_PRICE: u32 = 12;
pub const LUNCH_PRICE: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hal {
    pub trips: u16,
    pub carrying_flour: i32,
    pub carrying_ale: i32,
}

impl Hal {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("hal").expect("hal exists");
        let mut plan = Plan::baseline(me, ctx);
        if matches!(weekday(ctx), Weekday::Mon | Weekday::Thu) {
            plan.at(
                7,
                30,
                go_tile(ctx, "town", ROAD_OUT.0, ROAD_OUT.1, "to-market"),
            );
            plan.at(
                7,
                30,
                Command::Note("market day: out along the road".into()),
            );
        }
        plan.build()
    }

    pub fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) -> Option<Vec<Command>> {
        let me = ctx.villager_id("hal").expect("hal exists");
        Some(match trigger {
            // At the edge of town: buy a load and be gone a while.
            Trigger::Arrived if activity_of(ctx, "hal") == "to-market" => {
                // A full load, a half load when short, or home empty when broke.
                let coins = ctx.coins(me);
                let (cost, share, what) = if coins >= LOAD_COST {
                    (LOAD_COST, 1, "flour and ale at market")
                } else if coins >= LOAD_COST / 2 {
                    (LOAD_COST / 2, 2, "a half load at market")
                } else {
                    return Some(vec![
                        Command::Note("can't afford a load; back empty".into()),
                        go(ctx, "cart_hal", "tend-cart"),
                    ]);
                };
                self.carrying_flour = FLOUR_PER_TRIP / share;
                self.carrying_ale = ALE_PER_TRIP / share;
                self.trips += 1;
                vec![
                    pay(ctx, "hal", "outside", cost, what),
                    wait_minutes(90, "to-market"),
                ]
            }
            Trigger::Waited if activity_of(ctx, "hal") == "to-market" => {
                vec![go(ctx, "pantry_bakery", "deliver")]
            }
            Trigger::Arrived if activity_of(ctx, "hal") == "deliver" => {
                let flour = self.carrying_flour;
                self.carrying_flour = 0;
                let price = FLOUR_PRICE * u32::try_from(flour).unwrap_or(0)
                    / u32::try_from(FLOUR_PER_TRIP).unwrap_or(1);
                vec![
                    stock(ctx, "pantry_bakery", "flour", flour),
                    pay(ctx, "mara", "hal", price, "flour"),
                    go(ctx, "barrel_tavern", "deliver-ale"),
                ]
            }
            Trigger::Arrived if activity_of(ctx, "hal") == "deliver-ale" => {
                let ale = self.carrying_ale;
                self.carrying_ale = 0;
                let price = ALE_DELIVERY_PRICE * u32::try_from(ale).unwrap_or(0)
                    / u32::try_from(ALE_PER_TRIP).unwrap_or(1);
                vec![
                    stock(ctx, "barrel_tavern", "ale", ale),
                    pay(ctx, "tom", "hal", price, "ale"),
                    go(ctx, "cart_hal", "tend-cart"),
                ]
            }
            // Lunch at the tavern is paid for when Tom is behind the bar.
            Trigger::Arrived
                if activity_of(ctx, "hal") == "eat"
                    && ctx.villager(me).place == ctx.villager(villager(ctx, "tom")).place
                    && activity_of(ctx, "tom") == "tend-bar" =>
            {
                vec![pay(ctx, "hal", "tom", LUNCH_PRICE, "lunch")]
            }
            Trigger::Arrived if activity_of(ctx, "hal") == "drink" => {
                buy_ale(ctx, "hal").unwrap_or_else(|| vec![Command::Note("barrel's dry".into())])
            }
            Trigger::Passing { other } if activity_of(ctx, "hal") != "sleep" => {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}
