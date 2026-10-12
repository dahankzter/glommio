# Model checking the SleepNotifier wake protocol with loom

**Status:** design, not started
**Target:** upstream `glommio/glommio` `main`, offered there as a PR per phase
**Longer goal:** formal verification in Lean. This is the first verification
style tooling glommio has, and it starts where the tooling is cheapest and the
bugs have been: cross-thread protocols, not data structures.

## Why this protocol

`SleepNotifier` (`glommio/src/sys/mod.rs`) is how a thread that is not the
executor wakes one of its tasks. Recent bugs in cross-thread signalling were
each found by a person reasoning about interleavings rather than by a test:

- #89's coalescing gate, `notify(force_notify && first)`, drops the forced
  eventfd write when an ordinary wake set `foreign_pending` first. A
  `Latency::Matters` task then waits for the preempt timer, 86 to 102 ms against
  2 ms on `main`. This is the protocol modelled here.
- The executor park hang (#53), a sleep decision taken while work was still
  runnable.
- #62, a blocking closure that panicked and so never signalled the task
  awaiting it.

loom runs a test under every interleaving its scheduler can reach, so this
class of bug becomes a failing test instead of a review finding.

## The protocol

Producer side, any thread:

- `queue_waker(waker, force)` takes `foreign_wakes_lock`, checks
  `accepts_foreign_wakes`, sends the waker on a crossbeam unbounded channel,
  releases the lock, then calls `notify(force)`.
- `notify(force)` writes the eventfd if it can flip `should_notify` from true
  to false, which means the owner is going to sleep, or if `force` is set.
  `force` is the task's `latency_matters`, and a forced write is what makes
  `need_preempt()` true on a busy executor.

Owner side, the executor thread, in `wait()` in `sys/uring.rs`:

- `prepare_to_sleep()` stores `should_notify = true`.
- `membarrier::heavy()`.
- `process_remote_channels()`, which drains the channel through
  `process_foreign_wakes()`.
- If nothing was drained, sleep on the rings, then `wake_up()` stores
  `should_notify = false`.

Shutdown: `close_foreign_wakes()` takes the lock, sets `foreign_wakes_closed`,
releases it, and drops whatever is queued.

## What loom cannot see

loom only interleaves at operations on its own types. Three things the protocol
depends on are outside that:

| Dependency | Why loom cannot see it | Phase 1 | Phase 2 |
|---|---|---|---|
| crossbeam channel | `crossbeam-channel` 0.5.17 uses `std` atomics; its `crossbeam_loom` cfg covers `crossbeam-utils` and `crossbeam-epoch`, not the channel | trusted: runs as one indivisible step | replaced by a loom-visible queue under `cfg(loom)` |
| eventfd write | a syscall | a loom counter | same |
| `membarrier::heavy()` / `light()` | asymmetric barrier, no loom equivalent | `fence(SeqCst)` | same |

Trusting crossbeam is sound for interleaving bugs and blind to memory ordering
bugs: inside loom every channel operation is instantly visible to every thread,
which is stronger than the `Release`/`Acquire` plus `SeqCst` the real channel
uses.

## Phase 1: the protocol, with crossbeam trusted

### Seams

All of these are no-ops in a normal build.

1. **A sync shim.** `sys/sync.rs` re-exports `AtomicBool`, `AtomicU64`,
   `Mutex` and `fence` from `loom` under `cfg(loom)` and from `std` otherwise.
   `SleepNotifier` imports its atomics and its lock from there, so loom
   interleaves at `should_notify`, `foreign_wakes_closed` and
   `foreign_wakes_lock`.
2. **The eventfd.** `notify` calls a private `signal()`. In a normal build it
   is `write_eventfd`. Under `cfg(loom)` it increments a loom `AtomicU64` field
   that exists only in that build. The real eventfd stays in the struct so the
   rest of the crate compiles unchanged under `cfg(loom)`.
3. **membarrier.** Under `cfg(loom)`, `heavy()` and `light()` are both
   `fence(SeqCst)`.
4. **The owner's sleep sequence moves into `SleepNotifier`.** Today it is
   inline in `wait()`, which needs io_uring and cannot run under loom. A test
   that copied the sequence would check the copy, and a mutation to `wait()`
   would never reach it. So the sequence becomes a method, roughly
   `sleep_unless_woken(drain, sleep)`, that `wait()` calls with closures for
   draining the remote channels and for sleeping on the rings. It must keep the
   current behaviour exactly, including that `wake_up()` runs only on the path
   that slept. This is the one change normal builds see, and it is a pure
   refactor.

### The model

A unit test module `sys/loom_tests.rs`, `cfg(all(test, loom))`, so it can
reach the `pub(crate)` API. Each test is a `loom::model` with one owner thread
and one or two producer threads. Wakers count their `wake` calls and their
drops through `std` atomics, which loom does not need to see.

The notifier is created through `new_sleep_notifier()`, not
`SleepNotifier::new`, because `Drop` unregisters it from `REACTOR_GLOBAL_STATE`
and panics on an id that was never registered.

In the model the owner never blocks. "Sleep" records that the owner decided to
sleep, and the properties are checked after every thread has joined.

### Properties

- **P1, no lost wake.** If the owner decided to sleep, then every waker sent
  was either woken by the owner or the eventfd counter is above zero, so a real
  owner would wake and drain it.
- **P2, a forced wake gets through.** Every waker sent with `force` set was
  either woken by the owner or the eventfd counter is above zero. This is the
  property #89 breaks, with the owner awake throughout.
- **P3, nothing left behind after close.** Once `close_foreign_wakes` has
  returned and every producer has finished, the channel is empty, and each
  waker was woken or dropped exactly once.

### Evaluation

The risk is a model that passes because it never reaches the interesting
states. So it is judged by mutations of the real code. Each mutation is applied
by exact-string match with an asserted match count of one, so it cannot fail to
apply silently, and each must make loom fail with a counterexample:

| Mutation | Must break |
|---|---|
| #89's gate: add `foreign_pending`, and `notify(force_notify && first)` | P2 |
| `notify` writes only when forced, never on the `should_notify` flip | P1 |
| owner drains before `prepare_to_sleep` instead of after | P1 |
| `accepts_foreign_wakes` checked before taking `foreign_wakes_lock` | P3 |

Phase 1 is done when:

1. All properties hold on upstream `main`.
2. Every mutation fails, within the preemption bound CI will run with. The
   bound (`LOOM_MAX_PREEMPTIONS`), the iterations loom explored and the runtime
   are recorded here. Target: under a minute for the whole module.
3. The model calls the real `SleepNotifier` methods. No protocol step is
   restated in the test.
4. A normal build is unaffected: everything new is behind `cfg(loom)` except
   the `sleep_unless_woken` extraction.

A mutation that passes means the model does not explore enough, and nothing
else it says is believed until that is fixed.

### Build plumbing

- `[target.'cfg(loom)'.dependencies] loom = "0.7"` in `glommio/Cargo.toml`.
  loom 0.7.2 is current.
- `loom` added to the `unexpected_cfgs` check-cfg list.
- `make loom`, running
  `RUSTFLAGS="--cfg loom" cargo test --release -p glommio --lib loom_tests`.
  The filter matters: the whole library builds under `cfg(loom)`, and any other
  test touching a `SleepNotifier` outside `loom::model` would panic on the loom
  types.
- A CI job running the same, offered upstream once the module is stable.

## Phase 2: memory ordering, with a loom-visible queue

Phase 2 answers one question. The owner sets `should_notify` and then reads the
channel. A producer writes the channel and then reads `should_notify`. Every
`should_notify` access is `Relaxed`. The owner side has `membarrier::heavy()`
between its two steps. The producer side calls nothing: `membarrier::light()`
is defined in `sys/membarrier.rs` and never called. That is the store buffering
shape, and under the C11 model it may let both sides miss each other, so the
owner sleeps with a waker queued.

On x86 the producer's `compare_exchange` is a locked instruction and acts as a
full fence, so this is probably safe on that hardware. Whether it is sound
under the language's memory model is the open question. **Unverified; a
hypothesis.**

### The queue

Under `cfg(loom)` the channel type is an alias for a loom-visible queue in
`sys/loom_queue.rs` with the same `unbounded`, `send` and `try_recv` surface,
so no call site changes. crossbeam stays the production channel.

The queue must be as strong as crossbeam and no stronger. From
`crossbeam-channel` 0.5.17 `src/flavors/list.rs`:

- `send` claims its slot with a `SeqCst` compare-and-swap on the tail index,
  line 284.
- `try_recv` on an apparently empty channel runs `fence(SeqCst)` and then a
  `Relaxed` load of the tail, lines 349 and 350.

Each atomic in the mock cites the crossbeam line it reproduces.
`Mutex<VecDeque>` is ruled out: the lock orders owner and producer in a way
crossbeam does not, and would close the very window being checked.

### loom's own limits

From loom's README:

- `SeqCst` loads, stores and read-modify-writes are treated as `AcqRel`, which
  can raise false alarms.
- `fence(SeqCst)` is modelled properly.
- Load buffering is not explored, so a pass is not a proof.

The `SeqCst` compare-and-swap in crossbeam's `send` is exactly the kind of
access loom weakens. A store buffering report is therefore not believed on loom
alone.

### Evaluation

The queue's fidelity:

- Review against crossbeam, line by line, using the citations.
- A message passing litmus test: a receiver that sees a message sees its
  contents. Must pass.
- Removing the mock's `SeqCst` fence from the empty path must change the
  outcome of a store buffering litmus test, showing the fence carries weight in
  the model.

A store buffering finding counts only if all three hold:

1. loom produces a readable counterexample.
2. It disappears when the producer gets the missing `light()`, modelled as a
   fence.
3. It survives a mock made deliberately stronger than crossbeam, a `SeqCst`
   fence after the tail compare-and-swap, which takes loom's `SeqCst`
   weakening out of the picture.

Outcomes:

- A finding that meets all three becomes an upstream issue with the trace.
- A pass is written up here: the producer side is safe without `light()`,
  under this model.
- If the mock's fidelity cannot be established, phase 2 is abandoned and
  phase 1 stands alone.

## Out of scope

- The `notify(false)` calls in `channels/shared_channel.rs`. Same eventfd,
  different queue; a candidate for a later model.
- io_uring, eventfd reads, and executor scheduling.
- Lean. This is groundwork: properties stated precisely, and a habit of
  checking them by mutation, carry over to a proof later.
