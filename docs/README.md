# Glommio Fork Documentation

This fork contains fixes and investigations for critical Glommio issues while the upstream repository awaits maintainer response.

## Fixes Implemented

### ✅ [Issue #700](https://github.com/DataDog/glommio/issues/700) - Memory Corruption in spsc_queue
**Status:** Fixed in PR #703
**Severity:** Critical (heap corruption in safe code)

Removed public `Clone` trait from `Producer` and `Consumer` in SPSC queue to prevent memory corruption when multiple producers/consumers are created.

**Branch:** `fix/issue-700-remove-spsc-clone`

## Investigations

### [Issue #448 - Eventfd Leak on Executor Drop](./investigations/issue_448/)
**Status:** Documented, workarounds available
**Severity:** High (resource exhaustion in long-running apps)

Comprehensive investigation of eventfd file descriptor leak when executors are repeatedly created and destroyed. Includes root cause analysis, potential fix approaches, and practical workarounds.

**Key Finding:** This is an architectural issue that the original maintainer attempted to fix but found "really hard" due to task lifecycle complexity.

**Workarounds:**
- Use long-lived executors (recommended)
- Thread-local executor pattern for tests

### [Issue #695 - Non-Panicking spawn_local()](./investigations/issue_695/)
**Status:** ✅ Implemented
**Severity:** Medium (API design issue)

Investigation of confusing `spawn_local()` API that panics even when called on a `LocalExecutor` instance. The current design ignores `self` and uses thread-local storage instead.

**Key Finding:** The private `spawn()` method actually uses `self` but is not public. Making it public solves the issue without breaking changes.

**Fix:** `LocalExecutor::spawn()` is now public. Additive — thread safety still
comes from `!Send`, and no existing API changed.

### [Task Arena Allocator - Post-Mortem](./investigations/task-arena/)
**Status:** Tried, measured, reverted
**Outcome:** No arena — recommend mimalloc instead

A slot allocator for task blocks was built and benchmarked, then removed. Its
premise (global allocator lock contention on the spawn path) did not survive
measurement: allocation cost is flat from 1 to 8 concurrent executors. The
arena also cost ~98 MB resident per executor, panicked on task closures over
1 KB, and segfaulted on detached tasks.

**Key Finding:** The contention was in glommio's own sleep-notifier registry,
not in malloc — a process-wide `RwLock` taken on every spawn. Fixing that took
spawn at 64 executors from 4350.9 ns to ~28-37 ns, in safe code, and fixed
#448 at the root as a side effect.

### [Where the Cycles Actually Go](./investigations/mechanical-sympathy/)
**Status:** Three candidates, each premise measured before proposal
**Approach:** measure the machine, then the runtime, then propose

Measures what a glommio task switch actually spends its 28.7 ns on, and what
this hardware charges for the primitives involved. Reference counting is 58% of
a task switch, and two of the four atomics per switch exist only because the
schedule closure captures 8 bytes.

**Candidates, with ceilings established by patching and re-benchmarking:**
- Zero-sized schedule closure — **-32%** task switch, low risk, safe code
- Cache-domain-aware placement — **-41%** cross-shard round trip; `CpuLocation`
  has no L3 level, and cross-L3 line transfer costs 11.4x here
- Biased reference counting — **-51%** task switch, high risk, the real prize

**Also records what not to do:** the planned `RefCell` → `UnsafeCell` work
(P2b, "10-15%", Critical unsoundness risk) has an unmeasured premise — a
`RefCell` borrow costs 0.449 ns.

### [The DMA Read Path](./investigations/io-path/)
**Status:** Measured — no fat found
**Result:** glommio adds ~2.2 µs at queue depth 1, ~0.4 µs at 16, nothing at 64

4 KiB `O_DIRECT` random reads against a raw io_uring floor. The device dominates
at every depth, and glommio's absolute overhead *falls* as concurrency rises —
the shape of one reactor-loop iteration, which overlaps with device time once
several reads are in flight.

