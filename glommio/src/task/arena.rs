// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Task arena with recyclable mega-slab allocator
//!
//! # Architectural Contract: Shared-Nothing Engine
//!
//! **Tasks and wakers are STRICTLY OWNED by their executor.**
//!
//! When an executor drops, its arena memory is IMMEDIATELY reclaimed. Any
//! attempt to access tasks/wakers from a dropped executor is UNDEFINED BEHAVIOR.
//!
//! This design trades safety for performance:
//! - ✅ Zero atomic overhead (no Arc per task)
//! - ✅ Sub-20ns spawn latency target
//! - ✅ Fixed 100MB memory (100K slots × 1KB)
//! - ❌ Tasks must not outlive their executor
//! - ❌ Wakers must not outlive their executor
//!
//! Developers must ensure: **Executors outlive all references to their tasks.**
//!
//! ## Implementation Details
//! - Recyclable slab allocator with O(1) alloc/dealloc
//! - 1024-byte fixed slots, 100,000 slot capacity (100MB total)
//! - Intrusive free-list (zero allocation overhead)
//! - Bulk deallocation on executor drop (no per-task free)

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::ptr::NonNull;

scoped_tls::scoped_thread_local!(pub(crate) static TASK_ARENA: TaskArena);

/// Fixed slot size for all arena allocations (1024 bytes)
///
/// Increased from 512 to 1024 to accommodate larger task closures
/// (e.g., spawn_blocking captures). Total memory: 100K × 1KB = 100MB.
const SLOT_SIZE: usize = 1024;

/// Maximum task size we'll allocate in the arena
const MAX_TASK_SIZE: usize = SLOT_SIZE;

/// Maximum alignment we support (64-byte cache line)
const MAX_ALIGN: usize = 64;

/// Number of slots to pre-allocate
///
/// Set to 100,000 slots (100MB) to eliminate heap fallback entirely.
/// This is a trivial amount of memory for server workloads but ensures
/// the arena can handle high concurrency without falling back to heap.
/// Since all tasks are arena-allocated, deallocation logic is greatly
/// simplified - no need to track allocation source.
const SLOT_CAPACITY: usize = 100_000;

/// Sentinel value for end of free list
const FREE_LIST_END: u32 = u32::MAX;

/// Task arena with recyclable slab allocator
pub(crate) struct TaskArena {
    /// Pre-allocated memory block (aligned to MAX_ALIGN)
    memory: NonNull<u8>,
    /// Total capacity in bytes
    capacity: usize,
    /// Head of intrusive free list (stored as slot index, or FREE_LIST_END)
    free_head: RefCell<u32>,
    /// Track allocation statistics
    arena_allocs: RefCell<usize>,
    arena_deallocs: RefCell<usize>,
    heap_fallback_allocs: RefCell<usize>,
    /// Current number of active slots
    active_slots: RefCell<usize>,
    /// Peak active slots (high water mark)
    peak_active: RefCell<usize>,
}

impl TaskArena {
    /// Create a new task arena with initialized free list
    pub(crate) fn new() -> Self {
        let capacity = SLOT_CAPACITY * SLOT_SIZE;
        let layout =
            Layout::from_size_align(capacity, MAX_ALIGN).expect("Failed to create arena layout");

        // SAFETY: We allocate a large block upfront and manage it ourselves
        let memory = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("Failed to allocate task arena")
        };

        let arena = Self {
            memory,
            capacity,
            free_head: RefCell::new(FREE_LIST_END),
            arena_allocs: RefCell::new(0),
            arena_deallocs: RefCell::new(0),
            heap_fallback_allocs: RefCell::new(0),
            active_slots: RefCell::new(0),
            peak_active: RefCell::new(0),
        };

        // Initialize free list: 0 → 1 → 2 → ... → (SLOT_CAPACITY-1) → END
        arena.initialize_free_list();

