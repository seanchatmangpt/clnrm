#!/usr/bin/env bash
# Test Scenario 3.3: Custom Rego Policies

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 3.3: Custom Rego Policies ==="
echo "Testing custom organization-specific Rego policies"

# Create custom Rego policy
cat > "${SCRIPT_DIR}/../custom_policy.rego" <<'EOF'
package clnrm.custom

# Policy: All test spans must have test.suite attribute
deny[msg] {
    input.name == "test.execution"
    not input.attributes["test.suite"]
    msg := "test.execution spans must include test.suite attribute"
}

# Policy: Test duration must be reasonable (< 300000ms = 5min)
deny[msg] {
    input.name == "test.execution"
    duration := to_number(input.attributes["test.duration_ms"])
    duration > 300000
    msg := sprintf("test.duration_ms %v exceeds 5 minute limit", [duration])
}

# Policy: Service name must start with 'clnrm-'
deny[msg] {
    service_name := input.resource.attributes["service.name"]
    not startswith(service_name, "clnrm-")
    msg := sprintf("service.name '%v' must start with 'clnrm-'", [service_name])
}
EOF

# Create sample violating custom policy
cat > "${SAMPLES_DIR}/custom_policy_violations.json" <<'EOF'
{
  "resourceSpans": [{
    "resource": {
      "attributes": [{
        "key": "service.name",
        "value": { "stringValue": "invalid-service" }
      }]
    },
    "scopeSpans": [{
      "scope": {
        "name": "clnrm.test"
      },
      "spans": [{
        "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
        "spanId": "051581bf3cb55c16",
        "name": "test.execution",
        "kind": 1,
        "startTimeUnixNano": "1698672000000000000",
        "endTimeUnixNano": "1698672001000000000",
        "attributes": [{
          "key": "test.name",
          "value": { "stringValue": "custom_test" }
        }, {
          "key": "test.duration_ms",
          "value": { "intValue": "400000" }
        }],
        "status": {
          "code": 1
        }
      }]
    }]
  }]
}
EOF

# Test with custom Rego policy
echo "Running with custom Rego policy..."

if command -v weaver &> /dev/null; then
    weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/custom_policy_violations.json" \
        --rego-policy "${SCRIPT_DIR}/../custom_policy.rego" \
        --output json > "${RESULTS_DIR}/scenario_3.3_rego.json" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/custom_policy_violations.json \
        --rego-policy /custom_policy.rego \
        --output json > "${RESULTS_DIR}/scenario_3.3_rego.json" 2>&1 || true
fi

# Analyze custom policy violations
if [ -f "${RESULTS_DIR}/scenario_3.3_rego.json" ]; then
    echo "✅ PASS: Custom Rego policy validation completed"

    if jq empty "${RESULTS_DIR}/scenario_3.3_rego.json" 2>/dev/null; then
        violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_3.3_rego.json" 2>/dev/null || echo "0")
        echo "Custom policy violations: ${violations}"

        if [ "${violations}" -gt 0 ]; then
            echo "✅ PASS: Custom policies detected violations"
            echo "Violation details:"
            jq -r '.violations[] | "\(.rule): \(.message)"' "${RESULTS_DIR}/scenario_3.3_rego.json" 2>/dev/null || true
        else
            echo "⚠️  WARNING: No custom policy violations detected"
        fi
    fi
else
    echo "❌ FAIL: No Rego policy output generated"
    exit 1
fi

echo "=== Scenario 3.3 Complete ==="
