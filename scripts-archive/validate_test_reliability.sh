#!/usr/bin/env bash
# False Positive Validator - Test Reliability Validation Script
# This script runs tests multiple times to detect flakiness and validates test isolation

set -euo pipefail

# Configuration
ITERATIONS=${1:-100}
TEST_PATTERN=${2:-""}
RESULTS_DIR="./tests/validation_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${RESULTS_DIR}/validation_report_${TIMESTAMP}.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "${RESULTS_DIR}"

echo "========================================" | tee -a "${REPORT_FILE}"
echo "False Positive Validation Report" | tee -a "${REPORT_FILE}"
echo "Generated: $(date)" | tee -a "${REPORT_FILE}"
echo "Iterations: ${ITERATIONS}" | tee -a "${REPORT_FILE}"
echo "========================================" | tee -a "${REPORT_FILE}"
echo "" | tee -a "${REPORT_FILE}"

# Function to run tests N times and collect results
run_flakiness_detection() {
    local test_name=$1
    local pass_count=0
    local fail_count=0
    local duration_sum=0
    local min_duration=999999
    local max_duration=0

    echo "Testing: ${test_name}" | tee -a "${REPORT_FILE}"
    echo "Running ${ITERATIONS} iterations..." | tee -a "${REPORT_FILE}"

    for i in $(seq 1 "${ITERATIONS}"); do
        local start_time=$(date +%s%N)

        if cargo test "${test_name}" --lib --quiet 2>&1 > /dev/null; then
            ((pass_count++))
        else
            ((fail_count++))
        fi

        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 )) # Convert to ms

        duration_sum=$((duration_sum + duration))

        if [ ${duration} -lt ${min_duration} ]; then
            min_duration=${duration}
        fi

        if [ ${duration} -gt ${max_duration} ]; then
            max_duration=${duration}
        fi

        # Progress indicator
        if [ $((i % 10)) -eq 0 ]; then
            echo -n "." >&2
        fi
    done

    echo "" >&2

    local avg_duration=$((duration_sum / ITERATIONS))
    local success_rate=$((pass_count * 100 / ITERATIONS))
    local flakiness_score=$((fail_count * 100 / ITERATIONS))

    # Report results
    echo "  Results:" | tee -a "${REPORT_FILE}"
    echo "    - Passed: ${pass_count}/${ITERATIONS}" | tee -a "${REPORT_FILE}"
    echo "    - Failed: ${fail_count}/${ITERATIONS}" | tee -a "${REPORT_FILE}"
    echo "    - Success Rate: ${success_rate}%" | tee -a "${REPORT_FILE}"
    echo "    - Flakiness Score: ${flakiness_score}%" | tee -a "${REPORT_FILE}"
    echo "    - Avg Duration: ${avg_duration}ms" | tee -a "${REPORT_FILE}"
    echo "    - Min Duration: ${min_duration}ms" | tee -a "${REPORT_FILE}"
    echo "    - Max Duration: ${max_duration}ms" | tee -a "${REPORT_FILE}"
    echo "    - Duration Variance: $((max_duration - min_duration))ms" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"

    # Classify test stability
    if [ ${flakiness_score} -eq 0 ]; then
        echo -e "  ${GREEN}✓ STABLE${NC} - No flakiness detected" | tee -a "${REPORT_FILE}"
    elif [ ${flakiness_score} -lt 5 ]; then
        echo -e "  ${YELLOW}⚠ LOW FLAKINESS${NC} - Minor instability detected" | tee -a "${REPORT_FILE}"
    elif [ ${flakiness_score} -lt 20 ]; then
        echo -e "  ${YELLOW}⚠ MODERATE FLAKINESS${NC} - Significant instability" | tee -a "${REPORT_FILE}"
    else
        echo -e "  ${RED}✗ HIGH FLAKINESS${NC} - Critical instability" | tee -a "${REPORT_FILE}"
    fi
    echo "" | tee -a "${REPORT_FILE}"

    # Timing analysis
    local timing_variance=$((max_duration - min_duration))
    local timing_variance_pct=$((timing_variance * 100 / avg_duration))

    if [ ${timing_variance_pct} -gt 200 ]; then
        echo "  ⚠ WARNING: High timing variance (${timing_variance_pct}%) - possible timing-dependent behavior" | tee -a "${REPORT_FILE}"
    fi
    echo "" | tee -a "${REPORT_FILE}"

    return ${flakiness_score}
}

