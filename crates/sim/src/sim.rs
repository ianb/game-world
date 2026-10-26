//! The `Sim` handle: step, apply inputs, emit events, snapshot, hash.
use crate::brain::{
    CharacterSet, Command, Ctx, ENCOUNTER_COOLDOWN, ENCOUNTER_RANGE, Party, Planned, TALK_REACH,
    Target, Trigger,
};
use crate::event::Event;
use crate::hash::{HashParts, hash_of};
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::input::{Input, InputKind, Source};
use crate::needs::{Need, Needs};
use crate::protocol::{Cause, Emitted, Frame, Mishap, PlanItem, Topic};
use crate::queue::EventQueue;
use crate::recall::{self, Observer, Record};
use crate::rng::Streams;
use crate::snapshot::{self, SnapshotError};
use crate::social::Conversation;
use crate::terrain::{Surface, Terrain};
use crate::time::{TICKS_PER_DAY, TICKS_PER_HOUR, TICKS_PER_MINUTE, Tick};
use crate::villager::{Approach, Crossing, Errand, Interlude, Movement, PlannedLeg, VillagerState};
use content::{Content, Nav, Pos, Spot, WALK_TICKS_PER_TILE, World, nav};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Nested reactions (a greeting answered by a greeting answered by...) stop here.
const MAX_REACT_DEPTH: u8 = 4;
/// How often an awake villager takes in their surroundings.
const LOOK_TICKS: u32 = 15 * TICKS_PER_MINUTE;
/// Chance per thousand steps of tripping in brush or an overgrown path, and in long grass
/// or a weedy path.
const TRIP_OVERGROWN: u32 = 150;
const TRIP_LONG_GRASS: u32 = 40;

mod conversation;

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
    /// Every input ever accepted, in acceptance order, each carrying its `seq`. The save game.
    pub inputs: Vec<Input>,
    pub next_input_seq: u32,
    pub villagers: Vec<VillagerState>,
    /// Last proximity trigger per (kind, pair with smaller id first), for the cooldown.
    /// Kind 0 is an encounter, 1 is passing.
    pub met: BTreeMap<(u8, u16, u16), Tick>,
    /// Goods on objects, by object index then good name. Sloppy by design: no weights,
    /// no capacities, just counts where somebody cares.
    pub stock: BTreeMap<u16, BTreeMap<String, u16>>,
    /// Conversations in progress, by id.
    pub conversations: BTreeMap<u32, Conversation>,
    pub next_conversation: u32,
    /// Wear and growth on every outdoor tile: cowpaths form where villagers walk, grass
    /// and weeds grow where they do not.
    pub terrain: Terrain,
    pub day_rollovers: u32,
    pub markers: Vec<(Tick, String)>,
    /// Recent events worth knowing, with who could perceive them.
    pub record: Record,
    /// Character private state.
    pub characters: C,
    /// Events produced while constructing the sim (the first day's plans), returned by the
    /// first `step_until`. Part of the state so a restored sim behaves the same.
    pub pending: Vec<Emitted>,
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
    /// Transient: current nesting of `react` calls.
    depth: u8,
    /// Transient: why the commands now running are running. Set before every dispatch.
    cause: Cause,
}

/// Content is hashed through JSON rather than postcard because the schedule target uses
/// `serde(flatten)`, which postcard cannot encode. Field order is fixed, so this is stable.
#[must_use]
pub fn content_hash(content: &Content) -> u64 {
    let bytes = serde_json::to_vec(content).expect("content is always serializable");
    xxhash_rust::xxh3::xxh3_64(&bytes)
}

/// Asleep or mid-conversation: no proximity triggers to or about them.
fn unavailable(v: &VillagerState) -> bool {
    matches!(v.activity.as_str(), "sleep" | "talk")
}

