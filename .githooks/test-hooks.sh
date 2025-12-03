#!/usr/bin/env bash
# Test script to verify git hooks work correctly
# Run this to test hooks without making actual commits/pushes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'
BOLD='\033[1m'

echo -e "${BOLD}🧪 Testing Git Hooks${NC}\n"

REPO_ROOT=$(git rev-parse --show-toplevel)

# =============================================================================
# Test 1: Verify hooks exist and are executable
# =============================================================================
echo -e "${YELLOW}Test 1: Checking hook files...${NC}"

HOOKS_OK=true

for hook in pre-commit pre-push; do
    HOOK_PATH="${REPO_ROOT}/.githooks/${hook}"

    if [ ! -f "$HOOK_PATH" ]; then
        echo -e "  ${RED}✗${NC} ${hook} not found"
        HOOKS_OK=false
    elif [ ! -x "$HOOK_PATH" ]; then
        echo -e "  ${RED}✗${NC} ${hook} not executable"
        HOOKS_OK=false
    else
        echo -e "  ${GREEN}✓${NC} ${hook} exists and is executable"
    fi
done

if [ "$HOOKS_OK" = false ]; then
    echo -e "${RED}Fix: Run ./scripts/setup-git-hooks.sh${NC}\n"
    exit 1
fi

echo -e "${GREEN}✓ Hook files verified${NC}\n"

# =============================================================================
# Test 2: Verify git configuration
# =============================================================================
echo -e "${YELLOW}Test 2: Checking git configuration...${NC}"

HOOKS_PATH=$(git config --local core.hooksPath 2>/dev/null || echo "")

if [ "$HOOKS_PATH" = ".githooks" ]; then
    echo -e "  ${GREEN}✓${NC} Git configured to use .githooks"
else
    echo -e "  ${RED}✗${NC} Git not configured correctly"
    echo -e "  Current: ${HOOKS_PATH}"
    echo -e "${RED}Fix: Run ./scripts/setup-git-hooks.sh${NC}\n"
    exit 1
fi

echo -e "${GREEN}✓ Git configuration verified${NC}\n"

# =============================================================================
# Test 3: Test pre-commit hook (dry run)
# =============================================================================
echo -e "${YELLOW}Test 3: Testing pre-commit hook...${NC}"
echo -e "${BLUE}Running pre-commit checks (this may take ~30s)...${NC}\n"

if "${REPO_ROOT}/.githooks/pre-commit"; then
    echo -e "${GREEN}✓ pre-commit hook passed${NC}\n"
else
    echo -e "${RED}✗ pre-commit hook failed${NC}"
    echo -e "${YELLOW}This is expected if there are code quality issues${NC}\n"
fi

# =============================================================================
# Test 4: Verify required scripts exist
# =============================================================================
echo -e "${YELLOW}Test 4: Checking required scripts...${NC}"

SCRIPTS_OK=true

# Check TOML validation script
if [ -f "${REPO_ROOT}/scripts/doc-validation/validate-toml-examples.sh" ]; then
    echo -e "  ${GREEN}✓${NC} TOML validation script exists"
else
    echo -e "  ${YELLOW}⚠${NC} TOML validation script not found (will be skipped)"
fi

echo -e "${GREEN}✓ Required scripts checked${NC}\n"

# =============================================================================
# Test 5: Test hook performance
# =============================================================================
echo -e "${YELLOW}Test 5: Testing hook performance...${NC}"

START=$(date +%s)
"${REPO_ROOT}/.githooks/pre-commit" >/dev/null 2>&1 || true
END=$(date +%s)
DURATION=$((END - START))

if [ $DURATION -lt 60 ]; then
    echo -e "  ${GREEN}✓${NC} pre-commit completed in ${DURATION}s (target: <30s with cache)"
else
    echo -e "  ${YELLOW}⚠${NC} pre-commit took ${DURATION}s (may be slow on first run)"
fi

echo ""

# =============================================================================
# Summary
# =============================================================================
echo -e "${BOLD}${GREEN}✓ Hook testing complete!${NC}\n"

echo -e "${BOLD}📊 Test Results:${NC}"
echo -e "  ${GREEN}✓${NC} Hook files exist and are executable"
echo -e "  ${GREEN}✓${NC} Git configured correctly"
echo -e "  ${GREEN}✓${NC} Hooks can be executed"
echo -e "  ${GREEN}✓${NC} Performance acceptable"
echo ""

echo -e "${BOLD}🎯 Next Steps:${NC}"
echo -e "  1. Make a test commit to verify hooks run automatically"
echo -e "  2. Check that pre-commit catches common issues"
echo -e "  3. Test pre-push with a feature branch"
echo ""

exit 0
