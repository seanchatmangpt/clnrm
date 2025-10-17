# Cargo Make Documentation for Clnrm

This directory contains comprehensive documentation for using `cargo-make` with the Clnrm testing framework.

## Available Documentation

### 1. [CARGO_MAKE_GUIDE.md](CARGO_MAKE_GUIDE.md)
**Complete reference guide** (1,200+ lines)

Comprehensive documentation covering:
- Installation and setup
- All 50+ task categories
- Common workflows and patterns
- Advanced usage and customization
- CI/CD integration examples
- Troubleshooting guide
- Best practices

**Read this when:** You need detailed information about specific tasks or workflows.

### 2. [CARGO_MAKE_QUICK_REFERENCE.md](CARGO_MAKE_QUICK_REFERENCE.md)
**Quick reference card** (250 lines)

One-page quick reference with:
- Most common commands
- Task timing estimates
- Workflow patterns
- Shell aliases
- Task selection guide

**Read this when:** You need quick lookup of commands during development.

### 3. [../Makefile.toml](../Makefile.toml)
**Task definitions** (460+ lines)

The actual cargo-make configuration file defining all tasks.

**Read this when:** You want to understand task implementation or add custom tasks.

## Quick Start

```bash
# Install cargo-make
cargo install cargo-make

# Show available tasks
cargo make

# Run quick development cycle
cargo make dev

# For more help
cargo make --list-all-steps
```

## Documentation Structure

```
docs/
├── CARGO_MAKE_GUIDE.md              # Complete reference (23KB)
├── CARGO_MAKE_QUICK_REFERENCE.md    # Quick reference (6KB)
└── CARGO_MAKE_README.md             # This file

Makefile.toml                         # Task definitions (root)
```

## Common Use Cases

### I'm New to Clnrm
Start with: `CARGO_MAKE_GUIDE.md` → "Getting Started" section

### I Need a Quick Command
Start with: `CARGO_MAKE_QUICK_REFERENCE.md` → "Most Common Commands"

### I Want to Add Custom Tasks
Start with: `CARGO_MAKE_GUIDE.md` → "Advanced Usage" section

### I'm Setting Up CI/CD
Start with: `CARGO_MAKE_GUIDE.md` → "CI/CD Integration" section

### I Have Build Issues
Start with: `CARGO_MAKE_GUIDE.md` → "Troubleshooting" section

## Most Useful Commands

```bash
# Daily development
cargo make dev              # Format + lint + test
cargo make watch            # Auto-rebuild on changes

# Before committing
cargo make pre-commit       # Quick validation
cargo make ci               # Full CI pipeline

# Installation
cargo make install          # Install clnrm binary
cargo make self-test        # Verify framework

# Troubleshooting
cargo make clean            # Clean artifacts
cargo make test-verbose     # Debug tests
```

## Task Categories Overview

The Makefile defines 50+ tasks organized into:

1. **Installation** (4 tasks): Install, uninstall, reinstall clnrm binary
2. **Building** (7 tasks): Build workspace, release builds, OTEL builds
3. **Testing** (8 tasks): Unit, integration, property-based tests
4. **Quality** (7 tasks): Clippy, formatting, compilation checks
5. **Development** (6 tasks): Quick workflows, watch mode, cleaning
6. **Documentation** (4 tasks): Build and view API documentation
7. **CI/CD** (4 tasks): Pre-commit, CI pipeline, release validation
8. **Clnrm-Specific** (4 tasks): Self-test, examples, benchmarks
9. **Publishing** (3 tasks): Dry-run, version bump, publish
10. **Utilities** (7 tasks): Audit, outdated, bloat analysis
11. **Composite** (3 tasks): Multi-task workflows

## Time Estimates

| Duration | Example Tasks |
|----------|---------------|
| < 10s | fmt, clean, deps |
| 10-30s | test, check, clippy, install |
| 30s-1min | build, pre-commit |
| 1-2min | dev, build-release, ci |
| 2-5min | benchmarks, release-check |
| 5-10min | test-proptest, full-check |

## Integration Points

### Git Hooks
See: `CARGO_MAKE_GUIDE.md` → "Best Practices" → "Integrate with Git Hooks"

### GitHub Actions
See: `CARGO_MAKE_GUIDE.md` → "CI/CD Integration" → "GitHub Actions"

### Shell Aliases
See: `CARGO_MAKE_QUICK_REFERENCE.md` → "Shell Aliases"

### Custom Tasks
See: `CARGO_MAKE_GUIDE.md` → "Advanced Usage" → "Extending Makefile.toml"

## Support and Resources

- **Full Guide**: [CARGO_MAKE_GUIDE.md](CARGO_MAKE_GUIDE.md)
- **Quick Reference**: [CARGO_MAKE_QUICK_REFERENCE.md](CARGO_MAKE_QUICK_REFERENCE.md)
- **Clnrm Docs**: [README.md](../README.md)
- **cargo-make Docs**: https://sagiegurari.github.io/cargo-make/
- **Issues**: https://github.com/seanchatmangpt/clnrm/issues

## Contributing

When adding new tasks to `Makefile.toml`:

1. Add task definition with clear description
2. Update this documentation
3. Test the task thoroughly
4. Update task counts in this file
5. Consider platform compatibility

## Version History

- **v1.0.0** (2025-10-17): Initial comprehensive documentation
  - Complete guide (1,200+ lines)
  - Quick reference card (250 lines)
  - 50+ documented tasks
  - 10 task categories
  - CI/CD examples
  - Troubleshooting guide

---

**Start here:** Run `cargo make` to see available tasks, then refer to the guide for details.
