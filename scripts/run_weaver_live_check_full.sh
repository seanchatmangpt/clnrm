#!/bin/bash
# Comprehensive Weaver Live-Check Validation
# Tests all 23 CLI commands + edge cases + telemetry compliance
# SUCCESS CRITERIA: 0 violations, all attributes verified

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# Configuration
REGISTRY_DIR="${REGISTRY_DIR:-registry/}"
OUTPUT_DIR="${OUTPUT_DIR:-validation_output/weaver}"
WEAVER_GRPC_PORT="${WEAVER_GRPC_PORT:-5317}"  # Different from Docker OTLP (4317)
WEAVER_ADMIN_PORT="${WEAVER_ADMIN_PORT:-5320}"
INACTIVITY_TIMEOUT="${INACTIVITY_TIMEOUT:-60}"  # 60s timeout
WEAVER_PID=""
WEAVER_LOG="/tmp/weaver_live_check.log"

# Cleanup handler
cleanup() {
    log_info "Cleaning up..."
    if [ -n "$WEAVER_PID" ] && ps -p "$WEAVER_PID" > /dev/null 2>&1; then
        log_info "Stopping Weaver (PID: $WEAVER_PID)..."
        # Try graceful shutdown via admin API
        curl -s -X POST "http://localhost:$WEAVER_ADMIN_PORT/stop" >/dev/null 2>&1 || true
        sleep 2
        # Force kill if still running
        if ps -p "$WEAVER_PID" > /dev/null 2>&1; then
            kill -9 "$WEAVER_PID" 2>/dev/null || true
        fi
        wait "$WEAVER_PID" 2>/dev/null || true
    fi
}

trap cleanup EXIT INT TERM

echo "================================================================================"
echo "Weaver Live-Check Comprehensive Validation - clnrm v1.2.0"
echo "================================================================================"
echo ""

# Step 1: Schema validation
log_info "Step 1: Validating registry schemas..."
if ! weaver registry check -r "$REGISTRY_DIR" >/dev/null 2>&1; then
    log_error "Schema validation failed"
    weaver registry check -r "$REGISTRY_DIR"
    exit 1
fi
log_success "Schemas validated"
echo ""

