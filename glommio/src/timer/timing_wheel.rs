// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Hierarchical timing wheel for O(1) timer operations.
//!
//! This module implements a 4-level timing wheel optimized for the common case
//! of short-to-medium duration timers (0-18 hours). Longer timers overflow into
//! a BTreeMap fallback.
//!
//! # Architecture
//!
//! The wheel has 4 levels with different granularities:
//! - Level 0: 256 slots × 1ms = 0-255ms range (1ms resolution)
//! - Level 1: 64 slots × 256ms = 256ms-16s range (256ms resolution)
//! - Level 2: 64 slots × 16s = 16s-17min range (16s resolution)
//! - Level 3: 64 slots × 17min = 17min-18hr range (17min resolution)
//! - Overflow: BTreeMap for timers > 18 hours (rare)
//!
//! # Complexity
//!
//! - Insert: O(1) for most timers, O(log n) for overflow (>18hr)
//! - Remove: O(1) lookup in index, then O(1) swap-remove from slot
//! - Tick: O(k) where k = number of timers expiring this tick
//! - Cascade: O(k) where k = number of timers to re-insert
//!
//! # Example
//!
//! ```rust
//! use glommio::timer::timing_wheel::TimingWheel;
//! use std::time::{Duration, Instant};
//! use std::task::Waker;
//!
//! let mut wheel = TimingWheel::new();
//! let now = Instant::now();
//!
//! // Insert timer
//! let id = wheel.insert(now + Duration::from_millis(100), waker);
//!
//! // Advance time and process expired timers
//! wheel.advance_to(now + Duration::from_millis(100));
//! let expired = wheel.drain_expired();
//! ```

use ahash::AHashMap;
use std::collections::BTreeMap;
use std::task::Waker;
use std::time::{Duration, Instant};

/// Number of slots in Level 0 (1ms resolution, 0-255ms range)
const LEVEL_0_SLOTS: usize = 256;

/// Number of slots in Level 1 (256ms resolution, 256ms-16s range)
const LEVEL_1_SLOTS: usize = 64;
const LEVEL_1_RESOLUTION_MS: u64 = 256;

/// Number of slots in Level 2 (16s resolution, 16s-17min range)
const LEVEL_2_SLOTS: usize = 64;
const LEVEL_2_RESOLUTION_MS: u64 = 16_384;

/// Number of slots in Level 3 (17min resolution, 17min-18hr range)
const LEVEL_3_SLOTS: usize = 64;
const LEVEL_3_RESOLUTION_MS: u64 = 1_048_576;

/// Threshold for overflow to BTreeMap (18 hours in milliseconds)
const OVERFLOW_THRESHOLD_MS: u64 = 67_108_864;

/// A timer entry in the wheel
#[derive(Debug)]
struct TimerEntry {
    id: u64,
    expires_at: Instant,
    waker: Waker,
}

/// Location of a timer in the wheel (for O(1) removal)
#[derive(Debug, Clone, Copy)]
struct TimerLocation {
    level: u8,
    slot: usize,
    index_in_slot: usize,
}

/// Hierarchical timing wheel with O(1) operations for most timers
///
/// Field ordering optimized for cache locality:
/// - Hot fields (current_tick, start_time, index, expired) in first ~80 bytes
/// - Large slot arrays follow (span many cache lines anyway)
/// - Cold overflow field at the end
#[derive(Debug)]
pub struct TimingWheel {
    /// Current tick (HOT: checked/updated every tick) - 8 bytes
    current_tick: u64,

    /// Base time (HOT: used for time calculations) - 16 bytes
    start_time: Instant,

    /// Timer ID allocator (WARM: only touched on insert) - 8 bytes
    next_id: u64,

    /// Fast lookup: timer_id → location (HOT: accessed on insert/remove) - 24 bytes
    index: AHashMap<u64, TimerLocation>,

    /// Timers that have expired and are ready to fire (HOT: checked every tick) - 24 bytes
    expired: Vec<TimerEntry>,

    // Hot fields above total ~80 bytes (fit in 2 cache lines)
    // Large slot arrays below (span many cache lines)

    /// Level 0: 256 slots × 1ms = 0-255ms range
    slots_1ms: [Vec<TimerEntry>; LEVEL_0_SLOTS],

    /// Level 1: 64 slots × 256ms = 256ms-16s range
    slots_256ms: [Vec<TimerEntry>; LEVEL_1_SLOTS],

