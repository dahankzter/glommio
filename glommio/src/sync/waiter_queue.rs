//! Unless explicitly stated otherwise all files in this repository are licensed
//! under the MIT/Apache-2.0 License, at your convenience
//!
//! A FIFO of suspended callers, kept in a slab and linked by index.

use std::task::Waker;

/// Names a queued waiter.
///
/// The generation makes a key stale once its waiter has left the queue, so a
/// future that was woken and removed cannot later remove whoever reused its
/// slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WaiterKey {
    index: u32,
    generation: u32,
}

#[derive(Debug)]
enum Slot<T> {
    Occupied {
        generation: u32,
        data: T,
        waker: Option<Waker>,
        prev: Option<u32>,
        next: Option<u32>,
    },
    Vacant {
        generation: u32,
        next_free: Option<u32>,
    },
}

/// Suspended callers in arrival order, with removal from anywhere in O(1).
#[derive(Debug)]
pub(crate) struct WaiterQueue<T> {
    slots: Vec<Slot<T>>,
    free: Option<u32>,
    head: Option<u32>,
    tail: Option<u32>,
    len: usize,
}

impl<T> Default for WaiterQueue<T> {
    fn default() -> Self {
        WaiterQueue {
            slots: Vec::new(),
            free: None,
            head: None,
            tail: None,
            len: 0,
        }
    }
}

impl<T> WaiterQueue<T> {
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Queues a waiter at the back.
    pub(crate) fn push_back(&mut self, data: T, waker: Waker) -> WaiterKey {
        let prev = self.tail;
        let (index, generation) = match self.free {
            Some(index) => {
                let Slot::Vacant {
                    generation,
                    next_free,
                } = self.slots[index as usize]
                else {
                    unreachable!("the free list names only vacant slots");
                };
                self.free = next_free;
                (index, generation)
            }
            None => {
                let index =
                    u32::try_from(self.slots.len()).expect("more than u32::MAX queued waiters");
                self.slots.push(Slot::Vacant {
                    generation: 0,
                    next_free: None,
                });
                (index, 0)
            }
        };
        self.slots[index as usize] = Slot::Occupied {
            generation,
            data,
            waker: Some(waker),
            prev,
            next: None,
        };
        match prev {
            Some(prev) => self.set_next(prev, Some(index)),
            None => self.head = Some(index),
        }
        self.tail = Some(index);
        self.len += 1;
        WaiterKey { index, generation }
    }

    /// Whether `key` still names a queued waiter.
    pub(crate) fn contains(&self, key: WaiterKey) -> bool {
        matches!(
            self.slots.get(key.index as usize),
            Some(Slot::Occupied { generation, .. }) if *generation == key.generation
        )
    }

    /// Replaces the waker of a queued waiter. Returns false if `key` is stale.
    pub(crate) fn set_waker(&mut self, key: WaiterKey, new: Waker) -> bool {
        match self.slots.get_mut(key.index as usize) {
            Some(Slot::Occupied {
                generation, waker, ..
            }) if *generation == key.generation => {
                *waker = Some(new);
                true
            }
            _ => false,
        }
    }

    /// Removes a waiter from anywhere in the queue, if `key` is not stale.
    pub(crate) fn remove(&mut self, key: WaiterKey) -> Option<(T, Option<Waker>)> {
        if !self.contains(key) {
            return None;
        }
        let vacant = Slot::Vacant {
            generation: key.generation.wrapping_add(1),
            next_free: self.free,
        };
        let Slot::Occupied {
            data,
            waker,
            prev,
            next,
            ..
        } = std::mem::replace(&mut self.slots[key.index as usize], vacant)
        else {
            unreachable!("contains checked the slot is occupied");
        };
        self.free = Some(key.index);
        match prev {
            Some(prev) => self.set_next(prev, next),
            None => self.head = next,
        }
        match next {
            Some(next) => self.set_prev(next, prev),
            None => self.tail = prev,
        }
        self.len -= 1;
        Some((data, waker))
    }

    /// The waiter at the front, if any.
    pub(crate) fn front(&self) -> Option<WaiterKey> {
        self.head.map(|index| self.key_at(index))
    }

    /// The waiter queued after `key`, if `key` is current and has one.
    pub(crate) fn next(&self, key: WaiterKey) -> Option<WaiterKey> {
        match self.slots.get(key.index as usize) {
            Some(Slot::Occupied {
                generation, next, ..
            }) if *generation == key.generation => next.map(|index| self.key_at(index)),
            _ => None,
        }
    }

    /// The data of a queued waiter, if `key` is current.
    pub(crate) fn get(&self, key: WaiterKey) -> Option<&T> {
        match self.slots.get(key.index as usize) {
            Some(Slot::Occupied {
                generation, data, ..
            }) if *generation == key.generation => Some(data),
            _ => None,
        }
    }

    fn key_at(&self, index: u32) -> WaiterKey {
        match self.slots[index as usize] {
            Slot::Occupied { generation, .. } => WaiterKey { index, generation },
            Slot::Vacant { .. } => unreachable!("links name only occupied slots"),
        }
    }

    fn set_next(&mut self, index: u32, to: Option<u32>) {
        match &mut self.slots[index as usize] {
            Slot::Occupied { next, .. } => *next = to,
            Slot::Vacant { .. } => unreachable!("links name only occupied slots"),
        }
    }

    fn set_prev(&mut self, index: u32, to: Option<u32>) {
        match &mut self.slots[index as usize] {
            Slot::Occupied { prev, .. } => *prev = to,
            Slot::Vacant { .. } => unreachable!("links name only occupied slots"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(q: &WaiterQueue<u32>) -> Vec<u32> {
        let mut out = Vec::new();
        let mut at = q.front();
        while let Some(key) = at {
            out.push(*q.get(key).unwrap());
            at = q.next(key);
        }
        out
    }

    #[test]
    fn keeps_arrival_order_through_removals_anywhere() {
        let mut q = WaiterQueue::default();
        let keys: Vec<_> = (0..5)
            .map(|i| q.push_back(i, Waker::noop().clone()))
            .collect();
        q.remove(keys[2]).unwrap();
        q.remove(keys[0]).unwrap();
        q.remove(keys[4]).unwrap();
        assert_eq!(order(&q), [1, 3]);
        assert_eq!(q.len(), 2);
        q.push_back(5, Waker::noop().clone());
        assert_eq!(order(&q), [1, 3, 5]);
    }

    #[test]
    fn a_stale_key_cannot_remove_the_slot_reused_after_it() {
        let mut q = WaiterQueue::default();
        let first = q.push_back(1, Waker::noop().clone());
        q.remove(first).unwrap();
        let second = q.push_back(2, Waker::noop().clone());
        assert_eq!(first.index, second.index);
        assert!(!q.contains(first));
        assert!(q.remove(first).is_none());
        assert!(!q.set_waker(first, Waker::noop().clone()));
        assert_eq!(order(&q), [2]);
    }

    #[test]
    fn empties_completely_and_reuses_every_slot() {
        let mut q = WaiterQueue::default();
        let keys: Vec<_> = (0..64)
            .map(|i| q.push_back(i, Waker::noop().clone()))
            .collect();
        for key in keys {
            q.remove(key).unwrap();
        }
        assert!(q.is_empty());
        assert_eq!(q.front(), None);
        for i in 0..64 {
            q.push_back(i, Waker::noop().clone());
        }
        assert_eq!(q.slots.len(), 64);
        assert_eq!(order(&q), (0..64).collect::<Vec<_>>());
    }
}
