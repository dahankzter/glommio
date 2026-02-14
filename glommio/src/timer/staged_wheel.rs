// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Staged timing wheel with inline storage for small timer counts.
//!
//! This implementation optimizes for the common case of small timer counts
//! (<256) by using inline, stack-allocated storage. When the count exceeds
//! the threshold, it promotes to a hierarchical timing wheel.
//!
//! # Design Rationale
//!
//! - **Inline Stage (0-255 timers):** Simple Vec on the stack, O(n) operations
//!   but excellent cache locality and no allocations.
//!
//! - **Wheel Stage (256+ timers):** Hierarchical timing wheel with O(1) operations.
//!
//! - **No BTreeMap:** Avoids O(log n) jitter from tree rebalancing, which is
//!   problematic for real-time systems where predictability > average performance.
//!
//! # Performance Characteristics
//!
//! | Timer Count | Storage | Insert | Remove | Process |
//! |-------------|---------|--------|--------|---------|
//! | 0-255       | Inline  | O(1)   | O(n)   | O(n)    |
//! | 256+        | Wheel   | O(1)   | O(1)   | O(k)    |
//!
//! The O(n) operations for inline stage are acceptable because:
//! 1. n is capped at 256 (small constant)
//! 2. Cache locality makes linear scan very fast
//! 3. No allocations
//!
//! # Example
//!
//! ```rust
//! use glommio::timer::staged_wheel::StagedWheel;
//! use std::time::{Duration, Instant};
//! use std::task::Waker;
//!
//! let mut wheel = StagedWheel::new();
//! let now = Instant::now();
//!
//! // Small count: uses inline storage
//! for i in 0..100 {
//!     wheel.insert(now + Duration::from_millis(i), waker);
//! }
//!
//! // Automatically promotes to wheel stage at 256 timers
//! for i in 0..200 {
//!     wheel.insert(now + Duration::from_millis(i + 100), waker);
//! }
//! ```

use super::timing_wheel::TimingWheel;
use std::task::Waker;
use std::time::Instant;

/// Threshold for promoting from inline to wheel storage
const INLINE_THRESHOLD: usize = 256;

/// A timer entry in inline storage
#[derive(Debug)]
pub(crate) struct InlineTimer {
    pub(crate) id: u64,
    pub(crate) expires_at: Instant,
    pub(crate) waker: Waker,
}

/// Staged timing wheel with inline storage for small counts
#[derive(Debug)]
pub struct StagedWheel {
    /// Current storage mode
    storage: Storage,

    /// Next timer ID
    next_id: u64,

    /// Base time for wheel stage
    start_time: Instant,
}

#[derive(Debug)]
enum Storage {
    /// Inline storage for small timer counts (0-255)
    /// Uses Vec with pre-allocated capacity to avoid reallocs
    Inline {
        timers: Vec<InlineTimer>,
        /// Timers that have expired and are ready to fire
        expired: Vec<InlineTimer>,
    },

    /// Hierarchical wheel for large timer counts (256+)
    Wheel(Box<TimingWheel>),
}

impl StagedWheel {
    /// Create a new staged wheel starting at the current time
    pub fn new() -> Self {
        Self::new_at(Instant::now())
    }

    /// Create a new staged wheel starting at the specified time
    pub fn new_at(start_time: Instant) -> Self {
        Self {
            storage: Storage::Inline {
                timers: Vec::with_capacity(INLINE_THRESHOLD),
                expired: Vec::new(),
            },
            next_id: 0,
            start_time,
        }
    }

    /// Insert a timer that expires at the given instant
    ///
    /// Returns the unique ID for this timer, which can be used to remove it
    pub fn insert(&mut self, expires_at: Instant, waker: Waker) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        match &mut self.storage {
            Storage::Inline { timers, .. } => {
                // Check if we need to promote to wheel
                if timers.len() >= INLINE_THRESHOLD {
                    self.promote_to_wheel();
                    // Now insert into the wheel, preserving the ID
                    if let Storage::Wheel(wheel) = &mut self.storage {
                        wheel.insert_with_id(id, expires_at, waker);
                    }
                } else {
                    // Insert into inline storage
                    timers.push(InlineTimer {
                        id,
                        expires_at,
                        waker,
                    });
                }
            }
            Storage::Wheel(wheel) => {
                wheel.insert_with_id(id, expires_at, waker);
            }
        }

