# Quality Gates Quick Reference

## Run All Gates Locally (Before Push)

```bash
# Gate 1: TOML Syntax (30 seconds)
./scripts/doc-validation/validate-toml-examples.sh

# Gate 2: Weaver Schema (1-2 minutes)
weaver registry check -r registry/

# Gate 3: Example Tests (2-3 minutes)
cargo test --test toml_examples_validation

# Gate 4: Clippy Zero Warnings (2-3 minutes)
cargo clippy --workspace --all-features -- -D warnings

# Gate 5: Unwrap Detection (10 seconds)
bash .github/workflows/check-unwraps.sh  # See below

# Gate 6: Formatting (30 seconds)
cargo fmt --all -- --check

# Gate 7: Unit Tests (2-3 minutes)
cargo test --lib --workspace

# Gate 8: Build (3-5 minutes)
cargo build --workspace --release --all-features
```

**Total time:** ~10-15 minutes (all gates)

## Quick Pre-Commit Checks (1 minute)

```bash
# Minimum checks before commit
cargo fmt --all
cargo clippy --workspace --all-features -- -D warnings
```

## Fix Common Failures

### Gate 1: TOML Syntax Error
```bash
# Find the error
./scripts/doc-validation/validate-toml-examples.sh

# Fix syntax, then verify
python3 -c "import tomllib; tomllib.load(open('path/to/file.toml', 'rb'))"
```

### Gate 4: Clippy Warning
```bash
# See warnings
cargo clippy --workspace --all-features

# Auto-fix if possible
cargo clippy --workspace --all-features --fix

# Manual fix required for -D warnings
```

### Gate 5: Unwrap Found
```rust
// ❌ WRONG
let result = operation().unwrap();

// ✅ CORRECT
let result = operation()
    .map_err(|e| CleanroomError::internal_error(format!("Failed: {}", e)))?;
```

### Gate 6: Formatting
```bash
# Auto-format
cargo fmt --all

# Verify
cargo fmt --all -- --check
```

## Unwrap Detection Script

Create `.github/workflows/check-unwraps.sh`:
```bash
#!/bin/bash
FOUND_UNWRAPS=0
for file in $(find crates/clnrm-core/src -name "*.rs" -type f | grep -v "/tests/"); do
  TEST_START=$(grep -n "^#\[cfg(test)\]" "$file" | head -1 | cut -d: -f1)
  if [ -n "$TEST_START" ]; then
    UNWRAPS=$(head -n $((TEST_START - 1)) "$file" | grep -n "\.unwrap()\|\.expect(" | grep -v "// OK: " || true)
  else
    UNWRAPS=$(grep -n "\.unwrap()\|\.expect(" "$file" | grep -v "// OK: " || true)
  fi
  if [ -n "$UNWRAPS" ]; then
    echo "❌ $file"
    echo "$UNWRAPS" | sed 's/^/   Line /'
    FOUND_UNWRAPS=1
  fi
done
exit $FOUND_UNWRAPS
```

Make executable:
```bash
chmod +x .github/workflows/check-unwraps.sh
```

## Pre-Commit Hook (Recommended)

Create `.git/hooks/pre-commit`:
```bash
#!/bin/bash
set -e

echo "🔍 Running quality gates..."

# Quick gates only
cargo fmt --all -- --check || {
  echo "❌ Formatting failed. Run: cargo fmt --all"
  exit 1
}

cargo clippy --workspace --all-features -- -D warnings || {
  echo "❌ Clippy failed. Fix warnings before committing."
  exit 1
}

echo "✅ Quality gates passed!"
```

Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

## CI Status Checks

View status in PR:
```
✅ Gate 1: TOML Syntax Validation          PASSED
✅ Gate 2: Weaver Schema Validation        PASSED
✅ Gate 3: Example TOML Tests              PASSED
✅ Gate 4: Clippy (Zero Warnings)          PASSED
✅ Gate 5: No New Unwraps                  PASSED
✅ Gate 6: Code Formatting                 PASSED
✅ Gate 7: Unit Tests                      PASSED
✅ Gate 8: Build with All Features         PASSED
```

## Download Failure Artifacts

If gate fails in CI:
1. Go to failed job
2. Scroll to bottom
3. Download artifacts:
   - `toml-validation-report`
   - `schema-validation-report`
   - `example-test-results`
   - `clippy-report`
   - `unwrap-detection-report`
   - `unit-test-results`

## Bypass Checks (Emergency Only)

**DON'T DO THIS unless absolutely necessary!**

To skip local pre-commit hook:
```bash
git commit --no-verify -m "Emergency fix"
```

**Note:** CI quality gates will still run and block merge.

## Gate Timing

| Gate | Typical Time | Can Cache |
|------|-------------|-----------|
| Gate 1 | 30s | No |
| Gate 2 | 1-2m | Yes (Weaver) |
| Gate 3 | 2-3m | Yes (Cargo) |
| Gate 4 | 2-3m | Yes (Cargo) |
| Gate 5 | 10s | No |
| Gate 6 | 30s | No |
| Gate 7 | 2-3m | Yes (Cargo) |
| Gate 8 | 3-5m | Yes (Cargo) |

**First run:** ~15-20 minutes
**Cached run:** ~5-8 minutes

## Useful Commands

```bash
# Check YAML syntax
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/quality.yml'))"

# Validate single TOML file
python3 -c "import tomllib; tomllib.load(open('path/to/file.toml', 'rb'))"

# Check formatting without fixing
cargo fmt --all -- --check

# Run single test file
cargo test --test toml_examples_validation

# Check for specific clippy lint
cargo clippy -- -W clippy::unwrap_used

# Build single crate
cargo build -p clnrm-core --release

# Run tests with output
cargo test -- --nocapture
```

## Exit Codes

- `0` - All gates passed ✅
- `1` - Gate failure ❌

## Emergency Contacts

- Workflow issues: Check `.github/workflows/QUALITY_GATES.md`
- Gate failures: Check `.github/workflows/QUALITY_IMPLEMENTATION_SUMMARY.md`
- General questions: See main `CLAUDE.md`
