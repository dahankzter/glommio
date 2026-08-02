# What the ~2 µs Per Blocking I/O Actually Is

**Date:** 2026-08-02
**Status:** attributed — **preempt-timer churn, not the I/O**
**Follows:** [synthesis.md](synthesis.md)

[synthesis.md](synthesis.md) established that glommio costs ~2 µs per operation
that blocks, and that the figure is the same on files and sockets. This
attributes it.

## Counts, per TCP echo round trip

Instrumented inside glommio, running the ladder's glommio case. Absolute times
are inflated by the timers themselves; **the counts are exact and are the
finding**.

| | per round trip |
|---|---:|
| `io_uring_enter` | **5.00** |
| completions processed | **9.00** |
| `Source`s created | 1.00 |

Nine completions to perform one read. Breaking them down by what they were:

| completion | count |
|---|---:|
| `PollAdd` — the actual read-readiness poll | **1.00** |
| `Timeout` — preempt timers | **3.00** |
| `user_data == 0` — cancellations and poll removes | **5.00** |
| `LinkRings` | 0.00 |
| `ForeignNotifier` | 0.00 |
| anything else | 0.00 |

**Eight of the nine completions are preempt-timer traffic.** Three timers firing
or being retired, five cancellations. One completion does the work.

The user-space handling is not what costs: `wake_waiters` measured 93 ns,
`consume_source` 70 ns, `Source::new` 30 ns, `add_source` 82 ns. The cost is the
kernel round trips those eight extra completions imply, and the enters that
submit and cancel them.

## Which explains earlier results

[reactor-loop.md](reactor-loop.md) found four of eight `io_uring_enter` calls on
the latency ring in a shard ping-pong, and that lengthening `preempt_timer` to 10
seconds changed nothing — because the cost is **installing and cancelling** a
timer every loop iteration, not the timer firing.

**Correction.** An earlier version of this document claimed the eventfd read
that wakes a parked shard rides into the kernel on the enter the preempt timer
forces, and that this is why removing the timer deadlocks. **That is not what the
code does.** `SleepableRing::install_eventfd` pushes the read and then calls
`submit_sqes()` itself, so the eventfd reaches the kernel at install time
regardless of the timer.

Removing the preempt timer does still deadlock — that was observed directly, by
patching `poll_io(|| Some(...))` to `poll_io(|| None)` — but **the mechanism is
not known**, and the explanation offered here was a guess stated as fact. The
likeliest remaining candidate is that `need_preempt` never fires, so
`run_task_queues`' inner loop never yields the shard back to the outer loop; that
has not been verified either and should not be repeated as fact until it is.

## The shape of a fix

Not "remove the preempt timer" — that was tried and it hangs. The timer exists so
a task queue cannot monopolise the shard, and the eventfd arming currently
depends on it.

What the counts suggest instead:

1. **Do not re-arm an unchanged timer.** If the deadline has not moved since the
   last iteration, the install and its matching cancellation are both pure waste.
   Five cancellations per round trip is the signature of arming and retiring the
   same thing repeatedly.
2. **Skip it when it cannot matter.** With a single runnable task queue there is
   nothing to preempt in favour of. That is exactly the ping-pong case, and
   exactly where the cost is 53% of the round trip.
3. **Understand the deadlock before removing anything.** Patching the timer out
   hangs the runtime, and the reason is not yet known — see the correction above.
   Item 1 does not remove the timer, only the churn, so it can be attempted
   first; item 2 does remove it in some cases and must wait.

Each needs its own before-and-after measurement, and the ladder in
`probe_read_ladder.rs` is the instrument: it isolates glommio from its own design
so a change can be scored without a runtime rewrite.

## Caveats

**The instrumented run inflates absolute times** by roughly a microsecond of
`Instant::now` calls. Counts are unaffected.

**The ladder's design-cost row is noisy.** It measured −34 ns in one run and
+866 ns in another. The claim that the readiness design is not the problem rests
on it being small and inconsistent in sign, not on the specific figure. The
glommio-over-its-own-design gap was stable at ~2.0–2.5 µs across both.

**Loopback only.** On a real NIC this is a rounding error again, as it is against
an NVMe read. This matters for loopback, IPC-style workloads, and anything else
where the underlying operation is a few microseconds.
