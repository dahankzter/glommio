//! Unless explicitly stated otherwise all files in this repository are licensed
//! under the MIT/Apache-2.0 License, at your convenience
//!
//! A FIFO of suspended callers, kept in arrival order in a ring buffer.

use std::{collections::VecDeque, task::Waker};

/// Names a queued waiter by its place in arrival order.
///
/// Tickets only grow, so a ticket whose waiter has left the queue can never
/// name a later waiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WaiterKey(u64);

#[derive(Debug)]
struct Entry<T> {
    data: T,
    waker: Waker,
}

/// Suspended callers in arrival order, with removal from anywhere in O(1).
///
/// A waiter removed from the middle leaves a vacant entry behind, reclaimed
/// once everything in front of it has gone.
#[derive(Debug)]
pub(crate) struct WaiterQueue<T> {
    entries: VecDeque<Option<Entry<T>>>,
    head: u64,
    live: usize,
}

impl<T> Default for WaiterQueue<T> {
    fn default() -> Self {
        WaiterQueue {
            entries: VecDeque::new(),
            head: 0,
            live: 0,
        }
    }
}

impl<T> WaiterQueue<T> {
    pub(crate) fn is_empty(&self) -> bool {
        self.live == 0
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.live
    }

    /// Queues a waiter at the back.
    pub(crate) fn push_back(&mut self, data: T, waker: Waker) -> WaiterKey {
        let key = WaiterKey(self.head + self.entries.len() as u64);
        self.entries.push_back(Some(Entry { data, waker }));
        self.live += 1;
        key
    }

    fn slot(&self, key: WaiterKey) -> Option<usize> {
        let offset = key.0.checked_sub(self.head)?;
        usize::try_from(offset)
            .ok()
            .filter(|&i| i < self.entries.len())
    }

    /// Whether `key` still names a queued waiter.
    #[cfg(test)]
    pub(crate) fn contains(&self, key: WaiterKey) -> bool {
        self.get(key).is_some()
    }

    /// Replaces the waker of a queued waiter. Returns false if `key` is stale.
    pub(crate) fn set_waker(&mut self, key: WaiterKey, new: Waker) -> bool {
        match self.slot(key).and_then(|i| self.entries[i].as_mut()) {
            Some(entry) => {
                entry.waker = new;
                true
            }
            None => false,
        }
    }

    /// Removes a waiter from anywhere in the queue, if `key` is not stale.
    pub(crate) fn remove(&mut self, key: WaiterKey) -> Option<(T, Option<Waker>)> {
        let i = self.slot(key)?;
        let entry = self.entries[i].take()?;
        self.live -= 1;
        while let Some(None) = self.entries.front() {
            self.entries.pop_front();
            self.head += 1;
        }
        while let Some(None) = self.entries.back() {
            self.entries.pop_back();
        }
        Some((entry.data, Some(entry.waker)))
    }

    /// The waiter at the front, if any.
    pub(crate) fn front(&self) -> Option<WaiterKey> {
        self.live_from(0)
    }

    /// The waiter queued after `key`, if it has one.
    pub(crate) fn next(&self, key: WaiterKey) -> Option<WaiterKey> {
        let after = key.0.checked_add(1)?;
        let start = after.saturating_sub(self.head);
        self.live_from(usize::try_from(start).ok()?)
    }

    /// The data of a queued waiter, if `key` is current.
    pub(crate) fn get(&self, key: WaiterKey) -> Option<&T> {
        let i = self.slot(key)?;
        self.entries[i].as_ref().map(|entry| &entry.data)
    }

    fn live_from(&self, start: usize) -> Option<WaiterKey> {
        if start >= self.entries.len() {
            return None;
        }
        self.entries
            .range(start..)
            .position(Option::is_some)
            .map(|offset| WaiterKey(self.head + (start + offset) as u64))
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
    fn a_stale_key_cannot_remove_a_later_waiter() {
        let mut q = WaiterQueue::default();
        let first = q.push_back(1, Waker::noop().clone());
        q.remove(first).unwrap();
        let second = q.push_back(2, Waker::noop().clone());
        assert_ne!(first, second);
        assert!(!q.contains(first));
        assert!(q.remove(first).is_none());
        assert!(!q.set_waker(first, Waker::noop().clone()));
        assert_eq!(order(&q), [2]);
    }

    #[test]
    fn vacancies_are_reclaimed_once_nothing_is_in_front() {
        let mut q = WaiterQueue::default();
        let keys: Vec<_> = (0..64)
            .map(|i| q.push_back(i, Waker::noop().clone()))
            .collect();
        for key in &keys[1..] {
            q.remove(*key).unwrap();
        }
        assert_eq!(q.entries.len(), 1);
        q.remove(keys[0]).unwrap();
        assert!(q.is_empty());
        assert!(q.entries.is_empty());
        assert_eq!(q.front(), None);
    }

    #[test]
    fn next_skips_a_key_removed_during_the_walk() {
        let mut q = WaiterQueue::default();
        let keys: Vec<_> = (0..4)
            .map(|i| q.push_back(i, Waker::noop().clone()))
            .collect();
        q.remove(keys[0]).unwrap();
        assert_eq!(q.next(keys[0]), Some(keys[1]));
        q.remove(keys[2]).unwrap();
        assert_eq!(q.next(keys[1]), Some(keys[3]));
    }
}
