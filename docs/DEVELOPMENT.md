# gVisor Development Guide

Setup your development environment and start contributing to clnrm with gVisor.

**Target Audience**: Developers, contributors
**Time Required**: 30-45 minutes
**Prerequisites**: Linux system with gVisor installed (see [SETUP.md](SETUP.md))

## Table of Contents

1. [Development Environment](#development-environment)
2. [Building from Source](#building-from-source)
3. [Running Tests](#running-tests)
4. [Debugging](#debugging)
5. [Contributing](#contributing)
6. [Code Standards](#code-standards)
7. [Performance Profiling](#performance-profiling)

---

## Development Environment

### Prerequisites

Ensure you have installed:

```bash
# gVisor (required)
runsc --version

# Rust toolchain (1.70+)
rustc --version
cargo --version

# Git
git --version

# Build dependencies
sudo apt-get install -y build-essential pkg-config libssl-dev
```

### Clone Repository

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm

# Verify directory structure
ls -la
# Expected:
# crates/          - Rust crates (core, cli, etc.)
# docs/            - Documentation
# tests/           - Test suites
# examples/        - Example code
# Cargo.toml       - Workspace configuration
```

### Configure IDE

#### VS Code (Recommended)

1. Install Rust Analyzer extension:
```
Extension: rust-analyzer
Publisher: The Rust Programming Language
ID: rust-lang.rust-analyzer
```

2. Create `.vscode/settings.json`:
```json
{
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
      "source.fixAll.clippy": true
    }
  },
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

3. Create `.vscode/launch.json` for debugging:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Test",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "test",
          "--lib",
          "--",
          "--nocapture"
        ],
        "filter": {
          "name": "specific_test",
          "kind": "test"
        }
      },
      "sourceLanguages": ["rust"]
    }
  ]
}
```

#### Other IDEs

- **IntelliJ IDEA**: Install "Rust" plugin by JetBrains
- **Vim**: Use rust.vim with LSP client (coc-rust-analyzer)
- **Emacs**: Use rustic-mode with eglot

### Development Environment Setup

Create `.env.development`:

```bash
# Enable debug logging
export CLNRM_DEBUG=true

# Use gVisor backend
export CLNRM_BACKEND=gvisor

# Increase timeouts for development
export CLNRM_STARTUP_TIMEOUT=60
export CLNRM_EXECUTION_TIMEOUT=600

# Optional: Cache directory
export CLNRM_CACHE_DIR=./target/clnrm-cache

# Rust backtrace for debugging
export RUST_BACKTRACE=1

# Run in development mode
export RUST_LOG=debug
```

Load environment:
```bash
source .env.development
```

---

## Building from Source

### Clean Build

```bash
# Full rebuild of all crates
cargo clean
cargo build --all

# With release optimizations
cargo build --all --release

# Build specific crate
cargo build -p clnrm-core
cargo build -p clnrm-cli
```

### Incremental Build

```bash
# Quick debug build (after source changes)
cargo build

# Watch for changes (requires cargo-watch)
cargo install cargo-watch
cargo watch -x build
```

### Build with Features

```bash
# Build with all features
cargo build --all-features

# Build with specific feature
cargo build --features gvisor

# Build without default features
cargo build --no-default-features
```

### Check Compilation Without Building

```bash
# Quick compilation check
cargo check --all

# Check with clippy (linter)
cargo clippy --all
```

---

## Running Tests

### Test Suites

The project has three types of tests:

1. **Unit Tests**: Logic verification (run in-process, fast)
2. **Integration Tests**: Component interaction (use gVisor, slower)
3. **End-to-End Tests**: Full workflow validation (use gVisor, slowest)

### Running All Tests

```bash
# Run all tests
cargo test --all

# With output
cargo test --all -- --nocapture

# Specific test
cargo test test_name

# Matching pattern
cargo test test_pattern -- --nocapture
```

### Running Specific Test Suites

```bash
# Unit tests only (fast, no gVisor needed)
cargo test --lib

# Integration tests (requires gVisor)
cargo test --test '*'

# E2E tests (full setup required)
cargo test --test e2e_*

# Doc tests
cargo test --doc
```

### Parallel Test Execution

```bash
# Run tests with specific thread count
cargo test --all -- --test-threads=4

# Sequential execution (use for debugging)
cargo test --all -- --test-threads=1

# Maximum parallelism
cargo test --all -- --test-threads=$(nproc)
```

### Test with Debug Output

```bash
# Show println! output from tests
cargo test --all -- --nocapture --test-threads=1

# With Rust backtrace
RUST_BACKTRACE=1 cargo test --all -- --nocapture

# Full debug logging
CLNRM_DEBUG=true RUST_LOG=debug cargo test --all -- --nocapture
```

### Test a Single Function

```bash
# Test specific function
cargo test my_function -- --nocapture --exact

# Example
cargo test test_gvisor_hello -- --nocapture --exact
```

---

## Debugging

### Print-Based Debugging

```rust
// Add debug output in tests
#[test]
fn test_example() {
    eprintln!("Debug output"); // Shows with --nocapture
    dbg!(variable);              // Prints and returns value
    println!("Info");             // Shows with --nocapture
}
```

### Using a Debugger

#### With lldb (macOS/Linux)

```bash
# Build debug binary
cargo build

# Debug test
lldb target/debug/test_name

# In lldb:
# (lldb) run --test-threads=1
# (lldb) breakpoint set -n test_function
# (lldb) continue
```

#### With gdb (Linux)

```bash
# Build debug binary
cargo build

# Debug test
gdb --args target/debug/test_name --test-threads=1

# In gdb:
# (gdb) break test_function
# (gdb) run
# (gdb) next
# (gdb) print variable
```

#### With VS Code Debugger

1. Install CodeLLDB extension
2. Create `.vscode/launch.json` (see IDE setup above)
3. Set breakpoints in code
4. Press F5 to start debugging

### Logging and Tracing

Enable detailed logging:

```bash
# Trace all debug messages
RUST_LOG=debug cargo test my_test -- --nocapture

# Specific module
RUST_LOG=clnrm_core::backend=debug cargo test

# Multiple modules
RUST_LOG=clnrm_core=debug,clnrm_cli=info cargo test

# File output
RUST_LOG=debug cargo test 2>&1 | tee test-output.log
```

### Common Debugging Patterns

**Checking gVisor Status**:
```bash
# View gVisor runtime status
sudo journalctl -u runsc -f

# Check container state
sudo runsc --root /var/run/runsc list

# Debug specific container
sudo runsc --root /var/run/runsc state CONTAINER_ID
```

**Inspecting Test Artifacts**:
```bash
# Find test artifacts
find ./target -type d -name "clnrm-*" -newer .git

# View container bundle
ls -la target/clnrm-bundles/*/

# Check image cache
ls -la /var/cache/clnrm/
```

**Memory Debugging**:
```bash
# Run with memory profiling
VALGRIND_OPTS="--leak-check=full" cargo test

# Or use Rust sanitizers
RUSTFLAGS="-Z sanitizer=memory" cargo test
```

---

## Contributing

### Code Review Checklist

Before submitting a PR, ensure:

- [ ] Code follows [Code Standards](#code-standards)
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy --all`
- [ ] Code formatted: `cargo fmt --all`
- [ ] Documentation updated for public APIs
- [ ] New tests added for new functionality
- [ ] Commit messages follow conventional commits

### Workflow

```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Make changes
# ... edit files ...

# 3. Run tests
cargo test --all

# 4. Format code
cargo fmt --all

# 5. Run clippy
cargo clippy --all

# 6. Commit changes
git add -A
git commit -m "feat: add new feature"

# 7. Push and create PR
git push origin feature/my-feature
```

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

Types:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `style` - Formatting (no logic change)
- `refactor` - Code reorganization (no feature change)
- `test` - Test additions
- `chore` - Build/tooling changes

Examples:
```bash
git commit -m "feat: add gVisor backend support"
git commit -m "fix: handle network timeout in service startup"
git commit -m "docs: update setup guide for gVisor"
```

---

## Code Standards

### Error Handling

Never use `unwrap()` or `expect()` in production code:

```rust
// Bad
let value = result.unwrap();

// Good
let value = result.map_err(|e| CleanroomError::new(e))?;
```

### Testing

Maintain 80%+ code coverage:

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage

# View report
open target/coverage/index.html
```

### Documentation

Document all public APIs:

```rust
/// Runs a container command and returns the result.
///
/// # Arguments
/// * `cmd` - Command to execute
/// * `env` - Environment variables (optional)
///
/// # Returns
/// Result with command output or error
///
/// # Examples
/// ```
/// let result = backend.run_cmd(Cmd::new("echo").arg("hello"))?;
/// assert_eq!(result.exit_code, 0);
/// ```
pub fn run_cmd(&self, cmd: Cmd) -> Result<RunResult>;
```

### Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Format specific file
cargo fmt -- path/to/file.rs
```

### Linting

```bash
# Run clippy checks
cargo clippy --all

# Fix some issues automatically
cargo clippy --fix --allow-dirty --all

# Check specific crate
cargo clippy -p clnrm-core
```

---

## Performance Profiling

### Benchmarking

Create benchmark in `benches/`:

```rust
// benches/startup_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_container_startup(c: &mut Criterion) {
    c.bench_function("cold_start", |b| {
        b.iter(|| {
            let backend = GVisorBackend::new(black_box("alpine:latest")).unwrap();
            backend.run_cmd(black_box(Cmd::new("echo").arg("hello"))).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_container_startup);
criterion_main!(benches);
```

Run benchmarks:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench benchmark_container_startup

# Save baseline
cargo bench -- --save-baseline my-baseline

# Compare to baseline
cargo bench -- --baseline my-baseline
```

### Memory Profiling

```bash
# Using valgrind
valgrind --tool=massif cargo test --lib

# View results
ms_print massif.out.* > memory-profile.txt

# Using heaptrack
heaptrack cargo test --lib
heaptrack_gui heaptrack.cargo.*.gz
```

### CPU Profiling

```bash
# Install perf tool
sudo apt-get install -y linux-tools-generic

# Profile test execution
perf record --call-graph=dwarf cargo test my_test

# View results
perf report

# Flame graph
cargo install flamegraph
cargo flamegraph --test my_test
```

---

## Troubleshooting Development Issues

### Issue: Tests Timeout

**Symptom**: Tests take too long or hang

**Solutions**:
```bash
# Increase timeout
export CLNRM_STARTUP_TIMEOUT=120

# Run specific test with debug output
CLNRM_DEBUG=true cargo test my_test -- --nocapture --test-threads=1

# Check if gVisor is responsive
runsc --version

# Kill stuck containers
sudo runsc --root /var/run/runsc delete -force $(sudo runsc --root /var/run/runsc list -quiet)
```

### Issue: "Permission Denied" Errors

**Symptom**: Tests fail with permission errors

**Solutions**:
```bash
# Run with sudo
sudo cargo test --all

# Or configure sudo to allow without password:
sudo visudo
# Add: username ALL=(ALL) NOPASSWD: /usr/bin/runsc

# Run with environment preservation
sudo -E cargo test --all
```

### Issue: Out of Memory

**Symptom**: Tests OOM killed or system becomes unresponsive

**Solutions**:
```bash
# Reduce parallel test execution
cargo test --all -- --test-threads=2

# Reduce resource limits
export CLNRM_MEMORY_LIMIT_MB=256

# Monitor system resources
watch -n 1 free -h

# Kill excess containers
sudo runsc --root /var/run/runsc delete -force $(sudo runsc --root /var/run/runsc list -quiet)
```

### Issue: "Image Pull Failures"

**Symptom**: Tests fail when pulling images

**Solutions**:
```bash
# Check network connectivity
ping docker.io

# Pre-pull image
skopeo copy docker://alpine:latest oci:///var/cache/clnrm/alpine:latest

# Increase timeout
export CLNRM_STARTUP_TIMEOUT=120

# Check registry auth
cat ~/.docker/config.json
```

---

## Next Steps

Now that your development environment is set up:

1. **First Test**: Run `cargo test --lib` to verify setup
2. **Read Code**: Explore `crates/clnrm-core/src/backend/gvisor.rs`
3. **Write Test**: Create a simple test in `tests/`
4. **Submit PR**: Make your first contribution!

---

## Additional Resources

- **gVisor Documentation**: https://gvisor.dev/docs
- **OCI Spec**: https://github.com/opencontainers/runtime-spec
- **Rust Book**: https://doc.rust-lang.org/book/
- **Clnrm Architecture**: See [GVISOR_ARCHITECTURE_DIAGRAMS.md](GVISOR_ARCHITECTURE_DIAGRAMS.md)

---

**Happy coding!** For questions, open an issue or start a discussion on GitHub.
