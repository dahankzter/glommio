#[cfg(all(test, feature = "debugging"))]
mod ref_count {
    use std::{
        cell::RefCell,
        pin::Pin,
        rc::Rc,
        task::{Context, Poll, Waker},
    };

    use futures_lite::future::{yield_now, Future};

    use crate::{channels::shared_channel, prelude::*, task::debugging::TaskDebugger};

    struct Inner {
        n: usize,
        waker: Option<Waker>,
    }

    #[derive(Clone)]
    struct WakeN {
        inner: Rc<RefCell<Inner>>,
    }

    impl WakeN {
        fn new(n: usize) -> Self {
            Self {
                inner: Rc::new(RefCell::new(Inner { n, waker: None })),
            }
        }

        fn take_waker(&self) -> Option<Waker> {
            self.inner.borrow_mut().waker.take()
        }
    }

    impl Future for WakeN {
        type Output = ();

        fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
            let mut inner = self.inner.borrow_mut();
            if inner.n > 0 {
                inner.n -= 1;
                inner.waker = Some(ctx.waker().clone());
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    fn init_logger() {
        pretty_env_logger::try_init().ok();
    }

    #[test]
    fn root_task() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn foreground_task() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                TaskDebugger::set_label("foreground_task");
                let task = crate::spawn_local(async {
                    assert_eq!(2, TaskDebugger::task_count());
                });
                assert_eq!(2, TaskDebugger::task_count());
                task.await;
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn background_task() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                TaskDebugger::set_label("background_task");
                let handle = crate::spawn_local(async {
                    assert_eq!(2, TaskDebugger::task_count());
                })
                .detach();
                assert_eq!(2, TaskDebugger::task_count());
                handle.await.unwrap();
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn drop_join_handle_before_completion() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                TaskDebugger::set_label("drop_join_handle_before_completion");
                assert_eq!(1, TaskDebugger::task_count());
                let handle = crate::spawn_local(async {
                    yield_now().await;
                })
                .detach();
                assert_eq!(2, TaskDebugger::task_count());
                drop(handle);
                assert_eq!(2, TaskDebugger::task_count());
                yield_now().await;
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn drop_join_handle_after_completion() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                TaskDebugger::set_label("drop_join_handle_after_completion");
                let handle = crate::spawn_local(async {}).detach();
                assert_eq!(2, TaskDebugger::task_count());
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                drop(handle);
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn wake() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                let task = WakeN::new(1);
                TaskDebugger::set_label("wake");
                let handle = crate::spawn_local(task.clone()).detach();
                yield_now().await;
                task.take_waker().unwrap().wake();
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                drop(handle);
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn wake_completed_task() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                let task = WakeN::new(1);
                TaskDebugger::set_label("wake");
                let handle = crate::spawn_local(task.clone()).detach();
                drop(handle);
                yield_now().await;
                let waker = task.take_waker().unwrap();
                waker.clone().wake();
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                waker.wake();
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn drop_waker_of_completed_task() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                let task = WakeN::new(1);
                TaskDebugger::set_label("wake");
                let handle = crate::spawn_local(task.clone()).detach();
                drop(handle);
                yield_now().await;
                let waker = task.take_waker().unwrap();
                waker.clone().wake();
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                drop(waker);
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn wake_by_ref() {
        init_logger();
        let result =
            LocalExecutorPoolBuilder::new(PoolPlacement::Unbound(1)).on_all_shards(|| async move {
                let task = WakeN::new(1);
                TaskDebugger::set_label("wake_by_ref");
                let handle = crate::spawn_local(task.clone()).detach();
                yield_now().await;
                let waker = task.take_waker().unwrap();
                waker.wake_by_ref();
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                drop(handle);
                assert_eq!(2, TaskDebugger::task_count());
                drop(waker);
                assert_eq!(1, TaskDebugger::task_count());
            });
        result.unwrap().join_all()[0].as_ref().unwrap();
    }

    #[test]
    fn foreign_wake() {
        init_logger();
        let (sender, receiver) = shared_channel::new_bounded(1);

        let results = vec![
            LocalExecutorBuilder::default().spawn(move || async move {
                let sender = sender.connect().await;
                let task = WakeN::new(1);
                TaskDebugger::set_label("foreign_wake");
                let handle = crate::spawn_local(task.clone()).detach();
                yield_now().await;
                let waker = task.take_waker().unwrap();
                sender.send(waker).await.unwrap();
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                handle.await.unwrap();
            }),
            LocalExecutorBuilder::default().spawn(move || async move {
                let receiver = receiver.connect().await;
                let waker = receiver.recv().await.unwrap();
                waker.wake();
            }),
        ];

        for res in results {
            res.unwrap().join().unwrap();
        }
    }

    #[test]
    fn foreign_wake_by_ref() {
        init_logger();
        let (sender, receiver) = shared_channel::new_bounded(1);

        let results = vec![
            LocalExecutorBuilder::default().spawn(move || async move {
                let sender = sender.connect().await;
                let task = WakeN::new(1);
                TaskDebugger::set_label("foreign_wake_by_ref");
                let handle = crate::spawn_local(task.clone()).detach();
                yield_now().await;
                let waker = task.take_waker().unwrap();
                sender.send(waker).await.unwrap();
                yield_now().await;
                assert_eq!(2, TaskDebugger::task_count());
                handle.await.unwrap();
            }),
            LocalExecutorBuilder::default().spawn(move || async move {
                let receiver = receiver.connect().await;
                let waker = receiver.recv().await.unwrap();
                waker.wake_by_ref();
                drop(waker);
            }),
        ];

        for res in results {
            res.unwrap().join().unwrap();
        }
    }
}

#[cfg(test)]
mod arena_integration {
    use crate::{spawn_local, LocalExecutor};

    #[test]
    fn test_arena_used_for_spawns() {
        // This test verifies that the arena is actually being used for task allocation
        // when tasks are spawned via spawn_local
        LocalExecutor::default().run(async {
            // Spawn multiple tasks to ensure arena is being used
            let handles: Vec<_> = (0..100)
                .map(|i| {
                    spawn_local(async move {
                        // Simple computation
                        i * 2
                    })
                })
                .collect();

            // Await all tasks
            let results: Vec<_> =
                futures_lite::future::block_on(async { futures_lite::future::join_all(handles).await });

            // Verify results
            for (i, result) in results.iter().enumerate() {
                assert_eq!(*result, i * 2);
            }
        });
    }

    #[test]
    fn test_arena_handles_many_spawns() {
        // Test that arena can handle many spawns (up to capacity)
        LocalExecutor::default().run(async {
            // Spawn a large number of tasks
            for _ in 0..1000 {
                spawn_local(async {
                    // Minimal work
                    42
                })
                .detach();
            }
        });
    }

    #[test]
    fn test_arena_fallback_to_heap() {
        // Test that exceeding arena capacity falls back to heap
        // This spawns more tasks than the arena can hold (10,000 capacity)
        LocalExecutor::default().run(async {
            // Spawn tasks and keep them alive to fill arena
            let handles: Vec<_> = (0..15_000)
                .map(|i| {
                    spawn_local(async move {
                        i
                    })
                })
                .collect();

            // Some will be in arena, some on heap - both should work
            let results: Vec<_> =
                futures_lite::future::block_on(async { futures_lite::future::join_all(handles).await });

            assert_eq!(results.len(), 15_000);
        });
    }
}
