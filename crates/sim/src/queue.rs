//! The discrete-event heap. Total order by `(tick, phase, seq)`, so no two events ever tie,
//! and serialized as a sorted list so a restored queue hashes the same as a live one.
//!
//! Phase 0 is inputs, ordered by the input's own sequence number (assigned when the sim
//! accepts it, stored in the log), so a replayed log reproduces the same order however
//! early or late the client submitted each input. Phase 1 is internal events in scheduling
//! order. Inputs for a tick therefore always apply before that tick's internal events.
use crate::event::Event;
use crate::time::Tick;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Scheduled {
    pub tick: Tick,
    /// 0 = input, 1 = internal.
    pub phase: u8,
    pub seq: u64,
    pub event: Event,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(from = "QueueRepr", into = "QueueRepr")]
pub struct EventQueue {
    heap: BinaryHeap<Reverse<Scheduled>>,
    next_seq: u64,
}

/// Serialized form: sorted, so the bytes depend only on the logical content.
#[derive(Serialize, Deserialize)]
struct QueueRepr {
    next_seq: u64,
    items: Vec<Scheduled>,
}

impl From<QueueRepr> for EventQueue {
    fn from(r: QueueRepr) -> Self {
        EventQueue {
            heap: r.items.into_iter().map(Reverse).collect(),
            next_seq: r.next_seq,
        }
    }
}

impl From<EventQueue> for QueueRepr {
    fn from(q: EventQueue) -> Self {
        QueueRepr {
            next_seq: q.next_seq,
            items: q.sorted(),
        }
    }
}

impl EventQueue {
    /// Schedules an internal event (phase 1) in scheduling order.
    pub fn schedule(&mut self, tick: Tick, event: Event) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.heap.push(Reverse(Scheduled {
            tick,
            phase: 1,
            seq,
            event,
        }));
        seq
    }

    /// Schedules an input (phase 0) under the sequence number stored in the input log.
    pub fn schedule_input(&mut self, tick: Tick, input_seq: u64, event: Event) {
        self.heap.push(Reverse(Scheduled {
            tick,
            phase: 0,
            seq: input_seq,
            event,
        }));
    }

    /// The earliest scheduled tick, if any.
    #[must_use]
    pub fn peek_tick(&self) -> Option<Tick> {
        self.heap.peek().map(|Reverse(s)| s.tick)
    }

    /// Pops the next event if it is due at or before `until`.
    pub fn pop_due(&mut self, until: Tick) -> Option<Scheduled> {
        match self.heap.peek() {
            Some(Reverse(s)) if s.tick <= until => self.heap.pop().map(|Reverse(s)| s),
            _ => None,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// All pending events in execution order.
    #[must_use]
    pub fn sorted(&self) -> Vec<Scheduled> {
        let mut v: Vec<Scheduled> = self.heap.iter().map(|Reverse(s)| s.clone()).collect();
        v.sort();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_tick_pops_in_schedule_order() {
        let mut q = EventQueue::default();
        q.schedule(Tick(10), Event::DayRollover);
        q.schedule(
            Tick(5),
            Event::ObjectFree {
                object: crate::ids::ObjectId(1),
            },
        );
        q.schedule(
            Tick(5),
            Event::ObjectFree {
                object: crate::ids::ObjectId(0),
            },
        );
        q.schedule_input(Tick(5), 7, Event::DayRollover);
        let order: Vec<(u32, u8, u64)> = std::iter::from_fn(|| q.pop_due(Tick(10)))
            .map(|s| (s.tick.0, s.phase, s.seq))
            .collect();
        assert_eq!(order, vec![(5, 0, 7), (5, 1, 1), (5, 1, 2), (10, 1, 0)]);
    }

    #[test]
    fn pop_due_respects_until() {
        let mut q = EventQueue::default();
        q.schedule(Tick(10), Event::DayRollover);
        assert!(q.pop_due(Tick(9)).is_none());
        assert!(q.pop_due(Tick(10)).is_some());
    }

    #[test]
    fn serialized_form_is_order_independent() {
        let mut a = EventQueue::default();
        let mut b = EventQueue::default();
        for (t, s) in [(3u32, 0u64), (1, 1), (2, 2)] {
            a.schedule(Tick(t), Event::DayRollover);
            let _ = s;
        }
        // Same logical content, different insertion history.
        b.schedule(Tick(1), Event::DayRollover);
        b.schedule(Tick(2), Event::DayRollover);
        b.schedule(Tick(3), Event::DayRollover);
        let ra: QueueRepr = a.clone().into();
        let rb: QueueRepr = b.clone().into();
        let ta: Vec<u32> = ra.items.iter().map(|s| s.tick.0).collect();
        let tb: Vec<u32> = rb.items.iter().map(|s| s.tick.0).collect();
        assert_eq!(ta, tb);
    }
}
