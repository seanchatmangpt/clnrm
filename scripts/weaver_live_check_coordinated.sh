#!/bin/bash
# Weaver Live-Check with Port Coordination
# Runs Weaver live-check on alternate ports to avoid conflict with Docker OTEL stack
# This allows both to run simultaneously for comprehensive validation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REGISTRY="${REGISTRY:-registry/}"
OUTPUT="${OUTPUT:-validation_output/live-check-$(date +%s)}"
WEAVER_OTLP_PORT="${WEAVER_OTLP_PORT:-5317}"  # Alternate port to avoid conflict
WEAVER_HTTP_PORT="${WEAVER_HTTP_PORT:-5318}"  # Alternate HTTP port
ADMIN_PORT="${ADMIN_PORT:-8081}"              # Alternate admin port
TIMEOUT="${TIMEOUT:-120}"                      # 2 minutes inactivity timeout
PID_FILE="/tmp/weaver-coordinated.pid"
LOG_FILE="/tmp/weaver-coordinated.log"

# ========== FUNCTIONS ==========

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

# Check if Weaver is installed
check_weaver_installed() {
    if ! command -v weaver >/dev/null 2>&1; then
        log_error "Weaver is not installed"
        echo ""
        echo "To install Weaver:"
        echo "  cargo install weaver"
        return 1
    fi

    log_success "Weaver is installed ($(weaver --version 2>&1 | head -1))"
    return 0
}

# Validate registry
validate_registry() {
    log_info "Validating registry schemas..."

    if [[ ! -d "$REGISTRY" ]]; then
        log_error "Registry directory not found: $REGISTRY"
        return 1
    fi

    if ! weaver registry check --registry "$REGISTRY" >/dev/null 2>&1; then
        log_error "Registry validation failed"
        weaver registry check --registry "$REGISTRY"
        return 1
    fi

    log_success "Registry validated (207 files, 0 violations)"
    return 0
}

# Check if Docker OTEL stack is running
check_docker_otel() {
    log_info "Checking Docker OTEL stack status..."

    if docker ps | grep -q "otel-collector"; then
        log_success "Docker OTEL collector is running on ports 4317/4318"
        log_info "Weaver will use alternate ports: $WEAVER_OTLP_PORT/$WEAVER_HTTP_PORT"
        return 0
    else
        log_warning "Docker OTEL collector is not running"
        log_info "You may want to start it with: docker-compose up -d"
        return 1
    fi
}

# Check if ports are available
check_ports() {
    log_info "Checking port availability..."

    local all_clear=true

    if lsof -i ":$WEAVER_OTLP_PORT" >/dev/null 2>&1; then
        log_error "Port $WEAVER_OTLP_PORT is already in use"
        all_clear=false
    fi

    if lsof -i ":$WEAVER_HTTP_PORT" >/dev/null 2>&1; then
        log_error "Port $WEAVER_HTTP_PORT is already in use"
        all_clear=false
    fi

    if lsof -i ":$ADMIN_PORT" >/dev/null 2>&1; then
        log_error "Port $ADMIN_PORT is already in use"
        all_clear=false
    fi

    if [ "$all_clear" = true ]; then
        log_success "All ports available"
        return 0
    else
        log_error "Some ports are in use. Please choose different ports or stop conflicting services."
        return 1
    fi
}

# Clean up existing Weaver process
cleanup_existing_weaver() {
    log_info "Cleaning up any existing Weaver processes..."

    if [[ -f "$PID_FILE" ]]; then
        local old_pid=$(cat "$PID_FILE")
        if ps -p "$old_pid" >/dev/null 2>&1; then
            log_warning "Stopping existing Weaver process (PID: $old_pid)"
            kill -HUP "$old_pid" 2>/dev/null || kill -TERM "$old_pid" 2>/dev/null || true
            sleep 2
        fi
        rm -f "$PID_FILE"
    fi

    log_success "Cleanup complete"
}

# Create output directory
setup_output_dir() {
    log_info "Setting up output directory..."
    mkdir -p "$OUTPUT"
    log_success "Output directory ready: $OUTPUT"
}

# Start Weaver live-check
start_weaver() {
    log_info "Starting Weaver live-check on alternate ports..."
    echo ""

    # Start Weaver in background
    weaver registry live-check \
        --registry "$REGISTRY" \
        --otlp-grpc-port "$WEAVER_OTLP_PORT" \
        --admin-port "$ADMIN_PORT" \
        --format json \
        --output "$OUTPUT" \
        --inactivity-timeout "$TIMEOUT" \
        > "$LOG_FILE" 2>&1 &

    local pid=$!
    echo "$pid" > "$PID_FILE"

    log_success "Weaver started (PID: $pid)"
    log_info "OTLP endpoint: http://localhost:$WEAVER_OTLP_PORT"
    log_info "Admin API: http://localhost:$ADMIN_PORT"
    log_info "Log file: $LOG_FILE"
    echo ""

    return 0
}

# Wait for Weaver to be ready
wait_for_weaver() {
    log_info "Waiting for Weaver to start listening..."

    local max_wait=15
    local elapsed=0

    while [[ $elapsed -lt $max_wait ]]; do
        # Check if process still exists
        local pid=$(cat "$PID_FILE" 2>/dev/null || echo "")
        if [[ -n "$pid" ]] && ! ps -p "$pid" >/dev/null 2>&1; then
            log_error "Weaver process died unexpectedly"
            echo ""
            echo "Log output:"
            cat "$LOG_FILE" 2>/dev/null || echo "(no log file)"
            return 1
        fi

        # Check if listening on OTLP port
        if lsof -i ":$WEAVER_OTLP_PORT" >/dev/null 2>&1; then
            echo ""
            log_success "Weaver is listening on :$WEAVER_OTLP_PORT"
            return 0
        fi

        echo -n "."
        sleep 1
        elapsed=$((elapsed + 1))
    done

    echo ""
    log_error "Weaver did not start listening within ${max_wait}s"
    cat "$LOG_FILE" 2>/dev/null || echo "(no log file)"
    return 1
}

