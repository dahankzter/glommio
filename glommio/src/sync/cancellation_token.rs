//! Hierarchical cancellation for task trees on one executor.
//!
//! A token cancels itself and everything below it, never upwards, so a request
//! can cancel its own sub-tasks without disturbing the connection it belongs
//! to.
//!
//! # Scope
//!
//! This token stays on the executor that created it: it is `!Send`, and so it
//! cannot itself carry a shutdown signal between shards. In a thread-per-core
//! program the shape that does work is one root token per executor, each
//! cancelled locally when a shared signal arrives:
//!
//! ```ignore
//! // on every shard
//! while shutdown.recv().await.is_some() {
//!     root.cancel();
//! }
//! ```
//!
//! # Examples
//!
//! ```
//! use glommio::{sync::CancellationToken, LocalExecutor};
//!
//! let ex = LocalExecutor::default();
//! ex.run(async {
//!     let root = CancellationToken::new();
//!     let request = root.child_token();
//!
//!     root.cancel();
//!
//!     assert!(request.is_cancelled());
//!     request.cancelled().await;
//! });
//! ```

use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

#[derive(Debug, Default)]
struct Node {
    cancelled: Cell<bool>,
    wakers: RefCell<Vec<Waker>>,
    /// Weak, so a child that outlives its parent keeps working rather than
    /// keeping the parent alive.
    children: RefCell<Vec<Weak<Node>>>,
}

impl Node {
    fn cancel(&self) {
        if self.cancelled.replace(true) {
            // Already cancelled: the tree below has been dealt with.
            return;
        }

        for waker in self.wakers.borrow_mut().drain(..) {
            waker.wake();
        }

        // Children that have been dropped simply fail to upgrade, which also
        // prunes them from the list.
        let children = std::mem::take(&mut *self.children.borrow_mut());
        for child in children {
            if let Some(child) = child.upgrade() {
                child.cancel();
            }
        }
    }
}

/// A token that can be cancelled, cancelling every token derived from it.
///
/// Cloning gives another handle to the same token, not a child: cancelling
/// either cancels both.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    node: Rc<Node>,
}

impl CancellationToken {
    /// Creates a token with no parent.
    pub fn new() -> Self {
        CancellationToken {
            node: Rc::new(Node::default()),
        }
    }

    /// Creates a token cancelled when this one is, or immediately if this one
    /// is already cancelled.
    ///
    /// The immediate case matters: a task that subscribes just after shutdown
    /// has begun must not wait forever.
    pub fn child_token(&self) -> Self {
        let child = Rc::new(Node {
            cancelled: Cell::new(self.node.cancelled.get()),
            ..Node::default()
        });

        if !child.cancelled.get() {
            self.node.children.borrow_mut().push(Rc::downgrade(&child));
        }

        CancellationToken { node: child }
    }

    /// Cancels this token and every token derived from it.
    ///
    /// Cancelling twice is harmless, and cancellation never travels upwards.
    pub fn cancel(&self) {
        self.node.cancel();
    }

    /// Returns whether this token has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.node.cancelled.get()
    }

    /// Waits until this token is cancelled, returning immediately if it
    /// already has been.
    ///
    /// Dropping the returned future is safe: it leaves behind a waker that is
    /// discarded when cancellation eventually drains the list.
    pub fn cancelled(&self) -> Cancelled<'_> {
        Cancelled { token: self }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        CancellationToken::new()
    }
}

/// The future returned by [`CancellationToken::cancelled`].
#[derive(Debug)]
pub struct Cancelled<'a> {
    token: &'a CancellationToken,
}

impl Future for Cancelled<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            return Poll::Ready(());
        }

        self.token.node.wakers.borrow_mut().push(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timer::Timer, LocalExecutor};
    use std::{cell::RefCell, rc::Rc, time::Duration};

    #[test]
    fn a_fresh_token_is_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn cancelled_waits_until_the_token_is_cancelled() {
        LocalExecutor::default().run(async {
            let token = CancellationToken::new();
            let order = Rc::new(RefCell::new(Vec::new()));

            let waiter = crate::spawn_local({
                let token = token.clone();
                let order = order.clone();
                async move {
                    token.cancelled().await;
                    order.borrow_mut().push("observed");
                }
            })
            .detach();

            Timer::new(Duration::from_millis(10)).await;
            order.borrow_mut().push("not cancelled yet");
            token.cancel();

            waiter.await;
            assert_eq!(
                *order.borrow(),
                vec!["not cancelled yet", "observed"],
                "cancelled() resolved before the token was cancelled"
            );
        });
    }

    #[test]
    fn cancelled_returns_immediately_on_an_already_cancelled_token() {
        LocalExecutor::default().run(async {
            let token = CancellationToken::new();
            token.cancel();
            token.cancelled().await;
        });
    }

    #[test]
    fn cancelling_a_parent_cancels_its_children_and_grandchildren() {
        let root = CancellationToken::new();
        let child = root.child_token();
        let grandchild = child.child_token();

        root.cancel();

        assert!(child.is_cancelled());
        assert!(
            grandchild.is_cancelled(),
            "cancellation must reach the whole tree"
        );
    }

    #[test]
    fn cancelling_a_child_leaves_its_parent_running() {
        let root = CancellationToken::new();
        let child = root.child_token();

        child.cancel();

        assert!(child.is_cancelled());
        assert!(!root.is_cancelled(), "cancellation must not travel upwards");
    }

    #[test]
    fn a_child_of_an_already_cancelled_parent_starts_cancelled() {
        let root = CancellationToken::new();
        root.cancel();

        let child = root.child_token();
        assert!(
            child.is_cancelled(),
            "subscribing after shutdown must not wait forever"
        );
    }

    #[test]
    fn clones_share_one_token() {
        let token = CancellationToken::new();
        let clone = token.clone();

        clone.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn dropping_a_parent_does_not_cancel_its_child() {
        let root = CancellationToken::new();
        let child = root.child_token();

        drop(root);

        assert!(
            !child.is_cancelled(),
            "dropping a token is not cancelling it"
        );
    }

    #[test]
    fn a_waiting_child_is_woken_when_the_parent_is_cancelled() {
        LocalExecutor::default().run(async {
            let root = CancellationToken::new();
            let child = root.child_token();

            let waiter = crate::spawn_local(async move { child.cancelled().await }).detach();

            Timer::new(Duration::from_millis(10)).await;
            root.cancel();

            waiter.await;
        });
    }
}