impl<C: CharacterSet> Sim<C> {
    /// A new town at midnight before day 1. Villagers start in their beds (or at their home
    /// place's arrival tile). The first day is planned immediately.
    #[must_use]
    pub fn new(seed: u64, content: Content) -> Sim<C> {
        let world = content::compile::compile(&content);
        let nav = Nav::build(&world);
        let villagers: Vec<VillagerState> = content
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
                    talk_until: None,
                    wait_until: None,
                    coins: v.coins,
                    needs: Needs::default(),
                    interlude: None,
                    resume_wait: None,
                    cause: Cause::Sim,
                    resume_cause: None,
                    approach: None,
                }
            })
            .collect();
        let characters = C::new(&content, &world);
        let ids: Vec<&str> = content.villagers.iter().map(|v| v.id.as_str()).collect();
        let mut rng = Streams::from_seed(seed, &ids);
        let mut villagers = villagers;
        for (v, streams) in villagers.iter_mut().zip(&mut rng.villagers) {
            v.needs = Needs::starting(&mut streams.body);
        }
        let mut queue = EventQueue::default();
        queue.schedule(Tick(TICKS_PER_DAY), Event::DayRollover);
        // Each body ticks hourly, staggered a few minutes apart so nobody acts in step.
        for (i, _) in villagers.iter().enumerate() {
            let who = VillagerId(u16::try_from(i).expect("few villagers"));
            let offset = u32::try_from(i).expect("few villagers") * 7 * TICKS_PER_MINUTE;
            queue.schedule(Tick(TICKS_PER_HOUR + offset), Event::Body { who });
            queue.schedule(Tick(LOOK_TICKS + offset / 7), Event::Look { who });
        }
        let mut sim = Sim {
            state: SimState {
                seed,
                content_hash: content_hash(&content),
                tick: Tick::ZERO,
                rng,
                queue,
                inputs: Vec::new(),
                next_input_seq: 1,
                villagers,
                met: BTreeMap::new(),
                stock: starting_stock(&content, &world),
                conversations: BTreeMap::new(),
                next_conversation: 0,
                terrain: Terrain::new(&world),
                day_rollovers: 0,
                markers: Vec::new(),
                record: Record::default(),
                characters,
                pending: Vec::new(),
            },
            content,
            world,
            nav,
            depth: 0,
            cause: Cause::Sim,
        };
        let mut out = Vec::new();
        sim.plan_everyone(&mut out);
        sim.state.pending = out;
        sim
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
            terrain: &self.state.terrain,
            stock: &self.state.stock,
            record: &self.state.record,
        }
    }

    /// Accepts an input: assigns its sequence number, logs it, and queues it for its tick.
    /// Inputs at the current tick are applied on the next step. The order of inputs is the
    /// order of acceptance, independent of internal events scheduled before or after.
    pub fn apply_input(&mut self, mut input: Input) -> Result<(), SimError> {
        if input.tick < self.state.tick {
            return Err(SimError::InputInPast {
                input: input.tick,
                current: self.state.tick,
            });
        }
        input.seq = self.state.next_input_seq;
        self.state.next_input_seq += 1;
        self.state.inputs.push(input.clone());
        self.state
            .queue
            .schedule_input(input.tick, u64::from(input.seq), Event::Input(input));
        Ok(())
    }

    /// Runs every event due at or before `target`, in `(tick, phase, seq)` order, then sets
    /// the clock to `target`. Returns what happened, for viewers and external actors.
    pub fn step_until(&mut self, target: Tick) -> Result<Vec<Emitted>, SimError> {
        if target < self.state.tick {
            return Err(SimError::StepBackwards {
                current: self.state.tick,
                target,
            });
        }
        let mut out = std::mem::take(&mut self.state.pending);
        while let Some(scheduled) = self.state.queue.pop_due(target) {
            self.state.tick = scheduled.tick;
            let start = out.len();
            self.handle(scheduled.event, &mut out);
            self.keep_record(&out[start..]);
        }
        self.state.tick = target;
        Ok(out)
    }

    fn handle(&mut self, event: Event, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        match event {
            Event::DayRollover => {
                self.state.day_rollovers += 1;
                self.state.record.forget(tick);
                self.state.terrain.new_day(&self.world);
                self.state
                    .queue
                    .schedule(tick.plus(TICKS_PER_DAY), Event::DayRollover);
                out.push(Emitted::DayStarted {
                    tick,
                    calendar: tick.calendar(),
                });
                self.plan_everyone(out);
            }
            Event::Input(input) => self.handle_input(&input, out),
            Event::WakeUp { who, generation } => {
                if self.state.villagers[who.index()].plan_generation != generation {
                    return;
                }
                let Some(cmds) = self.state.villagers[who.index()].plan.remove(&tick) else {
                    return;
                };
                for (cause, cmd) in cmds {
                    self.cause = cause;
                    self.run_command(who, cmd, out);
                }
                self.cause = Cause::Sim;
            }
            Event::Arrive { who, generation } => {
                if self.state.villagers[who.index()].generation == generation {
                    self.step_movement(who, out);
                }
            }
            Event::Waited { who, generation } => {
                let v = &mut self.state.villagers[who.index()];
                if v.generation != generation {
                    return;
                }
                // A conversation defers the wait until it ends. One ending this very tick
                // has not been wrapped up yet; re-queueing puts this wait after it.
                if let Some(end) = v.talk_until
                    && end >= tick
                {
                    v.wait_until = Some(end);
                    self.state
                        .queue
                        .schedule(end, Event::Waited { who, generation });
                    return;
                }
                v.wait_until = None;
                if v.interlude
                    .as_ref()
                    .is_some_and(|i| i.activity == v.activity)
                {
                    self.errand_done(who, out);
                    return;
                }
                self.react(who, Trigger::Waited, out);
            }
            Event::TalkEnd { who, other, topic } => {
                let v = &mut self.state.villagers[who.index()];
                if v.talk_until != Some(tick) || v.activity != "talk" {
                    return;
                }
                v.talk_until = None;
                let resume = v.resume.take().unwrap_or_else(|| "idle".into());
                self.set_activity(who, &resume, out);
                self.react(who, Trigger::TalkEnded { other, topic }, out);
            }
            Event::ObjectFree { .. } => {}
            Event::Woke { who } => self.woke(who, out),
            Event::Beat { convo } => self.beat(convo, out),
            Event::Body { who } => self.body_hour(who, out),
            Event::Look { who } => {
                self.state
                    .queue
                    .schedule(tick.plus(LOOK_TICKS), Event::Look { who });
                if self.state.villagers[who.index()].activity != "sleep" {
                    let since = tick.minus(LOOK_TICKS);
                    self.react(who, Trigger::Look { since }, out);
                }
            }
            Event::Tripped { who } => {
                let v = &self.state.villagers[who.index()];
                out.push(Emitted::Mishap {
                    tick,
                    who,
                    what: Mishap::Tripped,
                    place: v.place,
                    x: v.pos.x,
                    y: v.pos.y,
                });
                self.react(
                    who,
                    Trigger::Mishap {
                        what: Mishap::Tripped,
                    },
                    out,
                );
            }
        }
    }

    /// Needs do not ask while asleep, so anything that got pressing overnight asks on
    /// waking rather than waiting for the next hourly tick.
    fn woke(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        if self.state.villagers[who.index()].activity == "sleep" {
            return;
        }
        for need in [Need::Bladder, Need::Thirst, Need::Hunger] {
            let level = self.state.villagers[who.index()].needs.get(need);
            if level >= crate::needs::PRESSING {
                out.push(Emitted::Need {
                    tick,
                    who,
                    need,
                    level,
                });
                self.react(who, Trigger::Need { need, level }, out);
            }
        }
    }

    /// An hour of a body: needs rise, sleep repays, thresholds crossed ask the character.
    fn body_hour(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        self.state
            .queue
            .schedule(tick.plus(TICKS_PER_HOUR), Event::Body { who });
        let asleep = self.state.villagers[who.index()].activity == "sleep";
        let crossed = self.state.villagers[who.index()]
            .needs
            .hour(asleep, &mut self.state.rng.villagers[who.index()].body);
        for need in crossed {
            let level = self.state.villagers[who.index()].needs.get(need);
            out.push(Emitted::Need {
                tick,
                who,
                need,
                level,
            });
            if !asleep {
                self.react(who, Trigger::Need { need, level }, out);
            }
        }
    }

    fn handle_input(&mut self, input: &Input, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let summary = match (&input.source, &input.kind) {
            (_, InputKind::Noop) => "noop".to_string(),
            (_, InputKind::Marker(text)) => {
                self.state.markers.push((tick, text.clone()));
                format!("marker {text:?}")
            }
            (Source::System, InputKind::Teleport { who, place, x, y }) => {
                if who.index() >= self.state.villagers.len()
                    || place.index() >= self.world.places.len()
                    || !self.world.passable(place.index(), *x, *y)
                {
                    out.push(Emitted::InputRejected {
                        tick,
                        reason: format!(
                            "teleport target {place:?} ({x}, {y}) is not a passable tile"
                        ),
                    });
                    return;
                }
                self.interrupt(*who);
                let v = &mut self.state.villagers[who.index()];
                v.cause = Cause::Input { seq: input.seq };
                v.place = *place;
                v.pos = Pos { x: *x, y: *y };
                let (vp, vx, vy) = (v.place, v.pos.x, v.pos.y);
                out.push(Emitted::Moved {
                    tick,
                    who: *who,
                    place: vp,
                    x: vx,
                    y: vy,
                });
                format!(
                    "teleport {} to {} ({x}, {y})",
                    self.state.villagers[who.index()].name,
                    self.world.places[place.index()].id
                )
            }
            (source, InputKind::Teleport { .. }) => {
                out.push(Emitted::InputRejected {
                    tick,
                    reason: format!("{source:?} may not teleport"),
                });
                return;
            }
        };
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
            let (planned, items, dropped) = {
                let ctx = Ctx {
                    tick: now,
                    calendar: now.calendar(),
                    content: &self.content,
                    world: &self.world,
                    nav: &self.nav,
                    villagers: &self.state.villagers,
                    terrain: &self.state.terrain,
                    stock: &self.state.stock,
                    record: &self.state.record,
                };
                let baseline = crate::brain::baseline_plan(who, &ctx);
                let mut planned = self.state.characters.plan_day(
                    who,
                    &ctx,
                    &mut self.state.rng.villagers[who.index()].mind,
                );
                planned.sort_by_key(|p| p.at);
                // An item is the schedule's when the content schedule has the same command at
                // the same time; anything else is the character's own doing.
                let planned: Vec<(Cause, Planned)> = planned
                    .into_iter()
                    .map(|p| {
                        let cause = if baseline.contains(&p) {
                            Cause::Schedule
                        } else {
                            Cause::Plan
                        };
                        (cause, p)
                    })
                    .collect();
                let view = |p: &Planned, schedule: bool| PlanItem {
                    at: p.at,
                    text: p.command.summary(&ctx),
                    activity: match &p.command {
                        Command::GoTo { activity, .. }
                        | Command::Wait { activity, .. }
                        | Command::Interlude { activity, .. } => Some(activity.clone()),
                        _ => None,
                    },
                    schedule,
                };
                let items: Vec<PlanItem> = planned
                    .iter()
                    .map(|(c, p)| view(p, *c == Cause::Schedule))
                    .collect();
                let dropped: Vec<PlanItem> = baseline
                    .iter()
                    .filter(|b| !planned.iter().any(|(_, p)| p == *b))
                    .map(|b| view(b, true))
                    .collect();
                (planned, items, dropped)
            };
            out.push(Emitted::Planned {
                tick: now,
                who,
                items,
                dropped,
            });
            let v = &mut self.state.villagers[i];
            v.plan.clear();
            v.plan_generation += 1;
            let generation = v.plan_generation;
            for (cause, p) in planned {
                let at = day_start.plus(p.at.seconds());
                if at < now {
                    continue;
                }
                let slot = v.plan.entry(at).or_default();
                if slot.is_empty() {
                    self.state
                        .queue
                        .schedule(at, Event::WakeUp { who, generation });
                }
                slot.push((cause, p.command));
            }
        }
    }

    fn react(&mut self, who: VillagerId, trigger: Trigger, out: &mut Vec<Emitted>) {
        if self.depth >= MAX_REACT_DEPTH {
            out.push(Emitted::Decided {
                tick: self.state.tick,
                who,
                text: format!(
                    "reaction depth limit reached on {}",
                    trigger.label(&self.ctx())
                ),
            });
            return;
        }
        self.depth += 1;
        let (commands, text, label) = {
            let ctx = Ctx {
                tick: self.state.tick,
                calendar: self.state.tick.calendar(),
                content: &self.content,
                world: &self.world,
                nav: &self.nav,
                villagers: &self.state.villagers,
                terrain: &self.state.terrain,
                stock: &self.state.stock,
                record: &self.state.record,
            };
            let label = trigger.label(&ctx);
            let commands = self.state.characters.react(
                who,
                trigger,
                &ctx,
                &mut self.state.rng.villagers[who.index()].mind,
            );
            let text = (!commands.is_empty()).then(|| {
                let items: Vec<String> = commands.iter().map(|c| c.summary(&ctx)).collect();
                format!("{label} -> {}", items.join("; "))
            });
            (commands, text, label)
        };
        if let Some(text) = text {
            out.push(Emitted::Decided {
                tick: self.state.tick,
                who,
                text,
            });
        }
        let outer = std::mem::replace(&mut self.cause, Cause::Reaction { trigger: label });
        for cmd in commands {
            self.run_command(who, cmd, out);
        }
        self.cause = outer;
        self.depth -= 1;
    }

    /// Something new replaces an errand in progress: there is nothing to go back to.
    fn drop_errand(&mut self, who: VillagerId) {
        let v = &mut self.state.villagers[who.index()];
        v.interlude = None;
        v.resume_wait = None;
        v.resume_cause = None;
    }

    /// Drops whatever `who` was doing: movement, waits and conversations become stale.
    fn interrupt(&mut self, who: VillagerId) {
        let v = &mut self.state.villagers[who.index()];
        v.generation += 1;
        v.moving = None;
        v.wait_until = None;
        v.talk_until = None;
        v.resume = None;
        v.approach = None;
    }

    /// Commands that change nothing outside the villager: a reason for what follows, and a
    /// thought (which others may overhear, if said aloud).
    fn inner_command(&mut self, who: VillagerId, cmd: Command, out: &mut Vec<Emitted>) {
        match cmd {
            Command::Because {
                from,
                topic,
                heard_at,
            } => {
                self.cause = Cause::Knew {
                    from,
                    topic,
                    heard_at,
                };
            }
            Command::Think(thought) => {
                let aloud = thought.aloud.then(|| thought.clone());
                out.push(Emitted::Thought {
                    tick: self.state.tick,
                    who,
                    thought,
                });
                if let Some(thought) = aloud {
                    self.voice(who, &thought, out);
                }
            }
            _ => {}
        }
    }

    fn run_command(&mut self, who: VillagerId, cmd: Command, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        match cmd {
            Command::Because { .. } | Command::Think(_) => self.inner_command(who, cmd, out),
            Command::Idle => {
                self.interrupt(who);
                self.drop_errand(who);
                self.state.villagers[who.index()].cause = self.cause.clone();
                self.set_activity(who, "idle", out);
            }
            Command::Wait { ticks, activity } => {
                self.interrupt(who);
                self.drop_errand(who);
                let v = &mut self.state.villagers[who.index()];
                v.cause = self.cause.clone();
                let generation = v.generation;
                v.wait_until = Some(tick.plus(ticks));
                self.set_activity(who, &activity, out);
                self.state
                    .queue
                    .schedule(tick.plus(ticks), Event::Waited { who, generation });
            }
            Command::GoTo { target, activity } => {
                self.drop_errand(who);
                self.state.villagers[who.index()].cause = self.cause.clone();
                self.go_to(who, &target, activity, out);
            }
            Command::Note(text) => out.push(Emitted::Noted {
                tick,
                who,
                text,
                cause: self.cause.clone(),
            }),
            Command::Pay {
                from,
                to,
                amount,
                what,
            } => self.pay(who, from, to, amount, what, out),
            Command::Stock {
                object,
                good,
                delta,
            } => self.stock(who, object, &good, delta, out),
            Command::Interlude {
                target,
                activity,
                minutes,
            } => self.interlude(who, &target, activity, minutes, out),
            Command::Clear { radius } => {
                let v = &self.state.villagers[who.index()];
                let (place, at) = (v.place, v.pos);
                let tiles = self.state.terrain.clear(&self.world, place, at, radius);
                out.push(Emitted::Cleared {
                    tick,
                    who,
                    place,
                    x: at.x,
                    y: at.y,
                    tiles,
                    cause: self.cause.clone(),
                });
            }
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
                    cause: self.cause.clone(),
                });
                self.react(
                    with,
                    Trigger::TalkedTo {
                        other: who,
                        topic: Topic::new("greeting"),
                    },
                    out,
                );
            }
            Command::Talk {
                with,
                frame,
                topic,
                minutes,
            } => {
                self.talk(who, with, frame, topic, minutes, true, out);
            }
        }
    }

    /// Starts a conversation. Neither party's pending plan item or wait is cancelled: a
    /// wait is deferred until the talk ends, and a plan item that fires mid-talk simply ends
    /// this party's side of it. Someone further than `TALK_REACH` walks over first when
    /// `approach` is set (and the talk starts on arrival); otherwise the talk does not
    /// happen. Returns whether it started.
    #[allow(clippy::too_many_arguments)]
    fn talk(
        &mut self,
        who: VillagerId,
        with: VillagerId,
        frame: Frame,
        topic: Topic,
        minutes: u32,
        approach: bool,
        out: &mut Vec<Emitted>,
    ) -> bool {
        let tick = self.state.tick;
        if with == who || with.index() >= self.state.villagers.len() {
            return false;
        }
        let reason = {
            let a = &self.state.villagers[who.index()];
            let b = &self.state.villagers[with.index()];
            if a.place != b.place {
                Some("not in the same place")
            } else if a.moving.is_some() || b.moving.is_some() {
                Some("one of them is walking")
            } else if a.activity == "talk" || b.activity == "talk" {
                Some("one of them is already talking")
            } else if b.activity == "sleep" {
                Some("the other is asleep")
            } else {
                None
            }
        };
        if let Some(reason) = reason {
            out.push(Emitted::Decided {
                tick,
                who,
                text: format!(
                    "wanted to talk with {} but {reason}",
                    self.state.villagers[with.index()].name
                ),
            });
            return false;
        }
        if !self.within_reach(who, with) {
            if approach && self.walk_over(who, with, frame, &topic, minutes, out) {
                return false;
            }
            out.push(Emitted::Decided {
                tick,
                who,
                text: format!(
                    "wanted to talk with {} but they were too far away",
                    self.state.villagers[with.index()].name
                ),
            });
            return false;
        }
        let end = tick.plus(minutes.max(2) * TICKS_PER_MINUTE);
        self.open_conversation(who, with, frame, &topic, end, out);
        let started_by = Cause::Reaction {
            trigger: format!("{} started talking", self.state.villagers[who.index()].name),
        };
        for (me, other) in [(who, with), (with, who)] {
            let v = &mut self.state.villagers[me.index()];
            v.resume = Some(v.activity.clone());
            v.talk_until = Some(end);
            // The talk has its own cause; the villager's intention, and so what they go
            // back to afterwards, keeps the cause it had.
            let cause = if me == who {
                self.cause.clone()
            } else {
                started_by.clone()
            };
            self.set_activity_because(me, "talk", cause, out);
            self.state.queue.schedule(
                end,
                Event::TalkEnd {
                    who: me,
                    other,
                    topic: topic.clone(),
                },
            );
        }
        self.react(with, Trigger::TalkedTo { other: who, topic }, out);
        true
    }

    /// Close enough to talk: within `TALK_REACH` tiles in the same place.
    fn within_reach(&self, a: VillagerId, b: VillagerId) -> bool {
        let (a, b) = (
            &self.state.villagers[a.index()],
            &self.state.villagers[b.index()],
        );
        a.place == b.place && a.pos.x.abs_diff(b.pos.x).max(a.pos.y.abs_diff(b.pos.y)) <= TALK_REACH
    }

    /// Sets `who` walking to a free tile beside `with`, to start the talk there. False when
    /// there is no such tile.
    fn walk_over(
        &mut self,
        who: VillagerId,
        with: VillagerId,
        frame: Frame,
        topic: &Topic,
        minutes: u32,
        out: &mut Vec<Emitted>,
    ) -> bool {
        let (place, from, there) = {
            let a = &self.state.villagers[who.index()];
            let b = &self.state.villagers[with.index()];
            (b.place, a.pos, b.pos)
        };
        let taken = |pos: Pos| {
            self.state
                .villagers
                .iter()
                .any(|v| v.place == place && v.pos == pos && v.moving.is_none())
        };
        // The free neighbour nearest the walker, in a fixed order for ties.
        let spot = [
            (0i32, -1i32),
            (-1, 0),
            (1, 0),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ]
        .iter()
        .filter_map(|&(dx, dy)| {
            let x = u16::try_from(i32::from(there.x) + dx).ok()?;
            let y = u16::try_from(i32::from(there.y) + dy).ok()?;
            let pos = Pos { x, y };
            (self.step_cost(place.index(), pos).is_some() && !taken(pos)).then_some(pos)
        })
        .min_by_key(|p| p.x.abs_diff(from.x).max(p.y.abs_diff(from.y)));
        let Some(spot) = spot else {
            return false;
        };
        let (activity, wait_left) = {
            let v = &self.state.villagers[who.index()];
            let now = self.state.tick;
            (
                v.activity.clone(),
                v.wait_until.map(|t| t.0.saturating_sub(now.0)),
            )
        };
        self.go_to(who, &Target::Tile { place, pos: spot }, activity, out);
        let v = &mut self.state.villagers[who.index()];
        if v.moving.is_none() {
            return false;
        }
        v.approach = Some(Approach {
            with,
            frame,
            topic: topic.clone(),
            minutes,
            cause: self.cause.clone(),
            wait_left,
        });
        true
    }

    /// Whether a proximity trigger of `kind` may fire for this pair now; records it if so.
    fn pair_ready(&mut self, kind: u8, a: VillagerId, b: VillagerId) -> bool {
        let (lo, hi) = if a.0 <= b.0 { (a.0, b.0) } else { (b.0, a.0) };
        let now = self.state.tick;
        match self.state.met.get(&(kind, lo, hi)) {
            Some(&last) if now.0.saturating_sub(last.0) < ENCOUNTER_COOLDOWN => false,
            _ => {
                self.state.met.insert((kind, lo, hi), now);
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
    /// An encounter also uses up the pair's passing cooldown, so arriving next to someone
    /// does not fire both.
    fn encounters(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let others: Vec<VillagerId> = {
            let me = &self.state.villagers[who.index()];
            if unavailable(me) {
                return;
            }
            self.state
                .villagers
                .iter()
                .enumerate()
                .filter(|(i, v)| {
                    *i != who.index()
                        && v.moving.is_none()
                        && !unavailable(v)
                        && self.together(me, v)
                })
                .map(|(i, _)| VillagerId(u16::try_from(i).expect("few villagers")))
                .collect()
        };
        for other in others {
            if self.pair_ready(0, who, other) {
                self.pair_ready(1, who, other);
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

    /// Starts an activity under the villager's current intention.
    fn set_activity(&mut self, who: VillagerId, activity: &str, out: &mut Vec<Emitted>) {
        let cause = self.state.villagers[who.index()].cause.clone();
        self.set_activity_because(who, activity, cause, out);
    }

    fn set_activity_because(
        &mut self,
        who: VillagerId,
        activity: &str,
        cause: Cause,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let v = &mut self.state.villagers[who.index()];
        if v.activity != activity {
            let woke = v.activity == "sleep";
            v.activity = activity.to_string();
            v.since = tick;
            v.needs.on_activity(activity);
            if woke {
                self.state.queue.schedule(tick, Event::Woke { who });
            }
        }
        out.push(Emitted::Activity {
            tick,
            who,
            activity: activity.to_string(),
            cause,
        });
    }

    fn resolve_target(&self, who: VillagerId, target: &Target) -> Option<Spot> {
        match target {
            Target::Place(p) => {
                let place = self.world.places.get(p.index())?;
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
                let obj = self.world.objects.get(o.index())?;
                if obj.walkable {
                    return Some(Spot {
                        place: obj.place,
                        pos: obj.at,
                    });
                }
                // Nearest access tile to where the villager stands when in the same place,
                // else the first passable neighbour in N, E, S, W order.
                let v = &self.state.villagers[who.index()];
                let candidates: Vec<Pos> = content::validate::neighbours(obj.at.x, obj.at.y)
                    .into_iter()
                    .filter(|&(x, y)| self.world.passable(obj.place, x, y))
                    .map(|(x, y)| Pos { x, y })
                    .collect();
                if v.place.index() == obj.place {
                    let base = nav::base_cost(&self.world);
                    let dist = nav::cost_distances(
                        &self.world,
                        Spot {
                            place: obj.place,
                            pos: v.pos,
                        },
                        &base,
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

    #[must_use]
    pub fn terrain(&self) -> &Terrain {
        &self.state.terrain
    }

    /// What a tile looks like now, or None for a cell that neither wears nor grows.
    #[must_use]
    pub fn surface(&self, place: usize, pos: Pos) -> Option<Surface> {
        self.state.terrain.surface(&self.world, place, pos)
    }

    /// Step cost for routing: indoor cells and doors by their base cost, outdoor tiles by
    /// their surface.
    fn step_cost(&self, place: usize, pos: Pos) -> Option<u32> {
        if !self.world.passable(place, pos.x, pos.y) {
            return None;
        }
        Some(self.surface(place, pos).map_or_else(
            || self.world.places[place].cell(pos.x, pos.y).base_cost(),
            Surface::cost,
        ))
    }

    fn pay(
        &mut self,
        who: VillagerId,
        from: Party,
        to: Party,
        amount: u32,
        what: String,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let available = match from {
            Party::Villager(v) => self.state.villagers[v.index()].coins,
            Party::Outside => u32::MAX,
        };
        let amount = amount.min(available);
        if let Party::Villager(v) = from {
            self.state.villagers[v.index()].coins -= amount;
        }
        if let Party::Villager(v) = to {
            let c = &mut self.state.villagers[v.index()].coins;
            *c = c.saturating_add(amount);
        }
        let party = |p: Party| match p {
            Party::Villager(v) => Some(v),
            Party::Outside => None,
        };
        let payer_left = match from {
            Party::Villager(v) => self.state.villagers[v.index()].coins,
            Party::Outside => 0,
        };
        out.push(Emitted::Paid {
            tick,
            who,
            from: party(from),
            to: party(to),
            amount,
            what,
            payer_left,
            cause: self.cause.clone(),
        });
    }

    fn stock(
        &mut self,
        who: VillagerId,
        object: ObjectId,
        good: &str,
        delta: i32,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        if object.index() >= self.world.objects.len() {
            return;
        }
        let goods = self.state.stock.entry(object.0).or_default();
        let count = goods.entry(good.to_string()).or_insert(0);
        let before = i32::from(*count);
        let after = u16::try_from((before + delta).max(0)).unwrap_or(u16::MAX);
        *count = after;
        let delta = i32::from(after) - before;
        if after == 0 {
            goods.remove(good);
        }
        out.push(Emitted::Stocked {
            tick,
            who,
            object,
            good: good.to_string(),
            delta,
            now: after,
            cause: self.cause.clone(),
        });
    }

    /// An errand that comes back: remember where to return to, then go.
    fn interlude(
        &mut self,
        who: VillagerId,
        target: &Target,
        activity: String,
        minutes: u32,
        out: &mut Vec<Emitted>,
    ) {
        let v = &mut self.state.villagers[who.index()];
        if let Some(i) = &mut v.interlude {
            // Already on an errand: this one waits its turn, unless it is the same thing.
            let same = i.activity == activity || i.queued.iter().any(|e| e.activity == activity);
            if !same {
                i.queued.push(Errand {
                    target: target.clone(),
                    activity,
                    minutes,
                    cause: self.cause.clone(),
                });
            }
            return;
        }
        // Where to come back to: the current spot and what was going on, or
        // the destination and purpose if mid-journey.
        let (back_place, back_pos, back_activity) = match &v.moving {
            Some(m) => match m.legs.last() {
                Some(l) => (l.place, l.to, m.then.clone()),
                None => (
                    v.place,
                    m.path.last().copied().unwrap_or(v.pos),
                    m.then.clone(),
                ),
            },
            // Called away mid-conversation: the talk ends (the conversation sees they have
            // stopped) and they come back to what they were doing before it.
            None if v.activity == "talk" => {
                v.talk_until = None;
                let before = v.resume.take().unwrap_or_else(|| "idle".into());
                (v.place, v.pos, before)
            }
            None => (v.place, v.pos, v.activity.clone()),
        };
        let back_wait = v
            .wait_until
            .filter(|end| end.0 > self.state.tick.0)
            .map(|end| end.0 - self.state.tick.0);
        v.interlude = Some(Interlude {
            activity: activity.clone(),
            minutes,
            back_place,
            back_pos,
            back_activity,
            back_wait,
            back_cause: v.cause.clone(),
            queued: Vec::new(),
        });
        self.state.villagers[who.index()].cause = self.cause.clone();
        self.go_to(who, target, activity, out);
    }

    /// An errand's time is up: on to the next queued errand, or back where they were.
    fn errand_done(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let v = &mut self.state.villagers[who.index()];
        let Some(i) = v.interlude.as_mut() else {
            return;
        };
        if !i.queued.is_empty() {
            let next = i.queued.remove(0);
            i.activity.clone_from(&next.activity);
            i.minutes = next.minutes;
            v.cause = next.cause;
            self.go_to(who, &next.target, next.activity, out);
            return;
        }
        let Some(back) = v.interlude.take() else {
            return;
        };
        let target = Target::Tile {
            place: back.back_place,
            pos: back.back_pos,
        };
        v.resume_wait = back.back_wait;
        v.resume_cause = Some(back.back_cause);
        v.cause = Cause::Return;
        self.go_to_with(who, &target, back.back_activity, true, out);
    }

    fn stuck(&mut self, who: VillagerId, reason: String, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        self.interrupt(who);
        self.set_activity(who, "idle", out);
        let cause = self.state.villagers[who.index()].cause.clone();
        out.push(Emitted::Stuck {
            tick,
            who,
            reason,
            cause,
        });
        self.react(who, Trigger::Stuck, out);
    }

    fn go_to(
        &mut self,
        who: VillagerId,
        target: &Target,
        activity: String,
        out: &mut Vec<Emitted>,
    ) {
        self.go_to_with(who, target, activity, false, out);
    }

    /// `resuming` marks the walk back from an errand; arrival then fires `Resumed`.
    fn go_to_with(
        &mut self,
        who: VillagerId,
        target: &Target,
        activity: String,
        resuming: bool,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let Some(dest) = self.resolve_target(who, target) else {
            self.stuck(who, "target has no reachable tile".into(), out);
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
            self.interrupt(who);
            self.arrive(who, &activity, resuming, out);
            return;
        }
        let cost = |place: usize, pos: Pos| self.step_cost(place, pos);
        let route = self.nav.route_filtered(
            &self.world,
            here,
            dest,
            |d| self.door_allowed(who, d),
            &cost,
        );
        let Some(route) = route else {
            let reason = format!(
                "no route to {} ({}, {})",
                self.world.places[dest.place].id, dest.pos.x, dest.pos.y
            );
            self.stuck(who, reason, out);
            return;
        };
        let eta = tick.plus(route.ticks());
        let mut legs: Vec<PlannedLeg> = route
            .legs
            .iter()
            .map(|l| PlannedLeg {
                place: PlaceId(u16::try_from(l.place).expect("few places")),
                to: l.to,
                crossing: l.door.map(|d| Crossing {
                    door: u16::try_from(d).expect("few doors"),
                    reverse: l.door_reverse,
                }),
            })
            .collect();
        let first = legs.remove(0);
        let Some(path) =
            nav::grid_path_cost(&self.world, first.place.index(), here.pos, first.to, &cost)
        else {
            self.stuck(who, "no path inside the place".into(), out);
            return;
        };
        self.interrupt(who);
        let v = &mut self.state.villagers[who.index()];
        let generation = v.generation;
        v.moving = Some(Movement {
            legs,
            path: path.into_iter().skip(1).collect(),
            door_after: first.crossing,
            then: activity.clone(),
            resuming,
        });
        self.set_activity(who, "walking", out);
        let cause = self.state.villagers[who.index()].cause.clone();
        out.push(Emitted::Departed {
            tick,
            who,
            activity,
            eta,
            cause,
        });
        self.state.queue.schedule(
            tick.plus(WALK_TICKS_PER_TILE),
            Event::Arrive { who, generation },
        );
    }

    /// Reaching a destination: activity, the brain, encounters.
    fn arrive(&mut self, who: VillagerId, activity: &str, resuming: bool, out: &mut Vec<Emitted>) {
        if resuming && let Some(cause) = self.state.villagers[who.index()].resume_cause.take() {
            self.state.villagers[who.index()].cause = cause;
        }
        self.set_activity(who, activity, out);
        if resuming {
            // Re-arm the wait the errand interrupted, with the time it had left.
            if let Some(left) = self.state.villagers[who.index()].resume_wait.take() {
                let tick = self.state.tick;
                let v = &mut self.state.villagers[who.index()];
                let generation = v.generation;
                v.wait_until = Some(tick.plus(left));
                self.state
                    .queue
                    .schedule(tick.plus(left), Event::Waited { who, generation });
            }
            self.react(who, Trigger::Resumed, out);
            self.encounters(who, out);
            return;
        }
        // Walked over to talk to someone: talk now, if they are still there and free.
        if let Some(a) = self.state.villagers[who.index()].approach.take() {
            // The wait the walk interrupted goes on; a talk that starts defers it.
            if let Some(left) = a.wait_left {
                let tick = self.state.tick;
                let v = &mut self.state.villagers[who.index()];
                let generation = v.generation;
                v.wait_until = Some(tick.plus(left));
                self.state
                    .queue
                    .schedule(tick.plus(left), Event::Waited { who, generation });
            }
            self.cause = a.cause;
            if self.talk(who, a.with, a.frame, a.topic, a.minutes, false, out) {
                return;
            }
        }
        let errand = self.state.villagers[who.index()]
            .interlude
            .as_ref()
            .filter(|i| i.activity == activity)
            .map(|i| i.minutes);
        if let Some(minutes) = errand {
            let tick = self.state.tick;
            let v = &mut self.state.villagers[who.index()];
            let generation = v.generation;
            v.wait_until = Some(tick.plus(minutes * TICKS_PER_MINUTE));
            self.state.queue.schedule(
                tick.plus(minutes * TICKS_PER_MINUTE),
                Event::Waited { who, generation },
            );
        }
        self.react(who, Trigger::Arrived, out);
        self.encounters(who, out);
    }

    /// Long grass and weeds catch feet now and then. The fall itself is handled as its own
    /// event so the walker's movement is settled before they react.
    fn maybe_trip(&mut self, who: VillagerId, place: PlaceId, pos: Pos) {
        let per_mille = match self.state.terrain.surface(&self.world, place.index(), pos) {
            Some(Surface::Brush | Surface::OvergrownPath) => TRIP_OVERGROWN,
            Some(Surface::TallGrass | Surface::WeedyPath) => TRIP_LONG_GRASS,
            _ => return,
        };
        let body = &mut self.state.rng.villagers[who.index()].body;
        if body.random_range(0..1000u32) < per_mille {
            self.state
                .queue
                .schedule(self.state.tick, Event::Tripped { who });
        }
    }

    /// Keeps what is worth knowing of events just emitted, with who was there to see.
    fn keep_record(&mut self, new: &[Emitted]) {
        if new.iter().all(|e| recall::keep(e).is_none()) {
            return;
        }
        let observers: Vec<Observer> = self
            .state
            .villagers
            .iter()
            .enumerate()
            .map(|(i, v)| Observer {
                who: VillagerId(u16::try_from(i).expect("few villagers")),
                place: v.place,
                pos: v.pos,
                awake: v.activity != "sleep",
            })
            .collect();
        for e in new {
            self.state.record.note(e, &observers);
        }
    }

    /// One movement step: advance a tile, cross a door at a leg's end, or arrive.
    fn step_movement(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let Some(mut m) = self.state.villagers[who.index()].moving.take() else {
            return;
        };
        let mut stepped = false;
        if !m.path.is_empty() {
            let next = m.path.remove(0);
            let v = &mut self.state.villagers[who.index()];
            v.pos = next;
            let place = v.place;
            out.push(Emitted::Moved {
                tick,
                who,
                place,
                x: next.x,
                y: next.y,
            });
            self.maybe_trip(who, place, next);
            self.state.terrain.tread(&self.world, place, next);
            stepped = true;
        }
        if m.path.is_empty() {
            // Leg finished. Cross a door if one is pending, then start the next leg.
            if let Some(Crossing { door, reverse }) = m.door_after.take() {
                if !self.door_allowed(who, usize::from(door)) {
                    let name = self.world.doors[usize::from(door)].id.clone();
                    self.stuck(who, format!("door {name} is closed"), out);
                    return;
                }
                let d = &self.world.doors[usize::from(door)];
                let (to_place, to) = if reverse {
                    (d.from_place, d.from)
                } else {
                    (d.to_place, d.to)
                };
                let v = &mut self.state.villagers[who.index()];
                v.place = PlaceId(u16::try_from(to_place).expect("few places"));
                v.pos = to;
                out.push(Emitted::Moved {
                    tick,
                    who,
                    place: v.place,
                    x: to.x,
                    y: to.y,
                });
            }
            if m.legs.is_empty() {
                let activity = m.then;
                self.state.villagers[who.index()].generation += 1;
                self.arrive(who, &activity, m.resuming, out);
                return;
            }
            let next = m.legs.remove(0);
            let from = self.state.villagers[who.index()].pos;
            let cost = |place: usize, pos: Pos| self.step_cost(place, pos);
            let Some(path) =
                nav::grid_path_cost(&self.world, next.place.index(), from, next.to, &cost)
            else {
                self.stuck(who, "no path inside the place".into(), out);
                return;
            };
            m.path = path.into_iter().skip(1).collect();
            m.door_after = next.crossing;
        }
        let v = &mut self.state.villagers[who.index()];
        v.moving = Some(m);
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
                &s.terrain,
                &s.stock,
                &s.conversations,
                s.next_conversation,
                s.day_rollovers,
                &s.markers,
                &s.record,
                &s.characters,
            )),
            hash_of(&(&s.inputs, s.next_input_seq)),
        )
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<u8> {
        snapshot::encode(&self.state, C::LAYOUT_VERSION, C::BEHAVIOR)
    }

    /// Restores a snapshot against the same content it was made from.
    pub fn restore(bytes: &[u8], content: Content) -> Result<Sim<C>, SimError> {
        let state: SimState<C> = snapshot::decode(bytes, C::LAYOUT_VERSION, C::BEHAVIOR)?;
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
            depth: 0,
            cause: Cause::Sim,
        })
    }
}

/// Goods declared on objects in content, keyed like the live stock map.
fn starting_stock(content: &Content, world: &World) -> BTreeMap<u16, BTreeMap<String, u16>> {
    let specs = content
        .buildings
        .iter()
        .flat_map(|b| b.floors.iter().flat_map(|f| f.objects.iter()))
        .chain(content.zones.iter().flat_map(|z| z.objects.iter()));
    let mut stock = BTreeMap::new();
    for spec in specs.filter(|o| !o.stock.is_empty()) {
        if let Some(i) = world.objects.iter().position(|o| o.id == spec.id) {
            stock.insert(
                u16::try_from(i).expect("few objects"),
                spec.stock.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            );
        }
    }
    stock
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