# Function to test isolation by running tests in parallel
test_isolation() {
    echo "## Test Isolation Validation" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"
    echo "Running tests in parallel to verify isolation..." | tee -a "${REPORT_FILE}"

    local parallel_runs=10
    local success_count=0

    for i in $(seq 1 ${parallel_runs}); do
        if cargo test --lib --jobs 4 --quiet 2>&1 > /dev/null; then
            ((success_count++))
        fi
    done

    local parallel_success_rate=$((success_count * 100 / parallel_runs))

    echo "  Parallel Execution Results:" | tee -a "${REPORT_FILE}"
    echo "    - Runs: ${parallel_runs}" | tee -a "${REPORT_FILE}"
    echo "    - Success: ${success_count}/${parallel_runs}" | tee -a "${REPORT_FILE}"
    echo "    - Success Rate: ${parallel_success_rate}%" | tee -a "${REPORT_FILE}"

    if [ ${parallel_success_rate} -eq 100 ]; then
        echo -e "  ${GREEN}✓ ISOLATED${NC} - Tests are properly isolated" | tee -a "${REPORT_FILE}"
    else
        echo -e "  ${RED}✗ ISOLATION ISSUES${NC} - Tests may have interdependencies" | tee -a "${REPORT_FILE}"
    fi
    echo "" | tee -a "${REPORT_FILE}"
}

# Function to validate cleanup and teardown
test_cleanup_validation() {
    echo "## Cleanup and Teardown Validation" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"

    # Run tests and check for resource leaks
    echo "Checking for resource leaks..." | tee -a "${REPORT_FILE}"

    local before_docker=$(docker ps -q | wc -l)
    cargo test --lib --quiet 2>&1 > /dev/null || true
    sleep 2
    local after_docker=$(docker ps -q | wc -l)

    local container_leak=$((after_docker - before_docker))

    echo "  Docker Containers:" | tee -a "${REPORT_FILE}"
    echo "    - Before: ${before_docker}" | tee -a "${REPORT_FILE}"
    echo "    - After: ${after_docker}" | tee -a "${REPORT_FILE}"
    echo "    - Leaked: ${container_leak}" | tee -a "${REPORT_FILE}"

    if [ ${container_leak} -eq 0 ]; then
        echo -e "  ${GREEN}✓ CLEAN${NC} - No resource leaks detected" | tee -a "${REPORT_FILE}"
    else
        echo -e "  ${YELLOW}⚠ POSSIBLE LEAK${NC} - ${container_leak} containers may not be cleaned up" | tee -a "${REPORT_FILE}"
    fi
    echo "" | tee -a "${REPORT_FILE}"
}

# Main validation workflow
main() {
    echo "Starting False Positive Validation..."
    echo ""

    # Get list of all tests
    echo "Discovering tests..." | tee -a "${REPORT_FILE}"
    local test_list=$(cargo test --lib --no-run 2>&1 | grep "test result:" -B 10000 | grep "test " | awk '{print $2}' | sort -u)
    local test_count=$(echo "${test_list}" | wc -l)

    echo "Found ${test_count} tests" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"

    # Flakiness detection
    echo "## Flakiness Detection (${ITERATIONS} iterations)" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"

    local flaky_tests=0
    local stable_tests=0

    while IFS= read -r test_name; do
        if [ -n "${test_name}" ]; then
            run_flakiness_detection "${test_name}" || true
            local flakiness=$?

            if [ ${flakiness} -gt 0 ]; then
                ((flaky_tests++))
            else
                ((stable_tests++))
            fi
        fi
    done <<< "${test_list}"

    # Test isolation
    test_isolation

    # Cleanup validation
    test_cleanup_validation

    # Summary
    echo "## Validation Summary" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"
    echo "  - Total Tests: ${test_count}" | tee -a "${REPORT_FILE}"
    echo "  - Stable Tests: ${stable_tests}" | tee -a "${REPORT_FILE}"
    echo "  - Flaky Tests: ${flaky_tests}" | tee -a "${REPORT_FILE}"
    echo "  - Overall Stability: $((stable_tests * 100 / test_count))%" | tee -a "${REPORT_FILE}"
    echo "" | tee -a "${REPORT_FILE}"

    if [ ${flaky_tests} -eq 0 ]; then
        echo -e "${GREEN}✓ ALL TESTS STABLE${NC}" | tee -a "${REPORT_FILE}"
        echo "" | tee -a "${REPORT_FILE}"
        echo "Report saved to: ${REPORT_FILE}"
        exit 0
    else
        echo -e "${RED}✗ FLAKY TESTS DETECTED${NC}" | tee -a "${REPORT_FILE}"
        echo "" | tee -a "${REPORT_FILE}"
        echo "Report saved to: ${REPORT_FILE}"
        exit 1
    fi
}

# Run main workflow
main "$@"
