#!/bin/bash
# scripts/quick_validate.sh
# Fast validation for current changes
# Usage: ./scripts/quick_validate.sh "cargo test test_name"

set -euo pipefail

COMMAND=${1:-"cargo test --features otel"}
OUTPUT_DIR=".weaver/quick_$(date +%s)"
OTLP_PORT=4317

mkdir -p "$OUTPUT_DIR"

echo "⚡ Quick Validation"
echo "   Command: $COMMAND"
echo ""

# Start live-check in background
weaver registry live-check \
    --registry registry/ \
    --otlp-grpc-port "$OTLP_PORT" \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout 10 > "$OUTPUT_DIR/weaver.log" 2>&1 &
PID=$!

# Wait for server to start
for i in {1..10}; do
    if lsof -i :"$OTLP_PORT" >/dev/null 2>&1; then
        echo "✅ Live-check started"
        break
    fi
    sleep 1
done

# Run command with telemetry
echo "🧪 Running: $COMMAND"
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
    bash -c "$COMMAND"

# Wait for telemetry to be processed
sleep 2

# Stop live-check gracefully
echo "🛑 Stopping live-check..."
kill -HUP $PID 2>/dev/null
wait $PID 2>/dev/null || true

# Check results
if [ ! -f "$OUTPUT_DIR/live_check.json" ]; then
    echo "❌ No validation output generated"
    cat "$OUTPUT_DIR/weaver.log"
    exit 1
fi

# Show summary
echo ""
echo "📊 Validation Summary:"
jq '{
    violations: .statistics.advice_level_counts.violation // 0,
    warnings: .statistics.advice_level_counts.warning // 0,
    coverage: (.statistics.registry_coverage * 100 | round) + "%"
}' "$OUTPUT_DIR/live_check.json"

VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' "$OUTPUT_DIR/live_check.json")

if [ "$VIOLATIONS" -gt 0 ]; then
    echo ""
    echo "❌ Violations found:"
    jq -r '.violations[]? | "  - \(.level): \(.message)"' "$OUTPUT_DIR/live_check.json"
    exit 1
fi

echo "✅ Validation passed"
exit 0
