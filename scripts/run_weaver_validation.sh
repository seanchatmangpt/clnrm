#!/bin/bash
# Comprehensive Weaver Validation Script
# Handles all 5 failure modes documented in weaver-failure-modes.puml

set -e  # Exit on error

# Configuration
REGISTRY="registry/"
OUTPUT="validation_output/"
OTLP_PORT=4317
ADMIN_PORT=8080
TIMEOUT=300  # 5 minutes

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================================================"
echo "Weaver Live Check Validation - clnrm v1.2.0"
echo "================================================================================"
echo ""

# ========== PRE-FLIGHT CHECKS ==========
echo "🔍 Running pre-flight checks..."
echo ""

# 1. Check Docker (Failure Mode #3)
echo -n "Checking Docker daemon... "
if ! docker ps > /dev/null 2>&1; then
    echo -e "${RED}❌ FAILED${NC}"
    echo ""
    echo "Docker daemon is not running."
    echo ""
    echo "To start Docker:"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  1. Open Docker Desktop application"
        echo "  2. Wait for Docker to start (whale icon in menu bar)"
        echo "  3. Verify with: docker ps"
    else
        echo "  sudo systemctl start docker"
    fi
    echo ""
    exit 1
fi
echo -e "${GREEN}✓${NC}"

# 2. Check port availability (Failure Mode #4)
echo -n "Checking port $OTLP_PORT availability... "
if lsof -i :$OTLP_PORT > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Port in use${NC}"
    echo ""
    echo "Cleaning up existing process on port $OTLP_PORT..."
    EXISTING_PID=$(lsof -t -i :$OTLP_PORT)
    if [ -n "$EXISTING_PID" ]; then
        echo "  Stopping process $EXISTING_PID..."
        kill -HUP $EXISTING_PID 2>/dev/null || true
        sleep 2

        # Force kill if still running
        if lsof -i :$OTLP_PORT > /dev/null 2>&1; then
            echo "  Force killing process..."
            kill -9 $EXISTING_PID 2>/dev/null || true
            sleep 1
        fi
    fi

    # Verify port is now free
    if lsof -i :$OTLP_PORT > /dev/null 2>&1; then
        echo -e "${RED}❌ Failed to free port $OTLP_PORT${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Port freed${NC}"
else
    echo -e "${GREEN}✓${NC}"
fi

# 3. Check Weaver installed
echo -n "Checking Weaver installation... "
if ! command -v weaver &> /dev/null; then
    echo -e "${RED}❌ FAILED${NC}"
    echo ""
    echo "Weaver is not installed."
    echo ""
    echo "To install Weaver:"
    echo "  cargo install weaver"
    echo ""
    exit 1
fi
echo -e "${GREEN}✓${NC}"

# 4. Validate registry (Root Cause #5)
echo -n "Validating registry schemas... "
if ! weaver registry check --registry $REGISTRY > /dev/null 2>&1; then
    echo -e "${RED}❌ FAILED${NC}"
    echo ""
    echo "Registry validation failed. Running with output:"
    weaver registry check --registry $REGISTRY
    exit 1
fi
echo -e "${GREEN}✓${NC}"

# 5. Create output directory
echo -n "Creating output directory... "
mkdir -p $OUTPUT
echo -e "${GREEN}✓${NC}"

echo ""
echo -e "${GREEN}✅ All pre-flight checks passed${NC}"
echo ""

# ========== START WEAVER ==========
echo "🚀 Starting Weaver Live Check..."
echo ""

weaver registry live-check \
    --registry $REGISTRY \
    --otlp-grpc-port $OTLP_PORT \
    --admin-port $ADMIN_PORT \
    --format json \
    --output $OUTPUT \
    --inactivity-timeout $TIMEOUT &

WEAVER_PID=$!
echo "Weaver PID: $WEAVER_PID"
echo "Waiting for Weaver to start..."
sleep 3

# Verify Weaver started
if ! ps -p $WEAVER_PID > /dev/null 2>&1; then
    echo -e "${RED}❌ Weaver failed to start${NC}"
    echo ""
    echo "Check logs for errors"
    exit 1
fi

# Verify Weaver is listening
echo -n "Verifying Weaver is listening on :$OTLP_PORT... "
if ! lsof -i :$OTLP_PORT > /dev/null 2>&1; then
    echo -e "${RED}❌ FAILED${NC}"
    echo ""
    echo "Weaver process is running but not listening on port $OTLP_PORT"
    kill $WEAVER_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✓${NC}"

echo ""
echo -e "${GREEN}✅ Weaver listening on :$OTLP_PORT${NC}"
echo ""

