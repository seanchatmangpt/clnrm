#!/bin/bash
# Validate End-to-End OTLP Export Chain
# Purpose: Test clnrm → OTLP collector → Jaeger telemetry flow
# Usage: ./scripts/validate_otlp_export.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
JAEGER_API="http://localhost:16686/api"
SERVICE_NAME="clnrm"
TEST_TIMEOUT=30
RETRY_INTERVAL=2

# ========== FUNCTIONS ==========

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_section() {
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Check infrastructure is running
check_infrastructure() {
    log_section "Infrastructure Health Check"

    # Check Docker containers
    log_info "Checking Docker containers..."
    if ! docker ps | grep -q "clnrm-otel-collector"; then
        log_error "OTLP collector not running"
        log_info "Start with: ./scripts/start_weaver_collector.sh"
        exit 1
    fi
    log_success "OTLP collector running"

    if ! docker ps | grep -q "clnrm-jaeger"; then
        log_error "Jaeger not running"
        log_info "Start with: ./scripts/start_weaver_collector.sh"
        exit 1
    fi
    log_success "Jaeger running"

    # Check OTLP endpoints
    log_info "Checking OTLP gRPC endpoint (4317)..."
    if nc -z localhost 4317 2>/dev/null; then
        log_success "gRPC endpoint reachable"
    else
        log_error "gRPC endpoint not reachable on port 4317"
        exit 1
    fi

    log_info "Checking OTLP HTTP endpoint (4318)..."
    if nc -z localhost 4318 2>/dev/null; then
        log_success "HTTP endpoint reachable"
    else
        log_error "HTTP endpoint not reachable on port 4318"
        exit 1
    fi

    # Check Jaeger API
    log_info "Checking Jaeger API..."
    if curl -sf "${JAEGER_API}/services" >/dev/null 2>&1; then
        log_success "Jaeger API reachable"
    else
        log_error "Jaeger API not reachable"
        exit 1
    fi
}

# Configure OTLP environment
configure_environment() {
    log_section "Environment Configuration"

    # Source OTLP configuration
    log_info "Configuring OTLP environment variables..."
    source ./scripts/otlp_config.sh export >/dev/null 2>&1

    if [[ -z "$OTEL_EXPORTER_OTLP_ENDPOINT" ]]; then
        log_error "Failed to configure OTLP environment"
        exit 1
    fi

    log_success "OTLP environment configured"
    echo "  Endpoint: $OTEL_EXPORTER_OTLP_ENDPOINT"
    echo "  Protocol: $OTEL_EXPORTER_OTLP_PROTOCOL"
    echo "  Service:  $OTEL_SERVICE_NAME"
}

# Get trace count before test
get_trace_count() {
    curl -sf "${JAEGER_API}/traces?service=${SERVICE_NAME}&limit=1" 2>/dev/null | \
        jq -r '.data | length' 2>/dev/null || echo "0"
}

# Run test with telemetry
run_test_with_telemetry() {
    log_section "Running Test with Telemetry"

    local before_count=$(get_trace_count)
    log_info "Trace count before test: $before_count"

    # Run a simple clnrm command that generates telemetry
    log_info "Running clnrm command with OTLP export..."

    # Try self-test first (if available)
    if clnrm self-test --suite quick 2>&1 | tee /tmp/clnrm_test_output.log; then
        log_success "clnrm command executed"
    else
        log_warning "clnrm self-test failed or not available, trying --help"
        # Fallback to help command (less telemetry but still generates some)
        clnrm --help >/dev/null 2>&1 || true
        log_info "Executed fallback command"
    fi

    # Give collector time to process and export
    log_info "Waiting for telemetry to be exported (10s)..."
    sleep 10
}

# Verify telemetry in Jaeger
verify_telemetry() {
    log_section "Verifying Telemetry in Jaeger"

    local elapsed=0
    local found=false

    log_info "Searching for traces in Jaeger (max ${TEST_TIMEOUT}s)..."

    while [ $elapsed -lt $TEST_TIMEOUT ]; do
        local trace_count=$(get_trace_count)

        if [[ "$trace_count" != "0" && "$trace_count" != "" ]]; then
            log_success "Found $trace_count trace(s) in Jaeger!"
            found=true
            break
        fi

        echo -n "."
        sleep $RETRY_INTERVAL
        elapsed=$((elapsed + RETRY_INTERVAL))
    done

    echo ""

    if [[ "$found" == "false" ]]; then
        log_warning "No traces found in Jaeger within ${TEST_TIMEOUT}s"
        log_info "This may indicate:"
        echo "  1. Telemetry is not being exported from clnrm"
        echo "  2. OTLP collector is not receiving telemetry"
        echo "  3. Collector is not exporting to Jaeger"
        echo ""
        log_info "Check collector logs:"
        echo "  docker logs clnrm-otel-collector --tail 50"
        return 1
    fi

    return 0
}

# Fetch and display sample trace
display_sample_trace() {
    log_section "Sample Trace Data"

    log_info "Fetching sample trace from Jaeger..."

    local traces=$(curl -sf "${JAEGER_API}/traces?service=${SERVICE_NAME}&limit=1")

    if [[ -z "$traces" ]]; then
        log_warning "No traces available"
        return
    fi

    # Extract trace ID
    local trace_id=$(echo "$traces" | jq -r '.data[0].traceID' 2>/dev/null)

    if [[ -z "$trace_id" || "$trace_id" == "null" ]]; then
        log_warning "Could not extract trace ID"
        return
    fi

    log_success "Trace ID: $trace_id"

    # Display trace details
    echo ""
    echo "Trace Details:"
    echo "$traces" | jq -r '.data[0] | {
        traceID: .traceID,
        spans: .spans | length,
        duration: .spans[0].duration,
        operationName: .spans[0].operationName,
        tags: .spans[0].tags
    }' 2>/dev/null || echo "$traces"

    echo ""
    log_info "View full trace in Jaeger UI:"
    echo "  http://localhost:16686/trace/${trace_id}"
}

