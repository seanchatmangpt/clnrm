# Enforce Best Practices

## Overview
Systematically enforce FAANG-level code quality standards as defined in .cursorrules. This command validates adherence to core team standards and prevents anti-patterns from entering the codebase.

## Core Validation Checks

### 🎯 Error Handling Standards
```bash
# Check for unwrap()/expect() usage (MUST be zero in production code)
echo "=== Checking for unwrap()/expect() usage ==="
grep -r "\.unwrap()" src/ crates/ || echo "✓ No unwrap() found"
grep -r "\.expect(" src/ crates/ || echo "✓ No expect() found"

# Verify proper error handling patterns
echo "=== Checking error handling patterns ==="
grep -r "CleanroomError" src/ crates/ | wc -l
grep -r "map_err" src/ crates/ | wc -l
grep -r "\.context(" src/ crates/ | wc -l
```

### 🔄 Async/Sync Pattern Validation
```bash
# Check for async trait methods (breaks dyn compatibility)
echo "=== Checking for async trait methods ==="
grep -r "async fn" src/ | grep -v "impl.*for" | grep -v "test" || echo "✓ No async trait methods found"

# Verify proper async implementation patterns
echo "=== Checking async implementation patterns ==="
grep -r "block_in_place" src/ crates/ | wc -l
grep -r "tokio::task" src/ crates/ | wc -l
```

### 🧪 Testing Standards Compliance
```bash
# Verify proper async test patterns
echo "=== Checking async test patterns ==="
grep -r "#\[tokio::test\]" tests/ src/ crates/ | wc -l

# Check for AAA pattern in tests
echo "=== Checking test structure ==="
grep -r "// Arrange" tests/ | wc -l
grep -r "// Act" tests/ | wc -l
grep -r "// Assert" tests/ | wc -l
```

### 🚫 Anti-Pattern Detection
```bash
# Check for printing/logging in production code
echo "=== Checking for production logging ==="
grep -r "println!" src/ crates/ | grep -v "test" | grep -v "example" || echo "✓ No production println! found"
grep -r "eprintln!" src/ crates/ | grep -v "test" | grep -v "example" || echo "✓ No production eprintln! found"

# Check for fake implementations
echo "=== Checking for fake implementations ==="
grep -r "unimplemented!" src/ crates/ | wc -l
grep -r "todo!" src/ crates/ | wc -l
```

### 📦 Module Organization
```bash
# Check for wildcard imports in production code
echo "=== Checking import patterns ==="
grep -r "use crate::\*" src/ crates/ | grep -v "test" | grep -v "example" || echo "✓ No wildcard imports in production"
grep -r "use super::\*" src/ crates/ | grep -v "test" | grep -v "example" || echo "✓ No wildcard super imports in production"
```

## Automated Validation Script

Run the comprehensive best practices validation:

```bash
#!/bin/bash
# scripts/validate-best-practices.sh

echo "🚀 Starting comprehensive best practices validation..."
echo "=================================================="

# Compilation check
echo "📦 Checking compilation..."
cargo check
if [ $? -eq 0 ]; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed"
    exit 1
fi

# Linting check
echo "🔍 Checking linting..."
cargo clippy -- -D warnings
if [ $? -eq 0 ]; then
    echo "✅ No clippy warnings"
else
    echo "❌ Clippy warnings found"
    exit 1
fi

# Test execution
echo "🧪 Running test suite..."
cargo test
if [ $? -eq 0 ]; then
    echo "✅ All tests passing"
else
    echo "❌ Tests failing"
    exit 1
fi

# Error handling validation
echo "🎯 Validating error handling..."
if grep -r "\.unwrap()" src/ crates/ | grep -v "test" | grep -v "example"; then
    echo "❌ Found unwrap() in production code"
    exit 1
fi

if grep -r "\.expect(" src/ crates/ | grep -v "test" | grep -v "example"; then
    echo "❌ Found expect() in production code"
    exit 1
fi

echo "✅ Error handling validation passed"

# Async pattern validation
echo "🔄 Validating async patterns..."
if grep -r "async fn" src/ | grep -v "impl.*for" | grep -v "test"; then
    echo "❌ Found async trait methods (breaks dyn compatibility)"
    exit 1
fi

echo "✅ Async pattern validation passed"

# Production logging check
echo "🚫 Checking for production logging..."
if grep -r "println!" src/ crates/ | grep -v "test" | grep -v "example"; then
    echo "❌ Found println! in production code"
    exit 1
fi

echo "✅ Production logging check passed"

# Fake implementation check
echo "🎭 Checking for honest implementations..."
UNIMPLEMENTED_COUNT=$(grep -r "unimplemented!" src/ crates/ | wc -l)
echo "Found $UNIMPLEMENTED_COUNT unimplemented!() calls (good - honest about incomplete features)"

echo "=================================================="
echo "🎉 All best practices validation passed!"
echo "=================================================="
```

## Manual Review Checklist

### Code Quality Gates
- [ ] **Zero unwrap()/expect()** in production code
- [ ] **All traits dyn compatible** (no async trait methods)
- [ ] **Proper error handling** with CleanroomError and context
- [ ] **Correct async/sync patterns** throughout codebase
- [ ] **No production logging** (use tracing instead)
- [ ] **Honest implementations** (unimplemented!() for incomplete features)

### Architecture Compliance
- [ ] **Module structure** follows established patterns
- [ ] **Import organization** uses specific imports, no wildcards
- [ ] **Error propagation** follows established patterns
- [ ] **Testing patterns** use AAA structure and descriptive names

### Performance & Security
- [ ] **Efficient algorithms** and data structures
- [ ] **Input validation** for all user-facing functions
- [ ] **No hardcoded values** (use configuration)
- [ ] **Proper resource management** (cleanup, RAII patterns)

## Fix Common Issues

### Error Handling Fixes
```rust
// ❌ Before
let result = some_operation().unwrap();

// ✅ After
let result = some_operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;
```

### Async Pattern Fixes
```rust
// ❌ Before (breaks dyn compatibility)
pub trait ServicePlugin: Send + Sync {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ After (dyn compatible)
pub trait ServicePlugin: Send + Sync {
    fn start(&self) -> Result<ServiceHandle>;
}
```

### Production Logging Fixes
```rust
// ❌ Before
fn process_data() -> Result<(), CleanroomError> {
    println!("Processing data..."); // DON'T DO THIS
    Ok(())
}

// ✅ After
fn process_data() -> Result<(), CleanroomError> {
    tracing::info!("Processing data for user {}", user_id);
    Ok(())
}
```

## Success Metrics

- **Zero clippy warnings** in CI/CD pipeline
- **Zero unwrap()/expect()** in production code
- **100% test pass rate** on all test suites
- **All traits dyn compatible** for plugin architecture
- **No fake implementations** - honest about incomplete features
- **Consistent error handling** patterns throughout codebase

## Continuous Integration

Add to your CI/CD pipeline:

```bash
# In .github/workflows/ci.yml or similar
- name: Validate Best Practices
  run: |
    cargo check
    cargo clippy -- -D warnings
    cargo test
    ./scripts/validate-best-practices.sh
```
