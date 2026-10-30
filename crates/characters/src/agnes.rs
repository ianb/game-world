//! Agnes, the schoolmistress. Keeps count of pupils who are late and tells their parents.
//! Description and design notes: `docs/characters/agnes.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, between, go, heard_lately, seek, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;
use std::collections::BTreeMap;

/// Her pupils, and whom she tells when one keeps coming late.
const PARENTS: [(&str, [&str; 2]); 3] = [
    ("kit", ["rob", "jenna"]),
    ("tilly", ["will", "ada"]),
    ("benny", ["penn", "rosa"]),
];
/// Late mornings before she says something.
const TELL_AFTER: u8 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agnes {
    /// Mornings each pupil was not at their desk at half past nine, since she last told a
    /// parent.
    pub late: BTreeMap<VillagerId, u8>,
    pub mind: Memory,
}

impl Character for Agnes {
    const ID: &'static str = "agnes";
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "agnes");
        let mut plan = Plan::baseline(me, ctx);
        if weekday(ctx) == Weekday::Thu && heard_lately(ctx, &self.mind, "celia", "tea", 4) {
            plan.at(16, 0, go(ctx, "chair_parlour_1", "tea"));
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
            // Half past nine on a school day: who is not at their desk.
            Trigger::Look { .. }
                if activity_of(ctx, "agnes") == "teach" && between(ctx, (9, 30), (9, 45)) =>
            {
                let mut cmds = Vec::new();
                for (pupil, _) in PARENTS {
                    if activity_of(ctx, pupil) != "school" {
                        let id = villager(ctx, pupil);
                        *self.late.entry(id).or_default() += 1;
                        cmds.push(Command::Note(format!(
                            "{} is not at their desk",
                            ctx.villager(id).name
                        )));
                    }
                }
                cmds
            }
            // Out of school, meeting the parent of a pupil who keeps coming late: she says so.
            Trigger::Encounter { other } if activity_of(ctx, "agnes") != "teach" => {
                let me = villager(ctx, "agnes");
                let child = PARENTS.iter().find_map(|(pupil, parents)| {
                    let child = villager(ctx, pupil);
                    (parents.iter().any(|p| villager(ctx, p) == *other)
                        && self.late.get(&child).is_some_and(|n| *n >= TELL_AFTER))
                    .then_some(child)
                })?;
                let topic = ctx.topic("being-late").about(child);
                let talk = seek(ctx, me, *other, Frame::SmallTalk, topic, 10)?;
                self.late.remove(&child);
                vec![talk]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Agnes {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: -1,
            skill: 7,
            rejection_sensitivity: 3,
            charity: -2,
            discretion: 4,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Agnes: respect and order please her; teasing and slackness do not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank => 2,
            G::Compliment | G::Agree => 1,
            G::Joke | G::Awkward | G::Leave => -1,
            G::Tease | G::Pry => -2,
            G::Jab => -3,
            G::Ask
            | G::Tell
            | G::Complain
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Rebuff => gesture.warmth(),
        }
    }
}
