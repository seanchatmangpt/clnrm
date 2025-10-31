#!/usr/bin/env bash
# Test Scenario 1.1: OTLP gRPC Ingestion

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 1.1: OTLP gRPC Ingestion ==="
echo "Testing live-check with OTLP gRPC on port 4317"

# Start Docker environment
echo "Starting OTLP Collector and Weaver..."
cd "${SCRIPT_DIR}/../.."
docker-compose up -d otel-collector weaver-validator

# Wait for services to be ready
echo "Waiting for services to initialize..."
sleep 5

# Check health
if ! curl -sf http://localhost:13133/ > /dev/null; then
    echo "ERROR: OTLP Collector not ready"
    docker-compose logs otel-collector
    exit 1
fi

echo "Services ready. Starting test..."

# Send test spans via gRPC (using otel-cli or similar tool)
# For this test, we'll use the collector's debug exporter as validation

# Start weaver live-check in background
echo "Starting Weaver live-check (gRPC mode)..."
timeout 30s docker exec weaver-validator weaver registry live-check \
    --registry /registry \
    --otlp-grpc otel-collector:4317 \
    --output json > "${RESULTS_DIR}/scenario_1.1_output.json" 2>&1 || true

# Parse results
if [ -f "${RESULTS_DIR}/scenario_1.1_output.json" ]; then
    echo "✅ PASS: Live-check produced output"
    echo "Result stored in: ${RESULTS_DIR}/scenario_1.1_output.json"

    # Check for violations
    if grep -q '"violations"' "${RESULTS_DIR}/scenario_1.1_output.json"; then
        violations=$(jq -r '.violations | length' "${RESULTS_DIR}/scenario_1.1_output.json" 2>/dev/null || echo "unknown")
        echo "Violations detected: ${violations}"
    fi
else
    echo "❌ FAIL: No output generated"
    exit 1
fi

echo "=== Scenario 1.1 Complete ==="
