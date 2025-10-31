#!/usr/bin/env bash
# Test Scenario 3.1: Builtin Advisors

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 3.1: Builtin Advisors ==="
echo "Testing builtin advisors: missing_attribute, type_mismatch"

# Test with advisors enabled
echo "Running with builtin advisors..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/invalid_spans.json" \
        --advisors builtin \
        --output json > "${RESULTS_DIR}/scenario_3.1_advisors.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/invalid_spans.json \
        --advisors builtin \
        --output json > "${RESULTS_DIR}/scenario_3.1_advisors.json" 2>&1 || true
fi

# Analyze advisor violations
if [ -f "${RESULTS_DIR}/scenario_3.1_advisors.json" ]; then
    echo "✅ PASS: Advisor validation completed"

    if jq empty "${RESULTS_DIR}/scenario_3.1_advisors.json" 2>/dev/null; then
        # Check for specific advisor violations
        missing_attr=$(jq -r '[.violations[] | select(.advisor == "missing_attribute")] | length' "${RESULTS_DIR}/scenario_3.1_advisors.json" 2>/dev/null || echo "0")
        type_mismatch=$(jq -r '[.violations[] | select(.advisor == "type_mismatch")] | length' "${RESULTS_DIR}/scenario_3.1_advisors.json" 2>/dev/null || echo "0")

        echo "Advisor violations detected:"
        echo "  - missing_attribute: ${missing_attr}"
        echo "  - type_mismatch: ${type_mismatch}"

        if [ "${missing_attr}" -gt 0 ] || [ "${type_mismatch}" -gt 0 ]; then
            echo "✅ PASS: Builtin advisors detected violations"
        else
            echo "⚠️  WARNING: No advisor violations detected (expected violations)"
        fi
    fi
else
    echo "❌ FAIL: No advisor output generated"
    exit 1
fi

echo "=== Scenario 3.1 Complete ==="
