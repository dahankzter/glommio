//! A channel carrying exactly one value.
//!
//! Request/response pairing in front of a per-core worker is the shape this
//! exists for. A [`local_channel`](super::local_channel) bounded at one works
//! and says the wrong thing: it does not express that exactly one value will
//! ever be sent, and it cannot hand an unsent value back.
//!
//! Like the rest of `channels`, both halves stay on the executor that created
//! them.
//!
//! # Examples
//!
//! ```
//! use glommio::{channels::oneshot::oneshot, LocalExecutor};
//!
//! let ex = LocalExecutor::default();
//! ex.run(async {
//!     let (sender, receiver) = oneshot();
//!     glommio::spawn_local(async move { sender.send(42).unwrap() }).detach();
//!     assert_eq!(receiver.await.unwrap(), 42);
//! });
//! ```

use crate::{error::ResourceType, wakers::WakerList, GlommioError};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

#[derive(Debug)]
struct Inner<T> {
    value: Option<T>,
    /// Holds the receiver's waker while it is suspended. A `WakerList` rather
    /// than an `Option<Waker>` so that taking it yields an obligation to wake
    /// rather than a value that can be dropped on the floor.
    waker: WakerList,
    /// Set when either half is dropped, so the survivor can tell the
    /// difference between "not yet" and "never".
    sender_gone: bool,
    receiver_gone: bool,
}

/// Sends the single value. Consumed by [`send`](Sender::send).
#[derive(Debug)]
pub struct Sender<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

/// Resolves to the single value, or to an error if the sender goes away first.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

/// Creates a new one-shot channel, returning its two halves.
pub fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Rc::new(RefCell::new(Inner {
        value: None,
        waker: WakerList::new(),
        sender_gone: false,
        receiver_gone: false,
    }));

    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner },
    )
}

impl<T> Sender<T> {
    /// Sends the value, waking the receiver if it is already waiting.
    ///
    /// # Errors
    ///
    /// If the receiver has been dropped the value has nowhere to go, so it is
    /// handed back inside [`GlommioError::Closed`] rather than dropped
    /// silently.
    pub fn send(self, value: T) -> Result<(), GlommioError<T>> {
        let mut inner = self.inner.borrow_mut();
        if inner.receiver_gone {
            return Err(GlommioError::Closed(ResourceType::Channel(value)));
        }

        inner.value = Some(value);
        let pending = inner.waker.take();
        drop(inner);
        pending.wake();
        Ok(())
    }

    /// Returns whether the receiver has gone away, so a caller holding an
    /// expensive value can skip producing it.
    pub fn is_closed(&self) -> bool {
        self.inner.borrow().receiver_gone
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.sender_gone = true;
        let pending = inner.waker.take();
        drop(inner);
        pending.wake();
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, GlommioError<()>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.borrow_mut();

        if let Some(value) = inner.value.take() {
            return Poll::Ready(Ok(value));
        }

        if inner.sender_gone {
            return Poll::Ready(Err(GlommioError::Closed(ResourceType::Channel(()))));
        }

        inner.waker.push(cx.waker().clone());
        Poll::Pending
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().receiver_gone = true;
    }
}

/// State shared by a [`SharedSender`] and [`SharedReceiver`].
///
/// Reachable from both halves directly, rather than one holding a reference to
/// the other: once a sender is gone nothing could ever fill this, so the
/// receiver must be able to read "closed" out of state it owns. Same reasoning
/// as [`ForeignCancellation`](crate::sync::ForeignCancellation).
#[derive(Debug)]
struct SharedInner<T> {
    value: Mutex<Option<T>>,
    waker: Mutex<crate::wakers::WakerList>,
    sender_gone: AtomicBool,
    receiver_gone: AtomicBool,
}

/// Sends the single value, from anywhere.
#[derive(Debug)]
pub struct SharedSender<T> {
    inner: Arc<SharedInner<T>>,
}

/// Resolves to the single value, on whichever executor polls it.
#[derive(Debug)]
pub struct SharedReceiver<T> {
    inner: Arc<SharedInner<T>>,
}

/// Creates a one-shot channel whose halves can be sent between executors.
///
/// [`oneshot`] is `Rc`-based and stays on one core, which is the right default.
/// This one is for the ask-and-reply idiom across cores: the sender travels to
/// the service that will answer, the receiver is awaited where the question was
/// asked.
///
/// # Examples
///
/// ```
/// use glommio::{channels::oneshot::shared, LocalExecutor, LocalExecutorBuilder, Placement};
///
/// let (sender, receiver) = shared();
///
/// let answering = LocalExecutorBuilder::new(Placement::Unbound)
///     .spawn(move || async move { sender.send(42).unwrap() })
///     .unwrap();
///
/// let answer = LocalExecutor::default().run(async move { receiver.await });
/// answering.join().unwrap();
/// assert_eq!(answer.unwrap(), 42);
/// ```
pub fn shared<T: Send>() -> (SharedSender<T>, SharedReceiver<T>) {
    let inner = Arc::new(SharedInner {
        value: Mutex::new(None),
        waker: Mutex::new(crate::wakers::WakerList::new()),
        sender_gone: AtomicBool::new(false),
        receiver_gone: AtomicBool::new(false),
    });

    (
        SharedSender {
            inner: inner.clone(),
        },
        SharedReceiver { inner },
    )
}

impl<T: Send> SharedSender<T> {
    /// Sends the value, waking the receiver wherever it is waiting.
    ///
    /// # Errors
    ///
    /// Hands the value back if the receiver has been dropped.
    pub fn send(self, value: T) -> Result<(), GlommioError<T>> {
        if self.inner.receiver_gone.load(Ordering::Acquire) {
            return Err(GlommioError::Closed(ResourceType::Channel(value)));
        }

        *self.inner.value.lock().unwrap() = Some(value);

        // Taken under the lock, woken outside it.
        let pending = self.inner.waker.lock().unwrap().take();
        pending.wake();
        Ok(())
    }

