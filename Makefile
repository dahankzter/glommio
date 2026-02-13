# Glommio Development Makefile
#
# Quick commands for common development tasks

.PHONY: help test build fmt lint check bench clean all
.PHONY: install-tools

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
	@echo "  make bench             - Run benchmarks"
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
	@$(call run_cargo,install cargo-llvm-cov)
	@echo "✓ Tools installed"

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
