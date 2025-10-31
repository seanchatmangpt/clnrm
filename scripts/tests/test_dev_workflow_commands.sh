#!/usr/bin/env bash
# Test script for v0.7.0 development workflow commands
# Validates all 9 dev workflow commands with comprehensive test coverage

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test result tracking
declare -a FAILED_TEST_NAMES

# Helper function to print test status
test_status() {
    local status=$1
    local test_name=$2
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ PASS${NC}: $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC}: $test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("$test_name")
    fi
}

# Helper function to run command and check exit code
run_test() {
    local test_name=$1
    shift
    local cmd=("$@")

    echo ""
    echo "Running: ${cmd[*]}"

    if "${cmd[@]}" > /dev/null 2>&1; then
        test_status "PASS" "$test_name"
    else
        test_status "FAIL" "$test_name"
    fi
}

echo "========================================="
echo "Development Workflow Commands Test Suite"
echo "v0.7.0 - All 9 Commands"
echo "========================================="
echo ""

# =============================================================================
# Test 1: dev - Development Mode
# =============================================================================
echo "Test Group 1: dev command"
echo "---"

# Skip interactive watch mode tests (require manual validation)
echo -e "${YELLOW}⏸️  SKIP${NC}: dev --watch (requires manual validation)"
echo -e "${YELLOW}⏸️  SKIP${NC}: dev --only pattern (requires manual validation)"
echo -e "${YELLOW}⏸️  SKIP${NC}: dev --timebox (requires manual validation)"

# =============================================================================
# Test 2: dry-run - Validation Without Execution
# =============================================================================
echo ""
echo "Test Group 2: dry-run command"
echo "---"

run_test "dry-run: Valid file" \
    cargo run -p clnrm -- dry-run tests/basic.clnrm.toml

run_test "dry-run: Multiple files" \
    cargo run -p clnrm -- dry-run tests/rosetta-stone/cardinality-rosetta.clnrm.toml tests/rosetta-stone/env-vars-rosetta.clnrm.toml

run_test "dry-run: Verbose mode" \
    cargo run -p clnrm -- dry-run tests/basic.clnrm.toml -v

# =============================================================================
# Test 3: fmt - TOML Formatting
# =============================================================================
echo ""
echo "Test Group 3: fmt command"
echo "---"

# Create temporary test directory
TEMP_DIR=$(mktemp -d)
cp tests/basic.clnrm.toml "$TEMP_DIR/test.clnrm.toml"

run_test "fmt: Format single file" \
    cargo run -p clnrm -- fmt "$TEMP_DIR/test.clnrm.toml"

run_test "fmt: Check mode (should pass after formatting)" \
    cargo run -p clnrm -- fmt --check "$TEMP_DIR/test.clnrm.toml"

run_test "fmt: Verify idempotency" \
    cargo run -p clnrm -- fmt --verify "$TEMP_DIR/test.clnrm.toml"

# Cleanup
rm -rf "$TEMP_DIR"

# =============================================================================
# Test 4: lint - Static Analysis
# =============================================================================
echo ""
echo "Test Group 4: lint command"
echo "---"

run_test "lint: Single file" \
    cargo run -p clnrm -- lint tests/basic.clnrm.toml

run_test "lint: Multiple files" \
    cargo run -p clnrm -- lint tests/rosetta-stone/cardinality-rosetta.clnrm.toml tests/rosetta-stone/env-vars-rosetta.clnrm.toml

run_test "lint: JSON format" \
    cargo run -p clnrm -- lint tests/basic.clnrm.toml --format json

# =============================================================================
# Test 5: record - Baseline Recording
# =============================================================================
echo ""
echo "Test Group 5: record command"
echo "---"

BASELINE_DIR=$(mktemp -d)

run_test "record: Record baseline from specific tests" \
    cargo run -p clnrm -- record tests/rosetta-stone/cardinality-rosetta.clnrm.toml --output "$BASELINE_DIR/baseline.json"

