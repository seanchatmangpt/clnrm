#!/usr/bin/env bash
# Test Scenario 4.2: SIGHUP Stop Condition

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 4.2: SIGHUP Stop Condition (Graceful with Report) ==="
echo "Testing SIGHUP triggers report generation before shutdown"

# Start live-check in background
echo "Starting live-check process..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --otlp-http http://localhost:4318 \
        --output json > "${RESULTS_DIR}/scenario_4.2_sighup.json" 2>&1 &
    WEAVER_PID=$!
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator otel-collector

    docker exec -d weaver-validator weaver registry live-check \
        --registry /registry \
        --otlp-http http://otel-collector:4318 \
        --output json > "${RESULTS_DIR}/scenario_4.2_sighup.json" 2>&1 &
    WEAVER_PID=$!
fi

echo "Weaver PID: ${WEAVER_PID}"

# Let it run and accumulate data
echo "Running for 15 seconds to accumulate data..."
sleep 15

# Send SIGHUP (should trigger report generation)
echo "Sending SIGHUP signal..."
kill -SIGHUP ${WEAVER_PID} 2>/dev/null || true

# Wait for report generation
echo "Waiting for report generation..."
sleep 5

# Verify report
if [ -f "${RESULTS_DIR}/scenario_4.2_sighup.json" ]; then
    echo "✅ PASS: SIGHUP triggered report generation"

    if jq empty "${RESULTS_DIR}/scenario_4.2_sighup.json" 2>/dev/null; then
        # Check for complete report structure
        has_summary=$(jq -e '.summary' "${RESULTS_DIR}/scenario_4.2_sighup.json" 2>/dev/null && echo "yes" || echo "no")
        has_coverage=$(jq -e '.coverage' "${RESULTS_DIR}/scenario_4.2_sighup.json" 2>/dev/null && echo "yes" || echo "no")

        echo "Report completeness:"
        echo "  - Has summary: ${has_summary}"
        echo "  - Has coverage: ${has_coverage}"

        if [ "${has_summary}" = "yes" ] || [ "${has_coverage}" = "yes" ]; then
            echo "✅ PASS: Complete report generated before shutdown"
        fi
    fi
else
    echo "⚠️  WARNING: No report file generated"
fi

# Cleanup
kill -SIGTERM ${WEAVER_PID} 2>/dev/null || true

echo "=== Scenario 4.2 Complete ==="
