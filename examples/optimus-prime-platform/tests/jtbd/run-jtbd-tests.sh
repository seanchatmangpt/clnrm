#!/bin/bash

# JTBD Test Suite Runner
# Runs all Jobs To Be Done tests in sequence and generates comprehensive report

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLNRM_PATH="${CLNRM_PATH:-../../target/release/clnrm}"
TEST_DIR="/Users/sac/clnrm/examples/optimus-prime-platform/tests/jtbd"
RESULTS_DIR="$TEST_DIR/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="$RESULTS_DIR/jtbd_test_report_${TIMESTAMP}.txt"

# Test files
CHILD_TESTS=(
    "$TEST_DIR/child-surface/jtbd-001-achievement-recognition.clnrm.toml"
    "$TEST_DIR/child-surface/jtbd-002-virtue-mapping.clnrm.toml"
    "$TEST_DIR/child-surface/jtbd-003-reward-delivery.clnrm.toml"
    "$TEST_DIR/child-surface/jtbd-004-premium-cta.clnrm.toml"
)

EXECUTIVE_TESTS=(
    "$TEST_DIR/executive-surface/jtbd-005-kpi-queries.clnrm.toml"
    "$TEST_DIR/executive-surface/jtbd-006-dashboard-visualization.clnrm.toml"
    "$TEST_DIR/executive-surface/jtbd-007-ab-testing.clnrm.toml"
)

PARENT_TESTS=(
    "$TEST_DIR/parent-surface/jtbd-008-monitor-progress.clnrm.toml"
)

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Create results directory
mkdir -p "$RESULTS_DIR"

# Initialize report
echo "================================================================" > "$REPORT_FILE"
echo "JTBD Test Suite Execution Report" >> "$REPORT_FILE"
echo "Generated: $(date)" >> "$REPORT_FILE"
echo "================================================================" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}✓ PASS${NC} - $message"
            echo "✓ PASS - $message" >> "$REPORT_FILE"
            ;;
        "FAIL")
            echo -e "${RED}✗ FAIL${NC} - $message"
            echo "✗ FAIL - $message" >> "$REPORT_FILE"
            ;;
        "SKIP")
            echo -e "${YELLOW}⊘ SKIP${NC} - $message"
            echo "⊘ SKIP - $message" >> "$REPORT_FILE"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ INFO${NC} - $message"
            echo "ℹ INFO - $message" >> "$REPORT_FILE"
            ;;
    esac
}

# Function to run a single test
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .clnrm.toml)

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo ""
    echo "================================================================"
    echo "Running: $test_name"
    echo "================================================================"
    echo "" >> "$REPORT_FILE"
    echo "----------------------------------------------------------------" >> "$REPORT_FILE"
    echo "Test: $test_name" >> "$REPORT_FILE"
    echo "File: $test_file" >> "$REPORT_FILE"
    echo "Started: $(date)" >> "$REPORT_FILE"
    echo "----------------------------------------------------------------" >> "$REPORT_FILE"

    if [ ! -f "$test_file" ]; then
        print_status "SKIP" "$test_name - File not found"
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        echo "Status: SKIPPED - File not found" >> "$REPORT_FILE"
        return
    fi

    # Run the test
    local start_time=$(date +%s)
    if "$CLNRM_PATH" run "$test_file" >> "$REPORT_FILE" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_status "PASS" "$test_name (${duration}s)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "Status: PASSED" >> "$REPORT_FILE"
        echo "Duration: ${duration}s" >> "$REPORT_FILE"
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_status "FAIL" "$test_name (${duration}s)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "Status: FAILED" >> "$REPORT_FILE"
        echo "Duration: ${duration}s" >> "$REPORT_FILE"
    fi
}

# Function to run test category
run_category() {
    local category=$1
    shift
    local tests=("$@")

    echo ""
    echo "================================================================"
    echo "Testing Category: $category"
    echo "================================================================"
    echo "" >> "$REPORT_FILE"
    echo "================================================================" >> "$REPORT_FILE"
    echo "CATEGORY: $category" >> "$REPORT_FILE"
    echo "================================================================" >> "$REPORT_FILE"

    for test in "${tests[@]}"; do
        run_test "$test"
    done
}

# Main execution
print_status "INFO" "Starting JTBD Test Suite"
print_status "INFO" "CLNRM Path: $CLNRM_PATH"
print_status "INFO" "Results Directory: $RESULTS_DIR"

# Check if CLNRM exists
if [ ! -f "$CLNRM_PATH" ]; then
    print_status "FAIL" "CLNRM binary not found at $CLNRM_PATH"
    echo ""
    echo "Please build CLNRM first:"
    echo "  cd /Users/sac/clnrm"
    echo "  cargo build --release"
    exit 1
fi

# Run all test categories
run_category "Child Surface Tests" "${CHILD_TESTS[@]}"
run_category "Executive Surface Tests" "${EXECUTIVE_TESTS[@]}"
run_category "Parent Surface Tests" "${PARENT_TESTS[@]}"

# Generate summary
echo ""
echo "================================================================"
echo "Test Execution Summary"
echo "================================================================"
echo "Total Tests:   $TOTAL_TESTS"
echo "Passed:        $PASSED_TESTS"
echo "Failed:        $FAILED_TESTS"
echo "Skipped:       $SKIPPED_TESTS"
echo "Success Rate:  $(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")%"
echo "================================================================"

echo "" >> "$REPORT_FILE"
echo "================================================================" >> "$REPORT_FILE"
echo "SUMMARY" >> "$REPORT_FILE"
echo "================================================================" >> "$REPORT_FILE"
echo "Total Tests:   $TOTAL_TESTS" >> "$REPORT_FILE"
echo "Passed:        $PASSED_TESTS" >> "$REPORT_FILE"
echo "Failed:        $FAILED_TESTS" >> "$REPORT_FILE"
echo "Skipped:       $SKIPPED_TESTS" >> "$REPORT_FILE"
echo "Success Rate:  $(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")%" >> "$REPORT_FILE"
echo "================================================================" >> "$REPORT_FILE"
echo "Report saved to: $REPORT_FILE" >> "$REPORT_FILE"

print_status "INFO" "Report saved to: $REPORT_FILE"

# Generate JSON report
JSON_REPORT="$RESULTS_DIR/jtbd_test_report_${TIMESTAMP}.json"
cat > "$JSON_REPORT" << EOF
{
  "timestamp": "$TIMESTAMP",
  "summary": {
    "total": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "skipped": $SKIPPED_TESTS,
    "successRate": $(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
  },
  "categories": {
    "child_surface": {
      "total": ${#CHILD_TESTS[@]},
      "tests": [$(printf '"%s",' "${CHILD_TESTS[@]}" | sed 's/,$//')]
    },
    "executive_surface": {
      "total": ${#EXECUTIVE_TESTS[@]},
      "tests": [$(printf '"%s",' "${EXECUTIVE_TESTS[@]}" | sed 's/,$//')]
    },
    "parent_surface": {
      "total": ${#PARENT_TESTS[@]},
      "tests": [$(printf '"%s",' "${PARENT_TESTS[@]}" | sed 's/,$//')]
    }
  }
}
EOF

print_status "INFO" "JSON report saved to: $JSON_REPORT"

# Exit with appropriate code
if [ $FAILED_TESTS -gt 0 ]; then
    echo ""
    print_status "FAIL" "Some tests failed. Please review the report."
    exit 1
else
    echo ""
    print_status "PASS" "All tests passed successfully!"
    exit 0
fi
