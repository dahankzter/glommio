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

use crate::{error::ResourceType, GlommioError};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

#[derive(Debug)]
struct Inner<T> {
    value: Option<T>,
    /// Set while the receiver is suspended, taken when it is woken.
    waker: Option<Waker>,
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
        waker: None,
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
        let waker = inner.waker.take();
        drop(inner);
        if let Some(waker) = waker {
            waker.wake();
        }
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
        let waker = inner.waker.take();
        drop(inner);
        if let Some(waker) = waker {
            waker.wake();
        }
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

        inner.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().receiver_gone = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timer::Timer, LocalExecutor};
    use std::{cell::RefCell, rc::Rc, time::Duration};

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
