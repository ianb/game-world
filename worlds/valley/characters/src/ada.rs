//! Ada Hollins. Makes cheese, and sells it from a stall on the square on market days.
//! Description and design notes: `docs/characters/ada.md`.
use crate::Character;
use crate::toolbox::{activity_of, chat, go, hour, pay, stock, stock_of, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

const CHEESE_PRICE: u32 = 2;
const MAX_CHEESE: u16 = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ada {
    pub mind: Memory,
}

impl Character for Ada {
    const ID: &'static str = "ada";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Fond,
            crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // A morning in the dairy makes a cheese.
            Trigger::Arrived
                if activity_of(ctx, "ada") == "dairy"
                    && hour(ctx) < 12
                    && stock_of(ctx, "churn_hollins", "cheese") < MAX_CHEESE =>
            {
                vec![stock(ctx, "churn_hollins", "cheese", 1)]
            }
            // Market day: she sets out what she has.
            Trigger::Arrived if activity_of(ctx, "ada") == "market" => {
                let n = i32::from(stock_of(ctx, "churn_hollins", "cheese").min(4));
                vec![
                    stock(ctx, "churn_hollins", "cheese", -n),
                    stock(ctx, "stall_square", "cheese", n),
                ]
            }
            // Anyone who stops at the stall is offered a cheese.
            Trigger::Encounter { other }
                if activity_of(ctx, "ada") == "market"
                    && stock_of(ctx, "stall_square", "cheese") > 0 =>
            {
                let me = villager(ctx, "ada");
                vec![chat(ctx, me, *other, Frame::Transaction, "cheese", 10)?]
            }
            Trigger::TalkEnded { other, topic }
                if topic.is("cheese")
                    && stock_of(ctx, "stall_square", "cheese") > 0
                    && ctx.coins(*other) >= CHEESE_PRICE =>
            {
                vec![
                    stock(ctx, "stall_square", "cheese", -1),
                    pay(ctx, &ctx.villager(*other).id, "ada", CHEESE_PRICE, "cheese"),
                ]
            }
            // Market over at noon: what is left goes home.
            Trigger::Look { .. } if activity_of(ctx, "ada") == "market" && hour(ctx) >= 12 => {
                let n = i32::from(stock_of(ctx, "stall_square", "cheese"));
                vec![
                    stock(ctx, "stall_square", "cheese", -n),
                    stock(ctx, "churn_hollins", "cheese", n),
                    go(ctx, "table_hollins", "eat"),
                ]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Ada {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 2,
            skill: 5,
            rejection_sensitivity: 3,
            charity: 2,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Ada: thanks and a fair offer please her. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank => 2,
            G::Compliment | G::Offer => 1,
            G::Jab | G::Rebuff => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Complain
            | G::Request
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
