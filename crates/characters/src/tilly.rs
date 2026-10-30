//! Tilly Hollins, ten. Catches frogs at the pond after school and tells everyone; keeps away from Ned.
//! Description and design notes: `docs/characters/tilly.md`.
use crate::Character;
use crate::toolbox::{activity_of, chat, go, go_area, is_weekend, villager, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tilly {
    pub mind: Memory,
}

impl Character for Tilly {
    const ID: &'static str = "tilly";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => crate::Fondness::Delighted,
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            // After school, frogs at the pond.
            Trigger::Arrived if activity_of(ctx, "tilly") == "chores" && !is_weekend(ctx) => {
                vec![
                    go_area(ctx, "pond-bank", "catch-frogs", rng)
                        .unwrap_or_else(|| wait_minutes(30, "chores")),
                ]
            }
            Trigger::Arrived if activity_of(ctx, "tilly") == "catch-frogs" => {
                vec![wait_minutes(60, "catch-frogs")]
            }
            Trigger::Waited if activity_of(ctx, "tilly") == "catch-frogs" => {
                vec![go(ctx, "trough_hollins", "chores")]
            }
            // She will not stay near Ned.
            Trigger::Encounter { other } if *other == villager(ctx, "ned") => {
                vec![go(ctx, "trough_hollins", "chores")]
            }
            // Anyone else hears about the frog.
            Trigger::Encounter { other } if activity_of(ctx, "tilly") != "sleep" => {
                let me = villager(ctx, "tilly");
                vec![chat(ctx, me, *other, Frame::SmallTalk, "frog", 6)?]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Tilly {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 2,
            skill: 3,
            rejection_sensitivity: 3,
            charity: 2,
            discretion: 2,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Tilly: jokes and news; a dig or a rebuff makes her shy. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke | G::Tell => 2,
            G::Ask => 1,
            G::Tease => -1,
            G::Jab | G::Rebuff => -2,
            G::Agree
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
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
