//! Measures task spawn cost inside a single long-lived executor.
//!
//! Deliberately does NOT create an executor per iteration: io_uring ring setup
//! costs microseconds and swamps the nanosecond-scale allocation path being
//! measured here.
//!
//! Run with: cargo run --release --example spawn_bench

use glommio::{spawn_local, LocalExecutor};
use std::time::Instant;

const REPS: usize = 9;

fn report(name: &str, mut samples: Vec<f64>) {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[samples.len() / 2];
    let min = samples[0];
    println!("{:<34} {:>8.1} ns/task  (min {:>7.1})", name, median, min);
}

/// Sequential spawn + await. Steady state, one task live at a time.
async fn churn(iters: usize) -> f64 {
    let start = Instant::now();
    for i in 0..iters {
        let v = spawn_local(async move { i }).await;
        std::hint::black_box(v);
    }
    start.elapsed().as_nanos() as f64 / iters as f64
}

/// Spawn a batch, then await it. Many tasks live simultaneously.
async fn batch(count: usize) -> f64 {
    let start = Instant::now();
    let handles: Vec<_> = (0..count)
        .map(|i| spawn_local(async move { std::hint::black_box(i) }))
        .collect();
    for h in handles {
        h.await;
    }
    start.elapsed().as_nanos() as f64 / count as f64
}

/// Spawn and detach without awaiting; isolates the allocate-side cost.
async fn detached(count: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..count {
        spawn_local(async { std::hint::black_box(42) }).detach();
    }
    let elapsed = start.elapsed().as_nanos() as f64 / count as f64;
    // Let the detached tasks drain so they are not counted against later runs.
    glommio::executor().yield_task_queue_now().await;
    elapsed
}

/// Larger captured state, exercising a bigger task layout.
async fn churn_large(iters: usize) -> f64 {
    let start = Instant::now();
    for i in 0..iters {
        let v = spawn_local(async move {
            let buf = [i as u8; 512];
            std::hint::black_box(&buf);
            buf[0]
        })
        .await;
        std::hint::black_box(v);
    }
    start.elapsed().as_nanos() as f64 / iters as f64
}

fn main() {
    LocalExecutor::default().run(async {
        // Warm up: let the allocator reach steady state.
        churn(50_000).await;
        batch(10_000).await;

        let mut s_churn = Vec::new();
        let mut s_batch_1k = Vec::new();
        let mut s_batch_50k = Vec::new();
        let mut s_detached = Vec::new();
        let mut s_large = Vec::new();

        for _ in 0..REPS {
            s_churn.push(churn(200_000).await);
            s_batch_1k.push(batch(1_000).await);
            s_batch_50k.push(batch(50_000).await);
            s_detached.push(detached(50_000).await);
            s_large.push(churn_large(200_000).await);
        }

        println!("\nspawn cost, single long-lived executor (median of {REPS})\n");
        report("sequential spawn+await", s_churn);
        report("batch 1k spawn, then await", s_batch_1k);
        report("batch 50k spawn, then await", s_batch_50k);
        report("spawn+detach (alloc side)", s_detached);
        report("sequential, 512B capture", s_large);
        println!();
    });
}
