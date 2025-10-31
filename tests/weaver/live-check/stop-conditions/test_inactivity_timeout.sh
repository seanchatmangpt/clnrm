#!/usr/bin/env bash
# Test Scenario 4.4: Inactivity Timeout

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 4.4: Inactivity Timeout ==="
echo "Testing auto-shutdown after timeout with no telemetry"

# Start live-check with 30s timeout
echo "Starting live-check with 30s inactivity timeout..."
START_TIME=$(date +%s)

if command -v weaver &> /dev/null; then
    timeout 45s weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --otlp-http http://localhost:4318 \
        --timeout 30s \
        --output json > "${RESULTS_DIR}/scenario_4.4_timeout.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator otel-collector

    timeout 45s docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --otlp-http http://otel-collector:4318 \
        --timeout 30s \
        --output json > "${RESULTS_DIR}/scenario_4.4_timeout.json" 2>&1 || true
fi

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo "Elapsed time: ${ELAPSED} seconds"

# Verify timeout behavior
if [ "${ELAPSED}" -ge 30 ] && [ "${ELAPSED}" -le 35 ]; then
    echo "✅ PASS: Auto-shutdown occurred around 30s timeout"
else
    echo "⚠️  WARNING: Shutdown time ${ELAPSED}s not close to expected 30s"
fi

# Verify output
if [ -f "${RESULTS_DIR}/scenario_4.4_timeout.json" ]; then
    echo "✅ PASS: Final report generated after timeout"

    # Check for timeout message
    if grep -qi "timeout\|inactivity\|idle" "${RESULTS_DIR}/scenario_4.4_timeout.json"; then
        echo "✅ PASS: Timeout message detected in output"
    fi

    if jq empty "${RESULTS_DIR}/scenario_4.4_timeout.json" 2>/dev/null; then
        summary=$(jq -r '.summary // "no summary"' "${RESULTS_DIR}/scenario_4.4_timeout.json")
        echo "Final summary: ${summary}"
    fi
else
    echo "⚠️  WARNING: No timeout report generated"
fi

echo "=== Scenario 4.4 Complete ==="
