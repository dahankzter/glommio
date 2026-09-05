// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
//! Hierarchical timing wheel over stable slab handles.
//!
//! Timers live in a [`TimerSlab`]; the wheel's slots hold [`SlotIndex`] values
//! pointing into it. Cascading therefore moves four-byte indices between slots
//! rather than whole entries, and a handle stays valid for a timer's entire
//! life however many times it cascades — which is what makes cancellation a
//! bare array index rather than a hash lookup.
//!
//! Each entry records where in the wheel it currently sits, so removal goes
//! straight there. Removing from a slot is a `swap_remove`, and the entry that
//! moves into the hole has its recorded position corrected; nothing is left
//! behind to be skipped later.

use super::slab::{SlotIndex, TimerId, TimerSlab};
use std::{
    collections::BTreeMap,
    task::Waker,
    time::{Duration, Instant},
};

/// Level 0: 256 slots x 1ms
const LEVEL_0_SLOTS: usize = 256;
/// Level 1: 64 slots x 256ms
const LEVEL_1_SLOTS: usize = 64;
const LEVEL_1_RESOLUTION_MS: u64 = 256;
/// Level 2: 64 slots x 16.384s
const LEVEL_2_SLOTS: usize = 64;
const LEVEL_2_RESOLUTION_MS: u64 = 16_384;
/// Level 3: 64 slots x 17.48min
const LEVEL_3_SLOTS: usize = 64;
const LEVEL_3_RESOLUTION_MS: u64 = 1_048_576;

/// Beyond 18 hours a timer waits in a `BTreeMap` instead of a slot.
const OVERFLOW_THRESHOLD_MS: u64 = 67_108_864;

/// Where an entry currently sits, so that cancelling it goes straight there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WheelPos {
    /// In a wheel slot, waiting for its level to come round.
    Slot {
        level: u8,
        slot: usize,
        index: usize,
    },
    /// Due, and waiting to be drained.
    Expired { index: usize },
    /// Too far out for the wheel; parked in the overflow map under `key`.
    Overflow { key: Instant, index: usize },
}

#[derive(Debug)]
struct TimerEntry {
    expires_at: Instant,
    waker: Waker,
    at: WheelPos,
}

/// Hierarchical timing wheel with stable handles.
#[derive(Debug)]
pub(crate) struct TimingWheel {
    current_tick: u64,
    start_time: Instant,

    /// Owns every entry. Slots below hold indices into this.
    slab: TimerSlab<TimerEntry>,

    /// Due, awaiting drain.
    expired: Vec<SlotIndex>,

    slots_1ms: Box<[Vec<SlotIndex>; LEVEL_0_SLOTS]>,
    slots_256ms: Box<[Vec<SlotIndex>; LEVEL_1_SLOTS]>,
    slots_16s: Box<[Vec<SlotIndex>; LEVEL_2_SLOTS]>,
    slots_17min: Box<[Vec<SlotIndex>; LEVEL_3_SLOTS]>,

    /// Past the wheel's reach. Cold.
    overflow: BTreeMap<Instant, Vec<SlotIndex>>,
}

impl TimingWheel {
    pub(crate) fn new() -> Self {
        Self::new_at(Instant::now())
    }

