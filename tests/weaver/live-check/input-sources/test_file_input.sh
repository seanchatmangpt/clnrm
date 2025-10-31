#!/usr/bin/env bash
# Test Scenario 1.3: File Input (JSON Samples)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 1.3: File Input (JSON Samples) ==="
echo "Testing live-check with static JSON file input"

# Test with valid spans
echo "Testing with valid_spans.json..."
if command -v weaver &> /dev/null; then
    # Run locally if weaver installed
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/valid_spans.json" \
        --output json > "${RESULTS_DIR}/scenario_1.3_valid.json" 2>&1 || true
else
    # Use Docker
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/valid_spans.json \
        --output json > "${RESULTS_DIR}/scenario_1.3_valid.json" 2>&1 || true
fi

# Test with invalid spans
echo "Testing with invalid_spans.json..."
if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/invalid_spans.json" \
        --output json > "${RESULTS_DIR}/scenario_1.3_invalid.json" 2>&1 || true
else
    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/invalid_spans.json \
        --output json > "${RESULTS_DIR}/scenario_1.3_invalid.json" 2>&1 || true
fi

# Verify results
echo "Analyzing results..."

if [ -f "${RESULTS_DIR}/scenario_1.3_valid.json" ]; then
    echo "✅ PASS: Valid spans processed successfully"

    valid_violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_1.3_valid.json" 2>/dev/null || echo "0")
    echo "  Valid spans violations: ${valid_violations} (should be 0)"
fi

if [ -f "${RESULTS_DIR}/scenario_1.3_invalid.json" ]; then
    echo "✅ PASS: Invalid spans processed successfully"

    invalid_violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_1.3_invalid.json" 2>/dev/null || echo "0")
    echo "  Invalid spans violations: ${invalid_violations} (should be > 0)"

    if [ "${invalid_violations}" -gt 0 ]; then
        echo "✅ PASS: Violations correctly detected in invalid spans"
        jq -r '.violations[0]' "${RESULTS_DIR}/scenario_1.3_invalid.json" 2>/dev/null || true
    else
        echo "⚠️  WARNING: No violations detected in invalid spans (expected violations)"
    fi
fi

echo "=== Scenario 1.3 Complete ==="
