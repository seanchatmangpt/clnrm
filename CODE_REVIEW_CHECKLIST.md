# Code Review Checklist - Eliminating Mura

This checklist ensures consistency across the clnrm codebase and prevents unevenness (Mura) in code quality, patterns, and style.

## 🎯 Consistency Standards

### Documentation Consistency
- [ ] **Module docs**: Each module has `//!` module-level documentation explaining purpose
- [ ] **Function docs**: All public functions have `///` documentation with:
  - `# Arguments` section listing parameters
  - `# Returns` section describing return values
  - `# Core Team Standards` section for complex functions
- [ ] **Example consistency**: Code examples follow same patterns
- [ ] **Comment style**: Comments use consistent formatting

### Error Handling Consistency
- [ ] **No unwrap/expect**: Production code never uses `.unwrap()` or `.expect()`
- [ ] **Result types**: All fallible operations return `Result<T, E>`
- [ ] **Error propagation**: Errors are properly propagated with `?`
- [ ] **Error context**: Errors include helpful context messages

### Code Style Consistency
- [ ] **Naming**: Functions use `snake_case`, types use `PascalCase`
- [ ] **Imports**: `std::*` imports before `crate::*` imports
- [ ] **Line length**: Lines fit within reasonable width (<100 chars)
- [ ] **Formatting**: Code passes `cargo fmt`

### Pattern Consistency
- [ ] **CLI commands**: Follow same structure (validate inputs, show output, TODO comments)
- [ ] **Error types**: Use `clnrm_core::error::CleanroomError` consistently
- [ ] **Tracing**: Use `tracing::info!`, `tracing::error!`, etc. consistently
- [ ] **Async handling**: Async functions use consistent patterns

### TODO Comment Consistency
- [ ] **Format**: `TODO: <action> <details>` (e.g., `TODO: Implement trace analysis using clnrm_core::validation`)
- [ ] **Actionable**: Each TODO describes a concrete next step
- [ ] **Context**: TODOs include enough context to understand the work needed
- [ ] **No vague TODOs**: Avoid "TODO: implement this" - be specific

## 🔍 Quality Gates

### Functionality
- [ ] **Compiles**: Code compiles without errors
- [ ] **Tests pass**: All existing tests continue to pass
- [ ] **New tests**: New functionality includes appropriate tests
- [ ] **Integration**: Changes work with existing integrations

### Performance
- [ ] **No regressions**: Performance is maintained or improved
- [ ] **Efficient**: No unnecessary allocations or operations
- [ ] **Scalable**: Changes don't break scalability assumptions

### Security
- [ ] **Input validation**: All inputs are validated
- [ ] **Error handling**: Errors don't leak sensitive information
- [ ] **Resource cleanup**: Resources are properly cleaned up

## 📋 Review Process

### Pre-Review Checklist (Author)
- [ ] Run `./scripts/check-consistency.sh`
- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] Run `cargo test` (all tests pass)
- [ ] Update documentation if behavior changed
- [ ] Add tests for new functionality

### During Review
- [ ] Check all consistency standards above
- [ ] Verify functionality works as described
- [ ] Ensure no regressions in existing behavior
- [ ] Confirm appropriate test coverage

### Post-Review
- [ ] Address all review feedback
- [ ] Re-run consistency checks
- [ ] Update this checklist if new standards emerge

## 🚨 Critical Consistency Rules

### NEVER ALLOW
- ❌ `unwrap()` or `expect()` in production code
- ❌ Inconsistent error handling patterns
- ❌ Undocumented public APIs
- ❌ Inconsistent naming conventions
- ❌ Vague or non-actionable TODO comments

### ALWAYS REQUIRE
- ✅ Comprehensive documentation for public APIs
- ✅ Consistent error handling with Result types
- ✅ Clear, actionable TODO comments
- ✅ Consistent code formatting and style
- ✅ Appropriate test coverage

## 🎯 Mura Prevention

This checklist prevents unevenness by ensuring:
- **Consistent quality** across all code
- **Standardized patterns** for common operations
- **Uniform documentation** standards
- **Predictable error handling** everywhere
- **Actionable development** tasks

**Remember**: Consistency reduces cognitive load and maintenance cost. Consistent code is easier to understand, modify, and maintain.