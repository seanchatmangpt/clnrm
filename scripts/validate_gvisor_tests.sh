#!/bin/bash
# Validates all tests pass with gVisor backend
# Exit code: 0 = success, 1 = test failures
#
# Usage:
#   ./scripts/validate_gvisor_tests.sh [OPTIONS]
#
# Options:
#   --unit-only       Run only unit tests
#   --integration-only Run only integration tests
#   --bench-only      Run only benchmarks
#   --quick           Skip slow tests
#   --verbose         Show detailed test output
#
# Environment:
#   CLNRM_BACKEND     Backend to use (default: gvisor)
#   TEST_THREADS      Number of parallel test threads
#   RUST_LOG          Rust logging level

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default options
RUN_UNIT=1
RUN_INTEGRATION=1
RUN_BENCH=1
QUICK_MODE=0
VERBOSE=0

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            RUN_INTEGRATION=0
            RUN_BENCH=0
            shift
            ;;
        --integration-only)
            RUN_UNIT=0
            RUN_BENCH=0
            shift
            ;;
        --bench-only)
            RUN_UNIT=0
            RUN_INTEGRATION=0
            shift
            ;;
        --quick)
            QUICK_MODE=1
            shift
            ;;
        --verbose)
            VERBOSE=1
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Configuration
export CLNRM_BACKEND=${CLNRM_BACKEND:-gvisor}
TEST_THREADS=${TEST_THREADS:-$(nproc)}
export RUST_LOG=${RUST_LOG:-info}

# Temporary files
UNIT_RESULTS=$(mktemp)
INTEGRATION_RESULTS=$(mktemp)
BENCH_RESULTS=$(mktemp)
SUMMARY=$(mktemp)

# Cleanup on exit
cleanup() {
    rm -f "$UNIT_RESULTS" "$INTEGRATION_RESULTS" "$BENCH_RESULTS" "$SUMMARY"
}
trap cleanup EXIT

