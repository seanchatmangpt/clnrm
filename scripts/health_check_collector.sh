#!/bin/bash
# Health Check for OTLP Collector Infrastructure
# Purpose: Quick health status of collector and Jaeger
# Usage: ./scripts/health_check_collector.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

check_symbol() {
    if [[ "$1" == "true" ]]; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
    fi
}

# Check Docker containers
check_containers() {
    echo -e "${CYAN}Docker Containers:${NC}"

    local collector_running=false
    local jaeger_running=false

    if docker ps | grep -q "clnrm-otel-collector"; then
        collector_running=true
    fi

    if docker ps | grep -q "clnrm-jaeger"; then
        jaeger_running=true
    fi

    echo "  Collector: $(check_symbol $collector_running)"
    echo "  Jaeger:    $(check_symbol $jaeger_running)"

    if [[ "$collector_running" == "false" ]] || [[ "$jaeger_running" == "false" ]]; then
        return 1
    fi

    return 0
}

# Check container health
check_health() {
    echo ""
    echo -e "${CYAN}Container Health:${NC}"

    local collector_health=$(docker inspect --format='{{.State.Health.Status}}' clnrm-otel-collector 2>/dev/null || echo "unknown")
    local jaeger_health=$(docker inspect --format='{{.State.Health.Status}}' clnrm-jaeger 2>/dev/null || echo "unknown")

    echo "  Collector: $collector_health"
    echo "  Jaeger:    $jaeger_health"

    if [[ "$collector_health" != "healthy" ]] || [[ "$jaeger_health" != "healthy" ]]; then
        return 1
    fi

    return 0
}

# Check network ports
check_ports() {
    echo ""
    echo -e "${CYAN}Network Ports:${NC}"

    local grpc_listening=false
    local http_listening=false
    local jaeger_ui=false

    if nc -z localhost 4317 2>/dev/null; then
        grpc_listening=true
    fi

    if nc -z localhost 4318 2>/dev/null; then
        http_listening=true
    fi

    if nc -z localhost 16686 2>/dev/null; then
        jaeger_ui=true
    fi

    echo "  gRPC (4317):     $(check_symbol $grpc_listening)"
    echo "  HTTP (4318):     $(check_symbol $http_listening)"
    echo "  Jaeger UI (16686): $(check_symbol $jaeger_ui)"

    if [[ "$grpc_listening" == "false" ]] || [[ "$http_listening" == "false" ]] || [[ "$jaeger_ui" == "false" ]]; then
        return 1
    fi

    return 0
}

# Check HTTP endpoints
check_endpoints() {
    echo ""
    echo -e "${CYAN}HTTP Endpoints:${NC}"

    local collector_health=false
    local jaeger_api=false

    if curl -sf http://localhost:13133/ >/dev/null 2>&1; then
        collector_health=true
    fi

    if curl -sf http://localhost:16686/api/services >/dev/null 2>&1; then
        jaeger_api=true
    fi

    echo "  Collector Health: $(check_symbol $collector_health)"
    echo "  Jaeger API:       $(check_symbol $jaeger_api)"

    if [[ "$collector_health" == "false" ]] || [[ "$jaeger_api" == "false" ]]; then
        return 1
    fi

    return 0
}

# Check for traces
check_traces() {
    echo ""
    echo -e "${CYAN}Telemetry Data:${NC}"

    local trace_count=$(curl -sf http://localhost:16686/api/traces?service=clnrm&limit=1 2>/dev/null | \
        jq -r '.data | length' 2>/dev/null || echo "0")

    echo "  Traces in Jaeger: $trace_count"

    if [[ "$trace_count" == "0" ]]; then
        log_warning "No traces found (this is normal if no tests have run yet)"
    fi
}

# Show resource usage
check_resources() {
    echo ""
    echo -e "${CYAN}Resource Usage:${NC}"

    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" \
        clnrm-otel-collector clnrm-jaeger 2>/dev/null || echo "  Unable to fetch stats"
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "OTLP Collector Health Check - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    local all_healthy=true

    # Check containers
    if ! check_containers; then
        all_healthy=false
        echo ""
        log_error "Some containers not running"
        log_info "Start with: ./scripts/start_weaver_collector.sh"
        echo ""
    fi

    # Check health (only if containers are running)
    if [[ "$all_healthy" == "true" ]]; then
        if ! check_health; then
            all_healthy=false
            echo ""
            log_warning "Some containers not healthy yet (may still be starting)"
        fi
    fi

    # Check ports
    if [[ "$all_healthy" == "true" ]]; then
        if ! check_ports; then
            all_healthy=false
            echo ""
            log_error "Some ports not listening"
        fi
    fi

    # Check endpoints
    if [[ "$all_healthy" == "true" ]]; then
        if ! check_endpoints; then
            all_healthy=false
            echo ""
            log_error "Some endpoints not responding"
        fi
    fi

    # Check for traces (informational only)
    if [[ "$all_healthy" == "true" ]]; then
        check_traces
    fi

    # Show resource usage
    if docker ps | grep -q "clnrm-otel-collector"; then
        check_resources
    fi

    echo ""
    echo "================================================================================"
    if [[ "$all_healthy" == "true" ]]; then
        log_success "All Systems Healthy"
        echo ""
        echo "🔗 Quick Links:"
        echo "   Jaeger UI:  http://localhost:16686"
        echo "   Metrics:    http://localhost:8888/metrics"
        echo ""
        exit 0
    else
        log_error "Health Check Failed"
        echo ""
        echo "🔧 Troubleshooting:"
        echo "   View logs:  docker compose -f docker-compose.weaver.yml logs"
        echo "   Restart:    ./scripts/start_weaver_collector.sh"
        echo ""
        exit 1
    fi
    echo "================================================================================"
}

# ========== ENTRY POINT ==========

main "$@"
