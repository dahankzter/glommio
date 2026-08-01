# Investigation: Where the Cycles Actually Go

**Date:** 2026-08-01
**Method:** measure the machine first, then the runtime, then propose
**Status:** three candidates, each with its premise already tested

The task arena was built on an unmeasured premise and cost four months to
unwind ([post-mortem](../task-arena/)). This investigation inverts the order:
every candidate below has had its premise measured *before* being proposed, and
two of them have had their ceiling established by temporarily patching the
runtime and re-benchmarking. Nothing here is a projection.

---

## 0. The machine

All numbers are from the development box. They will differ elsewhere; the
*method* is what transfers.

```
AMD Ryzen Threadripper PRO 9975WX
32 cores / 64 threads, 1 socket, 1 NUMA node
L1d 32x48K   L2 32x1M   L3 4x32M
cache line 64B
```

The critical structural fact: **one NUMA node, one package, but four L3
domains** — cpus 0-7, 8-15, 16-23, 24-31 (plus SMT siblings). Section 4 shows
glommio cannot see this.

## 1. Primitive costs, measured here

`probe_primitives.rs`, 50M iterations each, uncontended and cache-hot — the
favourable case. The real path is never cheaper than this.

| operation | cost |
|---|---:|
| `AtomicI16` fetch_add + fetch_sub (Relaxed) | 8.822 ns |
| `Cell<i16>` inc + dec | 0.221 ns |
| `Arc` clone + drop | 9.029 ns |
| `Rc` clone + drop | 0.440 ns |
| `Weak::upgrade` + drop | 0.467 ns |
| `RefCell` borrow_mut + drop | 0.449 ns |
| `Mutex` lock + unlock (uncontended) | 9.501 ns |
| `AtomicBool` compare_exchange (Relaxed) | 4.460 ns |
| `AtomicI16` load (Relaxed) | 0.220 ns |

One read-modify-write is **~4.4 ns**. A relaxed atomic *load* is 0.22 ns — on
x86 it is a plain `mov`. The cost is the `lock` prefix, not the atomicity.
**Atomic RMW is 40x a `Cell` operation.** That is the whole story of section 3.

Note what is *not* expensive: `RefCell` and `Weak::upgrade`, at under half a
nanosecond each. Remember that for section 6.

## 2. What a task switch costs today

`probe_task_switch.rs`, one shard pinned to cpu 0:

```
single task, yield loop      28.72 ns/switch
two tasks alternating        28.82 ns/switch
```

28.7 ns per wake → schedule → poll cycle. That is the denominator for
everything below.

## 3. How much of it is reference counting: **58%**

`Header::references` is an `AtomicI16`. I instrumented
`increment_references` / `decrement_references` with a thread-local counter and
ran a spawn-plus-yield workload:

```
spawn+await, no yield :  1.00 RMW/task  (0.00 schedule calls)
spawn+await, 1 yield  : 11.00 RMW/task  (2.00 schedule calls)
spawn+await, 3 yields : 19.00 RMW/task  (4.00 schedule calls)
```

**Four atomic RMWs per wake cycle**, exactly. At 4.4 ns each that is ~17.6 ns
of a 28.7 ns task switch.

Rather than trust that multiplication, I patched the runtime and measured the
real ceiling (both patches reverted; the tree is clean):

| task switch | atomic RMW (today) | non-atomic (probe) |
|---|---:|---:|
| **schedule guard on (today)** | **28.72 ns** | 14.10 ns |
| **schedule guard off** | 19.45 ns | 11.97 ns |

Reference counting is **16.75 ns of a 28.72 ns task switch — 58%**. Glommio is
a thread-per-core runtime in which the overwhelming majority of these
increments and decrements are performed by the thread that owns the task, on a
line no other core is touching. It pays the `lock` prefix anyway.

### 3a. Two of those four atomics are an accident

Look at `RawTask::schedule`:

```rust
let guard = if mem::size_of::<S>() > 0 {
    Some(Waker::from_raw(Self::clone_waker(ptr)))   // atomic RMW
} else {
    None
};
(*raw.schedule)(task);
drop(guard);                                        // atomic RMW
```

`S` is the schedule closure, which lives *inside* the task allocation. If the
task is freed during the call, the closure's captured state dies underneath
it — hence the guard. But the guard is only needed because the closure is
non-zero-sized, and it is non-zero-sized for exactly one reason:

```rust
let tq = Rc::downgrade(&tq);
let schedule = move |runnable: Runnable| {
    let tq = tq.upgrade();                          // <- 8 captured bytes
    ...
};
```

