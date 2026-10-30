//! Rook, the charcoal burner. Every third night the clamp is lit and he watches it till morning, sleeping by day.
//! Description and design notes: `docs/characters/rook.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, day_number, go, pay, stock, stock_of, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;

const MAX_CHARCOAL: u16 = 9;
const CHARCOAL_PRICE: u32 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rook {
    pub mind: Memory,
}

impl Character for Rook {
    const ID: &'static str = "rook";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => {
                crate::Fondness::Indifferent
            }
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "rook");
        let mut plan = Plan::baseline(me, ctx);
        if day_number(ctx).is_multiple_of(3) {
            // A burn: the clamp is lit at dusk and watched through the night.
            plan.drop_activity("burn")
                .drop_activity("sleep")
                .drop_activity("eat");
            plan.at(8, 0, go(ctx, "table_rook", "eat"));
            plan.at(9, 0, go(ctx, "bed_rook", "sleep"));
            plan.at(17, 30, go(ctx, "table_rook", "eat"));
            plan.at(18, 30, go(ctx, "clamp_rook", "watch-clamp"));
        } else if weekday(ctx) == Weekday::Mon {
            plan.at(10, 0, go(ctx, "forge_smithy", "deliver-charcoal"));
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
            Trigger::Arrived
                if activity_of(ctx, "rook") == "burn"
                    && stock_of(ctx, "clamp_rook", "charcoal") < MAX_CHARCOAL =>
            {
                vec![stock(ctx, "clamp_rook", "charcoal", 1)]
            }
            Trigger::Arrived if activity_of(ctx, "rook") == "watch-clamp" => {
                let room = MAX_CHARCOAL.saturating_sub(stock_of(ctx, "clamp_rook", "charcoal"));
                vec![stock(ctx, "clamp_rook", "charcoal", i32::from(room.min(4)))]
            }
            // Monday: charcoal for the forge.
            Trigger::Arrived if activity_of(ctx, "rook") == "deliver-charcoal" => {
                let n = stock_of(ctx, "clamp_rook", "charcoal").min(3);
                let mut cmds = Vec::new();
                if n > 0 {
                    cmds.push(stock(ctx, "clamp_rook", "charcoal", -i32::from(n)));
                    cmds.push(stock(ctx, "forge_smithy", "charcoal", i32::from(n)));
                    cmds.push(pay(
                        ctx,
                        "bram",
                        "rook",
                        u32::from(n) * CHARCOAL_PRICE,
                        "charcoal",
                    ));
                }
                cmds.push(go(ctx, "clamp_rook", "burn"));
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Rook {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: -1,
            skill: 3,
            rejection_sensitivity: 5,
            charity: 0,
            discretion: 7,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Rook: people rarely thank him, so it counts; jokes about soot do not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank => 2,
            G::Sympathise => 1,
            G::Leave => -1,
            G::Tease | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }
}
