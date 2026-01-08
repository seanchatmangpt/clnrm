#!/bin/bash
# gVisor Health Check and Readiness Probes
# Comprehensive gVisor runtime verification (replaces docker_health_check.sh)

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

# Check 1: runsc binary installed
check_runsc_installed() {
    echo -n "Checking runsc installation... "
    if command -v runsc >/dev/null 2>&1; then
        log_success "runsc binary found"
        return 0
    else
        log_error "runsc not found in PATH"
        return 1
    fi
}

# Check 2: runsc responds to version command
check_runsc_responsive() {
    echo -n "Checking runsc responsiveness... "
    if timeout 5 runsc --version >/dev/null 2>&1; then
        log_success "runsc responsive"
        return 0
    else
        log_error "runsc not responding"
        return 1
    fi
}

# Check 3: runsc version info
check_runsc_version() {
    echo -n "Checking runsc version... "
    local version=$(runsc --version 2>&1 | head -1 || echo "unknown")

    if [[ "$version" != "unknown" ]] && [[ -n "$version" ]]; then
        log_success "runsc version: $version"
        return 0
    else
        log_error "Cannot determine runsc version"
        return 1
    fi
}

# Check 4: runsc list containers capability
check_runsc_list() {
    echo -n "Checking runsc list capability... "
    if runsc list >/dev/null 2>&1; then
        local count=$(runsc list | tail -n +2 | wc -l | tr -d ' ')
        log_success "Can list containers ($count running)"
        return 0
    else
        log_error "Cannot list containers"
        return 1
    fi
}

# Check 5: Seccomp support (gVisor-specific security feature)
check_seccomp_support() {
    echo -n "Checking seccomp support... "

    # Check if seccomp is available on the system
    if [[ -f /boot/config-$(uname -r) ]]; then
        if grep -q "CONFIG_SECCOMP=y" /boot/config-$(uname -r); then
            log_success "Seccomp supported"
            return 0
        else
            log_warning "Seccomp not enabled in kernel config"
            return 1
        fi
    else
        # For systems where config is not available
        log_warning "Cannot verify seccomp (config not accessible)"
        return 1
    fi
}

# Check 6: Namespace isolation support
check_namespace_support() {
    echo -n "Checking namespace isolation... "

    # Check for namespace support
    local ns_types=("pid" "mount" "ipc" "network" "user" "cgroup")
    local missing_ns=0

    for ns in "${ns_types[@]}"; do
        if ! [[ -f "/proc/self/ns/$ns" ]]; then
            ((missing_ns++))
        fi
    done

    if [[ $missing_ns -eq 0 ]]; then
        log_success "All namespaces available"
        return 0
    else
        log_warning "Missing $missing_ns namespace(s)"
        return 1
    fi
}

# Check 7: cgroup support (resource limits for gVisor)
check_cgroup_support() {
    echo -n "Checking cgroup support... "

    if [[ -d /sys/fs/cgroup ]] || [[ -d /proc/cgroups ]]; then
        local version=""
        if grep -q "cgroup2" /proc/filesystems 2>/dev/null; then
            version="v2"
        else
            version="v1"
        fi
        log_success "cgroup $version supported"
        return 0
    else
        log_error "No cgroup support detected"
        return 1
    fi
}

# Check 8: KVM capability (performance enhancement, not required)
check_kvm_support() {
    echo -n "Checking KVM capability... "

    if [[ -c /dev/kvm ]]; then
        # Check if we can access it
        if timeout 2 runsc --help >/dev/null 2>&1; then
            log_success "KVM available"
            return 0
        else
            log_warning "KVM available but may not be accessible"
            return 1
        fi
    else
        log_warning "KVM not available (nested virtualization not needed)"
        return 1
    fi
}

# Check 9: AppArmor/SELinux integration
check_mac_support() {
    echo -n "Checking MAC (AppArmor/SELinux) support... "

    local has_apparmor=0
    local has_selinux=0

    if command -v aa-status >/dev/null 2>&1; then
        has_apparmor=1
    fi

    if command -v getenforce >/dev/null 2>&1; then
        has_selinux=1
    fi

    if [[ $has_apparmor -eq 1 ]]; then
        log_success "AppArmor available"
        return 0
    elif [[ $has_selinux -eq 1 ]]; then
        log_success "SELinux available"
        return 0
    else
        log_warning "No MAC framework detected (optional)"
        return 1
    fi
}

# Check 10: gVisor state tracking capability
check_gvisor_state() {
    echo -n "Checking gVisor state tracking... "

    # Check if runsc can query state
    if runsc list -format json >/dev/null 2>&1; then
        log_success "State tracking functional"
        return 0
    else
        log_warning "State tracking may have issues"
        return 1
    fi
}

# Wait for gVisor to be ready
wait_for_gvisor_ready() {
    log_info "Waiting for gVisor to be ready (max ${TIMEOUT}s)..."
    echo ""

    local elapsed=0

    while [[ $elapsed -lt $TIMEOUT ]]; do
        if runsc --version >/dev/null 2>&1; then
            echo ""
            log_success "gVisor is ready!"
            return 0
        fi

        echo -n "."
        sleep $CHECK_INTERVAL
        elapsed=$((elapsed + CHECK_INTERVAL))
    done

    echo ""
    log_error "gVisor not ready within ${TIMEOUT}s"
    return 1
}

# Print detailed runsc info
print_gvisor_info() {
    echo ""
    echo "gVisor (runsc) Information:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Version
    echo "  Version:"
    runsc --version 2>/dev/null || echo "    (unavailable)"

    # List containers
    echo "  Running containers:"
    local running=$(runsc list 2>/dev/null | tail -n +2 | grep "running" | wc -l || echo "0")
    echo "    $running running"

    # Platform
    echo "  Platform:"
    echo "    Kernel: $(uname -r)"
    echo "    Arch: $(uname -m)"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Run all health checks
run_all_checks() {
    log_info "Running comprehensive gVisor health checks..."
    echo ""

    # Critical checks (must pass)
    check_runsc_installed || return 1
    check_runsc_responsive || return 1
    check_runsc_version || return 1
    check_runsc_list || return 1

    # Important checks (should pass)
    check_cgroup_support || true
    check_namespace_support || true
    check_seccomp_support || true

    # Optional checks (nice to have)
    check_kvm_support || true
    check_mac_support || true
    check_gvisor_state || true

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
        log_success "gVisor is healthy and ready for testing"
        return 0
    else
        log_error "gVisor health check failed"
        return 1
    fi
}

# ========== MAIN LOGIC ==========

main() {
    local mode="${1:-check}"  # check, wait, info

    echo "================================================================================"
    echo "gVisor Health Check - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    case "$mode" in
        check)
            # Run health checks
            if run_all_checks; then
                print_summary
                print_gvisor_info
                exit 0
            else
                print_summary
                exit 1
            fi
            ;;

        wait)
            # Wait for gVisor and then check
            if wait_for_gvisor_ready; then
                echo ""
                if run_all_checks; then
                    print_summary
                    print_gvisor_info
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
            if runsc --version >/dev/null 2>&1; then
                print_gvisor_info
                exit 0
            else
                log_error "gVisor not available"
                exit 1
            fi
            ;;

        quick)
            # Quick check (just runsc responsive)
            if check_runsc_responsive; then
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
            echo "  wait     Wait for gVisor to be ready, then check"
            echo "  info     Print gVisor information"
            echo "  quick    Quick check (runsc responsive only)"
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