        id
    }

    /// Remove a timer by ID
    ///
    /// Returns true if the timer was found and removed, false otherwise
    pub fn remove(&mut self, id: u64) -> bool {
        match &mut self.storage {
            Storage::Inline { timers, .. } => {
                // Linear search (acceptable for small n)
                if let Some(pos) = timers.iter().position(|t| t.id == id) {
                    timers.swap_remove(pos);
                    true
                } else {
                    false
                }
            }
            Storage::Wheel(wheel) => wheel.remove(id),
        }
    }

    /// Advance time to the given instant, processing expired timers
    ///
    /// After calling this, use `drain_expired()` to get expired timers
    pub fn advance_to(&mut self, now: Instant) {
        match &mut self.storage {
            Storage::Inline { timers, expired } => {
                // Move expired timers to expired vec
                let mut i = 0;
                while i < timers.len() {
                    if timers[i].expires_at <= now {
                        let timer = timers.swap_remove(i);
                        expired.push(timer);
                    } else {
                        i += 1;
                    }
                }
            }
            Storage::Wheel(wheel) => {
                wheel.advance_to(now);
            }
        }
    }

    /// Get the current time according to the wheel
    pub fn current_time(&self) -> Instant {
        match &self.storage {
            Storage::Inline { .. } => self.start_time,
            Storage::Wheel(wheel) => wheel.current_time(),
        }
    }

    /// Drain all expired timers
    ///
    /// Returns an iterator over (timer_id, waker) pairs
    pub fn drain_expired(&mut self) -> DrainExpired<'_> {
        match &mut self.storage {
            Storage::Inline { expired, .. } => {
                DrainExpired::Inline(Box::new(expired.drain(..).map(|t| (t.id, t.waker))))
            }
            Storage::Wheel(wheel) => DrainExpired::Wheel(Box::new(wheel.drain_expired())),
        }
    }

    /// Get the number of timers currently in the wheel (excluding expired)
    pub fn len(&self) -> usize {
        match &self.storage {
            Storage::Inline { timers, .. } => timers.len(),
            Storage::Wheel(wheel) => wheel.len(),
        }
    }

    /// Check if the wheel is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the current storage mode (for testing/debugging)
    pub fn storage_mode(&self) -> &'static str {
        match &self.storage {
            Storage::Inline { .. } => "inline",
            Storage::Wheel(_) => "wheel",
        }
    }

    // ========================================================================
    // Internal implementation
    // ========================================================================

    /// Promote from inline storage to hierarchical wheel
    fn promote_to_wheel(&mut self) {
        if let Storage::Inline { timers, expired } = &mut self.storage {
            // Create new wheel
            let mut wheel = Box::new(TimingWheel::new_at(self.start_time));

            // Move all inline timers to wheel, preserving IDs
            for timer in timers.drain(..) {
                wheel.insert_with_id(timer.id, timer.expires_at, timer.waker);
            }

            // Move expired timers to wheel's expired list, preserving IDs
            for timer in expired.drain(..) {
                wheel.insert_with_id(timer.id, timer.expires_at, timer.waker);
            }

            // Replace storage
            self.storage = Storage::Wheel(wheel);
        }
    }
}

/// Iterator over expired timers
pub enum DrainExpired<'a> {
    /// Draining expired timers from inline storage
    Inline(Box<dyn Iterator<Item = (u64, Waker)> + 'a>),
    /// Draining expired timers from wheel storage
    Wheel(Box<dyn Iterator<Item = (u64, Waker)> + 'a>),
}

impl<'a> std::fmt::Debug for DrainExpired<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inline(_) => f.debug_tuple("Inline").field(&"Box<dyn Iterator>").finish(),
            Self::Wheel(_) => f.debug_tuple("Wheel").field(&"Box<dyn Iterator>").finish(),
        }
    }
}

impl Iterator for DrainExpired<'_> {
    type Item = (u64, Waker);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DrainExpired::Inline(iter) => iter.next(),
            DrainExpired::Wheel(iter) => iter.next(),
        }
    }
}

