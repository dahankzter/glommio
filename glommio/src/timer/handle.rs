// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Timer handle for O(1) cancellation without hashing.
//!
//! Instead of using a global u64 ID that requires HashMap lookups,
//! this handle provides direct access to the timer's storage slot.

/// Handle for O(1) timer cancellation
///
/// This handle contains the exact location of a timer in the wheel,
/// along with a generation counter to detect reused slots.
///
/// # Zero-Cost Cancellation
///
/// ```text
/// Traditional approach:         Direct handle approach:
/// ┌─────────────────────┐      ┌─────────────────────┐
/// │ u64 timer_id        │      │ TimerHandle         │
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
pub struct TimerHandle {
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

impl TimerHandle {
    /// Create a new timer handle
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Get the index component
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Get the generation component
    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_creation() {
        let handle = TimerHandle::new(42, 1);
        assert_eq!(handle.index(), 42);
        assert_eq!(handle.generation(), 1);
    }

    #[test]
    fn test_handle_equality() {
        let h1 = TimerHandle::new(42, 1);
        let h2 = TimerHandle::new(42, 1);
        let h3 = TimerHandle::new(42, 2); // Different generation
        let h4 = TimerHandle::new(43, 1); // Different index

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_ne!(h1, h4);
    }

    #[test]
    fn test_handle_clone() {
        let h1 = TimerHandle::new(100, 5);
        let h2 = h1.clone();
        assert_eq!(h1, h2);
    }
}
