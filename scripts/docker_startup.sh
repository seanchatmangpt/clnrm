#!/bin/bash
# Enhanced Docker Daemon Startup Script
# Handles Docker Desktop, Colima, and native Docker across macOS/Linux/Windows

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MAX_WAIT=120  # 2 minutes max wait
CHECK_INTERVAL=3
ELAPSED=0

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

# Detect operating system
detect_os() {
    case "$OSTYPE" in
        darwin*)  echo "macos" ;;
        linux*)   echo "linux" ;;
        msys*|cygwin*|win32) echo "windows" ;;
        *)        echo "unknown" ;;
    esac
}

# Check if Docker daemon is running
is_docker_running() {
    docker ps >/dev/null 2>&1
    return $?
}

# Check if Docker Desktop is available (macOS/Windows)
has_docker_desktop() {
    if [[ "$(detect_os)" == "macos" ]]; then
        [[ -d "/Applications/Docker.app" ]]
    elif [[ "$(detect_os)" == "windows" ]]; then
        # Check for Docker Desktop on Windows
        command -v "Docker Desktop.exe" >/dev/null 2>&1
    else
        return 1
    fi
}

# Check if Colima is available (macOS/Linux)
has_colima() {
    command -v colima >/dev/null 2>&1
}

# Check if native Docker service is available (Linux)
has_docker_service() {
    [[ "$(detect_os)" == "linux" ]] && command -v systemctl >/dev/null 2>&1
}

# Start Docker Desktop
start_docker_desktop() {
    local os=$(detect_os)

    log_info "Starting Docker Desktop..."

    if [[ "$os" == "macos" ]]; then
        open -a Docker
        log_info "Docker Desktop launched. Waiting for daemon to start..."
    elif [[ "$os" == "windows" ]]; then
        start "" "Docker Desktop.exe"
        log_info "Docker Desktop launched. Waiting for daemon to start..."
    else
        log_error "Docker Desktop not supported on $os"
        return 1
    fi

    return 0
}

# Start Colima
start_colima() {
    log_info "Starting Colima..."

    # Check if already running
    if colima status | grep -q "colima is running"; then
        log_success "Colima already running"
        return 0
    fi

    # Start with reasonable defaults
    colima start --cpu 2 --memory 4 --disk 50 || {
        log_error "Failed to start Colima"
        return 1
    }

    log_success "Colima started"
    return 0
}

# Start native Docker service (Linux)
start_docker_service() {
    log_info "Starting Docker service..."

    if sudo systemctl start docker; then
        log_success "Docker service started"
        return 0
    else
        log_error "Failed to start Docker service"
        return 1
    fi
}

# Wait for Docker daemon to be ready
wait_for_docker() {
    log_info "Waiting for Docker daemon to be ready (max ${MAX_WAIT}s)..."
    echo ""

    while [ $ELAPSED -lt $MAX_WAIT ]; do
        if is_docker_running; then
            echo ""
            log_success "Docker daemon is ready!"
            return 0
        fi

        echo -n "."
        sleep $CHECK_INTERVAL
        ELAPSED=$((ELAPSED + CHECK_INTERVAL))
    done

    echo ""
    log_error "Docker daemon did not start within ${MAX_WAIT}s"
    return 1
}

# Get Docker info for verification
show_docker_info() {
    log_info "Docker daemon information:"
    echo ""
    docker version --format '  Version: {{.Server.Version}}'
    docker info --format '  Driver: {{.Driver}}'
    docker info --format '  OS/Arch: {{.OperatingSystem}} / {{.Architecture}}'
    docker info --format '  CPUs: {{.NCPU}}'
    docker info --format '  Memory: {{.MemTotal}}'
    echo ""
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "Docker Daemon Startup - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    local os=$(detect_os)
    log_info "Detected OS: $os"
    echo ""

    # 1. Check if already running
    log_info "Checking Docker daemon status..."
    if is_docker_running; then
        log_success "Docker daemon already running!"
        echo ""
        show_docker_info
        exit 0
    fi

    log_warning "Docker daemon not running"
    echo ""

    # 2. Determine startup method
    local startup_method=""

    if has_docker_desktop; then
        startup_method="docker_desktop"
        log_info "Docker Desktop detected"
    elif has_colima; then
        startup_method="colima"
        log_info "Colima detected"
    elif has_docker_service; then
        startup_method="docker_service"
        log_info "Docker service detected"
    else
        log_error "No Docker runtime found!"
        echo ""
        echo "Please install one of the following:"
        echo "  • Docker Desktop: https://www.docker.com/products/docker-desktop"
        echo "  • Colima (macOS/Linux): brew install colima"
        echo "  • Docker Engine (Linux): https://docs.docker.com/engine/install/"
        echo ""
        exit 1
    fi

    echo ""

    # 3. Start Docker using detected method
    case "$startup_method" in
        docker_desktop)
            if ! start_docker_desktop; then
                log_error "Failed to start Docker Desktop"
                exit 1
            fi
            ;;
        colima)
            if ! start_colima; then
                log_error "Failed to start Colima"
                exit 1
            fi
            ;;
        docker_service)
            if ! start_docker_service; then
                log_error "Failed to start Docker service"
                exit 1
            fi
            ;;
    esac

    echo ""

    # 4. Wait for daemon to be ready
    if ! wait_for_docker; then
        echo ""
        echo "Troubleshooting:"
        echo "  1. Check for errors in Docker logs"
        echo "  2. Verify Docker has sufficient resources"
        echo "  3. Try restarting Docker manually"
        echo ""

        if [[ "$startup_method" == "docker_desktop" ]]; then
            echo "Docker Desktop logs:"
            if [[ "$os" == "macos" ]]; then
                echo "  ~/Library/Containers/com.docker.docker/Data/log/"
            fi
        elif [[ "$startup_method" == "colima" ]]; then
            echo "Colima logs:"
            echo "  colima logs"
        fi

        echo ""
        exit 1
    fi

    echo ""

    # 5. Verify Docker is working
    log_info "Verifying Docker functionality..."
    if docker run --rm hello-world >/dev/null 2>&1; then
        log_success "Docker is fully functional"
    else
        log_warning "Docker daemon running but test container failed"
        log_info "This may not affect clnrm tests"
    fi

    echo ""
    show_docker_info

    echo "================================================================================"
    log_success "Docker daemon ready for testing"
    echo "================================================================================"
    echo ""

    exit 0
}

# ========== ENTRY POINT ==========

main "$@"
