#!/bin/bash
# scripts/pre-commit.sh
# Pre-commit hook for telemetry validation
# Install: cp scripts/pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

set -euo pipefail

# Configuration
REGISTRY_PATH="registry/"
OTLP_PORT=4317
OUTPUT_DIR="/tmp/weaver_precommit_$$"
INACTIVITY_TIMEOUT=30

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 Running Weaver telemetry validation...${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Start live-check
weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --otlp-grpc-port "$OTLP_PORT" \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout "$INACTIVITY_TIMEOUT" \
    2>&1 | tee "$OUTPUT_DIR/weaver.log" &
PID=$!

# Wait for server to start
echo "⏳ Starting validation server..."
for i in {1..10}; do
    if lsof -i :"$OTLP_PORT" >/dev/null 2>&1; then
        echo "✅ Server started"
        break
    fi
    sleep 1
done

# Get list of changed test files
CHANGED_TESTS=$(git diff --cached --name-only --diff-filter=ACMR | grep -E '(tests?/.*\.rs$|src/.*\.rs$)' || true)

if [ -z "$CHANGED_TESTS" ]; then
    echo "ℹ️  No test files changed, running smoke test"
    OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
        cargo test --features otel --lib -- --test-threads=1 >/dev/null 2>&1 || true
else
    echo "🧪 Running tests for changed files:"
    echo "$CHANGED_TESTS" | sed 's/^/  - /'
    echo ""

    # Run affected tests
    for test_file in $CHANGED_TESTS; do
        # Extract module path from file path
        MODULE=$(echo "$test_file" | sed 's/src\///' | sed 's/\.rs$//' | sed 's/\//::/')

        OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
            cargo test --features otel "$MODULE" -- --test-threads=1 >/dev/null 2>&1 || true
    done
fi

# Wait for telemetry processing
sleep 2

# Stop live-check gracefully
echo "🛑 Stopping validation..."
kill -HUP $PID 2>/dev/null
wait $PID 2>/dev/null || true

# Check output
if [ ! -f "$OUTPUT_DIR/live_check.json" ]; then
    echo -e "${RED}❌ No validation output generated${NC}"
    echo "   Log: $OUTPUT_DIR/weaver.log"
    exit 1
fi

# Parse results
VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' "$OUTPUT_DIR/live_check.json")
WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE=$(jq -r '.statistics.registry_coverage // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc -l | cut -d. -f1)

# Show summary
echo ""
echo "📊 Validation Summary:"
echo "   Violations: $VIOLATIONS"
echo "   Warnings: $WARNINGS"
echo "   Coverage: $COVERAGE_PCT%"

# Check for violations
if [ "$VIOLATIONS" -gt 0 ]; then
    echo -e "${RED}❌ Telemetry validation failed: $VIOLATIONS violations${NC}"
    echo ""
    echo "Violations:"
    jq -r '.violations[]? | "  - \(.level): \(.message)"' "$OUTPUT_DIR/live_check.json"
    echo ""
    echo "Full report: $OUTPUT_DIR/live_check.json"
    echo ""
    echo "To bypass (not recommended): git commit --no-verify"
    exit 1
fi

# Warn on low coverage
if [ "$COVERAGE_PCT" -lt 50 ]; then
    echo -e "${YELLOW}⚠️  Low registry coverage: $COVERAGE_PCT% (target: 80%)${NC}"
fi

echo -e "${GREEN}✅ Telemetry validation passed${NC}"

# Cleanup old outputs
find /tmp -name "weaver_precommit_*" -type d -mtime +1 -exec rm -rf {} + 2>/dev/null || true

exit 0
