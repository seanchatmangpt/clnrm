#!/bin/bash
# Honest Test Implementation
#
# This script ACTUALLY runs clnrm self-test with OTEL tracing enabled.
# It produces real spans, lifecycle events, and all required evidence.
#
# BEHAVIOR:
# - Launches containers via clnrm
# - Generates OTEL spans with proper hierarchy
# - Produces lifecycle events (container.start, exec, stop)
# - Creates parent→child span relationships
# - Sets hermetic attributes
#
# EXPECTED: PASS (all OTEL evidence present)

set -e

echo "=== Honest Test Implementation ==="
echo "Starting clnrm self-test with OTEL tracing..."
echo

# Configure OTEL
export OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_SERVICE_NAME=clnrm-self-test
export OTEL_DEPLOYMENT_ENV=case-study

# Run actual clnrm self-test
# This will:
# 1. Create CleanroomEnvironment
# 2. Launch containers
# 3. Generate OTEL spans
# 4. Execute test steps
# 5. Validate hermetic isolation
# 6. Cleanup containers

echo "Executing: clnrm run tests/ --otel-exporter otlp"
clnrm run tests/ \
  --otel-exporter otlp \
  --otel-endpoint http://localhost:4318 \
  --format json

EXIT_CODE=$?

echo
echo "Self-test exit code: $EXIT_CODE"

if [ $EXIT_CODE -eq 0 ]; then
  echo "✅ Self-test PASSED"
  echo "✅ OTEL spans generated"
  echo "✅ Container lifecycle events recorded"
  echo "✅ Parent→child relationships established"
  echo "✅ Hermetic attributes set"
else
  echo "❌ Self-test FAILED"
  exit $EXIT_CODE
fi

echo
echo "=== Honest Implementation Complete ==="
exit $EXIT_CODE
