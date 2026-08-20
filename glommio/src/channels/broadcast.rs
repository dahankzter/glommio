//! Multi-consumer fan-out where every receiver sees every value.
//!
//! Every receiver observes every value sent after it subscribed. A receiver
//! that falls too far behind is told so -- [`RecvError::Lagged`] -- rather than
//! being allowed to hold the sender up, which is the difference between this
//! and a bounded [`local_channel`](super::local_channel).
//!
//! Where only the newest value matters, [`watch`](super::watch) is the better
//! fit: it keeps one value rather than a window of them.
//!
//! Every value is cloned once per receiver. That is not a limitation worth
//! working around: `Rc<T>` is itself `Clone`, so broadcasting `Rc<T>` gives
//! refcount-cheap fan-out under the same API.
//!
//! Both halves stay on the executor that created them.
//!
//! # Examples
//!
//! ```
//! use glommio::{channels::broadcast::broadcast, LocalExecutor};
//!
//! let ex = LocalExecutor::default();
//! ex.run(async {
//!     let (sender, mut receiver) = broadcast(16);
//!     let mut second = sender.subscribe();
//!
//!     sender.send(1).unwrap();
//!
//!     assert_eq!(receiver.recv().await.unwrap(), 1);
//!     assert_eq!(second.recv().await.unwrap(), 1);
//! });
//! ```

use crate::{error::ResourceType, GlommioError};
use futures_lite::Stream;
use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/// Why [`Receiver::recv`] could not produce a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// Every sender is gone and every retained value has been read.
    Closed,
    /// This receiver fell behind and the values it had not read were
    /// overwritten. Carries how many were missed; the next `recv` returns the
    /// oldest value still retained, so consumption can simply continue.
    Lagged(u64),
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvError::Closed => write!(f, "the broadcast channel is closed"),
            RecvError::Lagged(missed) => {
                write!(f, "the receiver lagged behind by {missed} values")
            }
        }
    }
}

impl std::error::Error for RecvError {}

/// Why [`Receiver::try_recv`] could not produce a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    /// No value is waiting, but the channel is still open.
    Empty,
    /// Every sender is gone and every retained value has been read.
    Closed,
    /// As [`RecvError::Lagged`].
    Lagged(u64),
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "no value is waiting"),
            TryRecvError::Closed => write!(f, "the broadcast channel is closed"),
            TryRecvError::Lagged(missed) => {
                write!(f, "the receiver lagged behind by {missed} values")
            }
        }
    }
}

impl std::error::Error for TryRecvError {}

#[derive(Debug)]
struct Inner<T> {
    /// The retained window, oldest first, never longer than `capacity`.
    ///
    /// A value lives here until it is overwritten rather than until every
    /// receiver has read it. The retention bound is the same either way, and
    /// this keeps the bookkeeping to one cursor per receiver.
    values: VecDeque<(u64, T)>,
    capacity: usize,
    /// The sequence the next sent value will carry.
    next_seq: u64,
    wakers: Vec<Waker>,
    senders: usize,
    receivers: usize,
}

impl<T> Inner<T> {
    fn wake_all(&mut self) {
        for waker in self.wakers.drain(..) {
            waker.wake();
        }
    }

    /// The sequence of the oldest value still retained, if any.
    fn oldest(&self) -> Option<u64> {
        self.values.front().map(|(seq, _)| *seq)
    }
}

/// Sends values to every receiver. Clone it to send from more than one place.
#[derive(Debug)]
pub struct Sender<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

/// Receives every value sent after it subscribed.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: Rc<RefCell<Inner<T>>>,
    /// The sequence this receiver wants next.
    next: u64,
}

/// Creates a broadcast channel retaining at most `capacity` values.
///
/// # Panics
///
/// Panics if `capacity` is zero: a channel that can retain nothing would
/// report every value as lagged.
pub fn broadcast<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(
        capacity > 0,
        "a broadcast channel needs room for at least one value"
    );

    let inner = Rc::new(RefCell::new(Inner {
        values: VecDeque::with_capacity(capacity),
        capacity,
        next_seq: 0,
        wakers: Vec::new(),
        senders: 1,
        receivers: 1,
    }));

    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner, next: 0 },
    )
}

