# Fix Core Team Standards Violations

Scan the codebase for violations of clnrm core team standards and fix them automatically.

## Core Team Standards (MANDATORY)

### 1. Error Handling
- ❌ **NEVER** use `.unwrap()` in production code
- ❌ **NEVER** use `.expect()` in production code
- ✅ **ALWAYS** use `Result<T, CleanroomError>`
- ✅ **ALWAYS** use proper error context

### 2. Async/Sync Rules
- ❌ **NEVER** make trait methods async (breaks `dyn` compatibility)
- ✅ Use `tokio::task::block_in_place` for async operations in sync trait methods
- ✅ Keep trait methods sync for `dyn Trait` compatibility

### 3. Testing Standards
- ✅ Follow AAA pattern (Arrange, Act, Assert)
- ✅ Descriptive test names explaining what is tested
- ✅ No fake `Ok(())` stubs - use `unimplemented!()` for incomplete features

## What This Does

1. **Scan Production Code**
   - Find all `.unwrap()` calls in `src/` (excluding tests)
   - Find all `.expect()` calls in `src/` (excluding tests)
   - Detect async trait methods
   - Check for fake `Ok(())` returns

2. **Generate Fixes**
   - Replace `.unwrap()` with proper `?` operator and error mapping
   - Replace `.expect()` with meaningful error messages
   - Convert async trait methods to sync with `block_in_place`
   - Replace fake `Ok(())` with `unimplemented!()`

3. **Validate Changes**
   - Run `cargo check` to ensure compilation
   - Run `cargo clippy` to verify quality
   - Run tests to ensure behavior unchanged

## Commands to Execute

```bash
# Check for violations
grep -r "\.unwrap()" crates/*/src/ | grep -v "test" | grep -v "#\[cfg(test)\]"
grep -r "\.expect(" crates/*/src/ | grep -v "test" | grep -v "#\[cfg(test)\]"

# Run validation
cargo make validate-crate

# Auto-fix clippy issues
cargo clippy --fix --allow-dirty --allow-staged
```

## Example Fixes

### Before (WRONG):
```rust
let result = operation().unwrap();
```

### After (CORRECT):
```rust
let result = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;
```

## Expected Outcome

- ✅ Zero `.unwrap()` in production code
- ✅ Zero `.expect()` in production code
- ✅ All traits remain `dyn` compatible
- ✅ Proper error handling throughout
