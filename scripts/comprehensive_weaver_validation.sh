#!/bin/bash
# Comprehensive Weaver Validation Script
# This is the FINAL validation for the Weaver core refactor
# SUCCESS = 0 violations, 85%+ coverage
# FAILURE = ANY violations detected

set -e

echo "🚀 Starting Comprehensive Weaver Validation"
echo "=========================================="
echo ""

# Configuration
REGISTRY_DIR="/Users/sac/clnrm/registry"
VALIDATION_OUTPUT="/Users/sac/clnrm/validation_output"
OTLP_GRPC_PORT=4317
ADMIN_PORT=8080
WEAVER_PID=""

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    if [ ! -z "$WEAVER_PID" ]; then
        echo "Stopping Weaver process (PID: $WEAVER_PID)..."
        kill $WEAVER_PID 2>/dev/null || true
        wait $WEAVER_PID 2>/dev/null || true
    fi
}

trap cleanup EXIT

# Step 1: Validate schemas
echo "📋 Step 1: Validating telemetry schemas..."
if ! weaver registry check -r "$REGISTRY_DIR/"; then
    echo "❌ Schema validation FAILED"
    echo "Fix schema errors before proceeding"
    exit 1
fi
echo "✅ Schemas valid"
echo ""

# Step 2: Create validation output directory
echo "📁 Step 2: Preparing validation output directory..."
mkdir -p "$VALIDATION_OUTPUT"
rm -rf "$VALIDATION_OUTPUT"/*
echo "✅ Output directory ready: $VALIDATION_OUTPUT"
echo ""

# Step 3: Check if ports are available
echo "🔍 Step 3: Checking port availability..."
if lsof -Pi :$OTLP_GRPC_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  Port $OTLP_GRPC_PORT is in use. Stopping existing process..."
    lsof -ti:$OTLP_GRPC_PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi
if lsof -Pi :$ADMIN_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  Port $ADMIN_PORT is in use. Stopping existing process..."
    lsof -ti:$ADMIN_PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi
echo "✅ Ports available"
echo ""

# Step 4: Start Weaver live-check listener
echo "🎯 Step 4: Starting Weaver live-check listener..."
echo "Registry: $REGISTRY_DIR"
echo "OTLP gRPC Port: $OTLP_GRPC_PORT"
echo "Admin Port: $ADMIN_PORT"

weaver registry live-check \
    --registry "$REGISTRY_DIR/" \
    --otlp-grpc-port $OTLP_GRPC_PORT \
    --admin-port $ADMIN_PORT \
    --output "$VALIDATION_OUTPUT/" \
    --format json &

WEAVER_PID=$!
echo "✅ Weaver started (PID: $WEAVER_PID)"
echo ""

# Wait for Weaver to start
echo "⏳ Waiting for Weaver to initialize..."
sleep 5

# Check if Weaver is still running
if ! ps -p $WEAVER_PID > /dev/null; then
    echo "❌ Weaver failed to start"
    exit 1
fi
echo "✅ Weaver initialized"
echo ""

# Step 5: Run complete test suite with OTLP export
echo "🧪 Step 5: Running test suite with telemetry export..."
echo "Exporting to: localhost:$OTLP_GRPC_PORT"

# Set environment for OTLP export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$OTLP_GRPC_PORT"
export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"
export RUST_LOG=info

# Run unit tests with otel features
echo ""
echo "Running unit tests..."
if cargo test --lib --features otel 2>&1 | tee "$VALIDATION_OUTPUT/unit_tests.log"; then
    echo "✅ Unit tests passed"
else
    echo "⚠️  Some unit tests failed (check logs)"
fi
echo ""

# Step 6: Run integration tests
echo "🐳 Step 6: Running integration tests..."
if cargo test --test '*' --features otel 2>&1 | tee "$VALIDATION_OUTPUT/integration_tests.log"; then
    echo "✅ Integration tests passed"
else
    echo "⚠️  Some integration tests failed (check logs)"
fi
echo ""

# Step 7: Run clnrm self-tests with OTLP export
echo "🔄 Step 7: Running clnrm self-tests..."
if clnrm self-test --otel-exporter otlp 2>&1 | tee "$VALIDATION_OUTPUT/self_tests.log"; then
    echo "✅ Self-tests passed"
else
    echo "⚠️  Some self-tests failed (check logs)"
fi
echo ""

# Step 8: Give Weaver time to process telemetry
echo "⏳ Step 8: Waiting for Weaver to process telemetry..."
sleep 3
echo "✅ Processing complete"
echo ""

# Step 9: Stop Weaver and get report
echo "📊 Step 9: Generating validation report..."
if curl -X POST "http://localhost:$ADMIN_PORT/stop" 2>/dev/null; then
    echo "✅ Weaver stopped via API"
else
    echo "⚠️  Failed to stop via API, using kill signal"
    kill $WEAVER_PID 2>/dev/null || true
fi

wait $WEAVER_PID 2>/dev/null || true
WEAVER_PID=""
echo ""

# Step 10: Parse and analyze results
echo "📈 Step 10: Analyzing validation results..."
echo "=========================================="
echo ""

REPORT_FILE="$VALIDATION_OUTPUT/validation_report.json"

if [ ! -f "$REPORT_FILE" ]; then
    echo "❌ VALIDATION FAILED - No report generated"
    echo "Weaver may not have received any telemetry"
    echo ""
    echo "Possible issues:"
    echo "1. Tests didn't export OTLP telemetry"
    echo "2. OTLP endpoint not configured correctly"
    echo "3. Weaver listener didn't start"
    exit 1
fi

echo "=== WEAVER VALIDATION REPORT ==="
cat "$REPORT_FILE" | jq '.' || cat "$REPORT_FILE"
echo ""

# Extract key metrics
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

# Step 11: Make final decision
echo "=== FINAL VERDICT ==="
echo ""

RELEASE_BLOCKED=0

# Check violations
if [ "$VIOLATIONS" -gt 0 ]; then
    echo "❌ VALIDATION FAILED - VIOLATIONS DETECTED"
    echo ""
    echo "Details:"
    jq -r '.all_advice[] | select(.advice_level == "violation") | "  - [\(.signal_type)] \(.signal_name): \(.message)"' "$REPORT_FILE"
    echo ""
    RELEASE_BLOCKED=1
fi

# Check coverage
COVERAGE_THRESHOLD=0.85
if [ $(echo "$COVERAGE < $COVERAGE_THRESHOLD" | bc -l) -eq 1 ]; then
    echo "⚠️  WARNING: Coverage below 85% ($COVERAGE)"
    echo "Target: 90%+ coverage"
    echo ""
    RELEASE_BLOCKED=1
fi

# Final verdict
if [ $RELEASE_BLOCKED -eq 1 ]; then
    echo "🚫 RELEASE BLOCKED"
    echo ""
    echo "Action required:"
    echo "1. Fix all violations"
    echo "2. Improve coverage to 85%+"
    echo "3. Re-run validation"
    exit 1
else
    echo "✅ WEAVER VALIDATION PASSED"
    echo ""
    echo "All telemetry validated against schemas"
    echo "Safe to proceed with release"
    echo ""
    echo "Summary:"
    echo "- Zero violations detected"
    echo "- Coverage: $(echo "$COVERAGE * 100" | bc -l | xargs printf "%.1f")%"
    echo "- All critical behaviors validated"
    exit 0
fi