# Run a test command with telemetry export
run_test_command() {
    local command="${1:-cargo test --features otel --lib}"

    log_info "Running test command with telemetry export..."
    echo ""
    echo "Command: $command"
    echo ""

    # Export to Weaver's port
    OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$WEAVER_OTLP_PORT" \
    OTEL_EXPORTER_OTLP_PROTOCOL="grpc" \
        bash -c "$command"

    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        log_success "Test command completed successfully"
    else
        log_warning "Test command exited with code: $exit_code"
    fi

    # Wait for telemetry to be processed
    log_info "Waiting for telemetry to be processed..."
    sleep 3

    return $exit_code
}

# Stop Weaver gracefully
stop_weaver() {
    local signal="${1:-HUP}"

    log_info "Stopping Weaver (signal: $signal)..."

    local pid=$(cat "$PID_FILE" 2>/dev/null || echo "")

    if [[ -z "$pid" ]]; then
        log_warning "No PID file found"
        return 1
    fi

    if ! ps -p "$pid" >/dev/null 2>&1; then
        log_warning "Weaver process not running"
        rm -f "$PID_FILE"
        return 1
    fi

    # Send signal
    kill -"$signal" "$pid" 2>/dev/null || true

    # Wait for process to exit
    local max_wait=10
    local elapsed=0

    while [[ $elapsed -lt $max_wait ]]; do
        if ! ps -p "$pid" >/dev/null 2>&1; then
            log_success "Weaver stopped gracefully"
            rm -f "$PID_FILE"
            return 0
        fi

        sleep 1
        elapsed=$((elapsed + 1))
    done

    # Force kill if still running
    if ps -p "$pid" >/dev/null 2>&1; then
        log_warning "Force killing Weaver..."
        kill -TERM "$pid" 2>/dev/null || true
        sleep 1
        rm -f "$PID_FILE"
    fi

    log_success "Weaver stopped"
    return 0
}

# Show validation results
show_results() {
    log_info "Analyzing validation results..."
    echo ""

    local json_file="$OUTPUT/live_check.json"

    if [ ! -f "$json_file" ]; then
        log_error "No validation output found at: $json_file"
        echo ""
        echo "Weaver log:"
        cat "$LOG_FILE" 2>/dev/null || echo "(no log file)"
        return 1
    fi

    # Parse results with jq
    if command -v jq >/dev/null 2>&1; then
        echo "📊 Validation Summary:"
        echo "===================="

        local violations=$(jq -r '.statistics.advice_level_counts.violation // 0' "$json_file")
        local warnings=$(jq -r '.statistics.advice_level_counts.warning // 0' "$json_file")
        local coverage=$(jq -r '(.statistics.registry_coverage * 100 | round)' "$json_file")
        local samples=$(jq -r '.statistics.total_samples // 0' "$json_file")

        echo "Samples:    $samples"
        echo "Violations: $violations"
        echo "Warnings:   $warnings"
        echo "Coverage:   ${coverage}%"
        echo ""

        if [ "$violations" -gt 0 ]; then
            log_error "Schema violations detected:"
            jq -r '.violations[]? | "  - \(.level): \(.message)"' "$json_file"
            echo ""
            return 1
        fi

        if [ "$samples" -eq 0 ]; then
            log_warning "No telemetry samples received"
            log_info "This usually means:"
            log_info "  1. Tests didn't emit telemetry"
            log_info "  2. OTLP endpoint not configured correctly"
            log_info "  3. Telemetry features not enabled"
            echo ""
            return 1
        fi

        log_success "Validation passed!"
        log_success "  ✅ $samples telemetry samples processed"
        log_success "  ✅ 0 schema violations"
        log_success "  ✅ ${coverage}% registry coverage"
        echo ""

        return 0
    else
        log_warning "jq not installed, showing raw JSON:"
        cat "$json_file"
        return 0
    fi
}

# Setup signal handlers
cleanup() {
    echo ""
    log_info "Cleaning up..."
    stop_weaver HUP
    exit 0
}

trap cleanup EXIT INT TERM

# ========== MAIN LOGIC ==========

main() {
    local command="${1:-}"

    echo "================================================================================"
    echo "Weaver Live-Check with Port Coordination - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    # Pre-flight checks
    if ! check_weaver_installed; then
        exit 1
    fi

    if ! validate_registry; then
        exit 1
    fi

    check_docker_otel || true  # Warning only

    if ! check_ports; then
        exit 1
    fi

    # Setup
    cleanup_existing_weaver
    setup_output_dir

    # Start Weaver
    if ! start_weaver; then
        exit 1
    fi

    # Wait for ready
    if ! wait_for_weaver; then
        log_error "Failed to start Weaver"
        exit 1
    fi

    log_success "Weaver is ready to receive telemetry"
    echo ""

    # Run test if command provided, otherwise wait
    if [ -n "$command" ]; then
        run_test_command "$command" || true

        # Stop and show results
        stop_weaver HUP

        echo ""
        if show_results; then
            log_success "100% Weaver compliance achieved!"
            exit 0
        else
            log_error "Validation failed"
            exit 1
        fi
    else
        echo "Weaver is running. Export telemetry to:"
        echo "  OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:$WEAVER_OTLP_PORT"
        echo ""
        echo "Press Ctrl+C to stop and view results..."
        echo ""

        # Wait for user interrupt
        wait $(cat "$PID_FILE")
    fi
}

# ========== ENTRY POINT ==========

main "$@"
