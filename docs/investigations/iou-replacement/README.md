# Investigation: Replacing the Vendored `iou` / `uring_sys`

**Date:** 2026-08-01
**Status:** surveyed and scoped, not attempted
**Motivation:** delete a third of the fork's unsafe surface, and stop being
frozen at a 2020 view of io_uring

## Why this is on the table

`glommio/src/iou` and `glommio/src/uring_sys` are not dependencies. They are a
copy of the `iou` and `uring-sys` crates taken into the tree in commit
`447a38e` ("import uring-sys and iou"), and maintained by hand since.

Upstream `iou` is at **0.3.3** and `uring-sys` at **1.0.0-beta**. Neither has
the setup flags glommio needs, so *there is nothing to upgrade to* — the
vendored copy is not behind its upstream, its upstream is dead. Meanwhile the
`liburing` submodule is current (**liburing-2.13**), which is why the C header
in this repository declares `IORING_SETUP_*` flags to bit 18 while the Rust
wrapper beside it stops at bit 5.

| file | lines | `unsafe` |
|---|---:|---:|
| `iou/sqe.rs` | 757 | 38 |
| `iou/registrar/mod.rs` | 327 | 15 |
| `iou/registrar/registered.rs` | 366 | 14 |
| `iou/mod.rs` | 361 | 8 |
| `iou/cqe.rs` | 213 | 8 |
| `iou/completion_queue.rs` | 120 | 8 |
| `iou/submission_queue.rs` | 117 | 10 |
| `iou/probe.rs` | 39 | 4 |
| `uring_sys/mod.rs` | 702 | 0 |
| `uring_sys/syscalls.rs` | 40 | 3 |
| **total** | **3,042** | **108** |

**108 of glommio's 309 `unsafe` occurrences — 35% — are in hand-maintained FFI
for an abandoned crate.** Deleting it would do more for the "reduce unsafe" goal
than everything proposed in [unsafe-centralization](../unsafe-centralization/),
which spends its effort relocating unsafe rather than removing it.

The replacement is `io-uring` (tokio-rs), actively maintained, currently 0.7.13.
Every probe in the [mechanical-sympathy](../mechanical-sympathy/) investigation
is written against it, including `MSG_RING` and the modern setup flags, so it is
already known to cover what glommio needs.

## Why it is not a dependency swap

The two APIs are structurally different.

**`iou` hands you a slot to fill in place:**

```rust
if let Some(mut sqe) = self.ring.sq().prepare_sqe() {
    sqe.prep_read(op.fd, buf, pos);
    sqe.set_user_data(user_data);
}
```

**`io-uring` builds an entry and pushes it:**

```rust
let entry = opcode::Read::new(types::Fd(fd), ptr, len)
    .offset(pos)
    .build()
    .user_data(user_data);
unsafe { ring.submission().push(&entry)?; }
```

So `fill_sqe` cannot be translated line by line: it changes from
`fn fill_sqe(&mut SQE, ...)` to `fn build_sqe(...) -> squeue::Entry`, and every
caller that currently acquires-then-fills has to become build-then-push.

**The borrow shapes differ too.** `io-uring`'s `submission()` and `completion()`
take `&mut self`. `UringCommon` reaches them through `&self`:

```rust
fn waiting_kernel_submission(&self) -> usize { self.ring.sq().ready() as usize }
fn waiting_kernel_collection(&self) -> usize { self.ring.cq().ready() as usize }
```

Either those trait methods become `&mut self` — a refactor rippling through
`Reactor` — or they use `submission_shared()` / `completion_shared()`, which are
`unsafe` and would reintroduce exactly what this exercise exists to remove.
**Take the `&mut self` route**; using the shared accessors defeats the purpose.

## Surface to convert

22 opcodes, all of which exist in `io-uring` 0.7:

| glommio | `io-uring` |
|---|---|
| `prep_poll_add` / `prep_poll_remove` | `opcode::PollAdd` / `PollRemove` |
| `prep_cancel` | `opcode::AsyncCancel` |
| `prep_read` / `prep_write` | `opcode::Read` / `Write` |
| `prep_read_fixed` / `prep_write_fixed` | `opcode::ReadFixed` / `WriteFixed` |
| `prep_openat` / `prep_close` | `opcode::OpenAt` / `Close` |
| `prep_fsync` / `prep_fallocate` | `opcode::Fsync` / `Fallocate` |
| `prep_statx` | `opcode::Statx` |
| `prep_connect` / `prep_accept` | `opcode::Connect` / `Accept` |
| `prep_send` / `prep_recv` | `opcode::Send` / `Recv` |
| `prep_sendmsg` / `prep_recvmsg` | `opcode::SendMsg` / `RecvMsg` |
| `prep_timeout` / `prep_timeout_remove` | `opcode::Timeout` / `TimeoutRemove` |
| `prep_link_timeout` | `opcode::LinkTimeout` |
| `prep_nop` | `opcode::Nop` |

Plus: `Registrar` → `Submitter::register_buffers` / `unregister_buffers`;
`uring_sys::io_uring_get_probe` + `io_uring_opcode_supported` →
`Submitter::register_probe` + `Probe::is_supported`; `SetupFlags` /
`SetupFeatures` → `IoUring::builder()`; `__kernel_timespec` →
`io_uring::types::Timespec`; and `SockAddrStorage`, a 12-line helper that simply
moves into `sys`.

