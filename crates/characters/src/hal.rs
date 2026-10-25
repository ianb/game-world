//! Hal, the carter. Twice a week he goes out along the road, buys flour and ale from the
//! world beyond the town, and sells them on to the bakery and the tavern. Supplies enter
//! and money leaves through him. He is touchy: quick to hear a refusal as a rebuff and to
//! blame the person for it, and when money is short it is on his mind. How touchy moves
//! with experience: talks that go badly make him more so, warm ones ease him. When
//! something goes wrong he blames whoever is nearest to hand.
use crate::tom::YARD;
use crate::toolbox::{
    Plan, activity_of, buy_ale, counter_talk, go, go_tile, pay, pick, stock, think, villager,
    wait_minutes, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Mishap};
use sim::protocol::{Gesture, Topic};
use sim::recall::SIGHT;
use sim::social::{Episode, Interlocutor, Memory, Persona};
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

/// Touchiness is 0 to 100; his rejection sensitivity is a tenth of it.
pub const TOUCHINESS_START: u8 = 80;
pub const TOUCHINESS_MIN: u8 = 20;
pub const TOUCHINESS_MAX: u8 = 100;
/// Per point of how badly a talk went (peak plus end below zero).
pub const TOUCHIER_PER_POINT: i16 = 8;
/// After a talk that went well (peak plus end of 2 or more).
pub const EASIER: i16 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hal {
    pub trips: u16,
    pub carrying_flour: i32,
    pub carrying_ale: i32,
    pub mind: Memory,
    /// How ready he is to hear rejection, moved by how his talks go.
    pub touchiness: u8,
    /// The rejection sensitivity he last noted, so a change is news.
    pub noted_sensitivity: u8,
}

impl Default for Hal {
    fn default() -> Self {
        Hal {
            trips: 0,
            carrying_flour: 0,
            carrying_ale: 0,
            mind: Memory::default(),
            touchiness: TOUCHINESS_START,
            noted_sensitivity: TOUCHINESS_START / 10,
        }
    }
}

impl Hal {
    /// Anyone close enough to have seen; failing that, Tom if it was his yard; else the
    /// world.
    fn someone_to_blame(ctx: &Ctx<'_>, me: VillagerId) -> Blame {
        let here = ctx.villager(me);
        let near = (0..ctx.villagers.len())
            .map(|i| VillagerId(u16::try_from(i).expect("few villagers")))
            .filter(|&o| o != me)
            .find(|&o| {
                let v = ctx.villager(o);
                v.place == here.place
                    && v.activity != "sleep"
                    && v.pos
                        .x
                        .abs_diff(here.pos.x)
                        .max(v.pos.y.abs_diff(here.pos.y))
                        <= SIGHT
            });
        let town = ctx.place_id("town");
        match near {
            Some(o) => Blame::Them(o),
            None if Some(here.place) == town && YARD.contains(here.pos.x, here.pos.y) => {
                Blame::Them(villager(ctx, "tom"))
            }
            None => Blame::World,
        }
    }

    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("hal").expect("hal exists");
        let mut plan = Plan::baseline(me, ctx);
        let now = self.touchiness / 10;
        if now != self.noted_sensitivity {
            let how = if now > self.noted_sensitivity {
                "touchier"
            } else {
                "easier"
            };
            plan.at(
                0,
                1,
                Command::Note(format!("{how} lately (rejection sensitivity {now})")),
            );
            self.noted_sensitivity = now;
        }
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

