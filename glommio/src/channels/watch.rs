//! A channel that keeps only the latest value.
//!
//! Every receiver sees the most recent value rather than a queue of them, and
//! a receiver that falls behind simply skips what it missed. Configuration
//! that changes occasionally, and the current state of something a per-core
//! task must react to, are the shapes this fits.
//!
//! Both halves stay on the executor that created them.
//!
//! # Examples
//!
//! ```
//! use glommio::{channels::watch::watch, LocalExecutor};
//!
//! let ex = LocalExecutor::default();
//! ex.run(async {
//!     let (sender, mut receiver) = watch(0);
//!     sender.send(1).unwrap();
//!     receiver.changed().await.unwrap();
//!     assert_eq!(*receiver.borrow(), 1);
//! });
//! ```

use crate::{error::ResourceType, wakers::WakerList, GlommioError};
use futures_lite::Stream;
use std::{
    cell::{Ref, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

#[derive(Debug)]
struct Inner<T> {
    value: T,
    /// Bumped on every send. A receiver compares it against what it last saw,
    /// which is what lets a slow receiver skip values instead of queueing
    /// them.
    version: u64,
    wakers: WakerList,
    sender_gone: bool,
    receivers: usize,
}

impl<T> Inner<T> {
    /// Takes the obligation; the caller wakes it once it has released the
    /// borrow, so a woken poller does not immediately block on it.
    fn take_wakers(&mut self) -> crate::wakers::PendingWakes {
        self.wakers.take()
    }
}

/// Publishes new values. There is exactly one.
#[derive(Debug)]
pub struct Sender<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

/// Observes the latest value. Clone it to add another observer.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: Rc<RefCell<Inner<T>>>,
    /// The version this receiver has already been told about.
    seen: u64,
}

/// Creates a watch channel holding `initial`, returning its two halves.
pub fn watch<T>(initial: T) -> (Sender<T>, Receiver<T>) {
    let inner = Rc::new(RefCell::new(Inner {
        value: initial,
        version: 0,
        wakers: WakerList::new(),
        sender_gone: false,
        receivers: 1,
    }));

    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner, seen: 0 },
    )
}

impl<T> Sender<T> {
    /// Replaces the value and wakes every receiver.
    ///
    /// # Errors
    ///
    /// If every receiver has been dropped there is nobody to tell, so the
    /// value is handed back inside [`GlommioError::Closed`].
    pub fn send(&self, value: T) -> Result<(), GlommioError<T>> {
        let mut inner = self.inner.borrow_mut();
        if inner.receivers == 0 {
            return Err(GlommioError::Closed(ResourceType::Channel(value)));
        }

        inner.value = value;
        inner.version += 1;
        let pending = inner.take_wakers();
        drop(inner);
        pending.wake();
        Ok(())
    }

    /// Borrows the current value without changing it.
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref::map(self.inner.borrow(), |inner| &inner.value)
    }

    /// Returns whether every receiver has gone away.
    pub fn is_closed(&self) -> bool {
        self.inner.borrow().receivers == 0
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.sender_gone = true;
        let pending = inner.take_wakers();
        drop(inner);
        pending.wake();
    }
}

impl<T> Receiver<T> {
    /// Borrows the latest value.
    ///
    /// This does not mark the value as seen: only [`changed`](Self::changed)
    /// does that.
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref::map(self.inner.borrow(), |inner| &inner.value)
    }

    /// Waits until a value newer than the last one this receiver was told
    /// about arrives, and marks it seen.
    ///
    /// Resolves immediately if one is already waiting. Intermediate values are
    /// skipped rather than queued.
    ///
    /// # Errors
    ///
    /// Fails once the sender has been dropped and no unseen value remains.
    pub fn changed(&mut self) -> Changed<'_, T> {
        Changed { receiver: self }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.inner.borrow_mut().receivers += 1;
        Receiver {
            inner: self.inner.clone(),
            seen: self.seen,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().receivers -= 1;
    }
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = T;

    /// Yields the latest value each time one arrives, skipping any the
    /// consumer was too slow to see, and ends when the sender goes away.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut inner = this.inner.borrow_mut();

        if inner.version > this.seen {
            this.seen = inner.version;
            return Poll::Ready(Some(inner.value.clone()));
        }

        if inner.sender_gone {
            return Poll::Ready(None);
        }

        inner.wakers.push(cx.waker().clone());
        Poll::Pending
    }
}

