//! Sweeps the number of simultaneously-live tasks to show how task allocation
//! cost varies with concurrency.
//!
//! Run with: cargo run --release --features unsafe_detached --example spawn_sweep

use glommio::{spawn_local, LocalExecutor};
use std::time::Instant;

const REPS: usize = 7;

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

fn main() {
    LocalExecutor::default().run(async {
        batch(10_000).await; // warm up

        println!("live_tasks,ns_per_task");
        for &n in &[
            1usize, 2, 4, 7, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 65536,
        ] {
            let mut s = Vec::new();
            for _ in 0..REPS {
                s.push(batch(n).await);
            }
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("{},{:.1}", n, s[s.len() / 2]);
        }
    });
}