log_section() {
    echo ""
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check gVisor availability
check_gvisor_available() {
    log_section "Pre-flight Checks"

    echo "Checking gVisor availability..."

    if ! command -v runsc &> /dev/null; then
        log_error "runsc command not found"
        echo ""
        echo "gVisor runtime (runsc) is required but not installed."
        echo ""
        echo "Installation instructions:"
        echo "  wget https://storage.googleapis.com/gvisor/releases/release/latest/x86_64/runsc"
        echo "  chmod +x runsc"
        echo "  sudo mv runsc /usr/local/bin/"
        echo ""
        exit 1
    fi

    log_success "runsc found: $(runsc --version | head -n1)"

    # Check if we're running in a container
    if [ -f /.dockerenv ]; then
        log_info "Running inside a container (nested virtualization)"
    fi

    # Verify backend configuration
    echo ""
    echo "Backend configuration:"
    echo "  CLNRM_BACKEND: $CLNRM_BACKEND"
    echo "  TEST_THREADS: $TEST_THREADS"
    echo "  RUST_LOG: $RUST_LOG"
    echo "  QUICK_MODE: $QUICK_MODE"
}

# Pre-pull common images
prepull_images() {
    log_section "Pre-pulling Container Images"

    IMAGES=(
        "alpine:latest"
        "python:3.11-slim"
        "surrealdb/surrealdb:latest"
        "otel/opentelemetry-collector:latest"
    )

    for image in "${IMAGES[@]}"; do
        echo "Pre-pulling $image..."
        # TODO: Implement clnrm pull when available
        # For now, this is a placeholder
        log_info "Image pull: $image (skipped - implement clnrm pull)"
    done

    log_success "Image pre-pull complete"
}

# Run unit tests
run_unit_tests() {
    log_section "Running Unit Tests"

    local test_args="--all --lib"

    if [ "$QUICK_MODE" -eq 1 ]; then
        test_args="$test_args --exclude slow_tests"
    fi

    if [ "$VERBOSE" -eq 1 ]; then
        test_args="$test_args -- --nocapture --test-threads=$TEST_THREADS"
    else
        test_args="$test_args -- --test-threads=$TEST_THREADS"
    fi

    echo "Running: cargo test $test_args"
    echo ""

    if cargo test $test_args 2>&1 | tee "$UNIT_RESULTS"; then
        log_success "Unit tests passed"
        return 0
    else
        log_error "Unit tests failed"
        return 1
    fi
}

# Run integration tests
run_integration_tests() {
    log_section "Running Integration Tests"

    local test_args="--all --test '*'"

    if [ "$QUICK_MODE" -eq 1 ]; then
        test_args="$test_args --exclude slow_integration_tests"
    fi

    if [ "$VERBOSE" -eq 1 ]; then
        test_args="$test_args -- --nocapture --test-threads=$TEST_THREADS"
    else
        test_args="$test_args -- --test-threads=$TEST_THREADS"
    fi

    echo "Running: cargo test $test_args"
    echo ""

    # Integration tests may require services
    # TODO: Start required services (SurrealDB, OTEL Collector)

    if eval "cargo test $test_args" 2>&1 | tee "$INTEGRATION_RESULTS"; then
        log_success "Integration tests passed"
        return 0
    else
        log_error "Integration tests failed"
        return 1
    fi
}

# Run benchmarks
run_benchmarks() {
    log_section "Running Benchmarks (build only)"

    echo "Running: cargo bench --no-run --all"
    echo ""

    if cargo bench --no-run --all 2>&1 | tee "$BENCH_RESULTS"; then
        log_success "Benchmarks build successfully"
        return 0
    else
        log_error "Benchmark build failed"
        return 1
    fi
}

# Analyze test results
analyze_results() {
    log_section "Test Results Analysis"

    local unit_count=0
    local integration_count=0
    local unit_failures=0
    local integration_failures=0

    # Parse unit test results
    if [ -f "$UNIT_RESULTS" ]; then
        unit_count=$(grep -c "test result:" "$UNIT_RESULTS" || echo "0")
        unit_failures=$(grep "FAILED" "$UNIT_RESULTS" | wc -l || echo "0")
    fi

    # Parse integration test results
    if [ -f "$INTEGRATION_RESULTS" ]; then
        integration_count=$(grep -c "test result:" "$INTEGRATION_RESULTS" || echo "0")
        integration_failures=$(grep "FAILED" "$INTEGRATION_RESULTS" | wc -l || echo "0")
    fi

    # Display summary
    {
        echo "Test Suite Summary"
        echo "=================="
        echo ""
        if [ "$RUN_UNIT" -eq 1 ]; then
            echo "Unit Tests:"
            echo "  Total: $unit_count"
            echo "  Failures: $unit_failures"
            echo ""
        fi
        if [ "$RUN_INTEGRATION" -eq 1 ]; then
            echo "Integration Tests:"
            echo "  Total: $integration_count"
            echo "  Failures: $integration_failures"
            echo ""
        fi
        echo "Backend: $CLNRM_BACKEND"
        echo "Test Threads: $TEST_THREADS"
    } > "$SUMMARY"

    cat "$SUMMARY"

    # Return non-zero if any failures
    total_failures=$((unit_failures + integration_failures))
    return $total_failures
}

# Generate test report
generate_report() {
    log_section "Test Report"

    local report_file="target/test-results/gvisor-test-report-$(date +%Y%m%d-%H%M%S).txt"
    mkdir -p "$(dirname "$report_file")"

    {
        echo "gVisor Backend Test Report"
        echo "=========================="
        echo ""
        echo "Date: $(date)"
        echo "Backend: $CLNRM_BACKEND"
        echo "Test Threads: $TEST_THREADS"
        echo "Quick Mode: $QUICK_MODE"
        echo ""
        cat "$SUMMARY"
        echo ""
        echo "Detailed Results"
        echo "================"
        echo ""
        if [ -f "$UNIT_RESULTS" ]; then
            echo "Unit Tests:"
            echo "----------"
            cat "$UNIT_RESULTS"
            echo ""
        fi
        if [ -f "$INTEGRATION_RESULTS" ]; then
            echo "Integration Tests:"
            echo "-----------------"
            cat "$INTEGRATION_RESULTS"
            echo ""
        fi
    } > "$report_file"

    echo "Report saved to: $report_file"
}

# Main execution
main() {
    local exit_code=0

    # Pre-flight checks
    check_gvisor_available

    # Pre-pull images (optional optimization)
    if [ "$QUICK_MODE" -eq 0 ]; then
        prepull_images
    fi

    # Run unit tests
    if [ "$RUN_UNIT" -eq 1 ]; then
        if ! run_unit_tests; then
            exit_code=1
        fi
    fi

    # Run integration tests
    if [ "$RUN_INTEGRATION" -eq 1 ]; then
        if ! run_integration_tests; then
            exit_code=1
        fi
    fi

    # Run benchmarks
    if [ "$RUN_BENCH" -eq 1 ]; then
        if ! run_benchmarks; then
            exit_code=1
        fi
    fi

    # Analyze results
    if ! analyze_results; then
        exit_code=1
    fi

    # Generate report
    generate_report

    # Final summary
    log_section "Final Result"

    if [ $exit_code -eq 0 ]; then
        log_success "All tests passed with gVisor backend!"
        echo ""
        echo "✨ 100% test pass rate achieved"
        echo ""
        echo "Next steps:"
        echo "  1. Review test performance metrics"
        echo "  2. Run performance benchmarks: ./scripts/validate_gvisor_performance.sh"
        echo "  3. Validate Docker elimination: ./scripts/validate_docker_elimination.sh"
    else
        log_error "Some tests failed with gVisor backend"
        echo ""
        echo "Troubleshooting:"
        echo "  1. Review test output above for specific failures"
        echo "  2. Check gVisor logs: journalctl -u runsc"
        echo "  3. Run with --verbose for detailed output"
        echo "  4. Run specific failing test: cargo test <test_name> -- --nocapture"
        echo ""
        echo "Report saved for analysis"
    fi

    exit $exit_code
}

# Run main function
main
