//! Hal, the carter. Twice a week he goes out along the road, buys flour and ale from the
//! world beyond the town, and sells them on to the bakery and the tavern. Supplies enter
//! and money leaves through him. He is touchy: quick to hear a refusal as a rebuff and to
//! blame the person for it, and when money is short it is on his mind. How touchy moves
//! with experience: talks that go badly make him more so, warm ones ease him. When
//! something goes wrong he blames whoever is nearest to hand.
//!
//! He is sweet on Mara. Some afternoons he calls at the bakery for a word, and
//! with her he pays compliments. He watches whom she seeks out: each time he sees her
//! start a talk with someone that is not business at a counter, he grows more jealous of
//! them, and less so after a warm talk with her. Jealous enough, he mutters about it aloud
//! when he has been drinking, he takes digs at the rival, and he confides in someone who is
//! neither of them.
use crate::tom::YARD;
use crate::toolbox::{
    Plan, activity_of, buy_ale, counter_talk, go, go_area, in_area, pay, pick, seen_seeking_out,
    stock, think, think_aloud, villager, wait_minutes, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Act, Blame, Frame, Mishap};
use sim::protocol::{Gesture, Topic};
use sim::recall::SIGHT;
use sim::social::{Episode, Interlocutor, Memory, Persona, Situation, act, default_next_act};
use sim::time::{TICKS_PER_DAY, Tick, Weekday};

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

/// Jealousy is 0 to `JEALOUSY_MAX`. Each time he sees Mara seek someone out it rises by
/// `JEALOUSY_PER_SIGHT`; it falls by one each day and by `JEALOUSY_EASED` after a warm talk
/// with her. At `JEALOUS` it shows.
pub const JEALOUSY_MAX: u8 = 10;
pub const JEALOUSY_PER_SIGHT: u8 = 3;
pub const JEALOUSY_EASED: u8 = 2;
pub const JEALOUS: u8 = 4;
/// Out of 100: the chance of an afternoon call at the bakery on a day off the road.
pub const CALL_CHANCE: u32 = 50;

/// Chance labels, for tests to force.
pub const CALLS: &str = "hal-calls";
pub const COMPLIMENTS: &str = "hal-compliments";
pub const JABS: &str = "hal-jabs";
pub const CONFIDES_JEALOUSY: &str = "hal-confides-jealousy";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hal {
    pub carrying_flour: i32,
    pub carrying_ale: i32,
    pub mind: Memory,
    /// How ready he is to hear rejection, moved by how his talks go.
    pub touchiness: u8,
    /// The rejection sensitivity he last noted, so a change is news.
    pub noted_sensitivity: u8,
    /// How jealous he is of `rival`; what he does about it depends on how much.
    pub jealousy: u8,
    /// The last person he saw Mara seek out.
    pub rival: Option<VillagerId>,
}

impl Default for Hal {
    fn default() -> Self {
        Hal {
            carrying_flour: 0,
            carrying_ale: 0,
            mind: Memory::default(),
            touchiness: TOUCHINESS_START,
            noted_sensitivity: TOUCHINESS_START / 10,
            jealousy: 0,
            rival: None,
        }
    }
}

impl Hal {
    /// Something sharp overheard: he assumes it was meant for him.
    fn stung(&mut self, ctx: &Ctx<'_>, speaker: VillagerId) -> Vec<Command> {
        self.touchiness = self.touchiness.saturating_add(4).min(TOUCHINESS_MAX);
        vec![think(
            ctx.topic("what-was-said").about(speaker),
            Blame::Them(speaker),
            false,
            false,
            -1,
        )]
    }

    /// Whether he saw Mara seek someone out since `since`. Each time feeds his jealousy of
    /// them, and he thinks about it: aloud when he is drinking and already jealous.
    fn watch_mara(&mut self, ctx: &Ctx<'_>, me: VillagerId, since: Tick) -> Option<Vec<Command>> {
        let mara = villager(ctx, "mara");
        let seen = seen_seeking_out(ctx, me, mara, since);
        let &(_, rival) = seen.last()?;
        for _ in &seen {
            self.jealousy = self
                .jealousy
                .saturating_add(JEALOUSY_PER_SIGHT)
                .min(JEALOUSY_MAX);
        }
        self.rival = Some(rival);
        let topic = ctx.topic("rival").about(rival).with(mara);
        let thought = if self.jealousy >= JEALOUS && activity_of(ctx, "hal") == "drink" {
            think_aloud(topic, Blame::Them(rival), false, false, -1)
        } else {
            think(topic, Blame::Them(rival), false, false, -1)
        };
        Some(vec![thought])
    }

    /// A warm talk with Mara reassures him.
    fn reassure(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) {
        if let Trigger::TalkEnded { other, .. } = trigger
            && *other == villager(ctx, "mara")
            && self
                .mind
                .episodes
                .last()
                .is_some_and(|e| e.with == *other && i16::from(e.peak) + i16::from(e.end) >= 2)
        {
            self.jealousy = self.jealousy.saturating_sub(JEALOUSY_EASED);
        }
    }

