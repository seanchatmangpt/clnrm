#!/bin/bash
# scripts/dev_live_check.sh
# Interactive development validation with Weaver live-check
# Usage: ./scripts/dev_live_check.sh [--debug]

set -euo pipefail

DEBUG=${1:-""}
REGISTRY_PATH="registry/"
OTLP_PORT=4317
OUTPUT_DIR=".weaver/dev_$(date +%s)"

mkdir -p "$OUTPUT_DIR"

# Start live-check with streaming output
echo "🔍 Starting Weaver live-check on port $OTLP_PORT"
echo "📁 Output: $OUTPUT_DIR"
echo ""
echo "To send telemetry:"
echo "  export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:$OTLP_PORT"
echo "  cargo test --features otel"
echo ""
echo "Press Ctrl+C to stop and generate report"
echo ""

if [ "$DEBUG" = "--debug" ]; then
    # Debug mode: verbose output
    weaver registry live-check \
        --registry "$REGISTRY_PATH" \
        --otlp-grpc-port "$OTLP_PORT" \
        --format json \
        --output "$OUTPUT_DIR" \
        --verbose
else
    # Normal mode: structured output with color coding
    weaver registry live-check \
        --registry "$REGISTRY_PATH" \
        --otlp-grpc-port "$OTLP_PORT" \
        --format json \
        --output "$OUTPUT_DIR" \
        2>&1 | while IFS= read -r line; do
            # Parse and colorize output
            if echo "$line" | grep -qi "violation"; then
                echo "❌ $line"
            elif echo "$line" | grep -qi "warning"; then
                echo "⚠️  $line"
            elif echo "$line" | grep -qi "info"; then
                echo "ℹ️  $line"
            else
                echo "$line"
            fi
        done
fi

# Show final summary if report was generated
if [ -f "$OUTPUT_DIR/live_check.json" ]; then
    echo ""
    echo "📊 Final Summary:"
    jq '{
        violations: .statistics.advice_level_counts.violation // 0,
        warnings: .statistics.advice_level_counts.warning // 0,
        coverage: (.statistics.registry_coverage * 100 | round) + "%"
    }' "$OUTPUT_DIR/live_check.json"
fi
