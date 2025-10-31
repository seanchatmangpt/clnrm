#!/bin/bash
# Docker Health Check and Readiness Probes
# Comprehensive Docker daemon verification

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TIMEOUT=${TIMEOUT:-60}
CHECK_INTERVAL=${CHECK_INTERVAL:-2}

# Health check results
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNED=0

# ========== FUNCTIONS ==========

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    CHECKS_WARNED=$((CHECKS_WARNED + 1))
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
    CHECKS_FAILED=$((CHECKS_FAILED + 1))
}

# Check 1: Docker command available
check_docker_installed() {
    echo -n "Checking Docker installation... "
    if command -v docker >/dev/null 2>&1; then
        log_success "Docker CLI installed"
        return 0
    else
        log_error "Docker CLI not found"
        return 1
    fi
}

# Check 2: Docker daemon responsive
check_docker_responsive() {
    echo -n "Checking Docker daemon responsiveness... "
    if timeout 5 docker info >/dev/null 2>&1; then
        log_success "Docker daemon responsive"
        return 0
    else
        log_error "Docker daemon not responding"
        return 1
    fi
}

# Check 3: Docker API version
check_docker_version() {
    echo -n "Checking Docker version... "
    local version=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "unknown")

    if [[ "$version" != "unknown" ]]; then
        log_success "Docker version: $version"
        return 0
    else
        log_error "Cannot determine Docker version"
        return 1
    fi
}

# Check 4: Docker can list containers
check_docker_ps() {
    echo -n "Checking container listing... "
    if docker ps >/dev/null 2>&1; then
        local count=$(docker ps -q | wc -l | tr -d ' ')
        log_success "Can list containers ($count running)"
        return 0
    else
        log_error "Cannot list containers"
        return 1
    fi
}

# Check 5: Docker can pull images
check_docker_pull() {
    echo -n "Checking image pull capability... "

    # Use a tiny image for testing
    if docker pull alpine:latest >/dev/null 2>&1; then
        log_success "Can pull images"
        return 0
    else
        log_warning "Cannot pull images (may be offline)"
        return 1
    fi
}

# Check 6: Docker can run containers
check_docker_run() {
    echo -n "Checking container execution... "

    if docker run --rm alpine:latest echo "test" >/dev/null 2>&1; then
        log_success "Can run containers"
        return 0
    else
        log_error "Cannot run containers"
        return 1
    fi
}

# Check 7: Docker has sufficient resources
check_docker_resources() {
    echo -n "Checking Docker resources... "

    local cpus=$(docker info --format '{{.NCPU}}' 2>/dev/null || echo "0")
    local memory=$(docker info --format '{{.MemTotal}}' 2>/dev/null || echo "0")

    if [[ "$cpus" -ge 2 ]] && [[ "$memory" -gt 0 ]]; then
        log_success "Resources: ${cpus} CPUs, ${memory} memory"
        return 0
    else
        log_warning "Limited resources: ${cpus} CPUs"
        return 1
    fi
}

# Check 8: Docker network functional
check_docker_network() {
    echo -n "Checking Docker networking... "

    if docker network ls >/dev/null 2>&1; then
        local networks=$(docker network ls --format '{{.Name}}' | wc -l | tr -d ' ')
        log_success "Networking functional ($networks networks)"
        return 0
    else
        log_error "Cannot access Docker networks"
        return 1
    fi
}

# Check 9: Docker storage functional
check_docker_storage() {
    echo -n "Checking Docker storage... "

    local driver=$(docker info --format '{{.Driver}}' 2>/dev/null || echo "unknown")

    if [[ "$driver" != "unknown" ]]; then
        log_success "Storage driver: $driver"
        return 0
    else
        log_error "Cannot determine storage driver"
        return 1
    fi
}

# Check 10: Docker cleanup works
check_docker_cleanup() {
    echo -n "Checking Docker cleanup... "

    # Try to prune stopped containers
    if docker container prune -f >/dev/null 2>&1; then
        log_success "Cleanup functional"
        return 0
    else
        log_warning "Cleanup may not work properly"
        return 1
    fi
}

