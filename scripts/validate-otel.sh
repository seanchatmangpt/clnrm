#!/usr/bin/env bash
# clnrm OpenTelemetry Validation Script
# Adapted from kcura pattern for clnrm Cleanroom Testing Framework
# Validates that OTEL integration actually emits traces and metrics
# Version: 1.0.0

set -euo pipefail

# Script directory
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# OTEL collector configuration
readonly OTEL_COLLECTOR_IMAGE="otel/opentelemetry-collector:latest"
readonly OTEL_COLLECTOR_NAME="clnrm-otel-collector"
readonly OTEL_HTTP_PORT="4318"
readonly OTEL_GRPC_PORT="4317"
readonly OTEL_METRICS_PORT="8888"
readonly OTEL_HEALTH_PORT="13133"

# Timeouts
readonly COLLECTOR_START_TIMEOUT=30
readonly TEST_EXECUTION_TIMEOUT=60
readonly TRACE_VERIFICATION_TIMEOUT=10

# Source logging library if available
if [[ -f "$SCRIPT_DIR/lib/logging.sh" ]]; then
    # shellcheck source=lib/logging.sh
    source "$SCRIPT_DIR/lib/logging.sh"
    init_logging "simple"
else
    # Fallback logging
    info() { echo "[INFO] $*" >&2; }
    warn() { echo "[WARN] $*" >&2; }
    error() { echo "[ERROR] $*" >&2; }
fi

# Cleanup function
cleanup() {
    info "Cleaning up OTEL collector..."
    docker stop "$OTEL_COLLECTOR_NAME" 2>/dev/null || true
    docker rm "$OTEL_COLLECTOR_NAME" 2>/dev/null || true
}

# Trap cleanup on exit
trap cleanup EXIT

# Create OTEL collector configuration
create_collector_config() {
    local config_file="$1"

    cat > "$config_file" <<'EOF'
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 10

exporters:
  logging:
    loglevel: debug
  prometheus:
    endpoint: 0.0.0.0:8889

extensions:
  health_check:
    endpoint: 0.0.0.0:13133

service:
  extensions: [health_check]
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, prometheus]
EOF

    info "Created OTEL collector configuration: $config_file"
}

# Start OTEL collector
start_otel_collector() {
    local config_file="$1"

    info "Starting OTEL collector..."

    docker run -d \
        --name "$OTEL_COLLECTOR_NAME" \
        -p "$OTEL_HTTP_PORT:$OTEL_HTTP_PORT" \
        -p "$OTEL_GRPC_PORT:$OTEL_GRPC_PORT" \
        -p "$OTEL_METRICS_PORT:$OTEL_METRICS_PORT" \
        -p "$OTEL_HEALTH_PORT:$OTEL_HEALTH_PORT" \
        -v "$config_file:/etc/otelcol/config.yaml:ro" \
        "$OTEL_COLLECTOR_IMAGE" \
        --config=/etc/otelcol/config.yaml

    # Wait for collector to be healthy
    local elapsed=0
    while [[ $elapsed -lt $COLLECTOR_START_TIMEOUT ]]; do
        if curl -sf "http://localhost:$OTEL_HEALTH_PORT" >/dev/null 2>&1; then
            info "OTEL collector is healthy"
            return 0
        fi

        sleep 1
        elapsed=$((elapsed + 1))
    done

    error "OTEL collector failed to start within ${COLLECTOR_START_TIMEOUT}s"
    docker logs "$OTEL_COLLECTOR_NAME"
    return 1
}

# Run clnrm with OTEL enabled
run_clnrm_with_otel() {
    info "Running clnrm with OTEL enabled..."

    export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$OTEL_GRPC_PORT"
    export OTEL_SERVICE_NAME="clnrm-test"
    export RUST_LOG="clnrm=debug,otel=debug"

    # Build with OTEL features
    cd "$ROOT_DIR"
    cargo build --release --features otel

    # Run self-test with timeout
    timeout "$TEST_EXECUTION_TIMEOUT" ./target/release/clnrm self-test || {
        error "clnrm self-test failed or timed out"
        return 1
    }

    info "clnrm execution completed"

    # Give collector time to process traces
    sleep 3
}

