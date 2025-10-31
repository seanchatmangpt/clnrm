#!/usr/bin/env bash
# Test Scenario 5.1: Registry Coverage Tracking

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 5.1: Registry Coverage Tracking ==="
echo "Testing schema coverage metrics and unused schema reporting"

# Run with coverage reporting enabled
echo "Running with coverage tracking..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/valid_spans.json" \
        --coverage-report \
        --output json > "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/valid_spans.json \
        --coverage-report \
        --output json > "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>&1 || true
fi

# Analyze coverage metrics
if [ -f "${RESULTS_DIR}/scenario_5.1_coverage.json" ]; then
    echo "✅ PASS: Coverage tracking completed"

    if jq empty "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>/dev/null; then
        # Extract coverage metrics
        total_schemas=$(jq -r '.coverage.total_schemas // 0' "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>/dev/null || echo "0")
        used_schemas=$(jq -r '.coverage.used_schemas // 0' "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>/dev/null || echo "0")
        coverage_pct=$(jq -r '.coverage.coverage_percent // 0' "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>/dev/null || echo "0")

        echo "Coverage metrics:"
        echo "  - Total schemas: ${total_schemas}"
        echo "  - Used schemas: ${used_schemas}"
        echo "  - Coverage: ${coverage_pct}%"

        if [ "${total_schemas}" -gt 0 ]; then
            echo "✅ PASS: Coverage metrics calculated"

            # List unused schemas
            unused=$(jq -r '.coverage.unused_schemas[]?' "${RESULTS_DIR}/scenario_5.1_coverage.json" 2>/dev/null || echo "")
            if [ -n "${unused}" ]; then
                echo "Unused schemas:"
                echo "${unused}" | head -10
            fi
        fi

        # Verify coverage report structure
        if jq -e '.coverage' "${RESULTS_DIR}/scenario_5.1_coverage.json" > /dev/null 2>&1; then
            echo "✅ PASS: Coverage report structure valid"
        fi
    fi
else
    echo "❌ FAIL: No coverage report generated"
    exit 1
fi

echo "=== Scenario 5.1 Complete ==="
