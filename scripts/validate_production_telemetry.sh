#!/bin/bash
# scripts/validate_production_telemetry.sh
# Validate production telemetry samples against registry schemas
# Usage: ./scripts/validate_production_telemetry.sh [--source URL|FILE]

set -euo pipefail

# Configuration
SOURCE=${1:-"${PRODUCTION_OTLP_ENDPOINT:-}"}
REGISTRY_PATH="registry/"
OUTPUT_DIR=".weaver/production/$(date +%Y%m%d_%H%M%S)"
ALERT_WEBHOOK="${SLACK_WEBHOOK_URL:-}"

mkdir -p "$OUTPUT_DIR"

echo "🔍 Production Telemetry Validation"
echo "   Source: ${SOURCE:-stdin}"
echo "   Registry: $REGISTRY_PATH"
echo ""

# Collect telemetry sample
if [ -z "$SOURCE" ]; then
    echo "❌ No telemetry source specified"
    echo "   Set PRODUCTION_OTLP_ENDPOINT or pass --source argument"
    exit 1
fi

if [[ "$SOURCE" =~ ^http ]]; then
    echo "📥 Collecting sample from HTTP endpoint..."
    if ! curl -s -f "$SOURCE/telemetry/sample" > "$OUTPUT_DIR/sample.json"; then
        echo "❌ Failed to fetch telemetry from $SOURCE"
        exit 1
    fi
else
    echo "📥 Reading sample from file..."
    if [ ! -f "$SOURCE" ]; then
        echo "❌ Source file not found: $SOURCE"
        exit 1
    fi
    cp "$SOURCE" "$OUTPUT_DIR/sample.json"
fi

# Validate with Weaver
echo "⚙️  Running Weaver validation..."
if ! weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --input-source "$OUTPUT_DIR/sample.json" \
    --format json \
    --output "$OUTPUT_DIR" 2>&1 | tee "$OUTPUT_DIR/weaver.log"; then
    echo "⚠️  Weaver validation had issues (check log)"
fi

# Check if output was generated
if [ ! -f "$OUTPUT_DIR/live_check.json" ]; then
    echo "❌ No validation output generated"
    cat "$OUTPUT_DIR/weaver.log"
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
echo "   Timestamp: $(date -Iseconds)"
echo ""

# Create alert payload
if [ "$VIOLATIONS" -gt 0 ]; then
    SEVERITY="error"
    COLOR="#FF0000"
    TITLE="🚨 Production Telemetry Violations Detected"
    EMOJI="🚨"
else
    SEVERITY="info"
    COLOR="#00FF00"
    TITLE="✅ Production Telemetry Validation Passed"
    EMOJI="✅"
fi

# Send alert to Slack (if configured)
if [ -n "$ALERT_WEBHOOK" ]; then
    echo "📤 Sending alert to Slack..."
    curl -X POST "$ALERT_WEBHOOK" \
        -H "Content-Type: application/json" \
        -d @- <<EOF
{
  "text": "$TITLE",
  "attachments": [
    {
      "color": "$COLOR",
      "title": "Production Telemetry Validation",
      "fields": [
        {
          "title": "Violations",
          "value": "$VIOLATIONS",
          "short": true
        },
        {
          "title": "Warnings",
          "value": "$WARNINGS",
          "short": true
        },
        {
          "title": "Coverage",
          "value": "$COVERAGE_PCT%",
          "short": true
        },
        {
          "title": "Timestamp",
          "value": "$(date -Iseconds)",
          "short": true
        }
      ]
    }
  ]
}
EOF
    echo "✅ Alert sent"
fi

# Log to metrics (append-only log)
echo "{
  \"timestamp\": \"$(date -Iseconds)\",
  \"violations\": $VIOLATIONS,
  \"warnings\": $WARNINGS,
  \"coverage_percent\": $COVERAGE_PCT,
  \"environment\": \"production\"
}" >> .weaver/production/metrics.jsonl

echo "📝 Logged to .weaver/production/metrics.jsonl"

# Exit with error if violations found
if [ "$VIOLATIONS" -gt 0 ]; then
    echo ""
    echo "❌ Production validation failed: $VIOLATIONS violations"
    echo ""
    echo "Violations:"
    jq -r '.violations[]? | "  - \(.level): \(.message)"' "$OUTPUT_DIR/live_check.json"
    echo ""
    echo "Full report: $OUTPUT_DIR/live_check.json"
    exit 1
fi

echo ""
echo "✅ Production validation passed"
exit 0
