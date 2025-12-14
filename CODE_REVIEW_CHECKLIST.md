# Code Review Checklist - Eliminate Mura (Unevenness)

This checklist ensures consistent code quality, patterns, and style across the clnrm codebase. All code reviews must pass these checks before merging.

## 🎯 Core Consistency Standards

### Import Consistency
- [ ] **No duplicate imports** - Each module imported exactly once
- [ ] **Consistent import style** - Either fully qualified (`clnrm_core::error::Result`) or imported (`use clnrm_core::error::Result`)
- [ ] **Alphabetical ordering** - Imports sorted alphabetically within groups
- [ ] **Logical grouping** - std, external crates, internal modules properly separated

### Function Signature Consistency
- [ ] **Parameter types** - `&PathBuf` vs `&std::path::Path` vs `&[PathBuf]` used consistently
- [ ] **Return types** - `Result<T>` vs `Result<()>` vs concrete types consistent
- [ ] **Async patterns** - All similar operations either sync or async
- [ ] **Error handling** - Same error patterns for similar operations

### Documentation Consistency
- [ ] **Module docs** - `//!` comments present and informative
- [ ] **Function docs** - `///` comments with `# Arguments` and `# Returns` sections
- [ ] **Code comments** - `//` comments explain complex logic
- [ ] **Documentation completeness** - Public APIs fully documented

### Error Handling Consistency
- [ ] **Error constructors** - `CleanroomError::method()` vs fully qualified names consistent
- [ ] **Error context** - `.with_context()` and `.with_source()` used appropriately
- [ ] **Error types** - Correct error variants (`config_error`, `validation_error`, etc.)
- [ ] **Error messages** - Clear, actionable error messages

## 🔧 Code Quality Standards

### Compilation & Linting
- [ ] **Compiles cleanly** - `cargo check` passes without errors
- [ ] **Clippy clean** - `cargo clippy -- -D warnings` passes
- [ ] **Format consistent** - `cargo fmt -- --check` passes
- [ ] **No unused code** - No unused imports, variables, or functions

### Testing Standards
- [ ] **Tests compile** - All test code compiles
- [ ] **Test coverage** - New code includes appropriate tests
- [ ] **Test patterns** - AAA pattern (Arrange, Act, Assert) followed
- [ ] **Deterministic tests** - Tests don't rely on external state

### Fake Scanner Compliance
- [ ] **No unwrap/expect** - Zero usage in production code
- [ ] **No fake stubs** - All functions either work or use `unimplemented!()`
- [ ] **No println!** - Proper logging through tracing
- [ ] **No hardcoded responses** - Real implementations only

## 🚀 Architecture Consistency

### Module Organization
- [ ] **Logical structure** - Code organized into appropriate modules
- [ ] **Dependency flow** - Core doesn't depend on CLI, CLI configures core
- [ ] **Public API** - Clear, minimal public interfaces
- [ ] **Encapsulation** - Implementation details properly hidden

### Async/Sync Patterns
- [ ] **I/O operations async** - File, network, container operations use async
- [ ] **Computation sync** - Pure functions and simple operations sync
- [ ] **No async traits** - Per .cursorrules: "NEVER make trait methods async"
- [ ] **Block-on-place** - Async operations in sync contexts use proper patterns

### Error Propagation
- [ ] **Result types** - All fallible operations return `Result<T, E>`
- [ ] **Error context** - Errors include helpful context and sources
- [ ] **Typed errors** - Project-specific error types, not generic errors
- [ ] **No silent failures** - All errors handled or propagated

## 📊 Performance & Reliability

### Resource Management
- [ ] **No memory leaks** - Resources properly cleaned up
- [ ] **Efficient algorithms** - Appropriate data structures and algorithms
- [ ] **Bounded resources** - Collections and operations have reasonable limits
- [ ] **Cancellation safety** - Async operations handle cancellation properly

### Determinism
- [ ] **Seed-based random** - Any randomness uses deterministic seeds
- [ ] **Isolated file ops** - File operations use isolated temporary directories
- [ ] **Mocked externals** - External dependencies (time, network) mocked in tests
- [ ] **Reproducible builds** - Same inputs produce identical outputs

## 🔒 Security Standards

### Input Validation
- [ ] **All inputs validated** - User inputs checked for safety
- [ ] **Path sanitization** - File paths validated and sanitized
- [ ] **No injection** - SQL, command, template injection prevented
- [ ] **Secure defaults** - Conservative defaults for security settings

### Error Information
- [ ] **No sensitive leaks** - Errors don't expose internal implementation
- [ ] **Actionable messages** - Error messages help users fix issues
- [ ] **Appropriate detail** - Debug info in debug builds, user-friendly in release
- [ ] **Logging security** - Sensitive operations logged without exposing secrets

## ✅ Pre-Merge Validation

**CRITICAL**: All boxes must be checked before merging. If any box cannot be checked, the code needs revision.

### Automated Checks (CI)
- [ ] CI passes completely (including new fake scanner check)
- [ ] All tests pass on Linux and macOS
- [ ] Code coverage maintained or improved
- [ ] Performance benchmarks not regressed

### Manual Review
- [ ] Code follows established patterns
- [ ] No new technical debt introduced
- [ ] Documentation updated for any API changes
- [ ] Breaking changes properly communicated

## 🚨 Blocking Issues

**These issues will block merging and require immediate fixes:**

- ❌ Compilation errors
- ❌ Clippy warnings (unless explicitly allowed)
- ❌ Fake scanner failures
- ❌ Test failures
- ❌ Breaking changes without migration plan
- ❌ unwrap/expect in production code
- ❌ Async trait methods
- ❌ Missing documentation for public APIs

## 📝 Review Process

1. **Automated checks pass** - CI must be green
2. **Code review** - At least one maintainer reviews
3. **Checklist verification** - All boxes checked
4. **Merge approval** - Only after all checks pass

## 🔄 Continuous Improvement

This checklist evolves with the codebase. When new inconsistency patterns emerge, add checks to prevent them. Regular reviews of this checklist ensure it remains comprehensive and effective.

**Last updated**: December 2025
**Version**: 1.0