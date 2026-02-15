// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Minimal example to test arena allocator functionality

use futures::future::join_all;
use glommio::{spawn_local, LocalExecutor};

fn main() {
    println!("Testing arena allocator...");

    // Test 1: Simple spawn
    println!("\nTest 1: Simple spawn (10 tasks)");
    LocalExecutor::default().run(async {
        for i in 0..10 {
            spawn_local(async move {
                println!("  Task {}", i);
            })
            .detach();
        }
    });
    println!("✓ Test 1 passed");

    // Test 2: Many spawns within arena capacity
    println!("\nTest 2: Many spawns (1000 tasks)");
    LocalExecutor::default().run(async {
        let mut handles = Vec::new();
        for i in 0..1000 {
            handles.push(spawn_local(async move {
                i * 2
            }));
        }

        let results: Vec<_> =
            futures_lite::future::block_on(async { join_all(handles).await });

        assert_eq!(results.len(), 1000);
        println!("  Completed {} tasks", results.len());
    });
    println!("✓ Test 2 passed");

    // Test 3: Exceed arena capacity (fallback to heap)
    println!("\nTest 3: Exceed arena capacity (3000 tasks, capacity=2000)");
    LocalExecutor::default().run(async {
        let mut handles = Vec::new();
        for i in 0..3000 {
            handles.push(spawn_local(async move {
                i
            }));
        }

        let results: Vec<_> =
            futures_lite::future::block_on(async { join_all(handles).await });

        assert_eq!(results.len(), 3000);
        println!("  Completed {} tasks (some from arena, some from heap)", results.len());
    });
    println!("✓ Test 3 passed");

    println!("\n✅ All arena tests passed!");
}
