//! One module per villager, plus the shared toolbox. Character code reads through `Ctx`
//! and writes by returning commands; it never mutates world state directly. Each villager
//! is its own struct whose fields are that villager's sparse attributes, and the `Town`
//! holds them as named fields so one villager's code can read another's and the compiler
//! checks it.
#![deny(clippy::float_arithmetic)]

pub mod mara;
pub mod pip;
pub mod tom;
pub mod toolbox;

use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{CharacterSet, Command, Ctx, Planned, Trigger, baseline_plan};
use sim::ids::VillagerId;

/// Every character in the town, by name. Adding a villager adds a field here and an arm in
/// each match below.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Town {
    pub mara: mara::Mara,
    pub tom: tom::Tom,
    pub pip: pip::Pip,
}

impl CharacterSet for Town {
    fn new(_content: &content::Content, _world: &content::World) -> Self {
        Town {
            mara: mara::Mara::default(),
            tom: tom::Tom::default(),
            pip: pip::Pip::default(),
        }
    }

    fn plan_day(&mut self, who: VillagerId, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let Town { mara, tom, pip } = self;
        match ctx.villager(who).id.as_str() {
            "mara" => mara.plan_day(ctx, rng, tom),
            "tom" => tom.plan_day(ctx, rng),
            "pip" => pip.plan_day(ctx, rng),
            _ => baseline_plan(who, ctx),
        }
    }

    fn react(
        &mut self,
        who: VillagerId,
        trigger: Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
    ) -> Vec<Command> {
        let Town { mara, tom, pip } = self;
        match ctx.villager(who).id.as_str() {
            "mara" => mara.react(trigger, ctx, tom),
            "tom" => tom.react(trigger, ctx, rng, mara),
            "pip" => pip.react(trigger, ctx, rng, mara),
            _ => Vec::new(),
        }
    }
}