impl<T: Clone> Sender<T> {
    /// Sends a value to every receiver, dropping the oldest retained value if
    /// the channel is full.
    ///
    /// Returns how many receivers the value went to.
    ///
    /// # Errors
    ///
    /// With no receivers left the value has nowhere to go, so it is handed
    /// back inside [`GlommioError::Closed`] rather than dropped silently.
    pub fn send(&self, value: T) -> Result<usize, GlommioError<T>> {
        let mut inner = self.inner.borrow_mut();
        if inner.receivers == 0 {
            return Err(GlommioError::Closed(ResourceType::Channel(value)));
        }

        if inner.values.len() == inner.capacity {
            inner.values.pop_front();
        }

        let seq = inner.next_seq;
        inner.next_seq += 1;
        inner.values.push_back((seq, value));

        let reached = inner.receivers;
        inner.wake_all();
        Ok(reached)
    }

    /// Creates a receiver that will see values sent from now on, and none of
    /// those sent before.
    pub fn subscribe(&self) -> Receiver<T> {
        let mut inner = self.inner.borrow_mut();
        inner.receivers += 1;
        let next = inner.next_seq;
        drop(inner);

        Receiver {
            inner: self.inner.clone(),
            next,
        }
    }

    /// How many receivers are listening.
    pub fn receiver_count(&self) -> usize {
        self.inner.borrow().receivers
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.borrow_mut().senders += 1;
        Sender {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.senders -= 1;
        if inner.senders == 0 {
            inner.wake_all();
        }
    }
}

impl<T: Clone> Receiver<T> {
    /// Waits for the next value.
    ///
    /// Returns [`RecvError::Lagged`] if values this receiver had not read were
    /// overwritten; the cursor then sits on the oldest retained value, so the
    /// next call resumes from there.
    pub fn recv(&mut self) -> Recv<'_, T> {
        Recv { receiver: self }
    }

    /// Takes the next value if one is waiting, without suspending.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let inner = self.inner.borrow();

        if let Some(oldest) = inner.oldest() {
            if self.next < oldest {
                let missed = oldest - self.next;
                self.next = oldest;
                return Err(TryRecvError::Lagged(missed));
            }
        }

        let found = inner
            .values
            .iter()
            .find(|(seq, _)| *seq == self.next)
            .map(|(_, value)| value.clone());

        match found {
            Some(value) => {
                self.next += 1;
                Ok(value)
            }
            None if inner.senders == 0 => Err(TryRecvError::Closed),
            None => Err(TryRecvError::Empty),
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.inner.borrow_mut().receivers += 1;
        Receiver {
            inner: self.inner.clone(),
            next: self.next,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().receivers -= 1;
    }
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = Result<T, RecvError>;

    /// Yields `Err(RecvError::Lagged(n))` rather than skipping quietly, and
    /// ends when the channel closes.
    ///
    /// A lagging receiver is a fact its consumer usually wants to know -- a
    /// stream that hid it would turn a detectable gap into a silent one.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        match this.try_recv() {
            Ok(value) => Poll::Ready(Some(Ok(value))),
            Err(TryRecvError::Lagged(missed)) => Poll::Ready(Some(Err(RecvError::Lagged(missed)))),
            Err(TryRecvError::Closed) => Poll::Ready(None),
            Err(TryRecvError::Empty) => {
                this.inner.borrow_mut().wakers.push(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// The future returned by [`Receiver::recv`].
#[derive(Debug)]
pub struct Recv<'a, T> {
    receiver: &'a mut Receiver<T>,
}

impl<T: Clone> Future for Recv<'_, T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this.receiver.try_recv() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(TryRecvError::Lagged(missed)) => Poll::Ready(Err(RecvError::Lagged(missed))),
            Err(TryRecvError::Closed) => Poll::Ready(Err(RecvError::Closed)),
            Err(TryRecvError::Empty) => {
                this.receiver
                    .inner
                    .borrow_mut()
                    .wakers
                    .push(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timer::Timer, LocalExecutor};
    use std::{cell::RefCell, rc::Rc, time::Duration};

    #[test]
    fn every_receiver_sees_every_value() {
        LocalExecutor::default().run(async {
            let (sender, mut first) = broadcast(4);
            let mut second = sender.subscribe();

            sender.send(1).unwrap();
            sender.send(2).unwrap();

            assert_eq!(first.recv().await.unwrap(), 1);
            assert_eq!(first.recv().await.unwrap(), 2);
            assert_eq!(second.recv().await.unwrap(), 1);
            assert_eq!(second.recv().await.unwrap(), 2);
        });
    }

    #[test]
    fn recv_waits_until_a_value_is_sent() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = broadcast(4);
            let order = Rc::new(RefCell::new(Vec::new()));

            let waiter = crate::spawn_local({
                let order = order.clone();
                async move {
                    let value = receiver.recv().await.unwrap();
                    order.borrow_mut().push("received");
                    value
                }
            })
            .detach();

            Timer::new(Duration::from_millis(10)).await;
            order.borrow_mut().push("nothing sent yet");
            sender.send(8).unwrap();

            assert_eq!(waiter.await.unwrap(), 8);
            assert_eq!(
                *order.borrow(),
                vec!["nothing sent yet", "received"],
                "recv resolved before anything was sent"
            );
        });
    }

