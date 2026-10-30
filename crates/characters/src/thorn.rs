//! Thorn, the manor's gardener. Frets aloud when the garden gets ahead of him, and stays late to catch up.
//! Description and design notes: `docs/characters/thorn.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, go, think_aloud, villager, work_area};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

/// Overgrown tiles in the garden before it weighs on him.
const WILD: usize = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thorn {
    /// Whether he has fretted about it today.
    pub fretted: bool,
    pub mind: Memory,
}

impl Character for Thorn {
    const ID: &'static str = "thorn";
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "thorn");
        self.fretted = false;
        let mut plan = Plan::baseline(me, ctx);
        if Thorn::wild(ctx) > WILD {
            plan.at(18, 45, go(ctx, "well_manor", "garden"));
        }
        plan.build()
    }

    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "thorn") == "garden" => {
                let me = villager(ctx, "thorn");
                let mut cmds =
                    work_area(ctx, "thorn", trigger, "manor-garden", "garden", 30, 2, rng);
                if matches!(trigger, Trigger::Waited) && !self.fretted && Thorn::wild(ctx) > WILD {
                    self.fretted = true;
                    cmds.push(think_aloud(
                        ctx.topic("the-garden").about(me),
                        Blame::World,
                        true,
                        false,
                        -2,
                    ));
                }
                cmds
            }
            _ => return None,
        })
    }
}

impl Thorn {
    /// Overgrown tiles in the garden.
    fn wild(ctx: &Ctx<'_>) -> usize {
        let (place, rect) = ctx.area("manor-garden");
        ctx.overgrown_tiles(place)
            .iter()
            .filter(|p| rect.contains(p.x, p.y))
            .count()
    }
}

impl Interlocutor for Thorn {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: 0,
            skill: 3,
            rejection_sensitivity: 5,
            charity: -1,
            discretion: 7,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Thorn: praise for the garden; complaint and teasing are not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Compliment => 2,
            G::Thank => 1,
            G::Complain => -1,
            G::Tease | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }
}
