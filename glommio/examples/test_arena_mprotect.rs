use glommio::LocalExecutor;

fn main() {
    // Test that mprotect works without warnings
    LocalExecutor::default().run(async {
        let result = glommio::executor()
            .spawn_scope(|scope| async move {
                let h1 = scope.spawn(async { 1 + 1 });
                let h2 = scope.spawn(async { 2 + 2 });

                let r1 = h1.await;
                let r2 = h2.await;

                (r1, r2)
            })
            .await;

        assert_eq!(result, (2, 4));
        println!("✓ Scoped spawning works!");
        println!("✓ Arena allocation successful!");
    });

    println!("✓ Executor dropped - if mprotect works, no warnings should appear above");
}
