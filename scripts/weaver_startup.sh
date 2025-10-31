#!/bin/bash
# Weaver Live-Check Startup Script
# Manages Weaver process lifecycle with proper cleanup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REGISTRY="${REGISTRY:-registry/}"
OUTPUT="${OUTPUT:-validation_output/}"
OTLP_PORT="${OTLP_PORT:-4317}"
ADMIN_PORT="${ADMIN_PORT:-8080}"
TIMEOUT="${TIMEOUT:-300}"  # 5 minutes inactivity timeout
PID_FILE="${PID_FILE:-/tmp/weaver.pid}"
LOG_FILE="${LOG_FILE:-/tmp/weaver.log}"

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
        echo ""
        echo "Or download from:"
        echo "  https://github.com/open-telemetry/weaver/releases"
        echo ""
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
        echo ""
        echo "Running validation with output:"
        weaver registry check --registry "$REGISTRY"
        return 1
    fi

    log_success "Registry validated"
    return 0
}

# Check if port is available
check_port() {
    local port=$1
    local name=$2

    if lsof -i ":$port" >/dev/null 2>&1; then
        log_warning "$name port $port is already in use"
        return 1
    fi

    return 0
}

# Clean up existing Weaver process
cleanup_existing_weaver() {
    log_info "Checking for existing Weaver processes..."

    # Check PID file
    if [[ -f "$PID_FILE" ]]; then
        local old_pid=$(cat "$PID_FILE")
        if ps -p "$old_pid" >/dev/null 2>&1; then
            log_warning "Stopping existing Weaver process (PID: $old_pid)"
            kill -HUP "$old_pid" 2>/dev/null || kill -9 "$old_pid" 2>/dev/null || true
            sleep 2
        fi
        rm -f "$PID_FILE"
    fi

    # Check ports
    if lsof -i ":$OTLP_PORT" >/dev/null 2>&1; then
        log_warning "Cleaning up process on OTLP port $OTLP_PORT"
        local pid=$(lsof -t -i ":$OTLP_PORT")
        kill -HUP "$pid" 2>/dev/null || kill -9 "$pid" 2>/dev/null || true
        sleep 2
    fi

    if lsof -i ":$ADMIN_PORT" >/dev/null 2>&1; then
        log_warning "Cleaning up process on admin port $ADMIN_PORT"
        local pid=$(lsof -t -i ":$ADMIN_PORT")
        kill -HUP "$pid" 2>/dev/null || kill -9 "$pid" 2>/dev/null || true
        sleep 2
    fi

    log_success "Cleanup complete"
}

# Create output directory
setup_output_dir() {
    log_info "Setting up output directory..."

    mkdir -p "$OUTPUT"

    # Clean old reports if present
    if [[ -f "$OUTPUT/live_check.json" ]]; then
        local backup="$OUTPUT/live_check.json.$(date +%s)"
        mv "$OUTPUT/live_check.json" "$backup"
        log_info "Backed up previous report to: $backup"
    fi

    log_success "Output directory ready: $OUTPUT"
}

# Start Weaver live-check
start_weaver() {
    log_info "Starting Weaver live-check..."
    echo ""

    # Start Weaver in background
    weaver registry live-check \
        --registry "$REGISTRY" \
        --otlp-grpc-port "$OTLP_PORT" \
        --admin-port "$ADMIN_PORT" \
        --format json \
        --output "$OUTPUT" \
        --inactivity-timeout "$TIMEOUT" \
        > "$LOG_FILE" 2>&1 &

    local pid=$!
    echo "$pid" > "$PID_FILE"

    log_success "Weaver started (PID: $pid)"
    log_info "Log file: $LOG_FILE"
    log_info "PID file: $PID_FILE"
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
        if lsof -i ":$OTLP_PORT" >/dev/null 2>&1; then
            echo ""
            log_success "Weaver is listening on :$OTLP_PORT"
            return 0
        fi

        echo -n "."
        sleep 1
        elapsed=$((elapsed + 1))
    done

    echo ""
    log_error "Weaver did not start listening within ${max_wait}s"
    echo ""
    echo "Log output:"
    cat "$LOG_FILE" 2>/dev/null || echo "(no log file)"
    return 1
}

# Get Weaver status
get_weaver_status() {
    local pid=$(cat "$PID_FILE" 2>/dev/null || echo "")

    if [[ -z "$pid" ]]; then
        echo "not_running"
        return
    fi

    if ! ps -p "$pid" >/dev/null 2>&1; then
        echo "dead"
        return
    fi

    if lsof -i ":$OTLP_PORT" >/dev/null 2>&1; then
        echo "running"
    else
        echo "starting"
    fi
}

