//! `glommio::select!` against the real runtime.
#![cfg(feature = "macros")]

use glommio::{select, timer::sleep};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

#[glommio::test]
async fn the_ready_branch_wins_and_binds_its_output() {
    let value = select! {
        v = async { 7u32 } => v,
        _ = sleep(Duration::from_secs(30)) => 0,
    };
    assert_eq!(value, 7);
}

#[glommio::test]
async fn a_branch_body_can_await() {
    // The reason the expansion splits into a poll phase and a handler phase:
    // handlers run in the caller's async context.
    let value = select! {
        v = async { 3u32 } => {
            sleep(Duration::from_millis(1)).await;
            v * 2
        },
        _ = sleep(Duration::from_secs(30)) => 0,
    };
    assert_eq!(value, 6);
}

#[glommio::test]
async fn the_losing_future_is_dropped() {
    struct NotesItsDrop(Rc<RefCell<bool>>);
    impl Drop for NotesItsDrop {
        fn drop(&mut self) {
            *self.0.borrow_mut() = true;
        }
    }

    let dropped = Rc::new(RefCell::new(false));
    let flag = dropped.clone();

    // Constructed here, not inside the async block: a branch that never gets
    // polled never runs its body, so a guard created in there would prove
    // nothing about dropping the future.
    let notes = NotesItsDrop(flag);
    select! {
        v = async { 1u32 } => v,
        _ = async move {
            let _notes = notes;
            sleep(Duration::from_secs(30)).await;
        } => 0,
    };

    assert!(
        *dropped.borrow(),
        "the losing future should be dropped, cancelling its work"
    );
}

#[glommio::test]
async fn biased_polls_top_to_bottom() {
    // Both branches are ready, so only the order decides.
    for _ in 0..8 {
        let winner = select! {
            biased;
            _ = async {} => "first",
            _ = async {} => "second",
        };
        assert_eq!(winner, "first");
    }
}

#[glommio::test]
async fn the_default_order_rotates_so_neither_branch_starves() {
    let mut seen_first = false;
    let mut seen_second = false;

    for _ in 0..8 {
        match select! {
            _ = async {} => "first",
            _ = async {} => "second",
        } {
            "first" => seen_first = true,
            _ => seen_second = true,
        }
    }

    assert!(
        seen_first && seen_second,
        "with two always-ready branches the default order should reach both; \
         a fixed order starves the later one"
    );
}

#[glommio::test]
async fn five_branches_work() {
    // The largest arity in the downstream that asked for this.
    let value = select! {
        _ = sleep(Duration::from_secs(30)) => 1u32,
        _ = sleep(Duration::from_secs(30)) => 2,
        v = async { 3u32 } => v,
        _ = sleep(Duration::from_secs(30)) => 4,
        _ = sleep(Duration::from_secs(30)) => 5,
    };
    assert_eq!(value, 3);
}

#[glommio::test]
async fn patterns_bind_as_written() {
    let unit = select! {
        () = async {} => "unit",
        _ = sleep(Duration::from_secs(30)) => "timer",
    };
    assert_eq!(unit, "unit");

    let named = select! {
        maybe = async { Some(4u8) } => maybe,
        _ = sleep(Duration::from_secs(30)) => None,
    };
    assert_eq!(named, Some(4));
}

#[glommio::test]
async fn a_pending_branch_does_not_win() {
    let started = Instant::now();
    let value = select! {
        _ = sleep(Duration::from_secs(30)) => "slow",
        _ = sleep(Duration::from_millis(5)) => "quick",
    };
    assert_eq!(value, "quick");
    assert!(started.elapsed() < Duration::from_secs(1));
}
