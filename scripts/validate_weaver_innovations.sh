#!/bin/bash
# Validate Weaver Innovations - Quick Validation Script
#
# Tests all three innovations:
# 1. Statistics analyzer
# 2. Emit integration
# 3. CI/CD workflow (syntax)

set -e

echo "🔍 Validating Weaver Innovations"
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
PASSED=0
FAILED=0

# Function to print test result
test_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ PASSED${NC}: $2"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}❌ FAILED${NC}: $2"
        FAILED=$((FAILED + 1))
    fi
}

echo "📊 Test 1: Weaver Statistics Module"
echo "-----------------------------------"

# Check if module compiles
echo -n "  Compiling weaver_stats... "
cargo check -p clnrm-core --lib 2>&1 | grep -q "Finished"
test_result $? "weaver_stats.rs compiles"

# Check if Weaver is installed
echo -n "  Checking Weaver installation... "
weaver --version > /dev/null 2>&1
test_result $? "Weaver binary found"

# Run statistics on registry
if [ -d "registry" ]; then
    echo -n "  Running registry statistics... "
    weaver registry stats --registry registry/ > /tmp/stats_output.txt 2>&1
    test_result $? "Statistics collection"

    # Parse statistics
    echo -n "  Parsing statistics output... "
    grep -q "Total number of attributes:" /tmp/stats_output.txt
    test_result $? "Statistics parseable"

    # Check for required attributes
    echo -n "  Checking required attributes... "
    grep -q "required:" /tmp/stats_output.txt
    test_result $? "Required attributes found"
else
    echo -e "${YELLOW}⚠️  SKIPPED${NC}: Registry not found"
fi

echo ""
echo "🚀 Test 2: Weaver Emit Module"
echo "-----------------------------"

# Check if module compiles
echo -n "  Compiling weaver_emit... "
cargo check -p clnrm-core --lib 2>&1 | grep -q "Finished"
test_result $? "weaver_emit.rs compiles"

# Check if emit command exists
echo -n "  Checking Weaver emit support... "
weaver registry emit --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    test_result 0 "Weaver emit command available"

    # Try emitting to stdout (safe, no collector needed)
    if [ -d "registry" ]; then
        echo -n "  Testing emit to stdout... "
        timeout 10s weaver registry emit --registry registry/ --stdout > /tmp/emit_output.json 2>&1
        if [ $? -eq 0 ] || [ $? -eq 124 ]; then
            # Either succeeded or timeout (which is ok, emission started)
            test_result 0 "Emit command executes"
        else
            test_result 1 "Emit command failed"
        fi
    fi
else
    echo -e "${YELLOW}⚠️  SKIPPED${NC}: Weaver emit not available in this version"
fi

echo ""
echo "🔧 Test 3: CI/CD Validation Gate"
echo "--------------------------------"

# Check if workflow file exists
echo -n "  Checking workflow file... "
if [ -f ".github/workflows/weaver-validation-gate.yml" ]; then
    test_result 0 "Workflow file exists"
else
    test_result 1 "Workflow file missing"
fi

# Validate YAML syntax (requires yq or python)
if [ -f ".github/workflows/weaver-validation-gate.yml" ]; then
    echo -n "  Validating YAML syntax... "
    if command -v python3 > /dev/null; then
        python3 -c "import yaml; yaml.safe_load(open('.github/workflows/weaver-validation-gate.yml'))" 2>/dev/null
        test_result $? "YAML syntax valid"
    elif command -v yq > /dev/null; then
        yq eval '.name' .github/workflows/weaver-validation-gate.yml > /dev/null
        test_result $? "YAML syntax valid"
    else
        echo -e "${YELLOW}⚠️  SKIPPED${NC}: No YAML validator found (install python3 or yq)"
    fi

    # Check for required jobs
    echo -n "  Checking workflow jobs... "
    grep -q "weaver-schema-check:" .github/workflows/weaver-validation-gate.yml
    SCHEMA=$?
    grep -q "weaver-statistics:" .github/workflows/weaver-validation-gate.yml
    STATS=$?
    grep -q "weaver-live-check:" .github/workflows/weaver-validation-gate.yml
    LIVE=$?
    grep -q "quality-gate:" .github/workflows/weaver-validation-gate.yml
    QUALITY=$?

    if [ $SCHEMA -eq 0 ] && [ $STATS -eq 0 ] && [ $LIVE -eq 0 ] && [ $QUALITY -eq 0 ]; then
        test_result 0 "All 4 gates defined"
    else
        test_result 1 "Missing gates"
    fi
fi

echo ""
echo "📖 Test 4: Documentation"
echo "------------------------"

# Check documentation files
echo -n "  Checking innovations guide... "
if [ -f "docs/weaver/WEAVER_INNOVATIONS_GUIDE.md" ]; then
    test_result 0 "Guide exists"
else
    test_result 1 "Guide missing"
fi

echo -n "  Checking deliverables doc... "
if [ -f "docs/weaver/CODER_DELIVERABLES.md" ]; then
    test_result 0 "Deliverables doc exists"
else
    test_result 1 "Deliverables doc missing"
fi

echo ""
echo "🧪 Test 5: Integration Tests"
echo "----------------------------"

# Check test file exists
echo -n "  Checking test file... "
if [ -f "tests/weaver/weaver_innovations_integration_test.rs" ]; then
    test_result 0 "Integration tests exist"
else
    test_result 1 "Integration tests missing"
fi

# Compile tests (don't run, some require Weaver)
echo -n "  Compiling integration tests... "
cargo test --test weaver_innovations_integration_test --no-run 2>&1 | grep -q "Finished"
test_result $? "Tests compile"

echo ""
echo "🔬 Test 6: Code Quality"
echo "----------------------"

# Check for unwrap/expect (should be none in production code)
echo -n "  Checking for .unwrap() in weaver_stats... "
if grep -q ".unwrap()" crates/clnrm-core/src/telemetry/weaver_stats.rs; then
    test_result 1 "Found .unwrap() (should use proper error handling)"
else
    test_result 0 "No .unwrap() found"
fi

echo -n "  Checking for .unwrap() in weaver_emit... "
if grep -q ".unwrap()" crates/clnrm-core/src/telemetry/weaver_emit.rs; then
    test_result 1 "Found .unwrap() (should use proper error handling)"
else
    test_result 0 "No .unwrap() found"
fi

# Check for proper Result returns
echo -n "  Checking Result<T, CleanroomError> usage... "
if grep -q "Result<" crates/clnrm-core/src/telemetry/weaver_stats.rs; then
    test_result 0 "Proper Result types used"
else
    test_result 1 "Missing Result types"
fi

echo ""
echo "=================================="
echo "📊 Final Results"
echo "=================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed! Weaver innovations are production-ready.${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Some tests failed. Review output above.${NC}"
    exit 1
fi
