# Quick Validation Guide

## 1-Minute Validation

Verify clnrm v1.3.0 is production-ready:

```bash
# 1. Validate registry schemas (SOURCE OF TRUTH)
weaver registry check -r registry/
# Expected: ✔ 195 files, zero violations

# 2. Build with OTEL features
cargo build --release --features otel
# Expected: Finished in ~40s

# 3. Test Weaver integration
./target/release/clnrm live-check test-weaver
# Expected: ✓ All checks pass

# 4. Run self-tests
./target/release/clnrm self-test --suite container --otel-exporter stdout
# Expected: ✅ ALL PASSED

# 5. Verify zero-sample detection (CRITICAL!)
CLNRM_REGISTRY_PATH=./registry ./target/release/clnrm run tests/weaver-validation-test.clnrm.toml --live-check --otel-exporter stdout
# Expected: ❌ Fails with "Zero telemetry samples" (THIS IS CORRECT!)
```

## What "PASSED" Means

### ✅ Weaver Registry Check
- 195 schema files loaded
- Zero policy violations
- Complete semantic conventions

### ✅ Build Success
- Compiles with OTEL features
- 31MB optimized binary
- Zero errors

### ✅ Weaver Integration
- Weaver 0.16.1 detected
- All commands available
- Registry valid

### ✅ Self-Tests
- Container tests pass
- OTEL telemetry emitted
- Zero failures

### ✅ Zero-Sample Detection
- **MUST FAIL with "zero samples"**
- This proves false positive prevention
- Validates core value proposition

## Quick Smoke Test

```bash
# One-liner to verify everything
weaver registry check -r registry/ && \
  cargo build --release --features otel && \
  ./target/release/clnrm live-check test-weaver && \
  ./target/release/clnrm self-test --suite container --otel-exporter stdout
```

If all pass: **v1.3.0 is production-ready** ✅

## Full Validation (20 minutes)

Run the complete validation suite:

```bash
# 1. Registry validation
weaver registry check -r registry/

# 2. Build
cargo build --release --features otel

# 3. CLI commands
./target/release/clnrm live-check test-weaver
./target/release/clnrm live-check version
./target/release/clnrm live-check modes
./target/release/clnrm live-check validate-registry --registry ./registry

# 4. Self-tests (all suites)
./target/release/clnrm self-test --suite framework --otel-exporter stdout
./target/release/clnrm self-test --suite container --otel-exporter stdout
./target/release/clnrm self-test --suite otel --otel-exporter stdout

# 5. Live-check validation (expects failure - proves zero-sample detection)
CLNRM_REGISTRY_PATH=./registry ./target/release/clnrm run tests/weaver-validation-test.clnrm.toml --live-check --validation-mode 80_20 --otel-exporter stdout

# 6. OTLP export (if collector available)
CLNRM_REGISTRY_PATH=./registry ./target/release/clnrm run tests/weaver-validation-test.clnrm.toml --live-check --validation-mode 80_20 --otel-exporter otlp-grpc --otel-endpoint http://localhost:4317
```

## Validation Checklist

- [ ] Registry check passes (195 files)
- [ ] Build succeeds with OTEL features
- [ ] Weaver test passes
- [ ] Self-tests pass
- [ ] Zero-sample detection works (fails correctly)
- [ ] CLI commands all functional
- [ ] Validation modes documented
- [ ] Performance acceptable

## Expected Results

### Success Case (Registry Check)
```
✔ `clnrm` semconv registry loaded (195 files)
✔ No `before_resolution` policy violation
✔ `clnrm` semconv registry resolved
✔ No `after_resolution` policy violation
```

### Success Case (Self-Test)
```
Suite: container (1 tests)... ✅ PASS
Total: 3 tests, 3 passed, 0 failed
Overall: ✅ ALL PASSED
```

### Success Case (Zero-Sample Detection)
```
❌ VALIDATION FAILED: Zero telemetry samples received
Cannot validate telemetry that was never sent.
```
**This failure IS success!** It proves false positive prevention.

## Troubleshooting

### Registry Not Found
```bash
export CLNRM_REGISTRY_PATH=/path/to/clnrm/registry
```

### Weaver Not Installed
```bash
cargo install weaver-cli
weaver --version
```

### Build Fails
```bash
# Check Rust version
rustc --version  # Need 1.70+

# Clean and rebuild
cargo clean
cargo build --release --features otel
```

### Docker Not Available
```bash
# Start Docker
docker ps

# Pull images
docker pull alpine:latest
docker pull ubuntu:22.04
```

## Production Deployment

### Set Registry Path
```bash
export CLNRM_REGISTRY_PATH=/usr/local/share/clnrm/registry
```

### Install from Source
```bash
cargo install --path crates/clnrm --features otel
```

### Run with Validation
```bash
clnrm run tests/ \
  --live-check \
  --validation-mode 80_20 \
  --otel-exporter otlp-http \
  --otel-endpoint http://otel-collector:4318
```

---

**Key Takeaway:** If Weaver registry validation passes, v1.3.0 is production-ready. Everything else is supporting evidence.