    /// At the edge of town: buy a load and be gone a while.
    fn at_market(&mut self, ctx: &Ctx<'_>, me: VillagerId, rng: &mut Pcg32) -> Vec<Command> {
        // A full load, a half load when short, or home empty when broke.
        let coins = ctx.coins(me);
        let (cost, share, what) = if coins >= LOAD_COST {
            (LOAD_COST, 1, "flour and ale at market")
        } else if coins >= LOAD_COST / 2 {
            (LOAD_COST / 2, 2, "a half load at market")
        } else {
            return vec![
                Command::Note("can't afford a load; back empty".into()),
                go(ctx, "cart_hal", "tend-cart"),
            ];
        };
        self.carrying_flour = FLOUR_PER_TRIP / share;
        self.carrying_ale = ALE_PER_TRIP / share;
        vec![
            pay(ctx, "hal", "outside", cost, what),
            // The market keeps him longer some days.
            wait_minutes(*pick(rng, &[90u32, 120, 150]), "to-market"),
        ]
    }

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
        match near {
            Some(o) => Blame::Them(o),
            None if in_area(ctx, villager(ctx, "hal"), YARD) => Blame::Them(villager(ctx, "tom")),
            None => Blame::World,
        }
    }

    pub fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("hal").expect("hal exists");
        let mut plan = Plan::baseline(me, ctx);
        self.jealousy = self.jealousy.saturating_sub(1);
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
        if matches!(weekday(ctx), Weekday::Mon | Weekday::Thu)
            && let Some(go) = go_area(ctx, "road-out", "to-market", rng)
        {
            plan.at(7, 30, go);
            plan.at(
                7,
                30,
                Command::Note("market day: out along the road".into()),
            );
        } else if ctx.chance(rng, me, CALLS, i16::try_from(CALL_CHANCE).unwrap_or(0)) {
            plan.at(15, 30, go(ctx, "counter_bakery", "call-at-bakery"));
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
        self.reassure(trigger, ctx);
        Some(match trigger {
            Trigger::Arrived if activity_of(ctx, "hal") == "to-market" => {
                self.at_market(ctx, me, rng)
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
            // A word with Mara if she is free. He buys nothing: when he bought a loaf each
            // time, Mara never ran short of money.
            Trigger::Arrived if activity_of(ctx, "hal") == "call-at-bakery" => {
                vec![
                    counter_talk(ctx, "hal", "mara", "road")
                        .unwrap_or_else(|| go(ctx, "cart_hal", "tend-cart")),
                ]
            }
            Trigger::TalkEnded { .. } if activity_of(ctx, "hal") == "call-at-bakery" => {
                vec![go(ctx, "cart_hal", "tend-cart")]
            }
            Trigger::Look { since } if activity_of(ctx, "hal") != "sleep" => {
                return self.watch_mara(ctx, me, *since);
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
            // Overhearing something sharp, or his own name: he assumes it was meant for him.
            Trigger::Overheard {
                speaker,
                gesture,
                about_me,
                ..
            } if *about_me || self.warmth_of(*gesture) < 0 => self.stung(ctx, *speaker),
            Trigger::Mishap {
                what: Mishap::Tripped,
            } => {
                let blame = Hal::someone_to_blame(ctx, me);
                vec![think(
                    ctx.topic("tripped").about(me),
                    blame,
                    false,
                    false,
                    -1,
                )]
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
            discretion: 5,
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

    /// With Mara he pays compliments. Jealous, he takes digs at the rival, and confides in
    /// someone else that he is sweet on her.
    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        let mara = villager(s.ctx, "mara");
        let jealous = self.jealousy >= JEALOUS;
        let sweet = s.ctx.topic("sweet-on").about(s.me).with(mara);
        let week = s.ctx.tick.minus(7 * TICKS_PER_DAY);
        if s.frame == Frame::Confiding && s.topic.is("sweet-on") && !s.i_did(Gesture::Disclose) {
            return act(Gesture::Disclose, s.topic);
        }
        if s.turn >= 1 && s.frame != Frame::Confiding {
            if s.other == mara
                && s.frame == Frame::SmallTalk
                && s.ctx.chance(rng, s.me, COMPLIMENTS, 35)
            {
                return act(Gesture::Compliment, s.topic);
            }
            if jealous && Some(s.other) == self.rival && s.ctx.chance(rng, s.me, JABS, 40) {
                return act(Gesture::Jab, s.topic);
            }
            if jealous
                && s.other != mara
                && Some(s.other) != self.rival
                && matches!(s.frame, Frame::Greeting | Frame::SmallTalk)
                && self.mind.regard(s.other).familiarity >= 1
                && !s.i_bid(Frame::Confiding)
                && !self.mind.told_since(s.other, &sweet, week)
                && s.ctx.chance(rng, s.me, CONFIDES_JEALOUSY, 50)
            {
                return Act::Bid {
                    frame: Frame::Confiding,
                    topic: sweet,
                };
            }
        }
        default_next_act(self, s, rng)
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

impl crate::Character for Hal {
    const ID: &'static str = "hal";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Indifferent,
            crate::Animal::Sheep => crate::Fondness::Wary,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        Hal::plan_day(self, ctx, rng)
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Hal::react(self, trigger, ctx, rng)
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