# Verify baseline file created
if [ -f "$BASELINE_DIR/baseline.json" ]; then
    test_status "PASS" "record: Baseline file created"
else
    test_status "FAIL" "record: Baseline file created"
fi

# Verify digest file created
if [ -f "$BASELINE_DIR/baseline.sha256" ]; then
    test_status "PASS" "record: Digest file created"
else
    test_status "FAIL" "record: Digest file created"
fi

# =============================================================================
# Test 6: repro - Reproduce Baseline
# =============================================================================
echo ""
echo "Test Group 6: repro command"
echo "---"

if [ -f "$BASELINE_DIR/baseline.json" ]; then
    run_test "repro: Reproduce baseline" \
        cargo run -p clnrm -- repro "$BASELINE_DIR/baseline.json"

    run_test "repro: Reproduce with digest verification" \
        cargo run -p clnrm -- repro "$BASELINE_DIR/baseline.json" --verify-digest

    run_test "repro: Reproduce with output" \
        cargo run -p clnrm -- repro "$BASELINE_DIR/baseline.json" --output "$BASELINE_DIR/repro-results.json"
else
    echo -e "${YELLOW}⏸️  SKIP${NC}: repro tests (baseline not created)"
fi

# Cleanup
rm -rf "$BASELINE_DIR"

# =============================================================================
# Test 7: red-green - TDD Workflow
# =============================================================================
echo ""
echo "Test Group 7: red-green command"
echo "---"

# Test red state (tests should fail)
run_test "red-green: Verify red state (expect failures)" \
    cargo run -p clnrm -- red-green tests/fake_green/no_execution.clnrm.toml --expect red || true

# Test green state (tests should pass)
run_test "red-green: Verify green state (expect passes)" \
    cargo run -p clnrm -- red-green tests/rosetta-stone/cardinality-rosetta.clnrm.toml --expect green

# =============================================================================
# Test 8: pull - Pre-pull Docker Images
# =============================================================================
echo ""
echo "Test Group 8: pull command"
echo "---"

# Check if Docker is running
if docker info > /dev/null 2>&1; then
    run_test "pull: Pull images from test directory" \
        cargo run -p clnrm -- pull tests/rosetta-stone/

    run_test "pull: Pull images in parallel" \
        cargo run -p clnrm -- pull tests/rosetta-stone/ --parallel --jobs 2
else
    echo -e "${YELLOW}⏸️  SKIP${NC}: pull tests (Docker not running)"
fi

# =============================================================================
# Test 9: render - Template Rendering
# =============================================================================
echo ""
echo "Test Group 9: render command"
echo "---"

# Create a simple test template
RENDER_DIR=$(mktemp -d)
cat > "$RENDER_DIR/test.j2" <<'EOF'
[test.metadata]
name = "{{ name }}"
description = "{{ description }}"
EOF

run_test "render: Render template to stdout" \
    cargo run -p clnrm -- render "$RENDER_DIR/test.j2" --map '{"name":"test","description":"Test template"}'

run_test "render: Render template to file" \
    cargo run -p clnrm -- render "$RENDER_DIR/test.j2" \
        --map '{"name":"test","description":"Test template"}' \
        --output "$RENDER_DIR/output.toml"

run_test "render: Render with --show-vars" \
    cargo run -p clnrm -- render "$RENDER_DIR/test.j2" \
        --map '{"name":"test","description":"Test template"}' \
        --show-vars

# Cleanup
rm -rf "$RENDER_DIR"

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo "Total Tests:  $TOTAL_TESTS"
echo -e "${GREEN}Passed:       $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed:       $FAILED_TESTS${NC}"
    echo ""
    echo "Failed tests:"
    for test_name in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "  ${RED}❌${NC} $test_name"
    done
else
    echo -e "${GREEN}Failed:       $FAILED_TESTS${NC}"
fi
echo ""

# Exit with appropriate code
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
else
    echo -e "${GREEN}✅ All tests passed${NC}"
    exit 0
fi
