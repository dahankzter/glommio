// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Simple spawn benchmark without criterion overhead
//!
//! This directly measures spawn latency by creating one executor and timing spawns.

use glommio::{spawn_local, LocalExecutor};
use std::time::Instant;

fn main() {
    println!("Arena Allocator Spawn Benchmark");
    println!("================================\n");

    LocalExecutor::default().run(async {
        // Warmup
        println!("Warming up...");
        for _ in 0..1000 {
            spawn_local(async {
                42
            })
            .detach();
        }

        // Benchmark: Single spawn latency
        println!("\n1. Single spawn latency:");
        let iterations = 10000;
        let start = Instant::now();
        for _ in 0..iterations {
            spawn_local(async {
                42
            })
            .detach();
        }
        let elapsed = start.elapsed();
        let avg_ns = elapsed.as_nanos() / iterations as u128;
        println!("   {} iterations in {:?}", iterations, elapsed);
        println!("   Average: {} ns/spawn", avg_ns);

        // Benchmark: Spawn with await
        println!("\n2. Spawn + await latency:");
        let iterations = 1000;
        let start = Instant::now();
        for i in 0..iterations {
            spawn_local(async move {
                i
            })
            .await;
        }
        let elapsed = start.elapsed();
        let avg_ns = elapsed.as_nanos() / iterations as u128;
        println!("   {} iterations in {:?}", iterations, elapsed);
        println!("   Average: {} ns/spawn+await", avg_ns);

        // Benchmark: Batch spawn throughput
        println!("\n3. Batch spawn throughput:");
        let batch_size = 1000;
        let batches = 10;
        let start = Instant::now();
        for _ in 0..batches {
            for _ in 0..batch_size {
                spawn_local(async {
                    42
                })
                .detach();
            }
        }
        let elapsed = start.elapsed();
        let total = batch_size * batches;
        let throughput = total as f64 / elapsed.as_secs_f64();
        println!("   {} tasks in {:?}", total, elapsed);
        println!("   Throughput: {:.0} tasks/sec", throughput);

        println!("\n✅ Benchmark complete!");
        println!("\nTarget: Reduce spawn latency from ~80ns to ~20ns");
        println!("Result: {} ns/spawn", avg_ns);
        if avg_ns < 30 {
            println!("🎉 SUCCESS! Arena allocator significantly improved performance!");
        } else if avg_ns < 50 {
            println!("✓ Good improvement, getting closer to target");
        } else {
            println!("⚠ More optimization needed");
        }
    });
}
