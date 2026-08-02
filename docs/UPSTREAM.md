# Upstreaming: Where, What, and Whether

**Date:** 2026-08-02
**Short answer:** yes, it is worth it — but to **`glommio/glommio`**, not DataDog.

## The landscape changed

**`DataDog/glommio` is abandoned.** Last commit 2025-04-21. Sixteen open pull
requests, the oldest from 2021. Issue
[#707 "Call for glommio maintainers"](https://github.com/DataDog/glommio/issues/707)
is where the community organised itself.

**`glommio/glommio` is the live fork.** A new org, active through June 2026,
already merging the DataDog backlog (musl CI, dependency updates, a stdin fix
from 2022). Glauber Costa, the original author, has said in #707 that he is happy
to move the crates.io name to the fork "as long as there is a clear leader".

**Your `#700` fix is already in it.** Commit `a60c895` in `glommio/glommio` is
authored by this fork's maintainer — PR #703 landed there rather than at DataDog.

So: **do not open PRs against DataDog.** Everything below targets
`glommio/glommio`.

## Divergence

**Merged 2026-08-02.** Their 15 commits are now in; `upstream` points at
`glommio/glommio`. We are ahead only.

```
before the merge:
  ours ahead of community/main:  131 commits
  community/main ahead of ours:   15 commits
```

Their 15 included some that touched the same code we do:

| commit | why it matters to us |
|---|---|
| `4424815` Bump waker refcount from i16 to i32 | **Conflicts with our `Header` layout** |
| `59f5f55` Make the gate closure more robust | executor internals |
| `1981af4` `TcpStream::into_accepted` | new API we do not have |
| musl CI, dependency updates | infrastructure |

### The one real conflict — resolved

They widened `references` to `AtomicI32`. We had put `task_queue_index: u32` into
`Header`'s four bytes of end padding to keep it at 40 bytes. Combined, the header
grew to 48.

**Fixed** by narrowing `executor_id` from `usize` to `u32` (`ac43600`) — it is a
registry index that is only ever compared or looked up, so four billion
executors is ample. `Header` is back to 40 bytes carrying their wider refcount.
Kept as a separate commit from the merge so the type change is reviewable on its
own.

Other resolutions: their `nightly` cfg predicate with our `LOCAL_EX.is_set`
guard and `spawn_internal` rename; their manifest formatting (they use `taplo`,
config now in the tree) with our `io-uring` dependency and allocator
`check-cfg`s; their rewritten CI wholesale, with the branch filter widened to
cover `master`; and the deletion of `tests/linters.rs`, which they replaced with
direct fmt and clippy steps in CI.

## What we have that they do not

Verified against `community/main`, not assumed:

| | what | value | self-contained? |
|---|---|---|---|
| 1 | **`3f4d113`** — no process abort when a task panics in a custom queue | closes DataDog **#689** | yes, small |
| 2 | **`f28a619`** — `executor_id` in the header instead of `Arc<SleepNotifier>` | fixes **#448** at the root; 4,350 → 28 ns spawn at 64 executors | yes, but conflicts as above |
| 3 | **Timing wheel** (`timer/timing_wheel.rs`, `staged_wheel.rs`) | 17.7 → 10.3 ns at 100k timers, **zero unsafe** | large but standalone |
| 4 | **Public `LocalExecutor::spawn`** | closes DataDog **#695** | yes, small, additive |
| 5 | **`8fc2a8e`** — zero-sized schedule closure | −32% task switch | depends on 2 |
| 6 | **`f15c68c`** — cache-domain-aware placement | avoids a +69% cross-L3 penalty | yes; adds a public field |
| 7 | **Miri coverage of the task lifecycle** | first UB checking on `task::raw` | yes |
| 8 | **`32dd7dd`** — 17 broken doctests repaired | hygiene | ours to fix; the break was ours |
| 9 | **`docs/investigations/`** | answers DataDog **#641** ("what's the problem with performance") with measurements | docs only |
| 10 | **iou/uring_sys retirement** | −3,042 lines, −100 `unsafe` | **blocked**, see below |

They still vendor `iou` and `uring_sys`, and still have the old `timer_impl`
without a wheel — so 3 and 10 are not duplicated effort.

## Suggested order

Smallest and most obviously correct first, to establish review trust before
anything large arrives.

1. **`3f4d113`** (#689) and **public `spawn`** (#695). Both small, both close an
   open upstream issue, neither touches the header.
2. **Doctest repairs.** Pure hygiene, uncontroversial.
3. **Timing wheel.** Large but standalone, zero unsafe, and the measurement is
   in `docs/investigations/`.
4. **`f28a619`** with `executor_id` narrowed to `u32` to coexist with their
   `AtomicI32`. Include the #448 analysis.
5. **Cache-domain placement.** Note the new public field on `CpuLocation` is a
   semver consideration for them.
6. **Miri task-lifecycle tests**, plus the `test_executor_id` hook they require.
7. **The investigations**, or a condensed version. `#641` has been open since
   2025 asking exactly what these answer.

**Not yet: the iou retirement.** It depends on `CompletionQueue::head_tail_ptrs`,
which currently lives on a personal fork of `io-uring`. **Submitted upstream as
[tokio-rs/io-uring#404](https://github.com/tokio-rs/io-uring/pull/404)**
(2026-08-02); until it merges and ships, this change cannot be published by
anyone. See [investigations/iou-replacement](investigations/iou-replacement/).

## Honest caveats

**131 commits is not a pull request.** It is roughly seven, and several need
rebasing onto their 15. Expect real conflict work in `executor/mod.rs`,
`task/header.rs` and `task/raw.rs`.

**Some of our commits fix our own breakage** and should not be presented as
upstream value: the doctest repairs undo a rename we made, and the spawn-API
restoration undoes a gate we added. Squash or drop those parts when porting.

**The performance work is measured on one machine.** Every figure in
`docs/investigations/` comes from a single Threadripper. The method transfers;
the numbers may not, and the write-ups say so.

**A maintainer with review capacity is the scarce resource**, not patches. #707
suggests they are short of reviewers rather than short of code. Landing 1 and 2
quickly is worth more than arriving with everything at once.
