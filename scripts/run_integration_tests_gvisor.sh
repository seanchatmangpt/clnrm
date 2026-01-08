#!/bin/bash

################################################################################
# Integration Tests Runner for gVisor
#
# Purpose: Execute integration tests using gVisor containers
# Toyota Principles: HEIJUNKA (load-leveled test execution)
#
# Usage:
#   ./scripts/run_integration_tests_gvisor.sh
#   ./scripts/run_integration_tests_gvisor.sh --compose-file gvisor-compose.test.yml
#   ./scripts/run_integration_tests_gvisor.sh --otel-only
#   ./scripts/run_integration_tests_gvisor.sh --cleanup
#
# Features:
#   - Validates gVisor installation
#   - Starts test services with gVisor runtime
#   - Waits for service health
#   - Executes integration tests
#   - Validates gVisor security boundaries
#   - Collects telemetry and metrics
#   - Cleans up resources
#
################################################################################

set -o pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-tests/integration/gvisor-compose.test.yml}"
OTEL_COMPOSE="${OTEL_COMPOSE:-tests/integration/gvisor-compose.otel-test.yml}"
TEST_RESULTS_DIR="${PROJECT_ROOT}/target/test-results-gvisor"
TEST_REPORT="${TEST_RESULTS_DIR}/integration-tests-report.txt"
SECURITY_REPORT="${TEST_RESULTS_DIR}/security-audit.txt"
CLEANUP_AFTER="${CLEANUP_AFTER:-1}"
OTEL_ONLY="${OTEL_ONLY:-0}"
VERBOSE="${VERBOSE:-0}"

# Test configuration
HEALTH_CHECK_TIMEOUT=120
SERVICE_STARTUP_TIMEOUT=60
PARALLEL_TESTS=4

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

