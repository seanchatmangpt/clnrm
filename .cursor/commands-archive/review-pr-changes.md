# Review PR Changes

Comprehensive code review checklist for clnrm pull requests following Core Team Standards.

## Pre-Review Automated Checks

Before human review, ensure these automated checks pass:

```bash
# Run all quality gates
bash scripts/ci-gate.sh

# Scan for fake implementations
bash scripts/scan-fakes.sh

# Run clippy
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Run tests
cargo test --all-features

# Run determinism tests
cargo test --test determinism_test

# OTEL validation (if OTEL changes)
bash scripts/validate-otel.sh
```

## Core Team Standards Review

### 1. Error Handling ✅

**Check for**:
- [ ] No `.unwrap()` in production code
- [ ] No `.expect()` in production code
- [ ] All functions return `Result<T, CleanroomError>`
- [ ] Errors have meaningful messages
- [ ] Error context is preserved (use `.map_err()`)

**Examples**:
```rust
// ❌ BAD
let result = operation().unwrap();
let value = config.get("key").expect("key must exist");

// ✅ GOOD
let result = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;
```

### 2. Async/Sync Compliance ✅

**Check for**:
- [ ] Trait methods are sync (not async) for `dyn` compatibility
- [ ] Async used for I/O operations
- [ ] Sync used for computation
- [ ] `tokio::task::block_in_place` used correctly in sync trait methods

**Examples**:
```rust
// ❌ BAD - breaks dyn compatibility
#[async_trait]
trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ GOOD - dyn compatible
trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;
    // Use block_in_place internally for async
}
```

### 3. No Fake Implementations ✅

**Check for**:
- [ ] No `Ok(())` stubs without real work
- [ ] No `println!` in production code (use `tracing`)
- [ ] No hardcoded/canned responses
- [ ] Incomplete features use `unimplemented!()`

**Examples**:
```rust
// ❌ BAD - fake implementation
pub fn execute_test(&self) -> Result<()> {
    println!("Test executed");
    Ok(())  // Did nothing!
}

// ✅ GOOD - honest about incompleteness
pub fn execute_test(&self) -> Result<()> {
    unimplemented!("execute_test: needs container execution")
}
```

### 4. Testing Standards ✅

**Check for**:
- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Descriptive test names (what is being tested)
- [ ] No test interdependencies
- [ ] Proper async test setup (`#[tokio::test]`)
- [ ] Error propagation with `?` operator

**Examples**:
```rust
// ✅ GOOD - AAA pattern
#[tokio::test]
async fn test_container_creation_with_valid_image_succeeds() -> Result<()> {
    // Arrange
    let env = TestEnvironments::unit_test().await?;

    // Act
    let container = env.create_container("alpine:latest").await?;

    // Assert
    assert!(container.is_running());
    Ok(())
}
```

### 5. Code Quality ✅

**Check for**:
- [ ] No compiler warnings
- [ ] No clippy warnings with strict rules
- [ ] Module-level documentation
- [ ] Public items documented
- [ ] Examples in doc comments where helpful

## Functional Review

### 6. Feature Completeness ✅

**Check for**:
- [ ] Feature works end-to-end
- [ ] Edge cases handled
- [ ] Error cases tested
- [ ] Documentation updated
- [ ] TOML configuration example provided (if applicable)

### 7. Performance ✅

**Check for**:
- [ ] No unnecessary clones
- [ ] Efficient data structures
- [ ] Async operations don't block
- [ ] Resource cleanup is prompt

### 8. Security ✅

**Check for**:
- [ ] No hardcoded secrets
- [ ] Input validation
- [ ] No command injection vulnerabilities
- [ ] Safe container operations

## Hermetic Isolation Review

### 9. Container Lifecycle ✅

**Check for**:
- [ ] Containers are properly cleaned up
- [ ] No container state leakage
- [ ] Each test gets fresh environment
- [ ] Port conflicts avoided

### 10. Determinism ✅

**Check for**:
- [ ] Tests produce consistent results
- [ ] No reliance on timing/ordering
- [ ] No global state mutation
- [ ] Timestamps/UUIDs don't affect test results

## Documentation Review

### 11. Code Documentation ✅

**Check for**:
- [ ] Module-level docs explain purpose
- [ ] Public API documented
- [ ] Examples provided
- [ ] Error conditions documented

### 12. User Documentation ✅

**Check for**:
- [ ] README updated (if applicable)
- [ ] CLI guide updated (for new commands)
- [ ] TOML reference updated (for new config)
- [ ] Migration guide (for breaking changes)

## CI/CD Review

### 13. CI Integration ✅

**Check for**:
- [ ] All CI jobs pass
- [ ] No flaky tests
- [ ] Appropriate timeouts set
- [ ] Artifact uploads configured (if needed)

### 14. Feature Flags ✅

**Check for**:
- [ ] OTEL features properly gated (`#[cfg(feature = "otel")]`)
- [ ] AI features in isolated crate (`clnrm-ai`)
- [ ] Feature combinations tested

## Review Checklist Summary

Run through this abbreviated checklist:

**Automated** (must pass before review):
- [ ] CI gates pass
- [ ] Fake scanner clean
- [ ] Clippy clean
- [ ] Tests pass

**Manual**:
- [ ] No unwrap/expect in production
- [ ] Proper error handling
- [ ] AAA test pattern
- [ ] No fake implementations
- [ ] Dyn-compatible traits
- [ ] Documentation updated
- [ ] Hermetic isolation maintained
- [ ] Deterministic tests

**Final**:
- [ ] Feature works end-to-end
- [ ] Edge cases covered
- [ ] Performance acceptable
- [ ] Security verified

## Approval Criteria

PR is ready to merge when:
1. ✅ All automated checks pass
2. ✅ All Core Team Standards met
3. ✅ Code review feedback addressed
4. ✅ Documentation complete
5. ✅ No open questions or concerns

## Post-Merge

After merging:
- [ ] Verify main branch CI passes
- [ ] Test Homebrew installation
- [ ] Run self-tests: `clnrm self-test`
- [ ] Update changelog (if release pending)