It captures a `Weak<RefCell<TaskQueue>>`. My counter confirms the guard is
taken on **100% of schedule calls** (2 of 2, 4 of 4). Eight bytes of capture
cost two atomic RMWs on every single task switch.

---

## Candidate 1 — make the schedule closure zero-sized

**Measured win: 28.72 → 19.45 ns/switch, −32%.**
**Risk: low. Safe code. Directly precedented.**

Store the task-queue index in `Header` and have a ZST closure resolve it from
the thread-local executor, exactly as `f28a619` did for `executor_id`. That
commit removed a process-wide `RwLock` from the spawn path by replacing an
`Arc<SleepNotifier>` in the header with a `usize` id; this is the same move one
level down.

The pleasing detail — `Header` is 40 bytes with **exactly 4 bytes of end
padding**:

```
field .awaiter          16 bytes
field .executor_id       8 bytes
field .vtable            8 bytes
field .references        2 bytes
field .state             1 byte
field .latency_matters   1 byte
end padding              4 bytes   <- a u32 queue index fits here for free
```

A `u32` task-queue index lands in padding that is already being paid for. The
header stays 40 bytes, the task allocation is unchanged, and the cache
behaviour is identical.

**Cost of being wrong:** the `Weak` also handles a real case — a task whose
task queue has been destroyed. The index form must reproduce that (absent
index → drop the runnable), and `upgrade()` returning `None` is the current
signal. This needs care, not cleverness.

**Validate before building:** already done — that is the 19.45 ns row above,
produced by forcing the guard condition to `false`.

---

## Candidate 2 — stop paying for atomics on thread-local wakers

**Measured win: 28.72 → 14.10 ns/switch, −51%. With candidate 1: 11.97 ns, −58%.**
**Risk: high. This is the hard one, and it is the real prize.**

The refcount is atomic because Rust's `Waker` is
`unsafe impl Send + Sync` *unconditionally* (`core/src/task/wake.rs`). Any
third-party future may clone a waker and send it to another thread, so the
runtime cannot assume single-threaded access. Seastar avoids this by being a
closed world; Rust's ecosystem contract does not permit that.

Two routes, and they are not equally available:

