//! Benny, the Dorys' boy, seven. Some nights at bedtime he runs off and hides until his mother finds him.
//! Description and design notes: `docs/characters/benny.md`.
use crate::Character;
use crate::toolbox::{activity_of, go, go_area, hour, villager, wait_minutes, work_area};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::Gesture;
use sim::social::{Interlocutor, Memory, Persona};

/// Chance in a hundred he hides instead of going to bed.
const HIDE_CHANCE: i16 = 30;
/// The chance label for hiding at bedtime.
pub const HIDES: &str = "benny-hides";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Benny {
    pub mind: Memory,
}

impl Character for Benny {
    const ID: &'static str = "benny";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Delighted,
            content::AnimalKind::Sheep => crate::Fondness::Afraid,
        }
    }
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        if let Some(chase) = crate::toolbox::chase_hens(ctx, "benny", trigger) {
            return Some(chase);
        }
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "benny") == "play" => {
                work_area(ctx, "benny", trigger, "hamlet-green", "play", 20, 0, rng)
            }
            // Bedtime: some nights he runs off and hides instead.
            Trigger::Arrived
                if activity_of(ctx, "benny") == "sleep"
                    && hour(ctx) < 21
                    && ctx.chance(rng, villager(ctx, "benny"), HIDES, HIDE_CHANCE) =>
            {
                vec![go_area(ctx, "hiding-spots", "hiding", rng)?]
            }
            Trigger::Arrived if activity_of(ctx, "benny") == "hiding" => {
                vec![wait_minutes(90, "hiding")]
            }
            // Found, or tired of it: home to bed.
            Trigger::TalkEnded { other, .. } if *other == villager(ctx, "rosa") => {
                vec![go(ctx, "bed_benny", "sleep")]
            }
            Trigger::Waited if activity_of(ctx, "benny") == "hiding" => {
                vec![go(ctx, "bed_benny", "sleep")]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Benny {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 3,
            warmth: 2,
            skill: 2,
            rejection_sensitivity: 1,
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

    /// What each gesture means to Benny: jokes and teasing are all fun; being asked where he has been is not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 3,
            G::Tease | G::Tell => 1,
            G::Ask | G::Pry | G::Rebuff => -1,
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
            | G::Jab
            | G::Awkward => gesture.warmth(),
        }
    }
}
