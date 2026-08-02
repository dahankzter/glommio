# What Is Left, and What It Is Worth

**Date:** 2026-08-02
**Status:** current, replaces the perf sections of `PERFORMANCE_ROADMAP.md`

The easy wins are gone. Four paths were measured on 2026-08-01/02 and three came
back thin. Task switch is down 43% (28.72 → ~16.5 ns) from two landed changes,
and everything else is either device-dominated or already at the floor.

What remains is concentrated in one place, and it only exists in one regime.

## The one number

**glommio costs ~2 µs per I/O that actually blocks.** Not per byte, not per
message. See [io-path/synthesis.md](investigations/io-path/synthesis.md).

It disappears three ways:

- **Queue depth.** Per blocking operation, so N in flight amortises it N ways.
- **Busy shards never park.** `notify` skips the eventfd when the peer is awake;
  `yolo_recv` returns buffered data with one syscall and no io_uring.
- **Denominator.** 6% of a 35 µs NVMe read, 53% of a 3.8 µs loopback round trip,
  a rounding error against a real NIC.

**Before spending anything on items 1 or 2 below, measure how often shards
actually park in a real workload.** That number decides whether either is worth
doing, and it cannot be obtained from a microbenchmark. `spin_before_park` is a
builder knob today: if setting it changes throughput or p99 materially, the park
cost is being paid; if not, it is not.

## Ranked

### 1. Preempt-timer churn — the only large target left

A TCP echo round trip processes **nine io_uring completions to perform one
read**: one real poll, three preempt timers, five cancellations
([per-io-cost.md](investigations/io-path/per-io-cost.md)). That is the 2 µs.

**Cost to fix:** higher than it looks. The cancellation is entangled with the
sleep decision — cancelling changes `can_sleep()`, which feeds `should_sleep`,
which is computed after the cancel. So it is a restructure of the park decision,
not a guard clause. And removing the timer naively **hangs the runtime**; that
was observed directly and **the mechanism is still unknown**. Diagnosing the hang
gates everything else here.

**Payoff:** latency and low-queue-depth workloads only. No throughput win.

### 2. `MSG_RING` + `SINGLE_ISSUER|DEFER_TASKRUN`

Targets the *wake* half of the same 2 µs. Measured at ~1,480 ns against
~2,100–2,400 ns for the current eventfd path, stable to 1% across runs — roughly
−30% of the wake primitive. Unblocked now that the vendored `iou` is gone.

All-or-nothing: the flags alone measured as noise (±14%), so there is no cheap
intermediate step. Same regime as item 1, and the same parking caveat applies.

### 3. Many-shard and many-connection scaling — untested

Everything measured so far used one or two shards and one connection. This is
where a thread-per-core runtime either shines or falls over, and it is the
closest thing to a production shape that has not been looked at: `accept`
throughput, per-shard scaling curves, cross-shard fan-out.

**Cheap, and it might surface something new rather than shave something known.**
Recommended before item 1, because item 1 is expensive work whose payoff depends
on a parking rate nobody has measured. If item 3 also comes back thin, that is
strong evidence the remaining cost really is all in the park path, and item 1
gains a known denominator.

### 4. Breadth on the I/O path

Writes, buffered (non-DMA) I/O, `read_many`, registered buffers, UDP. Expected
value now lower: three siblings came back thin.

### 5. Upstream the io-uring accessor

Not performance. `glommio/Cargo.toml` points at `dahankzter/io-uring` branch
`feat/cq-head-tail-ptrs` for `CompletionQueue::head_tail_ptrs`, which
`need_preempt` requires. **Blocks `cargo publish`** until upstreamed or inlined.
Becomes urgent the moment a consumer wants a released version.

## Method, earned the hard way

Five proposed rewrites were killed by measurement on 2026-08-01/02 before anyone
wrote them, and three published claims were retracted the same day. The traps,
all of which cost time:

- **Never compare runs taken minutes apart.** The baseline for unchanged code
  drifted 28.72 → 19.46 → 16.50 ns across sessions on this box. Use
  `investigations/mechanical-sympathy/probe_ab_refcount.sh`, which alternates
  builds back to back in one session.
- **Best-of-three with no variance is not a measurement.**
- **Primitive cost badly overpredicts path cost.** An atomic RMW is 4.4 ns as an
  isolated dependent chain and roughly a quarter of that on a real path.
- **One-factor-at-a-time hides combinations.** `MSG_RING` alone is −3%; with the
  taskrun flags the same change is −25%.
- **Build a ladder.** Isolating a runtime from its own design — implementing the
  design by hand, without the runtime — settled the network question in twenty
  minutes after two wrong conclusions.
- **Check for a benchmark bug before believing a 10x.** One was Nagle left on for
  the control and off for the subject.
