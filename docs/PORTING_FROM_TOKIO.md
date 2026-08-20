# Porting from tokio: the differences that fail silently

Organised around **silence**. A compile error is cheap — you fix it and move
on. What costs days is correct tokio code that compiles under glommio and does
something else, with no error anywhere.

Everything here has cost somebody real hours.

## `Task` cancels when dropped; `JoinHandle` does not

The single most expensive difference.

```rust
// tokio: runs to completion. The handle is optional.
tokio::spawn(async { work().await });

// glommio: cancelled immediately. The work never happens.
glommio::spawn_local(async { work().await });

// glommio, intended:
glommio::spawn_local(async { work().await }).detach();
```

No panic, no error, no log. In one real case a table scan spawned its producer
tasks, dropped the handles, and returned **zero results for a table full of
data** — no failure of any kind, just nothing.

`Task` carries `#[must_use]` naming `.detach()`, which catches the direct case.

### It does not survive a newtype wrapper

**This is the case that actually bites, and the lint cannot see it.**

```rust
// A runtime-agnostic library wraps the handle in its own type:
pub struct JoinHandle<T>(glommio::Task<T>);

fn spawn<F>(f: F) -> JoinHandle<F::Output> { /* ... */ }

// The caller discards it. `must_use` is silent: it is on glommio::Task,
// not on your newtype.
spawn(async move { producer().await });
```

Every runtime-agnostic library has that wrapper — it is *what runtime-agnostic
means* — so the people most likely to hit this are structurally the people the
lint cannot reach.

**If you wrap `Task` in your own handle type, do one of these:**

- carry the attribute across:
  `#[must_use = "dropping this cancels the task; call .detach()"]`
- or `.detach()` inside the wrapper, and give your handle
  tokio's drop semantics deliberately.

## `shared_channel` needs `connect()` on both ends, inside an executor

Necessary and correct, but it is a runtime requirement the type does not state,
so the failure lands at runtime, far from the mistake. A sender or receiver
that is never connected simply never carries anything.

If one end has no executor — a thread you spawned yourself — use
[`ConnectedSender::into_foreign`], which yields a `Send` handle you can move
anywhere. It is `try_send`-only and deliberately not `Clone`: the buffer
underneath is single-producer, and cloning it across threads corrupts the heap.

## Cancellation does not cross cores

`sync::CancellationToken` is `Rc`-based and `!Send` on purpose: it cancels task
trees within one executor. A control plane on one core cannot hold a token that
cancels work on another.

Until a first-class shape exists, the working pattern is: send a `Send` signal
across (a `oneshot`, or `shared_channel`), and on the far core have a small
detached task cancel a *local* token when it fires.

## `timeout` exists twice

- [`future::timeout`] takes any future. Use this one.
- [`timer::timeout`] takes only futures returning glommio's `Result`, and is
  deprecated.

Both poll the inner future before the timer, so a future that completes exactly
at the deadline reports success rather than racing.

## `use glommio::*` shadows `#[test]`

Re-exporting a macro named `test` at the crate root means a glob import
shadows the built-in attribute. Import the macros by name, or write
`#[glommio::test]`. tokio has the same property.

## Things that are the same, and are worth not worrying about

- `tokio::select!` contains no runtime. It expands to a `poll_fn` over its
  branches and works fine over glommio futures. Keeping it is not a porting
  failure.
- `tokio::sync` primitives need no reactor either. They work; they are just
  `Send` when your program is per-core, so the type system stops helping you.
  Swapping them for glommio's buys typing, not speed.
- `std::time::Instant` is glommio's clock. There is no paused-time test mode
  and no bespoke `Instant`.

## For library authors: the seam argument

The crates that genuinely block a runtime swap are rarely runtimes. They are
libraries with one hardcoded call — a `tokio::time::sleep` in a retry path, a
sleeper trait with one implementation, a spawn buried in a constructor.

The productive request to those maintainers is never "port to glommio". It is
**a seam, not a rewrite**: an associated type, a trait with a default
implementation, an injectable sleeper. `object_store` already accepts a custom
`HttpConnector` and needs the equivalent for its sleeper. A one-associated-type
DNS change was accepted into `deboa` on exactly that framing.

A seam accepted upstream unblocks *every* runtime, not just this one, which is
why it is an easier argument to win than it looks.

[`ConnectedSender::into_foreign`]: https://docs.rs/glommio-ng/latest/glommio_ng/channels/shared_channel/struct.ConnectedSender.html#method.into_foreign
[`future::timeout`]: https://docs.rs/glommio-ng/latest/glommio_ng/future/fn.timeout.html
[`timer::timeout`]: https://docs.rs/glommio-ng/latest/glommio_ng/timer/fn.timeout.html
