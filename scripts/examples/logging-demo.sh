#!/usr/bin/env bash
# Demonstration of CLNRM Structured Logging Library
#
# This script showcases all features of the logging library:
# - Basic logging at different levels
# - Structured fields and metadata
# - Performance timers
# - Metrics (counters and gauges)
# - Correlation ID tracking
# - Context management
#
# Usage:
#   ./scripts/examples/logging-demo.sh              # Normal output
#   CLNRM_DEBUG=1 ./scripts/examples/logging-demo.sh  # With debug logs
#   NO_COLOR=1 ./scripts/examples/logging-demo.sh   # Without colors

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Source logging library
source "$SCRIPT_DIR/../lib/logging.sh"

# ============================================================================
# DEMO FUNCTIONS
# ============================================================================

demo_basic_logging() {
    echo ""
    echo "=== Basic Logging ==="
    echo ""

    log_debug "This is a debug message (only visible with CLNRM_DEBUG=1)"
    log_info "This is an info message"
    log_warn "This is a warning message"
    log_error "This is an error message"
}

demo_structured_fields() {
    echo ""
    echo "=== Structured Fields ==="
    echo ""

    log_info "Container started" \
        container_id="abc123" \
        image="alpine:latest" \
        status="running"

    log_info "Test execution completed" \
        test_name="integration_test_001" \
        duration_ms=1250 \
        success=true \
        assertions_passed=15 \
        assertions_failed=0
}

demo_performance_timers() {
    echo ""
    echo "=== Performance Timers ==="
    echo ""

    # Start timer
    timer_start "container_startup"
    log_info "Starting container..."

    # Simulate work
    sleep 0.5

    # Check elapsed time
    local elapsed
    elapsed=$(timer_elapsed "container_startup")
    log_info "Container startup in progress" elapsed_ms="$elapsed"

    # More work
    sleep 0.3

    # End timer
    timer_end "container_startup" true

    # Multiple timers
    timer_start "test_execution"
    log_info "Running tests..."
    sleep 0.2
    timer_end "test_execution" true

    # Failed operation timer
    timer_start "failed_operation"
    log_info "Attempting risky operation..."
    sleep 0.1
    timer_end "failed_operation" false
}

demo_metrics() {
    echo ""
    echo "=== Metrics ==="
    echo ""

    # Counters
    log_info "Recording test executions..."
    increment_counter "tests_passed" 5
    increment_counter "tests_failed" 2
    increment_counter "tests_skipped" 1

    log_info "Test counters updated" \
        passed=$(get_counter "tests_passed") \
        failed=$(get_counter "tests_failed") \
        skipped=$(get_counter "tests_skipped")

    # Gauges
    log_info "Recording system metrics..."
    record_gauge "memory_usage_mb" 512
    record_gauge "cpu_usage_percent" 45.7
    record_gauge "active_containers" 3

    log_info "System gauges updated" \
        memory=$(get_gauge "memory_usage_mb") \
        cpu=$(get_gauge "cpu_usage_percent") \
        containers=$(get_gauge "active_containers")

    # Log all metrics
    log_metrics "Current metrics snapshot"
}

demo_context_management() {
    echo ""
    echo "=== Context Management ==="
    echo ""

    # Original context
    log_info "Using default context"

    # Change correlation ID
    set_correlation_id "demo-run-$(date +%s)"
    log_info "Updated correlation ID"

    # Change service name
    set_service_name "demo-service"
    log_info "Updated service name"

    # Change environment
    set_environment "staging"
    log_info "Updated environment"

    # Log with context
    log_with_context "INFO" "Operation in new context" \
        operation="test_cleanup" \
        resources_freed=3
}

demo_error_scenarios() {
    echo ""
    echo "=== Error Scenarios ==="
    echo ""

    # Simulate container failure
    log_error "Container failed to start" \
        container_id="def456" \
        image="broken:latest" \
        exit_code=1 \
        error_message="Image not found"

    # Simulate test failure
    log_error "Test assertion failed" \
        test_name="test_database_connection" \
        expected="connected" \
        actual="timeout" \
        line_number=42

    # Simulate configuration error
    log_warn "Invalid configuration detected" \
        config_file=".clnrm.toml" \
        issue="missing_service_definition" \
        service="redis"
}

demo_real_world_workflow() {
    echo ""
    echo "=== Real-World Workflow ==="
    echo ""

    # Setup phase
    set_correlation_id "workflow-$(date +%s)"
    log_info "Starting test workflow" workflow="integration_tests"

    timer_start "workflow"
    increment_counter "workflows_started"

    # Phase 1: Container setup
    log_info "Phase 1: Setting up containers"
    timer_start "container_setup"

    for i in {1..3}; do
        log_info "Starting container" container_number="$i" image="alpine:latest"
        sleep 0.1
        increment_counter "containers_started"
    done

    timer_end "container_setup" true

    # Phase 2: Test execution
    log_info "Phase 2: Executing tests"
    timer_start "test_execution"

    for i in {1..10}; do
        local success=$((RANDOM % 10))
        if [[ $success -gt 1 ]]; then
            log_debug "Test passed" test_id="test_$i"
            increment_counter "tests_passed"
        else
            log_warn "Test failed" test_id="test_$i" reason="assertion_error"
            increment_counter "tests_failed"
        fi
    done

    timer_end "test_execution" true

    # Phase 3: Cleanup
    log_info "Phase 3: Cleaning up"
    timer_start "cleanup"

    record_gauge "final_memory_usage_mb" 256
    record_gauge "final_cpu_usage_percent" 12.3

    sleep 0.1
    timer_end "cleanup" true

    # Workflow complete
    timer_end "workflow" true
    increment_counter "workflows_completed"

    log_info "Workflow completed successfully" \
        total_tests=$(($(get_counter "tests_passed") + $(get_counter "tests_failed"))) \
        passed=$(get_counter "tests_passed") \
        failed=$(get_counter "tests_failed")

    # Export final metrics
    local metrics_file="/tmp/clnrm-demo-metrics.json"
    export_metrics "$metrics_file"
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    log_info "Starting CLNRM logging library demonstration"

    demo_basic_logging
    demo_structured_fields
    demo_performance_timers
    demo_metrics
    demo_context_management
    demo_error_scenarios
    demo_real_world_workflow

    echo ""
    echo "=== Demonstration Complete ==="
    echo ""
    log_info "Demonstration finished successfully"

    # Show metrics summary
    log_metrics "Final metrics"

    echo ""
    echo "Tip: Run with CLNRM_DEBUG=1 to see debug logs"
    echo "Tip: Run with NO_COLOR=1 to disable colors"
    echo "Tip: Redirect to file to see pure JSON: $0 > output.json 2>&1"
}

# Run demo
main "$@"
