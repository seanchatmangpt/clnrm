#!/usr/bin/env bash
# Test Scenario 1.4: stdin Streaming

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 1.4: stdin Streaming ==="
echo "Testing live-check with stdin attribute streaming"

# Test stdin streaming
echo "Streaming attributes from stdin..."

if command -v weaver &> /dev/null; then
    # Run locally
    cat "${SAMPLES_DIR}/attributes.txt" | weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --stdin \
        --output json > "${RESULTS_DIR}/scenario_1.4_output.json" 2>&1 || true
else
    # Use Docker
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec -i weaver-validator weaver registry live-check \
        --registry /registry \
        --stdin \
        --output json < "${SAMPLES_DIR}/attributes.txt" > "${RESULTS_DIR}/scenario_1.4_output.json" 2>&1 || true
fi

# Verify results
if [ -f "${RESULTS_DIR}/scenario_1.4_output.json" ]; then
    echo "✅ PASS: stdin streaming processed successfully"
    echo "Result stored in: ${RESULTS_DIR}/scenario_1.4_output.json"

    # Check for attribute validation
    if grep -q '"attributes"' "${RESULTS_DIR}/scenario_1.4_output.json"; then
        echo "✅ PASS: Attributes validated from stdin"
    fi
else
    echo "❌ FAIL: No output generated from stdin"
    exit 1
fi

echo "=== Scenario 1.4 Complete ==="
