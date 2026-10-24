//! Mara, the baker. Her content schedule, plus what is hers: she counts what she bakes,
//! takes Tom a pastry when he seemed low the last time they talked, and at the counter she
//! is friendly and discreet. She will hear a confidence from anyone she knows, but never
//! with someone else in the shop.
use crate::toolbox::{Plan, buy_ale, counter_talk, go, stock, stock_of, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Act, BidAnswer, Frame, MoveKind};
use sim::social::{
    Interlocutor, Memory, Persona, Situation, default_answer_bid, default_next_act, mv, roll,
};
use sim::time::Tick;

pub const FLOUR_PER_BATCH: u16 = 2;
pub const LOAVES_PER_BATCH: i32 = 6;
/// With this many loaves left from yesterday she skips a bake (stale bread goes out
/// first, so this only matters when the first bake of the day already happened).
pub const LOAVES_ENOUGH: u16 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mara {
    /// Baking sessions completed. Nobody else tracks this.
    pub batches_baked: u16,
    pub pastries_delivered: u16,
    /// When she last heard Tom tell about a fish, so a new story is news.
    pub fish_story_at: Option<Tick>,
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
        // She knows Tom only from talking to him.
        let tom = self.mind.regard(villager(ctx, "tom"));
        if tom.familiarity > 0 && tom.last_seemed < 0 {
            plan.at(17, 30, go(ctx, "bar_tavern", "bring-pastry"));
            plan.at(
                0,
                1,
                Command::Note(format!(
                    "Tom seemed low last time ({}); pastry at 5:30pm",
                    tom.last_seemed
                )),
            );
        }
        plan.build()
    }

    pub fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) -> Option<Vec<Command>> {
        let me = ctx.villager_id("mara").expect("mara exists");
        let tom = villager(ctx, "tom");
        Some(match trigger {
            Trigger::Arrived => match ctx.villager(me).activity.as_str() {
                // Two measures of flour make six loaves. No flour, no bread.
                "bake" => {
                    if stock_of(ctx, "shelf_bakery", "bread") >= LOAVES_ENOUGH {
                        vec![Command::Note("shelf still full; no bake today".into())]
                    } else if stock_of(ctx, "pantry_bakery", "flour") >= FLOUR_PER_BATCH {
                        self.batches_baked += 1;
                        vec![
                            stock(ctx, "pantry_bakery", "flour", -i32::from(FLOUR_PER_BATCH)),
                            stock(ctx, "shelf_bakery", "bread", LOAVES_PER_BATCH),
                        ]
                    } else {
                        vec![Command::Note("no flour; nothing to bake".into())]
                    }
                }
                "bring-pastry" => {
                    self.pastries_delivered += 1;
                    match counter_talk(ctx, "mara", "tom", "a pastry") {
                        Some(talk) => vec![talk],
                        None => vec![Command::Note("Tom isn't free; left the pastry".into())],
                    }
                }
                "drink" => {
                    let mut cmds = buy_ale(ctx, "mara")
                        .unwrap_or_else(|| vec![Command::Note("no ale".into())]);
                    cmds.extend(counter_talk(ctx, "mara", "tom", "ale"));
                    cmds
                }
                _ => return None,
            },
            Trigger::TalkEnded { other, .. } if *other == tom => {
                let news = self
                    .mind
                    .heard
                    .iter()
                    .rev()
                    .find(|h| h.from == tom && h.subject == "the fish")
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
        let subject = if s.other == villager(s.ctx, "hal") {
            "the road"
        } else {
            "the day"
        };
        Act::Bid {
            frame: Frame::SmallTalk,
            subject: subject.into(),
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
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// Running out of flour, or of money to pay the carter for more.
    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<String> {
        if stock_of(ctx, "pantry_bakery", "flour") < FLOUR_PER_BATCH {
            Some("the flour".into())
        } else if ctx.coins(me) < crate::hal::FLOUR_PRICE {
            Some("money".into())
        } else {
            None
        }
    }

    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        let ctx = s.ctx;
        match (s.frame, s.topic, s.initiator, s.turn) {
            (Frame::Transaction, "a pastry", true, 0) => mv(MoveKind::Offer, "a pastry"),
            // Pip's bun: free when there is bread on the shelf and she is fond of him.
            (Frame::Transaction, "a bun", false, 0) => {
                if stock_of(ctx, "shelf_bakery", "bread") > 0
                    && self.mind.regard(s.other).feeling >= 0
                {
                    mv(MoveKind::Offer, "a bun")
                } else {
                    mv(MoveKind::Decline, "a bun")
                }
            }
            // Behind the counter, she keeps people she knows for a chat rather than thanking
            // them off.
            (Frame::Transaction, _, false, 1)
                if self.mind.regard(s.other).familiarity >= 1
                    && roll(rng)
                        < 30 + 15 * i16::from(self.mind.regard(s.other).feeling.max(0)) =>
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
        if frame == Frame::Confiding && s.private() && r.familiarity >= 2 && r.feeling > -2 {
            BidAnswer::Toward
        } else {
            default_answer_bid(self, s, frame, rng)
        }
    }
}
