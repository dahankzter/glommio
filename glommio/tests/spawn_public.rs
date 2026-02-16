// Test for public spawn() method on LocalExecutor (safe scoped API)

use glommio::LocalExecutor;

#[test]
fn test_spawn_before_run() {
    // Spawn with safe scoped API before run() starts
    let executor = LocalExecutor::default();

    // Spawn task with scope before calling run()
    let task = executor.spawn(|scope| async move { scope.spawn(async { 42 }).await });

    // Now run the executor and await the task
    let result = executor.run(task);

    assert_eq!(result, 42);
}

#[test]
fn test_spawn_multiple_tasks() {
    let executor = LocalExecutor::default();

    // Spawn multiple tasks before run() using the safe scoped API
    let task1 = executor.spawn(|scope| async move { scope.spawn(async { 1 }).await });
    let task2 = executor.spawn(|scope| async move { scope.spawn(async { 2 }).await });
    let task3 = executor.spawn(|scope| async move { scope.spawn(async { 3 }).await });

    // Run and collect results
    let result = executor.run(async move {
        let r1 = task1.await;
        let r2 = task2.await;
        let r3 = task3.await;
        r1 + r2 + r3
    });

    assert_eq!(result, 6);
}

#[test]
fn test_spawn_with_computation() {
    let executor = LocalExecutor::default();

    // Spawn a task that does actual work
    let task = executor.spawn(|scope| async move {
        scope
            .spawn(async {
                let mut sum = 0;
                for i in 1..=100 {
                    sum += i;
                }
                sum
            })
            .await
    });

    let result = executor.run(task);

    assert_eq!(result, 5050); // Sum of 1 to 100
}

#[test]
fn test_spawn_inside_run_still_works() {
    let executor = LocalExecutor::default();

    // spawn() works inside run() context using the safe API
    let result = executor.run(async move {
        glommio::executor()
            .spawn(|scope| async move { scope.spawn(async { 42 }).await })
            .await
    });

    assert_eq!(result, 42);
}

#[test]
fn test_spawn_with_multiple_scoped_tasks() {
    let executor = LocalExecutor::default();

    // Test spawning multiple tasks within a single scope
    let result = executor.run(async move {
        glommio::executor()
            .spawn(|scope| async move {
                let h1 = scope.spawn(async { 10 });
                let h2 = scope.spawn(async { 20 });
                let h3 = scope.spawn(async { 30 });

                h1.await + h2.await + h3.await
            })
            .await
    });

    assert_eq!(result, 60);
}

// This test should NOT compile if uncommented - demonstrates !Send enforcement
/*
#[test]
fn test_cannot_send_executor_between_threads() {
    let executor = LocalExecutor::default();

    // This should fail to compile with:
    // error[E0277]: `Rc<RefCell<ExecutorQueues>>` cannot be sent between threads safely
    std::thread::spawn(move || {
        executor.spawn(|_scope| async { 42 });
    });
}
*/
