#!/bin/bash
# scripts/validate-best-practices.sh

echo "🚀 Starting comprehensive best practices validation..."
echo "=================================================="

# Compilation check
echo "📦 Checking compilation..."
if cargo check; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed"
    exit 1
fi

# Linting check
echo "🔍 Checking linting..."
if cargo clippy -- -D warnings; then
    echo "✅ No clippy warnings"
else
    echo "❌ Clippy warnings found"
    exit 1
fi

# Test execution
echo "🧪 Running test suite..."
if cargo test; then
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
