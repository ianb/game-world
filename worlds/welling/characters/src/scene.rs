//! Test scenes for the Welling: the city at a moment, set up by hand, then run.
use crate::City;
use content::{Content, load_dir};
use sim::Sim;
use sim::ids::VillagerId;
use sim::needs::Needs;
use sim::protocol::Emitted;
use sim::time::{TICKS_PER_DAY, Tick};
use std::path::PathBuf;

/// The Welling's content, from `worlds/welling/content/`.
#[must_use]
pub fn city_content() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../content")).expect("content loads")
}

/// Day `day` at `hour:minute`.
#[must_use]
pub fn at(day: u32, hour: u32, minute: u32) -> Tick {
    Tick(day * TICKS_PER_DAY).at_time(hour, minute)
}

#[allow(missing_debug_implementations)]
pub struct Scene {
    pub sim: Sim<City>,
    /// Everything emitted so far, across every `run_until`.
    pub events: Vec<Emitted>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    /// The city at the start of day 0, from a fixed seed.
    #[must_use]
    pub fn new() -> Scene {
        Scene {
            sim: Sim::new(1, city_content()),
            events: Vec::new(),
        }
    }

    #[must_use]
    pub fn id(&self, who: &str) -> VillagerId {
        self.sim
            .ctx()
            .villager_id(who)
            .unwrap_or_else(|| panic!("no villager {who}"))
    }

    /// Runs to `until` and returns what was emitted on the way.
    pub fn run_until(&mut self, until: Tick) -> &[Emitted] {
        let from = self.events.len();
        let new = self.sim.step_until(until).expect("runs");
        self.events.extend(new);
        &self.events[from..]
    }

    /// Forces chance `label` for everyone.
    pub fn force(&mut self, label: &str, happens: bool) -> &mut Self {
        self.sim.state_mut().odds.force(label, happens);
        self
    }

    /// Forces chance `label` for `who` only.
    pub fn force_for(&mut self, label: &str, who: &str, happens: bool) -> &mut Self {
        let id = self.id(who);
        self.sim.state_mut().odds.force_for(label, id, happens);
        self
    }

    pub fn set_coins(&mut self, who: &str, coins: u32) -> &mut Self {
        let id = self.id(who);
        self.sim.state_mut().villagers[id.index()].coins = coins;
        self
    }

    pub fn needs(&mut self, who: &str) -> &mut Needs {
        let id = self.id(who);
        &mut self.sim.state_mut().villagers[id.index()].needs
    }

    /// How many of `good` are on `object` now.
    #[must_use]
    pub fn stock(&self, object: &str, good: &str) -> u16 {
        let ctx = self.sim.ctx();
        ctx.object_id(object).map_or(0, |o| ctx.stock(o, good))
    }

    /// The activity `who` is at now.
    #[must_use]
    pub fn activity(&self, who: &str) -> String {
        self.sim.ctx().villager(self.id(who)).activity.clone()
    }
}
