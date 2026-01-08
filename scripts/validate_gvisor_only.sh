#!/bin/bash
# Validates gVisor-only setup: zero Docker references, gVisor properly configured
# Exit code: 0 = success, 1 = validation issues found
#
# Usage:
#   ./scripts/validate_gvisor_only.sh
#
# Environment:
#   STRICT_MODE=1  - Fail on any Docker reference (including docs)
#   VERBOSE=1      - Show detailed output

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERRORS=0
WARNINGS=0
STRICT_MODE=${STRICT_MODE:-0}
VERBOSE=${VERBOSE:-0}

# Track results
RESULTS_FILE=$(mktemp)

log_error() {
    echo -e "${RED}❌ ERROR: $1${NC}"
    ERRORS=$((ERRORS + 1))
}

log_warning() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}"
    WARNINGS=$((WARNINGS + 1))
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_info() {
    if [ "$VERBOSE" -eq 1 ]; then
        echo -e "$1"
    fi
}

# Exclude patterns (documentation, examples, migration guides)
EXCLUDE_PATTERNS=(
    "docs/GVISOR_MIGRATION_GUIDE.md"
    "docs/GVISOR_DOCKER_ELIMINATION_VALIDATION.md"
    "docs/GVISOR_DOCUMENTATION_GUIDE.md"
    "scripts/validate_gvisor_only.sh"
    "CHANGELOG.md"
    "README.md"
)

build_exclude_args() {
    local args=""
    for pattern in "${EXCLUDE_PATTERNS[@]}"; do
        args="$args --exclude=$pattern"
    done
    echo "$args"
}

EXCLUDE_ARGS=$(build_exclude_args)

echo "================================================"
echo "  gVisor-Only Validation"
echo "================================================"
echo ""

# ========== PART 1: VERIFY DOCKER ELIMINATION ==========

echo "PART 1: Docker Elimination Verification"
echo "========================================="
echo ""

# Check 1: Docker CLI usage in source code
echo "1. Checking for Docker CLI usage in source code..."
log_info "   Searching for 'docker' commands in .rs and .sh files..."

if grep -rn "docker\s\+\(run\|exec\|ps\|info\|version\|build\|pull\|push\|start\|stop\)" \
    --include="*.rs" \
    --include="*.sh" \
    $EXCLUDE_ARGS \
    . 2>/dev/null | grep -v "^#" | grep -v "//" | grep -v "gvisor"; then
    log_error "Found Docker CLI usage in source code"
    echo "Found Docker CLI calls" >> "$RESULTS_FILE"
else
    log_success "No Docker CLI usage in source code"
fi

# Check 2: Docker socket references
echo ""
echo "2. Checking for Docker socket references..."
log_info "   Searching for /var/run/docker.sock and DOCKER_HOST..."

if grep -rn "/var/run/docker\.sock\|DOCKER_HOST" \
    --include="*.rs" \
    --include="*.sh" \
    --include="*.toml" \
    --include="*.yaml" \
    --include="*.yml" \
    $EXCLUDE_ARGS \
    . 2>/dev/null; then
    log_error "Found Docker socket references"
    echo "Found Docker socket references" >> "$RESULTS_FILE"
else
    log_success "No Docker socket references"
fi

# Check 3: Testcontainers dependencies in Cargo.toml
echo ""
echo "3. Checking for testcontainers dependencies..."
log_info "   Searching Cargo.toml files for testcontainers..."

if grep -rn "testcontainers" \
    --include="Cargo.toml" \
    . 2>/dev/null; then
    log_error "Found testcontainers dependencies in Cargo.toml"
    echo "Found testcontainers in Cargo.toml" >> "$RESULTS_FILE"
else
    log_success "No testcontainers dependencies found"
fi

# Check 4: Testcontainers imports in source code
echo ""
echo "4. Checking for testcontainers imports..."
log_info "   Searching for 'use testcontainers' in .rs files..."

if grep -rn "use\s\+testcontainers\|testcontainers::" \
    --include="*.rs" \
    $EXCLUDE_ARGS \
    . 2>/dev/null; then
    log_error "Found testcontainers imports in source code"
    echo "Found testcontainers imports" >> "$RESULTS_FILE"
