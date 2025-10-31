#!/usr/bin/env bash
# Test Scenario 2.2: JSON Output (Machine-Readable)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 2.2: JSON Output (Machine-Readable) ==="
echo "Testing live-check with JSON output for CI/CD parsing"

# Run with JSON output
echo "Running live-check with JSON output..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/invalid_spans.json" \
        --output json > "${RESULTS_DIR}/scenario_2.2_json.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/invalid_spans.json \
        --output json > "${RESULTS_DIR}/scenario_2.2_json.json" 2>&1 || true
fi

# Verify JSON validity
if [ -f "${RESULTS_DIR}/scenario_2.2_json.json" ]; then
    echo "✅ PASS: JSON output generated"

    # Validate JSON structure
    if jq empty "${RESULTS_DIR}/scenario_2.2_json.json" 2>/dev/null; then
        echo "✅ PASS: Valid JSON format"

        # Extract key metrics
        violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_2.2_json.json" 2>/dev/null || echo "0")
        total_spans=$(jq -r '.total_spans // 0' "${RESULTS_DIR}/scenario_2.2_json.json" 2>/dev/null || echo "0")
        coverage=$(jq -r '.coverage_percent // "N/A"' "${RESULTS_DIR}/scenario_2.2_json.json" 2>/dev/null || echo "N/A")

        echo "Metrics extracted:"
        echo "  - Violations: ${violations}"
        echo "  - Total spans: ${total_spans}"
        echo "  - Coverage: ${coverage}"

        # CI/CD simulation: fail if violations > 0
        if [ "${violations}" -gt 0 ]; then
            echo "✅ PASS: Violations detected (CI/CD would fail)"
        fi

        # Show sample violation
        if [ "${violations}" -gt 0 ]; then
            echo "Sample violation:"
            jq -r '.violations[0]' "${RESULTS_DIR}/scenario_2.2_json.json" 2>/dev/null || true
        fi
    else
        echo "❌ FAIL: Invalid JSON format"
        cat "${RESULTS_DIR}/scenario_2.2_json.json"
        exit 1
    fi
else
    echo "❌ FAIL: No JSON output generated"
    exit 1
fi

echo "=== Scenario 2.2 Complete ==="