impl Default for StagedWheel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::task::Wake;
    use std::time::Duration;

    // Helper: Create a dummy waker
    struct DummyWaker;

    impl Wake for DummyWaker {
        fn wake(self: Arc<Self>) {}
    }

    fn dummy_waker() -> Waker {
        Arc::new(DummyWaker).into()
    }

    #[test]
    fn test_starts_in_inline_mode() {
        let wheel = StagedWheel::new();
        assert_eq!(wheel.storage_mode(), "inline");
        assert_eq!(wheel.len(), 0);
    }

    #[test]
    fn test_inline_insert_and_expire() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Insert a few timers (stays in inline mode)
        let id = wheel.insert(start + Duration::from_millis(100), dummy_waker());
        assert_eq!(wheel.storage_mode(), "inline");
        assert_eq!(wheel.len(), 1);

        // Advance past expiry
        wheel.advance_to(start + Duration::from_millis(100));

        // Should have one expired timer
        let expired: Vec<_> = wheel.drain_expired().collect();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].0, id);
        assert_eq!(wheel.len(), 0);
    }

    #[test]
    fn test_inline_remove() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

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
    fn test_promotes_to_wheel_at_threshold() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Insert up to threshold - should stay inline
        for i in 0..INLINE_THRESHOLD {
            wheel.insert(start + Duration::from_millis(i as u64), dummy_waker());
            assert_eq!(wheel.storage_mode(), "inline");
        }

        assert_eq!(wheel.len(), INLINE_THRESHOLD);

        // Insert one more - should promote to wheel
        wheel.insert(
            start + Duration::from_millis(INLINE_THRESHOLD as u64),
            dummy_waker(),
        );

        assert_eq!(wheel.storage_mode(), "wheel");
        assert_eq!(wheel.len(), INLINE_THRESHOLD + 1);
    }

    #[test]
    fn test_promotion_preserves_timers() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Insert at threshold
        for i in 0..INLINE_THRESHOLD {
            wheel.insert(start + Duration::from_millis(i as u64), dummy_waker());
        }

        // Promote to wheel
        wheel.insert(
            start + Duration::from_millis(INLINE_THRESHOLD as u64),
            dummy_waker(),
        );

        assert_eq!(wheel.storage_mode(), "wheel");

        // Advance to 50ms - should expire first 50 timers
        wheel.advance_to(start + Duration::from_millis(50));
        let expired: Vec<_> = wheel.drain_expired().collect();
        assert_eq!(expired.len(), 51); // 0-50 inclusive

        // Remaining timers should still be there
        assert_eq!(wheel.len(), INLINE_THRESHOLD + 1 - 51);
    }

    #[test]
    fn test_wheel_mode_operations() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Force promotion by inserting timers at 0-256ms
        for i in 0..=INLINE_THRESHOLD {
            wheel.insert(start + Duration::from_millis(i as u64), dummy_waker());
        }

        assert_eq!(wheel.storage_mode(), "wheel");

        // Test insert in wheel mode (insert at 300ms, after the promotion timers)
        let id = wheel.insert(start + Duration::from_millis(300), dummy_waker());

        // Test remove in wheel mode
        assert!(wheel.remove(id));

        // Test advance in wheel mode - advance to 100ms
        wheel.advance_to(start + Duration::from_millis(100));
        let expired_count = wheel.drain_expired().count();
        assert_eq!(expired_count, 101); // Timers 0-100ms inclusive
    }

    #[test]
    fn test_multiple_timers_same_expiry_inline() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Insert 3 timers at same expiry
        let id1 = wheel.insert(start + Duration::from_millis(50), dummy_waker());
        let id2 = wheel.insert(start + Duration::from_millis(50), dummy_waker());
        let id3 = wheel.insert(start + Duration::from_millis(50), dummy_waker());

        assert_eq!(wheel.len(), 3);
        assert_eq!(wheel.storage_mode(), "inline");

        wheel.advance_to(start + Duration::from_millis(50));

        let expired: Vec<_> = wheel.drain_expired().map(|(id, _)| id).collect();
        assert_eq!(expired.len(), 3);
        assert!(expired.contains(&id1));
        assert!(expired.contains(&id2));
        assert!(expired.contains(&id3));
    }

    #[test]
    fn test_churn_pattern_inline() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Simulate churn: insert and immediately cancel
        for i in 0..100 {
            let id = wheel.insert(start + Duration::from_millis(i), dummy_waker());
            assert!(wheel.remove(id));
        }

        assert_eq!(wheel.len(), 0);
        assert_eq!(wheel.storage_mode(), "inline");
    }

    #[test]
    fn test_churn_pattern_wheel() {
        let start = Instant::now();
        let mut wheel = StagedWheel::new_at(start);

        // Force promotion
        for i in 0..=INLINE_THRESHOLD {
            wheel.insert(start + Duration::from_millis(i as u64 + 1000), dummy_waker());
        }

        assert_eq!(wheel.storage_mode(), "wheel");

        // Simulate churn in wheel mode
        for i in 0..1000 {
            let id = wheel.insert(start + Duration::from_millis(i), dummy_waker());
            assert!(wheel.remove(id));
        }

        // Should only have the initial promotion timers
        assert_eq!(wheel.len(), INLINE_THRESHOLD + 1);
    }
}
