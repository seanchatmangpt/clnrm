#!/usr/bin/env bash
# Structured JSON Logging Library for CLNRM
# Adapted from kcura logging patterns for hermetic testing framework
#
# Features:
# - JSON-structured output for machine parsing
# - Color-coded terminal output (optional via NO_COLOR)
# - Correlation ID tracking
# - Performance timers
# - Metrics collection (counters, gauges)
# - Log levels: DEBUG, INFO, WARN, ERROR
#
# Usage:
#   source scripts/lib/logging.sh
#   log_info "Container started" service="alpine" image="alpine:latest"
#   timer_start "container_startup"
#   # ... operations ...
#   timer_end "container_startup"

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================

# Log level (DEBUG=0, INFO=1, WARN=2, ERROR=3)
CLNRM_LOG_LEVEL="${CLNRM_LOG_LEVEL:-1}"

# Debug mode
CLNRM_DEBUG="${CLNRM_DEBUG:-0}"

# Color output (set NO_COLOR=1 to disable)
NO_COLOR="${NO_COLOR:-0}"

# JSON output mode (always structured, but with/without colors)
CLNRM_JSON_OUTPUT="${CLNRM_JSON_OUTPUT:-1}"

# Correlation ID for tracking related operations
CLNRM_CORRELATION_ID="${CLNRM_CORRELATION_ID:-clnrm-$(date +%s)-$$}"

# Service name for logging context
CLNRM_SERVICE_NAME="${CLNRM_SERVICE_NAME:-clnrm}"

# Deployment environment
CLNRM_ENVIRONMENT="${CLNRM_ENVIRONMENT:-local}"

# ============================================================================
# COLOR DEFINITIONS
# ============================================================================

if [[ "$NO_COLOR" -eq 0 ]]; then
    RESET='\033[0m'
    BOLD='\033[1m'
    DIM='\033[2m'

    # Log levels
    DEBUG_COLOR='\033[0;36m'    # Cyan
    INFO_COLOR='\033[0;32m'     # Green
    WARN_COLOR='\033[0;33m'     # Yellow
    ERROR_COLOR='\033[0;31m'    # Red

    # Components
    TIMESTAMP_COLOR='\033[0;90m' # Gray
    KEY_COLOR='\033[0;34m'       # Blue
    VALUE_COLOR='\033[0;37m'     # White
else
    RESET=''
    BOLD=''
    DIM=''
    DEBUG_COLOR=''
    INFO_COLOR=''
    WARN_COLOR=''
    ERROR_COLOR=''
    TIMESTAMP_COLOR=''
    KEY_COLOR=''
    VALUE_COLOR=''
fi

# ============================================================================
# INTERNAL STATE
# ============================================================================

# Associative arrays for timers and counters
declare -A CLNRM_TIMERS
declare -A CLNRM_COUNTERS
declare -A CLNRM_GAUGES

# Log history (for testing and debugging)
declare -a CLNRM_LOG_HISTORY

# ============================================================================
# CORE LOGGING FUNCTIONS
# ============================================================================

# Get ISO 8601 timestamp
_log_timestamp() {
    # Try GNU date format first, fall back to basic ISO 8601
    if date -u +"%Y-%m-%dT%H:%M:%S.%3NZ" 2>/dev/null | grep -q 'N'; then
        # %3N not supported, use basic format
        date -u +"%Y-%m-%dT%H:%M:%SZ"
    else
        date -u +"%Y-%m-%dT%H:%M:%S.%3NZ" 2>/dev/null
    fi
}

# Escape JSON string (without requiring jq)
_json_escape() {
    local string="$1"
    # Basic JSON escaping
    string="${string//\\/\\\\}"  # Escape backslashes
    string="${string//\"/\\\"}"  # Escape quotes
    string="${string//$'\n'/\\n}"  # Escape newlines
    string="${string//$'\r'/\\r}"  # Escape carriage returns
    string="${string//$'\t'/\\t}"  # Escape tabs
    printf '"%s"' "$string"
}

