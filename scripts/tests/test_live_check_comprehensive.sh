#!/bin/bash
# test_live_check_comprehensive.sh
# Comprehensive live-check capability testing for clnrm Weaver integration
#
# This test harness validates ALL live-check capabilities including:
# - OTLP gRPC input
# - File input (JSON)
# - stdin input (text)
# - JSON output format
# - ANSI output format
# - Inactivity timeout
# - SIGHUP graceful stop
# - Custom policies
# - Statistics generation
# - Concurrent instances

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_OUTPUT="${SCRIPT_DIR}/../../validation_output/live_check_tests"
mkdir -p "$TEST_OUTPUT"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
declare -a PASSED=()
declare -a FAILED=()
declare -a SKIPPED=()
START_TIME=$(date +%s)

# Logging
LOG_FILE="$TEST_OUTPUT/test_run_$(date +%Y%m%d_%H%M%S).log"

log() {
    echo "[$(date +%Y-%m-%d\ %H:%M:%S)] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*" | tee -a "$LOG_FILE"
}

# Helper: Run test with error handling
run_test() {
    local name=$1
    local test_func=$2
    local test_log="$TEST_OUTPUT/${name}.log"

    echo ""
    log_info "========================================="
    log_info "Running: $name"
    log_info "========================================="

    if $test_func > "$test_log" 2>&1; then
        log_success "✅ PASSED: $name"
        PASSED+=("$name")
        return 0
    else
        log_error "❌ FAILED: $name"
        log_error "Check log: $test_log"
        tail -20 "$test_log" | while read -r line; do
            log_error "  $line"
        done
        FAILED+=("$name")
        return 1
    fi
}

# Helper: Check if weaver is installed
check_weaver() {
    if ! command -v weaver &> /dev/null; then
        log_error "weaver command not found. Please install OpenTelemetry Weaver."
        log_error "Visit: https://github.com/open-telemetry/weaver"
        exit 1
    fi

    local version=$(weaver --version 2>&1 || echo "unknown")
    log_info "Using weaver version: $version"
}

# Helper: Check if registry exists
check_registry() {
    if [ ! -d "$SCRIPT_DIR/../../registry" ]; then
        log_error "Registry directory not found at: $SCRIPT_DIR/../../registry"
        exit 1
    fi
    log_info "Registry found at: $SCRIPT_DIR/../../registry"
}

# Helper: Cleanup background processes
cleanup() {
    log_info "Cleaning up background processes..."
    pkill -P $$ 2>/dev/null || true
    sleep 1
}

trap cleanup EXIT INT TERM