/// The future returned by [`Receiver::changed`].
#[derive(Debug)]
pub struct Changed<'a, T> {
    receiver: &'a mut Receiver<T>,
}

impl<T> Future for Changed<'_, T> {
    type Output = Result<(), GlommioError<()>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut inner = this.receiver.inner.borrow_mut();

        // An unseen value outranks a departed sender: the last thing it sent
        // is still worth delivering.
        if inner.version > this.receiver.seen {
            this.receiver.seen = inner.version;
            return Poll::Ready(Ok(()));
        }

        if inner.sender_gone {
            return Poll::Ready(Err(GlommioError::Closed(ResourceType::Channel(()))));
        }

        inner.wakers.push(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timer::Timer, LocalExecutor};
    use std::{cell::RefCell, rc::Rc, time::Duration};

    #[test]
    fn a_receiver_starts_on_the_initial_value() {
        LocalExecutor::default().run(async {
            let (_sender, receiver) = watch(5);
            assert_eq!(*receiver.borrow(), 5);
        });
    }

    #[test]
    fn changed_resolves_after_a_send_and_the_value_is_visible() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = watch(0);
            sender.send(9).unwrap();
            receiver.changed().await.unwrap();
            assert_eq!(*receiver.borrow(), 9);
        });
    }

    #[test]
    fn changed_waits_until_something_is_sent() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = watch(0);
            let order = Rc::new(RefCell::new(Vec::new()));

            let waiter = crate::spawn_local({
                let order = order.clone();
                async move {
                    receiver.changed().await.unwrap();
                    order.borrow_mut().push("observed");
                    *receiver.borrow()
                }
            })
            .detach();

            Timer::new(Duration::from_millis(10)).await;
            order.borrow_mut().push("nothing sent yet");
            sender.send(4).unwrap();

            assert_eq!(waiter.await.unwrap(), 4);
            assert_eq!(
                *order.borrow(),
                vec!["nothing sent yet", "observed"],
                "changed() resolved before any value was sent"
            );
        });
    }

    #[test]
    fn every_receiver_observes_a_change() {
        LocalExecutor::default().run(async {
            let (sender, mut first) = watch(0);
            let mut second = first.clone();

            sender.send(1).unwrap();

            first.changed().await.unwrap();
            second.changed().await.unwrap();
            assert_eq!(*first.borrow(), 1);
            assert_eq!(*second.borrow(), 1);
        });
    }

    #[test]
    fn only_the_latest_value_is_kept() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = watch(0);
            sender.send(1).unwrap();
            sender.send(2).unwrap();
            sender.send(3).unwrap();

            receiver.changed().await.unwrap();
            assert_eq!(*receiver.borrow(), 3, "a watch keeps only the latest value");
        });
    }

    #[test]
    fn dropping_the_sender_wakes_a_waiting_receiver_with_an_error() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = watch(0);

            let waiter = crate::spawn_local(async move { receiver.changed().await }).detach();

            Timer::new(Duration::from_millis(10)).await;
            drop(sender);

            assert!(
                waiter.await.unwrap().is_err(),
                "a receiver waiting on a departed sender should be woken with an error"
            );
        });
    }

    #[test]
    fn a_receiver_is_a_stream_of_the_latest_values() {
        LocalExecutor::default().run(async {
            use futures_lite::StreamExt;

            let (sender, receiver) = watch(0);
            sender.send(1).unwrap();
            sender.send(2).unwrap();
            sender.send(3).unwrap();
            drop(sender);

            let seen: Vec<_> = receiver.collect().await;
            assert_eq!(
                seen,
                vec![3],
                "a watch stream yields the latest value, not every one"
            );
        });
    }

    #[test]
    fn sending_with_no_receivers_left_fails() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = watch(0);
            drop(receiver);
            assert!(sender.send(1).is_err());
        });
    }
}
