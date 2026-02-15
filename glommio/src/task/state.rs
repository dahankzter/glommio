// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2020 Datadog, Inc.
//
/// Set if the task is scheduled for running.
///
/// A task is considered to be scheduled whenever its [`Task`] reference exists.
/// It therefore also begins in scheduled state at the moment of creation.
///
/// This flag can't be set when the task is completed. However, it can be set
/// while the task is running, in which case it will be rescheduled as soon as
/// polling finishes.
pub(crate) const SCHEDULED: u8 = 1 << 0;

/// Set if the task is running.
///
/// A task is in running state while its future is being polled.
///
/// This flag can't be set when the task is completed. However, it can be in
/// scheduled state while it is running, in which case it will be rescheduled as
/// soon as polling finishes.
pub(crate) const RUNNING: u8 = 1 << 1;

/// Set if the task has been completed.
///
/// This flag is set when polling returns `Poll::Ready`. The output of the
/// future is then stored inside the task until it becomes closed. In fact,
/// [`JoinHandle`] picks up the output by marking the task as closed.
///
/// This flag can't be set when the task is scheduled or running.
pub(crate) const COMPLETED: u8 = 1 << 2;

/// Set if the task is closed.
///
/// If a task is closed, that means it's either canceled or its output has been
/// consumed by the [`JoinHandle`]. A task becomes closed when:
///
/// 1. It gets canceled by [`Task::cancel()`], [`Task::drop()`], or
///    [`JoinHandle::cancel()`]. 2. Its output gets awaited by the [`JoinHandle`].
/// 3. It panics while polling the future.
/// 4. It is completed and the [`JoinHandle`] gets dropped.
pub(crate) const CLOSED: u8 = 1 << 3;

/// Set if the [`JoinHandle`] still exists.
///
/// The [`JoinHandle`] is a special case in that it is only tracked by this
/// flag, while all other task references ([`Task`] and [`Waker`]s) are tracked
/// by the reference count.
pub(crate) const HANDLE: u8 = 1 << 4;

/// Set if the task was allocated from the arena.
///
/// This flag is set during task allocation if the memory came from the arena
/// allocator rather than the heap. It's crucial for correct deallocation:
/// - If set: Try to recycle to arena (if still in scope), else skip deallocation
///   (memory already freed when arena dropped)
/// - If clear: Safe to deallocate via heap allocator
///
/// This solves the "free(): invalid pointer" crash when tasks outlive the arena
/// scope. The arena might be dropped before the task, so we can't rely on
/// `TASK_ARENA.is_set()` to determine allocation source during deallocation.
pub(crate) const ARENA_ALLOCATED: u8 = 1 << 5;