# ========== RUN TESTS ==========
echo "🧪 Running tests with OTLP export..."
echo ""

# Set environment variable for tests to export to Weaver (Root Cause #4 fix)
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$OTLP_PORT"

echo "Environment configured:"
echo "  OTEL_EXPORTER_OTLP_ENDPOINT=$OTEL_EXPORTER_OTLP_ENDPOINT"
echo ""

# Run Docker integration tests specifically
echo "Running Docker integration tests..."
if ! cargo test -p clnrm-core --test docker_integration --features otel -- --test-threads=1; then
    echo -e "${RED}❌ Tests failed${NC}"
    echo ""
    echo "Stopping Weaver..."
    kill -HUP $WEAVER_PID 2>/dev/null || true
    wait $WEAVER_PID 2>/dev/null || true
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Tests passed${NC}"
echo ""

# ========== STOP WEAVER ==========
echo "📊 Stopping Weaver and generating report..."
echo ""

# Stop Weaver gracefully (sends SIGHUP to trigger report generation)
kill -HUP $WEAVER_PID
wait $WEAVER_PID 2>/dev/null || true
WEAVER_EXIT=$?

echo "Weaver exit code: $WEAVER_EXIT"
echo ""

# ========== VALIDATE REPORT ==========
echo "📋 Validating Weaver report..."
echo ""

REPORT="$OUTPUT/live_check.json"

# 1. Check report exists
if [ ! -f "$REPORT" ]; then
    echo -e "${RED}❌ No report generated${NC}"
    echo ""
    echo "Expected report at: $REPORT"
    exit 1
fi
echo -e "${GREEN}✓ Report generated${NC}"

# 2. Check for telemetry (Root Cause #4 verification)
SAMPLES=$(jq '.samples | length' $REPORT)
echo "Samples received: $SAMPLES"

if [ "$SAMPLES" -eq 0 ]; then
    echo -e "${RED}❌ No telemetry received${NC}"
    echo ""
    echo "Root cause: Tests did not export telemetry to Weaver"
    echo ""
    echo "Possible issues:"
    echo "  1. Tests not using OTEL_EXPORTER_OTLP_ENDPOINT"
    echo "  2. OTLP exporter not configured properly"
    echo "  3. Network connection to :$OTLP_PORT failed"
    echo ""
    echo "Check test configuration in:"
    echo "  crates/clnrm-core/tests/docker_integration.rs"
    exit 1
fi

echo -e "${GREEN}✓ Telemetry received: $SAMPLES samples${NC}"
echo ""

# 3. Check violations (CRITICAL)
VIOLATIONS=$(jq '.statistics.advice_level_counts.violation // 0' $REPORT)
echo "Violations: $VIOLATIONS"

if [ "$VIOLATIONS" -gt 0 ]; then
    echo -e "${RED}❌ VALIDATION FAILED: $VIOLATIONS violations found${NC}"
    echo ""
    echo "Violation details:"
    jq '.statistics' $REPORT
    echo ""
    echo "These violations prove false positives or missing instrumentation."
    echo "Review the full report at: $REPORT"
    exit 1
fi

echo -e "${GREEN}✓ Zero violations${NC}"
echo ""

# 4. Check coverage
COVERAGE=$(jq '.statistics.registry_coverage' $REPORT)
echo "Registry coverage: $COVERAGE"

# Coverage thresholds from clnrm v1.2.0 targets
MINIMUM_COVERAGE=0.70

if (( $(echo "$COVERAGE < $MINIMUM_COVERAGE" | bc -l) )); then
    echo -e "${YELLOW}⚠️  Coverage below target: $COVERAGE < $MINIMUM_COVERAGE${NC}"
    echo ""
    echo "Consider adding more telemetry to increase coverage."
else
    echo -e "${GREEN}✓ Coverage meets target${NC}"
fi

echo ""

# ========== FINAL SUMMARY ==========
echo "================================================================================"
echo -e "${GREEN}✅ WEAVER VALIDATION PASSED${NC}"
echo "================================================================================"
echo ""
echo "Results:"
echo "  ✓ Validated samples: $SAMPLES"
echo "  ✓ Violations: 0"
echo "  ✓ Coverage: $COVERAGE"
echo ""
echo "Full report: $REPORT"
echo ""
echo "Summary:"
jq '.statistics | {
    violations: .advice_level_counts.violation // 0,
    improvements: .advice_level_counts.improvement // 0,
    information: .advice_level_counts.information // 0,
    coverage: .registry_coverage,
    total_entities: .total_entities
}' $REPORT
echo ""
echo "================================================================================"

exit 0
