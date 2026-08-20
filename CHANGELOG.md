# Changelog

`glommio-ng` is a republish of the community [glommio](https://github.com/glommio/glommio)
fork. Versions track the fork's own `0.10.0`; the patch number belongs to this
republish and is applied at release time, so a bump here does not imply a
change upstream.

Dependency line, unchanged across every version below:

```toml
glommio = { package = "glommio-ng", version = "0.10" }
```

`glommio-ng-macros` is published in lockstep from `0.10.2` onward and is pulled
in automatically by the default `macros` feature. You do not depend on it
directly.

## 0.10.10 — 2026-08-20

### Added

- **Cross-core cancellation.** `CancellationToken::foreign_child()` returns a
  `Send + Clone` handle; `ForeignCancellation::attach()` turns it back into an
  ordinary token on the destination executor. The token itself stays `!Send`,
  and an attached token supports `child_token()` and `cancelled()` like any
  other, so nothing downstream of it need know another core exists.

  This closes the shape a control plane on one core needs to stop work on the
  others, which previously had no answer and pushed projects back to
  `tokio_util`.

  Two cases are settled deliberately, both drawn from a real shutdown path
  where they sit two lines apart:

  - attaching after the origin cancelled yields an already-cancelled token;
  - attaching after the origin was **dropped** does too. Once the last origin
    handle is gone nothing can ever cancel that state, so anything else is a
    permanent hang.

  Note the asymmetry with local tokens, which is deliberate: dropping a local
  parent leaves its children alive and cancellable through their own handles.
  Dropping the origin of a `ForeignCancellation` cancels everything attached
  from it, because no handle to cancel it through survived the crossing.

  Cancellation is asynchronous across the boundary: `cancel()` returns
  immediately, and the attached token becomes cancelled once its executor polls
  the task watching for it.

## 0.10.9 — 2026-08-20

Asks from a downstream that has removed tokio's executor entirely and reported
what it hit afterwards.

### Added

- `OnceCell::get_or_try_init` — for lazily-initialised resources that can fail,
  which is most of them. **A failed initialiser does not poison the cell:** the
  next caller, queued or later, runs its own. One transient failure must not
  leave a permanently dead cell.
- `Stream` for `broadcast::Receiver` and `watch::Receiver`, which
  `local_channel` and `shared_channel` already had. `broadcast` yields
  `Result<T, RecvError>` rather than skipping quietly — a stream that hid
  `Lagged(n)` would turn a detectable gap into a silent one.

### Deprecated

- `timer::timeout`, in favour of `future::timeout`, which accepts any future
  rather than only one returning glommio's `Result`. Two public functions with
  the same name and different contracts is a trap for whoever greps first. At
  the next breaking release the name moves and this becomes `try_timeout`.

### Documentation

- [Porting from tokio](docs/PORTING_FROM_TOKIO.md), organised around
  differences that fail **silently**. It leads with `Task` cancelling on drop,
  and with the case `#[must_use]` cannot catch: the attribute does not survive
  a newtype wrapper, so a runtime-agnostic library that wraps `Task` in its own
  handle type gets no warning at all. That cost one downstream more time than
  anything else in their port.

## 0.10.8 — 2026-08-20

### Fixed

- **The crate did not compile for musl targets.** `libc` does not define
  `AT_STATX_SYNC_AS_STAT` there; it is `0` in the kernel UAPI and is now
  spelled out, so both libcs take the same path. Verified by building and
  running on Alpine: with the C removal in 0.10.6, glommio itself needs no C
  toolchain, so `musl-dev` is required only for the linker that any Rust build
  script needs.
- Two paths that only `--all-features` compiles: `schedule_runnable` gated its
  raw-pointer access on the `native-tls` feature without the `nightly` cfg, and
  `task::debugging` compared the header's `u32` executor id against a `usize`
  accessor. Neither affects a default build.

## 0.10.7 — 2026-08-20

### Fixed

- **A panicking `spawn_blocking` closure no longer hangs its caller.** The
  closure wrote its result straight into a `MaybeUninit` on the pool thread; a
  panic skipped that write and unwound the worker, so no response was ever
  sent. The awaiting future hung forever and the pool lost a thread
  permanently — four panics exhausted a four-thread pool. The panic now
  surfaces where the caller awaited. **This bug is present in every earlier
  glommio release, including the canonical 0.9.0.**

### Added

- `ExecutorProxy::spawn_blocking_send` — same pool, but the returned future is
  `Send` and does not depend on the executor that created it. Lets a per-core
  client satisfy a trait demanding `Pin<Box<dyn Future + Send>>` without
  running a second thread pool. Panics are caught and re-raised at the await
  point.

## 0.10.6 — 2026-08-20

### Changed

- **No C toolchain, `make`, `configure` or submodule required.** The vendored
  `liburing` and the five C files it fed were deleted: nothing in the crate
  referenced them once the `io-uring` crate migration completed. The package
  went from 630 files and 3.9 MB to 110 files and 1.6 MB.

## 0.10.5 — 2026-08-20

### Added

- `future::timeout` — races any future against a deadline, whatever it returns.
  `timer::timeout` only accepts futures already returning a glommio `Result`.
  Both now document that the inner future is polled before the timer, so one
  ready exactly at the deadline reports success.
- `timer::interval`, `timer::interval_at`, `Interval`, `MissedTickBehavior` —
  a pollable stream of ticks, first one immediate, usable in a `select!` arm.
  `Burst`, `Delay` and `Skip` are three genuinely different schedules.

### Note

At the next breaking release `timer::timeout` becomes `try_timeout` and
`future::timeout` takes the plain name, matching the ecosystem's `try_`
convention for `Result`-aware combinators.

## 0.10.4 — 2026-08-20

### Added

- `sync::Mutex` — an async mutex for state held across an `await`. Prefer
  `RefCell` where no borrow crosses one.
- `sync::OnceCell` — initialised at most once by an initialiser that may
  itself await; concurrent callers wait for the first rather than running
  their own.
- `sync::CancellationToken` — hierarchical cancellation within one executor.
  `!Send` by design: cross-shard shutdown is one root token per executor.
- `channels::oneshot` — a channel carrying exactly one value; an unsent value
  is handed back rather than dropped.
- `channels::watch` — keeps only the latest value; slow receivers skip rather
  than queue.
- `channels::broadcast` — multi-consumer fan-out with `RecvError::Lagged(n)`
  for receivers that fall behind. Semantics mirror tokio's deliberately.
- `ConnectedSender::into_foreign` — a `Send`, non-`Clone`, `try_send`-only
  handle usable from a thread with no executor. Not `Clone` on purpose: the
  buffer underneath is strictly single-producer.
- `Semaphore::is_closed`.

## 0.10.3 — 2026-08-19

### Fixed

- **`#[glommio::main(name = "…")]` is now rejected rather than silently
  ignored.** `LocalExecutorBuilder::name` is only read by `spawn()`; the
  macros build with `make()`, which never reads it, so the argument was
  accepted and discarded. It now fails to compile with a message pointing at
  `spawn()`.

### Documentation

- `crate = …` accepts a path, not just an identifier, so a crate reaching
  glommio through a facade can point at the re-export.

## 0.10.2 — 2026-08-19

### Added

- `#[glommio::main]` and `#[glommio::test]`, from the new `glommio-ng-macros`
  crate, behind a default-on `macros` feature. Arguments: `placement` and
  `crate`. `#[glommio::test]` emits a plain `#[test]`, so `#[should_panic]`,
  `#[ignore]` and `Result` returns compose.

### Note

Re-exporting a macro named `test` means `use glommio::*` shadows the built-in
`#[test]`. Import the macros by name, or write `#[glommio::test]`.

## 0.10.1 — 2026-08-19

### Fixed

- **A kernel that cannot run glommio now returns an error instead of calling
  `exit(1)` on your process.** The failure names the missing `IORING_OP_*`, or
  reports `kernel.io_uring_disabled` by value, or points at a container seccomp
  policy — the three ways this fails on RHEL and in containers.
- The documented kernel floor was 5.8; the newest operation glommio submits
  landed in 5.6.

## 0.10.0 — 2026-08-19

First release. The canonical `glommio` crate last published 0.9.0 in March
2024, and 0.9.0 no longer compiles against current kernel headers, leaving
downstreams pinned to git revisions.

### Fixed

- **`cargo package` was impossible.** `build.rs` ran `./configure` inside the
  vendored `liburing`, mutating the crate's own source directory, which cargo
  rejects outright. Configuring outside the source tree made the crate
  publishable at all. (Superseded in 0.10.6, which removed the C entirely.)
