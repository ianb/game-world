//! Pell, the steward. Collects the rents from the two farms on Mondays, and remembers who was short.
//! Description and design notes: `docs/characters/pell.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, go, pay, seek, think, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;
use std::collections::BTreeSet;

const TENANTS: [&str; 2] = ["will", "silas"];
const RENT: u32 = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pell {
    /// Which tenant he is on in this round.
    pub round: u8,
    /// Tenants who were short last time.
    pub owing: BTreeSet<VillagerId>,
    pub mind: Memory,
}

impl Character for Pell {
    const ID: &'static str = "pell";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Annoyed,
            content::AnimalKind::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "pell");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Mon {
            self.round = 0;
            plan.at(10, 0, go(ctx, "desk_pell", "collect-rent"));
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
            // The rent round: each tenant in turn, wherever they are working.
            Trigger::Arrived if activity_of(ctx, "pell") == "collect-rent" => {
                Pell::next_tenant(self, ctx)
            }
            Trigger::TalkEnded { other, topic } if topic.is("the-rent") => {
                let mut cmds = Vec::new();
                if ctx.coins(*other) >= RENT {
                    self.owing.remove(other);
                    cmds.push(pay(ctx, &ctx.villager(*other).id, "celia", RENT, "rent"));
                } else {
                    self.owing.insert(*other);
                    cmds.push(think(
                        ctx.topic("the-rent").about(*other),
                        Blame::Them(*other),
                        false,
                        false,
                        -1,
                    ));
                }
                cmds.extend(Pell::next_tenant(self, ctx));
                cmds
            }
            _ => return None,
        })
    }
}

impl Pell {
    /// Seeks out the next tenant on the round, or goes back to the accounts.
    fn next_tenant(&mut self, ctx: &Ctx<'_>) -> Vec<Command> {
        let me = villager(ctx, "pell");
        while let Some(tenant) = TENANTS.get(usize::from(self.round)) {
            self.round += 1;
            let t = villager(ctx, tenant);
            if let Some(talk) = seek(
                ctx,
                me,
                t,
                Frame::Transaction,
                ctx.topic("the-rent").about(t),
                10,
            ) {
                return vec![talk];
            }
        }
        vec![go(ctx, "desk_pell", "accounts")]
    }
}

impl Interlocutor for Pell {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: -1,
            skill: 7,
            rejection_sensitivity: 3,
            charity: -2,
            discretion: 8,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Pell: agreement and proper thanks; complaint and familiarity are not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Agree => 2,
            G::Thank => 1,
            G::Joke => -1,
            G::Complain | G::Tease | G::Jab => -2,
            G::Rebuff => -3,
            G::Ask
            | G::Tell
            | G::Compliment
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