# Build JSON log entry
_build_json_log() {
    local level="$1"
    local message="$2"
    shift 2

    local timestamp
    timestamp="$(_log_timestamp)"

    local json="{\"timestamp\":\"$timestamp\""
    json+=",\"level\":\"$level\""
    json+=",\"message\":$(_json_escape "$message")"
    json+=",\"correlation_id\":\"$CLNRM_CORRELATION_ID\""
    json+=",\"service\":\"$CLNRM_SERVICE_NAME\""
    json+=",\"environment\":\"$CLNRM_ENVIRONMENT\""
    json+=",\"pid\":$$"

    # Add custom fields
    local metadata="{"
    local first=1
    while [[ $# -gt 0 ]]; do
        if [[ "$1" =~ ^([a-zA-Z_][a-zA-Z0-9_]*)=(.+)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local value="${BASH_REMATCH[2]}"

            if [[ $first -eq 0 ]]; then
                metadata+=","
            fi
            first=0

            # Try to detect if value is numeric or boolean
            if [[ "$value" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
                metadata+="\"$key\":$value"
            elif [[ "$value" == "true" || "$value" == "false" ]]; then
                metadata+="\"$key\":$value"
            else
                metadata+="\"$key\":$(_json_escape "$value")"
            fi
        fi
        shift
    done
    metadata+="}"

    if [[ "$metadata" != "{}" ]]; then
        json+=",\"metadata\":$metadata"
    fi

    json+="}"
    echo "$json"
}

# Format colored terminal output
_format_terminal_log() {
    local level="$1"
    local message="$2"
    local json="$3"

    local level_color
    case "$level" in
        DEBUG) level_color="$DEBUG_COLOR" ;;
        INFO)  level_color="$INFO_COLOR" ;;
        WARN)  level_color="$WARN_COLOR" ;;
        ERROR) level_color="$ERROR_COLOR" ;;
        *) level_color="$RESET" ;;
    esac

    local timestamp
    timestamp="$(_log_timestamp)"

    if [[ "$NO_COLOR" -eq 0 ]]; then
        echo -e "${TIMESTAMP_COLOR}${timestamp}${RESET} ${level_color}${BOLD}${level}${RESET} ${message}"

        # Pretty-print metadata if jq available
        if command -v jq >/dev/null 2>&1; then
            local metadata
            metadata=$(echo "$json" | jq -r '.metadata // empty' 2>/dev/null)
            if [[ -n "$metadata" && "$metadata" != "null" ]]; then
                echo "$metadata" | jq -C '.' 2>/dev/null || echo "$metadata"
            fi
        fi
    else
        echo "$json"
    fi
}

# Main log function
_log() {
    local level="$1"
    local level_num="$2"
    local message="$3"
    shift 3

    # Check log level
    if [[ "$level_num" -lt "$CLNRM_LOG_LEVEL" ]]; then
        return 0
    fi

    # Build JSON log
    local json
    json="$(_build_json_log "$level" "$message" "$@")"

    # Add to history
    CLNRM_LOG_HISTORY+=("$json")

    # Output
    if [[ "$CLNRM_JSON_OUTPUT" -eq 1 ]]; then
        if [[ -t 1 ]]; then
            # Terminal output: colorized
            _format_terminal_log "$level" "$message" "$json"
        else
            # Non-terminal (CI): pure JSON
            echo "$json"
        fi
    else
        # Simple text output
        echo "[$level] $message"
    fi
}

# ============================================================================
# PUBLIC LOGGING API
# ============================================================================

# Log at DEBUG level (only if CLNRM_DEBUG=1)
log_debug() {
    if [[ "$CLNRM_DEBUG" -eq 1 ]]; then
        _log "DEBUG" 0 "$@"
    fi
}

# Log at INFO level
log_info() {
    _log "INFO" 1 "$@"
}

# Log at WARN level
log_warn() {
    _log "WARN" 2 "$@"
}

# Log at ERROR level
log_error() {
    _log "ERROR" 3 "$@"
}

# Log with explicit context fields
log_with_context() {
    local level="$1"
    local message="$2"
    shift 2

    case "$level" in
        DEBUG) log_debug "$message" "$@" ;;
        INFO)  log_info "$message" "$@" ;;
        WARN)  log_warn "$message" "$@" ;;
        ERROR) log_error "$message" "$@" ;;
        *) log_info "$message" "$@" ;;
    esac
}

# ============================================================================
# PERFORMANCE TIMERS
# ============================================================================

# Start a named timer
timer_start() {
    local name="$1"
    # Use seconds * 1000 for millisecond precision (portable)
    local timestamp
    timestamp=$(date +%s)
    CLNRM_TIMERS["$name"]=$((timestamp * 1000))
    log_debug "Timer started" timer="$name"
}

# End a named timer and log duration
timer_end() {
    local name="$1"
    local success="${2:-true}"

    if [[ -z "${CLNRM_TIMERS[$name]:-}" ]]; then
        log_warn "Timer not found" timer="$name"
        return 1
    fi

    local start_time="${CLNRM_TIMERS[$name]}"
    local end_timestamp
    end_timestamp=$(date +%s)
    local end_time=$((end_timestamp * 1000))
    local duration=$((end_time - start_time))

    log_info "Timer completed" timer="$name" duration_ms="$duration" success="$success"

    # Clean up
    unset "CLNRM_TIMERS[$name]"
}

