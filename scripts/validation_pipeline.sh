#!/bin/bash
# Unified Validation Pipeline Orchestrator
# End-to-end Docker + OTLP + Weaver validation with error recovery

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
REGISTRY="${REGISTRY:-$PROJECT_ROOT/registry/}"
OUTPUT="${OUTPUT:-$PROJECT_ROOT/validation_output/}"
OTLP_PORT="${OTLP_PORT:-4317}"
ADMIN_PORT="${ADMIN_PORT:-8080}"
TIMEOUT="${TIMEOUT:-300}"
MAX_RETRIES="${MAX_RETRIES:-3}"
RETRY_DELAY="${RETRY_DELAY:-5}"

# Test configuration
TEST_PACKAGE="${TEST_PACKAGE:-clnrm-core}"
TEST_SUITE="${TEST_SUITE:-docker_integration}"
TEST_THREADS="${TEST_THREADS:-1}"

# State tracking
DOCKER_STARTED=false
WEAVER_STARTED=false
CLEANUP_ON_EXIT=true

# ========== FUNCTIONS ==========

log_header() {
    echo ""
    echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${MAGENTA}$1${NC}"
    echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

log_step() {
    echo -e "${CYAN}▶ $1${NC}"
}

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Cleanup function
cleanup() {
    local exit_code=$?

    if [[ "$CLEANUP_ON_EXIT" != "true" ]]; then
        log_info "Cleanup skipped (CLEANUP_ON_EXIT=false)"
        return
    fi

    log_header "Cleanup"

    # Stop Weaver
    if [[ "$WEAVER_STARTED" == "true" ]]; then
        log_step "Stopping Weaver..."
        "$SCRIPT_DIR/weaver_startup.sh" stop 2>/dev/null || true
    fi

    # Docker cleanup (containers only, not daemon)
    if [[ "$DOCKER_STARTED" == "true" ]]; then
        log_step "Cleaning up test containers..."
        docker ps -aq --filter "label=clnrm.test=true" 2>/dev/null | xargs -r docker rm -f 2>/dev/null || true
    fi

    log_success "Cleanup complete"

    if [[ $exit_code -ne 0 ]]; then
        echo ""
        log_error "Pipeline failed with exit code: $exit_code"
        echo ""
        echo "Troubleshooting:"
        echo "  • Check Weaver logs: $SCRIPT_DIR/weaver_startup.sh logs"
        echo "  • Check Docker: docker ps -a"
        echo "  • Review validation output: $OUTPUT"
        echo ""
    fi

    exit $exit_code
}

# Setup trap for cleanup
trap cleanup EXIT INT TERM

# Retry wrapper
retry() {
    local retries=$1
    shift
    local command=("$@")
    local attempt=1

    while [[ $attempt -le $retries ]]; do
        if "${command[@]}"; then
            return 0
        fi

        if [[ $attempt -lt $retries ]]; then
            log_warning "Attempt $attempt/$retries failed, retrying in ${RETRY_DELAY}s..."
            sleep "$RETRY_DELAY"
        fi

        attempt=$((attempt + 1))
    done

    log_error "All $retries attempts failed"
    return 1
}

# Phase 1: Docker startup
phase_docker_startup() {
    log_header "Phase 1: Docker Startup"

    log_step "Checking Docker daemon..."

    if docker ps >/dev/null 2>&1; then
        log_success "Docker daemon already running"
        DOCKER_STARTED=true
        return 0
    fi

    log_info "Docker daemon not running, attempting startup..."

    if ! retry 3 "$SCRIPT_DIR/docker_startup.sh"; then
        log_error "Failed to start Docker daemon"
        return 1
    fi

    DOCKER_STARTED=true
    log_success "Docker daemon ready"
}

# Phase 2: OTLP configuration
phase_otlp_config() {
    log_header "Phase 2: OTLP Configuration"

    log_step "Configuring OTLP environment..."

    # Source OTLP configuration
    if ! source <("$SCRIPT_DIR/otlp_config.sh" generate); then
        log_error "Failed to configure OTLP"
        return 1
    fi

    log_success "OTLP environment configured"

    # Validate configuration
    "$SCRIPT_DIR/otlp_config.sh" validate
}

# Phase 3: Weaver startup
phase_weaver_startup() {
    log_header "Phase 3: Weaver Startup"

    log_step "Starting Weaver live-check..."

    # Export configuration for Weaver script
    export REGISTRY OUTPUT OTLP_PORT ADMIN_PORT TIMEOUT

    if ! retry 3 "$SCRIPT_DIR/weaver_startup.sh" start; then
        log_error "Failed to start Weaver"
        return 1
    fi

    WEAVER_STARTED=true
    log_success "Weaver ready to receive telemetry"
}

# Phase 4: Run tests
phase_run_tests() {
    log_header "Phase 4: Test Execution"

    log_step "Running tests with OTLP export..."
    echo ""

    # Build test command
    local test_cmd=(
        cargo test
        -p "$TEST_PACKAGE"
        --test "$TEST_SUITE"
        --features otel
        --
        --test-threads="$TEST_THREADS"
    )

    log_info "Test command: ${test_cmd[*]}"
    echo ""

    # Run tests with retry
    local test_output="/tmp/clnrm_test_output.log"
    if ! "${test_cmd[@]}" 2>&1 | tee "$test_output"; then
        log_error "Tests failed"
        echo ""
        echo "Test output saved to: $test_output"
        return 1
    fi

    echo ""
    log_success "Tests passed"
}

# Phase 5: Generate report
phase_generate_report() {
    log_header "Phase 5: Report Generation"

    log_step "Stopping Weaver to generate report..."

    # Stop Weaver (triggers report generation)
    if ! "$SCRIPT_DIR/weaver_startup.sh" stop; then
        log_warning "Failed to stop Weaver gracefully"
    fi

    WEAVER_STARTED=false

    # Wait for report
    local report="$OUTPUT/live_check.json"
    local max_wait=10
    local elapsed=0

    while [[ $elapsed -lt $max_wait ]]; do
        if [[ -f "$report" ]]; then
            break
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done

    if [[ ! -f "$report" ]]; then
        log_error "Report not generated"
        return 1
    fi

    log_success "Report generated: $report"
}

# Phase 6: Validate report
phase_validate_report() {
    log_header "Phase 6: Report Validation"

    local report="$OUTPUT/live_check.json"

    if [[ ! -f "$report" ]]; then
        log_error "Report not found: $report"
        return 1
    fi

    log_step "Analyzing report..."
    echo ""

    # Extract statistics
    local samples=$(jq '.samples | length' "$report" 2>/dev/null || echo "0")
    local violations=$(jq '.statistics.advice_level_counts.violation // 0' "$report" 2>/dev/null || echo "0")
    local improvements=$(jq '.statistics.advice_level_counts.improvement // 0' "$report" 2>/dev/null || echo "0")
    local information=$(jq '.statistics.advice_level_counts.information // 0' "$report" 2>/dev/null || echo "0")
    local coverage=$(jq '.statistics.registry_coverage // 0' "$report" 2>/dev/null || echo "0")

    # Print statistics
    echo "Report Statistics:"
    echo "  Samples:      $samples"
    echo "  Violations:   $violations"
    echo "  Improvements: $improvements"
    echo "  Information:  $information"
    echo "  Coverage:     $coverage"
    echo ""

    # Validate results
    local failed=false

    # Check 1: Telemetry received
    if [[ "$samples" -eq 0 ]]; then
        log_error "No telemetry received"
        echo ""
        echo "Root causes:"
        echo "  1. Tests did not export OTLP telemetry"
        echo "  2. OTEL_EXPORTER_OTLP_ENDPOINT not configured"
        echo "  3. Network connection to Weaver failed"
        failed=true
    else
        log_success "Telemetry received: $samples samples"
    fi

    # Check 2: No violations
    if [[ "$violations" -gt 0 ]]; then
        log_error "Validation failed: $violations violations"
        echo ""
        echo "Violations indicate:"
        echo "  • Missing required telemetry attributes"
        echo "  • Schema non-compliance"
        echo "  • Potential false positives in testing"
        failed=true
    else
        log_success "Zero violations"
    fi

    # Check 3: Coverage threshold
    local min_coverage=0.70
    if (( $(echo "$coverage < $min_coverage" | bc -l 2>/dev/null || echo "0") )); then
        log_warning "Coverage below target: $coverage < $min_coverage"
        log_info "Consider adding more instrumentation"
    else
        log_success "Coverage meets target: $coverage >= $min_coverage"
    fi

    echo ""

    if [[ "$failed" == "true" ]]; then
        log_error "Report validation FAILED"
        echo ""
        echo "Review full report: $report"
        return 1
    fi

    log_success "Report validation PASSED"
}

# Final summary
print_summary() {
    log_header "Validation Summary"

    local report="$OUTPUT/live_check.json"

    if [[ -f "$report" ]]; then
        echo "Results:"
        jq '.statistics | {
            samples: (.total_entities // 0),
            violations: (.advice_level_counts.violation // 0),
            improvements: (.advice_level_counts.improvement // 0),
            information: (.advice_level_counts.information // 0),
            coverage: (.registry_coverage // 0)
        }' "$report"
        echo ""
    fi

    echo "Artifacts:"
    echo "  • Report:     $OUTPUT/live_check.json"
    echo "  • Logs:       /tmp/weaver.log"
    echo "  • Test logs:  /tmp/clnrm_test_output.log"
    echo ""

    log_success "Validation pipeline complete!"
}

# ========== MAIN LOGIC ==========

main() {
    local skip_phases=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --skip-docker)
                skip_phases="$skip_phases docker"
                shift
                ;;
            --skip-tests)
                skip_phases="$skip_phases tests"
                shift
                ;;
            --no-cleanup)
                CLEANUP_ON_EXIT=false
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --skip-docker    Skip Docker startup phase"
                echo "  --skip-tests     Skip test execution phase"
                echo "  --no-cleanup     Skip cleanup on exit"
                echo "  --help           Show this help"
                echo ""
                echo "Environment Variables:"
                echo "  REGISTRY         Registry path (default: $PROJECT_ROOT/registry/)"
                echo "  OUTPUT           Output directory (default: $PROJECT_ROOT/validation_output/)"
                echo "  OTLP_PORT        OTLP port (default: 4317)"
                echo "  TEST_PACKAGE     Cargo package to test (default: clnrm-core)"
                echo "  TEST_SUITE       Test suite name (default: docker_integration)"
                echo "  MAX_RETRIES      Max retry attempts (default: 3)"
                echo ""
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Run '$0 --help' for usage"
                exit 1
                ;;
        esac
    done

    log_header "clnrm v1.2.0 Validation Pipeline"

    echo "Configuration:"
    echo "  Registry:      $REGISTRY"
    echo "  Output:        $OUTPUT"
    echo "  OTLP Port:     $OTLP_PORT"
    echo "  Test Package:  $TEST_PACKAGE"
    echo "  Test Suite:    $TEST_SUITE"
    echo "  Max Retries:   $MAX_RETRIES"
    echo ""

    # Execute phases
    if [[ ! "$skip_phases" =~ "docker" ]]; then
        phase_docker_startup || exit 1
    fi

    phase_otlp_config || exit 1
    phase_weaver_startup || exit 1

    if [[ ! "$skip_phases" =~ "tests" ]]; then
        phase_run_tests || exit 1
    fi

    phase_generate_report || exit 1
    phase_validate_report || exit 1

    print_summary

    echo "================================================================================"
}

# ========== ENTRY POINT ==========

main "$@"
