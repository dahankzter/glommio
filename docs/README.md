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

## Repository Structure

```
docs/
├── README.md (this file)
└── investigations/
    └── issue_448/
        ├── README.md         # Detailed analysis
        └── reproduce.rs      # Test demonstrating the leak
```

## Contributing

This fork is maintained by [@dahankzter](https://github.com/dahankzter) while awaiting upstream response. If you encounter issues or have fixes, please open an issue or PR.

## Upstream Status

- **Original Repository:** [DataDog/glommio](https://github.com/DataDog/glommio)
- **Original Maintainer:** Glauber Costa (no longer at DataDog)
- **Current Status:** Awaiting new maintainer assignment

If DataDog resumes active maintenance, improvements from this fork can be contributed upstream.
