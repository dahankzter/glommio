// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Thread-local free lists for task allocation.
//!
//! Task allocation is a hot path: every `spawn` allocates and every completed
//! task frees. Tasks are also short-lived and highly uniform in size, so the
//! same few block sizes are requested over and over. This module keeps a small
//! per-thread cache of recently freed blocks, turning the common case into a
//! pointer pop instead of a call into the system allocator.
//!
//! # Design constraints
//!
//! This deliberately does *not* replace the heap, it sits in front of it:
//!
//! - **Always falls back.** Blocks larger than [`MAX_CACHED_SIZE`] or aligned
//!   more strictly than [`CLASS_ALIGN`] go straight to the system allocator, so
//!   a task of any size or alignment remains allocatable.
//! - **Bounded.** Each size class caches at most [`MAX_BLOCKS_PER_CLASS`]
//!   blocks; beyond that, frees return memory to the system allocator. Worst
//!   case is ~2MB per thread, and only if the thread actually reached that
//!   many live tasks.
//! - **Grows on demand.** Nothing is preallocated, so a thread that spawns
//!   nothing costs nothing.
//! - **Independent of executor lifetime.** The cache belongs to the thread, not
//!   to a `LocalExecutor`. A task freed after its executor has been dropped
//!   still lands somewhere valid, and blocks are returned to the system
//!   allocator when the thread exits.
//!
//! Blocks are interchangeable between these lists and the system allocator
//! because every cached block is allocated and freed under the same canonical
//! layout for its size class, regardless of the layout originally requested.

use std::alloc::{alloc, dealloc, Layout};

/// Size classes, in bytes. Task layouts cluster well below the top of this
/// range; anything above it falls back to the system allocator.
const CLASS_SIZES: [usize; 5] = [64, 128, 256, 512, 1024];

/// Number of size classes.
const NUM_CLASSES: usize = CLASS_SIZES.len();

/// Largest block size served from the cache.
const MAX_CACHED_SIZE: usize = 1024;

/// Alignment every cached block is allocated with. Layouts needing more than
/// this are not cached. Task layouts are 8- or 16-byte aligned in practice.
const CLASS_ALIGN: usize = 16;

/// Maximum blocks retained per size class before frees go back to the heap.
/// Bounds per-thread footprint at roughly 2MB across all classes.
const MAX_BLOCKS_PER_CLASS: usize = 1024;

/// Maps an allocation size to its size class index.
///
/// Returns the smallest class whose size is >= `size`. Callers must have
/// already established `size <= MAX_CACHED_SIZE`.
#[inline(always)]
fn class_index(size: usize) -> usize {
    debug_assert!(size <= MAX_CACHED_SIZE);
    if size <= CLASS_SIZES[0] {
        0
    } else {
        // Smallest power of two >= size, expressed as an index offset from 64.
        (usize::BITS - (size - 1).leading_zeros()) as usize - 6
    }
}

/// The canonical layout used for every block in a size class.
///
/// Allocation and deallocation both go through this, so a block can be handed
/// between the free list and the system allocator safely.
#[inline(always)]
fn class_layout(class: usize) -> Layout {
    debug_assert!(class < NUM_CLASSES);
    // SAFETY: CLASS_ALIGN is a non-zero power of two and every entry in
    // CLASS_SIZES is a multiple of it, so size rounded up to align cannot
    // overflow isize::MAX.
    unsafe { Layout::from_size_align_unchecked(CLASS_SIZES[class], CLASS_ALIGN) }
}

/// Per-thread cache of freed task blocks.
///
/// Each class is a singly linked LIFO stack. The link pointer is stored in the
/// first word of each free block, which is sound because every class is at
/// least 64 bytes and a free block holds no live data. LIFO ordering is
/// deliberate: the most recently freed block is the most likely to still be
/// cache-warm.
struct FreeLists {
    heads: [*mut u8; NUM_CLASSES],
    counts: [usize; NUM_CLASSES],
}

impl FreeLists {
    const fn new() -> Self {
        Self {
            heads: [std::ptr::null_mut(); NUM_CLASSES],
            counts: [0; NUM_CLASSES],
        }
    }

    /// Pops a block from `class`, or returns null if the class is empty.
    #[inline(always)]
    fn pop(&mut self, class: usize) -> *mut u8 {
        let head = self.heads[class];
        if !head.is_null() {
            // SAFETY: `head` came from this list, so it is a block of at least
            // CLASS_SIZES[class] >= 64 bytes whose first word holds the link to
            // the next free block.
            self.heads[class] = unsafe { *(head as *mut *mut u8) };
            self.counts[class] -= 1;
        }
        head
    }

    /// Pushes a block onto `class`. Returns false if the class is full and the
    /// caller should return the block to the system allocator instead.
    #[inline(always)]
    fn push(&mut self, class: usize, ptr: *mut u8) -> bool {
        if self.counts[class] >= MAX_BLOCKS_PER_CLASS {
            return false;
        }
        // SAFETY: `ptr` is a block of at least CLASS_SIZES[class] >= 64 bytes
        // that the caller has finished with, so writing the link into its first
        // word cannot clobber live data.
        unsafe { *(ptr as *mut *mut u8) = self.heads[class] };
        self.heads[class] = ptr;
        self.counts[class] += 1;
        true
    }
}

impl Drop for FreeLists {
    fn drop(&mut self) {
        // Return every cached block to the system allocator on thread exit,
        // so a thread that spawned tasks does not leak its cache.
        for class in 0..NUM_CLASSES {
            let layout = class_layout(class);
            let mut head = self.heads[class];
            while !head.is_null() {
                // SAFETY: every block in this list was allocated with exactly
                // `layout`, and holds its successor in the first word.
                unsafe {
                    let next = *(head as *mut *mut u8);
                    dealloc(head, layout);
                    head = next;
                }
            }
            self.heads[class] = std::ptr::null_mut();
            self.counts[class] = 0;
        }
    }
}

