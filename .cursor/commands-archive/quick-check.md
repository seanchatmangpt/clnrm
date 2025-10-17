# Quick Check - Fast Local Validation

Run a fast validation loop for quick feedback during development.

## What This Does

Performs essential quality checks without running the full test suite:
1. **Format check** - Verify code formatting with rustfmt
2. **Lint check** - Run clippy with zero-warning policy
3. **Quick tests** - Run unit tests only (skip integration tests)
4. **Build check** - Verify compilation with all features

## Commands to Run

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy (zero warnings)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Unit tests only
cargo test --lib --all-features

# 4. Build check
cargo build --all-features
```

## Expected Duration
- ⏱️ **~30-60 seconds** for quick iteration

## When to Use
- Before committing changes
- During active development
- When you want fast feedback

## Next Steps
If quick-check passes, consider running `/full-ci` before pushing.