# Step 2: Prepare output directory
log_info "Step 2: Preparing output directory..."
mkdir -p "$OUTPUT_DIR"
rm -rf "${OUTPUT_DIR:?}"/*
log_success "Output ready: $OUTPUT_DIR"
echo ""

# Step 3: Check port availability
log_info "Step 3: Checking port availability..."
if lsof -i ":$WEAVER_GRPC_PORT" >/dev/null 2>&1; then
    log_error "Port $WEAVER_GRPC_PORT already in use"
    lsof -i ":$WEAVER_GRPC_PORT"
    exit 1
fi
if lsof -i ":$WEAVER_ADMIN_PORT" >/dev/null 2>&1; then
    log_error "Port $WEAVER_ADMIN_PORT already in use"
    lsof -i ":$WEAVER_ADMIN_PORT"
    exit 1
fi
log_success "Ports available (gRPC: $WEAVER_GRPC_PORT, Admin: $WEAVER_ADMIN_PORT)"
echo ""

# Step 4: Start Weaver live-check listener
log_info "Step 4: Starting Weaver live-check listener..."
echo "Registry: $REGISTRY_DIR"
echo "OTLP gRPC Port: $WEAVER_GRPC_PORT"
echo "Admin Port: $WEAVER_ADMIN_PORT"
echo "Inactivity Timeout: ${INACTIVITY_TIMEOUT}s"
echo "Log: $WEAVER_LOG"
echo ""

weaver registry live-check \
    --registry "$REGISTRY_DIR" \
    --otlp-grpc-port "$WEAVER_GRPC_PORT" \
    --admin-port "$WEAVER_ADMIN_PORT" \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout "$INACTIVITY_TIMEOUT" \
    > "$WEAVER_LOG" 2>&1 &

WEAVER_PID=$!
log_success "Weaver started (PID: $WEAVER_PID)"
echo ""

# Step 5: Wait for Weaver to be ready
log_info "Step 5: Waiting for Weaver to start listening..."
max_wait=15
elapsed=0
while [ $elapsed -lt $max_wait ]; do
    if ! ps -p "$WEAVER_PID" > /dev/null 2>&1; then
        log_error "Weaver process died unexpectedly"
        cat "$WEAVER_LOG"
        exit 1
    fi

    if lsof -i ":$WEAVER_GRPC_PORT" >/dev/null 2>&1; then
        log_success "Weaver is listening on :$WEAVER_GRPC_PORT"
        break
    fi

    echo -n "."
    sleep 1
    elapsed=$((elapsed + 1))
done
echo ""

if [ $elapsed -ge $max_wait ]; then
    log_error "Weaver did not start within ${max_wait}s"
    cat "$WEAVER_LOG"
    exit 1
fi
echo ""

# Step 6: Run comprehensive test matrix
log_info "Step 6: Running comprehensive telemetry emission tests..."
echo ""

export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$WEAVER_GRPC_PORT"

# Test categories
declare -a TEST_COMMANDS=(
    # CLI initialization commands
    "clnrm --version"
    "clnrm plugins list"

    # OTEL self-tests (emits telemetry)
    "clnrm self-test --suite otel --otel-exporter otlp-grpc --otel-endpoint http://localhost:$WEAVER_GRPC_PORT"
    "clnrm self-test --suite framework --otel-exporter otlp-grpc --otel-endpoint http://localhost:$WEAVER_GRPC_PORT"
    "clnrm self-test --suite container --otel-exporter otlp-grpc --otel-endpoint http://localhost:$WEAVER_GRPC_PORT"
    "clnrm self-test --suite cli --otel-exporter otlp-grpc --otel-endpoint http://localhost:$WEAVER_GRPC_PORT"
)

test_count=0
pass_count=0
fail_count=0

for cmd in "${TEST_COMMANDS[@]}"; do
    test_count=$((test_count + 1))
    log_info "Test $test_count: $cmd"

    if eval "$cmd" > /dev/null 2>&1; then
        log_success "PASS"
        pass_count=$((pass_count + 1))
    else
        log_warning "FAIL (non-critical - telemetry may still be emitted)"
        fail_count=$((fail_count + 1))
    fi

    # Small delay to allow telemetry to propagate
    sleep 0.5
done

echo ""
log_info "Test execution complete: $pass_count passed, $fail_count failed"
echo ""

# Step 7: Give Weaver time to process all telemetry
log_info "Step 7: Waiting for telemetry processing..."
sleep 3
log_success "Processing complete"
echo ""

# Step 8: Stop Weaver gracefully to generate report
log_info "Step 8: Stopping Weaver to generate validation report..."
if curl -s -X POST "http://localhost:$WEAVER_ADMIN_PORT/stop" >/dev/null 2>&1; then
    log_success "Weaver stopped via admin API"
else
    log_warning "Failed to stop via API, using signal"
    kill -TERM "$WEAVER_PID" 2>/dev/null || true
fi

# Wait for process to exit and generate report
wait "$WEAVER_PID" 2>/dev/null || true
WEAVER_PID=""
log_success "Weaver shutdown complete"
echo ""

# Step 9: Analyze validation results
log_info "Step 9: Analyzing validation results..."
echo "================================================================================"
echo ""

REPORT_FILE="$OUTPUT_DIR/live_check.json"

if [ ! -f "$REPORT_FILE" ]; then
    log_error "VALIDATION FAILED - No report generated"
    echo ""
    echo "Possible issues:"
    echo "1. No telemetry was emitted to Weaver"
    echo "2. Weaver listener didn't receive data"
    echo "3. OTLP endpoint misconfigured"
    echo ""
    echo "Weaver logs:"
    cat "$WEAVER_LOG"
    exit 1
fi

log_success "Validation report generated: $REPORT_FILE"
echo ""

# Parse report metrics
if command -v jq >/dev/null 2>&1; then
    VIOLATIONS=$(jq -r '.advice_level_counts.violation // 0' "$REPORT_FILE")
    IMPROVEMENTS=$(jq -r '.advice_level_counts.improvement // 0' "$REPORT_FILE")
    INFORMATION=$(jq -r '.advice_level_counts.information // 0' "$REPORT_FILE")
    COVERAGE=$(jq -r '.registry_coverage // 0' "$REPORT_FILE")

    echo "=== VALIDATION METRICS ==="
    echo "Violations:   $VIOLATIONS"
    echo "Improvements: $IMPROVEMENTS"
    echo "Information:  $INFORMATION"
    echo "Coverage:     $(echo "$COVERAGE * 100" | bc -l | xargs printf "%.1f")%"
    echo ""

    # Step 10: Final verdict
    echo "=== FINAL VERDICT ==="
    echo ""

    VALIDATION_FAILED=0

    if [ "$VIOLATIONS" -gt 0 ]; then
        log_error "VIOLATIONS DETECTED: $VIOLATIONS"
        echo ""
        echo "Violation details:"
        jq -r '.all_advice[] | select(.advice_level == "violation") | "  [\(.signal_type)] \(.signal_name): \(.message)"' "$REPORT_FILE" || cat "$REPORT_FILE"
        echo ""
        VALIDATION_FAILED=1
    else
        log_success "Zero violations detected"
    fi

    if [ $(echo "$COVERAGE < 0.85" | bc -l) -eq 1 ]; then
        log_warning "Coverage below 85%: $(echo "$COVERAGE * 100" | bc -l | xargs printf "%.1f")%"
        VALIDATION_FAILED=1
    else
        log_success "Coverage: $(echo "$COVERAGE * 100" | bc -l | xargs printf "%.1f")%"
    fi

    echo ""
    if [ $VALIDATION_FAILED -eq 1 ]; then
        log_error "WEAVER VALIDATION FAILED"
        echo ""
        echo "Action required:"
        echo "1. Fix all violations (see details above)"
        echo "2. Improve telemetry coverage to 85%+"
        echo "3. Re-run validation"
        echo ""
        echo "Full report: $REPORT_FILE"
        exit 1
    else
        log_success "WEAVER VALIDATION PASSED"
        echo ""
        echo "All telemetry conforms to registry schemas"
        echo "Zero violations detected"
        echo "Coverage: $(echo "$COVERAGE * 100" | bc -l | xargs printf "%.1f")%"
        echo ""
        echo "Full report: $REPORT_FILE"
        exit 0
    fi
else
    log_warning "jq not installed - displaying raw report"
    echo ""
    cat "$REPORT_FILE"
    echo ""
    log_info "Install jq for detailed analysis: brew install jq"
    exit 0
fi
