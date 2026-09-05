// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
//! The reactor's view of the timer wheel.
//!
//! Thin on purpose. The wheel owns its entries and knows where each one sits,
//! so this layer holds no index of its own — an earlier version kept a
//! parallel `HashMap` from timer to deadline and answered "when is the next
//! timer" by scanning it, which made every poll cost the whole population.

use super::{slab::TimerId, timing_wheel::TimingWheel};
use std::{
    task::Waker,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(crate) struct ReactorTimers {
    wheel: TimingWheel,
}

impl ReactorTimers {
    pub(crate) fn new() -> Self {
        Self {
            wheel: TimingWheel::new(),
        }
    }

    /// Register a timer, returning a handle that stays valid however many
    /// times the timer cascades.
    pub(crate) fn insert(&mut self, expires_at: Instant, waker: Waker) -> TimerId {
        self.wheel.insert(expires_at, waker)
    }

    /// Withdraw a timer. `false` if it has already fired.
    pub(crate) fn remove(&mut self, id: TimerId) -> bool {
        self.wheel.remove(id)
    }

    /// Whether a handle still names a live timer.
    pub(crate) fn exists(&self, id: TimerId) -> bool {
        self.wheel.contains(id)
    }

    /// Expire what is due, hand back the wakers, and say when to wake next.
    ///
    /// Wakers are returned rather than woken here. The caller holds a
    /// `RefMut` on the reactor's timers for the duration of this call, and
    /// waking under it would let a waker that touches a timer re-enter and
    /// panic on the second borrow.
    pub(crate) fn process_timers(&mut self, wakers: &mut Vec<Waker>) -> (Option<Duration>, usize) {
        let now = Instant::now();
        self.wheel.advance_to(now);

        let before = wakers.len();
        wakers.extend(self.wheel.drain_expired().map(|(_, waker)| waker));
        let woke = wakers.len() - before;

        let next = self
            .wheel
            .next_expiry()
            .map(|expires_at| expires_at.saturating_duration_since(now));

        (next, woke)
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.wheel.len()
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.wheel.is_empty()
    }
}

impl Default for ReactorTimers {
    fn default() -> Self {
        Self::new()
    }
}
