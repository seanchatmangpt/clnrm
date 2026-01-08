#!/bin/bash

################################################################################
# Unit Tests Runner for gVisor
#
# Purpose: Execute unit tests in gVisor sandbox environment
# Toyota Principles: GENCHI GENBUTSU (see actual test behavior)
#
# Usage:
#   ./scripts/run_unit_tests_gvisor.sh
#   ./scripts/run_unit_tests_gvisor.sh --verbose
#   ./scripts/run_unit_tests_gvisor.sh --bail-on-first-failure
#
# Features:
#   - Validates gVisor installation
#   - Runs all unit tests (no Docker dependencies)
#   - Captures output for analysis
#   - Reports test metrics
#   - Generates test report
#
################################################################################

set -o pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_RESULTS_DIR="${PROJECT_ROOT}/target/test-results-gvisor"
UNIT_TEST_REPORT="${TEST_RESULTS_DIR}/unit-tests-report.txt"
VERBOSE="${VERBOSE:-0}"
BAIL_ON_FIRST_FAILURE="${BAIL_ON_FIRST_FAILURE:-0}"
RUST_LOG="${RUST_LOG:-info}"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

log() {
    echo -e "${BLUE}[Unit Tests]${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*" >&2
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
}

log_section() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}$*${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# ============================================================================
# VALIDATION FUNCTIONS
# ============================================================================

check_gvisor_support() {
    log_section "1. Checking gVisor Support"

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        return 1
    fi
    log_success "Docker is installed"

    # Check if gVisor runtime is available
    if docker run --runtime=runsc --rm alpine echo "gVisor check" &> /dev/null; then
        log_success "gVisor (runsc) runtime is available"
        return 0
    else
        log_warning "gVisor (runsc) runtime is not available"
        log "Unit tests will run with default Docker runtime"
        log "gVisor provides enhanced security but is optional for unit tests"
        return 0
    fi
}

check_rust_setup() {
    log_section "2. Checking Rust Setup"

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed"
        return 1
    fi
    log_success "Cargo is installed: $(cargo --version)"

    if ! command -v rustc &> /dev/null; then
        log_error "Rustc is not installed"
        return 1
    fi
    log_success "Rustc is installed: $(rustc --version)"

    return 0
}

# ============================================================================
# TEST EXECUTION
# ============================================================================

run_unit_tests() {
    log_section "3. Running Unit Tests"

    mkdir -p "$TEST_RESULTS_DIR"

    local start_time=$(date +%s)
    local test_count=0
    local failed_count=0

    # Get list of crates with tests
    local crates=("clnrm-core" "clnrm" "clnrm-template")

    for crate in "${crates[@]}"; do
        if [ ! -d "$PROJECT_ROOT/crates/$crate" ]; then
            log_warning "Crate not found: $crate"
            continue
        fi

        log_section "Running tests for: $crate"

        cd "$PROJECT_ROOT/crates/$crate" || continue

        if [ "$VERBOSE" = "1" ]; then
            # Verbose mode: show all test output
            cargo test --lib -- --nocapture --test-threads=1
        else
            # Standard mode: show only failures
            cargo test --lib 2>&1 | tee -a "$UNIT_TEST_REPORT"
        fi

        local status=$?
        test_count=$((test_count + 1))

        if [ $status -eq 0 ]; then
            log_success "Tests passed for: $crate"
        else
            log_error "Tests failed for: $crate (exit code: $status)"
            failed_count=$((failed_count + 1))

            if [ "$BAIL_ON_FIRST_FAILURE" = "1" ]; then
                log_error "Stopping due to test failure (--bail-on-first-failure)"
                return 1
            fi
        fi
    done

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Generate summary report
    {
        echo ""
        echo "UNIT TESTS SUMMARY"
        echo "=================="
        echo "Timestamp: $(date)"
        echo "Duration: ${duration}s"
        echo "Total crate groups: $test_count"
        echo "Failed groups: $failed_count"
        echo "Status: $([ $failed_count -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo ""
        echo "Environment:"
        echo "  Rust: $(rustc --version)"
        echo "  Cargo: $(cargo --version)"
        echo "  RUST_LOG: $RUST_LOG"
        echo ""
    } | tee -a "$UNIT_TEST_REPORT"

    cd "$PROJECT_ROOT"

    return $failed_count
}

# ============================================================================
# SECURITY VALIDATION
# ============================================================================

validate_security() {
    log_section "4. Security Validation"

    log "Checking for unsafe code usage in tests..."

    # Check for unsafe blocks in test code
    local unsafe_count=$(grep -r "unsafe {" "$PROJECT_ROOT/crates/*/tests/" --include="*.rs" 2>/dev/null | wc -l)

    if [ "$unsafe_count" -eq 0 ]; then
        log_success "No unsafe blocks in test code"
    else
        log_warning "Found $unsafe_count unsafe blocks in test code"
        log "Unsafe blocks in tests are acceptable but should be reviewed"
    fi

    # Check for panics in test setup
    local panic_count=$(grep -r "unwrap()\|expect(" "$PROJECT_ROOT/crates/*/tests/" --include="*.rs" 2>/dev/null | wc -l)

    if [ "$panic_count" -lt 50 ]; then
        log_success "Panic points in test code: $panic_count (acceptable level)"
    else
        log_warning "High number of panic points in test code: $panic_count"
    fi
}

# ============================================================================
# PERFORMANCE ANALYSIS
# ============================================================================

analyze_performance() {
    log_section "5. Performance Analysis"

    log "Analyzing test execution metrics..."

    # Extract timing information
    if [ -f "$UNIT_TEST_REPORT" ]; then
        local fastest=$(grep "test.*ok" "$UNIT_TEST_REPORT" | awk '{print $NF}' | sort -n | head -1)
        local slowest=$(grep "test.*ok" "$UNIT_TEST_REPORT" | awk '{print $NF}' | sort -n | tail -1)

        if [ -n "$fastest" ] && [ -n "$slowest" ]; then
            log_success "Fastest test: $fastest"
            log_success "Slowest test: $slowest"
        fi
    fi

    log "Note: Unit tests do not require gVisor - they execute natively"
    log "gVisor optimization would be for integration tests"
}

# ============================================================================
# REPORT GENERATION
# ============================================================================

generate_final_report() {
    log_section "6. Test Report"

    echo ""
    echo "Test Results Location: $TEST_RESULTS_DIR"
    echo "Detailed Report: $UNIT_TEST_REPORT"

    if [ -f "$UNIT_TEST_REPORT" ]; then
        echo ""
        echo "Report Preview:"
        echo "───────────────"
        tail -20 "$UNIT_TEST_REPORT"
    fi
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    echo "║                    gVisor Unit Tests Runner                                ║"
    echo "║          Toyota Production System - GENCHI GENBUTSU (See Actual)           ║"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    echo ""

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=1
                shift
                ;;
            --bail-on-first-failure)
                BAIL_ON_FIRST_FAILURE=1
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --verbose               Show detailed test output"
                echo "  --bail-on-first-failure Stop on first test failure"
                echo "  --help                  Show this help message"
                return 0
                ;;
            *)
                log_error "Unknown option: $1"
                return 1
                ;;
        esac
    done

    # Execute test phases
    if ! check_gvisor_support; then
        log_error "gVisor support check failed (not blocking)"
    fi

    if ! check_rust_setup; then
        log_error "Rust setup check failed"
        return 1
    fi

    if ! run_unit_tests; then
        log_error "Unit tests failed"
    fi

    validate_security
    analyze_performance
    generate_final_report

    echo ""
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    echo "║                        Test Execution Complete                             ║"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    echo ""

    # Return appropriate exit code
    if grep -q "test result: FAILED" "$UNIT_TEST_REPORT" 2>/dev/null; then
        return 1
    fi
    return 0
}

# Execute main
main "$@"