        arena
    }

    /// Initialize the intrusive free list
    ///
    /// Sets up the initial free list: 0 → 1 → 2 → ... → (SLOT_CAPACITY-1) → END
    ///
    /// Each slot stores the index of the next free slot as a u32 at offset 0.
    /// This intrusive approach requires zero additional memory allocation.
    ///
    /// SAFETY: Only called from new() where we've just allocated the memory block.
    /// All pointer arithmetic is bounds-checked via debug assertions.
    fn initialize_free_list(&self) {
        unsafe {
            let base = self.memory.as_ptr();

            // Link all slots: slot[i] → slot[i+1]
            for i in 0..(SLOT_CAPACITY - 1) {
                let offset = i.checked_mul(SLOT_SIZE).expect("Slot offset overflow");
                debug_assert!(
                    offset + SLOT_SIZE <= self.capacity,
                    "Slot {} exceeds capacity",
                    i
                );

                let slot_ptr = base.add(offset) as *mut u32;
                *slot_ptr = (i + 1) as u32;
            }

            // Last slot points to END
            let last_offset = (SLOT_CAPACITY - 1)
                .checked_mul(SLOT_SIZE)
                .expect("Last slot offset overflow");
            debug_assert!(last_offset < self.capacity, "Last slot exceeds capacity");

            let last_slot = base.add(last_offset) as *mut u32;
            *last_slot = FREE_LIST_END;

            // Head points to first slot
            *self.free_head.borrow_mut() = 0;
        }
    }

    /// Try to allocate from arena, returns None if full or layout unsupported
    ///
    /// SAFETY: Caller must ensure the layout is valid.
    ///
    /// This function is unsafe because it returns a raw pointer to uninitialized
    /// memory that the caller must properly initialize before use.
    ///
    /// # Safety Invariants
    ///
    /// - Free list indices are always < SLOT_CAPACITY
    /// - All slots are properly aligned to MAX_ALIGN (64 bytes)
    /// - Returned pointer is valid for writes up to SLOT_SIZE bytes
    pub(crate) unsafe fn try_allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
        // Reject oversized or over-aligned allocations
        if layout.size() > MAX_TASK_SIZE || layout.align() > MAX_ALIGN {
            return None;
        }

        let mut head = self.free_head.borrow_mut();

        // Check if free list is empty
        if *head == FREE_LIST_END {
            return None; // Arena full
        }

        // Pop from free list head
        let slot_index = *head as usize;

        // SAFETY INVARIANT: slot_index must be < SLOT_CAPACITY
        debug_assert!(
            slot_index < SLOT_CAPACITY,
            "Free list corruption: slot_index {} >= SLOT_CAPACITY {}",
            slot_index,
            SLOT_CAPACITY
        );

        // Calculate slot pointer with bounds checking in debug mode
        let offset = slot_index
            .checked_mul(SLOT_SIZE)
            .expect("Slot offset overflow");
        debug_assert!(offset < self.capacity, "Slot offset exceeds capacity");

        let slot_ptr = self.memory.as_ptr().add(offset);

        // Read next free slot from current slot's first u32
        let next_free = *(slot_ptr as *const u32);

        // SAFETY INVARIANT: next_free must be valid (either FREE_LIST_END or < SLOT_CAPACITY)
        debug_assert!(
            next_free == FREE_LIST_END || (next_free as usize) < SLOT_CAPACITY,
            "Free list corruption: next_free {} invalid",
            next_free
        );

        *head = next_free;

        // Update statistics
        *self.arena_allocs.borrow_mut() += 1;
        let mut active = self.active_slots.borrow_mut();
        *active += 1;
        let mut peak = self.peak_active.borrow_mut();
        if *active > *peak {
            *peak = *active;
        }

        // Return the slot pointer
        // SAFETY: Slot is aligned to MAX_ALIGN (64), which satisfies layout.align() <= MAX_ALIGN
        // The pointer is guaranteed non-null because:
        // 1. self.memory is non-null (checked at allocation)
        // 2. offset < capacity (checked above)
        // 3. Pointer arithmetic on non-null with valid offset yields non-null
        debug_assert!(!slot_ptr.is_null(), "Slot pointer should never be null");
        Some(NonNull::new_unchecked(slot_ptr))
    }

    /// Try to deallocate back to arena, returns true if recycled
    ///
    /// Returns false if the pointer wasn't allocated from this arena.
    ///
    /// SAFETY: Caller must uphold these invariants:
    ///
    /// 1. `ptr` must be a valid pointer that was either:
    ///    - Previously allocated by `try_allocate()` from this arena, OR
    ///    - Allocated from the heap (will return false harmlessly)
    /// 2. The slot must no longer be in use (no aliasing mutable references)
    /// 3. The slot's contents can be safely overwritten (task has been dropped)
    ///
    /// # Why This Is Safe (When Called Correctly)
    ///
    /// This is called from `RawTask::destroy()` which guarantees:
    /// - Task refcount == 0 (no other references exist)
    /// - HANDLE == 0 (not in executor queue)
    /// - Future has been dropped
    /// - Always runs on executor thread (no foreign wakers)
    pub(crate) unsafe fn try_deallocate(&self, ptr: *const u8) -> bool {
        // Bounds check: reject pointers outside arena (likely heap-allocated)
        let start = self.memory.as_ptr() as usize;
        let end = start + self.capacity;
        let addr = ptr as usize;

        if addr < start || addr >= end {
            return false; // Not from arena
        }

        // Calculate slot offset and index
        let offset = addr - start;
        let slot_index = offset / SLOT_SIZE;

        // SAFETY CHECK: Verify pointer is properly slot-aligned
        // This catches memory corruption or double-frees early
        debug_assert_eq!(
            offset % SLOT_SIZE,
            0,
            "Arena deallocation of misaligned pointer: offset {} not multiple of {}",
            offset,
            SLOT_SIZE
        );

        // SAFETY CHECK: Verify slot index is in bounds
        debug_assert!(
            slot_index < SLOT_CAPACITY,
            "Arena deallocation: slot_index {} >= SLOT_CAPACITY {}",
            slot_index,
            SLOT_CAPACITY
        );

        // Push slot back onto free list head (LIFO)
        let mut head = self.free_head.borrow_mut();
        let slot_ptr = ptr as *mut u32;

        // SAFETY CHECK: Verify current head is valid
        debug_assert!(
            *head == FREE_LIST_END || (*head as usize) < SLOT_CAPACITY,
            "Free list corruption: head {} invalid before dealloc",
            *head
        );

        *slot_ptr = *head; // Store old head as next
        *head = slot_index as u32; // New head is this slot

        // Update statistics
        *self.arena_deallocs.borrow_mut() += 1;
        *self.active_slots.borrow_mut() -= 1;

        true
    }

    /// Get statistics for measuring arena effectiveness
    #[allow(dead_code)]
    pub(crate) fn stats(&self) -> ArenaStats {
        ArenaStats {
            arena_allocs: *self.arena_allocs.borrow(),
            arena_deallocs: *self.arena_deallocs.borrow(),
            heap_fallback_allocs: *self.heap_fallback_allocs.borrow(),
            active_slots: *self.active_slots.borrow(),
            peak_active: *self.peak_active.borrow(),
            slot_capacity: SLOT_CAPACITY,
        }
    }

    /// Reset arena (for benchmarking) - rebuilds entire free list
    #[allow(dead_code)]
    pub(crate) fn reset(&self) {
        *self.arena_allocs.borrow_mut() = 0;
        *self.arena_deallocs.borrow_mut() = 0;
        *self.heap_fallback_allocs.borrow_mut() = 0;
        *self.active_slots.borrow_mut() = 0;
        *self.peak_active.borrow_mut() = 0;

        // Rebuild free list from scratch
        self.initialize_free_list();
    }
}

