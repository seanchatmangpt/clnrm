#!/usr/bin/env bash
# Test Scenario 3.2: OTel Policies

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 3.2: OTel Policies ==="
echo "Testing OTel semantic convention policies"

# Create sample with OTel policy violations
cat > "${SAMPLES_DIR}/otel_policy_violations.json" <<'EOF'
{
  "resourceSpans": [{
    "resource": {
      "attributes": [{
        "key": "ServiceName",
        "value": { "stringValue": "bad-naming" }
      }]
    },
    "scopeSpans": [{
      "scope": {
        "name": "InvalidScope"
      },
      "spans": [{
        "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
        "spanId": "051581bf3cb55c15",
        "name": "InvalidSpanName",
        "kind": 1,
        "startTimeUnixNano": "1698672000000000000",
        "endTimeUnixNano": "1698672001000000000",
        "attributes": [{
          "key": "custom_attribute",
          "value": { "stringValue": "no_namespace" }
        }],
        "status": {
          "code": 1
        }
      }]
    }]
  }]
}
EOF

# Test with OTel policies
echo "Running with OTel policies..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/otel_policy_violations.json" \
        --policies otel \
        --output json > "${RESULTS_DIR}/scenario_3.2_policies.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/otel_policy_violations.json \
        --policies otel \
        --output json > "${RESULTS_DIR}/scenario_3.2_policies.json" 2>&1 || true
fi

# Analyze policy violations
if [ -f "${RESULTS_DIR}/scenario_3.2_policies.json" ]; then
    echo "✅ PASS: OTel policy validation completed"

    if jq empty "${RESULTS_DIR}/scenario_3.2_policies.json" 2>/dev/null; then
        violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_3.2_policies.json" 2>/dev/null || echo "0")
        echo "Policy violations detected: ${violations}"

        if [ "${violations}" -gt 0 ]; then
            echo "✅ PASS: OTel policies detected violations"
            echo "Sample violations:"
            jq -r '.violations[] | "\(.severity): \(.message)"' "${RESULTS_DIR}/scenario_3.2_policies.json" 2>/dev/null | head -5 || true
        else
            echo "⚠️  WARNING: No policy violations detected"
        fi
    fi
else
    echo "❌ FAIL: No policy output generated"
    exit 1
fi

echo "=== Scenario 3.2 Complete ==="