    /// Level 2: 64 slots × 16s = 16s-17min range
    slots_16s: [Vec<TimerEntry>; LEVEL_2_SLOTS],

    /// Level 3: 64 slots × 17min = 17min-18hr range
    slots_17min: [Vec<TimerEntry>; LEVEL_3_SLOTS],

    /// Overflow: BTreeMap for timers > 18 hours (COLD: rarely used)
    overflow: BTreeMap<Instant, Vec<TimerEntry>>,
}

impl TimingWheel {
    /// Create a new timing wheel starting at the current time
    pub fn new() -> Self {
        Self::new_at(Instant::now())
    }

    /// Create a new timing wheel starting at the specified time
    pub fn new_at(start_time: Instant) -> Self {
        Self {
            slots_1ms: std::array::from_fn(|_| Vec::new()),
            slots_256ms: std::array::from_fn(|_| Vec::new()),
            slots_16s: std::array::from_fn(|_| Vec::new()),
            slots_17min: std::array::from_fn(|_| Vec::new()),
            overflow: BTreeMap::new(),
            current_tick: 0,
            start_time,
            next_id: 0,
            index: AHashMap::new(),
            expired: Vec::new(),
        }
    }

    /// Insert a timer that expires at the given instant
    ///
    /// Returns the unique ID for this timer, which can be used to remove it
    pub fn insert(&mut self, expires_at: Instant, waker: Waker) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        let entry = TimerEntry {
            id,
            expires_at,
            waker,
        };

