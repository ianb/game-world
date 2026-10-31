//! Ivo, the hermit. Moves deeper into the wood when anyone comes near, and buys his bread at dawn before anyone is about.
//! Description and design notes: `docs/characters/ivo.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, buy_bread, go, go_area, villager, weekday, work_area};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Act, BidAnswer, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona, Situation};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ivo {
    pub mind: Memory,
}

impl Character for Ivo {
    const ID: &'static str = "ivo";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Afraid,
            content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "ivo");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Sat {
            plan.at(5, 15, go(ctx, "shelf_bakery", "buy-bread"));
        }
        plan.build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "ivo") == "forage" => {
                work_area(ctx, "ivo", trigger, "hermit-wood", "forage", 40, 1, rng)
            }
            // Someone near: he goes elsewhere in the wood.
            Trigger::Encounter { .. }
                if !matches!(activity_of(ctx, "ivo"), "sleep" | "buy-bread") =>
            {
                vec![
                    go_area(ctx, "hermit-wood", "forage", rng)
                        .unwrap_or_else(|| go(ctx, "table_ivo", "rest")),
                ]
            }
            Trigger::Arrived if activity_of(ctx, "ivo") == "buy-bread" => {
                let mut cmds = buy_bread(ctx, "ivo")
                    .unwrap_or_else(|| vec![Command::Note("no bread yet".into())]);
                cmds.push(go(ctx, "table_ivo", "eat"));
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Ivo {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -3,
            warmth: -1,
            skill: 2,
            rejection_sensitivity: 7,
            charity: 0,
            discretion: 10,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Ivo: being let go is the only kindness; questions and invitations are intrusions. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Leave => 2,
            G::Decline => 0,
            G::Invite | G::Joke => -1,
            G::Ask | G::Tease => -2,
            G::Pry => -3,
            G::Tell
            | G::Agree
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Jab
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }

    /// He takes his leave at the first chance.
    fn next_act(&mut self, s: &Situation<'_>, _rng: &mut Pcg32) -> Act {
        Act::Gesture {
            gesture: Gesture::Leave,
            topic: s.topic.clone(),
        }
    }

    fn answer_bid(&mut self, _s: &Situation<'_>, _frame: Frame, _rng: &mut Pcg32) -> BidAnswer {
        BidAnswer::Away { account: false }
    }
}
