//! Will Hollins, farmer. Walks the boundary with Ashby's land of an evening and holds it against Silas when he finds them there.
//! Description and design notes: `docs/characters/will.md`.
use crate::Character;
use crate::toolbox::{
    Plan, activity_of, go, go_area, near, tend_fields, think_aloud, villager, wait_minutes, weekday,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture, Stance};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_stance};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Will {
    /// Times he has found Ashby's people at the boundary, easing by a little each Sunday.
    pub grudge: u8,
    pub mind: Memory,
}

impl Character for Will {
    const ID: &'static str = "will";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Indifferent,
            content::AnimalKind::Sheep => crate::Fondness::Annoyed,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "will");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Sun {
            self.grudge = self.grudge.saturating_sub(1);
        } else if let Some(go) = go_area(ctx, "boundary", "patrol", rng) {
            plan.at(17, 0, go);
        }
        plan.build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            // Field work: weed a patch, then the next.
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "will") == "farm" => {
                tend_fields(ctx, "will", trigger, "hollins-fields", "farm", rng)
            }
            Trigger::Arrived if activity_of(ctx, "will") == "patrol" => {
                vec![wait_minutes(30, "patrol")]
            }
            // Anyone of Ashby's close to the boundary counts against them.
            Trigger::Waited if activity_of(ctx, "will") == "patrol" => {
                let me = villager(ctx, "will");
                let mut cmds = Vec::new();
                let found = ["silas", "dan"]
                    .into_iter()
                    .map(|v| villager(ctx, v))
                    .find(|v| near(ctx, me, *v, 8));
                if let Some(v) = found {
                    self.grudge = self.grudge.saturating_add(1);
                    cmds.push(think_aloud(
                        ctx.topic("the-boundary").about(v),
                        Blame::Them(v),
                        true,
                        false,
                        -2,
                    ));
                }
                cmds.push(go(ctx, "table_hollins", "rest"));
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Will {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: 0,
            skill: 4,
            rejection_sensitivity: 4,
            charity: -1,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Will: plain dealing pleases him; complaint and prying do not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Agree | G::Thank => 1,
            G::Complain => -1,
            G::Jab | G::Rebuff | G::Pry => -2,
            G::Ask
            | G::Tell
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward => gesture.warmth(),
        }
    }

    /// Cold with Silas once the grudge has built up.
    fn stance(&self, s: &Situation<'_>) -> Stance {
        if self.grudge >= 2 && s.other == villager(s.ctx, "silas") {
            Stance::Cold
        } else {
            default_stance(self, s)
        }
    }
}