# Stop Weaver gracefully
stop_weaver() {
    local signal="${1:-HUP}"  # HUP for graceful, TERM for force

    log_info "Stopping Weaver (signal: $signal)..."

    local pid=$(cat "$PID_FILE" 2>/dev/null || echo "")

    if [[ -z "$pid" ]]; then
        log_warning "No PID file found"
        return 1
    fi

    if ! ps -p "$pid" >/dev/null 2>&1; then
        log_warning "Weaver process not running (PID: $pid)"
        rm -f "$PID_FILE"
        return 1
    fi

    # Send signal
    kill -"$signal" "$pid" 2>/dev/null || {
        log_error "Failed to send signal to process $pid"
        return 1
    }

    # Wait for process to exit
    local max_wait=10
    local elapsed=0

    while [[ $elapsed -lt $max_wait ]]; do
        if ! ps -p "$pid" >/dev/null 2>&1; then
            log_success "Weaver stopped"
            rm -f "$PID_FILE"
            return 0
        fi

        sleep 1
        elapsed=$((elapsed + 1))
    done

    # Force kill if still running
    if ps -p "$pid" >/dev/null 2>&1; then
        log_warning "Weaver did not stop gracefully, force killing..."
        kill -9 "$pid" 2>/dev/null || true
        sleep 1
        rm -f "$PID_FILE"
    fi

    log_success "Weaver stopped"
    return 0
}

# Print Weaver info
print_weaver_info() {
    echo ""
    echo "Weaver Configuration:"
    echo "  Registry:     $REGISTRY"
    echo "  Output:       $OUTPUT"
    echo "  OTLP Port:    $OTLP_PORT"
    echo "  Admin Port:   $ADMIN_PORT"
    echo "  Timeout:      ${TIMEOUT}s"
    echo "  PID File:     $PID_FILE"
    echo "  Log File:     $LOG_FILE"
    echo ""
}

# ========== MAIN LOGIC ==========

main() {
    local action="${1:-start}"  # start, stop, restart, status

    echo "================================================================================"
    echo "Weaver Process Manager - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    case "$action" in
        start)
            # Pre-flight checks
            if ! check_weaver_installed; then
                exit 1
            fi

            if ! validate_registry; then
                exit 1
            fi

            # Cleanup
            cleanup_existing_weaver
            setup_output_dir

            print_weaver_info

            # Start Weaver
            if ! start_weaver; then
                exit 1
            fi

            # Wait for ready
            if ! wait_for_weaver; then
                log_error "Failed to start Weaver"
                exit 1
            fi

            echo ""
            log_success "Weaver is ready to receive telemetry"
            echo ""
            echo "OTLP Endpoint: http://localhost:$OTLP_PORT"
            echo "Admin API:     http://localhost:$ADMIN_PORT"
            echo ""
            echo "To stop Weaver:"
            echo "  $0 stop"
            echo ""
            ;;

        stop)
            stop_weaver HUP
            ;;

        force-stop)
            stop_weaver TERM
            ;;

        restart)
            log_info "Restarting Weaver..."
            stop_weaver HUP
            sleep 2
            "$0" start
            ;;

        status)
            local status=$(get_weaver_status)
            local pid=$(cat "$PID_FILE" 2>/dev/null || echo "unknown")

            echo "Weaver Status: $status"
            echo "PID: $pid"
            echo ""

            if [[ "$status" == "running" ]]; then
                print_weaver_info
                log_success "Weaver is running and listening"

                # Show recent log lines
                if [[ -f "$LOG_FILE" ]]; then
                    echo "Recent logs:"
                    tail -5 "$LOG_FILE" 2>/dev/null || echo "(no logs)"
                fi
            elif [[ "$status" == "not_running" ]]; then
                log_warning "Weaver is not running"
            elif [[ "$status" == "dead" ]]; then
                log_error "Weaver process is dead"
                echo ""
                echo "Last log output:"
                tail -10 "$LOG_FILE" 2>/dev/null || echo "(no logs)"
            elif [[ "$status" == "starting" ]]; then
                log_info "Weaver is starting..."
            fi
            ;;

        logs)
            if [[ -f "$LOG_FILE" ]]; then
                echo "Weaver logs ($LOG_FILE):"
                echo "========================================"
                cat "$LOG_FILE"
            else
                log_warning "No log file found"
            fi
            ;;

        help|--help|-h)
            echo "Usage: $0 [ACTION]"
            echo ""
            echo "Actions:"
            echo "  start       Start Weaver live-check (default)"
            echo "  stop        Stop Weaver gracefully"
            echo "  force-stop  Force stop Weaver"
            echo "  restart     Restart Weaver"
            echo "  status      Show Weaver status"
            echo "  logs        Show Weaver logs"
            echo ""
            echo "Environment Variables:"
            echo "  REGISTRY      Registry path (default: registry/)"
            echo "  OUTPUT        Output directory (default: validation_output/)"
            echo "  OTLP_PORT     OTLP gRPC port (default: 4317)"
            echo "  ADMIN_PORT    Admin API port (default: 8080)"
            echo "  TIMEOUT       Inactivity timeout in seconds (default: 300)"
            echo "  PID_FILE      PID file location (default: /tmp/weaver.pid)"
            echo "  LOG_FILE      Log file location (default: /tmp/weaver.log)"
            echo ""
            echo "Examples:"
            echo "  # Start Weaver"
            echo "  $0 start"
            echo ""
            echo "  # Start with custom port"
            echo "  OTLP_PORT=5317 $0 start"
            echo ""
            echo "  # Check status"
            echo "  $0 status"
            echo ""
            echo "  # View logs"
            echo "  $0 logs"
            echo ""
            exit 0
            ;;

        *)
            log_error "Unknown action: $action"
            echo ""
            echo "Run '$0 help' for usage information"
            exit 1
            ;;
    esac

    echo "================================================================================"
}

# Setup signal handlers for cleanup
trap 'stop_weaver TERM' EXIT INT TERM

# ========== ENTRY POINT ==========

main "$@"
