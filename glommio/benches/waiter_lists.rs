//! What the waiter queues of `Semaphore`, `RwLock` and the local channel cost.
//!
//! Each primitive keeps its suspended callers in an intrusive linked list.
//! These benchmarks drive the futures by hand with a no-op waker, so the
//! measured work is queueing, waking and unlinking rather than task
//! scheduling. Futures are boxed before the timed window opens.
//!
//! `queue_and_wake` queues n waiters, releases the resource and completes
//! every waiter. `cancel_half` drops every other waiter while it is queued,
//! which unlinks from the middle of the list, then completes the rest.
//!
//! `fresh` builds the primitive every round and `reused` keeps one, since a
//! queue that owns storage pays for growing it only while it is new.
//!
//! The executor is pinned to `WAITER_BENCH_CPU`, default 10.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glommio::{
    channels::local_channel,
    sync::{RwLock, Semaphore},
    LocalExecutor, LocalExecutorBuilder, Placement,
};
use std::{
    future::Future,
    hint::black_box,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

const DEPTHS: [usize; 4] = [1, 16, 256, 4096];

fn executor() -> LocalExecutor {
    let cpu = std::env::var("WAITER_BENCH_CPU")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    LocalExecutorBuilder::new(Placement::Fixed(cpu))
        .make()
        .expect("failed to build the benchmark executor")
}

fn poll_once<F: Future + ?Sized>(fut: Pin<&mut F>) -> Poll<F::Output> {
    fut.poll(&mut Context::from_waker(Waker::noop()))
}

fn ready<F: Future + ?Sized>(fut: Pin<&mut F>) -> F::Output {
    match poll_once(fut) {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("waiter was not woken"),
    }
}

fn semaphore_round(sem: &Semaphore, n: usize, cancel_half: bool) -> Duration {
    let mut waiters: Vec<Pin<Box<_>>> = (0..n).map(|_| Box::pin(sem.acquire(1))).collect();

    let started = Instant::now();
    for w in waiters.iter_mut() {
        assert!(poll_once(w.as_mut()).is_pending());
    }
    if cancel_half {
        let mut i = 0;
        waiters.retain(|_| {
            i += 1;
            i % 2 == 0
        });
    }
    sem.signal(waiters.len() as u64);
    for w in waiters.iter_mut() {
        black_box(ready(w.as_mut())).unwrap();
    }
    drop(waiters);
    started.elapsed()
}

fn rwlock_round(lock: &RwLock<u64>, n: usize, cancel_half: bool) -> Duration {
    let held = ready(Box::pin(lock.write()).as_mut()).unwrap();
    let mut waiters: Vec<Pin<Box<_>>> = (0..n).map(|_| Box::pin(lock.read())).collect();

    let started = Instant::now();
    for w in waiters.iter_mut() {
        assert!(poll_once(w.as_mut()).is_pending());
    }
    if cancel_half {
        let mut i = 0;
        waiters.retain(|_| {
            i += 1;
            i % 2 == 0
        });
    }
    drop(held);
    let mut guards = Vec::with_capacity(waiters.len());
    for w in waiters.iter_mut() {
        guards.push(ready(w.as_mut()).unwrap());
    }
    black_box(&guards);
    drop(guards);
    drop(waiters);
    started.elapsed()
}

type Channel = (
    local_channel::LocalSender<u64>,
    local_channel::LocalReceiver<u64>,
);

fn channel_round((sender, receiver): &Channel, n: usize, cancel_half: bool) -> Duration {
    ready(Box::pin(sender.send(0)).as_mut()).unwrap();
    let mut waiters: Vec<Pin<Box<_>>> = (0..n).map(|i| Box::pin(sender.send(i as u64))).collect();

    let started = Instant::now();
    for w in waiters.iter_mut() {
        assert!(poll_once(w.as_mut()).is_pending());
    }
    if cancel_half {
        let mut i = 0;
        waiters.retain(|_| {
            i += 1;
            i % 2 == 0
        });
    }
    for w in waiters.iter_mut() {
        black_box(ready(Box::pin(receiver.recv()).as_mut()));
        ready(w.as_mut()).unwrap();
    }
    black_box(ready(Box::pin(receiver.recv()).as_mut()));
    drop(waiters);
    started.elapsed()
}

/// `fresh` builds the primitive for every round, so its queue starts empty;
/// `reused` keeps one across rounds, the steady state of a long-lived one.
fn bench_primitive<P>(
    c: &mut Criterion,
    name: &str,
    make: fn() -> P,
    round: fn(&P, usize, bool) -> Duration,
) {
    let ex = executor();
    for reuse in [false, true] {
        let mode = if reuse { "reused" } else { "fresh" };
        for (case, cancel_half) in [("queue_and_wake", false), ("cancel_half", true)] {
            let mut group = c.benchmark_group(format!("{name}/{mode}/{case}"));
            for n in DEPTHS {
                if cancel_half && n < 2 {
                    continue;
                }
                group.throughput(Throughput::Elements(n as u64));
                group.bench_function(BenchmarkId::from_parameter(n), |b| {
                    b.iter_custom(|iters| {
                        ex.run(async {
                            let kept = make();
                            let mut total = Duration::ZERO;
                            for _ in 0..iters {
                                total += if reuse {
                                    round(&kept, n, cancel_half)
                                } else {
                                    round(&make(), n, cancel_half)
                                };
                            }
                            total
                        })
                    })
                });
            }
            group.finish();
        }
    }
}

fn waiter_lists(c: &mut Criterion) {
    bench_primitive(c, "semaphore", || Semaphore::new(0), semaphore_round);
    bench_primitive(c, "rwlock", || RwLock::new(0u64), rwlock_round);
    bench_primitive(
        c,
        "local_channel",
        || local_channel::new_bounded::<u64>(1),
        channel_round,
    );
}

criterion_group!(benches, waiter_lists);
criterion_main!(benches);