        self.insert_entry(entry);
        id
    }

    /// Insert a timer with a specific ID (used during promotion from staged wheel)
    ///
    /// This is an internal method used by StagedWheel to preserve IDs during promotion
    pub(crate) fn insert_with_id(&mut self, id: u64, expires_at: Instant, waker: Waker) {
        let entry = TimerEntry {
            id,
            expires_at,
            waker,
        };

        self.insert_entry(entry);

        // Update next_id if necessary to avoid ID collision
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    /// Remove a timer by ID
    ///
    /// Returns true if the timer was found and removed, false otherwise
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(location) = self.index.remove(&id) {
            if location.level == 255 {
                // Overflow removal - need to search BTreeMap
                self.remove_from_overflow(id);
            } else {
                self.remove_at_location(location);
            }
            true
        } else {
            // Check if timer is in expired queue (can happen if timer expired immediately on insert)
            if let Some(pos) = self.expired.iter().position(|e| e.id == id) {
                self.expired.swap_remove(pos);
                true
            } else {
                false
            }
        }
    }

    /// Remove a timer from the overflow BTreeMap
    fn remove_from_overflow(&mut self, id: u64) {
        // Search through overflow to find and remove the timer
        for entries in self.overflow.values_mut() {
            if let Some(pos) = entries.iter().position(|e| e.id == id) {
                entries.swap_remove(pos);
                return;
            }
        }
    }

    /// Advance time to the given instant, cascading timers as needed
    ///
    /// After calling this, use `drain_expired()` to get expired timers
    pub fn advance_to(&mut self, now: Instant) {
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

        // After large time advances, check if any overflow timers are now in range
        self.check_overflow();
    }

    /// Get the current time according to the wheel
    pub fn current_time(&self) -> Instant {
        self.start_time + Duration::from_millis(self.current_tick)
    }

    /// Drain all expired timers
    ///
    /// Returns an iterator over (timer_id, waker) pairs
    pub fn drain_expired(&mut self) -> impl Iterator<Item = (u64, Waker)> + '_ {
        self.expired.drain(..).map(|entry| (entry.id, entry.waker))
    }

    /// Get the number of timers currently in the wheel (including expired but not yet drained)
    pub fn len(&self) -> usize {
        self.index.len() + self.expired.len()
    }

    /// Check if the wheel is empty (including expired but not yet drained)
    pub fn is_empty(&self) -> bool {
        self.index.is_empty() && self.expired.is_empty()
    }

    // ========================================================================
    // Internal implementation
    // ========================================================================

    /// Insert a timer entry into the appropriate level
    fn insert_entry(&mut self, entry: TimerEntry) {
        let deadline_ms = match entry.expires_at.checked_duration_since(self.start_time) {
            Some(duration) => duration.as_millis().min(u64::MAX as u128) as u64,
            None => {
                // Timer is in the past - expire immediately
                self.expired.push(entry);
                return;
            }
        };

        if deadline_ms <= self.current_tick {
            // Already expired
            self.expired.push(entry);
            return;
        }

        let ticks_until_expiry = deadline_ms - self.current_tick;

        // Determine which level and slot
        if ticks_until_expiry < LEVEL_1_RESOLUTION_MS {
            // Level 0: 0-255ms (1ms resolution)
            let slot = (deadline_ms % LEVEL_0_SLOTS as u64) as usize;
            self.insert_at_level(entry, 0, slot);
        } else if ticks_until_expiry < LEVEL_2_RESOLUTION_MS {
            // Level 1: 256ms-16s (256ms resolution)
            let slot = ((deadline_ms / LEVEL_1_RESOLUTION_MS) % LEVEL_1_SLOTS as u64) as usize;
            self.insert_at_level(entry, 1, slot);
        } else if ticks_until_expiry < LEVEL_3_RESOLUTION_MS {
            // Level 2: 16s-17min (16s resolution)
            let slot = ((deadline_ms / LEVEL_2_RESOLUTION_MS) % LEVEL_2_SLOTS as u64) as usize;
            self.insert_at_level(entry, 2, slot);
        } else if ticks_until_expiry < OVERFLOW_THRESHOLD_MS {
            // Level 3: 17min-18hr (17min resolution)
            let slot = ((deadline_ms / LEVEL_3_RESOLUTION_MS) % LEVEL_3_SLOTS as u64) as usize;
            self.insert_at_level(entry, 3, slot);
        } else {
            // Overflow: > 18 hours
            let id = entry.id;
            let expires_at = entry.expires_at;
            let entries = self.overflow.entry(expires_at).or_insert_with(Vec::new);
            let index_in_slot = entries.len();
            entries.push(entry);

            // Track in index with special level marker (255 = overflow)
            self.index.insert(
                id,
                TimerLocation {
                    level: 255,
                    slot: 0, // Not used for overflow
                    index_in_slot,
                },
            );
        }
    }

    /// Insert an entry at a specific level and slot
    fn insert_at_level(&mut self, entry: TimerEntry, level: u8, slot: usize) {
        let id = entry.id;
        let index_in_slot = match level {
            0 => {
                let index = self.slots_1ms[slot].len();
                self.slots_1ms[slot].push(entry);
                index
            }
            1 => {
                let index = self.slots_256ms[slot].len();
                self.slots_256ms[slot].push(entry);
                index
            }
            2 => {
                let index = self.slots_16s[slot].len();
                self.slots_16s[slot].push(entry);
                index
            }
            3 => {
                let index = self.slots_17min[slot].len();
                self.slots_17min[slot].push(entry);
                index
            }
            _ => unreachable!(),
        };

        self.index.insert(
            id,
            TimerLocation {
                level,
                slot,
                index_in_slot,
            },
        );
    }

    /// Remove a timer at a specific location
    fn remove_at_location(&mut self, location: TimerLocation) {
        let (removed_id, swapped_id) = match location.level {
            0 => {
                let slot = &mut self.slots_1ms[location.slot];
                if location.index_in_slot >= slot.len() {
                    return;
                }
                let removed = slot.swap_remove(location.index_in_slot);
                let swapped = if location.index_in_slot < slot.len() {
                    Some(slot[location.index_in_slot].id)
                } else {
                    None
                };
                (removed.id, swapped)
            }
            1 => {
                let slot = &mut self.slots_256ms[location.slot];
                if location.index_in_slot >= slot.len() {
                    return;
                }
                let removed = slot.swap_remove(location.index_in_slot);
                let swapped = if location.index_in_slot < slot.len() {
                    Some(slot[location.index_in_slot].id)
                } else {
                    None
                };
                (removed.id, swapped)
            }
            2 => {
                let slot = &mut self.slots_16s[location.slot];
                if location.index_in_slot >= slot.len() {
                    return;
                }
                let removed = slot.swap_remove(location.index_in_slot);
                let swapped = if location.index_in_slot < slot.len() {
                    Some(slot[location.index_in_slot].id)
                } else {
                    None
                };
                (removed.id, swapped)
            }
            3 => {
                let slot = &mut self.slots_17min[location.slot];
                if location.index_in_slot >= slot.len() {
                    return;
                }
                let removed = slot.swap_remove(location.index_in_slot);
                let swapped = if location.index_in_slot < slot.len() {
                    Some(slot[location.index_in_slot].id)
                } else {
                    None
                };
                (removed.id, swapped)
            }
            _ => return, // Invalid level
        };

        // Update index for the swapped element (if any)
        if let Some(swapped_id) = swapped_id {
            if let Some(loc) = self.index.get_mut(&swapped_id) {
                loc.index_in_slot = location.index_in_slot;
            }
        }

        // Remove from index
        self.index.remove(&removed_id);
    }

    /// Advance by one tick (1ms)
    fn tick(&mut self) {
        self.current_tick += 1;

        // Process Level 0 slot
        let slot_0 = (self.current_tick % LEVEL_0_SLOTS as u64) as usize;
        self.expire_slot(0, slot_0);

        // Cascade from higher levels when we wrap around
        if self.current_tick % LEVEL_1_RESOLUTION_MS == 0 {
            let slot_1 = ((self.current_tick / LEVEL_1_RESOLUTION_MS) % LEVEL_1_SLOTS as u64)
                as usize;
            self.cascade_slot(1, slot_1);
        }

        if self.current_tick % LEVEL_2_RESOLUTION_MS == 0 {
            let slot_2 = ((self.current_tick / LEVEL_2_RESOLUTION_MS) % LEVEL_2_SLOTS as u64)
                as usize;
            self.cascade_slot(2, slot_2);
        }

        if self.current_tick % LEVEL_3_RESOLUTION_MS == 0 {
            let slot_3 = ((self.current_tick / LEVEL_3_RESOLUTION_MS) % LEVEL_3_SLOTS as u64)
                as usize;
            self.cascade_slot(3, slot_3);
        }

        // Check overflow for timers that have moved into range
        self.check_overflow();
    }

    /// Expire all timers in a slot (Level 0 only)
    fn expire_slot(&mut self, level: u8, slot: usize) {
        debug_assert_eq!(level, 0, "Only Level 0 timers should expire directly");

        let timers = &mut self.slots_1ms[slot];

        for entry in timers.drain(..) {
            self.index.remove(&entry.id);
            self.expired.push(entry);
        }
    }

    /// Cascade timers from a higher level slot to lower levels
    fn cascade_slot(&mut self, level: u8, slot: usize) {
        let slots = match level {
            1 => &mut self.slots_256ms,
            2 => &mut self.slots_16s,
            3 => &mut self.slots_17min,
            _ => return,
        };

        let timers = std::mem::take(&mut slots[slot]);

        for entry in timers {
            // Remove from index (will be re-added at new location)
            self.index.remove(&entry.id);

            // Re-insert at lower level
            self.insert_entry(entry);
        }
    }

    /// Check overflow BTreeMap and move timers that are now in range or expired
    fn check_overflow(&mut self) {
        let now = self.current_time();
        let threshold = now + Duration::from_millis(OVERFLOW_THRESHOLD_MS);

        // Collect keys to process (both in-range and already expired)
        let keys_to_process: Vec<Instant> = self
            .overflow
            .range(..=threshold)
            .map(|(k, _)| *k)
            .collect();

        if keys_to_process.is_empty() {
            return;
        }

        // Extract and insert/expire entries
        for key in keys_to_process {
            if let Some(entries) = self.overflow.remove(&key) {
                for entry in entries {
                    // Remove old overflow location from index
                    self.index.remove(&entry.id);
                    // insert_entry will add new location and handle expired timers
                    self.insert_entry(entry);
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
    use std::sync::Arc;
    use std::task::Wake;

    // Helper: Create a dummy waker
    fn dummy_waker() -> Waker {
        struct DummyWaker;

        impl Wake for DummyWaker {
            fn wake(self: Arc<Self>) {}
        }

        Arc::new(DummyWaker).into()
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
        let id2 = wheel.insert(
            start + Duration::from_millis(260 + 10),
            dummy_waker(),
        );

        assert_ne!(id1, id2);

        // First timer should have expired
        assert!(wheel.drain_expired().any(|(id, _)| id == id1));

        // Advance to second timer
        wheel.advance_to(start + Duration::from_millis(270));
        assert!(wheel.drain_expired().any(|(id, _)| id == id2));
    }
}
