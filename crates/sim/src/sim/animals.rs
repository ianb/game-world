//! Animals on their own clocks: wandering in their area by day, roosting at night, laying
//! in the morning, and noticed by villagers who pass close.
use super::Sim;
use crate::animals::{Animal, Flock, LAY_CHANCE, LAYS, NOTICE_HOURS, step_gap, waking_hours};
use crate::brain::{CharacterSet, Trigger};
use crate::event::Event;
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::protocol::Emitted;
use crate::time::{TICKS_PER_DAY, TICKS_PER_HOUR, Tick};
use content::{AnimalKind, Pos};
use rand::RngExt;

/// Kind of `met` entry for a villager noticing a flock.
const NOTICED: u8 = 2;

impl<C: CharacterSet> Sim<C> {
    /// Flocks from content, references resolved. Validation has checked they resolve.
    pub(super) fn resolve_flocks(&self) -> Vec<Flock> {
        let mut out = Vec::new();
        for z in &self.content.zones {
            let place = PlaceId(u16::try_from(self.world.place_index[&z.id]).expect("few places"));
            for f in &z.flocks {
                let owner = self
                    .content
                    .villagers
                    .iter()
                    .position(|v| v.id == f.owner)
                    .map(|i| VillagerId(u16::try_from(i).expect("few villagers")));
                let area = self.world.area(&f.area).map(|a| a.rect);
                let (Some(owner), Some(area)) = (owner, area) else {
                    continue;
                };
                let coop = f.coop.as_ref().and_then(|c| {
                    self.world
                        .objects
                        .iter()
                        .position(|o| &o.id == c)
                        .map(|i| ObjectId(u16::try_from(i).expect("few objects")))
                });
                out.push(Flock {
                    id: f.id.clone(),
                    kind: f.kind,
                    owner,
                    place,
                    area,
                    coop,
                });
            }
        }
        out
    }

    /// Puts every animal somewhere in its area at the start, roosting if it has a coop, and
    /// schedules its first move.
    pub(super) fn place_animals(&mut self) {
        let counts: Vec<(usize, u8)> = self
            .content
            .zones
            .iter()
            .flat_map(|z| z.flocks.iter().map(|f| f.count))
            .enumerate()
            .collect();
        for (fi, count) in counts {
            let Some(flock) = self.flocks.get(fi).cloned() else {
                continue;
            };
            for _ in 0..count {
                let roosting = flock.coop.is_some();
                let pos = if let Some(coop) = flock.coop {
                    self.world.objects[coop.index()].at
                } else {
                    self.free_tile_in(&flock).unwrap_or(Pos {
                        x: flock.area.x,
                        y: flock.area.y,
                    })
                };
                let index = u16::try_from(self.state.animals.len()).expect("few animals");
                self.state.animals.push(Animal {
                    flock: u16::try_from(fi).expect("few flocks"),
                    kind: flock.kind,
                    place: flock.place,
                    pos,
                    roosting,
                });
                let (wake, _) = waking_hours(flock.kind);
                let at = Tick(wake + u32::from(index) * 37);
                self.state.queue.schedule(at, Event::Animal { index });
            }
        }
    }

    /// A random passable, unoccupied tile of the flock's area.
    fn free_tile_in(&mut self, flock: &Flock) -> Option<Pos> {
        for _ in 0..40 {
            let x = flock.area.x + self.state.rng.animals.random_range(0..flock.area.w);
            let y = flock.area.y + self.state.rng.animals.random_range(0..flock.area.h);
            if self.animal_can_stand(flock.place, Pos { x, y }, None) {
                return Some(Pos { x, y });
            }
        }
        None
    }

    fn animal_can_stand(&self, place: PlaceId, p: Pos, me: Option<usize>) -> bool {
        self.world.passable(place.index(), p.x, p.y)
            && !self
                .state
                .animals
                .iter()
                .enumerate()
                .any(|(i, a)| Some(i) != me && !a.roosting && a.place == place && a.pos == p)
    }

    pub(super) fn animal_step(&mut self, i: usize, out: &mut Vec<Emitted>) {
        let Some(a) = self.state.animals.get(i).cloned() else {
            return;
        };
        let flock = self.flocks[usize::from(a.flock)].clone();
        let tick = self.state.tick;
        let now = tick.seconds_into_day();
        let (wake, sleep) = waking_hours(a.kind);
        let index = u16::try_from(i).expect("few animals");
        let next_dawn = if now < wake {
            tick.day_start().plus(wake)
        } else {
            tick.day_start().plus(TICKS_PER_DAY + wake)
        };
        let awake = now >= wake && now < sleep;
        if a.roosting {
            if awake {
                self.come_out(i, &flock, out);
                self.schedule_step(index, a.kind);
            } else {
                self.state
                    .queue
                    .schedule(next_dawn, Event::Animal { index });
            }
            return;
        }
        if !awake {
            if let Some(coop) = flock.coop {
                let at = self.world.objects[coop.index()].at;
                if a.pos.x.abs_diff(at.x) + a.pos.y.abs_diff(at.y) <= 1 {
                    self.state.animals[i].roosting = true;
                    self.state.animals[i].pos = at;
                    self.state
                        .queue
                        .schedule(next_dawn, Event::Animal { index });
                } else {
                    self.step_toward(i, &flock, at);
                    self.state
                        .queue
                        .schedule(tick.plus(20), Event::Animal { index });
                }
            } else {
                // Lying down for the night where they are.
                self.state
                    .queue
                    .schedule(next_dawn, Event::Animal { index });
            }
            return;
        }
        self.wander(i, &flock);
        self.noticed_by_villagers(i, out);
        self.schedule_step(index, a.kind);
    }

