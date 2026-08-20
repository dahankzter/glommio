# Glommio Fork - Development Guide

## Project Context

This is a **maintained fork** of [DataDog/glommio](https://github.com/DataDog/glommio), created when that repository went quiet.

**The situation has since changed (checked 2026-08-02).** `DataDog/glommio` is
abandoned — last commit 2025-04-21, sixteen open PRs, the oldest from 2021. A
community fork at **[glommio/glommio](https://github.com/glommio/glommio)** took
over: a new org with the original author's blessing to take the crates.io name
(DataDog issue #707). This fork's `#700` fix is already merged there.

**But that fork has gone quiet too (checked 2026-08-10).** Last commit
2026-06-13, last merged PR 2026-06-22. Six of our PRs have been open since
2026-08-02 with no review, no comments, and no CI runs. Not abandoned — a
volunteer project between bursts — but **do not plan around an upstream merge
landing.** This fork is the artifact consumers should depend on. Keep the PRs
open, keep them rebasing cleanly, don't add more, don't chase. See
[docs/UPSTREAM.md](docs/UPSTREAM.md#upstream-activity).

**So upstream means `glommio/glommio`, not DataDog.** See
[docs/UPSTREAM.md](docs/UPSTREAM.md) for what is worth contributing, in what
order, and where it conflicts with their 15 commits we do not have.

**Fork Purpose:**
- Fix critical safety issues (memory corruption, resource leaks)
- Document complex architectural problems with comprehensive investigations
- Provide working solutions while upstream is in transition
- Maintain compatibility for production users who depend on Glommio

**Upstream Status:**
- Abandoned original: https://github.com/DataDog/glommio (last commit 2025-04-21)
- Community fork: https://github.com/glommio/glommio — contribute here, but it
  has been quiet since 2026-06-22
- This fork: https://github.com/dahankzter/glommio — merged with the community
  fork on 2026-08-02, now ahead only. **This is what consumers should use.**
- `io-uring` is a plain crates.io dependency at **0.7.14**, which carries the
  accessor `need_preempt` needs
  ([#404](https://github.com/tokio-rs/io-uring/pull/404), merged 2026-08-09,
  released 2026-08-11). **No git dependencies anywhere — this fork is
  publishable.**

## Development Environment

### Recommended Allocator

Glommio allocates one block per spawned task and frees it on the same thread.
The global allocator choice dominates that path, and mimalloc is the
recommendation for deployments:

```rust
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

At 512 tasks live, spawn costs ~45ns under glibc malloc, ~30ns under jemalloc
and ~24ns under mimalloc. Glommio deliberately does **not** put its own cache in
front of the allocator: a thread-local free list was tried and measured, and it
recovered most of the glibc gap but was worth nothing at all under mimalloc,
while retaining memory and displacing use-after-free diagnostics. Use a good
allocator instead.

Reproduce with:
```bash
RUSTFLAGS='--cfg alloc_mimalloc' cargo run --release \
  --example alloc_compare
```

### Platform Support

Glommio requires **Linux with io_uring support** (kernel 5.8+). This fork provides seamless development on both Linux and macOS:

- **Linux**: Uses native cargo and direct io_uring access
- **macOS**: Uses [Lima](https://lima-vm.io/) VM for Linux compatibility
- **Windows**: Should work with Lima as well (untested)

### Why Lima?

Lima provides a lightweight Linux VM specifically designed for Mac development:
- Automatic file sharing between macOS and Linux
- Native command integration (`lima` wraps Linux commands)
- Much lighter than Docker Desktop or full VMs
- Preserves your native macOS development workflow

The Makefile automatically detects your platform and routes commands appropriately!

### Setup Instructions

#### On Linux
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# You're ready to go!
make test
```

#### On macOS
```bash
# Install Lima (via Homebrew)
brew install lima

# Start the default Lima VM
limactl start

# Install Rust in the Lima VM
lima cargo --version  # This will install cargo if needed

# You're ready to go!
make test
```

The Makefile handles everything else automatically!

## Using the Makefile

The Makefile provides a unified interface for all development tasks. Commands work identically on Linux and macOS:

### Common Commands

```bash
# Testing
make test              # Run all tests
make test-lib          # Run library tests only
make bench             # Run benchmarks

# Code Quality
make fmt               # Format all code
make lint              # Run clippy linter
make check             # Check compilation

# Building
make build             # Build debug
make build-release     # Build optimized release
make build-examples    # Build examples

# Cleanup
make clean             # Remove build artifacts

# Meta
make all               # Format + lint + test (pre-commit check)
make ci                # Everything CI runs (pre-PUSH check)
```

### How It Works

The Makefile uses platform detection to route commands:

```makefile
ifeq ($(UNAME_S),Darwin)
    # macOS: Use Lima for io_uring support
    define run_cargo
        lima sh -c '. ~/.cargo/env && cargo $(1)'
    endef
else
    # Linux: Use native cargo
    define run_cargo
        cargo $(1)
    endef
endif
```

When you run `make test` on macOS, it becomes:
```bash
lima sh -c '. ~/.cargo/env && cargo test --workspace'
```

On Linux, it's simply:
```bash
cargo test --workspace
```

## Project Structure

```
glommio/
├── CLAUDE.md                    # This file - development guide
├── Makefile                     # Platform-aware build commands
├── docs/
│   ├── README.md               # Documentation index
│   └── investigations/
│       ├── issue_448/          # Eventfd leak investigation
│       │   ├── README.md       # Detailed root cause analysis
│       │   └── reproduce.rs    # Reproduction test
│       └── task-arena/         # Arena allocator: built, measured, reverted
│           └── README.md       # Post-mortem — read before proposing an arena
├── glommio/
│   ├── src/
│   │   ├── channels/
│   │   │   ├── spsc_queue.rs   # Fixed: Issue #700 (memory corruption)
│   │   │   └── shared_channel.rs
│   │   ├── sys/mod.rs          # SleepNotifier (related to #448)
│   │   ├── task/               # Task lifecycle management
│   │   └── executor/           # LocalExecutor implementation
│   ├── benches/                # Benchmarks
│   └── tests/                  # Integration tests
└── .gitignore                  # Excludes .claude/ and test files
```

## Work Completed

### ✅ Fixed Issues

**Issue #700 - Memory Corruption in SPSC Queue**
- **Severity:** Critical (heap corruption in safe code)
- **Root Cause:** Public `Clone` trait allowed multiple producers/consumers
- **Fix:** Removed public Clone, added crate-private `clone_internal()`
- **Status:** Fixed in PR #703 on upstream
- **Branch:** `fix/issue-700-remove-spsc-clone`

**Issue #448 - Eventfd Leak on Executor Drop**
- **Severity:** High (resource exhaustion in long-running apps)
- **Root Cause:** Non-runnable tasks don't have destructors called, so the
  `Arc<SleepNotifier>` each task held was never dropped and the eventfd leaked
- **Fix:** The task header stores `executor_id: usize` instead of the notifier
  (`f28a619`); the notifier is resolved only on the foreign-wake path, so tasks
  no longer pin the eventfd at all. This also removed a process-wide `RwLock`
  read from every spawn.
- **Documentation:** `docs/investigations/issue_448/README.md`
- **Quote from Original Maintainer:** "Really hard because tasks often get destroyed under our nose. This brought me back to the refcount hell in the task structures."

**Reverted Work**

**Task Arena Allocator** — built, benchmarked, removed. Read
`docs/investigations/task-arena/README.md` before proposing any glommio-side
allocator: the premise (malloc lock contention on the spawn path) was measured
and does not hold, and the arena cost ~98 MB resident per executor while
segfaulting on detached tasks. Recommend mimalloc to deployments instead.

### 📋 Historical Workarounds (#448, superseded by the fix above)
1. Use long-lived executors (don't create/destroy repeatedly)
2. Thread-local executor pattern for tests
3. Increase file descriptor limits: `ulimit -n 65536`

## Development Workflow

### Starting New Work

1. **Create a feature branch:**
   ```bash
   git checkout -b fix/issue-XXX-description
   ```

2. **Read relevant code:**
   ```bash
   # Use Glob/Grep tools to find code
   # Read files thoroughly before modifying
   ```

3. **Write reproduction test first** (if applicable):
   - Demonstrates the bug clearly
   - Can be used to verify the fix
   - Include in the PR

4. **Implement fix:**
   - Keep changes focused and minimal
   - Add comments explaining safety/correctness
   - Update documentation as needed

5. **Format and test before committing:**
   ```bash
   make fmt   # ALWAYS run before committing (formats all code)
   make lint  # Check for linting issues
   make test  # Run all tests

   # Or run all at once:
   make all   # Format, lint, and test
   ```

   **CRITICAL:** Always run `make fmt` before committing to avoid CI warnings!

   **And `make ci` before pushing.** `make all` does not compile everything CI
   compiles, which is how CI stayed red from 2026-08-02 to 2026-08-20 while
   every local run was green. Three of those four failures lived in code a
   default build never touches: two behind `--all-features` (the `debugging`
   and `native-tls` paths), one behind the musl target, one in Cargo.toml
   ordering. `make ci` runs all of them.

6. **Create commit** (see Commit Message Conventions below):
   ```bash
   git add <files>
   git commit -s  # Signed-off commit
   ```

7. **Push and create PR:**
   ```bash
   git push origin fix/issue-XXX-description
   gh pr create --title "Fix: Issue #XXX description"
   ```

### Investigation Workflow

For complex issues that may not have immediate fixes:

1. **Create investigation branch:**
   ```bash
   git checkout -b investigate/issue-XXX
   ```

2. **Document your findings:**
   - Create `docs/investigations/issue_XXX/README.md`
   - Include root cause analysis
   - Document attempted solutions
   - Provide workarounds
   - Rate complexity of potential fixes

3. **Create reproduction test:**
   - Save as `docs/investigations/issue_XXX/reproduce.rs`
   - Should clearly demonstrate the issue

4. **Merge documentation to master:**
   - Even if no fix is ready, documentation is valuable
   - Helps future developers understand the problem

## Commit Message Conventions

Follow these commit message standards for consistency and clarity. These conventions work with or without automation tools.

### Commit Workflow

Before writing your commit message, follow this workflow:

1. **Review what you're committing:**
   ```bash
   git status                # See all files
   git diff --cached         # Review staged changes
   ```

2. **Check recent commit style:**
   ```bash
   git log --oneline -5      # See recent messages
   git log --format=fuller -3  # See full messages with bodies
   ```

3. **Stage your files:**
   ```bash
   git add path/to/file.rs   # Add specific files
   # Or for multiple related files:
   git add path/to/file1.rs path/to/file2.rs
   ```

4. **Write your commit message** following the format below

5. **Always sign your commits:**
   ```bash
   git commit -s             # Opens editor with Signed-off-by
   # Or inline with HEREDOC for multi-line
   ```

### Format

```
type: subject line (imperative mood, max 72 chars, no period)

Optional body paragraph explaining what and why, not how.
Wrap at 72 characters for better readability in git log.

Multiple paragraphs are fine if needed.
```

### Commit Types

- `feat:` - New feature or enhancement
- `fix:` - Bug fix
- `refactor:` - Code restructuring without behavior change
- `docs:` - Documentation changes
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks (dependencies, build, etc.)
- `perf:` - Performance improvements

### Rules

1. **Subject line:**
   - Use imperative mood ("Add feature" not "Added feature")
   - Max 72 characters
   - No period at the end
   - Be specific but concise

2. **Body (optional but recommended for non-trivial changes):**
   - Blank line after subject
   - Wrap at 72 characters
   - Explain **what** and **why**, not how
   - Reference issue numbers if applicable

3. **Sign-off:**
   - Always use `git commit -s` for Developer Certificate of Origin
   - Adds `Signed-off-by: Your Name <your@email.com>`

4. **Never mention AI assistance:**
   - No "Generated by", "Co-Authored-By: Claude", etc.
   - Commits should reflect human authorship
   - Write as if you wrote the code yourself

### What NOT to Do

❌ **Bad examples - DO NOT use these patterns:**

```bash
# DON'T mention AI tools
git commit -m "fix: memory issue

Generated with Claude Code"

# DON'T add AI co-authorship
git commit -m "fix: memory issue

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"

# DON'T use past tense
git commit -m "fixed: memory issue"  # Wrong tense!

# DON'T add period to subject
git commit -m "fix: memory issue."  # No period!

# DON'T exceed 72 characters on subject
git commit -m "fix: memory corruption issue in the SPSC queue when cloning producers and consumers"  # Too long!

# DON'T explain HOW in body
git commit -m "fix: memory issue

Changed line 42 from x = y to x = z"  # Say WHY, not HOW!
```

✅ **Good examples - Use these patterns:**

```bash
# Simple and clean
git commit -s -m "fix: prevent memory corruption in SPSC queue clone"

# With explanatory body
git commit -s -m "$(cat <<'EOF'
fix: remove public Clone from SPSC queue types

Cloning violates single-producer-single-consumer guarantees
and causes memory corruption when multiple producers exist.

Fixes #700
EOF
)"
```

### Examples

**Simple one-line commit:**
```bash
git commit -s -m "fix: prevent memory corruption in SPSC queue clone"
```

**Commit with body using HEREDOC:**
```bash
git commit -s -m "$(cat <<'EOF'
fix: remove public Clone from SPSC queue types

Cloning Producer/Consumer violates single-producer-single-consumer
guarantees and causes memory corruption. The Clone trait was never
safe for public use.

Keep internal clone_internal() for shared_channel handoff logic.

Fixes #700
EOF
)"
```

**Documentation commit:**
```bash
git commit -s -m "$(cat <<'EOF'
docs: add comprehensive investigation of issue #448

Eventfd leak occurs when executors are repeatedly created/destroyed.
Root cause is Arc<SleepNotifier> held by non-runnable tasks that
never have destructors called.

Documents three potential fix approaches and practical workarounds
for production use.
EOF
)"
```

**Feature commit:**
```bash
git commit -s -m "$(cat <<'EOF'
feat: add platform-aware Makefile for seamless development

Automatically detects macOS vs Linux and routes cargo commands
appropriately. macOS uses Lima VM for io_uring support, Linux
uses native cargo.

Enables same commands to work on both platforms without thinking
about compatibility.
EOF
)"
```

### Why HEREDOC?

Using HEREDOC (`cat <<'EOF' ... EOF`) preserves formatting and allows multi-line messages:

```bash
# Good - preserves formatting
git commit -s -m "$(cat <<'EOF'
Subject line here

Body paragraph here.
EOF
)"

# Bad - hard to read, breaks on special characters
git commit -s -m "Subject line\n\nBody paragraph"
```

### Complete Workflow Example

Here's a full end-to-end commit workflow:

```bash
# 1. Check what you're committing
git status
# Output: Modified files: glommio/src/channels/spsc_queue.rs

# 2. Review the actual changes
git diff --cached glommio/src/channels/spsc_queue.rs
# Review the diff to understand what changed

# 3. Check recent commit message style
git log --oneline -5
# Output shows recent commit format - match this style!

# 4. Stage your changes (if not already staged)
git add glommio/src/channels/spsc_queue.rs

# 5. Write your commit using HEREDOC for multi-line
git commit -s -m "$(cat <<'EOF'
fix: remove public Clone from SPSC queue types

Cloning Producer/Consumer violates single-producer-single-consumer
guarantees and causes memory corruption when multiple producers exist.
The Clone trait was never safe for public use.

Keep internal clone_internal() for shared_channel handoff logic.

Fixes #700
EOF
)"

# 6. Verify the commit looks good
git log -1 --format=fuller
# Review your commit message

# 7. Push to your branch
git push origin fix/issue-700-remove-spsc-clone
```

### Quick Check

Before committing, ask:
1. Did I review `git status` and `git diff --cached`?
2. Did I check recent commit style with `git log`?
3. Is the subject line imperative and under 72 chars?
4. Does the commit do one logical thing?
5. Would a reviewer understand **why** this change was made?
6. Did I sign off with `-s`?
7. Did I avoid mentioning AI tools?

### Viewing Recent Commits

To see the style of recent commits:
```bash
git log --oneline -5        # Short format
git log --format=fuller -3  # See full messages with bodies
git show                    # See the last commit with diff
```

## Testing Philosophy

- **Write tests first** when fixing bugs
- **Test must fail** before the fix to validate it catches the issue
- **Run full test suite** before pushing: `make ci` (not `make all` — see below)
- **Check CI** after pushing - rebase on green PRs if needed

### Running Tests on Different Platforms

The Makefile handles platform differences automatically:

```bash
make test              # Works on both Linux and macOS
make test-lib          # Run library tests only (faster)
make bench             # Run benchmarks
```

On macOS, this runs in Lima. On Linux, it runs natively.

### Testing Unsafe Code with Miri

Miri is Rust's interpreter for detecting undefined behavior in unsafe code. Use it to validate task allocator safety:

```bash
# One-time setup (installs nightly + Miri)
make miri-setup

# Test task allocator unsafe code (fast, ~30 seconds)
make miri-alloc

# Test all library unsafe code (slow, several minutes)
make miri
```

**What Miri detects:**
- Use of uninitialized memory
- Use-after-free and double-free
- Invalid pointer arithmetic
- Data races
- Violations of pointer aliasing rules

**When to use Miri:**
- After modifying unsafe code in task/alloc.rs or task/raw.rs
- Before pushing changes that touch unsafe blocks
- When debugging memory corruption issues
- As part of thorough testing for safety-critical changes

**Note:** Miri is slower than regular tests (interprets every instruction), but catches undefined behavior that normal tests might miss.

## Git Configuration

### Remote Setup

```bash
git remote -v
# origin      https://github.com/dahankzter/glommio.git            (this fork, branch `master`)
# fork        https://github.com/dahankzter/glommio-community.git  (PR staging, branch `main`)
# upstream    https://github.com/glommio/glommio.git               (the live community fork)
```

**There are two GitHub forks, and this trips people up.** `dahankzter/glommio`
was forked from `DataDog/glommio` before the community fork existed, so GitHub
still labels it "forked from DataDog/glommio" — that badge is metadata fixed at
creation and cannot be repointed. It does **not** mean the remotes are wrong.
`dahankzter/glommio-community` was forked later from `glommio/glommio` and is
what upstream pull requests are raised from.

**The footgun:** GitHub's "Contribute" button on `dahankzter/glommio` defaults
the pull request base to `DataDog/glommio`, which is abandoned. Never accept
that default.

### Working with the Fork

```bash
# Day-to-day work lives on origin/master
git push origin master

# Upstream pull requests go through the community fork, based on `main`
git push fork <local-branch>:<pr-branch>
gh pr create --repo glommio/glommio --base main

# Sync with upstream
git fetch upstream && git merge upstream/main
```

## Code Style

- **ALWAYS run `make fmt` before committing** - prevents CI warnings
- **Follow existing patterns** in the codebase
- **Keep changes minimal** - don't refactor unrelated code
- **Comment safety invariants** especially in unsafe code
- **Document public APIs** thoroughly
- **Use descriptive commit messages** - see [Commit Message Conventions](#commit-message-conventions)

### Rust Guidelines

- Prefer safe code over unsafe when possible
- When using unsafe, document why it's safe
- Use proper memory ordering (Acquire/Release, not Relaxed unless proven safe)
- Consider edge cases (Drop, Clone, panic safety)

## Common Issues

### Lima-specific

**Problem:** Lima VM is slow or unresponsive
```bash
limactl stop
limactl start
```

**Problem:** File changes not syncing
```bash
# Lima auto-mounts your home directory, should work automatically
# If issues persist, restart Lima
```

## Getting Help

- **GitHub Issues:** https://github.com/dahankzter/glommio/issues
- **Upstream Issues:** https://github.com/glommio/glommio/issues — the live fork
- **DataDog issues:** https://github.com/DataDog/glommio/issues — abandoned, but
  still worth reading; several describe problems this fork has since measured or
  fixed (see `docs/investigations/`)
- **Maintainer:** @dahankzter

## Quick Reference for New Claude Sessions

When starting a new session:

1. **Check for HANDOFF.md** - may contain session context
2. **Read this CLAUDE.md** - understand the project
3. **Check `docs/README.md`** - see what's been done
4. **Review recent commits** - understand current work
5. **Use the Makefile** - don't run cargo directly

### Essential Commands

```bash
# BEFORE EVERY COMMIT - Format code to avoid CI warnings!
make fmt            # Format all code (ALWAYS run before committing!)

# Testing and quality checks
make test           # Test everything
make all            # Pre-commit checks (format, lint, test)
make help           # Show all available commands

# Committing (see Commit Message Conventions section)
git commit -s       # Signed-off commit with proper message format
git log --oneline -5  # View recent commit message style
```

### Platform Detection

The Makefile automatically detects your platform. You'll see:
```
Platform: macOS (via Lima)
Note:     Using Lima VM for io_uring support
```

or

```
Platform: Linux (native)
Note:     Direct io_uring access
```

This means **you never need to think about platform differences** - just use `make` commands!
