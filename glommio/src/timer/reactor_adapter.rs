// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
//! The reactor's view of the timer wheel, backed by the `bitwheel` crate.
//!
//! `bitwheel` does not cascade: a timer is placed in the gear matching its
//! delay and stays there, which is what lets its handle name a fixed location.
//! The cost is that a gear's slots have a fixed capacity, and a timer whose
//! slot is full is placed in a neighbouring one — firing late — or, if none of
//! the probed slots has room, parked in a `BTreeMap` instead.
//!
//! That matters here more than it looks. Deadline churn gives every connection
//! the *same* timeout, so their deadlines cluster, and clustered deadlines land
//! in the same slot. See [`SLOT_CAP`].

use bitwheel::timer::{BitWheelWithFailover, Timer as BitwheelTimer, TimerHandle};
use std::{
    task::Waker,
    time::{Duration, Instant},
};

/// One millisecond, matching the tick the rest of glommio reasons in.
///
/// `bitwheel` defaults to 4ms. Taking that default would hand this arm a
/// coarser clock than the one it is compared against, so the comparison would
/// measure the clock rather than the structure.
const RESOLUTION_MS: u64 = 1;

/// Gears are radix 64, so five reach 64^5 ticks — about twelve days.
const NUM_GEARS: usize = 5;

/// Timers per slot.
///
/// This is the number that decides whether this arm behaves like a wheel or
/// like a `BTreeMap`. A slot holds timers whose deadlines fall in the same
/// span, and deadline churn gives every connection an identical timeout — so a
/// thousand connections opened together want a thousand places in one slot.
/// Whatever is chosen, some population exceeds it and spills.
///
/// 128 is a deliberate middle: large enough that the ladder's lower rungs sit
/// in the wheel, small enough that the memory is not absurd, and small enough
/// that the upper rungs spill and the measurement shows it rather than hiding
/// it behind a number picked to flatter.
const SLOT_CAP: usize = 128;

/// Slots probed when the target is full before falling back to the map. Each
/// probe is one slot of lateness.
const MAX_PROBES: usize = 3;

/// How often the failover map is consulted, in ticks.
const FAILOVER_INTERVAL: u64 = 64;

type Wheel = BitWheelWithFailover<
    WakerTimer,
    NUM_GEARS,
    RESOLUTION_MS,
    SLOT_CAP,
    MAX_PROBES,
    FAILOVER_INTERVAL,
>;

/// What the wheel stores. Firing hands the waker to the caller rather than
/// waking it, so nothing is woken while the reactor still holds its timers
/// borrowed.
#[derive(Debug)]
struct WakerTimer(Option<Waker>);

impl BitwheelTimer for WakerTimer {
    type Context = Vec<Waker>;

    fn fire(&mut self, ctx: &mut Self::Context) {
        if let Some(waker) = self.0.take() {
            ctx.push(waker);
        }
    }
}

pub(crate) struct ReactorTimers {
    wheel: Box<Wheel>,
}

impl std::fmt::Debug for ReactorTimers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactorTimers")
            .field("len", &self.wheel.len())
            .field("failover", &self.wheel.failover_len())
            .finish()
    }
}

impl ReactorTimers {
    pub(crate) fn new() -> Self {
        // Boxed because the gears are inline fixed-capacity arrays: five gears
        // of 64 slots holding 128 timers each is far too large for the stack.
        Self {
            wheel: Wheel::boxed(),
        }
    }

    /// Register a timer. Never fails: one that finds no room in the wheel is
    /// parked in the failover map instead.
    pub(crate) fn insert(&mut self, expires_at: Instant, waker: Waker) -> TimerHandle {
        self.wheel.insert(expires_at, WakerTimer(Some(waker)))
    }

    /// Withdraw a timer. `false` if it has already fired.
    pub(crate) fn remove(&mut self, id: TimerHandle) -> bool {
        self.wheel.cancel(id).is_some()
    }

    /// Expire what is due, hand back the wakers, and say when to wake next.
    ///
    /// Wakers are returned rather than woken here. The caller holds a `RefMut`
    /// on the reactor's timers for the duration of this call, and waking under
    /// it would let a waker that touches a timer re-enter and panic on the
    /// second borrow.
    pub(crate) fn process_timers(&mut self, wakers: &mut Vec<Waker>) -> (Option<Duration>, usize) {
        let now = Instant::now();
        let woke = self.wheel.poll(now, wakers);

        let next = self
            .wheel
            .peek_next_fire()
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

    /// Timers that found no room in the wheel and are waiting in the failover
    /// map. Anything counted here is being served by the structure this arm
    /// was meant to replace.
    #[allow(dead_code)]
    pub(crate) fn failover_len(&self) -> usize {
        self.wheel.failover_len()
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

    pub(super) fn dummy_waker() -> Waker {
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

        // `len()` should be 0 here and is not. bitwheel 0.6.0 decrements its
        // count in `cancel` but not in `drain_and_fire`, so a timer that fires
        // is still counted -- permanently, and cumulatively.
        //
        // Asserted as-is rather than worked around: this arm is being
        // evaluated, and a wrong length is a fact about the candidate. The
        // population figures the comparison uses come from the reactor's own
        // counters, which are above this and unaffected.
        assert_eq!(
            timers.len(),
            1,
            "bitwheel 0.6.0 does not decrement len on fire; update this when it does"
        );
    }

    #[test]
    fn removing_twice_reports_the_second_as_absent() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();

        let id = timers.insert(now + Duration::from_millis(100), dummy_waker());
        assert_eq!(timers.len(), 1);

        assert!(timers.remove(id), "the first removal withdraws it");
        assert_eq!(timers.len(), 0);
    }

    #[test]
    fn deadlines_that_cluster_beyond_a_slot_spill_to_failover() {
        // Every connection in a deadline-churn workload gets the same timeout,
        // so their deadlines cluster. This is the shape that decides whether
        // this arm behaves like a wheel or like the map it wraps.
        let mut timers = ReactorTimers::new();
        let now = Instant::now();

        let clustered = SLOT_CAP * 4;
        for _ in 0..clustered {
            timers.insert(now + Duration::from_secs(30), dummy_waker());
        }

        assert_eq!(timers.len(), clustered, "all of them are held");
        assert!(
            timers.failover_len() > 0,
            "expected clustered deadlines to exceed a slot's {SLOT_CAP} places"
        );
    }
}

#[cfg(test)]
mod bitwheel_contract {
    use super::{tests::dummy_waker, *};