thread_local! {
    static FREE_LISTS: std::cell::UnsafeCell<FreeLists> =
        const { std::cell::UnsafeCell::new(FreeLists::new()) };
}

/// Runs `f` against this thread's free lists, or returns `None` if thread-local
/// storage is unavailable (during or after TLS destruction at thread exit).
#[inline(always)]
fn with_lists<R>(f: impl FnOnce(&mut FreeLists) -> R) -> Option<R> {
    FREE_LISTS
        .try_with(|cell| {
            // SAFETY: the cache is thread-local and the closure below neither
            // allocates tasks nor re-enters this function, so no second
            // reference to the same FreeLists can exist while this one is live.
            f(unsafe { &mut *cell.get() })
        })
        .ok()
}

/// Allocates a block for a task of `layout`.
///
/// Returns null on allocation failure, matching [`std::alloc::alloc`]. Callers
/// must pass the same `layout` to [`dealloc_task`] when freeing.
#[inline]
pub(crate) fn alloc_task(layout: Layout) -> *mut u8 {
    if layout.size() > MAX_CACHED_SIZE || layout.align() > CLASS_ALIGN {
        // SAFETY: caller guarantees a non-zero-sized layout, as task layouts
        // always include a Header.
        return unsafe { alloc(layout) };
    }

    let class = class_index(layout.size());
    if let Some(ptr) = with_lists(|lists| lists.pop(class)) {
        if !ptr.is_null() {
            return ptr;
        }
    }

    // Cache miss, or TLS unavailable: allocate under the class layout so the
    // block can be recycled by a later free.
    // SAFETY: class_layout yields a valid non-zero-sized layout.
    unsafe { alloc(class_layout(class)) }
}

/// Frees a block previously returned by [`alloc_task`].
///
/// # Safety
///
/// `ptr` must have come from [`alloc_task`] called with an identical `layout`,
/// and must not be used afterwards.
#[inline]
pub(crate) unsafe fn dealloc_task(ptr: *mut u8, layout: Layout) {
    if layout.size() > MAX_CACHED_SIZE || layout.align() > CLASS_ALIGN {
        // SAFETY: mirrors the fallback branch in alloc_task, so `layout`
        // matches the one this block was allocated with.
        unsafe { dealloc(ptr, layout) };
        return;
    }

    let class = class_index(layout.size());
    if with_lists(|lists| lists.push(class, ptr)) == Some(true) {
        return;
    }

    // Class full, or TLS unavailable: hand the block back. The class layout is
    // the one alloc_task used.
    // SAFETY: the block was allocated with exactly class_layout(class).
    unsafe { dealloc(ptr, class_layout(class)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_index_picks_smallest_fitting_class() {
        assert_eq!(class_index(1), 0);
        assert_eq!(class_index(64), 0);
        assert_eq!(class_index(65), 1);
        assert_eq!(class_index(128), 1);
        assert_eq!(class_index(129), 2);
        assert_eq!(class_index(256), 2);
        assert_eq!(class_index(257), 3);
        assert_eq!(class_index(512), 3);
        assert_eq!(class_index(513), 4);
        assert_eq!(class_index(1024), 4);
    }

    #[test]
    fn every_class_fits_its_sizes() {
        for (i, &size) in CLASS_SIZES.iter().enumerate() {
            assert_eq!(class_index(size), i);
            assert!(class_layout(i).size() >= size);
            // A free block must be able to hold the link pointer.
            assert!(class_layout(i).size() >= std::mem::size_of::<*mut u8>());
        }
    }

    #[test]
    fn roundtrip_reuses_blocks() {
        let layout = Layout::from_size_align(200, 8).unwrap();
        let first = alloc_task(layout);
        assert!(!first.is_null());
        unsafe { dealloc_task(first, layout) };

        // The block just freed should come straight back (LIFO).
        let second = alloc_task(layout);
        assert_eq!(first, second);
        unsafe { dealloc_task(second, layout) };
    }

    #[test]
    fn oversized_and_overaligned_still_allocate() {
        for layout in [
            Layout::from_size_align(MAX_CACHED_SIZE + 1, 8).unwrap(),
            Layout::from_size_align(64 * 1024, 8).unwrap(),
            Layout::from_size_align(64, CLASS_ALIGN * 4).unwrap(),
        ] {
            let ptr = alloc_task(layout);
            assert!(!ptr.is_null());
            unsafe {
                ptr.write_bytes(0xAB, layout.size());
                dealloc_task(ptr, layout);
            }
        }
    }

    #[test]
    fn cache_is_bounded() {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let class = class_index(layout.size());

        let blocks: Vec<*mut u8> = (0..MAX_BLOCKS_PER_CLASS + 100)
            .map(|_| alloc_task(layout))
            .collect();
        for b in &blocks {
            unsafe { dealloc_task(*b, layout) };
        }

        let count = with_lists(|lists| lists.counts[class]).unwrap();
        assert!(
            count <= MAX_BLOCKS_PER_CLASS,
            "cached {count} blocks, cap is {MAX_BLOCKS_PER_CLASS}"
        );
    }

    #[test]
    fn many_distinct_sizes_roundtrip() {
        let mut live = Vec::new();
        for size in 1..=MAX_CACHED_SIZE {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = alloc_task(layout);
            assert!(!ptr.is_null());
            // Writing the full requested size must stay in bounds.
            unsafe { ptr.write_bytes(0xCD, size) };
            live.push((ptr, layout));
        }
        for (ptr, layout) in live {
            unsafe { dealloc_task(ptr, layout) };
        }
    }
}
