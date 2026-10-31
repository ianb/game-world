//! Mara, the baker. Her content schedule, plus what is hers: she counts what she bakes,
//! takes Tom a pastry when he seemed low the last time they talked, and at the counter she
//! is friendly and discreet. She will hear a confidence from anyone she knows, but never
//! with someone else in the shop. Minding the shop, she keeps an eye on the shelf: if it is
//! bare and there is flour, she bakes again.
//!
//! She is fond of Tom. Evenings at the tavern she goes back to the bar for another word
//! with him when they have not talked for an hour, and with him she pays compliments. Once
//! she knows Hal is sweet on her (told so, or hearing him mutter about a rival), she keeps
//! him at a distance: no chat at the counter, no small talk.
use crate::toolbox::{
    Plan, activity_of, buy_ale, chat, counter_talk, errand, go, object, stock, stock_of, think,
    villager,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Act, BidAnswer, Blame, Frame, Gesture, Topic};
use sim::social::{
    Interlocutor, Memory, Persona, Situation, act, default_answer_bid, default_next_act,
};
use sim::time::{TICKS_PER_DAY, TICKS_PER_MINUTE, Tick};

pub const FLOUR_PER_BATCH: u16 = 2;
pub const LOAVES_PER_BATCH: i32 = 6;
/// With this many loaves left from yesterday she skips a bake (stale bread goes out
/// first, so this only matters when the first bake of the day already happened).
pub const LOAVES_ENOUGH: u16 = 4;
/// Minutes since her last talk with Tom before she goes back to the bar for another word.
pub const WORD_WITH_TOM_GAP: u32 = 60;

/// Chance labels, for tests to force.
pub const COMPLIMENTS_TOM: &str = "mara-compliments-tom";
pub const KEEPS_CHATTING: &str = "mara-keeps-chatting";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mara {
    /// When she last heard Tom tell about a fish, so a new story is news.
    pub fish_story_at: Option<Tick>,
    /// The day she last noticed the shelf bare, so she reacts once.
    pub bare_shelf_day: Option<u32>,
    /// She knows Hal is sweet on her, and keeps him at a distance.
    pub knows_hal_is_sweet: bool,
    pub mind: Memory,
}

