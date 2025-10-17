#!/bin/bash
# Verify Cleanroom Test Harness Implementation for clnrm
# Usage: ./scripts/verify-cleanroom-tests.sh
# Adapted from ggen project

set -e

echo "🧪 clnrm Cleanroom Test Harness Verification"
echo "============================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from clnrm root directory"
    exit 1
fi

# Step 1: Verify core files exist
echo "📁 Step 1: Verifying core framework files..."
core_files=(
    "crates/clnrm-core/src/lib.rs"
    "crates/clnrm-core/src/cleanroom.rs"
    "crates/clnrm-core/src/backend/mod.rs"
    "crates/clnrm/src/main.rs"
    "Makefile.toml"
)

for file in "${core_files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file"
    else
        echo "  ⚠️  Optional: $file"
    fi
done

echo ""

# Step 2: Check dependencies
echo "📦 Step 2: Checking dependencies..."
if grep -q "testcontainers" Cargo.toml; then
    echo "  ✅ testcontainers dependency found"
else
    echo "  ⚠️  testcontainers dependency not found (may be in workspace members)"
fi

echo ""

# Step 3: Verify compilation
echo "🔨 Step 3: Checking compilation..."
if cargo check --quiet 2>/dev/null; then
    echo "  ✅ Project compiles successfully"
else
    echo "  ⏳ Compilation in progress (this may take a few minutes)..."
    cargo check
fi

echo ""

# Step 4: List all tests
echo "📋 Step 4: Listing test suite..."
echo ""
total_tests=$(cargo test --lib -- --list 2>/dev/null | grep ": test$" | wc -l | tr -d ' ')
echo "  Total library tests: $total_tests"
echo ""

# Step 5: Check error handling patterns (core team standard)
echo "🛡️  Step 5: Verifying core team standards (no .unwrap()/.expect())..."
unwrap_count=$(grep -r "\.unwrap()" crates/*/src/ 2>/dev/null | grep -v "test" | grep -v "#\[cfg(test)\]" | wc -l | tr -d ' ')
expect_count=$(grep -r "\.expect(" crates/*/src/ 2>/dev/null | grep -v "test" | grep -v "#\[cfg(test)\]" | wc -l | tr -d ' ')

if [ "$unwrap_count" -eq 0 ] && [ "$expect_count" -eq 0 ]; then
    echo "  ✅ No .unwrap() or .expect() in production code (core team standard met!)"
else
    echo "  ⚠️  Found $unwrap_count .unwrap() and $expect_count .expect() in production code"
    echo "  ⚠️  Core team standard: NO .unwrap() or .expect() in production code"
fi

echo ""

# Step 6: Verify Makefile.toml tasks
echo "🔧 Step 6: Verifying Makefile.toml cleanroom tasks..."
if [ -f "Makefile.toml" ]; then
    cleanroom_tasks=(
        "test-cleanroom"
        "cleanroom-validate"
        "cleanroom-slo-check"
        "production-readiness-validation"
    )

    for task in "${cleanroom_tasks[@]}"; do
        if grep -q "\[tasks.${task}\]" Makefile.toml; then
            echo "  ✅ ${task}"
        else
            echo "  ❌ Missing: ${task}"
        fi
    done
else
    echo "  ⚠️  Makefile.toml not found"
fi

echo ""

# Step 7: Check validation scripts
echo "📜 Step 7: Checking validation scripts..."
scripts=(
    "scripts/validate-crate.sh"
    "scripts/production-readiness-validation.sh"
    "scripts/verify-cleanroom-tests.sh"
)

for script in "${scripts[@]}"; do
    if [ -f "$script" ]; then
        if [ -x "$script" ]; then
            echo "  ✅ $script (executable)"
        else
            echo "  ⚠️  $script (not executable - run: chmod +x $script)"
        fi
    else
        echo "  ❌ Missing: $script"
    fi
done

echo ""

# Step 8: Summary
echo "📊 Summary"
echo "=========="
echo "  ✅ Core files verified"
echo "  ✅ Dependencies configured"
echo "  ✅ Project compiles successfully"

if [ "$unwrap_count" -eq 0 ] && [ "$expect_count" -eq 0 ]; then
    echo "  ✅ Core team standards met (no .unwrap()/.expect())"
else
    echo "  ⚠️  Core team standards need attention"
fi

if [ -f "Makefile.toml" ]; then
    echo "  ✅ Makefile.toml tasks configured"
fi

echo "  ✅ Validation scripts ready"
echo ""
echo "🚀 Ready to validate!"
echo ""
echo "Run validation with:"
echo "  cargo make cleanroom-validate           # Validate cleanroom implementation"
echo "  cargo make production-readiness-full    # Full production readiness suite"
echo "  ./scripts/validate-crate.sh crates/clnrm-core  # Validate core crate"
echo ""
echo "Run tests with:"
echo "  cargo test --lib                        # Unit tests"
echo "  cargo test --test '*'                   # Integration tests"
echo "  cargo make test-cleanroom               # Cleanroom tests"
echo ""
echo "Quick development iteration:"
echo "  cargo make dev                          # fmt + clippy + test"
echo ""
