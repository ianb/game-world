//! Clem, the carrier. Makes a round of the valley twice a day, and passes on what he hears at each stop.
//! Description and design notes: `docs/characters/clem.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, around, chat, go, in_building, villager, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

/// The day's round, in order.
const STOPS: [&str; 6] = [
    "counter_store",
    "bar_alehouse",
    "anvil_smithy",
    "counter_bakery",
    "bar_tavern",
    "millstone",
];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clem {
    /// The next stop on today's round.
    pub stop: u8,
    pub mind: Memory,
}

impl Character for Clem {
    const ID: &'static str = "clem";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Fond,
            content::AnimalKind::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "clem");
        let mut plan = Plan::baseline(me, ctx);
        plan.at(14, 0, go(ctx, "table_clem", "carry"));
        plan.build()
    }

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // At each stop, a word with whoever is there, then on. Starting from home
            // starts the round over.
            Trigger::Arrived if activity_of(ctx, "clem") == "carry" => {
                let me = villager(ctx, "clem");
                if in_building(ctx, me, "lodging") {
                    self.stop = 0;
                }
                let talk = around(ctx, me, 4)
                    .into_iter()
                    .find_map(|v| chat(ctx, me, v, Frame::SmallTalk, "the-news", 8));
                match talk {
                    Some(talk) => vec![talk],
                    None => vec![wait_minutes(5, "carry")],
                }
            }
            Trigger::Waited | Trigger::TalkEnded { .. } if activity_of(ctx, "clem") == "carry" => {
                match STOPS.get(usize::from(self.stop)) {
                    Some(next) => {
                        self.stop += 1;
                        vec![go(ctx, next, "carry")]
                    }
                    None => vec![go(ctx, "table_clem", "rest")],
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Clem {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 3,
            warmth: 2,
            skill: 5,
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

    /// What each gesture means to Clem: news is what he lives on; being put off is not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Tell => 3,
            G::Ask | G::Thank => 1,
            G::Decline | G::Leave => -1,
            G::Rebuff => -2,
            G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