    pub fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
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
                    // The market keeps him longer some days.
                    wait_minutes(*pick(rng, &[90u32, 120, 150]), "to-market"),
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
                    counter_talk(ctx, "hal", "mara", "flour")
                        .unwrap_or_else(|| go(ctx, "barrel_tavern", "deliver-ale")),
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
                    counter_talk(ctx, "hal", "tom", "ale")
                        .unwrap_or_else(|| go(ctx, "cart_hal", "tend-cart")),
                ]
            }
            // Deliveries go on once the talk at the counter is done.
            Trigger::TalkEnded { topic, .. }
                if topic.is("flour") && activity_of(ctx, "hal") == "deliver" =>
            {
                vec![go(ctx, "barrel_tavern", "deliver-ale")]
            }
            Trigger::TalkEnded { topic, .. }
                if topic.is("ale") && activity_of(ctx, "hal") == "deliver-ale" =>
            {
                vec![go(ctx, "cart_hal", "tend-cart")]
            }
            // Lunch at the tavern is paid for when Tom is behind the bar.
            Trigger::Arrived
                if activity_of(ctx, "hal") == "eat"
                    && ctx.villager(me).place == ctx.villager(villager(ctx, "tom")).place
                    && activity_of(ctx, "tom") == "tend-bar" =>
            {
                let mut cmds = vec![pay(ctx, "hal", "tom", LUNCH_PRICE, "lunch")];
                cmds.extend(counter_talk(ctx, "hal", "tom", "lunch"));
                cmds
            }
            Trigger::Arrived if activity_of(ctx, "hal") == "drink" => {
                let mut cmds = buy_ale(ctx, "hal")
                    .unwrap_or_else(|| vec![Command::Note("barrel's dry".into())]);
                cmds.extend(counter_talk(ctx, "hal", "tom", "ale"));
                cmds
            }
            Trigger::Mishap {
                what: Mishap::Tripped,
            } => {
                let blame = Hal::someone_to_blame(ctx, me);
                vec![think("tripped on weeds", blame, false, false, -1)]
            }
            Trigger::Passing { other } if activity_of(ctx, "hal") != "sleep" => {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Hal {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 0,
            skill: 5,
            rejection_sensitivity: self.touchiness / 10,
            charity: -2,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Hal: a refusal or a question about his business is never
    /// neutral, and an offer means money. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Decline | G::Pry => -2,
            G::Offer => 2,
            G::Joke => 0,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }

    /// Money is on his mind whenever he could not pay for two full loads.
    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<Topic> {
        (ctx.coins(me) < 2 * LOAD_COST).then(|| ctx.topic("money").about(me))
    }

    fn remember(&mut self, e: Episode) {
        let felt = i16::from(e.peak) + i16::from(e.end);
        let delta = if felt < 0 {
            -felt * TOUCHIER_PER_POINT
        } else if felt >= 2 {
            -EASIER
        } else {
            0
        };
        let t = (i16::from(self.touchiness) + delta)
            .clamp(i16::from(TOUCHINESS_MIN), i16::from(TOUCHINESS_MAX));
        self.touchiness = u8::try_from(t).unwrap_or(TOUCHINESS_START);
        self.mind.remember(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::ids::VillagerId;
    use sim::protocol::Frame;
    use sim::time::Tick;

    fn talk(peak: i8, end: i8) -> Episode {
        Episode {
            tick: Tick(0),
            with: VillagerId(0),
            frame: Frame::SmallTalk,
            peak,
            end,
            they_seemed: 0,
            notes: Vec::new(),
        }
    }

    #[test]
    fn bad_talks_make_him_touchier_and_good_ones_ease_him() {
        let mut hal = Hal::default();
        assert_eq!(hal.persona().rejection_sensitivity, 8);
        hal.remember(talk(-1, -1));
        assert_eq!(hal.touchiness, 96);
        hal.remember(talk(-3, -3));
        assert_eq!(hal.touchiness, TOUCHINESS_MAX);
        assert_eq!(hal.persona().rejection_sensitivity, 10);
        // A neutral talk changes nothing; each good one eases him a little.
        hal.remember(talk(1, 0));
        assert_eq!(hal.touchiness, TOUCHINESS_MAX);
        for _ in 0..10 {
            hal.remember(talk(1, 1));
        }
        assert_eq!(hal.touchiness, 80);
        for _ in 0..100 {
            hal.remember(talk(2, 2));
        }
        assert_eq!(hal.touchiness, TOUCHINESS_MIN);
    }
}
