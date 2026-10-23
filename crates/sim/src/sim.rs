//! The `Sim` handle: step, apply inputs, emit events, snapshot, hash.
use crate::brain::{
    CharacterSet, Command, Ctx, ENCOUNTER_COOLDOWN, ENCOUNTER_RANGE, Target, Trigger,
};
use crate::event::Event;
use crate::hash::{HashParts, hash_of};
use crate::ids::{PlaceId, VillagerId};
use crate::input::{Input, InputKind, Source};
use crate::protocol::Emitted;
use crate::queue::EventQueue;
use crate::rng::Streams;
use crate::snapshot::{self, SnapshotError};
use crate::time::{TICKS_PER_DAY, Tick};
use crate::villager::{Movement, VillagerState};
use content::{Content, Nav, Pos, Spot, WALK_TICKS_PER_TILE, World, nav};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Everything that determines the future. Serializable in full; hashed by subsystem.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimState<C> {
    pub seed: u64,
    /// Hash of the content this state was built from; a snapshot is refused against
    /// different content.
    pub content_hash: u64,
    pub tick: Tick,
    pub rng: Streams,
    pub queue: EventQueue,
    /// Every input ever applied, in `(tick, seq)` order. The save game.
    pub inputs: Vec<Input>,
    pub villagers: Vec<VillagerState>,
    /// Last proximity trigger per (kind, pair with smaller id first), for the cooldown.
    /// Kind 0 is an encounter, 1 is passing.
    pub met: BTreeMap<(u8, u16, u16), Tick>,
    pub day_rollovers: u32,
    pub markers: Vec<(Tick, String)>,
    /// Character private state.
    pub characters: C,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SimError {
    #[error("input tick {input} is before the current tick {current}")]
    InputInPast { input: Tick, current: Tick },
    #[error("cannot step backwards from {current} to {target}")]
    StepBackwards { current: Tick, target: Tick },
    #[error("snapshot was made from different content")]
    ContentMismatch,
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

/// A town in motion. `content`, `world` and `nav` are derived from content and never
/// serialized; `state` is the whole truth.
#[derive(Debug)]
pub struct Sim<C: CharacterSet> {
    content: Content,
    world: World,
    nav: Nav,
    state: SimState<C>,
    startup: Vec<Emitted>,
}

/// Content is hashed through JSON rather than postcard because the schedule target uses
/// `serde(flatten)`, which postcard cannot encode. Field order is fixed, so this is stable.
#[must_use]
pub fn content_hash(content: &Content) -> u64 {
    let bytes = serde_json::to_vec(content).expect("content is always serializable");
    xxhash_rust::xxh3::xxh3_64(&bytes)
}

impl<C: CharacterSet> Sim<C> {
    /// A new town at midnight before day 1. Villagers start in their beds (or at their home
    /// place's arrival tile). The first day is planned immediately.
    #[must_use]
    pub fn new(seed: u64, content: Content) -> Sim<C> {
        let world = content::compile::compile(&content);
        let nav = Nav::build(&world);
        let villagers = content
            .villagers
            .iter()
            .map(|v| {
                let (place, pos) = start_spot(&world, &v.bed);
                VillagerState {
                    id: v.id.clone(),
                    name: v.name.clone(),
                    place,
                    pos,
                    generation: 0,
                    plan_generation: 0,
                    activity: "sleep".into(),
                    moving: None,
                    plan: BTreeMap::new(),
                    since: Tick::ZERO,
                    resume: None,
                }
            })
            .collect();
        let characters = C::new(&content, &world);
        let mut queue = EventQueue::default();
        queue.schedule(Tick(TICKS_PER_DAY), Event::DayRollover);
        let mut sim = Sim {
            state: SimState {
                seed,
                content_hash: content_hash(&content),
                tick: Tick::ZERO,
                rng: Streams::from_seed(seed),
                queue,
                inputs: Vec::new(),
                villagers,
                met: BTreeMap::new(),
                day_rollovers: 0,
                markers: Vec::new(),
                characters,
            },
            content,
            world,
            nav,
            startup: Vec::new(),
        };
        let mut out = Vec::new();
        sim.plan_everyone(&mut out);
        sim.startup = out;
        sim
    }

    /// Decisions made while constructing the sim (the first day's plans). Returned by the
    /// first `step_until` so no trace is lost.
    fn take_startup(&mut self) -> Vec<Emitted> {
        std::mem::take(&mut self.startup)
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        self.state.tick
    }

    #[must_use]
    pub fn state(&self) -> &SimState<C> {
        &self.state
    }

    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub fn content(&self) -> &Content {
        &self.content
    }

    #[must_use]
    pub fn nav(&self) -> &Nav {
        &self.nav
    }

    #[must_use]
    pub fn ctx(&self) -> Ctx<'_> {
        Ctx {
            tick: self.state.tick,
            calendar: self.state.tick.calendar(),
            content: &self.content,
            world: &self.world,
            nav: &self.nav,
            villagers: &self.state.villagers,
        }
    }

    /// Queues an input for its tick. Inputs at the current tick are applied on the next step.
    pub fn apply_input(&mut self, input: Input) -> Result<(), SimError> {
        if input.tick < self.state.tick {
            return Err(SimError::InputInPast {
                input: input.tick,
                current: self.state.tick,
            });
        }
        self.state.queue.schedule(input.tick, Event::Input(input));
        Ok(())
    }

    /// Runs every event due at or before `target`, in `(tick, seq)` order, then sets the
    /// clock to `target`. Returns what happened, for viewers and external actors.
    pub fn step_until(&mut self, target: Tick) -> Result<Vec<Emitted>, SimError> {
        if target < self.state.tick {
            return Err(SimError::StepBackwards {
                current: self.state.tick,
                target,
            });
        }
        let mut out = self.take_startup();
        while let Some(scheduled) = self.state.queue.pop_due(target) {
            self.state.tick = scheduled.tick;
            self.handle(scheduled.event, &mut out);
        }
        self.state.tick = target;
        Ok(out)
    }

    fn handle(&mut self, event: Event, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        match event {
            Event::DayRollover => {
                self.state.day_rollovers += 1;
                self.state
                    .queue
                    .schedule(tick.plus(TICKS_PER_DAY), Event::DayRollover);
                out.push(Emitted::DayStarted {
                    tick,
                    calendar: tick.calendar(),
                });
                self.plan_everyone(out);
            }
            Event::Input(input) => self.handle_input(input, out),
            Event::WakeUp { who, generation } => {
                if self.state.villagers[who.index()].plan_generation != generation {
                    return;
                }
                let Some(cmd) = self.state.villagers[who.index()].plan.remove(&tick) else {
                    return;
                };
                self.run_command(who, cmd, out);
            }
            Event::Arrive { who, generation } => {
                if self.state.villagers[who.index()].generation == generation {
                    self.step_movement(who, out);
                }
            }
            Event::Waited { who, generation } => {
                if self.state.villagers[who.index()].generation == generation {
                    self.react(who, Trigger::Waited, out);
                }
            }
            Event::TalkEnd {
                who,
                other,
                topic,
                generation,
            } => {
                if self.state.villagers[who.index()].generation != generation {
                    return;
                }
                let resume = self.state.villagers[who.index()]
                    .resume
                    .take()
                    .unwrap_or_else(|| "idle".into());
                self.set_activity(who, &resume, out);
                self.react(who, Trigger::TalkEnded { other, topic }, out);
            }
            Event::ObjectFree { .. } => {}
        }
    }

    fn handle_input(&mut self, input: Input, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let summary = match (&input.source, &input.kind) {
            (_, InputKind::Noop) => "noop".to_string(),
            (_, InputKind::Marker(text)) => {
                self.state.markers.push((tick, text.clone()));
                format!("marker {text:?}")
            }
            (Source::System, InputKind::Teleport { who, x, y }) => {
                let v = &mut self.state.villagers[who.index()];
                v.pos = Pos { x: *x, y: *y };
                v.moving = None;
                v.generation += 1;
                format!("teleport {who:?} to ({x}, {y})")
            }
            (source, InputKind::Teleport { .. }) => {
                out.push(Emitted::InputRejected {
                    tick,
                    reason: format!("{source:?} may not teleport"),
                });
                return;
            }
        };
        self.state.inputs.push(input);
        out.push(Emitted::InputApplied { tick, summary });
    }

    /// Asks every character for today's plan and schedules the wake-ups. Items already in
    /// the past (at sim start, or if the plan lists them) are dropped.
    fn plan_everyone(&mut self, out: &mut Vec<Emitted>) {
        let now = self.state.tick;
        let day_start = now.day_start();
        let count = self.state.villagers.len();
        for i in 0..count {
            let who = VillagerId(u16::try_from(i).expect("few villagers"));
            let (planned, text) = {
                let ctx = Ctx {
                    tick: now,
                    calendar: now.calendar(),
                    content: &self.content,
                    world: &self.world,
                    nav: &self.nav,
                    villagers: &self.state.villagers,
                };
                let planned =
                    self.state
                        .characters
                        .plan_day(who, &ctx, &mut self.state.rng.schedule);
                let mut sorted = planned.clone();
                sorted.sort_by_key(|p| p.at);
                let items: Vec<String> = sorted
                    .iter()
                    .map(|p| format!("{} {}", p.at, p.command.summary(&ctx)))
                    .collect();
                (planned, format!("plan: {}", items.join("; ")))
            };
            out.push(Emitted::Decided {
                tick: now,
                who,
                text,
            });
            let v = &mut self.state.villagers[i];
            v.plan.clear();
            v.plan_generation += 1;
            let generation = v.plan_generation;
            for p in planned {
                let at = day_start.plus(p.at.seconds());
                if at < now {
                    continue;
                }
                v.plan.insert(at, p.command);
                self.state
                    .queue
                    .schedule(at, Event::WakeUp { who, generation });
            }
        }
    }

    fn react(&mut self, who: VillagerId, trigger: Trigger, out: &mut Vec<Emitted>) {
        let (commands, text) = {
            let ctx = Ctx {
                tick: self.state.tick,
                calendar: self.state.tick.calendar(),
                content: &self.content,
                world: &self.world,
                nav: &self.nav,
                villagers: &self.state.villagers,
            };
            let label = trigger.label(&ctx);
            let commands =
                self.state
                    .characters
                    .react(who, trigger, &ctx, &mut self.state.rng.idle);
            let text = (!commands.is_empty()).then(|| {
                let items: Vec<String> = commands.iter().map(|c| c.summary(&ctx)).collect();
                format!("{label} -> {}", items.join("; "))
            });
            (commands, text)
        };
        if let Some(text) = text {
            out.push(Emitted::Decided {
                tick: self.state.tick,
                who,
                text,
            });
        }
        for cmd in commands {
            self.run_command(who, cmd, out);
        }
    }

    fn run_command(&mut self, who: VillagerId, cmd: Command, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        match cmd {
            Command::Idle => {
                let v = &mut self.state.villagers[who.index()];
                v.moving = None;
                v.generation += 1;
                self.set_activity(who, "idle", out);
            }
            Command::Wait { ticks, activity } => {
                let v = &mut self.state.villagers[who.index()];
                v.moving = None;
                v.generation += 1;
                let generation = v.generation;
                self.set_activity(who, &activity, out);
                self.state
                    .queue
                    .schedule(tick.plus(ticks), Event::Waited { who, generation });
            }
            Command::GoTo { target, activity } => self.go_to(who, &target, activity, out),
            Command::Note(_) => {}
            Command::Greet { with } => {
                if with == who || with.index() >= self.state.villagers.len() {
                    return;
                }
                out.push(Emitted::Talked {
                    tick,
                    who,
                    with,
                    topic: "greeting".into(),
                    minutes: 0,
                });
                self.react(
                    with,
                    Trigger::TalkedTo {
                        other: who,
                        topic: "greeting".into(),
                    },
                    out,
                );
            }
            Command::Talk {
                with,
                topic,
                minutes,
            } => self.talk(who, with, topic, minutes, out),
        }
    }

    fn talk(
        &mut self,
        who: VillagerId,
        with: VillagerId,
        topic: String,
        minutes: u32,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        if with == who || with.index() >= self.state.villagers.len() {
            return;
        }
        let ok = {
            let a = &self.state.villagers[who.index()];
            let b = &self.state.villagers[with.index()];
            a.place == b.place
                && a.moving.is_none()
                && b.moving.is_none()
                && a.activity != "talk"
                && b.activity != "talk"
        };
        if !ok {
            out.push(Emitted::Decided {
                tick,
                who,
                text: format!(
                    "wanted to talk with {} but could not (apart, walking, or busy talking)",
                    self.state.villagers[with.index()].name
                ),
            });
            return;
        }
        out.push(Emitted::Talked {
            tick,
            who,
            with,
            topic: topic.clone(),
            minutes,
        });
        for (me, other) in [(who, with), (with, who)] {
            let v = &mut self.state.villagers[me.index()];
            v.resume = Some(v.activity.clone());
            v.generation += 1;
            let generation = v.generation;
            self.set_activity(me, "talk", out);
            self.state.queue.schedule(
                tick.plus(minutes * crate::time::TICKS_PER_MINUTE),
                Event::TalkEnd {
                    who: me,
                    other,
                    topic: topic.clone(),
                    generation,
                },
            );
        }
        self.react(with, Trigger::TalkedTo { other: who, topic }, out);
    }

    /// Whether a proximity trigger may fire for this pair now; records it if so.
    fn pair_ready(&mut self, kind: u8, a: VillagerId, b: VillagerId) -> bool {
        let key = if a.0 <= b.0 {
            (kind, a.0, b.0)
        } else {
            (kind, b.0, a.0)
        };
        let now = self.state.tick;
        match self.state.met.get(&key) {
            Some(&last) if now.0.saturating_sub(last.0) < ENCOUNTER_COOLDOWN => false,
            _ => {
                self.state.met.insert(key, now);
                true
            }
        }
    }

    /// Together: same room, or both outside any room and within `ENCOUNTER_RANGE` tiles.
    fn together(&self, a: &VillagerState, b: &VillagerState) -> bool {
        if a.place != b.place {
            return false;
        }
        let place = a.place.index();
        match (
            self.world.room_at(place, a.pos.x, a.pos.y),
            self.world.room_at(place, b.pos.x, b.pos.y),
        ) {
            (Some(ra), Some(rb)) => ra == rb,
            (None, None) => {
                a.pos.x.abs_diff(b.pos.x) + a.pos.y.abs_diff(b.pos.y) <= ENCOUNTER_RANGE
            }
            _ => false,
        }
    }

    /// `who` just came to rest: fire `Encounter` for every other resting villager with them.
    fn encounters(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let me = self.state.villagers[who.index()].clone();
        let others: Vec<VillagerId> = self
            .state
            .villagers
            .iter()
            .enumerate()
            .filter(|(i, v)| {
                *i != who.index()
                    && v.moving.is_none()
                    && !unavailable(v)
                    && !unavailable(&me)
                    && self.together(&me, v)
            })
            .map(|(i, _)| VillagerId(u16::try_from(i).expect("few villagers")))
            .collect();
        for other in others {
            if self.pair_ready(0, who, other) {
                self.react(who, Trigger::Encounter { other }, out);
                self.react(other, Trigger::Encounter { other: who }, out);
            }
        }
    }

    /// `who` just stepped: fire `Passing` for anyone within one tile.
    fn passing(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let (place, pos) = {
            let v = &self.state.villagers[who.index()];
            (v.place, v.pos)
        };
        let others: Vec<VillagerId> = self
            .state
            .villagers
            .iter()
            .enumerate()
            .filter(|(i, v)| {
                *i != who.index()
                    && v.place == place
                    && !unavailable(v)
                    && v.pos.x.abs_diff(pos.x) + v.pos.y.abs_diff(pos.y) <= 1
            })
            .map(|(i, _)| VillagerId(u16::try_from(i).expect("few villagers")))
            .collect();
        for other in others {
            if self.pair_ready(1, who, other) {
                self.react(who, Trigger::Passing { other }, out);
                self.react(other, Trigger::Passing { other: who }, out);
            }
        }
    }

    fn set_activity(&mut self, who: VillagerId, activity: &str, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let v = &mut self.state.villagers[who.index()];
        if v.activity != activity {
            v.activity = activity.to_string();
            v.since = tick;
        }
        out.push(Emitted::Activity {
            tick,
            who,
            activity: activity.to_string(),
        });
    }

    fn resolve_target(&self, who: VillagerId, target: &Target) -> Option<Spot> {
        match target {
            Target::Place(p) => {
                let place = &self.world.places[p.index()];
                Some(Spot {
                    place: p.index(),
                    pos: place.arrival,
                })
            }
            Target::Tile { place, pos } => Some(Spot {
                place: place.index(),
                pos: *pos,
            }),
            Target::Object(o) => {
                let obj = &self.world.objects[o.index()];
                if obj.walkable {
                    return Some(Spot {
                        place: obj.place,
                        pos: obj.at,
                    });
                }
                // Nearest access tile to where the villager stands, by BFS from the villager
                // when in the same place, else the first passable neighbour in N,E,S,W order.
                let v = &self.state.villagers[who.index()];
                let candidates: Vec<Pos> = content::validate::neighbours(obj.at.x, obj.at.y)
                    .into_iter()
                    .filter(|&(x, y)| self.world.passable(obj.place, x, y))
                    .map(|(x, y)| Pos { x, y })
                    .collect();
                if v.place.index() == obj.place {
                    let dist = nav::bfs_distances(
                        &self.world,
                        Spot {
                            place: obj.place,
                            pos: v.pos,
                        },
                    );
                    candidates
                        .iter()
                        .copied()
                        .filter_map(|c| dist.get(&c).map(|d| (*d, c)))
                        .min()
                        .map(|(_, c)| Spot {
                            place: obj.place,
                            pos: c,
                        })
                } else {
                    candidates.first().map(|&c| Spot {
                        place: obj.place,
                        pos: c,
                    })
                }
            }
        }
    }

    /// Whether `who` may pass `door` now: hours and owner conditions.
    fn door_allowed(&self, who: VillagerId, door: usize) -> bool {
        let d = &self.world.doors[door];
        let v = &self.state.villagers[who.index()];
        if d.owner.as_deref() == Some(v.id.as_str()) {
            return true;
        }
        match d.hours.as_deref().and_then(content::validate::parse_hours) {
            None => true,
            Some((open, close)) => {
                let now = self.state.tick.seconds_into_day();
                if open <= close {
                    (open..=close).contains(&now)
                } else {
                    now >= open || now <= close
                }
            }
        }
    }

    fn go_to(
        &mut self,
        who: VillagerId,
        target: &Target,
        activity: String,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let Some(dest) = self.resolve_target(who, target) else {
            out.push(Emitted::Stuck {
                tick,
                who,
                reason: "target has no reachable tile".into(),
            });
            self.react(who, Trigger::Stuck, out);
            return;
        };
        let here = {
            let v = &self.state.villagers[who.index()];
            Spot {
                place: v.place.index(),
                pos: v.pos,
            }
        };
        if here == dest {
            let v = &mut self.state.villagers[who.index()];
            v.moving = None;
            v.generation += 1;
            self.set_activity(who, &activity, out);
            self.react(who, Trigger::Arrived, out);
            self.encounters(who, out);
            return;
        }
        let route = self
            .nav
            .route_filtered(&self.world, here, dest, |d| self.door_allowed(who, d));
        let Some(route) = route else {
            out.push(Emitted::Stuck {
                tick,
                who,
                reason: format!(
                    "no route to {} ({}, {})",
                    self.world.places[dest.place].id, dest.pos.x, dest.pos.y
                ),
            });
            self.react(who, Trigger::Stuck, out);
            return;
        };
        let eta = tick.plus(route.ticks());
        let mut legs: Vec<(PlaceId, Pos, Option<u16>)> = route
            .legs
            .iter()
            .map(|l| {
                (
                    PlaceId(u16::try_from(l.place).expect("few places")),
                    l.to,
                    l.door.map(|d| u16::try_from(d).expect("few doors")),
                )
            })
            .collect();
        let first = legs.remove(0);
        let path =
            nav::grid_path(&self.world, first.0.index(), here.pos, first.1).unwrap_or_default();
        let v = &mut self.state.villagers[who.index()];
        v.generation += 1;
        let generation = v.generation;
        v.moving = Some(Movement {
            legs,
            path: path.into_iter().skip(1).collect(),
            door_after: first.2,
            then: activity.clone(),
        });
        self.set_activity(who, "walking", out);
        out.push(Emitted::Departed {
            tick,
            who,
            activity,
            eta,
        });
        self.state.queue.schedule(
            tick.plus(WALK_TICKS_PER_TILE),
            Event::Arrive { who, generation },
        );
    }

    /// One movement step: advance a tile, cross a door at a leg's end, or arrive.
    fn step_movement(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let v = &mut self.state.villagers[who.index()];
        let Some(m) = v.moving.as_mut() else { return };
        let mut stepped = false;
        if !m.path.is_empty() {
            let next = m.path.remove(0);
            v.pos = next;
            let (place, pos) = (v.place, v.pos);
            out.push(Emitted::Moved {
                tick,
                who,
                place,
                x: pos.x,
                y: pos.y,
            });
            stepped = true;
        }
        if m.path.is_empty() {
            // Leg finished. Cross a door if one is pending, then start the next leg.
            if let Some(door) = m.door_after.take() {
                let d = &self.world.doors[usize::from(door)];
                let (to_place, to) = if v.place.index() == d.from_place && v.pos == d.from {
                    (d.to_place, d.to)
                } else {
                    (d.from_place, d.from)
                };
                v.place = PlaceId(u16::try_from(to_place).expect("few places"));
                v.pos = to;
                let (place, pos) = (v.place, v.pos);
                out.push(Emitted::Moved {
                    tick,
                    who,
                    place,
                    x: pos.x,
                    y: pos.y,
                });
            }
            if m.legs.is_empty() {
                let activity = m.then.clone();
                v.moving = None;
                v.generation += 1;
                self.set_activity(who, &activity, out);
                self.react(who, Trigger::Arrived, out);
                self.encounters(who, out);
                return;
            }
            let (place, to, door_after) = m.legs.remove(0);
            let path = nav::grid_path(&self.world, place.index(), v.pos, to).unwrap_or_default();
            m.path = path.into_iter().skip(1).collect();
            m.door_after = door_after;
        }
        let generation = v.generation;
        self.state.queue.schedule(
            tick.plus(WALK_TICKS_PER_TILE),
            Event::Arrive { who, generation },
        );
        if stepped {
            self.passing(who, out);
        }
    }

    #[must_use]
    pub fn hash(&self) -> HashParts {
        let s = &self.state;
        HashParts::combine(
            hash_of(&(s.tick, s.content_hash, s.seed)),
            hash_of(&s.rng),
            hash_of(&s.queue),
            hash_of(&(
                &s.villagers,
                &s.met,
                s.day_rollovers,
                &s.markers,
                &s.characters,
            )),
            hash_of(&s.inputs),
        )
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<u8> {
        snapshot::encode(&self.state)
    }

    /// Restores a snapshot against the same content it was made from.
    pub fn restore(bytes: &[u8], content: Content) -> Result<Sim<C>, SimError> {
        let state: SimState<C> = snapshot::decode(bytes)?;
        if state.content_hash != content_hash(&content) {
            return Err(SimError::ContentMismatch);
        }
        let world = content::compile::compile(&content);
        let nav = Nav::build(&world);
        Ok(Sim {
            content,
            world,
            nav,
            state,
            startup: Vec::new(),
        })
    }
}

/// Asleep or mid-conversation: no proximity triggers to or about them.
fn unavailable(v: &VillagerState) -> bool {
    matches!(v.activity.as_str(), "sleep" | "talk")
}

/// Where a villager begins: the access tile of its bed, else place 0's arrival.
fn start_spot(world: &World, bed: &str) -> (PlaceId, Pos) {
    if let Some(obj) = world.objects.iter().find(|o| o.id == bed) {
        let access = content::validate::neighbours(obj.at.x, obj.at.y)
            .into_iter()
            .find(|&(x, y)| world.passable(obj.place, x, y));
        if let Some((x, y)) = access {
            return (
                PlaceId(u16::try_from(obj.place).expect("few places")),
                Pos { x, y },
            );
        }
    }
    (PlaceId(0), world.places[0].arrival)
}
