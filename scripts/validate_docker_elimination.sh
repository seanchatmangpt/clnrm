#!/bin/bash
# Validates zero Docker references in codebase
# Exit code: 0 = success, 1 = Docker references found
#
# Usage:
#   ./scripts/validate_docker_elimination.sh
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
    "scripts/validate_docker_elimination.sh"
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
echo "  Docker Elimination Validation"
echo "================================================"
echo ""

# Check 1: Docker CLI usage in source code
echo "1. Checking for Docker CLI usage in source code..."
log_info "   Searching for 'docker' commands in .rs and .sh files..."

if grep -rn "docker\s\+\(run\|exec\|ps\|info\|version\|build\|pull\|push\|start\|stop\)" \
    --include="*.rs" \
    --include="*.sh" \
    $EXCLUDE_ARGS \
    . 2>/dev/null | grep -v "^#" | grep -v "//"; then
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

# Check 6: TestcontainerBackend usage
echo ""
echo "6. Checking for TestcontainerBackend references..."
log_info "   Searching for TestcontainerBackend in backend modules..."

# Allow TestcontainerBackend in deprecated/removed files
TESTCONTAINER_BACKEND_FILES=$(grep -rn "TestcontainerBackend" \
    --include="*.rs" \
    $EXCLUDE_ARGS \
    crates/clnrm-core/src/backend/ 2>/dev/null || true)

if [ -n "$TESTCONTAINER_BACKEND_FILES" ]; then
    # Check if files are marked as deprecated
    if echo "$TESTCONTAINER_BACKEND_FILES" | grep -q "deprecated\|removed"; then
        log_warning "Found TestcontainerBackend in deprecated files (acceptable)"
    else
        log_error "Found active TestcontainerBackend usage"
        echo "Found TestcontainerBackend usage" >> "$RESULTS_FILE"
    fi
else
    log_success "No TestcontainerBackend references found"
fi

# Check 7: Docker Compose files
echo ""
echo "7. Checking for Docker Compose files..."
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

# Check 8: Dockerfile references
echo ""
echo "8. Checking for Dockerfile references..."
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

# Check 9: Docker daemon checks in source
echo ""
echo "9. Checking for Docker daemon availability checks..."
log_info "   Searching for docker info/version checks..."

if grep -rn "docker\s\+info\|docker\s\+--version\|verify_docker_available" \
    --include="*.rs" \
    --include="*.sh" \
    $EXCLUDE_ARGS \
    . 2>/dev/null; then
    log_error "Found Docker daemon availability checks"
    echo "Found Docker daemon checks" >> "$RESULTS_FILE"
else
    log_success "No Docker daemon checks found"
fi

# Check 10: Docker in CI/CD workflows
echo ""
echo "10. Checking for Docker usage in CI/CD..."
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

# Summary
echo ""
echo "================================================"
echo "  Validation Summary"
echo "================================================"

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    log_success "All checks passed! Docker completely eliminated."
    echo ""
    echo "✨ Zero Docker references found in codebase"
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
    echo "❌ Docker references still exist in codebase"
    echo ""
    echo "Failed checks:"
    cat "$RESULTS_FILE" | sed 's/^/   - /'
    echo ""
    echo "Run with VERBOSE=1 for detailed output:"
    echo "   VERBOSE=1 ./scripts/validate_docker_elimination.sh"
    rm -f "$RESULTS_FILE"
    exit 1
fi
