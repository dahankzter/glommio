// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Timer ID for O(1) cancellation without hashing.
//!
//! Instead of using a global u64 ID that requires HashMap lookups,
//! this ID provides direct access to the timer's storage slot.

/// Unique identifier for O(1) timer cancellation
///
/// This ID contains the exact location of a timer in the wheel,
/// along with a generation counter to detect reused slots.
///
/// # Zero-Cost Cancellation
///
/// ```text
/// Traditional approach:         Direct ID approach:
/// ┌─────────────────────┐      ┌─────────────────────┐
/// │ u64 timer_id        │      │ TimerId             │
/// └─────────────────────┘      │ - index: u32        │
///          │                   │ - generation: u32   │
///          ▼                   └─────────────────────┘
/// ┌─────────────────────┐               │
/// │ Hash timer_id       │               ▼
/// └─────────────────────┘      ┌─────────────────────┐
///          │                   │ Direct array[index] │
///          ▼                   └─────────────────────┘
/// ┌─────────────────────┐               │
/// │ HashMap lookup      │               ▼
/// └─────────────────────┘      ┌─────────────────────┐
///          │                   │ Gen check (in-cache)│
///          ▼                   └─────────────────────┘
/// ┌─────────────────────┐
/// │ Get location        │      Result: ~0.5µs faster
/// └─────────────────────┘      No cache misses
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerId {
    /// Index into the timer storage
    ///
    /// For inline storage: index into Vec
    /// For wheel storage: encodes (level, slot, index_in_slot)
    pub(crate) index: u32,

    /// Generation counter to detect slot reuse
    ///
    /// When a timer is removed, its slot can be reused for a new timer.
    /// The generation counter ensures we don't accidentally cancel a
    /// different timer that now occupies the same slot.
    pub(crate) generation: u32,
}

impl TimerId {
    /// Create a new timer ID
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Get the index component
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Get the generation component
    #[allow(dead_code)]
    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_id_creation() {
        let id = TimerId::new(42, 1);
        assert_eq!(id.index(), 42);
        assert_eq!(id.generation(), 1);
    }

    #[test]
    fn test_timer_id_equality() {
        let id1 = TimerId::new(42, 1);
        let id2 = TimerId::new(42, 1);
        let id3 = TimerId::new(42, 2); // Different generation
        let id4 = TimerId::new(43, 1); // Different index

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
    }

    #[test]
    fn test_timer_id_clone() {
        let id1 = TimerId::new(100, 5);
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }
}
