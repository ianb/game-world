//! The `Sim` handle: step, apply inputs, emit events, snapshot, hash.
use crate::event::Event;
use crate::hash::{HashParts, hash_of};
use crate::input::{Input, InputKind, Source};
use crate::protocol::Emitted;
use crate::queue::EventQueue;
use crate::rng::Streams;
use crate::snapshot::{self, SnapshotError};
use crate::time::{TICKS_PER_DAY, Tick};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Everything that determines the future. Serializable in full; hashed by subsystem.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimState {
    pub seed: u64,
    pub tick: Tick,
    pub rng: Streams,
    pub queue: EventQueue,
    /// Every input ever applied, in `(tick, seq)` order. The save game.
    pub inputs: Vec<Input>,
    /// Placeholder for villagers, places and objects. Grows with the content work.
    pub world: World,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub day_rollovers: u32,
    pub markers: Vec<(Tick, String)>,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SimError {
    #[error("input tick {input} is before the current tick {current}")]
    InputInPast { input: Tick, current: Tick },
    #[error("cannot step backwards from {current} to {target}")]
    StepBackwards { current: Tick, target: Tick },
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

#[derive(Debug)]
pub struct Sim {
    state: SimState,
}

impl Sim {
    /// A new world at midnight before day 1. The first day rollover is scheduled for the
    /// next midnight.
    #[must_use]
    pub fn new(seed: u64) -> Sim {
        let mut queue = EventQueue::default();
        queue.schedule(Tick(TICKS_PER_DAY), Event::DayRollover);
        Sim {
            state: SimState {
                seed,
                tick: Tick::ZERO,
                rng: Streams::from_seed(seed),
                queue,
                inputs: Vec::new(),
                world: World::default(),
            },
        }
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        self.state.tick
    }

    #[must_use]
    pub fn state(&self) -> &SimState {
        &self.state
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
        let mut out = Vec::new();
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
                self.state.world.day_rollovers += 1;
                self.state
                    .queue
                    .schedule(tick.plus(TICKS_PER_DAY), Event::DayRollover);
                out.push(Emitted::DayStarted {
                    tick,
                    calendar: tick.calendar(),
                });
            }
            Event::Input(input) => self.handle_input(input, out),
            Event::WakeUp { .. } | Event::Arrive { .. } | Event::ObjectFree { .. } => {
                // Villagers arrive with the content work.
            }
        }
    }

    fn handle_input(&mut self, input: Input, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let summary = match (&input.source, &input.kind) {
            (_, InputKind::Noop) => "noop".to_string(),
            (_, InputKind::Marker(text)) => {
                self.state.world.markers.push((tick, text.clone()));
                format!("marker {text:?}")
            }
            (Source::System, InputKind::Teleport { who, x, y }) => {
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

    #[must_use]
    pub fn hash(&self) -> HashParts {
        let s = &self.state;
        HashParts::combine(
            hash_of(&s.tick),
            hash_of(&s.rng),
            hash_of(&s.queue),
            hash_of(&s.world),
            hash_of(&s.inputs),
        )
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<u8> {
        snapshot::encode(&self.state)
    }

    pub fn restore(bytes: &[u8]) -> Result<Sim, SimError> {
        Ok(Sim {
            state: snapshot::decode(bytes)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Emitted;
    use proptest::prelude::*;

    fn marker(tick: u32, text: &str) -> Input {
        Input {
            tick: Tick(tick),
            source: Source::Player(0),
            kind: InputKind::Marker(text.into()),
        }
    }

    #[test]
    fn day_rollover_emits_once_per_day() {
        let mut sim = Sim::new(1);
        let events = sim.step_until(Tick(3 * TICKS_PER_DAY + 10)).unwrap();
        let days: Vec<u32> = events
            .iter()
            .filter_map(|e| match e {
                Emitted::DayStarted { calendar, .. } => Some(calendar.day_index),
                _ => None,
            })
            .collect();
        assert_eq!(days, vec![1, 2, 3]);
        assert_eq!(sim.tick(), Tick(3 * TICKS_PER_DAY + 10));
    }

    #[test]
    fn inputs_apply_at_their_tick_in_order() {
        let mut sim = Sim::new(1);
        sim.apply_input(marker(100, "b")).unwrap();
        sim.apply_input(marker(50, "a")).unwrap();
        sim.apply_input(marker(100, "c")).unwrap();
        let events = sim.step_until(Tick(100)).unwrap();
        let applied: Vec<String> = events
            .iter()
            .filter_map(|e| match e {
                Emitted::InputApplied { summary, .. } => Some(summary.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            applied,
            vec!["marker \"a\"", "marker \"b\"", "marker \"c\""]
        );
        assert_eq!(sim.state().world.markers.len(), 3);
    }

    #[test]
    fn past_inputs_and_backward_steps_are_rejected() {
        let mut sim = Sim::new(1);
        sim.step_until(Tick(10)).unwrap();
        assert_eq!(
            sim.apply_input(marker(5, "late")),
            Err(SimError::InputInPast {
                input: Tick(5),
                current: Tick(10)
            })
        );
        assert_eq!(
            sim.step_until(Tick(9)).unwrap_err(),
            SimError::StepBackwards {
                current: Tick(10),
                target: Tick(9)
            }
        );
    }

    #[test]
    fn players_may_not_teleport() {
        let mut sim = Sim::new(1);
        sim.apply_input(Input {
            tick: Tick(1),
            source: Source::Player(3),
            kind: InputKind::Teleport {
                who: crate::ids::VillagerId(0),
                x: 1,
                y: 1,
            },
        })
        .unwrap();
        let events = sim.step_until(Tick(1)).unwrap();
        assert!(matches!(events[0], Emitted::InputRejected { .. }));
        assert!(sim.state().inputs.is_empty());
    }

    #[test]
    fn snapshot_round_trip_preserves_hash_and_future() {
        let mut live = Sim::new(99);
        live.apply_input(marker(1000, "x")).unwrap();
        live.step_until(Tick(500)).unwrap();
        let bytes = live.snapshot();
        let mut restored = Sim::restore(&bytes).unwrap();
        assert_eq!(live.hash(), restored.hash());
        let a = live.step_until(Tick(2 * TICKS_PER_DAY)).unwrap();
        let b = restored.step_until(Tick(2 * TICKS_PER_DAY)).unwrap();
        assert_eq!(a, b);
        assert_eq!(live.hash(), restored.hash());
    }

    #[test]
    fn wrong_snapshot_version_is_refused() {
        let mut bytes = Sim::new(1).snapshot();
        bytes[0] = bytes[0].wrapping_add(1); // the leading varint is the snapshot version
        assert!(matches!(Sim::restore(&bytes), Err(SimError::Snapshot(_))));
    }

    #[test]
    fn hash_names_the_first_divergent_part() {
        let mut a = Sim::new(1);
        let b = Sim::new(1);
        a.apply_input(marker(0, "only in a")).unwrap();
        a.step_until(Tick(0)).unwrap();
        // Tick and rng are equal. The queue differs first: scheduling the input advanced
        // `next_seq`, which is part of the queue's serialized form on purpose.
        assert_eq!(a.hash().first_difference(&b.hash()), Some("queue"));
        assert_eq!(a.hash().first_difference(&a.hash()), None);
    }

    #[test]
    fn golden_hash_after_a_week() {
        let mut sim = Sim::new(2026);
        sim.apply_input(marker(3 * TICKS_PER_DAY, "midweek"))
            .unwrap();
        sim.step_until(Tick(7 * TICKS_PER_DAY)).unwrap();
        insta::assert_snapshot!(sim.hash().hex());
    }

    proptest! {
        #[test]
        fn same_seed_and_inputs_give_same_hash(
            seed in any::<u64>(),
            ticks in prop::collection::vec(0u32..(3 * TICKS_PER_DAY), 0..8),
            stop in 0u32..(4 * TICKS_PER_DAY),
        ) {
            let run = |seed: u64| {
                let mut sim = Sim::new(seed);
                for (i, &t) in ticks.iter().enumerate() {
                    sim.apply_input(marker(t, &i.to_string())).unwrap();
                }
                sim.step_until(Tick(stop)).unwrap();
                sim.hash()
            };
            prop_assert_eq!(run(seed), run(seed));
        }

        #[test]
        fn snapshot_at_t_then_k_equals_straight_run(
            seed in any::<u64>(),
            t in 0u32..(2 * TICKS_PER_DAY),
            k in 0u32..(2 * TICKS_PER_DAY),
        ) {
            let mut straight = Sim::new(seed);
            straight.step_until(Tick(t + k)).unwrap();
            let mut paused = Sim::new(seed);
            paused.step_until(Tick(t)).unwrap();
            let mut resumed = Sim::restore(&paused.snapshot()).unwrap();
            resumed.step_until(Tick(t + k)).unwrap();
            prop_assert_eq!(straight.hash(), resumed.hash());
        }
    }
}
