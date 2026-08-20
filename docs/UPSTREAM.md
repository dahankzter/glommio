# Upstreaming: Where, What, and Whether

**Date:** 2026-08-02, status updated 2026-08-10
**Short answer:** yes, it is worth it — but to **`glommio/glommio`**, not DataDog.
**Caveat added 2026-08-10:** that repository has gone quiet too. See
[Upstream activity](#upstream-activity) before planning around it.

## The landscape changed

**`DataDog/glommio` is abandoned.** Last commit 2025-04-21. Sixteen open pull
requests, the oldest from 2021. Issue
[#707 "Call for glommio maintainers"](https://github.com/DataDog/glommio/issues/707)
is where the community organised itself.

**`glommio/glommio` is the live fork.** A new org, active through June 2026,
already merging the DataDog backlog (musl CI, dependency updates, a stdin fix
from 2022). Glauber Costa, the original author, has said in #707 that he is happy
to move the crates.io name to the fork "as long as there is a clear leader".

## Upstream activity

**Checked 2026-08-10, and it is quieter than the paragraph above suggests.**

| | |
|---|---|
| last commit to `main` | 2026-06-13 |
| last merged PR | 2026-06-22 (#25) |
| open PRs | 9, of which **6 are ours** |
| reviews on our 6 | none; no comments, and CI has not run on them |

Our PRs went up 2026-08-02 and have sat untouched since. Two months without a
merge is not abandonment — it is one or two people with other jobs, which is
what a community fork usually is — but it does mean **nothing here should be
planned around an upstream merge landing on a schedule**.

Practical consequences:

- **This fork is the artifact.** Consumers should depend on
  `dahankzter/glommio` directly. That is already true of Slipstream.
- **Keep the PRs open and keep them rebasing cleanly.** They cost nothing while
  they wait and they are the whole point of having written them separately.
- **Do not pile on more.** Six unreviewed PRs is already more than most
  maintainers want to receive at once. Adding a seventh makes the pile likelier
  to be ignored, not likelier to be read.
- **Do not chase.** One factual comment when a real blocker clears is
  reasonable — that is what the io-uring merge earned on #34. Repeated nudging
  on a volunteer project is not.

Re-check before spending effort on upstream sequencing: if `main` moves again,
the calculus changes.

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

## What has been submitted

Opened 2026-08-02, smallest and most obviously correct first, so review trust
could be established before anything large arrived. All still open and
unreviewed — see [Upstream activity](#upstream-activity).

| PR | What | Notes |
|---|---|---|
| [#29](https://github.com/glommio/glommio/pull/29) | don't abort the process when a task panics in a custom queue | closes their #689 |
| [#30](https://github.com/glommio/glommio/pull/30) | non-panicking `LocalExecutor::spawn` | closes their #695 |
| [#31](https://github.com/glommio/glommio/pull/31) | cache-domain-aware placement | new public field on `CpuLocation` is a semver consideration for them |
| [#32](https://github.com/glommio/glommio/pull/32) | indices in the task header, not owned references | the #448 fix; `executor_id` narrowed to `u32` to coexist with their `AtomicI32` |
| [#33](https://github.com/glommio/glommio/pull/33) | timing wheel replacing the timer `BTreeMap` | large but standalone, zero unsafe, measurement in `docs/investigations/` |
| [#35](https://github.com/glommio/glommio/pull/35) | retire the vendored iou / uring_sys | −3,042 lines, −105 `unsafe`; discussion in their [#34](https://github.com/glommio/glommio/issues/34) |

**#35 was blocked and no longer is.** It depended on an `io-uring` accessor that
lived on a personal fork; that **merged upstream as
[tokio-rs/io-uring#404](https://github.com/tokio-rs/io-uring/pull/404) on
2026-08-09** as `CompletionQueue::status()`. Taken out of draft 2026-08-10; the
dependency points at `tokio-rs/io-uring` directly.

**Fully unblocked as of 2026-08-11**, when `io-uring` released 0.7.14 carrying
the accessor. The dependency is now a plain `io-uring = "0.7.14"` — no git
dependency, nothing preventing publication. See
[investigations/iou-replacement](investigations/iou-replacement/).

### Not yet submitted

- **Miri task-lifecycle tests**, plus the `test_executor_id` hook they require.
  Genuinely depends on #32 landing first — it uses the new `spawn_local`
  signature.
- **The investigations**, or a condensed version. DataDog **#641** has been open
  since 2025 asking exactly what these answer. Deliberately held: see the note
  about not piling on above, and
  [investigations/io-path/monoio-gap.md](investigations/io-path/monoio-gap.md)
  for why that one needs framing as measurement rather than correction.

## Plan for the 2026-08-19/20 additions

Everything below was written after the six PRs above were opened, and none of
it has been offered upstream. The constraint has not changed: **their scarce
resource is review capacity, not code.** Six of ours have sat unreviewed since
2026-08-02. Arriving with nine more would make that worse, not better.

So this is staged, and each stage has a gate that must open before the next
begins.

### Stage 0 — three bug reports, no patches (do this first, unconditionally)

Issues cost a maintainer minutes to read and nothing to review. All three are
defects in **their** code, found while working here, and two of them block
things they want.

| Report | Why it matters to them |
|---|---|
| A panicking `spawn_blocking` closure hangs its caller forever and permanently costs the pool a worker | Live in canonical 0.9.0 and their `main`. Four panics exhaust a four-thread pool. The obvious fix is worse than the bug: the awaiting side calls `assume_init` on memory a panicked closure never wrote |
| `build.rs` runs `./configure` inside the vendored `liburing`, mutating the crate's own source directory | `cargo package` rejects the tarball outright, so **they cannot publish 0.10 under any name.** DataDog's `configure()` runs it in `OUT_DIR`; the regression is theirs alone |
| `LocalExecutorBuilder::name` is silently ignored by `make()` | Only `spawn()` reads it. Every caller building with `make()` sets a name that goes nowhere |

Report the `configure` one as a bug, **not** as our patch: ours copies the whole
tree because the liburing revision we vendored needs `Makefile.common`, and
theirs may not. Tell them what breaks and let them pick the fix.

**Gate to Stage 1: any human reply on any of the three.** That is the signal
that review capacity exists. Without it, nothing else is worth queueing.

### Stage 1 — two small bug fixes

| PR | Size | Notes |
|---|---|---|
| `spawn_blocking` panic fix (`dcab422`) | ~40 lines | Deletes an `unsafe` block, an `assume_init` and an `Arc::try_unwrap(..).expect("leak")`. Best review-to-value ratio we have. **Code applies cleanly to their `main`**; only its tests need re-homing, since ours live in a module a later commit introduced |
| Kernel probe returns an error instead of calling `exit(1)` (`cd7c9d3`) | ~150 lines, **needs a rewrite** | See below — it does not apply to their tree |

**The kernel probe fix is not independent of #35, contrary to what an earlier
draft of this plan said.** Their `main` still contains `glommio/src/iou` and
`glommio/src/uring_sys`; ours is written against the `io-uring` crate's
`Probe`, which only exists after that PR. Two ways forward, and the second is
better: gate it behind #35, or **rewrite it against their liburing-based
probe**. The valuable half — a library not calling `exit(1)` on its caller's
process — is independent of which probe API reports the opcodes. Do not let
the best bug fix we have wait behind a stalled PR for an incidental reason.

**Gate to Stage 2: one of these merged.** A merge proves the pipeline works
end to end. Anything larger before that is speculative.

### Stage 2 — API additions, smallest commitment first

Each is additive, each stands alone, and each is sized to be reviewable in one
sitting. Ordered by how much a maintainer is being asked to take on, not by how
much we want it.

1. **`spawn_blocking_send`** (~150 lines). One method. No new module.
2. **`future::timeout`** (~120 lines). Carries a question they must answer, and
   the PR should ask it plainly: by the `try_` convention the general form
   should own the name `timeout` and the `Result`-aware one should be
   `try_timeout`. We split by module to avoid breaking their callers. Their
   call whether to take the rename at their next breaking release.
3. **`Interval` + `MissedTickBehavior`** (~400 lines). Rebase risk if #33
   lands first, since both touch `timer/`.
4. **`sync::Mutex` + `Semaphore::is_closed`** (~400 lines). `Mutex` needs
   `is_closed`, so they ship together.
5. **`sync::OnceCell`** (~250 lines).
6. **`channels::oneshot` + `channels::watch`** (~600 lines). Two small channels
   with one review context.
7. **`channels::broadcast`** (~700 lines). Its own PR: the semantics are the
   substance, and the design doc travels with it.
8. **`CancellationToken`** (~350 lines). Last of the additions because it is
   the one with an argument attached — `!Send` means it cannot carry a
   cross-shard shutdown, and a maintainer may reasonably want the `Send` one
   instead.
9. **`ConnectedSender::into_foreign`** (~250 lines). Touches `shared_channel`,
   so it wants a rested reviewer rather than a ninth-in-a-row one.

**Never more than two of these open at once.** The failure mode to avoid is
recreating today's queue of six.

### Stage 3 — the structural ones, each behind a hard dependency

- **Deleting the vendored `liburing` and the five C files.** Strictly
  downstream of #35: their `main` still has `iou`/`uring_sys` calling into that
  C. Cannot be offered before #35 lands, and is trivial afterwards — the
  finding that made it possible is that *nothing references the C symbols* once
  the `io-uring` migration is complete.
- **`#[glommio::main]` / `#[glommio::test]`.** Raise as an **issue**, never as
  an unsolicited PR. Merging it obliges them to publish and maintain a second
  crate on crates.io under their name, permanently. That is a policy decision,
  not a code review, and no amount of test coverage makes it for them. The
  branch is ready (`upstream/macros-proposal`) if they say yes.

### Verified porting mechanics

Every row below was tested by cherry-picking onto `upstream/main` on
2026-08-20, not estimated.

| Item | Commit | Onto their `main` alone | Note |
|---|---|---|---|
| `spawn_blocking_send` | `61bb142` | **clean**, 1 file, +228 | — |
| `sync::Mutex` + `Semaphore::is_closed` | `133a565` | **clean**, 3 files, +257 | — |
| `channels::oneshot` | `c362fb0` | **clean**, 2 files, +226 | — |
| `channels::broadcast` | `f86bb8a` | **clean**, 2 files, +482 | design doc travels with it |
| `ConnectedSender::into_foreign` | `986213e` | **clean**, 1 file, +302 | — |
| `watch` + `OnceCell` | `b67355f` | module-list conflict alone; clean when stacked | **split into two PRs** — one commit, two unrelated features |
| `CancellationToken` | `5bf58cd` | `sync/mod.rs` list conflict; clean when stacked | — |
| `future::timeout` + `Interval` | `8a867b9` | `timer/mod.rs` list conflict, 3 lines | **split into two PRs**; their `timeout` sits at `timer/mod.rs:56`, unchanged |
| `spawn_blocking` panic fix | `dcab422` | code clean; test module conflicts | re-home the tests |
| Kernel probe error | `cd7c9d3` | **conflicts on `sys/uring.rs`** | needs rewriting against their `iou` probe — see Stage 1 |
| eventfd leak test race | `eff2c49` | conflicts | their test file differs; port the mutex idea, not the diff |
| Delete the vendored C | `322fb8d` | n/a | **hard-gated on #35** — verified: `iou` and `uring_sys` are still in their tree |

**Applied in order, the seven additive commits stack onto `upstream/main` with
zero conflicts and the result compiles.** The conflicts in that table are
almost all one kind — two branches adding lines to the same `mod` / `pub use`
list — and they evaporate when the PRs land in sequence. That is worth knowing
before anyone budgets rebase time for this.

### Rules that apply to every stage

- **Strip the fork.** No `glommio-ng` in any diff. The macros branch already
  uses a generic alias in its `crate = ` examples for this reason.
- **One coherent change per PR**, rebased on their `main`, with tests, and
  under ~600 lines including them.
- **Nothing new opens while the previous stage's gate is shut.** If the six
  existing PRs are still untouched, the answer to "should we send this too" is
  no, regardless of how good it is.
- **If the canonical `glommio` name transfers to us, this plan is void.**
  Upstream becomes us and the mergeability constraint dissolves — including the
  one that shaped several decisions this week.

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
