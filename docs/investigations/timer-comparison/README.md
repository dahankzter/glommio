# Which timer structure should glommio use?

**Date:** 2026-09-07
**Design:** [2026-09-04-timer-wheel-redesign-design.md](../../superpowers/specs/2026-09-04-timer-wheel-redesign-design.md)
**Prompted by:** review of [#33](https://github.com/glommio/glommio/pull/33)

Three implementations built and measured against each other, after review of
#33 found the hierarchical wheel it proposed had four defects and that the
benchmark arguing for it was taken with one of them in the loop.

**Outcome: the slab wheel wins, narrowly, and not for the reason the original
PR gave.**

## Reproducing it

```bash
make timer-arms                       # every arm
make timer-arms ARMS="a-slab-wheel"   # one of them
```

Needs a clean tree. The script checks out each arm branch in turn, builds it,
runs the benchmark and puts your branch back; results land in
`target/timer-arms/`. An arm that does not contain the benchmark's own commit
is refused rather than measured.

The two programs behind it, runnable on any branch:

```bash
cargo run --release --features debugging --example timer_ladder  # populations
cargo run --release --features debugging --example timer_bench   # costs
```

## What the workload actually is

Before comparing structures, count what they hold. Only two sites register a
timer with the reactor, `net/stream.rs` for socket timeouts and
`timer_impl.rs` for `Timer`/`sleep`, glommio registers none of its own, and
socket timeouts are `None` unless an application asks for them.

`timer_ladder`, connections with a read timeout doing request/response cycles
that complete well inside it:

```
  conns    inserted       fired   cancelled   highwater   cancel%
     64        1024           0        1024          64     100.0
    256        4096           0        4096         256     100.0
   1024       16384           0       16384        1024     100.0
   4096       65536           0       65536        4096     100.0
```

**Nothing fires.** Cancellation is the whole hot path, and the live count is
exactly the connection count, one pending read per connection, not two. The
100k figure #33 was argued from implies a 50,000-connection proxy.

Scheduled work (`sleep`, `Interval`) is the mirror image, 100% fired, and is
the secondary case.

## The arms

All three branch from the same commit and carry the same benchmark, the same
counters and the same reactor path. They differ only in how timers are stored,
so a difference in the numbers is a difference in the structure.

| Branch | Structure |
|---|---|
| `arm/control-btreemap` | `BTreeMap<(Instant, u64), Waker>`, the incumbent |
| `arm/a-slab-wheel` | Slab handles over a cascading wheel with occupancy bitmaps |
| `arm/b-bitwheel` | The [`bitwheel`](https://crates.io/crates/bitwheel) crate |

The control differs from upstream in one respect, in its own favour: upstream's
handle is a bare `u64` and a side `AHashMap<u64, Instant>` recovers the deadline
before a cancellation can remove anything. The handle carries its own key here,
so neither is needed. Measuring the other version would charge the incumbent for
a hash lookup the candidates never pay.

## Results

```
                       arm ns/op   cancel ns/op   100µs sleep   idle overshoot
control (BTreeMap)        69–74        51–55         ~105 µs        0.1–0.6 ms
arm A (slab wheel)        55–75        34–40         ~105 µs        0.1–0.5 ms
arm B (bitwheel+3)        45–80        43–57       ~1,009 µs        0.1–0.4 ms
```

**The asymptotic argument did not survive.** Every arm is flat from 64 to 4,096
timers. `log₂(4096)` is twice `log₂(64)` and the difference does not appear,
the tree operations are swamped by allocation, waker clone and poll machinery.
The wheel wins on constants, roughly 18ns per cancellation, not on complexity.

**Cancellation is where it shows**, and the ladder says cancellation is 100% of
the workload: 34ns against the control's 52.

**Precision had to be fixed before the wheel was usable at all.** A wheel
rounds deadlines to whole ticks, so a 100µs sleep took ~1ms, a floor under
every short sleep, which is exactly the case a low-latency caller reaches for.
Arm A now reports the real deadline from the earliest occupied slot rather than
the tick boundary it was rounded up to, and expires by deadline rather than by
tick, so it matches the ordered map at 105µs. **This is still unfixed on fork
`master`.**

## What the defects were

Found by review of #33 and by measurement, all confirmed in the code:

| Defect | Evidence |
|---|---|
| Handle truncates a monotonic counter | `reactor_adapter.rs:62` casts a `u64` to `u32`. At 100k timers/sec, cancellation targets the wrong timer after ~12 hours |
| `generation` never set | Hardcoded `0` and `#[allow(dead_code)]`, while its documentation describes a slot-reuse guard |
| Next expiry is O(n) | A scan of every live timer, on every poll, where the `BTreeMap` answers from its first key in O(1) |
| `advance_to` is O(elapsed ms) | A tick per millisecond regardless of work; ten idle minutes cost 600,000 iterations on the next poll |
| Timers fire early | Deadlines truncated to whole ticks, so a timer due at 1.7ms fired at 1.0ms and the future re-armed. 4,096 sleeps cost 10,749 registrations |
| Sub-tick sleeps rounded up | A 100µs sleep took ~1ms |

The first four are what arm A replaces. The last two are fixed on `master`
already, except the sixth.

## bitwheel

Three defects had to be patched before it could be measured at all, all
reported upstream:

- [#18](https://github.com/Abso1ut3Zer0/bitwheel/issues/18), `cancel`
  performs an unchecked removal justified by "a timer whose deadline is still
  in the future cannot have fired". `poll_tick` fires whole gear slots early,
  so that is false, and cancelling such a timer reaches
  `hint::unreachable_unchecked`, undefined behaviour in release. glommio's
  suite hits it, because cancelling before the deadline is our common case.
- [#19](https://github.com/Abso1ut3Zer0/bitwheel/issues/19), `insert` clamps
  the delay to at least one tick but computes the slot from the unclamped
  deadline, so a timer due inside the current tick lands in the slot the clamp
  was avoiding. `poll` fires it; `duration_until_next` cannot see it for a
  whole gear revolution. A 100µs sleep took 64ms.
- `len` was decremented on cancellation but not on firing.

Two further properties are design rather than defect, and both are wrong for
this workload. Slots have a fixed compile-time capacity, and identical timeouts
give identical deadlines, so a deadline-churn population above `SLOT_CAP` spills
into the `BTreeMap` failover the wheel was meant to replace. And because it
fires slots early by design, a timer can be woken before its deadline and be
gone from the wheel, with no way to ask whether a registration still exists,
so every pending poll must re-arm unconditionally.

Three `TimerActionRepeat` tests also fail on cadence: an action expected to run
eleven times in a window runs five.

## Verdict

Arm A. It matches the ordered map on precision, beats it by about a third on
the one operation the workload actually performs, and removes four defects that
are live on `master` today.

The honest caveat: the win is a constant, not an asymptote, and roughly 18ns per
cancellation against ~800 lines of wheel is a judgement call rather than a
conclusion the measurement makes for you. If that trade is not wanted, the
control is a complete implementation that also removes all four defects, by
deleting the wheel.

bitwheel is not viable here regardless of its speed: three patches to a crate
last published 2025-12-18, a capacity model that degrades into a `BTreeMap` on
precisely this workload, and unsafe code we cannot Miri.
