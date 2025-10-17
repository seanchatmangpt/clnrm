# Production Validation

Run comprehensive production readiness validation for clnrm, ensuring all core team standards are met before deployment.

## What This Does

1. **Prerequisites Check**
   - Verify Docker daemon is running
   - Check Rust/Cargo installation
   - Validate cargo-make availability

2. **Core Team Standards**
   - ✅ NO `.unwrap()` or `.expect()` in production code (ENFORCED)
   - ✅ All errors handled with `Result<T, CleanroomError>`
   - ✅ Trait methods remain `dyn` compatible (no async)

3. **Test Suite**
   - Unit tests (library tests)
   - Integration tests
   - Cleanroom hermetic tests
   - Property-based tests (if available)

4. **Quality Gates**
   - Clippy with `-D warnings` (ZERO warnings)
   - Code formatting check
   - Compilation check with all features

5. **Production Readiness**
   - Release build validation
   - Performance benchmarks (build < 120s, CLI < 2s)
   - Security audit (cargo-audit)
   - Dependency check (cargo-outdated)

6. **Reports Generated**
   - Production readiness report (Markdown)
   - Validation report for each crate
   - Performance metrics

## Commands to Execute

```bash
# Full production validation suite
cargo make production-readiness-full

# Or step-by-step:
cargo make cleanroom-validate              # Cleanroom tests
cargo make validate-crate                  # Crate validation
./scripts/production-readiness-validation.sh --full
```

## Success Criteria

- ✅ Zero `.unwrap()` or `.expect()` in production code
- ✅ All tests passing
- ✅ Clippy clean (zero warnings)
- ✅ Release build succeeds
- ✅ Performance SLOs met
- ✅ Security audit clean

## Expected Output

Production readiness report saved to:
- `production-readiness-report-YYYYMMDD-HHMMSS.md`

## Time Estimate

- Quick validation: ~2-3 minutes
- Full validation: ~5-10 minutes (includes all tests)
