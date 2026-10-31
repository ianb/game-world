//! Elias, the priest. On weekday afternoons he calls on whoever he knows least.
//! Description and design notes: `docs/characters/elias.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, go, heard_lately, is_weekend, near, seek, villager, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Elias {
    pub mind: Memory,
}

impl Character for Elias {
    const ID: &'static str = "elias";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken | crate::Animal::Sheep => crate::Fondness::Fond,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "elias");
        let mut plan = Plan::baseline(me, ctx);
        if !is_weekend(ctx) {
            plan.at(15, 0, go(ctx, "chair_vicarage", "rounds"));
        }
        if weekday(ctx) == Weekday::Thu && heard_lately(ctx, &self.mind, "celia", "tea", 4) {
            plan.at(16, 0, go(ctx, "chair_parlour_2", "tea"));
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
            // An afternoon round: of those resting or at home within reach of a walk, whoever
            // he knows least.
            Trigger::Arrived if activity_of(ctx, "elias") == "rounds" => {
                let me = villager(ctx, "elias");
                let who = (0..ctx.villagers.len())
                    .map(|i| VillagerId(u16::try_from(i).expect("few villagers")))
                    .filter(|v| *v != me && near(ctx, me, *v, 40))
                    .filter(|v| {
                        matches!(
                            ctx.villager(*v).activity.as_str(),
                            "rest" | "keep-house" | "watch" | "sit" | "idle"
                        )
                    })
                    .min_by_key(|v| (self.mind.regard(*v).familiarity, *v));
                let topic = |v: VillagerId| ctx.topic("how-they-are").about(v);
                match who.and_then(|v| seek(ctx, me, v, Frame::SmallTalk, topic(v), 15)) {
                    Some(talk) => vec![Command::Note("on his rounds".into()), talk],
                    None => vec![Command::Note("nobody to call on today".into())],
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Elias {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 2,
            skill: 6,
            rejection_sensitivity: 4,
            charity: 3,
            discretion: 9,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Elias: a confidence is the warmest thing, and he takes even a complaint as trust. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Disclose => 3,
            G::Thank | G::Sympathise => 2,
            G::Complain => 1,
            G::Jab | G::Leave => -1,
            G::Rebuff => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Request
            | G::Offer
            | G::Decline
            | G::Invite
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
