# Makefile.toml Guide

This guide explains how to use the `Makefile.toml` for clnrm development.

## Prerequisites

Install cargo-make:

```bash
cargo install cargo-make
```

## Quick Start

### Most Common Tasks

```bash
# Quick development workflow (format + clippy + test)
cargo make dev

# Fast check without full compilation
cargo make quick

# Full CI pipeline
cargo make ci

# Install clnrm binary
cargo make install
```

### Viewing Available Tasks

```bash
# Show help with common tasks
cargo make

# List all available tasks
cargo make --list-all-steps

# Get info about a specific task
cargo make --print-steps <task-name>
```

## Task Categories

### 1. Installation Tasks

```bash
cargo make install          # Install clnrm to ~/.cargo/bin
cargo make install-dev      # Install with debug symbols
cargo make uninstall        # Remove clnrm binary
cargo make reinstall        # Uninstall and reinstall
```

### 2. Build Tasks

```bash
cargo make build            # Build all workspace crates (excludes AI)
cargo make build-release    # Release build with all features
cargo make build-core       # Build only clnrm-core
cargo make build-bin        # Build only clnrm binary
cargo make build-otel       # Build with OpenTelemetry features
cargo make build-ai         # Build experimental AI crate
cargo make build-all        # Build everything including AI
```

### 3. Test Tasks

```bash
cargo make test             # Run library tests
cargo make test-unit        # Unit tests only (single-threaded)
cargo make test-integration # Integration tests only
cargo make test-quick       # Fast test run with output
cargo make test-all         # All tests (unit + integration)
cargo make test-core        # Test clnrm-core only
cargo make test-verbose     # Tests with full output
cargo make test-proptest    # Property-based tests
cargo make test-otel        # Tests with OTEL features
```

### 4. Quality Tasks

```bash
cargo make clippy           # Lint with strict warnings
cargo make clippy-fix       # Auto-fix clippy issues
cargo make fmt              # Format code
cargo make fmt-check        # Check formatting
cargo make check            # Fast compilation check
cargo make check-all        # Check all features
cargo make fix              # Auto-fix format + clippy
```

### 5. Development Tasks

```bash
cargo make dev              # Quick dev loop (fmt + clippy + test)
cargo make quick            # Fast iteration (check + test-quick)
cargo make watch            # Watch for changes and rebuild
cargo make watch-test       # Watch and run tests
cargo make clean            # Clean build artifacts
cargo make clean-all        # Deep clean including target/
```

### 6. Documentation Tasks

```bash
cargo make doc              # Build documentation
cargo make doc-open         # Build and open docs in browser
cargo make doc-private      # Include private items
cargo make doc-core         # Document clnrm-core only
```

### 7. CI/CD Tasks

```bash
cargo make ci               # Full CI pipeline
cargo make pre-commit       # Quick pre-commit checks
cargo make release-check    # Verify release readiness
cargo make validate         # Full validation suite
```

### 8. Clnrm-Specific Tasks

```bash
cargo make self-test        # Run clnrm self-test
cargo make init-example     # Initialize example project
cargo make run-examples     # Run case-study examples
cargo make benchmarks       # Run benchmarks
cargo make bench-hot-reload # Hot reload benchmark only
```

### 9. Publishing Tasks

```bash
cargo make publish-check    # Dry-run publish verification
cargo make publish          # Publish to crates.io (CAUTION!)
cargo make version-bump     # Show version bump instructions
```

### 10. Utility Tasks

```bash
cargo make deps             # Show dependency tree
cargo make deps-duplicates  # Find duplicate dependencies
cargo make outdated         # Check for outdated deps
cargo make audit            # Security audit
cargo make bloat            # Analyze binary size
cargo make bloat-time       # Analyze compile time
cargo make update           # Update dependencies
cargo make licenses         # List dependency licenses
```

## Composite Workflows

These tasks run multiple operations in sequence:

```bash
cargo make full-check       # CI + benchmarks + docs + audit
cargo make setup-dev        # Install all development tools
```

## Tips & Best Practices

### Daily Development Workflow

1. **Start of day**: `cargo make check` (fast validation)
2. **During development**: `cargo make quick` (fast iteration)
3. **Before commit**: `cargo make pre-commit` (format + clippy + test)
4. **Before push**: `cargo make ci` (full validation)

### Watch Mode for Continuous Testing

```bash
cargo make watch-test
```

This automatically runs tests whenever files change.

### Working with Specific Crates

```bash
# Build only core library
cargo make build-core

# Test only core library
cargo make test-core

# Document only core library
cargo make doc-core
```

### Handling the AI Crate

The `clnrm-ai` crate is experimental and excluded from default builds:

```bash
# Build AI crate explicitly
cargo make build-ai

# Build everything including AI
cargo make build-all
```

### Installing Development Tools

```bash
cargo make setup-dev
```

This installs:
- cargo-watch
- cargo-tree
- cargo-outdated
- cargo-audit
- cargo-bloat
- cargo-license

## Troubleshooting

### Task Not Found Error

If you get "Task not found", make sure you're in the repository root:

```bash
cd /path/to/clnrm
cargo make <task>
```

### Workspace Mode Issues

The Makefile.toml is configured to run tasks at the workspace root by default. If you need to run tasks in a specific crate directory:

```bash
cd crates/clnrm-core
cargo build  # Use cargo directly
```

### Build Failures

If builds fail with compilation errors:

```bash
# Clean everything and rebuild
cargo make clean-all
cargo make build

# Or use cargo directly
cargo clean
cargo build
```

### Test Failures

If tests fail:

```bash
# Run with output to see details
cargo make test-verbose

# Run specific test
cargo test <test_name>

# Run tests in specific crate
cargo test -p clnrm-core
```

## Advanced Usage

### Custom Environment Variables

Set environment variables before running tasks:

```bash
# Run with specific Rust flags
RUSTFLAGS="-C target-cpu=native" cargo make build-release

# Run with verbose output
CARGO_MAKE_CARGO_VERBOSE_FLAGS="--verbose" cargo make build
```

### Dry Run

See what a task would do without executing:

```bash
cargo make --print-steps <task>
```

### Parallel Execution

Some tasks can be run in parallel:

```bash
# Run multiple independent tasks
cargo make build &
cargo make doc &
wait
```

## Integration with IDEs

### VS Code

Add to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo make dev",
      "type": "shell",
      "command": "cargo make dev",
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo make test",
      "type": "shell",
      "command": "cargo make test",
      "problemMatcher": ["$rustc"]
    }
  ]
}
```

### Git Hooks

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/sh
cargo make pre-commit
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Contributing

When adding new tasks to `Makefile.toml`:

1. Add clear descriptions
2. Group related tasks
3. Use meaningful task names
4. Document dependencies
5. Test the task works
6. Update this guide

## Resources

- [cargo-make Documentation](https://github.com/sagiegurari/cargo-make)
- [clnrm CLAUDE.md](../CLAUDE.md) - Project guidelines
- [CLI Guide](CLI_GUIDE.md) - CLI command reference

---

**Need help?** Run `cargo make` to see the most common tasks.
