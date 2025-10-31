# Quick Validation Guide - v1.1.0

**For Developers:** Fast validation workflow
**Time:** 5-10 minutes total
**Script:** `/scripts/validate_v1_1_0_release.sh`

---

## 🚀 One-Command Validation

```bash
# Run complete validation suite
./scripts/validate_v1_1_0_release.sh

# Exit code 0 = Release ready ✅
# Exit code 1 = Needs fixes ❌
```

---

## 📋 Manual Quick Check (2 minutes)

### 1. Compilation (30 seconds)
```bash
cargo build --release --features otel
# ✅ Must succeed with < 10 warnings
```

### 2. Unit Tests (30 seconds)
```bash
cargo test --lib
# ✅ Must show: "test result: ok"
```

### 3. Self-Test (30 seconds)
```bash
clnrm self-test
# ✅ Must show: "All tests passed"
```

### 4. Version Check (10 seconds)
```bash
clnrm --version
# ✅ Must show: "clnrm 1.1.0"
```

### 5. README Validation (30 seconds)
```bash
cargo test --test readme_validation_complete
# ✅ Must show: "49 passed; 0 failed"
```

---

## 🔍 Troubleshooting

### If Compilation Fails
```bash
# Check specific errors
cargo build 2>&1 | grep "error\[E"

# Common fix: Update dependencies
cargo update

# Nuclear option: Clean rebuild
cargo clean && cargo build --release
```

### If Tests Fail
```bash
# Run specific test
cargo test --lib test_name_here

# Verbose output
cargo test -- --nocapture

# Show backtrace
RUST_BACKTRACE=1 cargo test
```

### If Self-Test Fails
```bash
# Check installation
which clnrm

# Reinstall
sudo cp target/release/clnrm /usr/local/bin/clnrm

# Run specific suite
clnrm self-test --suite framework
```

---

## ✅ Release Readiness Criteria

**MUST PASS:**
- ✅ Compilation succeeds
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Self-test passes
- ✅ README validation passes

**Good to Have:**
- ✅ Example configurations valid
- ✅ No clippy warnings
- ✅ Documentation updated

---

## 🎯 Fast Validation Workflow

```bash
# 1. Quick compile check (30s)
cargo check --release --features otel

# 2. Fast test subset (1m)
cargo test --lib -- --test-threads=8

# 3. Critical integration tests (2m)
cargo test --test integration_self_test_otel

# 4. Self-test validation (1m)
clnrm self-test --suite framework --suite cli

# 5. README sanity check (30s)
cargo test --test readme_validation_complete -- --test-threads=1

# Total: ~5 minutes
```

---

## 📊 Validation Status

Check current status:
```bash
# Overall
./scripts/validate_v1_1_0_release.sh

# Just compilation
cargo build --release --features otel && echo "✓ Build OK"

# Just tests
cargo test && echo "✓ Tests OK"

# Just self-test
clnrm self-test && echo "✓ Self-test OK"
```

---

## 🐛 Known Issues & Quick Fixes

### Issue: clnrm-template dependency
```bash
# Quick fix: Comment out in Cargo.toml
# File: crates/clnrm-core/Cargo.toml:73
# clnrm-template = { path = "../clnrm-template", optional = true }
```

### Issue: README false positives
```bash
# Fix: Remove contradictory lines
sed -i '' '440d' README.md  # Remove unimplemented claim
sed -i '' 's/v0\.[4-7]\.0/v1.1.0/g' README.md  # Fix version
```

### Issue: Binary not in PATH
```bash
sudo cp target/release/clnrm /usr/local/bin/clnrm
```

---

## 📈 CI/CD Integration

### GitHub Actions
```yaml
- name: Validate Release
  run: |
    ./scripts/validate_v1_1_0_release.sh
    if [ $? -ne 0 ]; then
      echo "Validation failed"
      exit 1
    fi
```

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit
cargo test --lib --quiet
```

---

## 🎓 Learning Resources

- **Full Plan:** `docs/validation/V1_1_0_VALIDATION_PLAN.md`
- **Validation Results:** `docs/validation/CLNRM_VALIDATION_RESULTS.md`
- **Discrepancies:** `docs/validation/CLNRM_DISCREPANCIES.md`

---

**Quick Reference Card:**
```
┌──────────────────────────────────────────┐
│ v1.1.0 VALIDATION QUICK REFERENCE        │
├──────────────────────────────────────────┤
│ 1. ./scripts/validate_v1_1_0_release.sh  │
│    → Complete validation                  │
│                                           │
│ 2. cargo build --release --features otel │
│    → Must succeed                         │
│                                           │
│ 3. cargo test                             │
│    → All tests pass                       │
│                                           │
│ 4. clnrm self-test                        │
│    → Framework validates itself           │
│                                           │
│ 5. Check version: clnrm --version         │
│    → Should show 1.1.0                    │
└──────────────────────────────────────────┘
```

---

**Status:** Ready to use
**Last Updated:** 2025-10-30
