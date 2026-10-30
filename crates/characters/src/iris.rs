//! Iris, Amos's daughter, a weaver. Notices who keeps seeking whom out, and keeps it to herself.
//! Description and design notes: `docs/characters/iris.md`.
use crate::Character;
use crate::toolbox::{activity_of, between, stock, stock_of, think, villager};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Trigger};
use sim::ids::VillagerId;
use sim::protocol::{Blame, Emitted, Frame, Gesture};
use sim::social::{Interlocutor, Memory, Persona};
use sim::time::TICKS_PER_DAY;
use std::collections::{BTreeMap, BTreeSet};

const MAX_CLOTH: u16 = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Iris {
    /// Pairs she has already noticed, so she thinks it once.
    pub noticed: BTreeSet<(VillagerId, VillagerId)>,
    pub mind: Memory,
}

impl Character for Iris {
    const ID: &'static str = "iris";

    #[deny(clippy::wildcard_enum_match_arm)]
    fn fondness(&self, kind: content::AnimalKind) -> crate::Fondness {
        match kind {
            content::AnimalKind::Chicken => crate::Fondness::Wary,
            content::AnimalKind::Sheep => crate::Fondness::Fond,
        }
    }
    fn react(
        &mut self,
        trigger: &Trigger,
        ctx: &Ctx<'_>,
        _rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived
                if activity_of(ctx, "iris") == "weave"
                    && stock_of(ctx, "loom_reed", "cloth") < MAX_CLOTH =>
            {
                vec![stock(ctx, "loom_reed", "cloth", 1)]
            }
            // Late afternoon at the loom she thinks over the day: a pair who sought each
            // other out twice or more, she notices.
            Trigger::Look { .. }
                if activity_of(ctx, "iris") == "weave" && between(ctx, (16, 0), (16, 15)) =>
            {
                let me = villager(ctx, "iris");
                let mut pairs: BTreeMap<(VillagerId, VillagerId), u8> = BTreeMap::new();
                for (r, _) in ctx.recall(me, ctx.tick.minus(TICKS_PER_DAY)) {
                    if let Emitted::ConversationStarted {
                        who, with, frame, ..
                    } = &r.event
                        && *frame != Frame::Transaction
                    {
                        let key = if who < with {
                            (*who, *with)
                        } else {
                            (*with, *who)
                        };
                        *pairs.entry(key).or_default() += 1;
                    }
                }
                let mut cmds = Vec::new();
                for (pair, n) in pairs {
                    if n >= 2 && self.noticed.insert(pair) {
                        let topic = ctx.topic("sweet-on").about(pair.0).with(pair.1);
                        cmds.push(think(topic, Blame::World, false, false, 0));
                    }
                }
                cmds
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Iris {
    fn persona(&self) -> Persona {
        Persona {
            sociable: -1,
            warmth: 1,
            skill: 6,
            rejection_sensitivity: 4,
            charity: 2,
            discretion: 10,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Iris: sympathy matters; prying and teasing do not. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Sympathise => 2,
            G::Thank => 1,
            G::Tease | G::Jab | G::Pry => -2,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff => gesture.warmth(),
        }
    }
}
