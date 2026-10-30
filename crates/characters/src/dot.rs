//! Dot, who keeps the alehouse. Counts the keg against the takings, and knows when Wat has been at it.
//! Description and design notes: `docs/characters/dot.md`.
use crate::Character;
use crate::toolbox::{ALE_PRICE, activity_of, pay, seek, stock, stock_of, think, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

const RESTOCK_BELOW: u16 = 6;
const RESTOCK: i32 = 8;
const RESTOCK_COST: u32 = 12;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dot {
    /// Ale in the keg and her coins when she opened, to square against at closing.
    pub opened_with: Option<(u16, u32)>,
    /// Whether she found ale missing last night, to have it out with Wat.
    pub short: bool,
    pub mind: Memory,
}

impl Character for Dot {
    const ID: &'static str = "dot";
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // Morning at the keg: order more if it is low, and have it out with Wat if the
            // keg came up short.
            Trigger::Arrived if activity_of(ctx, "dot") == "tap" => {
                let me = villager(ctx, "dot");
                let mut cmds = Vec::new();
                if stock_of(ctx, "bar_alehouse", "ale") < RESTOCK_BELOW {
                    cmds.push(pay(ctx, "dot", "outside", RESTOCK_COST, "ale"));
                    cmds.push(stock(ctx, "bar_alehouse", "ale", RESTOCK));
                }
                if std::mem::take(&mut self.short) {
                    let wat = villager(ctx, "wat");
                    let topic = ctx.topic("the-drink").about(wat);
                    cmds.extend(seek(ctx, me, wat, Frame::SmallTalk, topic, 10));
                }
                cmds
            }
            Trigger::Arrived if activity_of(ctx, "dot") == "keep-bar" => {
                let me = villager(ctx, "dot");
                self.opened_with = Some((stock_of(ctx, "bar_alehouse", "ale"), ctx.coins(me)));
                Vec::new()
            }
            // Closing: ale gone that was not paid for.
            Trigger::Arrived if activity_of(ctx, "dot") == "sleep" => {
                let me = villager(ctx, "dot");
                let (ale, coins) = self.opened_with.take()?;
                let sold = ctx.coins(me).saturating_sub(coins) / ALE_PRICE;
                let gone = u32::from(ale.saturating_sub(stock_of(ctx, "bar_alehouse", "ale")));
                if gone > sold {
                    self.short = true;
                    let wat = villager(ctx, "wat");
                    vec![think(
                        ctx.topic("the-drink").about(wat),
                        Blame::Them(wat),
                        true,
                        false,
                        -2,
                    )]
                } else {
                    return None;
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Dot {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
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

    /// What each gesture means to Dot: custom and thanks; complaint about her ale is not welcome. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank => 2,
            G::Compliment | G::Joke => 1,
            G::Complain => -1,
            G::Rebuff | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Tease
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