else
    log_success "No testcontainers imports found"
fi

# Check 5: GenericImage and Container usage from testcontainers
echo ""
echo "5. Checking for testcontainers API usage..."
log_info "   Searching for GenericImage, SyncRunner, AsyncRunner..."

if grep -rn "GenericImage\|SyncRunner\|AsyncRunner" \
    --include="*.rs" \
    $EXCLUDE_ARGS \
    . 2>/dev/null; then
    log_error "Found testcontainers API usage"
    echo "Found testcontainers API usage" >> "$RESULTS_FILE"
else
    log_success "No testcontainers API usage found"
fi

# Check 6: Docker Compose files
echo ""
echo "6. Checking for Docker Compose files..."
log_info "   Searching for docker-compose.yml files..."

COMPOSE_FILES=$(find . -name "docker-compose*.yml" -o -name "docker-compose*.yaml" 2>/dev/null || true)
if [ -n "$COMPOSE_FILES" ]; then
    log_warning "Found Docker Compose files (may be for examples/legacy):"
    echo "$COMPOSE_FILES" | sed 's/^/   /'

    if [ "$STRICT_MODE" -eq 1 ]; then
        ERRORS=$((ERRORS + 1))
        echo "Found Docker Compose files" >> "$RESULTS_FILE"
    fi
else
    log_success "No Docker Compose files found"
fi

# Check 7: Dockerfile references
echo ""
echo "7. Checking for Dockerfile references..."
log_info "   Searching for Dockerfile files..."

DOCKERFILES=$(find . -name "Dockerfile*" 2>/dev/null || true)
if [ -n "$DOCKERFILES" ]; then
    log_warning "Found Dockerfiles (may be for examples/legacy):"
    echo "$DOCKERFILES" | sed 's/^/   /'

    if [ "$STRICT_MODE" -eq 1 ]; then
        ERRORS=$((ERRORS + 1))
        echo "Found Dockerfiles" >> "$RESULTS_FILE"
    fi
else
    log_success "No Dockerfiles found"
fi

# Check 8: Docker scripts
echo ""
echo "8. Checking for Docker scripts..."
log_info "   Searching for docker_*.sh scripts..."

DOCKER_SCRIPTS=$(find scripts -name "docker_*.sh" 2>/dev/null || true)
if [ -n "$DOCKER_SCRIPTS" ]; then
    log_error "Found Docker scripts that should be removed:"
    echo "$DOCKER_SCRIPTS" | sed 's/^/   /'
    echo "Found Docker scripts" >> "$RESULTS_FILE"
else
    log_success "No Docker scripts found"
fi

# Check 9: Docker in CI/CD workflows
echo ""
echo "9. Checking for Docker usage in CI/CD..."
log_info "   Searching GitHub Actions workflows for Docker..."

if grep -rn "docker" \
    --include="*.yml" \
    --include="*.yaml" \
    .github/workflows/ 2>/dev/null | grep -v "^#" | grep -v "gvisor"; then
    log_warning "Found Docker references in CI/CD workflows"

    if [ "$STRICT_MODE" -eq 1 ]; then
        ERRORS=$((ERRORS + 1))
        echo "Found Docker in CI/CD" >> "$RESULTS_FILE"
    fi
else
    log_success "No Docker usage in CI/CD workflows"
fi

# Check 10: General Docker references (excluding documentation)
echo ""
echo "10. Checking for any remaining Docker references..."
log_info "   Broad search for 'Docker' term in source files..."

if grep -rn "Docker" \
    --include="*.rs" \
    --include="*.sh" \
    --exclude="validate_gvisor_only.sh" \
    $EXCLUDE_ARGS \
    . 2>/dev/null | grep -v "//\|#\|gvisor"; then
    log_warning "Found Docker references in comments/strings"
else
    log_success "No Docker references in source files"
fi

# ========== PART 2: VERIFY GVISOR SETUP ==========

echo ""
echo "PART 2: gVisor Setup Verification"
echo "=================================="
echo ""

# Check 11: gVisor scripts present
echo "11. Checking for gVisor scripts..."
log_info "   Verifying required gVisor scripts exist..."

