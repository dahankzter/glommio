// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
//! The reactor's view of its timers: an ordered map keyed by deadline.
//!
//! This is the incumbent — what upstream glommio has always used — and it is
//! here to be beaten rather than improved on. Insert and remove are O(log n);
//! the next deadline is the first key, which is O(1) and is the one thing a
//! wheel has to work to match.
//!
//! It differs from upstream in one respect, in the incumbent's favour.
//! Upstream's handle is a bare `u64` and it keeps an `AHashMap<u64, Instant>`
//! so a cancellation can recover the deadline before removing anything.
//! Carrying the deadline in the handle removes the map and the lookup.
//! Measuring the version with it would charge the incumbent for a hash lookup
//! the other arms do not pay, which would flatter them.

use super::timer_id::TimerId;
use std::{
    collections::BTreeMap,
    task::Waker,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(crate) struct ReactorTimers {
    /// Ordered by deadline, so the first entry is the next to fire. The `u64`
    /// separates timers sharing a deadline.
    timers: BTreeMap<(Instant, u64), Waker>,
    next_seq: u64,
}

impl ReactorTimers {
    pub(crate) fn new() -> Self {
        Self {
            timers: BTreeMap::new(),
            next_seq: 0,
        }
    }

    /// Register a timer, returning a handle that names its place directly.
    pub(crate) fn insert(&mut self, expires_at: Instant, waker: Waker) -> TimerId {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);
        self.timers.insert((expires_at, seq), waker);
        TimerId {
            when: expires_at,
            seq,
        }
    }

    /// Withdraw a timer. `false` if it has already fired.
    pub(crate) fn remove(&mut self, id: TimerId) -> bool {
        self.timers.remove(&(id.when, id.seq)).is_some()
    }

    /// Expire what is due, hand back the wakers, and say when to wake next.
    ///
    /// Wakers are returned rather than woken here. The caller holds a `RefMut`
    /// on the reactor's timers for the duration of this call, and waking under
    /// it would let a waker that touches a timer re-enter and panic on the
    /// second borrow. Upstream wakes inline and has the same hazard; it is
    /// fixed here so all three arms share one reactor path and differ only in
    /// how timers are stored.
    pub(crate) fn process_timers(&mut self, wakers: &mut Vec<Waker>) -> (Option<Duration>, usize) {
        let now = Instant::now();

        // Everything before `now` is due. `split_off` hands back the rest.
        let pending = self.timers.split_off(&(now, 0));
        let ready = std::mem::replace(&mut self.timers, pending);

        let woke = ready.len();
        wakers.extend(ready.into_values());

        let next = self
            .timers
            .keys()
            .next()
            .map(|(when, _)| when.saturating_duration_since(now));

        (next, woke)
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.timers.len()
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }
}

impl Default for ReactorTimers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_waker() -> Waker {
        Waker::noop().clone()
    }

    #[test]
    fn a_timer_fires_once_its_deadline_passes() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();
        timers.insert(now + Duration::from_millis(20), dummy_waker());
        assert_eq!(timers.len(), 1);

        std::thread::sleep(Duration::from_millis(30));
        let mut wakers = Vec::new();
        let (_, woke) = timers.process_timers(&mut wakers);

        assert_eq!(woke, 1);
        assert_eq!(wakers.len(), 1);
        assert_eq!(timers.len(), 0);
    }

    #[test]
    fn removing_twice_reports_the_second_as_absent() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();

        let id = timers.insert(now + Duration::from_millis(100), dummy_waker());
        assert_eq!(timers.len(), 1);

        assert!(timers.remove(id), "the first removal withdraws it");
        assert_eq!(timers.len(), 0);
        assert!(!timers.remove(id), "the second finds nothing to withdraw");
    }

    #[test]
    fn timers_sharing_a_deadline_stay_distinct() {
        let mut timers = ReactorTimers::new();
        let deadline = Instant::now() + Duration::from_millis(100);

        let first = timers.insert(deadline, dummy_waker());
        let second = timers.insert(deadline, dummy_waker());
        assert_eq!(timers.len(), 2, "the same instant holds two timers");

        assert!(timers.remove(first));
        assert_eq!(timers.len(), 1);
        assert!(timers.remove(second), "the other is untouched");
    }

    #[test]
    fn the_next_deadline_is_the_earliest_one_held() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();

        timers.insert(now + Duration::from_secs(30), dummy_waker());
        let soonest = timers.insert(now + Duration::from_millis(50), dummy_waker());
        timers.insert(now + Duration::from_secs(10), dummy_waker());

        let mut wakers = Vec::new();
        let (next, woke) = timers.process_timers(&mut wakers);
        assert_eq!(woke, 0, "none are due yet");
        let next = next.expect("three are pending");
        assert!(
            next <= Duration::from_millis(50),
            "reported {next:?} as the wait for a timer due in 50ms"
        );

        assert!(timers.remove(soonest));
        let (next, _) = timers.process_timers(&mut wakers);
        assert!(
            next.expect("two remain") > Duration::from_secs(1),
            "withdrawing the earliest moves the answer to the next one"
        );
    }
}