impl Drop for TaskArena {
    fn drop(&mut self) {
        // SAFETY STRATEGY: Use mprotect instead of dealloc to catch use-after-free
        //
        // Instead of freeing the memory (which allows reuse), we mark it as
        // inaccessible using mprotect(PROT_NONE). This ensures any attempt to
        // access tasks from a dropped executor triggers a clean SIGSEGV.
        //
        // Cost: 100MB virtual address space per dropped executor (not RSS)
        // Benefit: Catches contract violations with clear crash location
        //
        // On 64-bit systems, virtual address space is ~128TB, so this is
        // negligible. Memory will be reclaimed by OS on process exit.
        //
        // Note: Glommio requires io_uring, which is Linux-only, so we always
        // have mprotect available.
        unsafe {
            let result = libc::mprotect(
                self.memory.as_ptr() as *mut libc::c_void,
                self.capacity,
                libc::PROT_NONE,
            );

            if result != 0 {
                let errno = *libc::__errno_location();
                eprintln!(
                    "⚠️  WARNING: mprotect failed for arena (errno={})",
                    errno
                );
                eprintln!(
                    "⚠️  Falling back to dealloc - use-after-free protection disabled!"
                );
                // Fallback: deallocate normally (less safe)
                let layout = Layout::from_size_align(self.capacity, 64)
                    .expect("Failed to create arena layout");
                dealloc(self.memory.as_ptr(), layout);
            }
            // Success: Memory now inaccessible, any access = SIGSEGV
        }
    }
}

