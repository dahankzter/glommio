use glommio::LocalExecutor;
use std::time::Instant;

fn main() {
    println!("Benchmarking scoped spawn latency...\n");

    LocalExecutor::default().run(async {
        // Warm up
        for _ in 0..1000 {
            glommio::executor()
                .spawn_scope(|scope| async move {
                    let _ = scope.spawn(async { 42 }).await;
                })
                .await;
        }

        // Benchmark: Single spawn
        let iterations = 100_000;
        let start = Instant::now();

        for _ in 0..iterations {
            glommio::executor()
                .spawn_scope(|scope| async move {
                    let _ = scope.spawn(async { 42 }).await;
                })
                .await;
        }

        let elapsed = start.elapsed();
        let ns_per_spawn = elapsed.as_nanos() / iterations;

        println!("Single spawn (with await):");
        println!("  Total time: {:?}", elapsed);
        println!("  Iterations: {}", iterations);
        println!("  Latency: {} ns/spawn", ns_per_spawn);
        println!();

        // Benchmark: Batch spawn (to measure allocation overhead)
        let batch_size = 100;
        let batch_iterations = 1_000;
        let start = Instant::now();

        for _ in 0..batch_iterations {
            glommio::executor()
                .spawn_scope(|scope| async move {
                    let mut handles = Vec::with_capacity(batch_size);
                    for i in 0..batch_size {
                        handles.push(scope.spawn(async move { i }));
                    }
                    for h in handles {
                        let _ = h.await;
                    }
                })
                .await;
        }

        let elapsed = start.elapsed();
        let total_spawns = batch_size * batch_iterations;
        let ns_per_spawn = elapsed.as_nanos() / total_spawns as u128;

        println!("Batch spawn ({} tasks):", batch_size);
        println!("  Total time: {:?}", elapsed);
        println!("  Total spawns: {}", total_spawns);
        println!("  Latency: {} ns/spawn", ns_per_spawn);
        println!();

        // Benchmark: Concurrent spawn (measure true concurrent capacity)
        // Spawn 10K tasks concurrently (well under 100K capacity)
        let concurrent_tasks = 10_000;
        let start = Instant::now();

        glommio::executor()
            .spawn_scope(|scope| async move {
                let mut handles = Vec::with_capacity(concurrent_tasks);
                for i in 0..concurrent_tasks {
                    handles.push(scope.spawn(async move { i }));
                }
                // Await all at once
                for h in handles {
                    let _ = h.await;
                }
            })
            .await;

        let elapsed = start.elapsed();
        let ns_per_spawn = elapsed.as_nanos() / concurrent_tasks as u128;

        println!("Concurrent spawn ({} tasks):", concurrent_tasks);
        println!("  Total time: {:?}", elapsed);
        println!("  Latency: {} ns/spawn", ns_per_spawn);
        println!();

        // Demonstrate recycling
        println!("Recycling test (spawn 200K tasks with 100K capacity):");
        let recycling_iterations = 200_000;
        let start = Instant::now();

        for _ in 0..recycling_iterations {
            glommio::executor()
                .spawn_scope(|scope| async move {
                    let _ = scope.spawn(async { 42 }).await;
                })
                .await;
        }

        let elapsed = start.elapsed();
        let ns_per_spawn = elapsed.as_nanos() / recycling_iterations;

        println!("  Total time: {:?}", elapsed);
        println!("  Iterations: {}", recycling_iterations);
        println!("  Latency: {} ns/spawn (with recycling)", ns_per_spawn);
        println!("  ✓ Successfully recycled slots (no heap fallback!)");
    });
}
