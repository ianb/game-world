//! Celia, the lady at the manor, widowed and lonely. Calls on the priest and the schoolmistress early in the week and asks them to tea on Thursday.
//! Description and design notes: `docs/characters/celia.md`.
use crate::Character;
use crate::toolbox::{Plan, activity_of, around, chat, go, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::protocol::{Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Celia {
    /// Whether she has asked anyone to tea this week.
    pub asked: bool,
    pub mind: Memory,
}

impl Character for Celia {
    const ID: &'static str = "celia";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Wary,
            content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "celia");
        let mut plan = Plan::baseline(me, ctx);
        match weekday(ctx) {
            Weekday::Mon => self.asked = false,
            Weekday::Tue => {
                plan.at(10, 30, go(ctx, "altar_chapel", "calling"));
            }
            Weekday::Wed => {
                plan.at(16, 0, go(ctx, "table_agnes", "calling"));
            }
            Weekday::Thu if self.asked => {
                plan.at(15, 45, go(ctx, "chair_celia", "tea"));
            }
            _ => {}
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
            // Calling: whoever she came to see, she asks to tea.
            Trigger::Arrived if activity_of(ctx, "celia") == "calling" => {
                let me = villager(ctx, "celia");
                let host = around(ctx, me, 6)
                    .into_iter()
                    .find(|v| matches!(ctx.villager(*v).id.as_str(), "elias" | "agnes"));
                match host.and_then(|h| chat(ctx, me, h, Frame::SmallTalk, "tea", 10)) {
                    Some(talk) => {
                        self.asked = true;
                        vec![talk]
                    }
                    None => vec![
                        Command::Note("nobody at home".into()),
                        go(ctx, "chair_celia", "rest"),
                    ],
                }
            }
            Trigger::TalkEnded { topic, .. }
                if topic.is("tea") && activity_of(ctx, "celia") == "calling" =>
            {
                vec![go(ctx, "chair_celia", "rest")]
            }
            // At tea, a conversation with each guest who comes.
            Trigger::Encounter { other } if activity_of(ctx, "celia") == "tea" => {
                let me = villager(ctx, "celia");
                vec![chat(ctx, me, *other, Frame::SmallTalk, "tea", 20)?]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Celia {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 0,
            warmth: 0,
            skill: 7,
            rejection_sensitivity: 6,
            charity: 0,
            discretion: 8,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Celia: courtesy and a compliment; familiarity, prying and being left are not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Thank | G::Compliment => 2,
            G::Agree => 1,
            G::Joke | G::Complain => -1,
            G::Tease | G::Pry | G::Leave => -2,
            G::Jab => -3,
            G::Ask
            | G::Tell
            | G::Request
            | G::Offer
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
