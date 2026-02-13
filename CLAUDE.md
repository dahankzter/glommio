# Glommio Fork - Development Guide

## Project Context

This is a **maintained fork** of [DataDog/glommio](https://github.com/DataDog/glommio) created while awaiting upstream maintainer engagement. The original maintainer (Glauber Costa) is no longer at DataDog, and the repository has limited activity.

**Fork Purpose:**
- Fix critical safety issues (memory corruption, resource leaks)
- Document complex architectural problems with comprehensive investigations
- Provide working solutions while upstream is in transition
- Maintain compatibility for production users who depend on Glommio

**Upstream Status:**
- Original Repository: https://github.com/DataDog/glommio
- Fork Repository: https://github.com/dahankzter/glommio
- If DataDog resumes maintenance, improvements from this fork can be contributed upstream

## Development Environment

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
│       └── issue_448/          # Eventfd leak investigation
│           ├── README.md       # Detailed root cause analysis
│           └── reproduce.rs    # Reproduction test
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

### 📋 Documented Issues

**Issue #448 - Eventfd Leak on Executor Drop**
- **Severity:** High (resource exhaustion in long-running apps)
- **Root Cause:** Non-runnable tasks don't have destructors called, Arc<SleepNotifier> leaks
- **Status:** Comprehensively documented with workarounds
- **Documentation:** `docs/investigations/issue_448/README.md`
- **Quote from Original Maintainer:** "Really hard because tasks often get destroyed under our nose. This brought me back to the refcount hell in the task structures."

**Recommended Workarounds:**
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

5. **Test thoroughly:**
   ```bash
   make all  # Format, lint, and test
   ```

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
- **Run full test suite** before pushing: `make all`
- **Check CI** after pushing - rebase on green PRs if needed

### Running Tests on Different Platforms

The Makefile handles platform differences automatically:

```bash
make test              # Works on both Linux and macOS
make test-lib          # Run library tests only (faster)
make bench             # Run benchmarks
```

On macOS, this runs in Lima. On Linux, it runs natively.

## Git Configuration

### Remote Setup

This fork uses the following remote configuration:

```bash
git remote -v
# origin        git@github.com:dahankzter/glommio.git (fork)
# upstream      https://github.com/DataDog/glommio.git (original)
```

### Working with the Fork

```bash
# Push to fork
git push origin <branch>

# Create PR to upstream
gh pr create --repo DataDog/glommio --base master

# Sync fork with upstream
gh repo sync dahankzter/glommio --source DataDog/glommio
```

## Code Style

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

### Git-specific

**Problem:** Submodule showing as dirty
```bash
git config submodule.glommio/liburing.ignore dirty
```

## Getting Help

- **GitHub Issues:** https://github.com/dahankzter/glommio/issues
- **Upstream Issues:** https://github.com/DataDog/glommio/issues
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