**(a) Biased reference counting — works on stable today.** The owning thread
uses a non-atomic counter; any other thread uses a separate atomic one; the
task is freed when both reach zero. The owner's path — which is the 4 RMWs per
switch measured above — becomes plain loads and stores. This is a known
technique (Choi et al., "Biased Reference Counting", PACT'18) and needs no
ecosystem change. It costs one branch on the owner-thread check, which
predicts perfectly.

**(b) `LocalWaker`** — `!Send + !Sync`, `#[repr(transparent)]`, exists in core,
still unstable (tracking issue #118959). Cleaner, but it does not help until
the futures ecosystem accepts it, and it cannot be relied on for a library that
must work with arbitrary third-party futures.

Recommendation: (a). Glommio owns its own timers, io_uring sources and
channels, so a large fraction of wakers provably never leave their shard.

**Relationship to candidate 1:** they overlap — candidate 1 removes two of the
four atomics, so doing candidate 2 first would leave candidate 1 worth only
~2.1 ns rather than 9.3 ns. Do candidate 1 first anyway: it is small, safe,
and ships the larger part of its value immediately.

**Validate before building:** already done — the non-atomic column above is a
literal measurement of the ceiling, produced by replacing `fetch_add`/
`fetch_sub` with `load`+`store`. That patch is unsound as a shipping change;
it is sound as a measurement of what is available.

---

## Candidate 3 — glommio cannot see this machine's topology

**Measured: cross-L3 shard placement costs +69% on a shard-to-shard round trip.**
**Risk: low. Safe code. Pure gain on multi-CCX parts.**

`probe_topology.rs`, cache-line ping-pong between pinned threads:

| | round trip |
|---|---:|
| same core, SMT sibling (0↔1) | 58.5 ns |
| same L3 domain (0↔4) | 42.5 ns |
| **cross L3 domain (0↔16)** | **486.6 ns** |
| far cross domain (0↔24) | 516.9 ns |

**11.4x.** Crossing an L3 domain on this part costs an order of magnitude more
than staying inside one. And SMT siblings are *worse* than separate cores in
the same domain — co-locating two shards on one physical core is a pessimisation.

`probe_shard_ping.rs` measures whether that reaches the runtime. Two shards
ping-pong a `u64` over paired `shared_channel`s; only the CPU assignment changes:

| | default (park) | `spin_before_park(10us)` |
|---|---:|---:|
| same L3 (0↔4) | 8002 ns | 4428 ns |
| cross L3 (0↔16) | 13509 ns | 5276 ns |
| **penalty** | **+69%** | +19% |

Now the structural problem. `CpuLocation` is:

```rust
pub struct CpuLocation {
    pub cpu: usize,
    pub core: usize,
    pub package: usize,
    pub numa_node: usize,
}
```

and the placement tree is `SystemRoot → NumaNode → Package → Core → Cpu`
(`placement/pq_tree.rs`, `enum Level`). **There is no cache-domain level.** This
machine reports one NUMA node and one package, so `MaxSpread` and `MaxPack` see
32 undifferentiated cores and cannot distinguish an arrangement that is 69%
slower. The kernel has been telling us all along:

```
$ cat /sys/devices/system/cpu/cpu0/cache/index3/shared_cpu_list
0-7,32-39
```

**The work:** parse `cache/index3/shared_cpu_list` alongside the existing
`physical_package_id` / `core_id` reads, add an `l3_domain` field to
`CpuLocation`, insert a `Level::CacheDomain` between `Package` and `Core`, and
extend the sort key in `hardware_topology.rs` (currently
`(numa_node, package, core, cpu)`). `MaxPack` then packs within an L3 domain
and `MaxSpread` spreads across domains deliberately rather than by accident.
This is a new public field on a public struct, so it is a semver consideration.

**The second half of this candidate is bigger than the first.** Note the
absolute numbers: 8 µs for a round trip whose data movement costs 42 ns. Park
and unpark are ~3.6 µs of it same-domain and ~8.2 µs cross-domain — spinning
removes most of the penalty *and* most of the topology sensitivity. That points
at the wake mechanism itself: glommio writes an eventfd (a syscall) and the
peer reads it (another syscall). Since kernel 5.18, `IORING_OP_MSG_RING` posts
a CQE directly into a peer's ring, which is precisely this operation without
the eventfd round trip. Glommio already requires 5.8+. **Measure before
building:** compare `IORING_OP_MSG_RING` against the eventfd path in isolation
first — this candidate has a premise too, and it is exactly the kind that felt
obvious to the arena's author.

---

## 6. What not to do next

`OPTIMIZATION_PLAN.md` has **P2b: RefCell → UnsafeCell**, claiming "10-15% task
switch", rated High risk, with "RefCell → UnsafeCell unsoundness" listed as a
**Critical** consequence in its own risk table.

A `RefCell` borrow_mut + drop costs **0.449 ns** (section 1). The task switch is
28.72 ns. Removing *every* borrow on the path — there are not more than a
handful — cannot reach 10%, and the same measurement shows 58% of that switch
sitting in atomics that can be removed **without any unsafe at all**.

This is the arena's failure mode exactly: a plausible mechanism, a confident
percentage, no measurement, and a proposal to trade safety for it. It should be
struck from the plan or re-scoped to the safe caching half (P2a), and the
effort moved to candidates 1 and 2.

A second negative result, from `probe_topology.rs`:

```
contended fetch_add on one line:
  1 thread                         4.2 ns/op
  8 threads, same L3               5.9 ns/op
  8 threads, spread across L3      7.0 ns/op
  32 threads, all domains          7.3 ns/op
```

Contention on a single hot line degrades by 1.7x across 32 threads and four
cache domains. This is worth keeping in view whenever someone proposes removing
"lock contention" — it independently corroborates the arena post-mortem's
finding, on the same hardware, by a different route.

---

## 7. Order of work

| | candidate | measured win | risk | why this order |
|---|---|---:|---|---|
| 1 | ZST schedule closure | −32% task switch | low | small, safe, precedented by `f28a619` |
| 2 | cache-domain placement | −41% cross-shard round trip | low | independent of 1; safe code; helps every multi-CCX deployment |
| 3 | biased reference counting | −51% task switch | high | the real prize; do it once 1 has proven the header-index pattern |

Candidate 3's ceiling is already known (14.10 ns), which is the difference
between this list and the roadmap that produced the arena.

## 8. Reproducing

The probes are in this directory. `probe_primitives.rs` and
`probe_topology.rs` are standalone (the latter needs `libc`);
`probe_shard_ping.rs` and `probe_task_switch.rs` depend on `glommio` by path.

```bash
cargo run --release --bin probe      # primitive costs
cargo run --release --bin topo       # cache-domain ping-pong
cargo run --release                  # shard round trip / task switch
```

The two ceiling measurements in section 3 require temporarily patching
`glommio/src/task/raw.rs`:

1. **Candidate 1 ceiling:** change `if mem::size_of::<S>() > 0` to `if false`
   in `RawTask::schedule`.
2. **Candidate 2 ceiling:** replace `fetch_add`/`fetch_sub` in
   `increment_references`/`decrement_references` with `load` + `store`.

Both are unsound as shipping changes and sound as measurements. Revert them.
