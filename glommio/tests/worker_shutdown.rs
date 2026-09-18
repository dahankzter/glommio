// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
//! A blocking job finishing after its executor is gone must not panic.
//!
//! The worker finds a closed response channel, which is how an executor's life
//! ends rather than a failure. Panicking there is stderr noise under
//! unwinding and a dead process under `panic = "abort"`.
//!
//! This file is its own test binary because the panic hook is process wide.

use glommio::LocalExecutorBuilder;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

#[test]
fn a_job_outliving_its_executor_does_not_panic() {
    let panics = Arc::new(AtomicUsize::new(0));
    let counter = panics.clone();
    std::panic::set_hook(Box::new(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    LocalExecutorBuilder::default()
        .spawn(|| async {
            // Polled once so the job is submitted, then abandoned, which is
            // what `select!` does when another branch wins. The closure keeps
            // running and the executor is free to exit first.
            let job = glommio::executor()
                .spawn_blocking(|| std::thread::sleep(Duration::from_millis(200)));
            let mut job = Box::pin(job);
            assert!(
                futures_lite::future::poll_once(&mut job).await.is_none(),
                "the job should still be running"
            );
            drop(job);
        })
        .unwrap()
        .join()
        .unwrap();

    // Long enough for the worker to finish and find the channel closed.
    std::thread::sleep(Duration::from_millis(600));
    let seen = panics.load(Ordering::SeqCst);
    let _ = std::panic::take_hook();

    assert_eq!(
        seen, 0,
        "a worker panicked when its executor had already gone"
    );
}
