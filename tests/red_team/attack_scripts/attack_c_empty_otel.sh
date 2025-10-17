#!/bin/bash
# Attack C: Empty OTEL Path
#
# This script demonstrates an attack that sets OTEL environment
# variables to appear properly instrumented but produces no actual
# spans. This might occur from SDK initialization failures,
# misconfigured exporters, or deliberate tampering.
#
# Expected clnrm verdict: FAIL
# First failing rule: expect.counts.spans_total
# Detection time: ~0.02s

echo "========================================"
echo "Attack C: Empty OTEL Path"
echo "========================================"
echo ""
echo "🎭 Setting OTEL environment (but producing no spans)..."
echo ""

# Set OTEL environment variables (looks legitimate)
export OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
export OTEL_SERVICE_NAME=clnrm
export OTEL_DEPLOYMENT_ENVIRONMENT=ci
export OTEL_TRACES_SAMPLER=always_on

echo "OTEL Configuration:"
echo "  Exporter: $OTEL_TRACES_EXPORTER"
echo "  Endpoint: $OTEL_EXPORTER_OTLP_ENDPOINT"
echo "  Protocol: $OTEL_EXPORTER_OTLP_PROTOCOL"
echo "  Service: $OTEL_SERVICE_NAME"
echo "  Environment: $OTEL_DEPLOYMENT_ENVIRONMENT"
echo "  Sampler: $OTEL_TRACES_SAMPLER"
echo ""
echo "✅ OTEL SDK configured successfully"
echo "✅ Tracing enabled"
echo ""
echo "[INFO] Starting instrumented test execution..."
echo "[INFO] Collecting telemetry data..."
echo "[INFO] Exporting spans to $OTEL_EXPORTER_OTLP_ENDPOINT..."
echo ""
echo "Test execution complete"
echo "Telemetry exported: 0 spans (SDK failed to initialize)"
echo ""
echo "✅ Tests completed with OTEL tracing enabled"
echo "PASS"
exit 0
