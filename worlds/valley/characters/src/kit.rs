//! Kit, the Coopers' boy. After school he goes looking for Pip, and some mornings he looks for him on the way to school and is late.
//! Description and design notes: `docs/characters/kit.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, go_area, is_weekend, seek, villager, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

/// Chance in a hundred he goes looking for Pip before school.
const DAWDLE_CHANCE: i16 = 30;
/// The chance label for looking for Pip before school.
pub const DAWDLES: &str = "kit-dawdles";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kit {
    pub mind: Memory,
}

impl Character for Kit {
    const ID: &'static str = "kit";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Delighted,
            crate::Animal::Sheep => crate::Fondness::Fond,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "kit");
        let mut plan = Plan::baseline(me, ctx);
        if !is_weekend(ctx)
            && ctx.chance(rng, me, DAWDLES, DAWDLE_CHANCE)
            && let Some(go) = go_area(ctx, "square", "look-for-pip", rng)
        {
            plan.shift((8, 40), (9, 40));
            plan.at(8, 40, go);
        }
        plan.build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        if let Some(chase) = crate::toolbox::chase_hens(ctx, "kit", trigger) {
            return Some(chase);
        }
        Some(match trigger {
            // Out to the square to look for Pip.
            Trigger::Arrived if activity_of(ctx, "kit") == "play" => {
                vec![
                    go_area(ctx, "square", "look-for-pip", rng)
                        .unwrap_or_else(|| wait_minutes(30, "play")),
                ]
            }
            Trigger::Arrived if activity_of(ctx, "kit") == "look-for-pip" => {
                vec![wait_minutes(15, "look-for-pip")]
            }
            Trigger::Waited if activity_of(ctx, "kit") == "look-for-pip" => {
                let me = villager(ctx, "kit");
                let pip = villager(ctx, "pip");
                match seek(ctx, me, pip, Frame::SmallTalk, ctx.topic("secret-den"), 15) {
                    Some(talk) => vec![talk],
                    None => vec![
                        go_area(ctx, "square", "look-for-pip", rng)
                            .unwrap_or_else(|| wait_minutes(15, "look-for-pip")),
                    ],
                }
            }
            Trigger::TalkEnded { other, .. } if *other == villager(ctx, "pip") => {
                vec![wait_minutes(20, "look-for-pip")]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Kit {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 3,
            warmth: 2,
            skill: 2,
            rejection_sensitivity: 2,
            charity: 2,
            discretion: 1,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Kit: jokes and news are the best things; being left is the worst. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke | G::Tell => 2,
            G::Tease => 0,
            G::Leave | G::Rebuff => -2,
            G::Ask
            | G::Agree
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
