//! Nell, a pensioner. Her pension (in her content file) brings money into the town once a
//! week, and it leaves through the bakery and the tavern, which is most of what makes the
//! economy a loop.
//! She is shy: anxious with anyone she does not know well or with others listening, at
//! ease only with people she has come to like. When something goes wrong she blames
//! herself and expects it to keep happening.
use crate::toolbox::{Plan, activity_of, buy_ale, buy_bread, counter_talk, think, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Gesture, Mishap, Stance, Topic};
use sim::social::{Interlocutor, Memory, Persona, Situation, default_stance};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nell {
    pub days_without_bread: u8,
    pub mind: Memory,
}

impl Nell {
    pub fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("nell").expect("nell exists");
        Plan::baseline(me, ctx).build()
    }

    pub fn react(&mut self, trigger: &Trigger, ctx: &Ctx<'_>) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived if activity_of(ctx, "nell") == "shop" => {
                if let Some(mut cmds) = buy_bread(ctx, "nell") {
                    self.days_without_bread = 0;
                    cmds.extend(counter_talk(ctx, "nell", "mara", "bread"));
                    cmds
                } else {
                    self.days_without_bread = self.days_without_bread.saturating_add(1);
                    vec![Command::Note("no bread to be had".into())]
                }
            }
            Trigger::Arrived if activity_of(ctx, "nell") == "drink" => {
                let mut cmds = buy_ale(ctx, "nell")
                    .unwrap_or_else(|| vec![Command::Note("no ale tonight".into())]);
                cmds.extend(counter_talk(ctx, "nell", "tom", "ale"));
                cmds
            }
            Trigger::Mishap {
                what: Mishap::Tripped,
            } => vec![think(
                ctx.topic("tripped").about(villager(ctx, "nell")),
                Blame::Me,
                true,
                false,
                -1,
            )],
            // Her name in someone else's talk: she assumes she did something wrong.
            Trigger::Overheard { about_me: true, .. } => {
                vec![think(
                    ctx.topic("talked-about").about(villager(ctx, "nell")),
                    Blame::Me,
                    true,
                    false,
                    -1,
                )]
            }
            Trigger::Passing { other } if activity_of(ctx, "nell") != "sleep" => {
                vec![Command::Greet { with: *other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Nell {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -2,
            warmth: 1,
            skill: 4,
            rejection_sensitivity: 7,
            charity: 0,
            discretion: 7,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Nell: sympathy is the warmest thing there is; teasing,
    /// prying and being left all sting. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Sympathise => 3,
            G::Joke => 0,
            G::Jab => -3,
            G::Pry => -2,
            G::Tease | G::Leave => -1,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }

    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<Topic> {
        if self.days_without_bread >= 1 {
            Some(ctx.topic("going-without").about(me))
        } else if ctx.coins(me) < 3 {
            Some(ctx.topic("money").about(me))
        } else {
            None
        }
    }

    fn stance(&self, s: &Situation<'_>) -> Stance {
        let r = self.mind.regard(s.other);
        if r.feeling >= 1 && r.familiarity >= 4 {
            default_stance(self, s)
        } else if r.familiarity < 4 || !s.private() {
            Stance::Anxious
        } else {
            default_stance(self, s)
        }
    }
}

impl crate::Character for Nell {
    const ID: &'static str = "nell";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken | content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        Nell::plan_day(self, ctx, rng)
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Nell::react(self, trigger, ctx)
    }
}