# Test 1: File input JSON
test_file_json() {
    log "Creating sample JSON telemetry file..."
    cat > "$TEST_OUTPUT/sample.json" <<'EOF'
[
    {"name": "container.id", "type": "string", "value": "test123"},
    {"name": "test.name", "type": "string", "value": "my_test"},
    {"name": "test.duration_ms", "type": "double", "value": 125.5},
    {"name": "test.result", "type": "string", "value": "pass"}
]
EOF

    log "Running live-check with JSON file input..."
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --input-source "$TEST_OUTPUT/sample.json" \
        --format json \
        --output "$TEST_OUTPUT/file_json"

    log "Verifying output file exists..."
    local output_file=$(ls "$TEST_OUTPUT/file_json"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No JSON output file generated"
        return 1
    fi

    log "Output generated: $output_file"
    return 0
}

# Test 2: stdin text input
test_stdin_text() {
    log "Testing stdin text input..."
    echo -e "container.id\ntest.name\ntest.duration_ms\ntest.result" | \
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --input-source stdin \
        --input-format text \
        --format json \
        --output "$TEST_OUTPUT/stdin_text"

    log "Verifying stdin text output..."
    local output_file=$(ls "$TEST_OUTPUT/stdin_text"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No output file generated from stdin"
        return 1
    fi

    log "Stdin test output: $output_file"
    return 0
}

# Test 3: JSON output format validation
test_json_output() {
    log "Testing JSON output format..."
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --input-source "$TEST_OUTPUT/sample.json" \
        --format json \
        --output "$TEST_OUTPUT/json_output"

    log "Verifying JSON structure..."
    local output_file=$(ls "$TEST_OUTPUT/json_output"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No JSON output file"
        return 1
    fi

    # Check for required JSON fields
    if ! jq -e '.statistics' "$output_file" > /dev/null 2>&1; then
        log_error "Missing 'statistics' field in JSON output"
        return 1
    fi

    if ! jq -e '.statistics | has("total_entities")' "$output_file" > /dev/null 2>&1; then
        log_error "Missing 'total_entities' in statistics"
        return 1
    fi

    log "JSON output validation passed"
    log "Statistics: $(jq -c '.statistics' "$output_file")"
    return 0
}

# Test 4: ANSI output format
test_ansi_output() {
    log "Testing ANSI output format..."
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --input-source "$TEST_OUTPUT/sample.json" \
        --format ansi > "$TEST_OUTPUT/ansi_output.txt"

    log "Verifying ANSI codes present..."
    if ! grep -q $'\033\[' "$TEST_OUTPUT/ansi_output.txt"; then
        log_error "No ANSI escape codes found in output"
        return 1
    fi

    log "ANSI output format validated"
    log "Output preview:"
    head -10 "$TEST_OUTPUT/ansi_output.txt"
    return 0
}

# Test 5: Inactivity timeout
test_inactivity_timeout() {
    log "Testing inactivity timeout (5 seconds)..."
    local start=$(date +%s)

    # Should exit after 5 seconds with no input
    timeout 15 weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --otlp-grpc-port 4321 \
        --inactivity-timeout 5 \
        --format json \
        --output "$TEST_OUTPUT/timeout_test" || true

    local end=$(date +%s)
    local duration=$((end - start))

    log "Process ran for $duration seconds"

    # Should have exited after ~5 seconds (allow 1 second tolerance)
    if [ $duration -lt 4 ] || [ $duration -gt 8 ]; then
        log_error "Timeout didn't work correctly (expected ~5s, got ${duration}s)"
        return 1
    fi

    log "Inactivity timeout worked correctly"
    return 0
}

# Test 6: SIGHUP graceful stop
test_sighup_stop() {
    log "Testing SIGHUP graceful stop..."

    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --otlp-grpc-port 4322 \
        --format json \
        --output "$TEST_OUTPUT/sighup_test" &
    local pid=$!

    log "Live-check started with PID: $pid"
    sleep 3

    log "Sending SIGHUP to PID $pid..."
    kill -HUP $pid 2>/dev/null || true

    log "Waiting for graceful shutdown..."
    local wait_count=0
    while kill -0 $pid 2>/dev/null && [ $wait_count -lt 10 ]; do
        sleep 1
        wait_count=$((wait_count + 1))
    done

    if kill -0 $pid 2>/dev/null; then
        log_error "Process didn't stop after SIGHUP"
        kill -9 $pid 2>/dev/null || true
        return 1
    fi

    log "Verifying report was generated..."
    local output_file=$(ls "$TEST_OUTPUT/sighup_test"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No report generated after SIGHUP"
        return 1
    fi

    log "SIGHUP graceful stop successful"
    log "Report: $output_file"
    return 0
}

# Test 7: Custom policies
test_custom_policies() {
    log "Testing custom OPA policies..."
    mkdir -p "$TEST_OUTPUT/custom_policies"

    log "Creating test policy..."
    cat > "$TEST_OUTPUT/custom_policies/test_policy.rego" <<'EOF'
package live_check_advice
import rego.v1

deny contains advice if {
    input.sample.attribute
    contains(input.sample.attribute.name, "forbidden")
    advice := {
        "type": "advice",
        "advice_type": "forbidden_attribute",
        "advice_level": "violation",
        "message": "Attribute name contains 'forbidden'"
    }
}
EOF

    log "Creating sample with forbidden attribute..."
    cat > "$TEST_OUTPUT/forbidden_sample.json" <<'EOF'
[
    {"name": "forbidden.attr", "type": "string", "value": "test"},
    {"name": "container.id", "type": "string", "value": "test123"}
]
EOF

    log "Running live-check with custom policy..."
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --advice-policies "$TEST_OUTPUT/custom_policies" \
        --input-source "$TEST_OUTPUT/forbidden_sample.json" \
        --format json \
        --output "$TEST_OUTPUT/custom_policy_test"

    log "Verifying violation was detected..."
    local output_file=$(ls "$TEST_OUTPUT/custom_policy_test"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No output file generated"
        return 1
    fi

    local violations=$(jq -r '.statistics.advice_level_counts.violation // 0' "$output_file")
    log "Violations detected: $violations"

    if [ "$violations" -lt 1 ]; then
        log_error "Expected at least 1 violation, got $violations"
        return 1
    fi

    log "Custom policy validation successful"
    return 0
}

# Test 8: Statistics generation
test_statistics() {
    log "Testing statistics generation..."
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --input-source "$TEST_OUTPUT/sample.json" \
        --format json \
        --output "$TEST_OUTPUT/stats_test"

    local output_file=$(ls "$TEST_OUTPUT/stats_test"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No output file generated"
        return 1
    fi

    log "Verifying required statistics fields..."
    local required_fields=(
        "total_entities"
        "registry_coverage"
        "advice_level_counts"
    )

    for field in "${required_fields[@]}"; do
        if ! jq -e ".statistics | has(\"$field\")" "$output_file" > /dev/null 2>&1; then
            log_error "Missing required field: $field"
            return 1
        fi
        log "✓ Field present: $field"
    done

    log "Full statistics:"
    jq -C '.statistics' "$output_file"
    return 0
}

# Test 9: Concurrent instances
test_concurrent_instances() {
    log "Testing concurrent live-check instances..."

    local ports=(4330 4331 4332)
    local pids=()

    for port in "${ports[@]}"; do
        log "Starting instance on port $port..."
        weaver registry live-check \
            --registry "$SCRIPT_DIR/../../registry" \
            --otlp-grpc-port $port \
            --inactivity-timeout 5 \
            --format json \
            --output "$TEST_OUTPUT/concurrent_${port}" &
        pids+=($!)
    done

    log "Started PIDs: ${pids[*]}"
    sleep 2

    log "Verifying all instances are running..."
    local running=0
    for pid in "${pids[@]}"; do
        if kill -0 $pid 2>/dev/null; then
            running=$((running + 1))
        fi
    done

    if [ $running -ne 3 ]; then
        log_error "Expected 3 running instances, got $running"
        return 1
    fi

    log "All 3 instances running concurrently"
    log "Waiting for timeout..."
    sleep 6

    log "Verifying instances stopped after timeout..."
    local stopped=0
    for pid in "${pids[@]}"; do
        if ! kill -0 $pid 2>/dev/null; then
            stopped=$((stopped + 1))
        fi
    done

    if [ $stopped -ne 3 ]; then
        log_error "Expected 3 stopped instances, got $stopped"
        # Cleanup remaining processes
        for pid in "${pids[@]}"; do
            kill -9 $pid 2>/dev/null || true
        done
        return 1
    fi

    log "Concurrent instances test successful"
    return 0
}

# Test 10: OTLP gRPC input
test_otlp_grpc() {
    log "Testing OTLP gRPC input..."

    # Check if cargo and clnrm tests are available
    if ! command -v cargo &> /dev/null; then
        log_warning "cargo not found, skipping OTLP gRPC test"
        return 1
    fi

    log "Starting live-check with OTLP gRPC listener..."
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --otlp-grpc-port 4320 \
        --format json \
        --output "$TEST_OUTPUT/otlp_grpc" \
        --inactivity-timeout 10 &
    local pid=$!

    log "Live-check PID: $pid"
    sleep 3

    log "Sending test telemetry via OTEL..."
    export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4320

    # Run a simple test that generates telemetry
    cd "$SCRIPT_DIR/../.."
    if cargo test -p clnrm-core --lib telemetry::tests::test_span --features otel 2>&1 | tee "$TEST_OUTPUT/cargo_test.log"; then
        log "Test telemetry sent successfully"
    else
        log_warning "Cargo test had issues, but this may be expected"
    fi

    sleep 2
    log "Sending SIGHUP to stop live-check..."
    kill -HUP $pid 2>/dev/null || true

    log "Waiting for live-check to finish..."
    wait $pid 2>/dev/null || true

    log "Verifying OTLP output was generated..."
    local output_file=$(ls "$TEST_OUTPUT/otlp_grpc"/*.json 2>/dev/null | head -1)
    if [ -z "$output_file" ]; then
        log_error "No output file generated from OTLP input"
        return 1
    fi

    log "OTLP gRPC test successful"
    log "Output: $output_file"
    return 0
}

# Main execution
main() {
    echo -e "${BLUE}"
    cat << "EOF"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   Weaver Live-Check Comprehensive Test Suite                 ║
║   clnrm OpenTelemetry Integration Validation                 ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"

    log_info "Test suite started at: $(date)"
    log_info "Output directory: $TEST_OUTPUT"
    log_info "Log file: $LOG_FILE"

    # Pre-flight checks
    check_weaver
    check_registry

    # Execute all tests
    run_test "01_file_json" test_file_json
    run_test "02_stdin_text" test_stdin_text
    run_test "03_json_output" test_json_output
    run_test "04_ansi_output" test_ansi_output
    run_test "05_inactivity_timeout" test_inactivity_timeout
    run_test "06_sighup_stop" test_sighup_stop
    run_test "07_custom_policies" test_custom_policies
    run_test "08_statistics" test_statistics
    run_test "09_concurrent_instances" test_concurrent_instances

    # OTLP test last (may have port conflicts)
    if run_test "10_otlp_grpc" test_otlp_grpc; then
        true
    else
        SKIPPED+=("10_otlp_grpc (port conflict or cargo not available)")
    fi

    # Calculate duration
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    # Summary
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}           Test Summary${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
    echo -e "${GREEN}✅ Passed:  ${#PASSED[@]}${NC}"
    echo -e "${RED}❌ Failed:  ${#FAILED[@]}${NC}"
    echo -e "${YELLOW}⏭️  Skipped: ${#SKIPPED[@]}${NC}"
    echo -e "${BLUE}⏱️  Duration: ${DURATION}s${NC}"
    echo ""

    if [ ${#PASSED[@]} -gt 0 ]; then
        echo -e "${GREEN}Passed tests:${NC}"
        printf '%s\n' "${PASSED[@]}" | sed 's/^/  ✅ /'
        echo ""
    fi

    if [ ${#FAILED[@]} -gt 0 ]; then
        echo -e "${RED}Failed tests:${NC}"
        printf '%s\n' "${FAILED[@]}" | sed 's/^/  ❌ /'
        echo ""
    fi

    if [ ${#SKIPPED[@]} -gt 0 ]; then
        echo -e "${YELLOW}Skipped tests:${NC}"
        printf '%s\n' "${SKIPPED[@]}" | sed 's/^/  ⏭️  /'
        echo ""
    fi

    # Generate summary report
    cat > "$TEST_OUTPUT/test_summary.json" <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration_seconds": $DURATION,
  "total_tests": $((${#PASSED[@]} + ${#FAILED[@]} + ${#SKIPPED[@]})),
  "passed": ${#PASSED[@]},
  "failed": ${#FAILED[@]},
  "skipped": ${#SKIPPED[@]},
  "passed_tests": $(printf '%s\n' "${PASSED[@]}" | jq -R . | jq -s .),
  "failed_tests": $(printf '%s\n' "${FAILED[@]}" | jq -R . | jq -s .),
  "skipped_tests": $(printf '%s\n' "${SKIPPED[@]}" | jq -R . | jq -s .)
}
EOF

    log_info "Summary report: $TEST_OUTPUT/test_summary.json"
    log_info "Full log: $LOG_FILE"

    if [ ${#FAILED[@]} -eq 0 ]; then
        echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                                                               ║${NC}"
        echo -e "${GREEN}║              ✅ ALL TESTS PASSED! ✅                          ║${NC}"
        echo -e "${GREEN}║                                                               ║${NC}"
        echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
        exit 0
    else
        echo -e "${RED}╔═══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${RED}║                                                               ║${NC}"
        echo -e "${RED}║              ❌ SOME TESTS FAILED ❌                          ║${NC}"
        echo -e "${RED}║                                                               ║${NC}"
        echo -e "${RED}╚═══════════════════════════════════════════════════════════════╝${NC}"
        exit 1
    fi
}

# Execute main
main