# Wait for Docker to be ready
wait_for_docker_ready() {
    log_info "Waiting for Docker to be ready (max ${TIMEOUT}s)..."
    echo ""

    local elapsed=0

    while [[ $elapsed -lt $TIMEOUT ]]; do
        if docker ps >/dev/null 2>&1; then
            echo ""
            log_success "Docker is ready!"
            return 0
        fi

        echo -n "."
        sleep $CHECK_INTERVAL
        elapsed=$((elapsed + CHECK_INTERVAL))
    done

    echo ""
    log_error "Docker not ready within ${TIMEOUT}s"
    return 1
}

# Print detailed Docker info
print_docker_info() {
    echo ""
    echo "Docker Information:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    docker info --format '
  Server Version:    {{.ServerVersion}}
  OS/Arch:           {{.OperatingSystem}} / {{.Architecture}}
  Storage Driver:    {{.Driver}}
  Logging Driver:    {{.LoggingDriver}}
  CPUs:              {{.NCPU}}
  Total Memory:      {{.MemTotal}}
  Running:           {{.ContainersRunning}} containers
  Paused:            {{.ContainersPaused}} containers
  Stopped:           {{.ContainersStopped}} containers
  Images:            {{.Images}} images
' 2>/dev/null || echo "  (Unable to retrieve Docker info)"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Run all health checks
run_all_checks() {
    log_info "Running comprehensive Docker health checks..."
    echo ""

    # Critical checks (must pass)
    check_docker_installed || return 1
    check_docker_responsive || return 1
    check_docker_version || return 1
    check_docker_ps || return 1

    # Important checks (should pass)
    check_docker_storage || true
    check_docker_network || true
    check_docker_resources || true

    # Optional checks (nice to have)
    check_docker_run || true
    check_docker_pull || true
    check_docker_cleanup || true

    return 0
}

# Print summary
print_summary() {
    echo ""
    echo "Health Check Summary:"
    echo "  ✅ Passed:  $CHECKS_PASSED"
    echo "  ⚠️  Warned:  $CHECKS_WARNED"
    echo "  ❌ Failed:  $CHECKS_FAILED"
    echo ""

    if [[ $CHECKS_FAILED -eq 0 ]]; then
        log_success "Docker is healthy and ready for testing"
        return 0
    else
        log_error "Docker health check failed"
        return 1
    fi
}

# ========== MAIN LOGIC ==========

main() {
    local mode="${1:-check}"  # check, wait, info

    echo "================================================================================"
    echo "Docker Health Check - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    case "$mode" in
        check)
            # Run health checks
            if run_all_checks; then
                print_summary
                print_docker_info
                exit 0
            else
                print_summary
                exit 1
            fi
            ;;

        wait)
            # Wait for Docker and then check
            if wait_for_docker_ready; then
                echo ""
                if run_all_checks; then
                    print_summary
                    print_docker_info
                    exit 0
                else
                    print_summary
                    exit 1
                fi
            else
                exit 1
            fi
            ;;

        info)
            # Just print info
            if docker info >/dev/null 2>&1; then
                print_docker_info
                exit 0
            else
                log_error "Docker daemon not running"
                exit 1
            fi
            ;;

        quick)
            # Quick check (just daemon responsive)
            if check_docker_responsive; then
                exit 0
            else
                exit 1
            fi
            ;;

        help|--help|-h)
            echo "Usage: $0 [MODE]"
            echo ""
            echo "Modes:"
            echo "  check    Run comprehensive health checks (default)"
            echo "  wait     Wait for Docker to be ready, then check"
            echo "  info     Print Docker information"
            echo "  quick    Quick check (daemon responsive only)"
            echo ""
            echo "Environment Variables:"
            echo "  TIMEOUT         Max wait time in seconds (default: 60)"
            echo "  CHECK_INTERVAL  Check interval in seconds (default: 2)"
            echo ""
            echo "Exit Codes:"
            echo "  0  All critical checks passed"
            echo "  1  One or more critical checks failed"
            echo ""
            exit 0
            ;;

        *)
            log_error "Unknown mode: $mode"
            echo ""
            echo "Run '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# ========== ENTRY POINT ==========

main "$@"
