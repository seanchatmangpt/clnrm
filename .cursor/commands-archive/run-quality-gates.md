# Run Quality Gates

Execute all quality gates before committing changes to ensure code meets Core Team Standards.

## Quality Checks

Run the comprehensive CI gate script that performs:

1. **Critical Pattern Detection**
   - Scan for `.unwrap()`, `.expect()`, `panic!()`
   - Detect fake/stub implementations
   - Find `println!` in production code

2. **Core API Verification**
   - Verify `CleanroomEnvironment::new` exists
   - Check `ServicePlugin::start` is present
   - Validate `Backend::create_container` exists
   - Ensure `CleanroomError` is defined

3. **Compilation Testing**
   - Test default features
   - Test OTEL features (traces, metrics, logs)
   - Verify all feature combinations build

4. **Linting**
   - Run clippy with strict rules
   - Enforce `-D warnings -D clippy::unwrap_used -D clippy::expect_used`

5. **Error Handling**
   - Verify `Result<T, CleanroomError>` usage
   - Ensure no unwrap/expect in production

6. **Documentation**
   - Check module-level documentation
   - Verify public item documentation

## Commands to Execute

```bash
# Run all quality gates
bash scripts/ci-gate.sh

# Run specific gate
bash scripts/ci-gate.sh --check critical_patterns

# Fail fast on first error
bash scripts/ci-gate.sh --fail-fast

# Run fake code scanner
bash scripts/scan-fakes.sh

# Validate best practices
bash scripts/validate-best-practices.sh
```

## Expected Results

All gates should pass with exit code 0. Any failures will generate:
- JSON report: `target/ci-gate-report/report.json`
- Markdown report: `target/ci-gate-report/report.md`

## Core Team Standards

This command ensures compliance with:
- ✅ No `.unwrap()` or `.expect()` in production
- ✅ Proper `Result<T, CleanroomError>` error handling
- ✅ No fake implementations (stubs must use `unimplemented!()`)
- ✅ Production code uses `tracing` instead of `println!`
- ✅ All tests follow AAA pattern
- ✅ Comprehensive documentation
