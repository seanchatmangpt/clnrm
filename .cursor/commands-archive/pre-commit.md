# Pre-Commit Validation

Run comprehensive pre-commit checks to ensure code meets clnrm core team standards before committing.

## What This Does

1. **Format Code**
   - Run `cargo fmt` on all workspace crates
   - Ensure consistent code style

2. **Lint Code**
   - Run `cargo clippy` with `-D warnings` (ZERO tolerance)
   - Check all targets and features
   - Auto-fix safe issues

3. **Run Tests**
   - Quick unit test suite
   - Single-threaded for consistency
   - Fail fast on errors

4. **Validate Best Practices**
   - Check for `.unwrap()` in production code (MUST be zero)
   - Check for `.expect()` in production code (MUST be zero)
   - Verify trait methods are sync
   - Check for fake `Ok(())` stubs

## Commands to Execute

```bash
# Full pre-commit workflow
cargo make pre-commit-full

# Or step-by-step:
cargo fmt                          # Format code
cargo clippy -- -D warnings        # Lint (zero warnings)
cargo test --lib -- --test-threads=1  # Quick tests

# Check core standards
grep -r "\.unwrap()" crates/*/src/ | grep -v "test" | grep -v "#\[cfg(test)\]"
grep -r "\.expect(" crates/*/src/ | grep -v "test" | grep -v "#\[cfg(test)\]"
```

## Pre-Commit Checklist

- [ ] Code formatted with `cargo fmt`
- [ ] Clippy passes with zero warnings
- [ ] Unit tests pass
- [ ] No `.unwrap()` in production code
- [ ] No `.expect()` in production code
- [ ] All traits remain `dyn` compatible
- [ ] Proper error handling throughout
- [ ] Tests follow AAA pattern
- [ ] Descriptive test names

## Auto-Fix Available

```bash
# Auto-fix formatting
cargo fmt

# Auto-fix clippy issues (safe fixes only)
cargo clippy --fix --allow-dirty --allow-staged
```

## Expected Output

```
✅ Code formatted
✅ Clippy clean (0 warnings)
✅ Tests passed (X tests)
✅ Core standards met
✅ Ready to commit
```

## Failure Actions

If any check fails:

1. **Format failures** - Run `cargo fmt` automatically
2. **Clippy warnings** - Review and fix or run `cargo clippy --fix`
3. **Test failures** - Debug failing test
4. **Standard violations** - Fix `.unwrap()` and `.expect()` manually

## Time Estimate

- Fast path: ~30 seconds (if everything passes)
- Full path: ~2-3 minutes (if fixes needed)

## Integration with Git Hooks

To run automatically on git commit:

```bash
# Create pre-commit hook
echo '#!/bin/bash
cargo make pre-commit-full || exit 1
' > .git/hooks/pre-commit

chmod +x .git/hooks/pre-commit
```

## Quick Development Iteration

For rapid development:

```bash
# Quick check + format + test
cargo make dev
```