# Get timer duration without ending it
timer_elapsed() {
    local name="$1"

    if [[ -z "${CLNRM_TIMERS[$name]:-}" ]]; then
        echo "0"
        return 1
    fi

    local start_time="${CLNRM_TIMERS[$name]}"
    local current_timestamp
    current_timestamp=$(date +%s)
    local current_time=$((current_timestamp * 1000))
    echo $((current_time - start_time))
}

# ============================================================================
# METRICS
# ============================================================================

# Increment a named counter
increment_counter() {
    local name="$1"
    local increment="${2:-1}"

    local current="${CLNRM_COUNTERS[$name]:-0}"
    CLNRM_COUNTERS["$name"]=$((current + increment))

    log_debug "Counter incremented" counter="$name" value="${CLNRM_COUNTERS[$name]}"
}

# Get counter value
get_counter() {
    local name="$1"
    echo "${CLNRM_COUNTERS[$name]:-0}"
}

# Record a gauge value
record_gauge() {
    local name="$1"
    local value="$2"

    CLNRM_GAUGES["$name"]="$value"
    log_debug "Gauge recorded" gauge="$name" value="$value"
}

# Get gauge value
get_gauge() {
    local name="$1"
    echo "${CLNRM_GAUGES[$name]:-0}"
}

# Log all metrics
log_metrics() {
    local message="${1:-Metrics snapshot}"

    local metrics="{"
    local first=1

    # Add counters
    for key in "${!CLNRM_COUNTERS[@]}"; do
        if [[ $first -eq 0 ]]; then
            metrics+=","
        fi
        first=0
        metrics+="\"counter_$key\":${CLNRM_COUNTERS[$key]}"
    done

    # Add gauges
    for key in "${!CLNRM_GAUGES[@]}"; do
        if [[ $first -eq 0 ]]; then
            metrics+=","
        fi
        first=0
        metrics+="\"gauge_$key\":${CLNRM_GAUGES[$key]}"
    done

    metrics+="}"

    if [[ "$metrics" != "{}" ]]; then
        log_info "$message" metrics="$metrics"
    fi
}

# ============================================================================
# STRUCTURED CONTEXT
# ============================================================================

# Set correlation ID
set_correlation_id() {
    local new_id="$1"
    CLNRM_CORRELATION_ID="$new_id"
    export CLNRM_CORRELATION_ID
    log_debug "Correlation ID updated" correlation_id="$CLNRM_CORRELATION_ID"
}

# Set service name
set_service_name() {
    local new_name="$1"
    CLNRM_SERVICE_NAME="$new_name"
    export CLNRM_SERVICE_NAME
    log_debug "Service name updated" service="$CLNRM_SERVICE_NAME"
}

# Set environment
set_environment() {
    local new_env="$1"
    CLNRM_ENVIRONMENT="$new_env"
    export CLNRM_ENVIRONMENT
    log_debug "Environment updated" environment="$CLNRM_ENVIRONMENT"
}

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

# Clear log history
clear_log_history() {
    CLNRM_LOG_HISTORY=()
    log_debug "Log history cleared"
}

# Get log history (for testing)
get_log_history() {
    printf '%s\n' "${CLNRM_LOG_HISTORY[@]}"
}

# Export metrics to JSON file
export_metrics() {
    local output_file="$1"

    local json="{"
    json+="\"timestamp\":\"$(_log_timestamp)\""
    json+=",\"correlation_id\":\"$CLNRM_CORRELATION_ID\""
    json+=",\"counters\":{"

    local first=1
    for key in "${!CLNRM_COUNTERS[@]}"; do
        if [[ $first -eq 0 ]]; then
            json+=","
        fi
        first=0
        json+="\"$key\":${CLNRM_COUNTERS[$key]}"
    done
    json+="}"

    json+=",\"gauges\":{"
    first=1
    for key in "${!CLNRM_GAUGES[@]}"; do
        if [[ $first -eq 0 ]]; then
            json+=","
        fi
        first=0
        json+="\"$key\":${CLNRM_GAUGES[$key]}"
    done
    json+="}"

    json+="}"

    # Pretty-print with jq if available, otherwise just write raw JSON
    if command -v jq >/dev/null 2>&1; then
        echo "$json" | jq '.' > "$output_file" 2>/dev/null || echo "$json" > "$output_file"
    else
        echo "$json" > "$output_file"
    fi
    log_info "Metrics exported" file="$output_file"
}

# ============================================================================
# INITIALIZATION
# ============================================================================

# Log library initialization
log_debug "Logging library initialized" \
    correlation_id="$CLNRM_CORRELATION_ID" \
    service="$CLNRM_SERVICE_NAME" \
    environment="$CLNRM_ENVIRONMENT" \
    debug="$CLNRM_DEBUG"
