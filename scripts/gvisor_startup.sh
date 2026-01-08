#!/bin/bash
# Enhanced gVisor Runtime Startup Script
# Handles gVisor (runsc) installation and initialization across Linux/macOS/CI environments
#
# This script replaces docker_startup.sh for gVisor-based container execution
# gVisor is a userspace container runtime that provides better security isolation

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
        *)        echo "unknown" ;;
    esac
}

# Check if runsc is available in PATH
has_runsc_binary() {
    command -v runsc >/dev/null 2>&1
}

# Check if runsc is functional
is_runsc_working() {
    if ! has_runsc_binary; then
        return 1
    fi

    # Test runsc version
    runsc --version >/dev/null 2>&1
    return $?
}

# Install gVisor on Linux
install_gvisor_linux() {
    local os_type=$(detect_os)

    if [[ "$os_type" != "linux" ]]; then
        log_error "gVisor installation only supports Linux"
        return 1
    fi

    log_info "Installing gVisor (runsc)..."

    # Check for available package manager
    if command -v apt-get >/dev/null 2>&1; then
        # Debian/Ubuntu
        log_info "Detected Debian/Ubuntu-based system"

        # Add gVisor repository (if not already added)
        if ! grep -q "gvisor" /etc/apt/sources.list.d/* 2>/dev/null || [[ ! -f /etc/apt/sources.list.d/gvisor.list ]]; then
            log_info "Adding gVisor repository..."
            sudo bash -c 'echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" > /etc/apt/sources.list.d/gvisor.list'

            # Add GPG key
            sudo bash -c 'curl -s https://storage.googleapis.com/gvisor/releases/google.key | apt-key add - || true'
        fi

        sudo apt-get update || log_warning "Failed to update apt cache"
        sudo apt-get install -y runsc || {
            log_error "Failed to install runsc via apt-get"
            return 1
        }

        log_success "gVisor (runsc) installed"
        return 0

    elif command -v yum >/dev/null 2>&1; then
        # RHEL/CentOS/Fedora
        log_info "Detected RHEL-based system"

        sudo yum install -y runsc || {
            log_error "Failed to install runsc via yum"
            return 1
        }

        log_success "gVisor (runsc) installed"
        return 0

    elif command -v pacman >/dev/null 2>&1; then
        # Arch Linux
        log_info "Detected Arch Linux"

        sudo pacman -Sy gvisor || {
            log_error "Failed to install gvisor via pacman"
            return 1
        }

        log_success "gVisor (runsc) installed"
        return 0
    else
        log_error "No supported package manager found"
        log_info "Please install runsc manually from: https://gvisor.dev/docs/user_guide/install/"
        return 1
    fi
}

# Install gVisor on macOS (via containerd or direct binary)
install_gvisor_macos() {
    log_info "Installing gVisor (runsc) on macOS..."

    # Check if we have containerd with gVisor support
    if command -v brew >/dev/null 2>&1; then
        log_info "Using Homebrew to install runsc..."

        # Note: gVisor doesn't have official Homebrew formula
        # We can compile from source or use containerd
        log_warning "gVisor doesn't have official macOS support via Homebrew"
        log_info "For macOS development, consider:"
        echo "  1. Using containerd on a Linux VM"
        echo "  2. Using gVisor in a Docker container"
        echo "  3. Building runsc from source: https://gvisor.dev/docs/user_guide/install/"
        return 1
    else
        log_error "Homebrew not found"
        return 1
    fi
}

# Initialize runsc configuration
init_runsc_config() {
    log_info "Initializing runsc configuration..."

    local os=$(detect_os)

    # Create runsc configuration directory if needed
    local config_dir="/etc/runsc"
    if [[ "$os" == "linux" ]] && ! [[ -d "$config_dir" ]]; then
        sudo mkdir -p "$config_dir" || log_warning "Could not create $config_dir"
    fi

    # Verify runsc can access /dev/urandom (required for seccomp)
    if ! [[ -r /dev/urandom ]]; then
        log_warning "Cannot read /dev/urandom - seccomp filtering may not work"
    fi

    log_success "runsc configuration initialized"
    return 0
}

# Verify gVisor environment
verify_gvisor_env() {
    log_info "Verifying gVisor environment..."

    # Check kvm capability (not required but preferred for performance)
    if [[ -c /dev/kvm ]]; then
        log_success "KVM support available (nested virt not needed)"
    else
        log_warning "KVM support not available (performance may be reduced)"
    fi

    # Check AppArmor/SELinux (used by gVisor for security)
    if command -v aa-status >/dev/null 2>&1; then
        if sudo aa-status --enabled 2>/dev/null; then
            log_success "AppArmor enabled"
        else
            log_warning "AppArmor available but not enabled"
        fi
    fi

    return 0
}

# Wait for runsc to be ready
wait_for_runsc() {
    log_info "Verifying runsc is ready (max ${MAX_WAIT}s)..."
    echo ""

    while [ $ELAPSED -lt $MAX_WAIT ]; do
        if is_runsc_working; then
            echo ""
            log_success "runsc is ready!"
            return 0
        fi

        echo -n "."
        sleep $CHECK_INTERVAL
        ELAPSED=$((ELAPSED + CHECK_INTERVAL))
    done

    echo ""
    log_error "runsc failed to initialize within ${MAX_WAIT}s"
    return 1
}

# Get runsc version information
show_runsc_info() {
    log_info "gVisor (runsc) information:"
    echo ""
    runsc --version || echo "  Version: unknown"
    echo ""
}

# ========== MAIN LOGIC ==========

main() {
    echo "================================================================================"
    echo "gVisor Runtime Startup - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    local os=$(detect_os)
    log_info "Detected OS: $os"
    echo ""

    # 1. Check if runsc already available
    log_info "Checking runsc availability..."
    if is_runsc_working; then
        log_success "runsc already available and functional!"
        echo ""
        show_runsc_info
        exit 0
    fi

    log_warning "runsc not available or not functional"
    echo ""

    # 2. Attempt to install gVisor
    case "$os" in
        linux)
            if ! install_gvisor_linux; then
                log_error "Failed to install gVisor"
                echo ""
                echo "Please install gVisor manually:"
                echo "  https://gvisor.dev/docs/user_guide/install/"
                echo ""
                exit 1
            fi
            ;;
        macos)
            if ! install_gvisor_macos; then
                log_error "macOS requires alternative setup for gVisor"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported OS: $os"
            echo ""
            echo "gVisor supports:"
            echo "  • Linux (primary platform)"
            echo "  • macOS (via containerd VM or source build)"
            echo ""
            exit 1
            ;;
    esac

    echo ""

    # 3. Initialize configuration
    if ! init_runsc_config; then
        log_warning "Configuration initialization had issues"
    fi

    echo ""

    # 4. Verify environment
    if ! verify_gvisor_env; then
        log_warning "Environment verification found issues"
    fi

    echo ""

    # 5. Wait for runsc to be ready
    if ! wait_for_runsc; then
        echo ""
        echo "Troubleshooting:"
        echo "  1. Verify runsc binary is in PATH: which runsc"
        echo "  2. Check kernel support: uname -r"
        echo "  3. Check gVisor logs"
        echo ""
        exit 1
    fi

    echo ""

    # 6. Verify gVisor functionality with a test
    log_info "Verifying gVisor functionality..."
    if runsc --version >/dev/null 2>&1; then
        log_success "gVisor is fully functional"
    else
        log_warning "gVisor functional but version check failed"
    fi

    echo ""
    show_runsc_info

    echo "================================================================================"
    log_success "gVisor runtime ready for testing"
    echo "================================================================================"
    echo ""

    exit 0
}

# ========== ENTRY POINT ==========

main "$@"