**Nothing here is worth optimising**, and it relocates the roadmap's monoio
question: per-operation path cost cannot explain a large gap, because there is
almost none to give back.

### [Replacing the Vendored `iou` / `uring_sys`](./investigations/iou-replacement/)
**Status:** Surveyed and scoped, not attempted
**Prize:** 35% of the fork's unsafe surface, and an unfreezing of io_uring

`glommio/src/iou` and `glommio/src/uring_sys` are 3,042 hand-maintained lines
holding **108 of glommio's 309 `unsafe` occurrences**. They are a copy of two
abandoned crates — there is nothing to upgrade to. The vendored `liburing`
submodule is current, which is why the C header knows `IORING_SETUP_*` flags to
bit 18 while the Rust wrapper stops at bit 5.

Replacing them with the maintained `io-uring` crate would delete more unsafe
than everything the centralization analysis proposes. Contains the full API
mapping, the structural reasons this is not a dependency swap, where the risk
concentrates, and a step-by-step sequencing that keeps the suite green.

### [Unsafe Code Centralization Analysis](./investigations/unsafe-centralization/)
**Status:** Analysis complete
**Complexity:** High (7-12 weeks refactoring)

Comprehensive analysis of eliminating or centralizing unsafe code in glommio without performance degradation. Identifies 320 unsafe blocks scattered across 43+ files and proposes centralization into 4 core modules.

**⚠️ Partly superseded:** the analysis counts the task arena's 19 unsafe blocks
in its baseline and plans to relocate them. The arena has since been deleted.
See the note at the top of that document.

**Key Findings:**
- Unsafe code cannot be eliminated without 10-100x performance loss
- Can be centralized from 43+ files to 4 core modules (~1000 lines)
- Current scattering makes auditing and maintenance difficult

**Recommended Approach:**
1. Short-term: Document all unsafe with safety comments
2. Medium-term: Add Miri CI for continuous validation
3. Long-term: Incrementally refactor into `core/` modules

## Repository Structure

```
docs/
├── README.md (this file)
└── investigations/
    ├── issue_448/
    │   ├── README.md         # Eventfd leak analysis
    │   └── reproduce.rs      # Test demonstrating the leak
    ├── issue_695/
    │   └── README.md         # API design investigation
    ├── io-path/
    │   ├── README.md         # DMA read path vs the raw io_uring floor
    │   └── probe_dma_read.rs
    ├── iou-replacement/
    │   └── README.md         # Retiring the vendored io_uring wrappers
    ├── mechanical-sympathy/
    │   ├── README.md         # Where the cycles go + three measured candidates
    │   └── probe_*.rs        # Reproducible probes (primitives, topology, shard, switch)
    ├── task-arena/
    │   └── README.md         # Arena allocator post-mortem (built, measured, reverted)
    └── unsafe-centralization/
        └── README.md         # Unsafe code analysis & centralization strategy
```

Top-level documents:

| | |
|---|---|
| `PERFORMANCE_ROADMAP.md` | high-level phases and prioritization |
| `OPTIMIZATION_PLAN.md` | detailed implementation plans |
| `TASK_ALLOCATION_AUDIT.md` | allocation lifecycle — arena conclusions retracted inline |
| `BENCHMARKING.md`, `COVERAGE.md`, `LIMA_TESTING.md` | tooling |
| `CACHE_OPTIMIZATION_ANALYSIS.md` | cache-line layout analysis |

## Contributing

This fork is maintained by [@dahankzter](https://github.com/dahankzter) while awaiting upstream response. If you encounter issues or have fixes, please open an issue or PR.

## Upstream Status

- **Original Repository:** [DataDog/glommio](https://github.com/DataDog/glommio)
- **Original Maintainer:** Glauber Costa (no longer at DataDog)
- **Current Status:** Awaiting new maintainer assignment

If DataDog resumes active maintenance, improvements from this fork can be contributed upstream.
