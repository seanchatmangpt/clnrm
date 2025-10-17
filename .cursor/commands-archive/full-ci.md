# Full CI - Complete Local Validation

Run the complete CI/CD pipeline locally before pushing to ensure your changes will pass in CI.

## What This Does

Mirrors the complete CI pipeline:
1. **Format** - Check code formatting
2. **Lint** - Run clippy with strict rules
3. **Test Suite** - All tests (unit, integration, doc tests)
4. **Build** - Release build with all features
5. **Security** - Security audit (if tools installed)
6. **Self-Test** - Run clnrm self-tests

## Commands to Run

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy (zero warnings, all features)
cargo clippy --all-targets --all-features -- -D warnings

# 3. All tests
cargo test --all-features

# 4. Integration tests
cargo test --test '*' --all-features

# 5. Doc tests
cargo test --doc --all-features

# 6. Release build
cargo build --release --all-features

# 7. Self-test with OTEL (using Homebrew installation)
clnrm self-test --suite otel --otel-exporter stdout

# 8. Security audit (if installed)
cargo audit || echo "⚠️  cargo-audit not installed"
```

## Expected Duration
- ⏱️ **~3-5 minutes** for complete validation

## Success Criteria
- ✅ All tests pass
- ✅ Zero clippy warnings
- ✅ Code is formatted
- ✅ Release build succeeds
- ✅ Self-tests pass

## When to Use
- Before creating a pull request
- Before pushing to main/master
- After significant changes
- Weekly quality check
