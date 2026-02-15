// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Simple task arena for fast allocation
//!
//! This is a prototype arena allocator for tasks. It pre-allocates memory for
//! a fixed number of tasks and serves allocations from a free list.
//!
//! **Prototype Status:**
//! - No recycling yet (allocate-only for measuring best case)
//! - Falls back to heap when full
//! - Not yet optimized for production

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::ptr::NonNull;

scoped_tls::scoped_thread_local!(pub(crate) static TASK_ARENA: TaskArena);

/// Maximum task size we'll allocate in the arena (512 bytes)
const MAX_TASK_SIZE: usize = 512;

/// Number of tasks to pre-allocate
const ARENA_CAPACITY: usize = 10_000;

/// Simple task arena with free-list allocation
pub(crate) struct TaskArena {
    /// Pre-allocated memory block
    memory: NonNull<u8>,
    /// Total capacity in bytes
    capacity: usize,
    /// Next free offset (bump allocator style for prototype)
    next_offset: RefCell<usize>,
    /// Track how many allocations from arena vs heap
    arena_allocs: RefCell<usize>,
    heap_fallback_allocs: RefCell<usize>,
}

impl TaskArena {
    /// Create a new task arena
    pub(crate) fn new() -> Self {
        let capacity = ARENA_CAPACITY * MAX_TASK_SIZE;
        let layout = Layout::from_size_align(capacity, 64)
            .expect("Failed to create arena layout");

        // SAFETY: We allocate a large block upfront and manage it ourselves
        let memory = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("Failed to allocate task arena")
        };

        Self {
            memory,
            capacity,
            next_offset: RefCell::new(0),
            arena_allocs: RefCell::new(0),
            heap_fallback_allocs: RefCell::new(0),
        }
    }

    /// Try to allocate from arena, returns None if full
    ///
    /// SAFETY: Caller must ensure the layout is valid and doesn't exceed MAX_TASK_SIZE
    pub(crate) unsafe fn try_allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
        // For prototype: only handle tasks up to MAX_TASK_SIZE
        if layout.size() > MAX_TASK_SIZE {
            return None;
        }

        let mut offset = self.next_offset.borrow_mut();

        // Align the offset
        let aligned_offset = (*offset + layout.align() - 1) & !(layout.align() - 1);
        let new_offset = aligned_offset + layout.size();

        // Check if we have space
        if new_offset > self.capacity {
            return None;  // Arena full
        }

        // Allocate from arena
        *offset = new_offset;
        *self.arena_allocs.borrow_mut() += 1;

        // SAFETY: We've checked bounds and alignment
        let ptr = self.memory.as_ptr().add(aligned_offset);
        Some(NonNull::new_unchecked(ptr))
    }

    /// Record a heap fallback allocation
    pub(crate) fn record_heap_fallback(&self) {
        *self.heap_fallback_allocs.borrow_mut() += 1;
    }

    /// Get statistics for measuring arena effectiveness
    pub(crate) fn stats(&self) -> ArenaStats {
        ArenaStats {
            arena_allocs: *self.arena_allocs.borrow(),
            heap_fallback_allocs: *self.heap_fallback_allocs.borrow(),
            bytes_used: *self.next_offset.borrow(),
            bytes_capacity: self.capacity,
        }
    }

    /// Reset arena (for benchmarking)
    pub(crate) fn reset(&self) {
        *self.next_offset.borrow_mut() = 0;
        *self.arena_allocs.borrow_mut() = 0;
        *self.heap_fallback_allocs.borrow_mut() = 0;
    }
}

impl Drop for TaskArena {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 64)
            .expect("Failed to create arena layout for deallocation");

        // SAFETY: We allocated this memory in new()
        unsafe {
            dealloc(self.memory.as_ptr(), layout);
        }
    }
}

// SAFETY: TaskArena is only accessed from the thread that owns the executor
unsafe impl Send for TaskArena {}

/// Statistics for measuring arena effectiveness
#[derive(Debug, Clone, Copy)]
pub(crate) struct ArenaStats {
    pub arena_allocs: usize,
    pub heap_fallback_allocs: usize,
    pub bytes_used: usize,
    pub bytes_capacity: usize,
}

impl ArenaStats {
    pub fn arena_hit_rate(&self) -> f64 {
        let total = self.arena_allocs + self.heap_fallback_allocs;
        if total == 0 {
            0.0
        } else {
            (self.arena_allocs as f64 / total as f64) * 100.0
        }
    }

    pub fn utilization(&self) -> f64 {
        (self.bytes_used as f64 / self.bytes_capacity as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_allocation() {
        let arena = TaskArena::new();

        // Allocate a few tasks
        let layout = Layout::from_size_align(256, 8).unwrap();

        unsafe {
            let ptr1 = arena.try_allocate(layout);
            assert!(ptr1.is_some());

            let ptr2 = arena.try_allocate(layout);
            assert!(ptr2.is_some());

            // Check pointers are different
            assert_ne!(ptr1.unwrap().as_ptr(), ptr2.unwrap().as_ptr());
        }

        let stats = arena.stats();
        assert_eq!(stats.arena_allocs, 2);
        assert_eq!(stats.heap_fallback_allocs, 0);
    }

    #[test]
    fn test_arena_full() {
        let arena = TaskArena::new();
        let layout = Layout::from_size_align(MAX_TASK_SIZE, 8).unwrap();

        // Fill the arena
        let mut count = 0;
        unsafe {
            while arena.try_allocate(layout).is_some() {
                count += 1;
            }
        }

        // Should have allocated ARENA_CAPACITY tasks
        assert_eq!(count, ARENA_CAPACITY);

        // Next allocation should fail
        unsafe {
            assert!(arena.try_allocate(layout).is_none());
        }
    }

    #[test]
    fn test_arena_reset() {
        let arena = TaskArena::new();
        let layout = Layout::from_size_align(256, 8).unwrap();

        unsafe {
            arena.try_allocate(layout);
            arena.try_allocate(layout);
        }

        assert_eq!(arena.stats().arena_allocs, 2);

        arena.reset();

        let stats = arena.stats();
        assert_eq!(stats.arena_allocs, 0);
        assert_eq!(stats.bytes_used, 0);
    }
}
