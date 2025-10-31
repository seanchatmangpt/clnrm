#!/bin/bash
# Start OTLP Collector Infrastructure for Weaver Validation
# Purpose: Automated startup of OTLP collector + Jaeger for clnrm telemetry
# Usage: ./scripts/start_weaver_collector.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.weaver.yml"
PROJECT_NAME="clnrm-weaver"
MAX_WAIT=60
CHECK_INTERVAL=3

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

# Check prerequisites
check_prerequisites() {
    log_section "Prerequisites Check"

    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi
    log_success "Docker installed"

    # Check Docker Compose
    if ! docker compose version >/dev/null 2>&1; then
        log_error "Docker Compose not found. Please install Docker Compose."
        exit 1
    fi
    log_success "Docker Compose installed"

    # Check Docker daemon
    if ! docker ps >/dev/null 2>&1; then
        log_error "Docker daemon not running. Please start Docker."
        log_info "Run: ./scripts/docker_startup.sh"
        exit 1
    fi
    log_success "Docker daemon running"

    # Check compose file exists
    if [[ ! -f "$COMPOSE_FILE" ]]; then
        log_error "Compose file not found: $COMPOSE_FILE"
        exit 1
    fi
    log_success "Compose file exists: $COMPOSE_FILE"

    # Check config file exists
    if [[ ! -f "config/otel-collector-config.yaml" ]]; then
        log_error "Collector config not found: config/otel-collector-config.yaml"
        exit 1
    fi
    log_success "Collector config exists"
}

# Stop existing infrastructure
stop_existing() {
    log_section "Stopping Existing Infrastructure"

    if docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps | grep -q "Up"; then
        log_info "Stopping existing containers..."
        docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" down
        log_success "Existing containers stopped"
    else
        log_info "No existing containers running"
    fi
}

# Start infrastructure
start_infrastructure() {
    log_section "Starting OTLP Infrastructure"

    log_info "Starting services..."
    docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" up -d

    if [[ $? -eq 0 ]]; then
        log_success "Services started"
    else
        log_error "Failed to start services"
        exit 1
    fi
}

# Wait for service health
wait_for_health() {
    local service="$1"
    local max_wait="$2"
    local elapsed=0

    log_info "Waiting for $service to be healthy (max ${max_wait}s)..."

    while [ $elapsed -lt $max_wait ]; do
        local health=$(docker inspect --format='{{.State.Health.Status}}' "$service" 2>/dev/null || echo "unknown")

        if [[ "$health" == "healthy" ]]; then
            log_success "$service is healthy"
            return 0
        fi

        echo -n "."
        sleep $CHECK_INTERVAL
        elapsed=$((elapsed + CHECK_INTERVAL))
    done

    echo ""
    log_error "$service did not become healthy within ${max_wait}s"
    return 1
}

# Test OTLP endpoints
test_endpoints() {
    log_section "Testing OTLP Endpoints"

    # Test gRPC endpoint (4317)
    log_info "Testing gRPC endpoint (4317)..."
    if nc -z localhost 4317 2>/dev/null; then
        log_success "gRPC endpoint listening on port 4317"
    else
        log_warning "gRPC endpoint not reachable on port 4317"
    fi

    # Test HTTP endpoint (4318)
    log_info "Testing HTTP endpoint (4318)..."
    if nc -z localhost 4318 2>/dev/null; then
        log_success "HTTP endpoint listening on port 4318"
    else
        log_warning "HTTP endpoint not reachable on port 4318"
    fi

    # Test collector health endpoint
    log_info "Testing collector health endpoint..."
    if curl -sf http://localhost:13133/ >/dev/null 2>&1; then
        log_success "Collector health endpoint responding"
    else
        log_warning "Collector health endpoint not responding"
    fi

    # Test Jaeger UI
    log_info "Testing Jaeger UI..."
    if curl -sf http://localhost:16686/ >/dev/null 2>&1; then
        log_success "Jaeger UI accessible at http://localhost:16686"
    else
        log_warning "Jaeger UI not accessible"
    fi
}

# Show service status
show_status() {
    log_section "Service Status"

    docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps

    echo ""
    log_info "Container logs (last 10 lines):"
    echo ""
    docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" logs --tail=10
}

# Show access information
show_access_info() {
    log_section "Access Information"

    cat <<EOF
${GREEN}✅ OTLP Infrastructure Ready!${NC}

${CYAN}📡 OTLP Endpoints:${NC}
  gRPC:        http://localhost:4317
  HTTP:        http://localhost:4318

${CYAN}🔍 Jaeger UI:${NC}
  URL:         http://localhost:16686
  Description: View and analyze traces

${CYAN}📊 Collector Metrics:${NC}
  URL:         http://localhost:8888/metrics
  Description: Prometheus metrics

${CYAN}🏥 Health Checks:${NC}
  Collector:   http://localhost:13133
  Jaeger:      http://localhost:14269

${CYAN}🛠️ Debug Tools:${NC}
  zpages:      http://localhost:55679/debug/tracez
  pprof:       http://localhost:1777/debug/pprof

${CYAN}📝 Environment Setup:${NC}
  # Export OTLP endpoint for clnrm
  export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
  export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"

  # Or use the configuration script
  source ./scripts/otlp_config.sh

${CYAN}🧪 Test OTLP Export:${NC}
  ./scripts/validate_otlp_export.sh

${CYAN}🛑 Stop Infrastructure:${NC}
  docker compose -f docker-compose.weaver.yml -p clnrm-weaver down

${CYAN}📋 View Logs:${NC}
  docker compose -f docker-compose.weaver.yml -p clnrm-weaver logs -f

EOF
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "OTLP Collector Infrastructure Startup - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    # 1. Check prerequisites
    check_prerequisites

    # 2. Stop existing infrastructure
    stop_existing

    # 3. Start infrastructure
    start_infrastructure

    echo ""

    # 4. Wait for services to be healthy
    log_section "Health Checks"

    if ! wait_for_health "clnrm-jaeger" 30; then
        log_error "Jaeger failed to start"
        log_info "Check logs: docker compose -f $COMPOSE_FILE -p $PROJECT_NAME logs jaeger"
        exit 1
    fi

    if ! wait_for_health "clnrm-otel-collector" 30; then
        log_error "OTLP collector failed to start"
        log_info "Check logs: docker compose -f $COMPOSE_FILE -p $PROJECT_NAME logs otel-collector"
        exit 1
    fi

    echo ""

    # 5. Test endpoints
    test_endpoints

    echo ""

    # 6. Show status
    show_status

    echo ""

    # 7. Show access information
    show_access_info

    echo "================================================================================"
    log_success "OTLP Infrastructure Ready for Weaver Validation"
    echo "================================================================================"
    echo ""

    exit 0
}

# ========== ENTRY POINT ==========

main "$@"
