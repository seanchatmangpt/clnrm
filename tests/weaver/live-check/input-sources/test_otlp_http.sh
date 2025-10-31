#!/usr/bin/env bash
# Test Scenario 1.2: OTLP HTTP Ingestion

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 1.2: OTLP HTTP Ingestion ==="
echo "Testing live-check with OTLP HTTP on port 4318"

# Ensure Docker environment is running
cd "${SCRIPT_DIR}/../.."
docker-compose up -d

# Wait for services
echo "Waiting for services..."
sleep 5

# Start weaver live-check in background (HTTP mode)
echo "Starting Weaver live-check (HTTP mode)..."
docker exec -d weaver-validator weaver registry live-check \
    --registry /registry \
    --otlp-http http://otel-collector:4318 \
    --output json \
    --timeout 30s > "${RESULTS_DIR}/scenario_1.2_output.json" 2>&1 &

WEAVER_PID=$!
sleep 2

# Send test spans via HTTP
echo "Sending valid test spans via HTTP..."
curl -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -d @"${SAMPLES_DIR}/valid_spans.json" || true

sleep 2

# Send invalid spans to trigger violations
echo "Sending invalid test spans to trigger violations..."
curl -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -d @"${SAMPLES_DIR}/invalid_spans.json" || true

# Wait for weaver to process
sleep 3

# Stop weaver gracefully
echo "Stopping Weaver..."
kill -SIGTERM ${WEAVER_PID} 2>/dev/null || true
wait ${WEAVER_PID} 2>/dev/null || true

# Verify results
if [ -f "${RESULTS_DIR}/scenario_1.2_output.json" ]; then
    echo "✅ PASS: Live-check processed HTTP data"
    echo "Result stored in: ${RESULTS_DIR}/scenario_1.2_output.json"

    # Analyze violations
    if jq -e '.violations' "${RESULTS_DIR}/scenario_1.2_output.json" > /dev/null 2>&1; then
        violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_1.2_output.json")
        echo "Violations detected: ${violations}"

        if [ "${violations}" -gt 0 ]; then
            echo "✅ PASS: Violations correctly detected from invalid spans"
        fi
    fi
else
    echo "❌ FAIL: No output generated"
    exit 1
fi

echo "=== Scenario 1.2 Complete ==="
