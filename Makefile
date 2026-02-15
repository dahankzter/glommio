# Glommio Development Makefile
#
# Quick commands for common development tasks

.PHONY: help test build fmt lint check bench clean all
.PHONY: install-tools coverage coverage-summary coverage-lcov coverage-open
.PHONY: bench-timer bench-ci

# =============================================================================
# Platform Detection & Smart Cargo Commands
# =============================================================================
#
# Automatically detects OS and routes cargo commands appropriately:
#   - macOS:  Uses Lima VM (io_uring support for Glommio)
#   - Linux:  Uses native cargo (direct io_uring access)
#
# This allows the same Makefile commands to work on both platforms!
#

# Detect operating system
UNAME_S := $(shell uname -s)

# Lima configuration (for macOS)
# Use home directory instead of /tmp (which is only 2GB tmpfs)
LIMA_TARGET_DIR := ~/glommio-target

# Platform-aware cargo command
ifeq ($(UNAME_S),Darwin)
    # macOS: Use Lima for Glommio compatibility
    PLATFORM := macOS (via Lima)
    PLATFORM_NOTE := Using Lima VM for io_uring support
    define run_cargo
		lima sh -c '. ~/.cargo/env && CARGO_TARGET_DIR=$(LIMA_TARGET_DIR) cargo $(1)'
    endef
else ifeq ($(UNAME_S),Linux)
    # Linux: Use native cargo
    PLATFORM := Linux (native)
    PLATFORM_NOTE := Direct io_uring access
    define run_cargo
		cargo $(1)
    endef
else
    # Other Unix
    PLATFORM := $(UNAME_S)
    PLATFORM_NOTE := Using native cargo
    define run_cargo
		cargo $(1)
    endef
endif

# =============================================================================
# Default target
# =============================================================================
help:
	@echo "Glommio Development Commands"
	@echo "============================="
	@echo ""
	@echo "Platform: $(PLATFORM)"
	@echo "Note:     $(PLATFORM_NOTE)"
	@echo ""
	@echo "Testing:"
	@echo "  make test              - Run all tests"
	@echo "  make test-lib          - Run library tests only"
	@echo ""
	@echo "Benchmarking:"
	@echo "  make bench             - Run all benchmarks"
	@echo "  make bench-timer       - Run timer benchmarks only"
	@echo "  make bench-spawn       - Run spawn benchmarks only"
	@echo "  make bench-ci          - Run benchmarks in CI format"
	@echo ""
	@echo "Coverage:"
	@echo "  make coverage-summary  - Quick coverage summary in terminal"
	@echo "  make coverage          - Generate HTML coverage report"
	@echo "  make coverage-open     - Generate and open HTML report"
	@echo "  make coverage-lcov     - Generate lcov format (for CI)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt               - Format all code"
	@echo "  make lint              - Run clippy linter"
	@echo "  make check             - Check compilation without building"
	@echo ""
	@echo "Build:"
	@echo "  make build             - Build all crates"
	@echo "  make build-release     - Build optimized release"
	@echo "  make build-examples    - Build examples"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean             - Remove build artifacts"
	@echo ""
	@echo "Tools:"
	@echo "  make install-tools     - Install development tools"
	@echo ""
	@echo "Meta:"
	@echo "  make all               - Format, lint, test"

# =============================================================================
# Testing
# =============================================================================

test:
	@echo "→ Running all tests on $(PLATFORM)..."
	@$(call run_cargo,test --workspace)

test-lib:
	@echo "→ Running library tests on $(PLATFORM)..."
	@$(call run_cargo,test --package glommio --lib)

bench:
	@echo "→ Running benchmarks on $(PLATFORM)..."
	@$(call run_cargo,bench --benches)

bench-timer:
	@echo "→ Running timer benchmarks on $(PLATFORM)..."
	@$(call run_cargo,bench --bench timer_benchmark)

bench-spawn:
	@echo "→ Running spawn benchmarks on $(PLATFORM)..."
	@$(call run_cargo,bench --bench spawn_benchmark)

bench-ci:
	@echo "→ Running benchmarks in CI mode (bencher format) on $(PLATFORM)..."
	@$(call run_cargo,bench --bench timer_benchmark -- --output-format bencher)

# =============================================================================
# Coverage
# =============================================================================

coverage-summary:
	@echo "→ Generating coverage summary on $(PLATFORM)..."
	@$(call run_cargo,llvm-cov --lib --summary-only -- --skip test_shares_high_disparity)

coverage:
	@echo "→ Generating HTML coverage report on $(PLATFORM)..."
	@$(call run_cargo,llvm-cov --lib --html -- --skip test_shares_high_disparity)
	@echo ""
	@echo "✓ Coverage report generated!"
	@echo "  View at: target/llvm-cov/html/index.html"
	@echo "  Note: Skips slow stress tests (test_shares_high_disparity)"

coverage-open: coverage
	@echo "→ Opening coverage report..."
ifeq ($(UNAME_S),Darwin)
	@open target/llvm-cov/html/index.html
else
	@xdg-open target/llvm-cov/html/index.html 2>/dev/null || \
		echo "Please open target/llvm-cov/html/index.html manually"
endif

coverage-lcov:
	@echo "→ Generating lcov coverage report on $(PLATFORM)..."
	@$(call run_cargo,llvm-cov --lib --lcov --output-path lcov.info -- --skip test_shares_high_disparity)
	@echo "✓ lcov.info generated (skips slow stress tests)"

# =============================================================================
# Code Quality
# =============================================================================

fmt:
	@echo "→ Formatting code on $(PLATFORM)..."
	@$(call run_cargo,fmt --all)

lint:
	@echo "→ Running clippy on $(PLATFORM)..."
	@$(call run_cargo,clippy --workspace --all-targets --all-features -- -D warnings)

check:
	@echo "→ Checking compilation on $(PLATFORM)..."
	@$(call run_cargo,check --workspace --all-targets)

# =============================================================================
# Build
# =============================================================================

build:
	@echo "→ Building debug on $(PLATFORM)..."
	@$(call run_cargo,build --workspace)

build-release:
	@echo "→ Building release on $(PLATFORM)..."
	@$(call run_cargo,build --workspace --release)

build-examples:
	@echo "→ Building examples on $(PLATFORM)..."
	@$(call run_cargo,build --examples)

# =============================================================================
# Cleanup
# =============================================================================

clean:
	@echo "→ Cleaning build artifacts on $(PLATFORM)..."
	@$(call run_cargo,clean)
ifeq ($(UNAME_S),Darwin)
	@echo "→ Cleaning Lima target directory..."
	@lima sh -c 'rm -rf $(LIMA_TARGET_DIR)' 2>/dev/null || true
endif

# =============================================================================
# Development Tools
# =============================================================================

install-tools:
	@echo "→ Installing development tools on $(PLATFORM)..."
	@echo "  - cargo-llvm-cov (for coverage reports)"
	@$(call run_cargo,install cargo-llvm-cov)
	@echo ""
	@echo "✓ Tools installed successfully!"
	@echo "  Try: make coverage-summary"

# =============================================================================
# Meta Commands
# =============================================================================

all: fmt lint test
	@echo ""
	@echo "✓ All checks passed!"
	@echo "  - Code formatted"
	@echo "  - Linting passed"
	@echo "  - Tests passed"

# CI-friendly target
ci: fmt lint test
	@echo "✓ CI checks complete"