impl Mara {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("mara").expect("mara exists");
        let mut plan = Plan::baseline(me, ctx);
        // Yesterday's bread goes out before the first bake.
        let stale = i32::from(stock_of(ctx, "shelf_bakery", "bread"));
        if stale > 0 {
            plan.at(6, 35, stock(ctx, "shelf_bakery", "bread", -stale));
        }
        // She knows Tom only from talking to him and from hearing him grumble.
        let tom_id = villager(ctx, "tom");
        let tom = self.mind.regard(tom_id);
        let yesterday = ctx.tick.day_start().minus(TICKS_PER_DAY);
        let grumbling = self
            .mind
            .heard()
            .from(tom_id)
            .to(tom_id)
            .gesture(Gesture::Complain)
            .since(yesterday)
            .any();
        let why = if grumbling {
            Some("heard Tom grumbling".to_string())
        } else if tom.familiarity > 0 && tom.last_seemed < 0 {
            Some(format!("Tom seemed low last time ({})", tom.last_seemed))
        } else {
            None
        };
        if let Some(why) = why {
            plan.at(17, 30, go(ctx, "bar_tavern", "bring-pastry"));
            plan.at(0, 1, Command::Note(format!("{why}; pastry at 5:30pm")));
        }
        plan.build()
    }

    /// After a talk or something overheard: whether she has now heard, in words, that Hal is
    /// sweet on her, or Hal muttering about a rival for her.
    fn notice_hal(&mut self, ctx: &Ctx<'_>, me: VillagerId) -> Option<Command> {
        let hal = villager(ctx, "hal");
        let told = self.mind.heard().words().iter().any(|h| {
            h.topic.as_ref().is_some_and(|t| {
                t.with == Some(me)
                    && ((t.is("sweet-on") && t.about == Some(hal))
                        || (t.is("rival") && h.speaker == hal))
            })
        });
        if self.knows_hal_is_sweet || !told {
            return None;
        }
        self.knows_hal_is_sweet = true;
        Some(think(
            ctx.topic("sweet-on").about(hal).with(me),
            Blame::Luck,
            true,
            false,
            -1,
        ))
    }

    pub fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) -> Option<Vec<Command>> {
        let me = ctx.villager_id("mara").expect("mara exists");
        if matches!(
            trigger,
            Trigger::TalkEnded { .. } | Trigger::Overheard { .. }
        ) && let Some(thought) = self.notice_hal(ctx, me)
        {
            let mut cmds = vec![thought];
            cmds.extend(self.react_to(trigger, ctx, me).unwrap_or_default());
            return Some(cmds);
        }
        self.react_to(trigger, ctx, me)
    }

    fn react_to(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        me: VillagerId,
    ) -> Option<Vec<Command>> {
        let tom = villager(ctx, "tom");
        Some(match trigger {
            Trigger::Arrived => match ctx.villager(me).activity.as_str() {
                // Two measures of flour make six loaves. No flour, no bread.
                "bake" => {
                    if stock_of(ctx, "shelf_bakery", "bread") >= LOAVES_ENOUGH {
                        vec![Command::Note("shelf still full; no bake today".into())]
                    } else if stock_of(ctx, "pantry_bakery", "flour") >= FLOUR_PER_BATCH {
                        vec![
                            stock(ctx, "pantry_bakery", "flour", -i32::from(FLOUR_PER_BATCH)),
                            stock(ctx, "shelf_bakery", "bread", LOAVES_PER_BATCH),
                        ]
                    } else {
                        vec![Command::Note("no flour; nothing to bake".into())]
                    }
                }
                "bring-pastry" => match counter_talk(ctx, "mara", "tom", "pastry") {
                    Some(talk) => vec![talk],
                    None => vec![Command::Note("Tom isn't free; left the pastry".into())],
                },
                "drink" => {
                    let mut cmds = buy_ale(ctx, "mara")
                        .unwrap_or_else(|| vec![Command::Note("no ale".into())]);
                    cmds.extend(counter_talk(ctx, "mara", "tom", "ale"));
                    cmds
                }
                _ => return None,
            },
            // Behind the counter, the shelf is in front of her.
            Trigger::Look { .. }
                if activity_of(ctx, "mara") == "tend-shop"
                    && stock_of(ctx, "shelf_bakery", "bread") == 0
                    && self.bare_shelf_day != Some(ctx.calendar.day_index) =>
            {
                self.bare_shelf_day = Some(ctx.calendar.day_index);
                let mut cmds = vec![think(
                    ctx.topic("bare-shelf").about(me),
                    Blame::Luck,
                    false,
                    false,
                    -1,
                )];
                if stock_of(ctx, "pantry_bakery", "flour") >= FLOUR_PER_BATCH
                    && ctx.calendar.time.hour < 16
                {
                    cmds.push(Command::Note("baking again".into()));
                    cmds.push(errand(object(ctx, "oven_bakery"), "bake", 30));
                }
                cmds
            }
            // At the tavern, back to the bar for another word with Tom.
            Trigger::Look { .. }
                if activity_of(ctx, "mara") == "drink"
                    && activity_of(ctx, "tom") == "tend-bar"
                    && self
                        .mind
                        .episodes
                        .iter()
                        .rev()
                        .find(|e| e.with == tom)
                        .is_none_or(|e| {
                            e.tick.plus(WORD_WITH_TOM_GAP * TICKS_PER_MINUTE) <= ctx.tick
                        }) =>
            {
                vec![chat(ctx, me, tom, Frame::SmallTalk, "day", 10)?]
            }
            Trigger::TalkEnded { other, .. } if *other == tom => {
                let news = self
                    .mind
                    .heard()
                    .from(tom)
                    .topic("fish")
                    .last()
                    .map(|h| h.tick)
                    .filter(|&t| self.fish_story_at.is_none_or(|seen| t > seen));
                self.fish_story_at = Some(news?);
                vec![Command::Note("heard about Tom's fish".into())]
            }
            // A word for anyone she passes, unless she is in the middle of baking.
            Trigger::Passing { other }
                if !matches!(ctx.villager(me).activity.as_str(), "bake" | "sleep") =>
            {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}

impl Mara {
    /// A bid for small talk; with the carter it is about the road.
    fn small_talk(s: &Situation<'_>) -> Act {
        let topic = if s.other == villager(s.ctx, "hal") {
            "road"
        } else {
            "day"
        };
        Act::Bid {
            frame: Frame::SmallTalk,
            topic: s.ctx.topic(topic),
        }
    }
}

impl Interlocutor for Mara {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 2,
            skill: 7,
            rejection_sensitivity: 2,
            charity: 1,
            discretion: 9,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Mara: being confided in matters to her, and a dig stings
    /// less than it might. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Disclose => 2,
            G::Jab => -1,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }

    /// Running out of flour, or of money to pay the carter for more.
    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<Topic> {
        if stock_of(ctx, "pantry_bakery", "flour") < FLOUR_PER_BATCH {
            Some(ctx.topic("flour").about(me))
        } else if ctx.coins(me) < crate::hal::FLOUR_PRICE {
            Some(ctx.topic("money").about(me))
        } else {
            None
        }
    }

    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        let ctx = s.ctx;
        let hal = villager(ctx, "hal");
        if s.other == hal && self.knows_hal_is_sweet && s.turn >= 1 {
            return match (s.frame, s.turn) {
                (Frame::Transaction, 1) => act(Gesture::Thank, s.topic),
                _ => act(Gesture::Leave, s.topic),
            };
        }
        if s.other == villager(ctx, "tom")
            && s.frame == Frame::SmallTalk
            && s.turn >= 1
            && s.ctx.chance(rng, s.me, COMPLIMENTS_TOM, 30)
        {
            return act(Gesture::Compliment, s.topic);
        }
        match (s.frame, s.topic.key.as_str(), s.initiator, s.turn) {
            (Frame::Transaction, "pastry", true, 0) => act(Gesture::Offer, s.topic),
            // Pip's bun: free when there is bread on the shelf and she is fond of him.
            (Frame::Transaction, "bun", false, 0) => {
                if stock_of(ctx, "shelf_bakery", "bread") > 0
                    && self.mind.regard(s.other).feeling >= 0
                {
                    act(Gesture::Offer, s.topic)
                } else {
                    act(Gesture::Decline, s.topic)
                }
            }
            // Behind the counter, she keeps people she knows for a chat rather than thanking
            // them off.
            (Frame::Transaction, _, false, 1)
                if self.mind.regard(s.other).familiarity >= 1
                    && s.ctx.chance(
                        rng,
                        s.me,
                        KEEPS_CHATTING,
                        30 + 15 * i16::from(self.mind.regard(s.other).feeling.max(0)),
                    ) =>
            {
                Mara::small_talk(s)
            }
            _ => match default_next_act(self, s, rng) {
                Act::Bid {
                    frame: Frame::SmallTalk,
                    ..
                } => Mara::small_talk(s),
                act => act,
            },
        }
    }

    fn answer_bid(&mut self, s: &Situation<'_>, frame: Frame, rng: &mut Pcg32) -> BidAnswer {
        let r = self.mind.regard(s.other);
        if s.other == villager(s.ctx, "hal") && self.knows_hal_is_sweet {
            return BidAnswer::Away { account: true };
        }
        if frame == Frame::Confiding && s.private() && r.familiarity >= 2 && r.feeling > -2 {
            BidAnswer::Toward
        } else {
            default_answer_bid(self, s, frame, rng)
        }
    }
}

impl crate::Character for Mara {
    const ID: &'static str = "mara";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Fond,
            crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        Mara::plan_day(self, ctx, rng)
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Mara::react(self, trigger, ctx)
    }
}