    /// Whether the receiver has gone away, so an expensive answer can be
    /// skipped rather than computed and thrown away.
    pub fn is_closed(&self) -> bool {
        self.inner.receiver_gone.load(Ordering::Acquire)
    }
}

impl<T> Drop for SharedSender<T> {
    fn drop(&mut self) {
        self.inner.sender_gone.store(true, Ordering::Release);
        let pending = self.inner.waker.lock().unwrap().take();
        pending.wake();
    }
}

impl<T> Future for SharedReceiver<T> {
    type Output = Result<T, GlommioError<()>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.inner.value.lock().unwrap().take() {
            return Poll::Ready(Ok(value));
        }

        let mut waker = self.inner.waker.lock().unwrap();

        // Re-checked under the waker lock: a sender may have filled the value
        // between the read above and here, in which case nobody is left to
        // wake us.
        if let Some(value) = self.inner.value.lock().unwrap().take() {
            return Poll::Ready(Ok(value));
        }

        if self.inner.sender_gone.load(Ordering::Acquire) {
            return Poll::Ready(Err(GlommioError::Closed(ResourceType::Channel(()))));
        }

        waker.push(cx.waker().clone());
        Poll::Pending
    }
}

impl<T> Drop for SharedReceiver<T> {
    fn drop(&mut self) {
        self.inner.receiver_gone.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timer::Timer, LocalExecutor};
    use std::{
        cell::RefCell,
        rc::Rc,
        time::{Duration, Instant},
    };

    #[test]
    fn a_shared_value_crosses_cores() {
        // The control-plane idiom: a reply channel handed to a service on
        // another core, fired there, awaited here.
        let (sender, receiver) = shared::<u32>();

        let worker = crate::LocalExecutorBuilder::new(crate::Placement::Unbound)
            .spawn(move || async move { sender.send(9).unwrap() })
            .unwrap();

        let value = crate::LocalExecutor::default().run(receiver);

        worker.join().unwrap();
        assert_eq!(value.unwrap(), 9);
    }

    #[test]
    fn a_shared_receiver_is_woken_when_it_was_parked() {
        let (sender, receiver) = shared::<u32>();

        let sender_thread = std::thread::spawn(move || {
            // Long enough that the receiving executor has parked in the
            // kernel: waking it is the whole mechanism.
            std::thread::sleep(Duration::from_millis(300));
            let sent_at = std::time::Instant::now();
            sender.send(1).unwrap();
            sent_at
        });

        let received_at = crate::LocalExecutor::default()
            .run(async move { receiver.await.map(|_| Instant::now()) });

        let sent_at = sender_thread.join().unwrap();
        let delay = received_at.unwrap().duration_since(sent_at);
        assert!(
            delay < Duration::from_millis(100),
            "a parked executor took {delay:?} to see the reply: it was not woken"
        );
    }

    #[test]
    fn dropping_a_shared_sender_wakes_the_receiver_with_an_error() {
        let (sender, receiver) = shared::<u32>();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            drop(sender);
        });

        let outcome = crate::LocalExecutor::default().run(receiver);
        assert!(
            outcome.is_err(),
            "a receiver whose sender went away must not wait forever"
        );
    }

    #[test]
    fn sending_to_a_departed_shared_receiver_hands_the_value_back() {
        let (sender, receiver) = shared::<String>();
        drop(receiver);

        match sender.send("payload".to_string()) {
            Err(GlommioError::Closed(ResourceType::Channel(value))) => {
                assert_eq!(value, "payload")
            }
            other => panic!("expected the value back, got {other:?}"),
        }
    }

    #[test]
    fn a_sent_value_is_received() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = oneshot::<u32>();
            sender.send(7).unwrap();
            assert_eq!(receiver.await.unwrap(), 7);
        });
    }

    #[test]
    fn the_receiver_waits_until_the_value_is_sent() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = oneshot::<u32>();
            let order = Rc::new(RefCell::new(Vec::new()));

            let waiter = crate::spawn_local({
                let order = order.clone();
                async move {
                    let value = receiver.await.unwrap();
                    order.borrow_mut().push("received");
                    value
                }
            })
            .detach();

            Timer::new(Duration::from_millis(10)).await;
            order.borrow_mut().push("still waiting");
            sender.send(3).unwrap();

            assert_eq!(waiter.await.unwrap(), 3);
            assert_eq!(
                *order.borrow(),
                vec!["still waiting", "received"],
                "the receiver resolved before anything was sent"
            );
        });
    }

    #[test]
    fn dropping_the_sender_wakes_the_receiver_with_an_error() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = oneshot::<u32>();

            let waiter = crate::spawn_local(receiver).detach();

            Timer::new(Duration::from_millis(10)).await;
            drop(sender);

            assert!(
                waiter.await.unwrap().is_err(),
                "a receiver whose sender went away should be woken with an error"
            );
        });
    }

    #[test]
    fn sending_to_a_dropped_receiver_hands_the_value_back() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = oneshot::<String>();
            drop(receiver);

            match sender.send("payload".to_string()) {
                Err(GlommioError::Closed(ResourceType::Channel(value))) => {
                    assert_eq!(value, "payload", "the unsent value should come back");
                }
                other => panic!("expected the value back, got {other:?}"),
            }
        });
    }

    #[test]
    fn the_sender_can_see_that_the_receiver_is_gone() {
        LocalExecutor::default().run(async {
            let (sender, receiver) = oneshot::<u32>();
            assert!(!sender.is_closed());
            drop(receiver);
            assert!(sender.is_closed());
        });
    }
}
