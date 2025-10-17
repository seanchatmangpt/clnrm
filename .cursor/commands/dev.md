# Development Workflow

Quick development iteration: format, lint, and test.

## Command
```bash
cargo make dev
```

## What It Does
- **Format code** (`cargo fmt`)
- **Lint with clippy** (zero warnings enforced)
- **Run quick tests** (single-threaded for consistency)

## Use When
- Before every commit
- During active development
- After making code changes

## Time: ~30 seconds
