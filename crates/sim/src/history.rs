//! A sim with a memory: a snapshot at each day start, the furthest tick ever reached, and
//! summaries of every finished day. Rewinding restores the nearest earlier snapshot and
//! steps forward; the sim is deterministic, so the past replays exactly. Nothing here is
//! part of the sim's state or hash.
use crate::brain::CharacterSet;
use crate::input::Input;
use crate::protocol::{DaySummary, Emitted};
use crate::sim::{Sim, SimError};
use crate::summary::Summarizer;
use crate::time::{TICKS_PER_DAY, Tick};
use content::Content;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HistoryError {
    #[error(transparent)]
    Sim(#[from] SimError),
    #[error("viewing the past ({now:?}); go back to {frontier:?} before changing anything")]
    InThePast { now: Tick, frontier: Tick },
    #[error("{target:?} is beyond the furthest point reached ({frontier:?})")]
    Beyond { target: Tick, frontier: Tick },
    #[error("no snapshot at or before {0:?}")]
    NoSnapshot(Tick),
}

#[derive(Debug)]
pub struct History<C: CharacterSet> {
    sim: Sim<C>,
    content: Content,
    summarizer: Summarizer,
    /// Snapshots by the tick they were taken at: the start, then every day start.
    snapshots: BTreeMap<u32, Vec<u8>>,
    /// Every event with a tick at or before this has been summarised.
    frontier: Option<Tick>,
    /// Every input accepted, with its sequence number, to replay after a rewind.
    inputs: Vec<Input>,
}

impl<C: CharacterSet> History<C> {
    #[must_use]
    pub fn new(seed: u64, content: Content) -> History<C> {
        let sim = Sim::new(seed, content.clone());
        History::from_sim(sim, content)
    }

    /// Starts a history from a snapshot. Summaries begin at that point.
    pub fn restore(bytes: &[u8], content: Content) -> Result<History<C>, SimError> {
        let sim = Sim::restore(bytes, content.clone())?;
        Ok(History::from_sim(sim, content))
    }

    fn from_sim(sim: Sim<C>, content: Content) -> History<C> {
        let names = sim
            .state()
            .villagers
            .iter()
            .map(|v| v.name.clone())
            .collect();
        let objects = sim.world().objects.iter().map(|o| o.id.clone()).collect();
        let glosses = content
            .topics
            .iter()
            .map(|t| (t.id.clone(), t.gloss.clone()))
            .collect();
        let mut snapshots = BTreeMap::new();
        snapshots.insert(sim.tick().0, sim.snapshot());
        let inputs = sim.state().inputs.clone();
        History {
            sim,
            content,
            summarizer: Summarizer::new(names, objects, glosses),
            snapshots,
            frontier: None,
            inputs,
        }
    }

    #[must_use]
    pub fn sim(&self) -> &Sim<C> {
        &self.sim
    }

    /// The furthest tick ever reached.
    #[must_use]
    pub fn frontier(&self) -> Tick {
        self.frontier
            .map_or(self.sim.tick(), |f| f.max(self.sim.tick()))
    }

    /// True while looking at a moment before the furthest point reached.
    #[must_use]
    pub fn in_past(&self) -> bool {
        self.sim.tick() < self.frontier()
    }

    /// Summaries of every finished day, oldest first.
    #[must_use]
    pub fn summaries(&self) -> &[DaySummary] {
        self.summarizer.days()
    }

    /// Steps the sim, snapshotting at each day start and summarising events not seen before.
    pub fn step_until(&mut self, target: Tick) -> Result<Vec<Emitted>, SimError> {
        let mut all = Vec::new();
        loop {
            let now = self.sim.tick();
            let next_day = Tick((now.0 / TICKS_PER_DAY + 1) * TICKS_PER_DAY);
            let stop = next_day.min(target);
            let events = self.sim.step_until(stop)?;
            let seen = self.frontier;
            for e in &events {
                if seen.is_none_or(|f| e.tick() > f) {
                    self.summarizer.ingest(e);
                }
            }
            if seen.is_none_or(|f| stop > f) {
                self.summarizer.advance_to(stop);
                self.frontier = Some(stop);
            }
            if stop == next_day {
                let sim = &self.sim;
                self.snapshots
                    .entry(stop.0)
                    .or_insert_with(|| sim.snapshot());
            }
            all.extend(events);
            if stop >= target {
                return Ok(all);
            }
        }
    }

    /// Accepts an input. Refused while viewing the past, since it would fork history.
    pub fn apply_input(&mut self, input: Input) -> Result<(), HistoryError> {
        if self.in_past() {
            return Err(HistoryError::InThePast {
                now: self.sim.tick(),
                frontier: self.frontier(),
            });
        }
        self.sim.apply_input(input)?;
        if let Some(accepted) = self.sim.state().inputs.last() {
            self.inputs.push(accepted.clone());
        }
        Ok(())
    }

    /// Goes back (or forward, up to the frontier) to `to`: restores the nearest snapshot at
    /// or before it, replays inputs the snapshot did not have, and steps to `to`.
    pub fn rewind(&mut self, to: Tick) -> Result<(), HistoryError> {
        let frontier = self.frontier();
        if to > frontier {
            return Err(HistoryError::Beyond {
                target: to,
                frontier,
            });
        }
        let bytes = self
            .snapshots
            .range(..=to.0)
            .next_back()
            .map(|(_, b)| b.clone())
            .ok_or(HistoryError::NoSnapshot(to))?;
        let mut sim = Sim::<C>::restore(&bytes, self.content.clone())?;
        let had = sim.state().next_input_seq;
        for input in self.inputs.iter().filter(|i| i.seq >= had) {
            sim.apply_input(input.clone())?;
        }
        sim.step_until(to)?;
        self.sim = sim;
        Ok(())
    }

    /// Back to the furthest point reached.
    pub fn back_to_now(&mut self) -> Result<(), HistoryError> {
        let frontier = self.frontier();
        self.step_until(frontier)?;
        Ok(())
    }
}
