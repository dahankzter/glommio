// Integration test for issue #448 - Eventfd leak on executor drop
// This test verifies that eventfds are properly closed when executors drop

use glommio::channels::shared_channel;
use glommio::prelude::*;

#[cfg(target_os = "linux")]
fn count_open_fds() -> usize {
    let pid = std::process::id();
    let fd_dir = format!("/proc/{}/fd", pid);

    std::fs::read_dir(&fd_dir)
        .map(|entries| entries.count())
        .unwrap_or(0)
}

#[cfg(target_os = "macos")]
fn count_open_fds() -> usize {
    let pid = std::process::id();
    let output = std::process::Command::new("lsof")
        .args(&["-p", &pid.to_string()])
        .output()
        .expect("Failed to run lsof");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.contains("eventfd") || line.contains("KQUEUE"))
        .count()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn count_open_fds() -> usize {
    0 // Unsupported platform
}

fn test_spsc_cycle(capacity: usize, runs: u32) {
    let (sender, receiver) = shared_channel::new_bounded(capacity);

    let sender_handle = LocalExecutorBuilder::new(Placement::Fixed(0))
        .spawn(move || async move {
            let sender = sender.connect().await;
            for _ in 0..runs {
                sender.send(1).await.unwrap();
            }
            drop(sender);
        })
        .unwrap();

    let receiver_handle = LocalExecutorBuilder::new(Placement::Fixed(1))
        .spawn(move || async move {
            let receiver = receiver.connect().await;
            for _ in 0..runs {
                receiver.recv().await.unwrap();
            }
        })
        .unwrap();

    sender_handle.join().unwrap();
    receiver_handle.join().unwrap();
}

#[test]
fn test_no_eventfd_leak_on_executor_drop() {
    let initial_fds = count_open_fds();
    println!("Initial FD count: {}", initial_fds);

    // Run multiple cycles of executor creation/destruction
    for round in 0..5 {
        println!("\n=== Round {} ===", round);

        // Create and destroy executors with shared channels
        test_spsc_cycle(100, 100);
        test_spsc_cycle(1000, 100);
        test_spsc_cycle(10000, 100);

        let current_fds = count_open_fds();
        let leaked = current_fds.saturating_sub(initial_fds);

        println!("Current FD count: {} (leaked: {})", current_fds, leaked);

        // Allow some tolerance for timing variations, but leak should be minimal
        // Before fix: ~36 fds leaked per cycle (12 per test * 3 tests)
        // After fix: should be 0-5 (just timing noise)
        assert!(
            leaked < 10,
            "Too many FDs leaked: {} after {} rounds. Expected < 10",
            leaked,
            round + 1
        );
    }

    let final_fds = count_open_fds();
    println!("\n✅ Test passed! FD count stable: {} -> {}", initial_fds, final_fds);

    // Final check: total leak should be very small
    let total_leaked = final_fds.saturating_sub(initial_fds);
    assert!(
        total_leaked < 15,
        "Total FDs leaked: {}. Fix may not be working correctly",
        total_leaked
    );
}

#[test]
fn test_rapid_executor_creation() {
    let initial_fds = count_open_fds();
    println!("Initial FD count: {}", initial_fds);

    // Rapidly create and destroy many executors
    for _ in 0..20 {
        let executor = LocalExecutorBuilder::new(Placement::Fixed(0))
            .spawn(|| async move {
                // Minimal work
                glommio::timer::sleep(std::time::Duration::from_millis(1)).await;
            })
            .unwrap();

        executor.join().unwrap();
    }

    let final_fds = count_open_fds();
    let leaked = final_fds.saturating_sub(initial_fds);

    println!("Final FD count: {} (leaked: {})", final_fds, leaked);

    // Should have minimal leak (< 5 for noise)
    assert!(
        leaked < 5,
        "FDs leaked after rapid executor creation: {}",
        leaked
    );
}

#[test]
fn test_executor_with_tasks() {
    let initial_fds = count_open_fds();

    // Create executors that spawn tasks
    for _ in 0..10 {
        let executor = LocalExecutorBuilder::new(Placement::Fixed(0))
            .spawn(|| async move {
                // Spawn some tasks
                let tasks: Vec<_> = (0..10)
                    .map(|i| {
                        glommio::spawn_local(async move {
                            i * 2
                        })
                    })
                    .collect();

                // Await some (but not all) tasks
                for task in tasks.into_iter().take(5) {
                    task.await;
                }

                // Leave some tasks non-runnable when executor drops
            })
            .unwrap();

        executor.join().unwrap();
    }

    let final_fds = count_open_fds();
    let leaked = final_fds.saturating_sub(initial_fds);

    println!("FD count after executor with tasks: {} -> {} (leaked: {})",
             initial_fds, final_fds, leaked);

    // Even with non-runnable tasks, eventfds should be closed
    assert!(
        leaked < 5,
        "FDs leaked with non-runnable tasks: {}",
        leaked
    );
}
