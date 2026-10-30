//! Sal, the Coopers' daughter, apprenticed to Bram. Stays on at the forge to practise once he has gone.
//! Description and design notes: `docs/characters/sal.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, between, is_weekend, think, villager, wait_minutes};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Blame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sal {
    pub mind: Memory,
}

impl Character for Sal {
    const ID: &'static str = "sal";
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "sal");
        let mut plan = Plan::baseline(me, ctx);
        if !is_weekend(ctx) {
            plan.shift((18, 0), (19, 15));
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
            // With Bram gone from the anvil she stays on and practises.
            Trigger::Look { .. }
                if activity_of(ctx, "sal") == "smith"
                    && between(ctx, (18, 0), (18, 15))
                    && activity_of(ctx, "bram") != "smith" =>
            {
                let me = villager(ctx, "sal");
                vec![
                    think(ctx.topic("the-forge").about(me), Blame::Me, true, false, 2),
                    wait_minutes(60, "practise"),
                ]
            }
            Trigger::Waited if activity_of(ctx, "sal") == "practise" => {
                vec![Command::Note("done for the day".into())]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Sal {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 1,
            skill: 5,
            rejection_sensitivity: 4,
            charity: 0,
            discretion: 6,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Sal: praise for her work means the most. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Compliment => 3,
            G::Thank => 2,
            G::Tease | G::Decline => -1,
            G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }
}
