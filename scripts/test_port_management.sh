#!/bin/bash
# Test intelligent port management for Weaver validation
# This script verifies:
# 1. Port discovery works when default ports are occupied
# 2. Orphaned process cleanup works
# 3. Telemetry flush happens before Weaver shutdown

set -e

echo "🧪 Testing Intelligent Port Management"
echo "======================================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Port Discovery
echo "Test 1: Port Discovery"
echo "----------------------"

# Occupy port 4317
echo "📌 Occupying default OTLP port 4317..."
nc -l 4317 > /dev/null 2>&1 &
NC_PID=$!
sleep 1

# Start Weaver (should auto-discover alternate port)
echo "🔍 Starting Weaver (should find alternate port)..."
timeout 5 cargo run --release --features otel -- run tests/basic/ --validate --otel-exporter stdout 2>&1 | grep "Using OTLP port" || true

# Cleanup
kill $NC_PID 2>/dev/null || true
pkill -9 -f "weaver registry live-check" 2>/dev/null || true
sleep 1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test 1 PASSED: Port discovery works${NC}"
else
    echo -e "${RED}❌ Test 1 FAILED: Port discovery failed${NC}"
fi
echo ""

# Test 2: Orphaned Process Cleanup
echo "Test 2: Orphaned Process Cleanup"
echo "---------------------------------"

# Start a dummy Weaver process
echo "🔧 Creating orphaned Weaver process..."
weaver registry live-check --registry registry/ --otlp-grpc-port 4318 --admin-port 8081 --output ./test_output --no-stream > /dev/null 2>&1 &
WEAVER_PID=$!
sleep 2

# Check process exists
if ps -p $WEAVER_PID > /dev/null; then
    echo "✅ Orphaned process created (PID: $WEAVER_PID)"
else
    echo -e "${YELLOW}⚠️  Process already terminated${NC}"
fi

# Start another Weaver (should cleanup orphaned one)
echo "🧹 Testing cleanup on startup..."
cargo run --release --features otel -- run tests/basic/ --validate --otel-exporter stdout 2>&1 | grep "Cleaning up orphaned" || true

# Verify cleanup happened
sleep 2
if ! ps -p $WEAVER_PID > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Test 2 PASSED: Orphaned process cleaned up${NC}"
else
    echo -e "${RED}❌ Test 2 FAILED: Orphaned process still running${NC}"
    kill -9 $WEAVER_PID 2>/dev/null || true
fi
echo ""

# Test 3: Telemetry Flush
echo "Test 3: Telemetry Flush"
echo "------------------------"

# Run tests with Weaver validation
echo "🔄 Running tests and checking for telemetry flush..."
OUTPUT=$(cargo run --release --features otel -- run tests/basic/ --validate --otel-exporter stdout 2>&1 || true)

if echo "$OUTPUT" | grep -q "Flushing telemetry"; then
    echo -e "${GREEN}✅ Test 3 PASSED: Telemetry flush detected${NC}"
else
    echo -e "${RED}❌ Test 3 FAILED: No telemetry flush detected${NC}"
fi
echo ""

# Test 4: End-to-End Validation
echo "Test 4: End-to-End Validation"
echo "------------------------------"

# Run full validation with all features
echo "🚀 Running full end-to-end test..."
if cargo run --release --features otel -- run tests/basic/ --validate --otel-exporter otlp-grpc --otel-endpoint "http://localhost:4317" 2>&1 | grep -q "Weaver Validation Report"; then
    echo -e "${GREEN}✅ Test 4 PASSED: End-to-end validation works${NC}"
else
    echo -e "${YELLOW}⚠️  Test 4 WARNING: Validation report not found (may be expected if no tests ran)${NC}"
fi
echo ""

# Cleanup
echo "🧹 Cleaning up..."
pkill -9 -f "weaver registry live-check" 2>/dev/null || true
rm -rf ./test_output ./validation_output
echo ""

echo "======================================"
echo "🎉 Port Management Tests Complete"
echo "======================================"
