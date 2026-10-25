//! Pip, a placeholder kid: roams the square all day, greets everyone who passes, and
//! turns up at the bakery at noon to ask Mara for a bun. Eclectic by design, to exercise
//! wandering, greetings and conversations. In talk he ignores the frame: he tells anyone
//! anything, wherever they are, and never minds when others do the same.
use crate::toolbox::{
    Plan, activity_of, counter_talk, go_tile, pick, stock, stock_of, wait_minutes,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Act, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona, Situation, act, default_next_act, roll};
use sim::time::Tick;

/// Things a child might say to anyone.
const NEWS: [&str; 4] = ["frog", "loose-tooth", "secret-den", "dead-mouse"];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pip {
    /// When he last asked for a bun.
    pub asked_at: Option<Tick>,
    pub mind: Memory,
}

impl Pip {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("pip").expect("pip exists");
        Plan::baseline(me, ctx).build()
    }

    pub fn react(
        &mut self,
        trigger: Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // Roaming: pick a random ground tile, walk there, pause, repeat.
            Trigger::Arrived if activity_of(ctx, "pip") == "roam" => {
                vec![wait_minutes(*pick(rng, &[3u32, 5, 8]), "roam")]
            }
            Trigger::Waited if activity_of(ctx, "pip") == "roam" => {
                let town = ctx.place_id("town").expect("town exists");
                match ctx.random_tile(town, content::Cell::Ground, rng) {
                    Some(p) => vec![go_tile(ctx, "town", p.x, p.y, "roam")],
                    None => vec![wait_minutes(5, "roam")],
                }
            }
            // At the bakery: ask for a bun.
            Trigger::Arrived if activity_of(ctx, "pip") == "pester" => {
                match counter_talk(ctx, "pip", "mara", "bun") {
                    Some(talk) => {
                        self.asked_at = Some(ctx.tick);
                        vec![talk]
                    }
                    None => vec![Command::Note("nobody at the bakery to ask".into())],
                }
            }
            // He takes a bun if he understood Mara to offer one.
            Trigger::TalkEnded { other, topic } if topic.is("bun") => {
                let asked = self.asked_at.unwrap_or_default();
                let offered = self
                    .mind
                    .heard()
                    .from(other)
                    .topic("bun")
                    .gesture(Gesture::Offer)
                    .since(asked)
                    .any();
                if offered && stock_of(ctx, "shelf_bakery", "bread") > 0 {
                    vec![
                        stock(ctx, "shelf_bakery", "bread", -1),
                        Command::Note("got a bun".into()),
                    ]
                } else {
                    vec![Command::Note("no bun today".into())]
                }
            }
            // Children fall over all the time; he does not give it a thought.
            Trigger::Mishap { .. } => Vec::new(),
            // An unroutable roam target: shrug and try somewhere else.
            Trigger::Stuck => {
                let town = ctx.place_id("town").expect("town exists");
                match ctx.random_tile(town, content::Cell::Ground, rng) {
                    Some(p) => vec![
                        Command::Note("couldn't get there; trying elsewhere".into()),
                        go_tile(ctx, "town", p.x, p.y, "roam"),
                    ],
                    None => vec![wait_minutes(5, "roam")],
                }
            }
            // Greets anyone passing, once per cooldown; the sim handles the pairing.
            Trigger::Passing { other } | Trigger::Encounter { other }
                if activity_of(ctx, "pip") != "sleep" =>
            {
                vec![Command::Greet { with: other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Pip {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 3,
            warmth: 2,
            skill: 2,
            rejection_sensitivity: 1,
            charity: 2,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Pip: jokes and being asked things delight him, and a no is
    /// the worst thing there is. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Joke => 2,
            G::Ask => 1,
            G::Decline => -2,
            G::Complain => 0,
            G::Tell
            | G::Agree
            | G::Tease
            | G::Compliment
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }

    /// Once the business is done, a quarter of the time he tells whatever is on his mind,
    /// whatever the frame.
    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        if s.turn >= 1 && s.turn < 4 && s.frame != Frame::Confiding && roll(rng) < 25 {
            return act(Gesture::Disclose, &s.ctx.topic(pick(rng, &NEWS)));
        }
        default_next_act(self, s, rng)
    }

    /// He never sanctions anyone; he joins in.
    fn out_of_frame(&mut self, s: &Situation<'_>, did: &Act, rng: &mut Pcg32) -> Act {
        let _ = (s, did);
        act(Gesture::Tell, &s.ctx.topic(pick(rng, &NEWS)))
    }
}
