#!/usr/bin/env bash
# Unit tests for CLNRM structured logging library
#
# Tests all core functionality:
# - JSON output format
# - Log levels
# - Structured fields
# - Timers
# - Metrics
# - Context management
#
# Usage:
#   ./scripts/tests/test-logging.sh

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Source logging library
source "$SCRIPT_DIR/../lib/logging.sh"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# ============================================================================
# TEST FRAMEWORK
# ============================================================================

assert_equals() {
    local expected="$1"
    local actual="$2"
    local message="${3:-Assertion failed}"

    if [[ "$expected" == "$actual" ]]; then
        echo "✓ $message"
        ((TESTS_PASSED++)) || true
        return 0
    else
        echo "✗ $message"
        echo "  Expected: $expected"
        echo "  Actual:   $actual"
        ((TESTS_FAILED++)) || true
        return 1
    fi
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-Assertion failed}"

    if [[ "$haystack" == *"$needle"* ]]; then
        echo "✓ $message"
        ((TESTS_PASSED++)) || true
        return 0
    else
        echo "✗ $message"
        echo "  Expected to contain: $needle"
        echo "  In: $haystack"
        ((TESTS_FAILED++)) || true
        return 1
    fi
}

assert_not_empty() {
    local value="$1"
    local message="${2:-Value should not be empty}"

    if [[ -n "$value" ]]; then
        echo "✓ $message"
        ((TESTS_PASSED++)) || true
        return 0
    else
        echo "✗ $message"
        ((TESTS_FAILED++)) || true
        return 1
    fi
}

run_test() {
    local test_name="$1"
    ((TESTS_RUN++)) || true
    echo ""
    echo "Running: $test_name"
}

# ============================================================================
# TESTS
# ============================================================================

test_json_escaping() {
    run_test "JSON escaping"

    local result
    result=$(_json_escape 'hello "world"')
    assert_equals '"hello \"world\""' "$result" "Should escape quotes"

    result=$(_json_escape $'line1\nline2')
    assert_contains "$result" '\n' "Should escape newlines"
}

test_log_levels() {
    run_test "Log levels"

    # Clear history
    clear_log_history

    # Log at different levels
    log_info "Info message"
    log_warn "Warning message"
    log_error "Error message"

    # Check history
    local history
    history=$(get_log_history)

    assert_contains "$history" '"level":"INFO"' "Should have INFO level"
    assert_contains "$history" '"level":"WARN"' "Should have WARN level"
    assert_contains "$history" '"level":"ERROR"' "Should have ERROR level"
}

test_structured_fields() {
    run_test "Structured fields"

    clear_log_history

    log_info "Test message" field1="value1" field2=123 field3=true

    local history
    history=$(get_log_history | tail -1)

    assert_contains "$history" '"field1":"value1"' "Should have string field"
    assert_contains "$history" '"field2":123' "Should have numeric field"
    assert_contains "$history" '"field3":true' "Should have boolean field"
}

test_correlation_id() {
    run_test "Correlation ID"

    clear_log_history

    local old_id="$CLNRM_CORRELATION_ID"
    set_correlation_id "test-correlation-123"

    log_info "Test message"

    local history
    history=$(get_log_history | tail -1)

    assert_contains "$history" '"correlation_id":"test-correlation-123"' "Should have custom correlation ID"

    # Restore
    CLNRM_CORRELATION_ID="$old_id"
}

test_timers() {
    run_test "Timers"

    # Start timer
    timer_start "test_timer"

    # Timer should exist
    assert_not_empty "${CLNRM_TIMERS[test_timer]:-}" "Timer should be started"

    # Wait a bit
    sleep 0.1

    # Check elapsed time
    local elapsed
    elapsed=$(timer_elapsed "test_timer")
    assert_not_empty "$elapsed" "Elapsed time should not be empty"

    # End timer
    clear_log_history
    timer_end "test_timer" true

    local history
    history=$(get_log_history | tail -1)

    assert_contains "$history" '"timer":"test_timer"' "Should log timer name"
    assert_contains "$history" '"success":true' "Should log success status"
    assert_contains "$history" 'duration_ms' "Should log duration"
}

