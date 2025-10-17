# Code Review Checklist

## Overview
Comprehensive code review following FAANG-level standards and core team best practices. Ensures code quality, proper error handling, async/sync patterns, and adherence to .cursorrules guidelines.

## Pre-Review Steps

1. **Verify Compilation**
   ```bash
   cargo check
   cargo clippy -- -D warnings
   ```

2. **Run Test Suite**
   ```bash
   cargo test
   ```

3. **Check Best Practices Compliance**
   ```bash
   ./scripts/check-best-practices.sh
   ```

## Core Review Areas

### 🎯 Error Handling
- [ ] **No unwrap()/expect()** in production code
- [ ] **Proper Result<T, E>** types with meaningful errors
- [ ] **Structured error context** using CleanroomError
- [ ] **Error propagation** follows established patterns

### 🔄 Async/Sync Patterns
- [ ] **Async for I/O operations** (containers, network, files)
- [ ] **Sync for pure computation** (hashing, validation)
- [ ] **No async trait methods** (breaks dyn compatibility)
- [ ] **Proper async implementation** using tokio patterns

### 🧪 Testing Standards
- [ ] **AAA pattern** (Arrange, Act, Assert)
- [ ] **Descriptive test names** explaining intent
- [ ] **Proper async test functions** with #[tokio::test]
- [ ] **No external dependencies** in unit tests
- [ ] **Honest test implementation** (no fake Ok(()) stubs)

### 📦 Module Organization
- [ ] **Proper module structure** with clear hierarchies
- [ ] **Specific imports** (no wildcard imports in prod)
- [ ] **Consistent naming** following project conventions
- [ ] **Proper use declarations** and visibility

### 🚫 Anti-Patterns
- [ ] **No printing/logging** in production code
- [ ] **No hardcoded values** (use configurable constants)
- [ ] **No fake implementations** (use unimplemented!() for incomplete features)
- [ ] **No breaking changes** without migration plan

### 🔒 Security & Performance
- [ ] **Input validation** for all user data
- [ ] **Proper error handling** doesn't leak internals
- [ ] **Efficient data structures** and algorithms
- [ ] **No unnecessary allocations** or complexity

## Post-Review Actions

1. **Address All Issues**
   - Critical issues must be fixed before merge
   - Non-critical issues should be noted for future improvement

2. **Update Documentation**
   - Ensure README reflects any API changes
   - Update inline documentation for new features

3. **Run Full Validation**
   ```bash
   ./scripts/production-readiness-validation.sh
   ```

## Definition of Done
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] No unwrap()/expect() in production code
- [ ] All traits maintain dyn compatibility
- [ ] No breaking changes to public APIs
- [ ] Proper error handling throughout
- [ ] Async/sync patterns followed correctly
- [ ] No fake implementations - incomplete features call unimplemented!()

## Review Commands
Run these commands to validate the changes:

```bash
# Full validation suite
cargo test && cargo clippy -- -D warnings

# Best practices check
./scripts/check-best-practices.sh

# Production readiness
./scripts/production-readiness-validation.sh
```
