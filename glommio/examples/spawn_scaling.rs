//! Per-spawn cost as a function of how many executors are spawning concurrently.
//!
//! Each executor runs an independent spawn loop and shares nothing at the
//! application level, so in a thread-per-core runtime this should be flat. Any
//! growth here is the runtime itself serialising cores against each other, and
//! it is the regression this benchmark exists to catch.
//!
//! Run with: cargo run --release --example spawn_scaling

use glommio::{spawn_local, LocalExecutor};
use std::sync::{Arc, Barrier};
use std::time::Instant;

const WARMUP: usize = 20_000;
const ITERS: usize = 200_000;

/// Runs `threads` executors in lockstep, returning mean ns per spawn+await.
fn run(threads: usize) -> f64 {
    let barrier = Arc::new(Barrier::new(threads));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let ex = LocalExecutor::default();
                ex.run(async move {
                    for i in 0..WARMUP {
                        std::hint::black_box(spawn_local(async move { i }).await);
                    }
                    // Start every executor together so the measured window is
                    // the contended one.
                    barrier.wait();
                    let start = Instant::now();
                    for i in 0..ITERS {
                        std::hint::black_box(spawn_local(async move { i }).await);
                    }
                    start.elapsed().as_nanos() as f64 / ITERS as f64
                })
            })
        })
        .collect();

    let times: Vec<f64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    times.iter().sum::<f64>() / times.len() as f64
}

fn main() {
    let cores = std::thread::available_parallelism().unwrap().get();
    println!("concurrent executors vs per-spawn cost ({cores} cores)\n");
    println!("{:>9} | {:>12} | {:>8}", "executors", "ns/spawn", "vs 1");
    println!("{}", "-".repeat(35));

    let base = run(1);
    println!("{:>9} | {:>9.1} ns | {:>8}", 1, base, "1.00x");

    for &t in &[2usize, 4, 8, 16, 32, 64] {
        if t > cores {
            continue;
        }
        let r = run(t);
        println!("{:>9} | {:>9.1} ns | {:>7.2}x", t, r, r / base);
    }
    println!("\nA flat final column is the goal; growth means cores are contending.");
}
