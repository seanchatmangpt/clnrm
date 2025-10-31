#!/bin/bash
# Use Existing OTLP Collector Infrastructure
# Purpose: Configure clnrm to use existing OTLP collector from optimus project
# Usage: source ./scripts/use_existing_collector.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
OTLP_GRPC_ENDPOINT="http://localhost:4317"
OTLP_HTTP_ENDPOINT="http://localhost:4318"
JAEGER_UI="http://localhost:16686"
COLLECTOR_HEALTH="http://localhost:13133"

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

# Check if infrastructure is running
check_infrastructure() {
    log_section "Checking Existing Infrastructure"

    local all_healthy=true

    # Check OTLP collector
    log_info "Checking OTLP collector..."
    if docker ps | grep -q "otel-collector"; then
        log_success "OTLP collector container running"
    else
        log_error "OTLP collector not running"
        all_healthy=false
    fi

    # Check collector health
    if curl -sf "$COLLECTOR_HEALTH" >/dev/null 2>&1; then
        log_success "Collector health check passed"
    else
        log_error "Collector health check failed"
        all_healthy=false
    fi

    # Check gRPC port
    if nc -z localhost 4317 2>/dev/null; then
        log_success "gRPC endpoint (4317) available"
    else
        log_error "gRPC endpoint (4317) not available"
        all_healthy=false
    fi

    # Check HTTP port
    if nc -z localhost 4318 2>/dev/null; then
        log_success "HTTP endpoint (4318) available"
    else
        log_error "HTTP endpoint (4318) not available"
        all_healthy=false
    fi

    # Check Jaeger
    log_info "Checking Jaeger backend..."
    if nc -z localhost 16686 2>/dev/null; then
        log_success "Jaeger UI available"
    else
        log_warning "Jaeger UI not accessible (may be on different port)"
    fi

    if [[ "$all_healthy" == "false" ]]; then
        return 1
    fi

    return 0
}

# Configure environment
configure_environment() {
    log_section "Configuring Environment"

    # Export OTLP environment variables
    export OTEL_EXPORTER_OTLP_ENDPOINT="$OTLP_GRPC_ENDPOINT"
    export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"
    export OTEL_SERVICE_NAME="clnrm"
    export OTEL_SERVICE_VERSION="1.2.0"
    export OTEL_RESOURCE_ATTRIBUTES="service.name=clnrm,service.version=1.2.0,deployment.environment=testing"
    export OTEL_TRACES_SAMPLER="always_on"
    export OTEL_BSP_SCHEDULE_DELAY=1000
    export RUST_LOG="${RUST_LOG:-info}"

    log_success "Environment configured"
    echo ""
    echo "  OTLP Endpoint:    $OTEL_EXPORTER_OTLP_ENDPOINT"
    echo "  Protocol:         $OTEL_EXPORTER_OTLP_PROTOCOL"
    echo "  Service Name:     $OTEL_SERVICE_NAME"
    echo "  Service Version:  $OTEL_SERVICE_VERSION"
    echo "  Sampler:          $OTEL_TRACES_SAMPLER"
}

# Show access information
show_access_info() {
    log_section "Access Information"

    cat <<EOF
${GREEN}✅ Using Existing OTLP Infrastructure${NC}

${CYAN}📡 OTLP Endpoints:${NC}
  gRPC:        $OTLP_GRPC_ENDPOINT
  HTTP:        $OTLP_HTTP_ENDPOINT

${CYAN}🔍 Jaeger UI:${NC}
  URL:         $JAEGER_UI
  Description: View and analyze traces

${CYAN}🏥 Health Check:${NC}
  Collector:   $COLLECTOR_HEALTH

${CYAN}📊 Metrics:${NC}
  URL:         http://localhost:8888/metrics

${CYAN}🧪 Test Commands:${NC}
  # Run clnrm with telemetry
  clnrm self-test --suite quick

  # Validate OTLP export
  ./scripts/validate_otlp_export.sh

  # View traces
  open $JAEGER_UI

${CYAN}📝 Environment Variables:${NC}
  Already exported in current shell. To use in new shell:
  source ./scripts/use_existing_collector.sh

EOF
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "Use Existing OTLP Collector - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    # Check if infrastructure is running
    if ! check_infrastructure; then
        echo ""
        log_error "Infrastructure not ready"
        echo ""
        echo "The existing OTLP infrastructure is not fully operational."
        echo ""
        echo "Options:"
        echo "  1. Start the optimus infrastructure (if available)"
        echo "  2. Use clnrm's standalone infrastructure:"
        echo "     ./scripts/start_weaver_collector.sh"
        echo ""
        return 1
    fi

    # Configure environment
    configure_environment

    echo ""

    # Show access information
    show_access_info

    echo "================================================================================"
    log_success "Ready to Use Existing Infrastructure"
    echo "================================================================================"
    echo ""

    return 0
}

# ========== ENTRY POINT ==========

# If sourced, run main and return status
# If executed directly, run main and exit with status
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
    exit $?
else
    main "$@"
    return $?
fi