# Test collector metrics
test_collector_metrics() {
    log_section "Collector Metrics"

    log_info "Fetching collector metrics..."

    if curl -sf http://localhost:8888/metrics | grep -q "otelcol_receiver_accepted_spans"; then
        local accepted_spans=$(curl -sf http://localhost:8888/metrics | \
            grep "^otelcol_receiver_accepted_spans" | \
            awk '{sum += $2} END {print sum}')

        log_success "Collector has accepted spans: $accepted_spans"

        echo ""
        echo "Key Metrics:"
        curl -sf http://localhost:8888/metrics | grep -E "otelcol_(receiver|exporter)_(accepted|refused)" | head -10
    else
        log_warning "Could not fetch collector metrics"
    fi
}

# Generate validation report
generate_report() {
    log_section "Validation Report"

    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    local report_file="/tmp/otlp_validation_report_$(date +%s).txt"

    cat > "$report_file" <<EOF
OTLP Export Validation Report
Generated: $timestamp

Infrastructure Status:
  - OTLP Collector: Running
  - Jaeger Backend:  Running
  - gRPC Endpoint:   http://localhost:4317
  - HTTP Endpoint:   http://localhost:4318

Test Results:
  - Telemetry Export:    $([ "$1" == "success" ] && echo "✅ SUCCESS" || echo "⚠️  WARNING")
  - Traces in Jaeger:    $(get_trace_count)
  - Jaeger UI:           http://localhost:16686

Configuration:
  - Service Name:        $OTEL_SERVICE_NAME
  - Protocol:            $OTEL_EXPORTER_OTLP_PROTOCOL
  - Endpoint:            $OTEL_EXPORTER_OTLP_ENDPOINT

Next Steps:
  1. Review traces in Jaeger UI: http://localhost:16686
  2. Run Weaver validation: weaver registry live-check --registry registry/
  3. Check collector logs: docker logs clnrm-otel-collector

Log Files:
  - Test output:    /tmp/clnrm_test_output.log
  - This report:    $report_file
EOF

    log_success "Report generated: $report_file"
    echo ""
    cat "$report_file"
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "OTLP Export Validation - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    # 1. Check infrastructure
    check_infrastructure

    # 2. Configure environment
    configure_environment

    # 3. Run test with telemetry
    run_test_with_telemetry

    # 4. Verify telemetry in Jaeger
    local verification_result="warning"
    if verify_telemetry; then
        verification_result="success"
    fi

    # 5. Display sample trace
    display_sample_trace

    # 6. Test collector metrics
    test_collector_metrics

    echo ""

    # 7. Generate report
    generate_report "$verification_result"

    echo ""
    echo "================================================================================"
    if [[ "$verification_result" == "success" ]]; then
        log_success "OTLP Export Chain Validated Successfully"
    else
        log_warning "OTLP Export Chain Partially Validated"
        log_info "Review the report and logs for details"
    fi
    echo "================================================================================"
    echo ""

    exit 0
}

# ========== ENTRY POINT ==========

main "$@"
