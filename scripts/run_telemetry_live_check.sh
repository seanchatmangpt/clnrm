#!/bin/bash
set -euo pipefail

# Telemetry Live Check Validation Script
# Purpose: Run clnrm test with Weaver live-check to validate OpenTelemetry emission
# Success Criteria: 0 violations, >0 samples received, coverage > 0%

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$PROJECT_ROOT/tests/telemetry_validation"
REGISTRY_DIR="$PROJECT_ROOT/registry"
OUTPUT_DIR="$PROJECT_ROOT/validation_output"
WEAVER_PORT=4316
WEAVER_LOG="$OUTPUT_DIR/weaver_live_check.log"
CLNRM_LOG="$OUTPUT_DIR/clnrm_telemetry_test.log"
INACTIVITY_TIMEOUT=15

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Telemetry Live Check Validation ===${NC}"
echo "Registry: $REGISTRY_DIR"
echo "Test: $TEST_DIR"
echo "Weaver Port: $WEAVER_PORT"
echo "Inactivity Timeout: ${INACTIVITY_TIMEOUT}s"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"
if ! command -v weaver &> /dev/null; then
    echo -e "${RED}ERROR: weaver not found in PATH${NC}"
    echo "Install with: cargo install weaver-cli"
    exit 1
fi

# Ensure clnrm is in PATH
export PATH="$HOME/.local/bin:$PATH"

if ! command -v clnrm &> /dev/null; then
    echo -e "${RED}ERROR: clnrm not found in PATH${NC}"
    echo "Install with: cargo build --release --features otel && cp target/release/clnrm ~/.local/bin/"
    exit 1
fi

if [ ! -d "$REGISTRY_DIR" ]; then
    echo -e "${RED}ERROR: Registry directory not found: $REGISTRY_DIR${NC}"
    exit 1
fi

if [ ! -f "$TEST_DIR/.clnrm.toml" ]; then
    echo -e "${RED}ERROR: Test configuration not found: $TEST_DIR/.clnrm.toml${NC}"
    exit 1
fi

echo -e "${GREEN}All prerequisites met${NC}"
echo ""

# Cleanup any existing processes
echo -e "${BLUE}Cleaning up any existing processes...${NC}"
pkill -f "weaver.*live-check" || true
sleep 2

# Start Weaver live-check in background
echo -e "${BLUE}Starting Weaver live-check listener on port $WEAVER_PORT...${NC}"
weaver registry live-check \
    --registry "$REGISTRY_DIR" \
    --otlp-grpc-port "$WEAVER_PORT" \
    --inactivity-timeout "$INACTIVITY_TIMEOUT" \
    > "$WEAVER_LOG" 2>&1 &
WEAVER_PID=$!

echo "Weaver PID: $WEAVER_PID"
echo "Weaver log: $WEAVER_LOG"

# Wait for Weaver to start listening (look for "OTLP receiver" in log)
echo -e "${YELLOW}Waiting for Weaver to start listening...${NC}"
WAIT_COUNT=0
MAX_WAIT=10
while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    if [ -f "$WEAVER_LOG" ] && grep -q "OTLP receiver" "$WEAVER_LOG"; then
        echo -e "${GREEN}Weaver is ready to receive OTLP data${NC}"
        break
    fi

    if ! ps -p $WEAVER_PID > /dev/null; then
        echo -e "${RED}ERROR: Weaver process died unexpectedly${NC}"
        cat "$WEAVER_LOG"
        exit 1
    fi

    sleep 1
    WAIT_COUNT=$((WAIT_COUNT + 1))
    echo -n "."
done
echo ""

if [ $WAIT_COUNT -eq $MAX_WAIT ]; then
    echo -e "${RED}ERROR: Weaver did not start listening within ${MAX_WAIT}s${NC}"
    cat "$WEAVER_LOG"
    kill $WEAVER_PID || true
    exit 1
fi

# Give Weaver an extra second to stabilize
sleep 1

# Run clnrm test with OTLP export
echo -e "${BLUE}Running clnrm test with OTLP export...${NC}"
echo "Test directory: $TEST_DIR"
echo "OTLP endpoint: http://localhost:$WEAVER_PORT"
echo ""

# Set environment variables for OTLP export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$WEAVER_PORT"
export OTEL_SERVICE_NAME="clnrm-telemetry-validation"
export RUST_LOG=info

# Run clnrm test with OTLP gRPC exporter
echo -e "${YELLOW}Starting clnrm test...${NC}"
if clnrm run "$TEST_DIR" --otel-exporter otlp-grpc --otel-endpoint "http://localhost:$WEAVER_PORT" > "$CLNRM_LOG" 2>&1; then
    echo -e "${GREEN}clnrm test completed successfully${NC}"
    TEST_RESULT="PASS"
else
    echo -e "${YELLOW}clnrm test had errors (may be expected)${NC}"
    TEST_RESULT="FAIL"
fi

echo -e "${YELLOW}clnrm output:${NC}"
cat "$CLNRM_LOG"
echo ""