Only **14 sites outside the vendored directories** name `iou::` at all, and 26
`prep_*` call sites — the coupling is shallow in breadth. The depth is in
`sys/uring.rs`, which is where essentially all of it lives.

## Where the risk is

Not in the opcode translation, which is mechanical. In these:

1. **Fixed buffers.** `ReadFixed` / `WriteFixed` plus registration is the DMA
   path. A wrong buffer index or a lifetime mistake here is silent data
   corruption, not a test failure.
2. **`IOPOLL`.** `PollRing` is built with `SetupFlags::IOPOLL` and has different
   completion semantics; it must keep them.
3. **Linked timeouts.** `LinkTimeout` depends on `IOSQE_IO_LINK` ordering
   between consecutive SQEs, so the build-then-push restructure must preserve
   submission order exactly. `submit_event_chain` exists for this reason.
4. **Cancellation.** The separate cancellation queue and `AsyncCancel` semantics.
5. **Miri cannot cover any of it** — the reactor hits unsupported syscalls, as
   recorded in [mechanical-sympathy](../mechanical-sympathy/). Validation is the
   test suite and nothing else.

## The one thing that does not translate: `SleepableRing::sleep`

Found while converting, and it is the reason the core step is not finished.

```rust
if let Some(mut sqe) = self.ring.sq().prepare_sqe() {
    let sqe_ptr = unsafe { sqe.raw_mut() as *mut _ };
    ...
    if self.submit_sqes()... != 1 {
        // We make the SQE a no-op and return
        unsafe { crate::uring_sys::io_uring_prep_nop(sqe_ptr) };
        Err(io::Error::from_raw_os_error(libc::EBUSY))
    }
```

`sleep` reserves an SQE, fills it with a poll that links the rings, and submits
it. If that submit fails it **rewrites the SQE it already prepared into a nop**,
so the ring is not left holding a poll the reactor never meant to arm, and then
declines to sleep.

`io-uring` cannot express this. `SubmissionQueue` offers `push`,
`push_multiple`, `sync`, `len`, `capacity` and `is_full` — **no rewind, no pop,
and no access to an entry once pushed**. Building an entry and pushing it are
separate steps, and after the push the entry is the queue's.

So this needs redesigning, not translating, and the options each need thought:

- Don't push until the submit is known to succeed — but submission failure
  (`EBUSY`, `EAGAIN`, `EINTR`) is only observable by attempting it.
- Push and let a failed submit leave the entry queued for the next submit. It is
  the same poll on the same fd, just later; whether that is safe depends on
  whether the reactor can reach a state where it is no longer wanted.
- Push a compensating `AsyncCancel` for the same `user_data` on the failure path.

**Get this wrong and the executor either sleeps when it should not — a hang — or
arms a poll on the link fd that nobody is expecting.** It deserves a careful
session with the sleep/wake path in front of you, not the tail end of one.

Work in progress sits on branch `refactor/iou-core` (pushed, not merged). It
does not compile: `fill_sqe` and `submit_event_chain` are converted, ~20
mechanical errors remain, plus this one.

## Suggested sequencing

Each step should leave the tree compiling and the suite green.

1. Move the leaf types out: `SockAddrStorage` into `sys`, `__kernel_timespec`
   into a local `repr(C)` struct or `io_uring::types::Timespec`. *(Attempted:
   this works, but it ripples into the `iou` `prep_*` boundary and only pays off
   as part of the whole, so it was reverted rather than landed as churn.)*
2. ✅ **Done** (`aab0278`): `UringCommon`'s queue accessors take `&mut self`.
   Pure refactor; compiled first try, suite green.
3. ✅ **Done** (`94aaeb7`): opcode probing moved to `io-uring`'s `Probe`,
   deleting `iou/probe.rs`. Independent of the submission path, so it landed on
   its own.
4. ~~Convert `PollRing` first~~ — **this does not decompose.** `submit_event_chain`
   reserves a contiguous run of SQEs via `prepare_sqes(n)` for linked chains, and
   both rings share it, so the submission path has to move as one piece.
   Resolve the `sleep` question above first, then convert `fill_sqe`,
   `submit_event_chain` and both rings together.
5. Convert the registrar and the fixed-buffer path. Slowest and most dangerous;
   do it last with the DMA tests in front of you.
6. Delete `glommio/src/iou` and `glommio/src/uring_sys`. Verify the `unsafe`
   count drops from 309 to roughly 200.
7. Only then add the setup flags
   ([candidate 4](../mechanical-sympathy/)), which is what prompted this.

## Honest estimate

This is a rewrite of glommio's io_uring interaction layer: on the order of
700-900 lines of `sys/uring.rs` reworked, against ~3,000 lines deleted. It is
several sessions of careful work, not one, and the validation available is a
test suite that no UB checker can supplement.

It is worth doing — 35% of the unsafe surface and an unfreezing of the whole
io_uring feature set is a large prize, and the fork's stated purpose is exactly
this kind of work. But it should be done as a dedicated piece with each step
green, not folded into an afternoon.
