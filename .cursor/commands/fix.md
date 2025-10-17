# Auto-Fix Issues

Automatically fix formatting and safe clippy issues.

## Commands
```bash
# Format code
cargo make fmt

# Fix clippy issues (safe fixes only)
cargo make clippy-fix

# Both
cargo make fix
```

## What It Does
- **Format all code** (cargo fmt)
- **Apply safe clippy fixes** (cargo clippy --fix)

## Core Team Standards Check
After auto-fix, verify no .unwrap() or .expect() remain:
```bash
cargo make validate-crate
```

## Use When
- After bulk code changes
- Before pre-commit
- When linting fails

## Time: ~10-30 seconds