    fn schedule_step(&mut self, index: u16, kind: AnimalKind) {
        let (lo, hi) = step_gap(kind);
        let gap = self.state.rng.animals.random_range(lo..=hi);
        self.state
            .queue
            .schedule(self.state.tick.plus(gap), Event::Animal { index });
    }

    /// Out of the coop in the morning, onto a free tile beside it, and perhaps an egg.
    fn come_out(&mut self, i: usize, flock: &Flock, out: &mut Vec<Emitted>) {
        let Some(coop) = flock.coop else {
            self.state.animals[i].roosting = false;
            return;
        };
        let at = self.world.objects[coop.index()].at;
        let spot = [(0i32, 1i32), (1, 0), (-1, 0), (0, -1), (1, 1), (-1, 1)]
            .into_iter()
            .filter_map(|(dx, dy)| {
                Some(Pos {
                    x: u16::try_from(i32::from(at.x) + dx).ok()?,
                    y: u16::try_from(i32::from(at.y) + dy).ok()?,
                })
            })
            .find(|p| self.animal_can_stand(flock.place, *p, Some(i)));
        let Some(spot) = spot else { return };
        self.state.animals[i].roosting = false;
        self.state.animals[i].pos = spot;
        if flock.kind == AnimalKind::Chicken
            && self
                .state
                .odds
                .chance_any(&mut self.state.rng.animals, LAYS, LAY_CHANCE)
        {
            let now = {
                let goods = self.state.stock.entry(coop.0).or_default();
                let n = goods.entry("eggs".to_string()).or_insert(0);
                *n = n.saturating_add(1);
                *n
            };
            out.push(Emitted::Laid {
                tick: self.state.tick,
                flock: flock.id.clone(),
                object: coop,
                now,
            });
        }
    }

    /// One step in a random direction, or none, staying inside the area.
    fn wander(&mut self, index: usize, flock: &Flock) {
        let dir = self.state.rng.animals.random_range(0..6u32);
        let (dx, dy) = match dir {
            0 => (1i32, 0i32),
            1 => (-1, 0),
            2 => (0, 1),
            3 => (0, -1),
            _ => return,
        };
        let from = self.state.animals[index].pos;
        let (Ok(nx), Ok(ny)) = (
            u16::try_from(i32::from(from.x) + dx),
            u16::try_from(i32::from(from.y) + dy),
        ) else {
            return;
        };
        let area = flock.area;
        if nx < area.x || ny < area.y || nx >= area.x + area.w || ny >= area.y + area.h {
            return;
        }
        let to = Pos { x: nx, y: ny };
        if self.animal_can_stand(flock.place, to, Some(index)) {
            self.state.animals[index].pos = to;
        }
    }

    /// A step that brings the animal closer to `to`, if one is free.
    fn step_toward(&mut self, i: usize, flock: &Flock, to: Pos) {
        let p = self.state.animals[i].pos;
        let mut options = Vec::new();
        if p.x < to.x {
            options.push(Pos { x: p.x + 1, y: p.y });
        }
        if p.x > to.x {
            options.push(Pos { x: p.x - 1, y: p.y });
        }
        if p.y < to.y {
            options.push(Pos { x: p.x, y: p.y + 1 });
        }
        if p.y > to.y {
            options.push(Pos { x: p.x, y: p.y - 1 });
        }
        if let Some(next) = options
            .into_iter()
            .find(|q| self.animal_can_stand(flock.place, *q, Some(i)))
        {
            self.state.animals[i].pos = next;
        } else {
            // Penned in by the others: a random step instead, so nobody stays stuck.
            self.wander(i, flock);
        }
    }

    /// Villagers standing next to animal `i` notice its flock.
    fn noticed_by_villagers(&mut self, i: usize, out: &mut Vec<Emitted>) {
        let a = self.state.animals[i].clone();
        let near: Vec<VillagerId> = self
            .state
            .villagers
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                v.place == a.place
                    && v.activity != "sleep"
                    && v.pos.x.abs_diff(a.pos.x) + v.pos.y.abs_diff(a.pos.y) <= 1
            })
            .map(|(vi, _)| VillagerId(u16::try_from(vi).expect("few villagers")))
            .collect();
        for who in near {
            self.notice(who, usize::from(a.flock), out);
        }
    }

    /// `who` just stepped: any animal next to them is noticed.
    pub(super) fn notice_animals(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let (place, pos) = {
            let v = &self.state.villagers[who.index()];
            (v.place, v.pos)
        };
        let flocks: Vec<usize> = self
            .state
            .animals
            .iter()
            .filter(|a| {
                !a.roosting
                    && a.place == place
                    && a.pos.x.abs_diff(pos.x) + a.pos.y.abs_diff(pos.y) <= 1
            })
            .map(|a| usize::from(a.flock))
            .collect();
        for f in flocks {
            self.notice(who, f, out);
        }
    }

    fn notice(&mut self, who: VillagerId, flock: usize, out: &mut Vec<Emitted>) {
        let key = (NOTICED, who.0, u16::try_from(flock).expect("few flocks"));
        let now = self.state.tick;
        if let Some(last) = self.state.met.get(&key)
            && now.0.saturating_sub(last.0) < NOTICE_HOURS * TICKS_PER_HOUR
        {
            return;
        }
        self.state.met.insert(key, now);
        let f = &self.flocks[flock];
        let trigger = Trigger::Animals {
            kind: f.kind,
            owner: f.owner,
        };
        self.react(who, trigger, out);
    }
}
