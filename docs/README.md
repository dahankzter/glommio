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
**Status:** Documented, ready for implementation
**Severity:** Medium (API design issue)

Investigation of confusing `spawn_local()` API that panics even when called on a `LocalExecutor` instance. The current design ignores `self` and uses thread-local storage instead.

**Key Finding:** The private `spawn()` method actually uses `self` but is not public. Making it public solves the issue without breaking changes.

**Recommended Fix:**
- Make `LocalExecutor::spawn()` public (trivial change)
- Maintains thread-safety via `!Send` trait
- No breaking changes to existing API

### [Unsafe Code Centralization Analysis](./investigations/unsafe-centralization/)
**Status:** Analysis complete
**Complexity:** High (7-12 weeks refactoring)

Comprehensive analysis of eliminating or centralizing unsafe code in glommio without performance degradation. Identifies 320 unsafe blocks scattered across 43+ files and proposes centralization into 4 core modules.

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
    └── unsafe-centralization/
        └── README.md         # Unsafe code analysis & centralization strategy
```

## Contributing

This fork is maintained by [@dahankzter](https://github.com/dahankzter) while awaiting upstream response. If you encounter issues or have fixes, please open an issue or PR.

## Upstream Status

- **Original Repository:** [DataDog/glommio](https://github.com/DataDog/glommio)
- **Original Maintainer:** Glauber Costa (no longer at DataDog)
- **Current Status:** Awaiting new maintainer assignment

If DataDog resumes active maintenance, improvements from this fork can be contributed upstream.
