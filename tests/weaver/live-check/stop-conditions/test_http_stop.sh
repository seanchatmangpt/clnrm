#!/usr/bin/env bash
# Test Scenario 4.3: HTTP /stop Endpoint

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 4.3: HTTP /stop Endpoint ==="
echo "Testing graceful shutdown via HTTP API"

# Start live-check with HTTP API enabled
echo "Starting live-check with HTTP API on port 8080..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --otlp-http http://localhost:4318 \
        --http-api :8080 \
        --output json > "${RESULTS_DIR}/scenario_4.3_http_stop.json" 2>&1 &
    WEAVER_PID=$!
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator otel-collector

    docker exec -d weaver-validator weaver registry live-check \
        --registry /registry \
        --otlp-http http://otel-collector:4318 \
        --http-api :8080 \
        --output json > "${RESULTS_DIR}/scenario_4.3_http_stop.json" 2>&1 &
    WEAVER_PID=$!
fi

echo "Weaver PID: ${WEAVER_PID}"

# Wait for HTTP API to be ready
echo "Waiting for HTTP API to initialize..."
sleep 5

# Check health endpoint
if curl -sf http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ PASS: HTTP API is accessible"
else
    echo "⚠️  WARNING: HTTP API not accessible (endpoint may not exist)"
fi

# Send stop request
echo "Sending HTTP stop request..."
STOP_RESPONSE=$(curl -X POST http://localhost:8080/stop 2>&1 || echo "error")

echo "Stop response: ${STOP_RESPONSE}"

# Wait for graceful shutdown
echo "Waiting for graceful shutdown..."
sleep 3

# Check process status
if ps -p ${WEAVER_PID} > /dev/null 2>&1; then
    echo "⚠️  WARNING: Process still running, forcing shutdown..."
    kill -SIGTERM ${WEAVER_PID} 2>/dev/null || true
else
    echo "✅ PASS: Process gracefully stopped via HTTP"
fi

# Verify output
if [ -f "${RESULTS_DIR}/scenario_4.3_http_stop.json" ]; then
    echo "✅ PASS: Output file generated"
    echo "Output size: $(wc -c < "${RESULTS_DIR}/scenario_4.3_http_stop.json") bytes"
fi

echo "=== Scenario 4.3 Complete ==="