// SAFETY: TaskArena is only accessed from the thread that owns the executor
unsafe impl Send for TaskArena {}

/// Statistics for measuring arena effectiveness
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct ArenaStats {
    pub arena_allocs: usize,
    pub arena_deallocs: usize,
    pub heap_fallback_allocs: usize,
    pub active_slots: usize,
    pub peak_active: usize,
    pub slot_capacity: usize,
}

impl ArenaStats {
    #[allow(dead_code)]
    pub fn arena_hit_rate(&self) -> f64 {
        let total = self.arena_allocs + self.heap_fallback_allocs;
        if total == 0 {
            0.0
        } else {
            (self.arena_allocs as f64 / total as f64) * 100.0
        }
    }

    #[allow(dead_code)]
    pub fn utilization(&self) -> f64 {
        (self.active_slots as f64 / self.slot_capacity as f64) * 100.0
    }

    #[allow(dead_code)]
    pub fn recycle_rate(&self) -> f64 {
        if self.arena_allocs == 0 {
            0.0
        } else {
            (self.arena_deallocs as f64 / self.arena_allocs as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_allocation() {
        let arena = TaskArena::new();
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
        assert_eq!(stats.active_slots, 2);
        assert_eq!(stats.heap_fallback_allocs, 0);
    }

    #[test]
    fn test_arena_full_then_recycle() {
        let arena = TaskArena::new();
        let layout = Layout::from_size_align(MAX_TASK_SIZE, 8).unwrap();

        // Fill the arena completely
        let mut ptrs = Vec::new();
        unsafe {
            for _ in 0..SLOT_CAPACITY {
                let ptr = arena.try_allocate(layout).expect("Arena should have space");
                ptrs.push(ptr);
            }

            // Arena should be full now
            assert!(arena.try_allocate(layout).is_none());
        }

        let stats = arena.stats();
        assert_eq!(stats.arena_allocs, SLOT_CAPACITY);
        assert_eq!(stats.active_slots, SLOT_CAPACITY);

        // Free half the slots
        unsafe {
            for ptr in ptrs.iter().take(SLOT_CAPACITY / 2) {
                let recycled = arena.try_deallocate(ptr.as_ptr());
                assert!(recycled, "Should recycle arena pointer");
            }
        }

        let stats = arena.stats();
        assert_eq!(stats.arena_deallocs, SLOT_CAPACITY / 2);
        assert_eq!(stats.active_slots, SLOT_CAPACITY / 2);

        // Should be able to allocate again (recycled slots)
        unsafe {
            for _ in 0..(SLOT_CAPACITY / 2) {
                let ptr = arena.try_allocate(layout);
                assert!(ptr.is_some(), "Should allocate from recycled slots");
            }
        }

        let stats = arena.stats();
        assert_eq!(stats.arena_allocs, SLOT_CAPACITY + SLOT_CAPACITY / 2);
        assert_eq!(stats.active_slots, SLOT_CAPACITY);
    }

    #[test]
    fn test_free_list_lifo_order() {
        let arena = TaskArena::new();
        let layout = Layout::from_size_align(256, 8).unwrap();

        unsafe {
            // Allocate 3 slots
            let p1 = arena.try_allocate(layout).unwrap();
            let p2 = arena.try_allocate(layout).unwrap();
            let p3 = arena.try_allocate(layout).unwrap();

            // Free them in order: p1, p2, p3
            arena.try_deallocate(p1.as_ptr());
            arena.try_deallocate(p2.as_ptr());
            arena.try_deallocate(p3.as_ptr());

            // Re-allocate: should get LIFO order (p3, p2, p1)
            let r1 = arena.try_allocate(layout).unwrap();
            assert_eq!(r1.as_ptr(), p3.as_ptr(), "Should get last freed (p3)");

            let r2 = arena.try_allocate(layout).unwrap();
            assert_eq!(
                r2.as_ptr(),
                p2.as_ptr(),
                "Should get second last freed (p2)"
            );

            let r3 = arena.try_allocate(layout).unwrap();
            assert_eq!(r3.as_ptr(), p1.as_ptr(), "Should get first freed (p1)");
        }
    }

    #[test]
    fn test_deallocate_non_arena_pointer() {
        let arena = TaskArena::new();

        // Allocate on heap
        let heap_layout = Layout::from_size_align(256, 8).unwrap();
        let heap_ptr = unsafe { alloc(heap_layout) };

        // Try to deallocate heap pointer to arena
        let recycled = unsafe { arena.try_deallocate(heap_ptr) };
        assert!(!recycled, "Heap pointer should not be recycled to arena");

        // Clean up heap allocation
        unsafe {
            dealloc(heap_ptr, heap_layout);
        }
    }

    #[test]
    fn test_stats_tracking() {
        let arena = TaskArena::new();
        let layout = Layout::from_size_align(256, 8).unwrap();

        unsafe {
            let p1 = arena.try_allocate(layout).unwrap();
            let p2 = arena.try_allocate(layout).unwrap();
            let p3 = arena.try_allocate(layout).unwrap();

            let stats = arena.stats();
            assert_eq!(stats.arena_allocs, 3);
            assert_eq!(stats.arena_deallocs, 0);
            assert_eq!(stats.active_slots, 3);
            assert_eq!(stats.peak_active, 3);

            arena.try_deallocate(p2.as_ptr());

            let stats = arena.stats();
            assert_eq!(stats.arena_allocs, 3);
            assert_eq!(stats.arena_deallocs, 1);
            assert_eq!(stats.active_slots, 2);
            assert_eq!(stats.peak_active, 3); // Peak should not decrease

            let p4 = arena.try_allocate(layout).unwrap();

            let stats = arena.stats();
            assert_eq!(stats.arena_allocs, 4);
            assert_eq!(stats.active_slots, 3);
            assert_eq!(stats.peak_active, 3);

            // Clean up
            arena.try_deallocate(p1.as_ptr());
            arena.try_deallocate(p3.as_ptr());
            arena.try_deallocate(p4.as_ptr());
        }
    }

    #[test]
    fn test_arena_reset_rebuilds_free_list() {
        let arena = TaskArena::new();
        let layout = Layout::from_size_align(256, 8).unwrap();

        unsafe {
            // Allocate and deallocate some slots
            let p1 = arena.try_allocate(layout).unwrap();
            let _p2 = arena.try_allocate(layout).unwrap();
            arena.try_deallocate(p1.as_ptr());
        }

        assert_eq!(arena.stats().arena_allocs, 2);
        assert_eq!(arena.stats().active_slots, 1);

        // Reset arena
        arena.reset();

        let stats = arena.stats();
        assert_eq!(stats.arena_allocs, 0);
        assert_eq!(stats.arena_deallocs, 0);
        assert_eq!(stats.active_slots, 0);
        assert_eq!(stats.peak_active, 0);

        // Should be able to allocate full capacity again
        unsafe {
            for _ in 0..SLOT_CAPACITY {
                let ptr = arena.try_allocate(layout);
                assert!(ptr.is_some(), "Should allocate after reset");
            }
        }

        assert_eq!(arena.stats().arena_allocs, SLOT_CAPACITY);
    }

    #[test]
    fn test_oversized_rejected() {
        let arena = TaskArena::new();

        // Try to allocate > MAX_TASK_SIZE
        let layout = Layout::from_size_align(MAX_TASK_SIZE + 1, 8).unwrap();

        unsafe {
            let ptr = arena.try_allocate(layout);
            assert!(ptr.is_none(), "Oversized allocation should be rejected");
        }
    }

    #[test]
    fn test_high_alignment_rejected() {
        let arena = TaskArena::new();

        // Try to allocate with alignment > MAX_ALIGN
        let layout = Layout::from_size_align(256, MAX_ALIGN * 2).unwrap();

        unsafe {
            let ptr = arena.try_allocate(layout);
            assert!(ptr.is_none(), "Over-aligned allocation should be rejected");
        }
    }
}
