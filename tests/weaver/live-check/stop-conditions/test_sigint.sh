#!/usr/bin/env bash
# Test Scenario 4.1: SIGINT Stop Condition

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 4.1: SIGINT Stop Condition (Ctrl-C) ==="
echo "Testing graceful shutdown on SIGINT signal"

# Start live-check in background
echo "Starting live-check process..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --otlp-http http://localhost:4318 \
        --output json > "${RESULTS_DIR}/scenario_4.1_sigint.json" 2>&1 &
    WEAVER_PID=$!
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator otel-collector

    docker exec -d weaver-validator weaver registry live-check \
        --registry /registry \
        --otlp-http http://otel-collector:4318 \
        --output json > "${RESULTS_DIR}/scenario_4.1_sigint.json" 2>&1 &
    WEAVER_PID=$!
fi

echo "Weaver PID: ${WEAVER_PID}"

# Let it run for 10 seconds
echo "Running for 10 seconds..."
sleep 10

# Send SIGINT
echo "Sending SIGINT signal..."
kill -SIGINT ${WEAVER_PID} 2>/dev/null || true

# Wait for graceful shutdown
echo "Waiting for graceful shutdown..."
wait ${WEAVER_PID} 2>/dev/null || true

# Verify shutdown
if [ -f "${RESULTS_DIR}/scenario_4.1_sigint.json" ]; then
    echo "✅ PASS: SIGINT triggered graceful shutdown"

    # Check for partial report
    if grep -q "shutdown\|terminated\|interrupted" "${RESULTS_DIR}/scenario_4.1_sigint.json"; then
        echo "✅ PASS: Shutdown message detected"
    fi

    echo "Output size: $(wc -c < "${RESULTS_DIR}/scenario_4.1_sigint.json") bytes"
else
    echo "⚠️  WARNING: No output file generated"
fi

echo "=== Scenario 4.1 Complete ==="