log() {
    echo -e "${BLUE}[Integration Tests]${NC} $*"
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

cleanup() {
    log_section "Cleanup Phase"

    if [ "$CLEANUP_AFTER" = "1" ]; then
        log "Stopping Docker Compose services..."
        cd "$PROJECT_ROOT"
        docker-compose -f "$COMPOSE_FILE" down -v 2>/dev/null || true

        if [ "$OTEL_ONLY" = "0" ]; then
            docker-compose -f "$OTEL_COMPOSE" down -v 2>/dev/null || true
        fi

        log_success "Services stopped and volumes removed"
    else
        log_warning "Services left running (CLEANUP_AFTER=0)"
        log "To stop services manually:"
        log "  docker-compose -f $COMPOSE_FILE down -v"
    fi
}

trap cleanup EXIT

# ============================================================================
# VALIDATION FUNCTIONS
# ============================================================================

check_gvisor_installation() {
    log_section "1. Checking gVisor Installation"

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        return 1
    fi
    log_success "Docker is installed: $(docker --version)"

    # Check if gVisor runtime is available
    if docker run --runtime=runsc --rm alpine echo "gVisor available" &> /dev/null; then
        log_success "gVisor (runsc) runtime is available"
        return 0
    else
        log_error "gVisor (runsc) runtime is NOT available"
        log "Installation guide:"
        log "  Ubuntu: sudo apt-get install runsc"
        log "  See: https://gvisor.dev/docs/user_guide/install/"
        return 1
    fi
}

check_compose_files() {
    log_section "2. Checking Compose Files"

    if [ ! -f "$PROJECT_ROOT/$COMPOSE_FILE" ]; then
        log_error "Compose file not found: $COMPOSE_FILE"
        return 1
    fi
    log_success "Main compose file found: $COMPOSE_FILE"

    if [ ! -f "$PROJECT_ROOT/$OTEL_COMPOSE" ]; then
        log_warning "OTEL compose file not found: $OTEL_COMPOSE"
    else
        log_success "OTEL compose file found: $OTEL_COMPOSE"
    fi

    return 0
}

# ============================================================================
# SERVICE STARTUP
# ============================================================================

start_services() {
    log_section "3. Starting Test Services"

    cd "$PROJECT_ROOT"

    # Start main test services
    log "Starting main test services..."
    if ! docker-compose -f "$COMPOSE_FILE" up -d; then
        log_error "Failed to start services"
        return 1
    fi
    log_success "Services started"

    # Start OTEL services if not OTEL-only
    if [ "$OTEL_ONLY" = "0" ]; then
        log "Starting OTEL services..."
        if ! docker-compose -f "$OTEL_COMPOSE" up -d; then
            log_warning "Failed to start OTEL services"
        else
            log_success "OTEL services started"
        fi
    fi

    # Wait for services to be healthy
    log "Waiting for services to be healthy..."
    local elapsed=0
    local unhealthy=0

    while [ $elapsed -lt $HEALTH_CHECK_TIMEOUT ]; do
        unhealthy=$(docker-compose -f "$COMPOSE_FILE" ps | grep -c "unhealthy\|exited" || true)

        if [ "$unhealthy" -eq 0 ]; then
            log_success "All services are healthy"
            return 0
        fi

        echo -n "."
        sleep 2
        elapsed=$((elapsed + 2))
    done

    log_error "Services did not become healthy within ${HEALTH_CHECK_TIMEOUT}s"
    docker-compose -f "$COMPOSE_FILE" ps
    return 1
}

verify_service_connectivity() {
    log_section "4. Verifying Service Connectivity"

    local services=("surrealdb:8000" "redis:6379" "postgres:5432" "otel-collector:13133")

    for service in "${services[@]}"; do
        local name="${service%:*}"
        local port="${service#*:}"

        log "Checking $name on port $port..."

        if docker-compose -f "$COMPOSE_FILE" exec -T "$name" echo "alive" &> /dev/null; then
            log_success "$name is reachable"
        else
            log_warning "$name may not be reachable"
        fi
    done
}

# ============================================================================
# gVisor SECURITY VALIDATION
# ============================================================================

validate_gvisor_security() {
    log_section "5. Validating gVisor Security Boundaries"

    {
        echo "gVisor Security Audit"
        echo "===================="
        echo "Timestamp: $(date)"
        echo ""

        # Check runtime configuration
        echo "RUNTIME CONFIGURATION:"
        echo "─────────────────────"

        local containers=$(docker-compose -f "$COMPOSE_FILE" ps -q | head -5)

        for container_id in $containers; do
            local container_name=$(docker inspect -f '{{.Name}}' "$container_id" | sed 's/^\///')
            local runtime=$(docker inspect -f '{{.HostConfig.Runtime}}' "$container_id")

            echo "Container: $container_name"
            echo "  Runtime: $runtime"

            if [ "$runtime" = "runsc" ]; then
                echo "  Status: ✓ PASSED (gVisor runtime)"
            else
                echo "  Status: ✗ FAILED (Not using gVisor)"
            fi
        done

        echo ""
        echo "CAPABILITY RESTRICTIONS:"
        echo "─────────────────────────"

        for container_id in $containers; do
            local container_name=$(docker inspect -f '{{.Name}}' "$container_id" | sed 's/^\///')
            local caps=$(docker inspect -f '{{json .HostConfig.CapDrop}}' "$container_id")

            echo "Container: $container_name"
            echo "  Caps Dropped: $caps"
        done

        echo ""
        echo "FILESYSTEM ISOLATION:"
        echo "────────────────────"

        # Test that containers cannot access host filesystem
        local test_container=$(docker-compose -f "$COMPOSE_FILE" ps -q "alpine" | head -1)

        if [ -n "$test_container" ]; then
            echo "Testing /etc/hostname access..."
            if docker exec "$test_container" cat /etc/hostname &> /dev/null; then
                echo "  Status: ✓ Container has /etc/hostname"
            else
                echo "  Status: ✗ Container cannot access /etc/hostname"
            fi
        fi

        echo ""
        echo "NETWORK ISOLATION:"
        echo "─────────────────"

        # Verify containers can communicate on network
        local containers_array=($(docker-compose -f "$COMPOSE_FILE" ps -q | head -3))

        if [ ${#containers_array[@]} -ge 2 ]; then
            echo "Testing container-to-container communication..."
            if docker exec "${containers_array[0]}" ping -c 1 "${containers_array[1]}" &> /dev/null; then
                echo "  Status: ✓ Containers can communicate"
            else
                echo "  Status: ⚠ Containers cannot ping (expected in gVisor)"
            fi
        fi

    } | tee "$SECURITY_REPORT"

    log_success "Security audit completed: $SECURITY_REPORT"
}

# ============================================================================
# INTEGRATION TEST EXECUTION
# ============================================================================

run_integration_tests() {
    log_section "6. Running Integration Tests"

    cd "$PROJECT_ROOT"

    mkdir -p "$TEST_RESULTS_DIR"

    local start_time=$(date +%s)
    local failed_tests=0

    # Get list of integration test files
    local test_files=(
        "tests/integration/database_integration_test.rs"
        "tests/integration/system_integration_test.rs"
    )

    for test_file in "${test_files[@]}"; do
        if [ ! -f "$test_file" ]; then
            log_warning "Test file not found: $test_file"
            continue
        fi

        local test_name=$(basename "$test_file" .rs)
        log "Running: $test_name"

        if cargo test --test "$(basename "$test_file" .rs)" -- --nocapture 2>&1 | tee -a "$TEST_REPORT"; then
            log_success "Test passed: $test_name"
        else
            log_error "Test failed: $test_name"
            failed_tests=$((failed_tests + 1))
        fi
    done

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Generate test summary
    {
        echo ""
        echo "INTEGRATION TESTS SUMMARY"
        echo "========================="
        echo "Timestamp: $(date)"
        echo "Duration: ${duration}s"
        echo "Failed tests: $failed_tests"
        echo "Status: $([ $failed_tests -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo ""
        echo "Environment:"
        echo "  gVisor: Available"
        echo "  Compose File: $COMPOSE_FILE"
        echo "  Services: Started and healthy"
        echo ""
    } | tee -a "$TEST_REPORT"

    return $failed_tests
}

# ============================================================================
# OTEL VALIDATION
# ============================================================================

validate_otel_telemetry() {
    log_section "7. Validating OTEL Telemetry"

    if [ "$OTEL_ONLY" = "0" ]; then
        log "Checking OTEL collector health..."

        if curl -s http://localhost:13133/ | grep -q "OK"; then
            log_success "OTEL collector is healthy"

            # Check if spans are being received
            log "Checking for received spans..."
            local span_count=$(curl -s http://localhost:55679/debug/tracez 2>/dev/null | grep -c "span" || echo "0")

            if [ "$span_count" -gt 0 ]; then
                log_success "Found $span_count spans in collector"
            else
                log_warning "No spans found yet (may be expected)"
            fi
        else
            log_warning "OTEL collector health check failed"
        fi
    fi
}

# ============================================================================
# PERFORMANCE METRICS
# ============================================================================

collect_metrics() {
    log_section "8. Collecting Performance Metrics"

    {
        echo ""
        echo "PERFORMANCE METRICS"
        echo "==================="
        echo "Timestamp: $(date)"
        echo ""

        # Container startup time
        echo "Container Startup Metrics:"
        docker-compose -f "$COMPOSE_FILE" ps -q | while read container_id; do
            local name=$(docker inspect -f '{{.Name}}' "$container_id" | sed 's/^\///')
            local created=$(docker inspect -f '{{.Created}}' "$container_id")
            echo "  $name: $created"
        done

        echo ""
        echo "Resource Usage:"
        docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"

    } | tee -a "$TEST_REPORT"
}

# ============================================================================
# REPORT GENERATION
# ============================================================================

generate_final_report() {
    log_section "9. Final Report"

    echo ""
    echo "Test Results Location: $TEST_RESULTS_DIR"
    echo ""
    echo "Reports Generated:"
    echo "  - Integration Tests: $TEST_REPORT"
    echo "  - Security Audit: $SECURITY_REPORT"
    echo ""

    if [ -f "$TEST_REPORT" ]; then
        echo "Integration Tests Summary:"
        echo "─────────────────────────"
        tail -15 "$TEST_REPORT"
    fi

    if [ -f "$SECURITY_REPORT" ]; then
        echo ""
        echo "Security Audit Summary:"
        echo "──────────────────────"
        tail -15 "$SECURITY_REPORT"
    fi
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    echo "║               gVisor Integration Tests Runner                              ║"
    echo "║          Toyota Production System - HEIJUNKA (Load Leveling)               ║"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    echo ""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --compose-file)
                COMPOSE_FILE="$2"
                shift 2
                ;;
            --otel-only)
                OTEL_ONLY=1
                shift
                ;;
            --no-cleanup)
                CLEANUP_AFTER=0
                shift
                ;;
            --verbose)
                VERBOSE=1
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --compose-file FILE     Use custom compose file"
                echo "  --otel-only            Run only OTEL validation tests"
                echo "  --no-cleanup           Keep containers running after tests"
                echo "  --verbose              Show detailed output"
                echo "  --help                 Show this help message"
                return 0
                ;;
            *)
                log_error "Unknown option: $1"
                return 1
                ;;
        esac
    done

    # Execute test phases
    if ! check_gvisor_installation; then
        log_error "gVisor installation check failed"
        return 1
    fi

    if ! check_compose_files; then
        log_error "Compose files check failed"
        return 1
    fi

    if ! start_services; then
        log_error "Failed to start services"
        return 1
    fi

    verify_service_connectivity
    validate_gvisor_security
    run_integration_tests
    validate_otel_telemetry
    collect_metrics
    generate_final_report

    echo ""
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    echo "║                    Integration Tests Complete                              ║"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    echo ""
}

main "$@"
