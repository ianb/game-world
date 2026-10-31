//! Hester, a widow on the square. Watches from her window, and tells anyone she meets who she saw go by.
//! Description and design notes: `docs/characters/hester.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, go, seek, seen_since, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hester {
    /// The last person she saw go by.
    pub last_seen: Option<VillagerId>,
    pub mind: Memory,
}

impl Character for Hester {
    const ID: &'static str = "hester";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Annoyed,
            content::AnimalKind::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "hester");
        let mut plan = Plan::baseline(me, ctx);
        plan.at(16, 30, go(ctx, "bench_square", "rest"));
        plan.build()
    }

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // At the window she takes note of whoever went past.
            Trigger::Look { since } if activity_of(ctx, "hester") == "watch" => {
                let me = villager(ctx, "hester");
                if let Some(who) = seen_since(ctx, me, *since).first() {
                    self.last_seen = Some(*who);
                }
                Vec::new()
            }
            // Whoever stops by hears about the last person she saw.
            Trigger::Encounter { other } if self.last_seen.is_some_and(|w| w != *other) => {
                let me = villager(ctx, "hester");
                let who = self.last_seen?;
                let topic = ctx.topic("comings-and-goings").about(who);
                vec![seek(ctx, me, *other, Frame::SmallTalk, topic, 10)?]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Hester {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 0,
            skill: 5,
            rejection_sensitivity: 3,
            charity: -1,
            discretion: 1,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Hester: news and confidences are meat and drink; being left or put off is not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Tell | G::Disclose => 3,
            G::Ask => 1,
            G::Decline | G::Leave | G::Rebuff => -2,
            G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Invite
            | G::Jab
            | G::Awkward
            | G::Pry => gesture.warmth(),
        }
    }
}
