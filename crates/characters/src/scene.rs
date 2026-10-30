//! Test scenes. Set the town up at a moment (who stands where, their coins and needs, what
//! they remember, which way each chance falls), then run it and look at what happened.
//! A scene states its own preconditions, so it does not depend on a seed happening to
//! produce them, and it keeps passing when unrelated behavior changes.
//!
//! ```ignore
//! let mut s = Scene::new();
//! s.force_for(kit::DAWDLES, "kit", true);
//! s.run_until(at(1, 12, 0));
//! ```
use crate::Town;
use content::{Content, Pos, load_dir};
use sim::Sim;
use sim::brain::Command;
use sim::ids::{PlaceId, VillagerId};
use sim::input::{Input, InputKind, Source};
use sim::needs::Needs;
use sim::protocol::{Emitted, Frame};
use sim::social::{Episode, Interlocutor, Memory};
use sim::time::{TICKS_PER_DAY, TICKS_PER_MINUTE, Tick};
use std::path::PathBuf;

/// The town's content, from the repository's `content/` directory.
#[must_use]
pub fn town_content() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content"))
        .expect("content loads")
}

/// Day `day` at `hour:minute`.
#[must_use]
pub fn at(day: u32, hour: u32, minute: u32) -> Tick {
    Tick(day * TICKS_PER_DAY).at_time(hour, minute)
}

#[allow(missing_debug_implementations)]
pub struct Scene {
    pub sim: Sim<Town>,
    /// Everything emitted so far, across every `run_until`.
    pub events: Vec<Emitted>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    /// The town at the start of day 0. The seed is fixed; a scene that needs something to
    /// happen forces it rather than looking for a seed where it does.
    #[must_use]
    pub fn new() -> Scene {
        Scene {
            sim: Sim::new(1, town_content()),
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

    /// Forces chance `label` for everyone. Day plans are made at midnight (day 0's when the
    /// sim is built), so a chance that a plan reads takes effect from the next day.
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

    /// What `who` remembers.
    pub fn memory(&mut self, who: &str) -> &mut Memory {
        self.sim
            .state_mut()
            .characters
            .memory_of(who)
            .unwrap_or_else(|| panic!("{who} has no character module"))
    }

    /// Gives `who` `times` pleasant conversations with `with`, ending just before now, so
    /// they know them (familiarity `times`) and like them (feeling 2).
    pub fn acquaint(&mut self, who: &str, with: &str, times: u16) -> &mut Self {
        let other = self.id(with);
        let now = self.sim.tick();
        let mind = self.memory(who);
        for i in 0..times {
            mind.remember(Episode {
                tick: Tick(now.0.saturating_sub(u32::from(times - i) * TICKS_PER_DAY)),
                with: other,
                frame: Frame::SmallTalk,
                peak: 2,
                end: 2,
                they_seemed: 2,
                notes: Vec::new(),
            });
        }
        self
    }

    /// Moves `who` to the tile `(x, y)` of place `place` now, stopping whatever they were
    /// doing. They stand there until their plan or a trigger moves them on.
    pub fn put(&mut self, who: &str, place: &str, x: u16, y: u16) -> &mut Self {
        let who = self.id(who);
        let place = self.place(place);
        self.teleport(who, place, Pos { x, y });
        self
    }

    /// Moves `who` onto the first passable tile beside object `object`.
    pub fn put_by(&mut self, who: &str, object: &str) -> &mut Self {
        let (place, at) = {
            let o = self
                .sim
                .world()
                .objects
                .iter()
                .find(|o| o.id == object)
                .unwrap_or_else(|| panic!("no object {object}"));
            (PlaceId(u16::try_from(o.place).expect("few places")), o.at)
        };
        let spot = self.beside(place, at);
        let who = self.id(who);
        self.teleport(who, place, spot);
        self
    }

    /// Moves `who` onto the first passable tile beside `other`.
    pub fn put_beside(&mut self, who: &str, other: &str) -> &mut Self {
        let (place, pos) = {
            let ctx = self.sim.ctx();
            let v = ctx.villager(self.id(other));
            (v.place, v.pos)
        };
        let spot = self.beside(place, pos);
        let who = self.id(who);
        self.teleport(who, place, spot);
        self
    }

    /// Moves `who` next to the first animal of `flock` that is out.
    pub fn put_by_flock(&mut self, who: &str, flock: &str) -> &mut Self {
        let fi = self
            .sim
            .flocks()
            .iter()
            .position(|f| f.id == flock)
            .unwrap_or_else(|| panic!("no flock {flock}"));
        let (place, pos) = self
            .sim
            .state()
            .animals
            .iter()
            .find(|a| usize::from(a.flock) == fi && !a.roosting)
            .map_or_else(
                || panic!("all of {flock} are roosting"),
                |a| (a.place, a.pos),
            );
        let spot = self.beside(place, pos);
        let who = self.id(who);
        self.teleport(who, place, spot);
        self
    }

    /// Has `who` do `command` at `when` (today), as if their own code had planned it.
    pub fn cue(&mut self, who: &str, when: Tick, command: Command) -> &mut Self {
        let who = self.id(who);
        self.sim.cue(who, when, command);
        self
    }

    /// Has `who` start a conversation with `with` a minute from now.
    pub fn talk(
        &mut self,
        who: &str,
        with: &str,
        frame: Frame,
        topic: &str,
        minutes: u32,
    ) -> &mut Self {
        let other = self.id(with);
        let topic = self.sim.ctx().topic(topic);
        let when = self.sim.tick().plus(TICKS_PER_MINUTE);
        self.cue(
            who,
            when,
            Command::Talk {
                with: other,
                frame,
                topic,
                minutes,
            },
        )
    }

    /// The first passable tile next to `pos`, other than any where a villager stands.
    fn beside(&self, place: PlaceId, pos: Pos) -> Pos {
        let world = self.sim.world();
        let ctx = self.sim.ctx();
        let taken = |p: Pos| ctx.villagers.iter().any(|v| v.place == place && v.pos == p);
        [
            (1i32, 0i32),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
        ]
        .into_iter()
        .find_map(|(dx, dy)| {
            let x = u16::try_from(i32::from(pos.x) + dx).ok()?;
            let y = u16::try_from(i32::from(pos.y) + dy).ok()?;
            let p = Pos { x, y };
            (world.passable(place.index(), x, y) && !taken(p)).then_some(p)
        })
        .unwrap_or_else(|| panic!("nowhere to stand beside {pos:?}"))
    }

    fn place(&self, id: &str) -> PlaceId {
        self.sim
            .ctx()
            .place_id(id)
            .unwrap_or_else(|| panic!("no place {id}"))
    }

    fn teleport(&mut self, who: VillagerId, place: PlaceId, pos: Pos) {
        let tick = self.sim.tick();
        self.sim
            .apply_input(Input {
                tick,
                source: Source::System,
                kind: InputKind::Teleport {
                    who,
                    place,
                    x: pos.x,
                    y: pos.y,
                },
                seq: 0,
            })
            .expect("teleport accepted");
        // Inputs at the current tick apply on the next step.
        let next = tick.plus(1);
        let new = self.sim.step_until(next).expect("runs");
        self.events.extend(new);
    }
}

impl Town {
    /// The memory of the character with content id `who`.
    pub fn memory_of(&mut self, who: &str) -> Option<&mut Memory> {
        self.character(who).map(Interlocutor::memory_mut)
    }
}