    pub(crate) fn new_at(start_time: Instant) -> Self {
        Self {
            current_tick: 0,
            start_time,
            slab: TimerSlab::new(),
            expired: Vec::new(),
            slots_1ms: Box::new(std::array::from_fn(|_| Vec::new())),
            slots_256ms: Box::new(std::array::from_fn(|_| Vec::new())),
            slots_16s: Box::new(std::array::from_fn(|_| Vec::new())),
            slots_17min: Box::new(std::array::from_fn(|_| Vec::new())),
            overflow: BTreeMap::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.slab.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn current_time(&self) -> Instant {
        self.start_time + Duration::from_millis(self.current_tick)
    }

    /// Earliest deadline held, if any.
    ///
    /// Linear in the slab today. The wheel knows structurally which slots are
    /// occupied and could answer from that in constant time; doing so is a
    /// separate change, and putting the naive version behind this name is what
    /// makes that change local.
    pub(crate) fn next_expiry(&self) -> Option<Instant> {
        self.slab.iter().map(|entry| entry.expires_at).min()
    }

    /// Whether a handle still names a live timer.
    pub(crate) fn contains(&self, id: TimerId) -> bool {
        self.slab.get(id).is_some()
    }

    /// Register a timer, returning a handle valid until it fires or is removed.
    pub(crate) fn insert(&mut self, expires_at: Instant, waker: Waker) -> TimerId {
        let id = self.slab.insert(TimerEntry {
            expires_at,
            waker,
            // Corrected below, once the entry has somewhere to be.
            at: WheelPos::Expired { index: usize::MAX },
        });
        self.place(id);
        id
    }

    /// Withdraw a timer. `false` if it has already fired or already gone.
    pub(crate) fn remove(&mut self, id: TimerId) -> bool {
        let Some(entry) = self.slab.get(id) else {
            return false;
        };
        let at = entry.at;
        self.unlink(at);
        self.slab.remove(id).is_some()
    }

    /// Move time forward, expiring whatever has come due.
    ///
    /// Cost is proportional to the ticks actually crossed. Skipping empty
    /// spans is a separate change; see [`Self::next_expiry`].
    pub(crate) fn advance_to(&mut self, now: Instant) {
        if now <= self.start_time {
            return;
        }

        let target_tick = now
            .duration_since(self.start_time)
            .as_millis()
            .min(u64::MAX as u128) as u64;

        while self.current_tick < target_tick {
            self.tick();
        }

        self.check_overflow();
    }

    /// Take everything that has come due.
    pub(crate) fn drain_expired(&mut self) -> impl Iterator<Item = (TimerId, Waker)> {
        let drained: Vec<_> = std::mem::take(&mut self.expired)
            .into_iter()
            .filter_map(|slot| {
                let id = self.slab.id_at(slot)?;
                let entry = self.slab.remove(id)?;
                Some((id, entry.waker))
            })
            .collect();
        drained.into_iter()
    }

    // ---- placement -------------------------------------------------------

    /// Put an already-stored entry wherever its deadline says it belongs, and
    /// record where that was.
    fn place(&mut self, id: TimerId) {
        let expires_at = self
            .slab
            .get(id)
            .expect("caller just stored this entry")
            .expires_at;

        let deadline_ms = match expires_at.checked_duration_since(self.start_time) {
            // Round up. `as_millis` truncates, which places a timer due at
            // 1.7ms in tick 1 and fires it 0.7ms early -- before its own
            // deadline, so the future polls, finds itself not ready, and has to
            // arm again. A timer may fire late; it may never fire early.
            Some(duration) => duration
                .as_nanos()
                .div_ceil(1_000_000)
                .min(u64::MAX as u128) as u64,
            // Already in the past.
            None => return self.mark_expired(id),
        };

        if deadline_ms <= self.current_tick {
            return self.mark_expired(id);
        }

        let ticks_until_expiry = deadline_ms - self.current_tick;
        let slot = id.slot();

        let at = if ticks_until_expiry < LEVEL_1_RESOLUTION_MS {
            let s = (deadline_ms % LEVEL_0_SLOTS as u64) as usize;
            self.slots_1ms[s].push(slot);
            WheelPos::Slot {
                level: 0,
                slot: s,
                index: self.slots_1ms[s].len() - 1,
            }
        } else if ticks_until_expiry < LEVEL_2_RESOLUTION_MS {
            let s = ((deadline_ms / LEVEL_1_RESOLUTION_MS) % LEVEL_1_SLOTS as u64) as usize;
            self.slots_256ms[s].push(slot);
            WheelPos::Slot {
                level: 1,
                slot: s,
                index: self.slots_256ms[s].len() - 1,
            }
        } else if ticks_until_expiry < LEVEL_3_RESOLUTION_MS {
            let s = ((deadline_ms / LEVEL_2_RESOLUTION_MS) % LEVEL_2_SLOTS as u64) as usize;
            self.slots_16s[s].push(slot);
            WheelPos::Slot {
                level: 2,
                slot: s,
                index: self.slots_16s[s].len() - 1,
            }
        } else if ticks_until_expiry < OVERFLOW_THRESHOLD_MS {
            let s = ((deadline_ms / LEVEL_3_RESOLUTION_MS) % LEVEL_3_SLOTS as u64) as usize;
            self.slots_17min[s].push(slot);
            WheelPos::Slot {
                level: 3,
                slot: s,
                index: self.slots_17min[s].len() - 1,
            }
        } else {
            let bucket = self.overflow.entry(expires_at).or_default();
            bucket.push(slot);
            WheelPos::Overflow {
                key: expires_at,
                index: bucket.len() - 1,
            }
        };

        self.record(id, at);
    }

    fn mark_expired(&mut self, id: TimerId) {
        self.expired.push(id.slot());
        let at = WheelPos::Expired {
            index: self.expired.len() - 1,
        };
        self.record(id, at);
    }

    fn record(&mut self, id: TimerId, at: WheelPos) {
        if let Some(entry) = self.slab.get_mut(id) {
            entry.at = at;
        }
    }

    /// Detach an entry from wherever it sits, correcting whatever moves into
    /// the hole it leaves.
    fn unlink(&mut self, at: WheelPos) {
        let (moved_slot, corrected) = match at {
            WheelPos::Slot { level, slot, index } => {
                let bucket = self.bucket_mut(level, slot);
                if index >= bucket.len() {
                    return;
                }
                bucket.swap_remove(index);
                let moved = bucket.get(index).copied();
                (moved, WheelPos::Slot { level, slot, index })
            }
            WheelPos::Expired { index } => {
                if index >= self.expired.len() {
                    return;
                }
                self.expired.swap_remove(index);
                let moved = self.expired.get(index).copied();
                (moved, WheelPos::Expired { index })
            }
            WheelPos::Overflow { key, index } => {
                let Some(bucket) = self.overflow.get_mut(&key) else {
                    return;
                };
                if index >= bucket.len() {
                    return;
                }
                bucket.swap_remove(index);
                let moved = bucket.get(index).copied();
                if bucket.is_empty() {
                    self.overflow.remove(&key);
                }
                (moved, WheelPos::Overflow { key, index })
            }
        };

        // `swap_remove` moved the last element into the hole. Its recorded
        // position now lies, so correct it before anything reads it.
        if let Some(moved) = moved_slot {
            if let Some(id) = self.slab.id_at(moved) {
                self.record(id, corrected);
            }
        }
    }

    fn bucket_mut(&mut self, level: u8, slot: usize) -> &mut Vec<SlotIndex> {
        match level {
            0 => &mut self.slots_1ms[slot],
            1 => &mut self.slots_256ms[slot],
            2 => &mut self.slots_16s[slot],
            3 => &mut self.slots_17min[slot],
            _ => unreachable!("wheel has four levels"),
        }
    }

    // ---- time ------------------------------------------------------------

    fn tick(&mut self) {
        self.current_tick += 1;

        let slot_0 = (self.current_tick % LEVEL_0_SLOTS as u64) as usize;
        self.expire_slot(slot_0);

        if self.current_tick.is_multiple_of(LEVEL_1_RESOLUTION_MS) {
            let s = ((self.current_tick / LEVEL_1_RESOLUTION_MS) % LEVEL_1_SLOTS as u64) as usize;
            self.cascade_slot(1, s);
        }
        if self.current_tick.is_multiple_of(LEVEL_2_RESOLUTION_MS) {
            let s = ((self.current_tick / LEVEL_2_RESOLUTION_MS) % LEVEL_2_SLOTS as u64) as usize;
            self.cascade_slot(2, s);
        }
        if self.current_tick.is_multiple_of(LEVEL_3_RESOLUTION_MS) {
            let s = ((self.current_tick / LEVEL_3_RESOLUTION_MS) % LEVEL_3_SLOTS as u64) as usize;
            self.cascade_slot(3, s);
        }

        self.check_overflow();
    }

    /// Everything in a level-0 slot is due.
    fn expire_slot(&mut self, slot: usize) {
        let due = std::mem::take(&mut self.slots_1ms[slot]);
        for entry_slot in due {
            self.expired.push(entry_slot);
            let at = WheelPos::Expired {
                index: self.expired.len() - 1,
            };
            if let Some(id) = self.slab.id_at(entry_slot) {
                self.record(id, at);
            }
        }
    }

    /// Re-place a higher level's slot into finer levels.
    fn cascade_slot(&mut self, level: u8, slot: usize) {
        let moving = std::mem::take(self.bucket_mut(level, slot));
        for entry_slot in moving {
            if let Some(id) = self.slab.id_at(entry_slot) {
                self.place(id);
            }
        }
    }

    /// Pull overflow entries that have come within the wheel's reach.
    fn check_overflow(&mut self) {
        let threshold = self.current_time() + Duration::from_millis(OVERFLOW_THRESHOLD_MS);

        let due: Vec<Instant> = self
            .overflow
            .range(..=threshold)
            .map(|(key, _)| *key)
            .collect();
        if due.is_empty() {
            return;
        }

        for key in due {
            if let Some(bucket) = self.overflow.remove(&key) {
                for entry_slot in bucket {
                    if let Some(id) = self.slab.id_at(entry_slot) {
                        self.place(id);
                    }
                }
            }
        }
    }
}

impl Default for TimingWheel {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // Helper: a waker that does nothing when woken.
    fn dummy_waker() -> Waker {
        Waker::noop().clone()
    }

    #[test]
    fn a_deadline_between_ticks_does_not_expire_at_the_earlier_one() {
        // A deadline is rounded up to a whole tick, never down. Truncating
        // places a timer due at 1.7ms in tick 1 and expires it at 1.0ms --
        // before it is due. The future then finds itself not ready and arms
        // again, so one sleep costs several registrations.
        //
        // Checked here rather than through the executor because the reactor
        // sleeps until the true deadline when nothing else is running, which
        // hides it; and because inline storage compares instants exactly, so
        // it only appears once the staged wheel has promoted.
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);
        wheel.insert(start + Duration::from_micros(1_700), dummy_waker());

        wheel.advance_to(start + Duration::from_millis(1));
        assert_eq!(
            wheel.drain_expired().count(),
            0,
            "expired at 1ms a timer that is due at 1.7ms"
        );

        wheel.advance_to(start + Duration::from_millis(2));
        assert_eq!(wheel.drain_expired().count(), 1, "due by 2ms");
    }

    #[test]
    fn test_basic_insert_and_expire() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timer expiring in 100ms
        let id = wheel.insert(start + Duration::from_millis(100), dummy_waker());

        assert_eq!(wheel.len(), 1);

        // Advance past expiry
        wheel.advance_to(start + Duration::from_millis(100));

        // Should have one expired timer
        let expired: Vec<_> = wheel.drain_expired().collect();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].0, id);

        // Wheel should be empty now
        assert_eq!(wheel.len(), 0);
    }

    #[test]
    fn test_remove_timer() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        let id = wheel.insert(start + Duration::from_millis(100), dummy_waker());
        assert_eq!(wheel.len(), 1);

        // Remove the timer
        assert!(wheel.remove(id));
        assert_eq!(wheel.len(), 0);

        // Advance time - should not expire
        wheel.advance_to(start + Duration::from_millis(100));
        assert_eq!(wheel.drain_expired().count(), 0);
    }

    #[test]
    fn test_multiple_timers_same_slot() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert 3 timers at same expiry
        let id1 = wheel.insert(start + Duration::from_millis(50), dummy_waker());
        let id2 = wheel.insert(start + Duration::from_millis(50), dummy_waker());
        let id3 = wheel.insert(start + Duration::from_millis(50), dummy_waker());

        assert_eq!(wheel.len(), 3);

        wheel.advance_to(start + Duration::from_millis(50));

        let expired: Vec<_> = wheel.drain_expired().map(|(id, _)| id).collect();
        assert_eq!(expired.len(), 3);
        assert!(expired.contains(&id1));
        assert!(expired.contains(&id2));
        assert!(expired.contains(&id3));
    }

    #[test]
    fn test_timer_ordering() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timers in reverse order
        wheel.insert(start + Duration::from_millis(30), dummy_waker());
        wheel.insert(start + Duration::from_millis(10), dummy_waker());
        wheel.insert(start + Duration::from_millis(20), dummy_waker());

        // Advance to 10ms
        wheel.advance_to(start + Duration::from_millis(10));
        assert_eq!(wheel.drain_expired().count(), 1);

        // Advance to 20ms
        wheel.advance_to(start + Duration::from_millis(20));
        assert_eq!(wheel.drain_expired().count(), 1);

        // Advance to 30ms
        wheel.advance_to(start + Duration::from_millis(30));
        assert_eq!(wheel.drain_expired().count(), 1);
    }

    #[test]
    fn test_cascading_level_1_to_0() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timer at 500ms (will be in Level 1)
        let id = wheel.insert(start + Duration::from_millis(500), dummy_waker());

        // Advance to just before Level 1 cascade (255ms)
        wheel.advance_to(start + Duration::from_millis(255));
        assert_eq!(wheel.drain_expired().count(), 0);

        // Advance to 256ms - should cascade from Level 1 to Level 0
        wheel.advance_to(start + Duration::from_millis(256));
        assert_eq!(wheel.drain_expired().count(), 0); // Not expired yet

        // Advance to 500ms - should expire
        wheel.advance_to(start + Duration::from_millis(500));
        let expired: Vec<_> = wheel.drain_expired().collect();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].0, id);
    }

    #[test]
    fn test_long_duration_timer() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timer at 1 hour (will be in Level 3)
        let id = wheel.insert(start + Duration::from_secs(3600), dummy_waker());

        assert_eq!(wheel.len(), 1);

        // Advance to expiry
        wheel.advance_to(start + Duration::from_secs(3600));

        let expired: Vec<_> = wheel.drain_expired().collect();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].0, id);
    }

    #[test]
    fn test_overflow_to_btreemap() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timer at 24 hours (should overflow to BTreeMap)
        let id = wheel.insert(start + Duration::from_secs(86400), dummy_waker());

        assert_eq!(wheel.len(), 1);

        // Advance to expiry
        wheel.advance_to(start + Duration::from_secs(86400));

        let expired: Vec<_> = wheel.drain_expired().collect();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].0, id);
    }

    #[test]
    fn test_past_timer_expires_immediately() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timer in the past
        wheel.insert(start - Duration::from_millis(100), dummy_waker());

        // Should expire immediately
        assert_eq!(wheel.drain_expired().count(), 1);
    }

    #[test]
    fn test_wrap_around() {
        let start = Instant::now();
        let mut wheel = TimingWheel::new_at(start);

        // Insert timer at slot 10
        let id1 = wheel.insert(start + Duration::from_millis(10), dummy_waker());

        // Advance to slot 260 (wraps around Level 0)
        wheel.advance_to(start + Duration::from_millis(260));

        // Insert another timer at slot 10 (should be different from id1)
        let id2 = wheel.insert(start + Duration::from_millis(260 + 10), dummy_waker());

        assert_ne!(id1, id2);

        // First timer should have expired
        assert!(wheel.drain_expired().any(|(id, _)| id == id1));

        // Advance to second timer
        wheel.advance_to(start + Duration::from_millis(270));
        assert!(wheel.drain_expired().any(|(id, _)| id == id2));
    }
}
