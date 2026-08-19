# Completing the Synchronisation Set

**Date:** 2026-08-19
**Status:** planned, not started
**Origin:** slipstream runs entirely on glommio but still imports `tokio::sync`
for primitives glommio does not have. Call-site counts below are from its
`src/`.

## The argument

Not performance. `tokio::sync` needs no reactor — it parks on wakers and is
driven by whatever polls it — which is exactly why slipstream's port worked.
The atomics in those channels sit next to syscalls and network round trips and
do not show up.

The argument is typing. A `!Send` primitive makes "this stays on one core" a
compile error instead of a convention. Anything that does not buy that is not
worth building here.

## Already covered — do not build

- **`ReceiverStream`.** Both channel types already implement `Stream`:
  `ConnectedReceiver` directly (`shared_channel.rs:427`), `local_channel` via
  `.stream()` (`local_channel.rs:731`).
- **`CancellationToken`** (78 sites). `tokio-util`'s is runtime-agnostic and
  cancellation legitimately crosses threads, so a `!Send` version buys nothing.
  Keep the dependency.
- **`RwLock`** (5 sites), **`Semaphore`**, **`Gate`** — glommio has these.
- **`Barrier`, `Notify`, `Semaphore`** are re-exported by slipstream's facade
  but unused; they should be trimmed there rather than implemented here.

## Order of work — simplest first

### 1. Async `Mutex` (28 sites)

`Semaphore` already carries the waiter queue. A `Mutex` is that with
`units = 1`, plus a guard giving `Deref`/`DerefMut` over an `UnsafeCell<T>`.
Follow `rwlock.rs` for the guard shape and the poisoning stance.

Open question: whether `lock()` returns `Result` like `RwLock` does here, or
infallible like tokio's. Match `RwLock` for consistency within the crate.

### 2. `oneshot` (14 sites)

Request/response pairing in front of per-core workers. A bounded
`local_channel` of 1 works and says the wrong thing.

Open questions: sender dropped without sending — `RecvError` or `None`;
whether the receiver is cancel-safe on drop.

### 3. `watch` (1 site) and `OnceCell` (4 sites)

Small, and worth doing once the two above establish the module conventions.
`watch` is a `RefCell<T>` plus a waker list and a version counter.

### 4. `broadcast` (30 sites) — the real project, needs a design pass

Multi-consumer fan-out with lagging-receiver semantics: every subscriber sees
every value, and a slow one gets `Lagged(n)` rather than blocking the sender.
No substitute exists — `local_channel` is single-consumer. This is the item
slipstream most wants.

Single-threaded makes the mechanics easier than tokio's — `Rc`/`RefCell`, no
atomics, no `Arc` traffic — but the semantics are the difficulty:

- ring size, and what happens when a receiver lags past the whole buffer
- whether a receiver subscribing later sees backlog or only new values
- what the sender observes when every receiver has gone away
- whether `Lagged` is recoverable, and where the cursor lands after it

Brainstorm before writing.

### 5. `shared_channel` usable from a thread with no executor — investigate first

Both ends currently require `connect()` inside a glommio executor
(`shared_channel.rs:163,328`). Slipstream's two facade sites (object_store's
HTTP connector, the gRPC channel) have a `Send` caller with no executor at all
on one end.

This is a different shape from everything above and the wake path needs
understanding before anything is promised.

## Separate axis, and ranked above all of the above

**`spawn_blocking` returns a `!Send` future** (`executor/mod.rs:2920`). The
closure and result cross threads; the future does not, because it awaits a
reactor source from `LOCAL_EX`. Documented in `e2f2aaf`, not fixed.

It is the only item here with a measured downstream cost: `deboa-glommio`
cannot satisfy a `Pin<Box<dyn Future + Send>>` resolver trait, so it runs a
second thread pool inside a thread-per-core process to wrap `getaddrinfo`.

A channel-backed variant whose future is `Send` looks feasible — a cross-thread
`waker.wake()` is fine — but cancellation on drop and the pool's ownership of
the result need design. Everything else on this page is a stylistic
improvement to code that already works; this one is a capability gap.

## Cost to keep in mind

Every item is new public API on a fork whose upstream has six unreviewed PRs
from us. `sync/` additions are additive and conflict-poor, so the rebase cost
is small — but "upstream may never take this" applies to all of it.