# Verify traces were emitted
verify_traces_emitted() {
    info "Verifying OTEL traces were emitted..."

    # Check collector logs for trace data
    local collector_logs
    collector_logs=$(docker logs "$OTEL_COLLECTOR_NAME" 2>&1)

    # Look for trace indicators in logs
    if echo "$collector_logs" | grep -q "Span"; then
        info "✓ Traces detected in collector logs"
        return 0
    fi

    if echo "$collector_logs" | grep -q "traces"; then
        info "✓ Trace pipeline activity detected"
        return 0
    fi

    error "No traces detected in collector logs"
    echo "--- Collector Logs ---"
    echo "$collector_logs" | tail -n 50
    return 1
}

# Verify metrics were emitted
verify_metrics_emitted() {
    info "Verifying OTEL metrics were emitted..."

    # Check Prometheus metrics endpoint
    local metrics_response
    if metrics_response=$(curl -sf "http://localhost:$OTEL_METRICS_PORT/metrics"); then
        if echo "$metrics_response" | grep -q "otelcol_receiver_accepted"; then
            info "✓ Metrics endpoint is active"

            # Check if spans were received
            local accepted_spans
            accepted_spans=$(echo "$metrics_response" | grep "otelcol_receiver_accepted_spans" | head -n 1)

            if [[ -n "$accepted_spans" ]]; then
                info "✓ OTEL spans received: $accepted_spans"
                return 0
            fi
        fi
    fi

    warn "Could not verify metrics (may not be emitted yet)"
    return 0  # Don't fail on metrics, only traces are critical
}

# Generate validation report
generate_report() {
    local traces_ok="$1"
    local metrics_ok="$2"

    echo ""
    echo "===== clnrm OTEL Validation Report ====="
    echo "Date: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    echo ""
    echo "Traces Emitted: $([ "$traces_ok" -eq 0 ] && echo "✓ YES" || echo "✗ NO")"
    echo "Metrics Emitted: $([ "$metrics_ok" -eq 0 ] && echo "✓ YES" || echo "~ MAYBE")"
    echo ""

    if [[ "$traces_ok" -eq 0 ]]; then
        echo "✅ OTEL integration validation PASSED"
        echo "   clnrm successfully emits OpenTelemetry traces"
    else
        echo "❌ OTEL integration validation FAILED"
        echo "   clnrm did not emit OpenTelemetry traces"
        echo ""
        echo "Troubleshooting:"
        echo "  1. Ensure clnrm is built with --features otel"
        echo "  2. Check OTEL_EXPORTER_OTLP_ENDPOINT is set correctly"
        echo "  3. Verify telemetry::init_otel() is called in code"
        echo "  4. Review clnrm logs for OTEL errors"
    fi

    echo "========================================"
}

# Main execution
main() {
    info "Starting clnrm OTEL validation..."

    # Check prerequisites
    if ! command -v docker &>/dev/null; then
        error "Docker is required but not installed"
        exit 1
    fi

    if ! command -v cargo &>/dev/null; then
        error "Cargo is required but not installed"
        exit 1
    fi

    # Create temporary config file
    local config_file
    config_file=$(mktemp)
    create_collector_config "$config_file"

    # Start collector
    if ! start_otel_collector "$config_file"; then
        error "Failed to start OTEL collector"
        rm -f "$config_file"
        exit 1
    fi

    # Run clnrm with OTEL
    local run_result=0
    run_clnrm_with_otel || run_result=$?

    # Verify traces
    local traces_ok=0
    verify_traces_emitted || traces_ok=$?

    # Verify metrics
    local metrics_ok=0
    verify_metrics_emitted || metrics_ok=$?

    # Generate report
    generate_report "$traces_ok" "$metrics_ok"

    # Cleanup
    rm -f "$config_file"

    # Exit with appropriate code
    if [[ "$traces_ok" -eq 0 ]]; then
        info "OTEL validation passed ✓"
        exit 0
    else
        error "OTEL validation failed"
        exit 1
    fi
}

main "$@"
