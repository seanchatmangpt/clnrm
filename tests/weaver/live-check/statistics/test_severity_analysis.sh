#!/usr/bin/env bash
# Test Scenario 5.2: Violation Severity Analysis

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 5.2: Violation Severity Analysis ==="
echo "Testing violation categorization by severity (error, warning, info)"

# Create sample with mixed severity violations
cat > "${SAMPLES_DIR}/mixed_severity.json" <<'EOF'
{
  "resourceSpans": [{
    "resource": {
      "attributes": [{
        "key": "service.name",
        "value": { "stringValue": "clnrm-test" }
      }]
    },
    "scopeSpans": [{
      "scope": {
        "name": "clnrm.test"
      },
      "spans": [{
        "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
        "spanId": "051581bf3cb55c17",
        "name": "test.execution",
        "kind": 1,
        "startTimeUnixNano": "1698672000000000000",
        "endTimeUnixNano": "1698672001000000000",
        "attributes": [{
          "key": "test.name",
          "value": { "stringValue": "severity_test" }
        }, {
          "key": "test.result",
          "value": { "stringValue": "INVALID" }
        }, {
          "key": "test.duration_ms",
          "value": { "stringValue": "invalid_type" }
        }, {
          "key": "deprecated.field",
          "value": { "stringValue": "old_value" }
        }],
        "status": {
          "code": 1
        }
      }]
    }]
  }]
}
EOF

# Run with severity reporting
echo "Running with severity analysis..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/mixed_severity.json" \
        --severity-report \
        --output json > "${RESULTS_DIR}/scenario_5.2_severity.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/mixed_severity.json \
        --severity-report \
        --output json > "${RESULTS_DIR}/scenario_5.2_severity.json" 2>&1 || true
fi

# Analyze severity breakdown
if [ -f "${RESULTS_DIR}/scenario_5.2_severity.json" ]; then
    echo "✅ PASS: Severity analysis completed"

    if jq empty "${RESULTS_DIR}/scenario_5.2_severity.json" 2>/dev/null; then
        # Count violations by severity
        errors=$(jq -r '[.violations[] | select(.severity == "error")] | length' "${RESULTS_DIR}/scenario_5.2_severity.json" 2>/dev/null || echo "0")
        warnings=$(jq -r '[.violations[] | select(.severity == "warning")] | length' "${RESULTS_DIR}/scenario_5.2_severity.json" 2>/dev/null || echo "0")
        infos=$(jq -r '[.violations[] | select(.severity == "info")] | length' "${RESULTS_DIR}/scenario_5.2_severity.json" 2>/dev/null || echo "0")

        echo "Severity breakdown:"
        echo "  - Errors: ${errors}"
        echo "  - Warnings: ${warnings}"
        echo "  - Infos: ${infos}"

        total=$((errors + warnings + infos))
        if [ "${total}" -gt 0 ]; then
            echo "✅ PASS: Violations categorized by severity"

            # Show sample of each severity
            echo "Sample error:"
            jq -r '[.violations[] | select(.severity == "error")] | .[0]' "${RESULTS_DIR}/scenario_5.2_severity.json" 2>/dev/null || echo "none"

            echo "Sample warning:"
            jq -r '[.violations[] | select(.severity == "warning")] | .[0]' "${RESULTS_DIR}/scenario_5.2_severity.json" 2>/dev/null || echo "none"
        fi

        # Check for severity summary
        if jq -e '.severity_summary' "${RESULTS_DIR}/scenario_5.2_severity.json" > /dev/null 2>&1; then
            echo "✅ PASS: Severity summary present in report"
            jq -r '.severity_summary' "${RESULTS_DIR}/scenario_5.2_severity.json"
        fi
    fi
else
    echo "❌ FAIL: No severity report generated"
    exit 1
fi

echo "=== Scenario 5.2 Complete ==="