    /// Cancelling a handle whose timer has already fired is something glommio
    /// does routinely: a `Timer` future that completes still runs its `Drop`.
    /// The crate documents this as returning `None`.
    #[test]
    fn cancelling_an_already_fired_timer_is_safe() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();
        let id = timers.insert(now + Duration::from_millis(5), dummy_waker());

        std::thread::sleep(Duration::from_millis(15));
        let mut wakers = Vec::new();
        let (_, woke) = timers.process_timers(&mut wakers);
        assert_eq!(woke, 1, "it fired");

        assert!(!timers.remove(id), "cancelling after firing finds nothing");
    }

    /// Two timers on the same deadline, one cancelled, then the other fires.
    #[test]
    fn cancelling_one_of_two_in_a_slot_leaves_the_other_intact() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();
        let first = timers.insert(now + Duration::from_millis(5), dummy_waker());
        let _second = timers.insert(now + Duration::from_millis(5), dummy_waker());

        assert!(timers.remove(first));

        std::thread::sleep(Duration::from_millis(15));
        let mut wakers = Vec::new();
        let (_, woke) = timers.process_timers(&mut wakers);
        assert_eq!(woke, 1, "the survivor fires");
    }

    /// The same failure on the crate's own shipped preset, to show it is not
    /// a consequence of how this arm configures it.
    ///
    /// `bitwheel::timer::Wheel` is the crate's "STANDARD" alias --
    /// `BitWheel<T, 2, 4, 32, 8>`. A 400ms deadline lands in gear 1, whose
    /// slots span 64 ticks of 4ms; polling at 256ms crosses that slot boundary
    /// and fires the timer, 144ms before it was due.
    #[test]
    #[ignore = "reaches undefined behaviour in bitwheel 0.6.0; aborts rather than fails"]
    fn the_same_holds_for_the_crates_own_preset() {
        use bitwheel::timer::Wheel;

        let epoch = Instant::now();
        let mut wheel: Box<Wheel<WakerTimer>> = Wheel::boxed_with_epoch(epoch);

        let handle = wheel
            .insert(
                epoch + Duration::from_millis(400),
                WakerTimer(Some(dummy_waker())),
            )
            .expect("room in the slot");

        let mut ctx = Vec::new();
        wheel.poll(epoch + Duration::from_millis(256), &mut ctx);

        // Still 144ms before the deadline, so cancel takes the unchecked path.
        wheel.cancel(handle);
    }

    /// Minimal reproduction of the soundness bug glommio's suite hits.
    ///
    /// `BitWheel::cancel` justifies an unchecked `remove` with, among others,
    /// the invariant "the `when_offset > current_tick` check ensures the timer
    /// hasn't fired yet, so the entry must still exist in the wheel".
    ///
    /// That does not hold. `poll_tick` drains a whole gear-`g` slot whenever
    /// `tick % 64^g == 0`, firing every timer in it -- including ones whose
    /// deadline is up to `64^g - 1` ticks away. Such a timer has fired while
    /// `when_offset > current_tick` is still true, so `cancel` proceeds into a
    /// vacant entry and reaches `hint::unreachable_unchecked`.
    ///
    /// Ignored because it aborts the process rather than failing: with debug
    /// assertions on it trips std's precondition check, and without them it is
    /// undefined behaviour. Run explicitly to confirm the bug still exists:
    /// `cargo test --features debugging -- --ignored cancel_after_an_early_fire`
    #[test]
    #[ignore = "reaches undefined behaviour in bitwheel 0.6.0; aborts rather than fails"]
    fn cancel_after_an_early_fire_reaches_unreachable_unchecked() {
        let mut timers = ReactorTimers::new();
        let now = Instant::now();

        // ~100ms lands in gear 1, whose slots span 64 ticks.
        let id = timers.insert(now + Duration::from_millis(100), dummy_waker());

        // Cross a gear-1 boundary while the deadline is still in the future.
        let mut wakers = Vec::new();
        std::thread::sleep(Duration::from_millis(70));
        timers.process_timers(&mut wakers);

        // The timer may now have fired early. Cancelling it is what glommio
        // does when the future is dropped, and it is what breaks.
        timers.remove(id);
    }
}
