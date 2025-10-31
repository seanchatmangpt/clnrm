#!/usr/bin/env bash
set -euo pipefail

# Final Weaver Live-Check Validation
# Goal: 0 violations, 100% compliance

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VALIDATION_LOG="$PROJECT_ROOT/validation_output/final_validation.log"
RESULT_FILE="$PROJECT_ROOT/validation_output/final_results.json"
WEAVER_OUTPUT="$PROJECT_ROOT/validation_output/weaver_output.log"

mkdir -p "$PROJECT_ROOT/validation_output"

echo "========================================="
echo "FINAL WEAVER VALIDATION - 0 VIOLATIONS"
echo "========================================="
echo ""

# 1. PRE-FLIGHT CHECKS
echo "1. Pre-flight checks..."

# Check Weaver installed
if ! command -v weaver &> /dev/null; then
    echo "ERROR: Weaver not installed"
    echo '{"status":"error","reason":"weaver_not_installed"}' > "$RESULT_FILE"
    exit 1
fi
echo "  ✓ Weaver installed: $(weaver --version | head -1)"

# Clean orphaned processes
echo "  Cleaning orphaned Weaver processes..."
pkill -f "weaver.*live-check" || true
sleep 2

# Verify test files
if [ ! -d "$PROJECT_ROOT/tests/telemetry_validation" ]; then
    echo "ERROR: Test directory not found"
    echo '{"status":"error","reason":"tests_not_found"}' > "$RESULT_FILE"
    exit 1
fi
echo "  ✓ Test files found"

# Verify registry
if ! weaver registry check -r "$PROJECT_ROOT/registry/" &> /dev/null; then
    echo "ERROR: Registry validation failed"
    echo '{"status":"error","reason":"invalid_registry"}' > "$RESULT_FILE"
    exit 1
fi
echo "  ✓ Registry schemas valid"

# Build with OTEL features
echo "  Building with OTEL features..."
cd "$PROJECT_ROOT"
if ! cargo build --release --features otel &> "$VALIDATION_LOG"; then
    echo "ERROR: Build failed"
    echo '{"status":"error","reason":"build_failed"}' > "$RESULT_FILE"
    exit 1
fi
echo "  ✓ Build successful"

# 2. PORT MANAGEMENT
echo ""
echo "2. Port management..."

find_free_port() {
    local start_port=5000
    local end_port=9000

    for port in $(seq $start_port $end_port); do
        if ! lsof -i ":$port" &> /dev/null; then
            echo "$port"
            return 0
        fi
    done

    echo "ERROR: No free ports in range $start_port-$end_port"
    return 1
}

OTLP_PORT=$(find_free_port)
if [ -z "$OTLP_PORT" ]; then
    echo '{"status":"error","reason":"no_free_ports"}' > "$RESULT_FILE"
    exit 1
fi
echo "  ✓ Using port: $OTLP_PORT"

# 3. WEAVER STARTUP
echo ""
echo "3. Starting Weaver live-check..."

# Start Weaver with dynamic port
weaver registry live-check \
    -r "$PROJECT_ROOT/registry/" \
    --otlp-grpc-port "$OTLP_PORT" \
    --inactivity-timeout 20 \
    > "$WEAVER_OUTPUT" 2>&1 &

WEAVER_PID=$!
echo "  Started Weaver (PID: $WEAVER_PID)"

# Wait for Weaver to be ready (check for admin UI port)
wait_for_weaver() {
    local timeout=$1
    local elapsed=0
    local admin_port=4320

    while [ $elapsed -lt $timeout ]; do
        # Check if admin port is listening (indicates Weaver is ready)
        if lsof -i ":$admin_port" &> /dev/null; then
            return 0
        fi

        # Check if process died
        if ! kill -0 $WEAVER_PID 2>/dev/null; then
            echo "ERROR: Weaver process died"
            return 1
        fi

        sleep 1
        elapsed=$((elapsed + 1))
    done

    echo "ERROR: Weaver did not start within ${timeout}s"
    return 1
}