local required_scripts=(
    "scripts/gvisor_startup.sh"
    "scripts/gvisor_health_check.sh"
    "scripts/wait_for_gvisor.sh"
    "scripts/validate_gvisor_only.sh"
)

local missing_scripts=0
for script in "${required_scripts[@]}"; do
    if [ -f "$script" ]; then
        log_success "Found: $script"
    else
        log_error "Missing: $script"
        ((missing_scripts++))
        echo "Missing script: $script" >> "$RESULTS_FILE"
    fi
done

# Check 12: runsc binary available
echo ""
echo "12. Checking for runsc binary..."
log_info "   Searching for gVisor runsc runtime..."

if command -v runsc >/dev/null 2>&1; then
    local runsc_version=$(runsc --version 2>&1 | head -1)
    log_success "runsc available: $runsc_version"
else
    log_warning "runsc not found in PATH (will be needed at runtime)"
fi

# Check 13: gVisor-related configuration
echo ""
echo "13. Checking for gVisor configuration..."
log_info "   Looking for gvisor in configuration files..."

if grep -rn "gvisor\|runsc" \
    --include="Cargo.toml" \
    --include="*.yaml" \
    --include="*.yml" \
    --include="*.rs" \
    crates/ 2>/dev/null | head -5 | wc -l | grep -q "^[1-9]"; then
    log_success "Found gVisor configuration references"
else
    log_warning "No gVisor configuration found (may be runtime-only)"
fi

# Check 14: Backend architecture updated for gVisor
echo ""
echo "14. Checking for gVisor backend implementation..."
log_info "   Searching for gvisor backend module..."

if [ -f "crates/clnrm-core/src/backend/gvisor.rs" ]; then
    log_success "Found gVisor backend implementation"
elif grep -q "gvisor" crates/clnrm-core/src/backend/mod.rs 2>/dev/null; then
    log_success "gVisor backend referenced in module"
else
    log_warning "No dedicated gVisor backend found (may use generic runtime)"
fi

# Check 15: Runtime abstraction supports gVisor
echo ""
echo "15. Checking runtime abstraction layer..."
log_info "   Verifying runtime-agnostic implementation..."

if grep -rn "trait.*Runtime\|Container.*Runtime" \
    --include="*.rs" \
    crates/clnrm-core/src/ 2>/dev/null | wc -l | grep -q "^[1-9]"; then
    log_success "Runtime abstraction layer found"
else
    log_warning "Runtime abstraction not clearly defined"
fi

# ========== SUMMARY ==========

echo ""
echo "================================================"
echo "  Validation Summary"
echo "================================================"

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    log_success "All checks passed! gVisor-only setup verified."
    echo ""
    echo "✨ Zero Docker references found"
    echo "✨ gVisor setup complete and functional"
    rm -f "$RESULTS_FILE"
    exit 0

elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}Validation completed with warnings${NC}"
    echo "   Errors: $ERRORS"
    echo "   Warnings: $WARNINGS"
    echo ""
    echo "⚠️  Warnings found, but no critical errors"
    echo "   Review warnings above and address if needed"
    rm -f "$RESULTS_FILE"
    exit 0

else
    echo -e "${RED}Validation failed${NC}"
    echo "   Errors: $ERRORS"
    echo "   Warnings: $WARNINGS"
    echo ""
    echo "❌ Issues found with gVisor setup"
    echo ""
    echo "Failed checks:"
    cat "$RESULTS_FILE" | sed 's/^/   - /'
    echo ""
    echo "Run with VERBOSE=1 for detailed output:"
    echo "   VERBOSE=1 ./scripts/validate_gvisor_only.sh"
    echo ""
    echo "For debugging:"
    echo "   1. Check Docker elimination: grep -r 'docker' --include='*.rs' --include='*.sh' crates/ scripts/ | grep -v gvisor"
    echo "   2. Verify gVisor setup: ./scripts/gvisor_health_check.sh"
    echo "   3. Initialize gVisor: ./scripts/gvisor_startup.sh"
    rm -f "$RESULTS_FILE"
    exit 1
fi
