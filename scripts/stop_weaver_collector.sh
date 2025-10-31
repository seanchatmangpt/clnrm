#!/bin/bash
# Stop OTLP Collector Infrastructure
# Purpose: Clean shutdown of OTLP collector + Jaeger
# Usage: ./scripts/stop_weaver_collector.sh [--clean]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.weaver.yml"
PROJECT_NAME="clnrm-weaver"

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

# Stop services
stop_services() {
    log_info "Stopping OTLP infrastructure..."

    if docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps | grep -q "Up"; then
        docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" down

        if [[ $? -eq 0 ]]; then
            log_success "Services stopped"
        else
            log_error "Failed to stop services"
            return 1
        fi
    else
        log_info "Services already stopped"
    fi

    return 0
}

# Clean volumes and data
clean_data() {
    log_info "Removing volumes and data..."

    docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" down -v

    if [[ $? -eq 0 ]]; then
        log_success "Volumes removed"
    else
        log_warning "Failed to remove some volumes"
    fi
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "OTLP Collector Infrastructure Shutdown - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    local clean_mode=false

    # Parse arguments
    if [[ "$1" == "--clean" ]]; then
        clean_mode=true
        log_warning "Clean mode: will remove all volumes and data"
        echo ""
    fi

    # Stop services
    if ! stop_services; then
        exit 1
    fi

    # Clean data if requested
    if [[ "$clean_mode" == "true" ]]; then
        echo ""
        clean_data
    fi

    echo ""
    echo "================================================================================"
    log_success "OTLP Infrastructure Stopped"
    echo "================================================================================"
    echo ""

    exit 0
}

# ========== ENTRY POINT ==========

main "$@"
