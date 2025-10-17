# Cargo Make Guide for Clnrm

## Table of Contents

1. [Getting Started](#getting-started)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Task Categories](#task-categories)
5. [Common Workflows](#common-workflows)
6. [Advanced Usage](#advanced-usage)
7. [Task Reference](#task-reference)
8. [Troubleshooting](#troubleshooting)
9. [CI/CD Integration](#cicd-integration)
10. [Best Practices](#best-practices)

---

## Getting Started

`cargo-make` is a task runner and build tool for Rust projects that provides a cross-platform way to define and run complex build workflows. This guide covers how to use cargo-make with the clnrm testing framework.

### Why cargo-make?

- **Consistency**: Define workflows once, run anywhere
- **Simplicity**: Easy-to-read TOML configuration
- **Power**: Complex task dependencies and compositions
- **Extensibility**: Platform-specific tasks and custom scripts
- **Developer Experience**: Reduces command memorization

---

## Installation

### Install cargo-make

```bash
# Install cargo-make
cargo install cargo-make

# Verify installation
cargo make --version
```

### Install Optional Development Tools

```bash
# Install all development tools at once
cargo make setup-dev

# Or install individually
cargo install cargo-watch      # Watch for file changes
cargo install cargo-tree       # Dependency tree visualization
cargo install cargo-outdated   # Check for outdated dependencies
cargo install cargo-audit      # Security audit
cargo install cargo-bloat      # Binary size analysis
cargo install cargo-license    # License checker
```

---

## Quick Start

### List Available Tasks

```bash
# Show quick help with common tasks
cargo make

# List all available tasks
cargo make --list-all-steps

# Get detailed information about a specific task
cargo make --print-steps <task-name>
```

### Run Common Tasks

```bash
# Quick development cycle (format + clippy + test)
cargo make dev

# Fast check (compilation check + quick test)
cargo make quick

# Full CI pipeline
cargo make ci

# Install clnrm binary
cargo make install
```

---

## Task Categories

### 1. Installation Tasks

Install, uninstall, and manage the clnrm binary.

```bash
# Install clnrm to ~/.cargo/bin
cargo make install

# Install with debug symbols (for development)
cargo make install-dev

# Uninstall clnrm
cargo make uninstall

# Reinstall (uninstall + install)
cargo make reinstall
```

**Use Cases:**
- `install`: Production installation after building release
- `install-dev`: Development with better backtraces
- `reinstall`: After major changes to verify fresh install

---

### 2. Build Tasks

Build workspace crates in various configurations.

```bash
# Build all default workspace crates (excludes clnrm-ai)
cargo make build

# Release build with optimizations
cargo make build-release

# Build only clnrm-core library
cargo make build-core

# Build only clnrm binary
cargo make build-bin

# Build with OpenTelemetry features
cargo make build-otel

# Build experimental AI crate
cargo make build-ai

# Build ALL crates including AI
cargo make build-all
```

**Build Strategy:**
- `build`: Daily development
- `build-release`: Before installation or benchmarking
- `build-core`: When working on core library only
- `build-otel`: Testing observability features
- `build-ai`: Experimental features (isolated from production)

---

### 3. Test Tasks

Run tests at various levels of granularity.

```bash
# Run library tests
cargo make test

# Run unit tests only (single-threaded)
cargo make test-unit

# Run integration tests
cargo make test-integration

# Run tests for clnrm-core only
cargo make test-core

# Run tests with output visible
cargo make test-verbose

# Quick test run (fast feedback)
cargo make test-quick

# Run all tests (unit + integration)
cargo make test-all

# Run property-based tests (160K+ generated cases)
cargo make test-proptest

# Run tests with OTEL features
cargo make test-otel
```

**Testing Strategy:**
- `test-quick`: During active development
- `test`: Regular development cycle
- `test-all`: Before commits
- `test-proptest`: Before releases (comprehensive)
- `test-otel`: When working on telemetry

---

### 4. Quality Tasks

Ensure code quality and formatting.

```bash
# Run clippy with strict warnings (zero warnings allowed)
cargo make clippy

# Auto-fix clippy warnings
cargo make clippy-fix

# Format all code
cargo make fmt

# Check formatting without modifying files
cargo make fmt-check

# Fast compilation check (no code generation)
cargo make check

# Check with all features
cargo make check-all

# Check with OTEL features
cargo make check-otel
```

**Quality Workflow:**
```bash
# Before committing
cargo make fmt      # Format code
cargo make clippy   # Check for issues

# Or auto-fix everything
cargo make fix      # fmt + clippy-fix
```

---

### 5. Development Tasks

Streamline daily development workflows.

```bash
# Quick development cycle: fmt + clippy + test
cargo make dev

# Watch for changes and rebuild
cargo make watch

# Watch and run tests on changes
cargo make watch-test

# Clean build artifacts
cargo make clean

# Deep clean (remove target/ directory)
cargo make clean-all

# Fast iteration (check + quick test)
cargo make quick
```

**Development Patterns:**

```bash
# Pattern 1: Manual iteration
cargo make dev          # After making changes

# Pattern 2: Automated watching
cargo make watch        # Automatically runs on file changes

# Pattern 3: Ultra-fast feedback
cargo make quick        # Minimal checks for rapid iteration
```

---

### 6. Documentation Tasks

Build and manage project documentation.

```bash
# Build documentation for all features
cargo make doc

# Build and open documentation in browser
cargo make doc-open

# Build documentation including private items
cargo make doc-private

# Build documentation for clnrm-core only
cargo make doc-core
```

**Documentation Workflow:**
```bash
# Review API documentation
cargo make doc-open

# Check documentation builds before release
cargo make doc
```

---

### 7. CI/CD Tasks

Pre-configured workflows for continuous integration.

```bash
# Full CI pipeline (fmt-check + clippy + test + build-release)
cargo make ci

# Pre-commit checks (quick validation)
cargo make pre-commit

# Verify release readiness (comprehensive)
cargo make release-check

# Complete validation (CI + docs + audit)
cargo make validate

# Complete validation with benchmarks
cargo make full-check
```

**CI Strategy:**

| Task | Use Case | Time | Coverage |
|------|----------|------|----------|
| `pre-commit` | Before git commit | ~30s | Quick checks |
| `ci` | CI pipeline | ~2min | Standard checks |
| `release-check` | Before release | ~5min | Comprehensive |
| `full-check` | Pre-release | ~10min | Everything |

---

### 8. Clnrm-Specific Tasks

Tasks specific to the clnrm framework.

```bash
# Run clnrm's self-test suite
cargo make self-test

# Initialize clnrm in examples directory
cargo make init-example

# Run example tests
cargo make run-examples

# Run all benchmarks
cargo make benchmarks

# Run hot reload benchmark specifically
cargo make bench-hot-reload
```

**Self-Testing Workflow:**
```bash
# Build and verify framework
cargo make build-release
cargo make self-test

# Test examples
cargo make run-examples
```

---

### 9. Publishing Tasks

Prepare and publish crates to crates.io.

```bash
# Dry-run publish to verify package contents
cargo make publish-check

# Publish all crates to crates.io (requires confirmation)
cargo make publish

# Check current versions and bump guidance
cargo make version-bump
```

**Publishing Workflow:**

```bash
# 1. Verify everything is ready
cargo make release-check

# 2. Dry-run publish
cargo make publish-check

# 3. Update versions
cargo make version-bump
# Manually edit Cargo.toml files

# 4. Publish (after manual confirmation)
cargo make publish
```

**Publishing Order:**
1. `clnrm-shared` (no dependencies)
2. `clnrm-core` (depends on shared)
3. `clnrm` (depends on core)

The `publish` task handles this order automatically with 30-second delays for crates.io indexing.

---

### 10. Utility Tasks

Analysis and maintenance utilities.

```bash
# Check dependency tree
cargo make deps

# Find duplicate dependencies
cargo make deps-duplicates

# Check for outdated dependencies
cargo make outdated

# Security audit
cargo make audit

# Find what's taking up space in binary
cargo make bloat

# Find what's taking longest to compile
cargo make bloat-time

# Update dependencies
cargo make update

# List all dependency licenses
cargo make licenses
```

**Maintenance Workflow:**

```bash
# Weekly maintenance
cargo make audit            # Security check
cargo make outdated         # Dependency updates
cargo make deps-duplicates  # Optimization opportunities

# Before release
cargo make bloat            # Binary size analysis
cargo make licenses         # License compliance
```

---

## Common Workflows

### Daily Development

```bash
# Option 1: Manual iteration
cargo make dev              # Run full dev cycle
# Make changes
cargo make dev              # Run again

# Option 2: Automated watching
cargo make watch            # Runs automatically on changes
# Make changes (tests run automatically)

# Option 3: Ultra-fast feedback
cargo make quick            # Minimal checks
```

### Before Committing

```bash
# Quick pre-commit checks
cargo make pre-commit

# Or comprehensive validation
cargo make ci
```

### Before Creating a Pull Request

```bash
# Complete validation
cargo make full-check

# Verify everything
cargo make release-check

# Self-test the framework
cargo make self-test
```

### Before Release

```bash
# 1. Complete validation
cargo make full-check

# 2. Check versions
cargo make version-bump

# 3. Verify publish
cargo make publish-check

# 4. Run comprehensive tests
cargo make test-all
cargo make test-proptest

# 5. Verify documentation
cargo make doc

# 6. Security audit
cargo make audit
```

### Troubleshooting Build Issues

```bash
# Clean and rebuild
cargo make clean
cargo make build

# Deep clean
cargo make clean-all
cargo make build

# Run tests with full output
cargo make test-verbose

# Check specific crate
cargo make build-core
cargo make test-core
```

### Working with OpenTelemetry Features

```bash
# Build with OTEL
cargo make build-otel

# Test with OTEL
cargo make test-otel

# Check OTEL features
cargo make check-otel
```

### Binary Size Optimization

```bash
# Analyze release binary
cargo make bloat

# Find compilation bottlenecks
cargo make bloat-time

# Check dependencies
cargo make deps-duplicates
```

---

## Advanced Usage

### Custom Task Composition

You can run multiple tasks sequentially:

```bash
# Run tasks in order
cargo make clean test build-release install
```

### Environment Variables

```bash
# Run with custom environment
RUST_LOG=debug cargo make test-verbose

# Run with OTEL endpoint configured
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo make build-otel
```

### Platform-Specific Tasks

The Makefile.toml can define platform-specific tasks:

```toml
[tasks.my-task.windows]
script = ["echo Windows-specific"]

[tasks.my-task.linux]
script = ["echo Linux-specific"]

[tasks.my-task.mac]
script = ["echo Mac-specific"]
```

### Extending Makefile.toml

You can extend the existing Makefile.toml with your own tasks:

```toml
# In your local Makefile.toml or as an override
[tasks.my-custom-task]
description = "My custom development task"
dependencies = ["fmt", "clippy"]
script = [
    "echo 'Running custom task'",
    "cargo test my_specific_test"
]
```

### Task Dependencies

Tasks can depend on other tasks:

```toml
[tasks.complex-workflow]
dependencies = [
    "fmt",
    "clippy",
    "test",
    "doc"
]
```

Dependencies run in parallel where possible.

### Conditional Task Execution

```bash
# Run task only if condition is met
cargo make --condition "<condition>" <task>

# Skip task if it's already done
cargo make --skip-tasks-pattern <pattern> <task>
```

---

## Task Reference

### Complete Task List

#### Installation
| Task | Description | Time |
|------|-------------|------|
| `install` | Install clnrm binary | 30s |
| `install-dev` | Install with debug symbols | 45s |
| `uninstall` | Remove clnrm binary | 1s |
| `reinstall` | Uninstall + install | 31s |

#### Building
| Task | Description | Time |
|------|-------------|------|
| `build` | Build workspace | 45s |
| `build-release` | Release build | 2min |
| `build-core` | Build clnrm-core | 30s |
| `build-bin` | Build clnrm binary | 40s |
| `build-otel` | Build with OTEL | 50s |
| `build-ai` | Build AI crate | 25s |
| `build-all` | Build everything | 1min |

#### Testing
| Task | Description | Time |
|------|-------------|------|
| `test` | Library tests | 20s |
| `test-unit` | Unit tests only | 15s |
| `test-integration` | Integration tests | 30s |
| `test-quick` | Fast test run | 10s |
| `test-all` | All tests | 50s |
| `test-proptest` | Property tests | 5min |
| `test-otel` | Tests with OTEL | 25s |

#### Quality
| Task | Description | Time |
|------|-------------|------|
| `clippy` | Lint with strict rules | 30s |
| `clippy-fix` | Auto-fix warnings | 35s |
| `fmt` | Format code | 5s |
| `fmt-check` | Check formatting | 3s |
| `check` | Compilation check | 20s |
| `check-all` | Check all features | 25s |

#### Development
| Task | Description | Time |
|------|-------------|------|
| `dev` | fmt + clippy + test | 1min |
| `quick` | check + test-quick | 30s |
| `watch` | Auto-rebuild | ∞ |
| `clean` | Clean artifacts | 2s |
| `fix` | Auto-fix issues | 40s |

#### Documentation
| Task | Description | Time |
|------|-------------|------|
| `doc` | Build docs | 45s |
| `doc-open` | Build + open docs | 46s |
| `doc-private` | Include private items | 50s |
| `doc-core` | Core docs only | 30s |

#### CI/CD
| Task | Description | Time |
|------|-------------|------|
| `ci` | Full CI pipeline | 2min |
| `pre-commit` | Quick checks | 45s |
| `release-check` | Release validation | 3min |
| `validate` | Complete validation | 4min |
| `full-check` | Everything + bench | 10min |

#### Clnrm-Specific
| Task | Description | Time |
|------|-------------|------|
| `self-test` | Framework self-test | 30s |
| `init-example` | Initialize example | 5s |
| `run-examples` | Run examples | 1min |
| `benchmarks` | Run benchmarks | 2min |

#### Publishing
| Task | Description | Time |
|------|-------------|------|
| `publish-check` | Dry-run publish | 1min |
| `publish` | Publish to crates.io | 5min |
| `version-bump` | Show version info | 1s |

#### Utilities
| Task | Description | Time |
|------|-------------|------|
| `deps` | Dependency tree | 5s |
| `deps-duplicates` | Find dupes | 5s |
| `outdated` | Check updates | 10s |
| `audit` | Security audit | 15s |
| `bloat` | Binary size | 2min |
| `update` | Update deps | 30s |
| `licenses` | List licenses | 5s |

---

## Troubleshooting

### Common Issues

#### 1. cargo-make Not Found

**Problem:**
```bash
cargo make
command not found: cargo-make
```

**Solution:**
```bash
cargo install cargo-make
```

#### 2. Task Fails with "crate not found"

**Problem:**
```bash
cargo make outdated
Error: cargo-outdated not found
```

**Solution:**
```bash
# Install missing tool
cargo install cargo-outdated

# Or install all dev tools
cargo make setup-dev
```

#### 3. Tests Fail Intermittently

**Problem:**
Tests pass sometimes, fail other times (race conditions).

**Solution:**
```bash
# Run tests single-threaded
cargo make test-unit

# Or with output for debugging
cargo make test-verbose
```

#### 4. Build Fails After Git Pull

**Problem:**
Build fails after pulling changes.

**Solution:**
```bash
# Clean and rebuild
cargo make clean
cargo make build

# Or deep clean
cargo make clean-all
cargo make build
```

#### 5. Clippy Has Too Many Warnings

**Problem:**
`cargo make clippy` shows many warnings.

**Solution:**
```bash
# Auto-fix what's possible
cargo make clippy-fix

# Format code
cargo make fmt

# Check remaining issues
cargo make clippy
```

#### 6. Docker Containers Not Cleaning Up

**Problem:**
Tests leave containers running.

**Solution:**
```bash
# Clean up Docker manually
docker ps -a | grep testcontainers | awk '{print $1}' | xargs docker rm -f

# Run tests again
cargo make test
```

#### 7. Out of Disk Space

**Problem:**
Build fails with disk space error.

**Solution:**
```bash
# Clean build artifacts
cargo make clean-all

# Clean Docker
docker system prune -af
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-make
        run: cargo install cargo-make

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run CI pipeline
        run: cargo make ci

      - name: Run self-test
        run: cargo make self-test
```

### GitLab CI

```yaml
stages:
  - test
  - build
  - deploy

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

before_script:
  - cargo install cargo-make
  - export PATH="$CARGO_HOME/bin:$PATH"

test:
  stage: test
  script:
    - cargo make ci
  cache:
    paths:
      - .cargo
      - target

build:
  stage: build
  script:
    - cargo make build-release
  artifacts:
    paths:
      - target/release/clnrm
```

### Jenkins

```groovy
pipeline {
    agent any

    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
        PATH = "${CARGO_HOME}/bin:${PATH}"
    }

    stages {
        stage('Setup') {
            steps {
                sh 'cargo install cargo-make'
            }
        }

        stage('Test') {
            steps {
                sh 'cargo make ci'
            }
        }

        stage('Build') {
            steps {
                sh 'cargo make build-release'
            }
        }

        stage('Self-Test') {
            steps {
                sh 'cargo make self-test'
            }
        }
    }
}
```

### Travis CI

```yaml
language: rust
rust:
  - stable

cache: cargo

before_install:
  - cargo install cargo-make

script:
  - cargo make ci
  - cargo make self-test

before_deploy:
  - cargo make build-release
```

---

## Best Practices

### 1. Use Task Aliases

Create shell aliases for frequently used commands:

```bash
# Add to ~/.bashrc or ~/.zshrc
alias cm='cargo make'
alias cmd='cargo make dev'
alias cmq='cargo make quick'
alias cmt='cargo make test'
alias cmc='cargo make ci'
```

Then use:
```bash
cm dev       # Instead of cargo make dev
cmq          # Quick check
cmt          # Run tests
```

### 2. Integrate with Git Hooks

Use cargo-make in git hooks for automatic checks:

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo make pre-commit
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

### 3. Use Watch Mode During Development

```bash
# Terminal 1: Watch and rebuild
cargo make watch

# Terminal 2: Make changes and see results automatically
```

### 4. Chain Tasks for Complex Workflows

```bash
# Clean, build, test, install in one command
cargo make clean build-release test-all install
```

### 5. Use Environment-Specific Tasks

```bash
# Development
RUST_LOG=debug cargo make dev

# Production testing
RUST_LOG=info cargo make test-all

# With OTEL
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo make build-otel
```

### 6. Create Custom Workflows

Add project-specific tasks to a local `Makefile.local.toml`:

```toml
extend = "Makefile.toml"

[tasks.my-workflow]
description = "My custom workflow"
dependencies = ["clean", "build", "test", "self-test"]
```

### 7. Optimize Build Times

```bash
# Use check instead of build during development
cargo make check        # Fast

# Use quick for rapid iteration
cargo make quick        # Faster

# Use watch for automatic rebuilds
cargo make watch        # Most efficient
```

### 8. Document Your Workflows

Add comments to custom tasks:

```toml
[tasks.my-task]
description = "Clear description of what this does"
# Dependencies explain order
dependencies = ["fmt", "clippy"]
# Script shows what happens
script = [
    "echo 'Step 1: Format'",
    "echo 'Step 2: Lint'",
]
```

### 9. Use Task Dependencies

Instead of:
```bash
cargo make fmt
cargo make clippy
cargo make test
```

Do:
```bash
cargo make dev  # Runs all three
```

### 10. Verify Before Publishing

```bash
# Complete pre-publish checklist
cargo make full-check       # All checks
cargo make publish-check    # Dry-run
cargo make version-bump     # Verify versions
# Update versions manually
cargo make publish          # Publish
```

---

## Additional Resources

### Official Documentation

- [cargo-make GitHub](https://github.com/sagiegurari/cargo-make)
- [cargo-make Documentation](https://sagiegurari.github.io/cargo-make/)
- [Clnrm README](/README.md)
- [Clnrm Testing Guide](/docs/TESTING.md)

### Related Tools

- **cargo-watch**: File watching for auto-rebuild
- **cargo-audit**: Security vulnerability scanning
- **cargo-outdated**: Dependency update checking
- **cargo-bloat**: Binary size analysis
- **cargo-tree**: Dependency tree visualization

### Help and Support

```bash
# Show task help
cargo make --help

# List all tasks
cargo make --list-all-steps

# Show task details
cargo make --print-steps <task-name>

# Show default task (quick reference)
cargo make
```

---

## Summary

### Quick Reference Card

```bash
# Daily Development
cargo make dev           # Format + lint + test
cargo make quick         # Fast check + test
cargo make watch         # Auto-rebuild on changes

# Before Commit
cargo make pre-commit    # Quick validation
cargo make ci            # Full CI pipeline

# Before Release
cargo make release-check # Complete validation
cargo make publish-check # Verify package

# Installation
cargo make install       # Install binary
cargo make self-test     # Verify framework

# Troubleshooting
cargo make clean         # Clean artifacts
cargo make test-verbose  # Debug tests

# Utilities
cargo make audit         # Security check
cargo make outdated      # Update check
cargo make doc-open      # View docs
```

### Task Selection Guide

| Scenario | Task | Why |
|----------|------|-----|
| Active coding | `watch` | Automatic feedback |
| Quick check | `quick` | Fastest validation |
| Before commit | `pre-commit` | Essential checks |
| Before PR | `ci` | Full validation |
| Before release | `full-check` | Everything |
| Binary install | `install` | Production install |
| Debug install | `install-dev` | Development |
| Troubleshooting | `clean` + `build` | Fresh start |
| Documentation | `doc-open` | View API docs |
| Security | `audit` | Vulnerability scan |

---

**Next Steps:**
1. Install cargo-make: `cargo install cargo-make`
2. Try quick workflow: `cargo make dev`
3. Explore tasks: `cargo make --list-all-steps`
4. Set up git hooks (optional)
5. Create aliases (optional)

For questions or issues, refer to the [Clnrm GitHub repository](https://github.com/seanchatmangpt/clnrm) or the cargo-make documentation.