if ! wait_for_weaver 10; then
    cat "$WEAVER_OUTPUT"
    echo '{"status":"error","reason":"weaver_startup_failed"}' > "$RESULT_FILE"
    kill $WEAVER_PID 2>/dev/null || true
    exit 1
fi

# Extra sleep to ensure OTLP receiver is fully ready
sleep 2
echo "  ✓ Weaver ready (listening on OTLP port $OTLP_PORT)"

# 4. TEST EXECUTION WITH FLUSH
echo ""
echo "4. Executing tests with telemetry..."

# Set environment for aggressive flushing
export OTEL_BSP_SCHEDULE_DELAY=100
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$OTLP_PORT"

# Run tests
TEST_OUTPUT="$PROJECT_ROOT/validation_output/test_output.log"
if "$PROJECT_ROOT/target/release/clnrm" run tests/telemetry_validation \
    --otel-exporter otlp-grpc \
    --otel-endpoint "http://localhost:$OTLP_PORT" \
    > "$TEST_OUTPUT" 2>&1; then
    echo "  ✓ Tests completed"
else
    echo "  ⚠ Tests completed with errors (checking telemetry anyway)"
fi

# Wait for telemetry to flush and Weaver to process
echo "  Waiting for telemetry flush and Weaver processing..."
sleep 8

# Signal Weaver to finish via HTTP API (cleaner than SIGTERM)
echo "  Signaling Weaver to stop..."
curl -X POST http://localhost:4320/stop &> /dev/null || kill -HUP $WEAVER_PID 2>/dev/null || true

# Wait for Weaver to finish processing
sleep 3

# 5. RESULT COLLECTION
echo ""
echo "5. Analyzing results..."

# Parse Weaver output
SAMPLES=$(grep -o "samples received: [0-9]*" "$WEAVER_OUTPUT" | tail -1 | grep -o "[0-9]*" || echo "0")
VIOLATIONS=$(grep -o "violations detected: [0-9]*" "$WEAVER_OUTPUT" | tail -1 | grep -o "[0-9]*" || echo "0")

# Calculate coverage
TOTAL_EVENTS=$(grep -c "name:" "$PROJECT_ROOT/registry/clnrm-spans.yaml" || echo "1")
COVERAGE=0
if [ "$SAMPLES" -gt 0 ] && [ "$TOTAL_EVENTS" -gt 0 ]; then
    COVERAGE=$((SAMPLES * 100 / TOTAL_EVENTS))
fi

echo ""
echo "========================================="
echo "VALIDATION RESULTS"
echo "========================================="
echo "Samples Received: $SAMPLES"
echo "Violations:       $VIOLATIONS"
echo "Coverage:         ${COVERAGE}%"
echo ""

# Generate JSON result
cat > "$RESULT_FILE" <<EOF
{
  "status": "completed",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "samples_received": $SAMPLES,
  "violations": $VIOLATIONS,
  "coverage_percent": $COVERAGE,
  "port_used": $OTLP_PORT,
  "weaver_output": "$WEAVER_OUTPUT",
  "test_output": "$TEST_OUTPUT"
}
EOF

# 6. SUCCESS CRITERIA CHECK
if [ "$VIOLATIONS" -eq 0 ] && [ "$SAMPLES" -gt 0 ]; then
    echo "✅ VALIDATION PASSED: 0 violations, $SAMPLES samples received"
    echo '{"validation_status":"PASSED"}' >> "$RESULT_FILE"
    exit 0
else
    echo "❌ VALIDATION FAILED"
    if [ "$SAMPLES" -eq 0 ]; then
        echo "   Reason: No samples received"
    fi
    if [ "$VIOLATIONS" -gt 0 ]; then
        echo "   Reason: $VIOLATIONS violations detected"
    fi
    echo '{"validation_status":"FAILED"}' >> "$RESULT_FILE"

    # Show relevant Weaver output
    echo ""
    echo "Weaver output (last 50 lines):"
    tail -50 "$WEAVER_OUTPUT"

    exit 1
fi
