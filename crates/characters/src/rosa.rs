//! Rosa, Penn's wife. Keeps geese on the green, and when Benny is not home by evening she goes and finds him.
//! Description and design notes: `docs/characters/rosa.md`.
use crate::Character;
use crate::toolbox::{activity_of, go, hour, seek, think, villager, work_area};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::protocol::{Blame, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rosa {
    pub mind: Memory,
}

impl Character for Rosa {
    const ID: &'static str = "rosa";
    fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived | Trigger::Waited if activity_of(ctx, "rosa") == "keep-geese" => {
                work_area(
                    ctx,
                    "rosa",
                    trigger,
                    "hamlet-green",
                    "keep-geese",
                    30,
                    0,
                    rng,
                )
            }
            // Evening and Benny is hiding: she goes to find him.
            Trigger::Look { .. }
                if hour(ctx) >= 19
                    && activity_of(ctx, "benny") == "hiding"
                    && activity_of(ctx, "rosa") != "sleep" =>
            {
                let me = villager(ctx, "rosa");
                let benny = villager(ctx, "benny");
                let topic = ctx.topic("hiding").about(benny);
                vec![
                    think(topic.clone(), Blame::Them(benny), false, false, -1),
                    seek(ctx, me, benny, Frame::SmallTalk, topic, 5)?,
                ]
            }
            Trigger::TalkEnded { topic, .. } if topic.is("hiding") => {
                vec![go(ctx, "table_dory", "keep-house")]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Rosa {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 2,
            skill: 4,
            rejection_sensitivity: 6,
            charity: 1,
            discretion: 5,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Rosa: sympathy above all; being walked away from hurts. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Sympathise => 2,
            G::Thank | G::Agree => 1,
            G::Tease => -1,
            G::Leave | G::Jab => -2,
            G::Ask
            | G::Tell
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }
}