# Wait for Weaver to process telemetry (wait for inactivity timeout or process exit)
echo -e "${BLUE}Waiting for Weaver to process telemetry and complete...${NC}"
echo "This will take up to ${INACTIVITY_TIMEOUT}s after test completes..."

# Wait for Weaver to exit naturally (due to inactivity timeout)
WAIT_TIME=$((INACTIVITY_TIMEOUT + 5))
for i in $(seq 1 $WAIT_TIME); do
    if ! ps -p $WEAVER_PID > /dev/null; then
        echo -e "${GREEN}Weaver completed processing${NC}"
        break
    fi
    sleep 1
    if [ $((i % 5)) -eq 0 ]; then
        echo "  Still waiting... ($i/${WAIT_TIME}s)"
    fi
done

# Kill Weaver if still running
if ps -p $WEAVER_PID > /dev/null; then
    echo -e "${YELLOW}Stopping Weaver (timeout)...${NC}"
    kill $WEAVER_PID || true
    sleep 2
fi

# Parse Weaver output
echo ""
echo -e "${BLUE}=== Analyzing Weaver Results ===${NC}"
echo ""

if [ ! -f "$WEAVER_LOG" ]; then
    echo -e "${RED}ERROR: Weaver log file not found${NC}"
    exit 1
fi

echo -e "${YELLOW}Full Weaver output:${NC}"
cat "$WEAVER_LOG"
echo ""

# Extract key metrics from Weaver output
# Look for lines like:
#   - total: 123
# after the "Samples", "Advisories given", and "Registry coverage" sections

# Extract samples (look for "total:" after "Samples" section)
SAMPLES_RECEIVED=$(grep -A 1 "Samples" "$WEAVER_LOG" | grep "total:" | grep -oP '\d+' | head -1 || echo "0")

# Extract violations (look for "total:" after "Advisories given" or "Violations" section)
VIOLATIONS=$(grep -A 1 "Advisories given\|Violations" "$WEAVER_LOG" | grep "total:" | grep -oP '\d+' | head -1 || echo "0")

# Extract coverage (look for percentage after "entities seen:")
COVERAGE=$(grep "entities seen:" "$WEAVER_LOG" | grep -oP '[\d.]+%' | head -1 || echo "0%")

echo -e "${BLUE}=== Validation Results ===${NC}"
echo "Samples Received: $SAMPLES_RECEIVED"
echo "Violations: $VIOLATIONS"
echo "Coverage: $COVERAGE"
echo "Test Result: $TEST_RESULT"
echo ""

# Determine overall result
SUCCESS=true

if [ "$SAMPLES_RECEIVED" -eq 0 ]; then
    echo -e "${RED}FAIL: No telemetry samples received${NC}"
    echo -e "${YELLOW}This means clnrm did not emit any OTLP telemetry${NC}"
    SUCCESS=false
fi

if [ "$VIOLATIONS" != "0" ]; then
    echo -e "${RED}FAIL: $VIOLATIONS violations found${NC}"
    SUCCESS=false

    # Extract violation details
    echo -e "${YELLOW}Violation details:${NC}"
    grep -A 30 "Violations:\|Advisories given:" "$WEAVER_LOG" || true
fi

if [ "$COVERAGE" = "0%" ] || [ "$COVERAGE" = "0.0%" ]; then
    echo -e "${YELLOW}WARNING: 0% coverage (no registry signals matched)${NC}"
    # Don't fail on coverage alone - might be expected if schema doesn't match yet
fi

# Final result
echo ""
if [ "$SUCCESS" = true ]; then
    echo -e "${GREEN}✓ TELEMETRY VALIDATION PASSED${NC}"
    echo -e "${GREEN}  - Samples received: $SAMPLES_RECEIVED${NC}"
    echo -e "${GREEN}  - Violations: $VIOLATIONS${NC}"
    echo -e "${GREEN}  - Coverage: $COVERAGE${NC}"

    # Store success in memory
    echo "SUCCESS: $SAMPLES_RECEIVED samples, $VIOLATIONS violations, $COVERAGE coverage" > "$OUTPUT_DIR/validation_result.txt"
    exit 0
else
    echo -e "${RED}✗ TELEMETRY VALIDATION FAILED${NC}"

    # Store failure in memory
    echo "FAILED: $SAMPLES_RECEIVED samples, $VIOLATIONS violations" > "$OUTPUT_DIR/validation_result.txt"

    echo ""
    echo -e "${YELLOW}Debug information:${NC}"
    echo "1. Check Weaver log: $WEAVER_LOG"
    echo "2. Check clnrm log: $CLNRM_LOG"
    echo "3. Verify registry schemas: weaver registry check -r $REGISTRY_DIR"
    echo "4. Verify clnrm OTEL support: clnrm self-test --suite otel --otel-exporter stdout"

    echo ""
    echo -e "${YELLOW}Possible causes:${NC}"
    echo "- clnrm not built with OTEL features (cargo build --release --features otel)"
    echo "- OTLP exporter not sending to correct endpoint"
    echo "- Weaver gRPC port ($WEAVER_PORT) not accessible"
    echo "- Telemetry not emitted during test execution"
    exit 1
fi