    #[test]
    fn a_later_subscriber_sees_only_later_values() {
        LocalExecutor::default().run(async {
            let (sender, mut early) = broadcast(4);
            sender.send(1).unwrap();

            let mut late = sender.subscribe();
            sender.send(2).unwrap();

            assert_eq!(early.recv().await.unwrap(), 1);
            assert_eq!(early.recv().await.unwrap(), 2);
            assert_eq!(
                late.recv().await.unwrap(),
                2,
                "a subscriber should not see values sent before it existed"
            );
        });
    }

    #[test]
    fn overflow_reports_how_much_was_missed_and_resumes_at_the_oldest_kept() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = broadcast(2);
            for value in 1..=5 {
                sender.send(value).unwrap();
            }

            // Capacity 2, five sent: values 1, 2 and 3 are gone.
            match receiver.recv().await {
                Err(RecvError::Lagged(missed)) => assert_eq!(missed, 3),
                other => panic!("expected Lagged(3), got {other:?}"),
            }

            assert_eq!(
                receiver.recv().await.unwrap(),
                4,
                "after lagging, the cursor should land on the oldest retained value"
            );
            assert_eq!(receiver.recv().await.unwrap(), 5);
        });
    }

    #[test]
    fn dropping_the_last_sender_wakes_a_waiting_receiver() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = broadcast::<u32>(4);

            let waiter = crate::spawn_local(async move { receiver.recv().await }).detach();

            Timer::new(Duration::from_millis(10)).await;
            drop(sender);

            assert!(matches!(waiter.await.unwrap(), Err(RecvError::Closed)));
        });
    }

    #[test]
    fn retained_values_are_drained_before_closed() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = broadcast(4);
            sender.send(1).unwrap();
            sender.send(2).unwrap();
            drop(sender);

            assert_eq!(receiver.recv().await.unwrap(), 1);
            assert_eq!(receiver.recv().await.unwrap(), 2);
            assert!(matches!(receiver.recv().await, Err(RecvError::Closed)));
        });
    }

    #[test]
    fn sending_with_no_receivers_hands_the_value_back() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = broadcast::<String>(4);
            drop(receiver);

            match sender.send("payload".to_string()) {
                Err(GlommioError::Closed(ResourceType::Channel(value))) => {
                    assert_eq!(value, "payload")
                }
                other => panic!("expected the value back, got {other:?}"),
            }
        });
    }

    #[test]
    fn send_reports_how_many_receivers_it_reached() {
        LocalExecutor::default().run(async {
            let (sender, _first) = broadcast(4);
            assert_eq!(sender.send(1).unwrap(), 1);
            let _second = sender.subscribe();
            assert_eq!(sender.send(2).unwrap(), 2);
            assert_eq!(sender.receiver_count(), 2);
        });
    }

    #[test]
    fn a_receiver_is_a_stream_that_surfaces_lag() {
        LocalExecutor::default().run(async {
            use futures_lite::StreamExt;

            let (sender, receiver) = broadcast(2);
            for value in 1..=4 {
                sender.send(value).unwrap();
            }
            drop(sender);

            let seen: Vec<_> = receiver.collect().await;

            // Capacity 2 of 4 sent: the stream reports the gap rather than
            // hiding it, then yields what survived, then ends.
            assert!(matches!(seen[0], Err(RecvError::Lagged(2))));
            assert_eq!(seen[1].as_ref().unwrap(), &3);
            assert_eq!(seen[2].as_ref().unwrap(), &4);
            assert_eq!(
                seen.len(),
                3,
                "the stream should end when the channel closes"
            );
        });
    }

    #[test]
    fn try_recv_tells_empty_lagged_and_closed_apart() {
        LocalExecutor::default().run(async {
            let (sender, mut receiver) = broadcast(2);
            assert!(matches!(receiver.try_recv(), Err(TryRecvError::Empty)));

            for value in 1..=4 {
                sender.send(value).unwrap();
            }
            assert!(matches!(receiver.try_recv(), Err(TryRecvError::Lagged(2))));
            assert_eq!(receiver.try_recv().unwrap(), 3);
            assert_eq!(receiver.try_recv().unwrap(), 4);

            drop(sender);
            assert!(matches!(receiver.try_recv(), Err(TryRecvError::Closed)));
        });
    }
}