test_counters() {
    run_test "Counters"

    # Reset counter
    unset "CLNRM_COUNTERS[test_counter]"

    # Increment
    increment_counter "test_counter" 5
    local value
    value=$(get_counter "test_counter")
    assert_equals "5" "$value" "Counter should be 5"

    # Increment again
    increment_counter "test_counter" 3
    value=$(get_counter "test_counter")
    assert_equals "8" "$value" "Counter should be 8"

    # Default increment
    increment_counter "test_counter"
    value=$(get_counter "test_counter")
    assert_equals "9" "$value" "Counter should be 9"
}

test_gauges() {
    run_test "Gauges"

    # Record gauge
    record_gauge "test_gauge" 42.5
    local value
    value=$(get_gauge "test_gauge")
    assert_equals "42.5" "$value" "Gauge should be 42.5"

    # Update gauge
    record_gauge "test_gauge" 100
    value=$(get_gauge "test_gauge")
    assert_equals "100" "$value" "Gauge should be 100"
}

test_context_management() {
    run_test "Context management"

    local old_service="$CLNRM_SERVICE_NAME"
    local old_env="$CLNRM_ENVIRONMENT"

    # Update context
    set_service_name "test-service"
    set_environment "testing"

    clear_log_history
    log_info "Test message"

    local history
    history=$(get_log_history | tail -1)

    assert_contains "$history" '"service":"test-service"' "Should have custom service name"
    assert_contains "$history" '"environment":"testing"' "Should have custom environment"

    # Restore
    CLNRM_SERVICE_NAME="$old_service"
    CLNRM_ENVIRONMENT="$old_env"
}

test_metrics_export() {
    run_test "Metrics export"

    # Set up some metrics
    increment_counter "export_test_counter" 10
    record_gauge "export_test_gauge" 55.5

    # Export to temp file
    local temp_file
    temp_file=$(mktemp)

    export_metrics "$temp_file"

    # Check file exists and has content
    assert_not_empty "$(cat "$temp_file")" "Exported metrics file should not be empty"

    # Check JSON structure
    local content
    content=$(cat "$temp_file")
    assert_contains "$content" '"counters"' "Should have counters section"
    assert_contains "$content" '"gauges"' "Should have gauges section"

    # Clean up
    rm -f "$temp_file"
}

test_log_with_context() {
    run_test "Log with context"

    clear_log_history

    log_with_context "INFO" "Context message" key1="val1"
    log_with_context "WARN" "Warning message" key2="val2"
    log_with_context "ERROR" "Error message" key3="val3"

    local history
    history=$(get_log_history)

    assert_contains "$history" '"level":"INFO"' "Should have INFO from context"
    assert_contains "$history" '"level":"WARN"' "Should have WARN from context"
    assert_contains "$history" '"level":"ERROR"' "Should have ERROR from context"
}

test_pid_in_logs() {
    run_test "PID in logs"

    clear_log_history
    log_info "Test message"

    local history
    history=$(get_log_history | tail -1)

    assert_contains "$history" "\"pid\":$$" "Should include process ID"
}

# ============================================================================
# RUN ALL TESTS
# ============================================================================

main() {
    echo "======================================"
    echo "CLNRM Structured Logging Library Tests"
    echo "======================================"

    # Disable colors and JSON output for testing
    NO_COLOR=1
    CLNRM_JSON_OUTPUT=1

    # Run all tests
    test_json_escaping
    test_log_levels
    test_structured_fields
    test_correlation_id
    test_timers
    test_counters
    test_gauges
    test_context_management
    test_metrics_export
    test_log_with_context
    test_pid_in_logs

    # Summary
    echo ""
    echo "======================================"
    echo "Test Summary"
    echo "======================================"
    echo "Total tests run:    $TESTS_RUN"
    echo "Tests passed:       $TESTS_PASSED"
    echo "Tests failed:       $TESTS_FAILED"
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo "✓ All tests passed!"
        exit 0
    else
        echo "✗ Some tests failed"
        exit 1
    fi
}

main "$@"
