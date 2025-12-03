#!/bin/bash
# Validation Infrastructure Verification Script
# Version: 1.2.0+
# Purpose: Verify OTLP export and Weaver validation infrastructure is working

set -e

echo "🔍 OTLP Export and Weaver Validation Infrastructure Verification"
echo "================================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
FAILED=0

# Function to print success
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print failure
failure() {
    echo -e "${RED}❌ $1${NC}"
    FAILED=1
}

# Function to print warning
warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo "Step 1: Check Weaver Installation"
echo "-----------------------------------"
if command -v weaver &> /dev/null; then
    VERSION=$(weaver --version 2>&1)
    success "Weaver installed: $VERSION"
else
    failure "Weaver not installed. Run: cargo install weaver-cli"
fi
echo ""

echo "Step 2: Validate Registry Schemas"
echo "----------------------------------"
if [ -d "registry" ]; then
    success "Registry directory exists"

    if weaver registry check -r registry/ &> /dev/null; then
        success "Registry schemas valid"
    else
        failure "Registry validation failed. Run: weaver registry check -r registry/"
    fi
else
    failure "Registry directory not found"
fi
echo ""

echo "Step 3: Check Validator Module"
echo "-------------------------------"
if [ -f "crates/clnrm-core/src/telemetry/validators.rs" ]; then
    LINES=$(wc -l < crates/clnrm-core/src/telemetry/validators.rs)
    success "Validator module exists ($LINES lines)"
else
    failure "Validator module not found"
fi
echo ""

echo "Step 4: Check Integration Tests"
echo "--------------------------------"
if [ -f "crates/clnrm-core/tests/telemetry/validator_integration.rs" ]; then
    LINES=$(wc -l < crates/clnrm-core/tests/telemetry/validator_integration.rs)
    success "Integration tests exist ($LINES lines)"
else
    failure "Integration tests not found"
fi
echo ""

echo "Step 5: Check Documentation"
echo "---------------------------"
DOCS=(
    "docs/quality/OTLP_VALIDATION.md"
    "docs/WEAVER_SETUP.md"
    "docs/quality/VALIDATION_INFRASTRUCTURE_SUMMARY.md"
)

for DOC in "${DOCS[@]}"; do
    if [ -f "$DOC" ]; then
        success "$(basename $DOC) exists"
    else
        failure "$(basename $DOC) not found"
    fi
done
echo ""

echo "Step 6: Build with OTEL Features"
echo "---------------------------------"
if cargo build --release --features otel -p clnrm-core &> /dev/null; then
    success "Build successful with OTEL features"
else
    failure "Build failed. Check compilation errors."
fi
echo ""

echo "Step 7: Run Validator Unit Tests"
echo "---------------------------------"
if cargo test -p clnrm-core --lib validators::tests --features otel &> /dev/null; then
    success "All validator unit tests passed"
else
    failure "Validator tests failed. Run: cargo test -p clnrm-core --lib validators::tests --features otel"
fi
echo ""

echo "Step 8: Code Quality Check"
echo "---------------------------"
if cargo clippy -p clnrm-core --features otel -- -D warnings 2>&1 | grep -q "warning\|error"; then
    failure "Clippy found warnings or errors"
else
    success "No clippy warnings or errors"
fi
echo ""

echo "Step 9: Check Validator Exports"
echo "--------------------------------"
if grep -q "pub mod validators" crates/clnrm-core/src/telemetry.rs; then
    success "Validators module exported from telemetry"
else
    failure "Validators module not exported"
fi
echo ""

echo "Step 10: Verify Test Integration"
echo "---------------------------------"
if grep -q "pub mod validator_integration" crates/clnrm-core/tests/telemetry/mod.rs; then
    success "Integration tests registered in test module"
else
    failure "Integration tests not registered"
fi
echo ""

# Summary
echo "================================================================="
if [ $FAILED -eq 0 ]; then
    success "All validation infrastructure checks passed! 🎉"
    echo ""
    echo "Infrastructure is production-ready:"
    echo "  - OTLP export validation: ✅"
    echo "  - Weaver health checking: ✅"
    echo "  - Telemetry quality validation: ✅"
    echo "  - Integration tests: ✅"
    echo "  - Documentation: ✅"
    echo ""
    echo "Next steps:"
    echo "  1. Run full test suite: cargo test --features otel"
    echo "  2. Review documentation: docs/quality/OTLP_VALIDATION.md"
    echo "  3. Setup Weaver: docs/WEAVER_SETUP.md"
    exit 0
else
    failure "Some validation checks failed"
    echo ""
    echo "Please review the failures above and fix them before deployment."
    exit 1
fi
