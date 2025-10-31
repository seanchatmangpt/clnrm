#!/bin/bash
# scripts/track_coverage.sh
# Track registry coverage over time
# Usage: ./scripts/track_coverage.sh [--upload] [--baseline PERCENT]

set -euo pipefail

# Configuration
REGISTRY_PATH="registry/"
OTLP_PORT=4317
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR=".weaver/coverage/$TIMESTAMP"
HISTORY_DIR=".weaver/coverage/history"
BASELINE=${BASELINE:-80}
UPLOAD=${1:-""}

mkdir -p "$OUTPUT_DIR" "$HISTORY_DIR"

echo "📊 Coverage Analysis"
echo "   Timestamp: $TIMESTAMP"
echo "   Baseline: $BASELINE%"
echo ""

# Start live-check
echo "🔍 Starting live-check..."
weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --otlp-grpc-port "$OTLP_PORT" \
    --format json \
    --output "$OUTPUT_DIR" \
    --inactivity-timeout 60 > "$OUTPUT_DIR/weaver.log" 2>&1 &
PID=$!

# Wait for server
for i in {1..10}; do
    if lsof -i :"$OTLP_PORT" >/dev/null 2>&1; then
        echo "✅ Server started"
        break
    fi
    sleep 1
done

# Run comprehensive test suite
echo "🧪 Running all tests..."
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:"$OTLP_PORT" \
    cargo test --all --features otel -- --test-threads=4

# Wait for processing
sleep 3

# Stop live-check
echo "🛑 Stopping live-check..."
kill -HUP $PID
wait $PID

# Extract metrics
if [ ! -f "$OUTPUT_DIR/live_check.json" ]; then
    echo "❌ No validation output generated"
    cat "$OUTPUT_DIR/weaver.log"
    exit 1
fi

COVERAGE=$(jq -r '.statistics.registry_coverage // 0' "$OUTPUT_DIR/live_check.json")
COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc -l | cut -d. -f1)
VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' "$OUTPUT_DIR/live_check.json")
WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' "$OUTPUT_DIR/live_check.json")

# Create history record
cat > "$HISTORY_DIR/$TIMESTAMP.json" <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "coverage_percent": $COVERAGE_PCT,
  "violations": $VIOLATIONS,
  "warnings": $WARNINGS,
  "git_commit": "$(git rev-parse HEAD)",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD)"
}
EOF

# Show summary
echo ""
echo "📊 Coverage Report:"
echo "   Coverage: $COVERAGE_PCT%"
echo "   Violations: $VIOLATIONS"
echo "   Warnings: $WARNINGS"
echo "   Git Commit: $(git rev-parse --short HEAD)"
echo "   Git Branch: $(git rev-parse --abbrev-ref HEAD)"
echo ""

# Check baseline
if [ "$COVERAGE_PCT" -lt "$BASELINE" ]; then
    echo "❌ Coverage below baseline: $COVERAGE_PCT% < $BASELINE%"
    exit 1
fi

# Generate trend chart (requires Python)
if command -v python3 >/dev/null 2>&1; then
    python3 - <<PYTHON
import json
import glob
from pathlib import Path

history_files = sorted(glob.glob("$HISTORY_DIR/*.json"))
if len(history_files) > 1:
    data = [json.load(open(f)) for f in history_files[-10:]]

    print("📈 Coverage Trend (last 10 runs):")
    for record in data:
        timestamp = record['timestamp'][:10]
        coverage = record['coverage_percent']
        bar = '█' * (coverage // 2)
        print(f"  {timestamp}: {bar:<50} {coverage}%")
PYTHON
fi

# Upload to metrics service (optional)
if [ "$UPLOAD" = "--upload" ] && [ -n "${METRICS_API_ENDPOINT:-}" ]; then
    echo ""
    echo "📤 Uploading metrics to $METRICS_API_ENDPOINT"
    curl -X POST "${METRICS_API_ENDPOINT}" \
        -H "Content-Type: application/json" \
        -d @"$HISTORY_DIR/$TIMESTAMP.json"
fi

echo ""
echo "✅ Coverage tracking complete"
echo "   History: $HISTORY_DIR/$TIMESTAMP.json"

exit 0
