//! Ned, who drinks. Short of money he asks whoever he meets, and does not ask again this week anyone who gave nothing.
//! Description and design notes: `docs/characters/ned.md`.
use crate::Character;
use crate::toolbox::{ALE_PRICE, Plan, seek, think, villager, weekday};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::Weekday;
use std::collections::BTreeSet;

const ASKS_A_DAY: u8 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ned {
    /// Who gave him nothing this week.
    pub refused: BTreeSet<VillagerId>,
    /// How many he has asked today.
    pub asked_today: u8,
    /// Whom he last asked, and his coins when the talk ended, to see by the next look
    /// whether anything came of it.
    pub asked: Option<(VillagerId, u32)>,
    pub mind: Memory,
}

impl Character for Ned {
    const ID: &'static str = "ned";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: crate::Animal) -> crate::Fondness {
        match kind {
            crate::Animal::Chicken => crate::Fondness::Annoyed,
            crate::Animal::Sheep => crate::Fondness::Indifferent,
        }
    }
    fn plan_day(&mut self, ctx: &Ctx<'_>, _rng: &mut Pcg32) -> Vec<Planned> {
        let me = villager(ctx, "ned");
        self.asked_today = 0;
        if weekday(ctx) == Weekday::Mon {
            self.refused.clear();
        }
        Plan::baseline(me, ctx).build()
    }

    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            // Short of money, he asks whoever he meets, unless they gave nothing this week;
            // twice a day at most, he has some pride.
            Trigger::Encounter { other }
                if ctx.coins(villager(ctx, "ned")) < ALE_PRICE
                    && self.asked_today < ASKS_A_DAY
                    && !self.refused.contains(other) =>
            {
                let me = villager(ctx, "ned");
                let topic = ctx.topic("money").about(me);
                let talk = seek(ctx, me, *other, Frame::SmallTalk, topic, 8)?;
                self.asked_today += 1;
                vec![talk]
            }
            Trigger::TalkEnded { other, topic } if topic.is("money") => {
                self.asked = Some((*other, ctx.coins(villager(ctx, "ned"))));
                Vec::new()
            }
            // By the next look he knows whether anything came of it.
            Trigger::Look { .. } if self.asked.is_some() => {
                let me = villager(ctx, "ned");
                let (who, before) = self.asked.take()?;
                if ctx.coins(me) > before {
                    Vec::new()
                } else {
                    self.refused.insert(who);
                    vec![think(
                        ctx.topic("money").about(me),
                        Blame::Them(who),
                        false,
                        false,
                        -1,
                    )]
                }
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Ned {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 2,
            warmth: 0,
            skill: 3,
            rejection_sensitivity: 5,
            charity: -1,
            discretion: 3,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Ned: an offer is everything; being turned down stings. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Offer => 3,
            G::Thank | G::Sympathise => 1,
            G::Pry => -1,
            G::Decline | G::Jab => -2,
            G::Rebuff => -3,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Disclose
            | G::Invite
            | G::Leave
            | G::Awkward => gesture.warmth(),
        }
    }
}
