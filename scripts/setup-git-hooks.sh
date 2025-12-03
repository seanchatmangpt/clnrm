#!/usr/bin/env bash
# Setup script for git hooks
# Configures git to use .githooks/ directory and makes hooks executable

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color
BOLD='\033[1m'

echo -e "${BOLD}🔧 Setting up Git hooks for clnrm...${NC}\n"

# =============================================================================
# 1. Check if we're in a git repository
# =============================================================================
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}✗ Not in a git repository${NC}"
    echo -e "${YELLOW}Run this script from the clnrm repository root${NC}"
    exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel)
echo -e "${BLUE}Repository root: ${REPO_ROOT}${NC}\n"

# =============================================================================
# 2. Check if .githooks directory exists
# =============================================================================
if [ ! -d "${REPO_ROOT}/.githooks" ]; then
    echo -e "${RED}✗ .githooks directory not found${NC}"
    echo -e "${YELLOW}Expected location: ${REPO_ROOT}/.githooks${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Found .githooks directory${NC}\n"

# =============================================================================
# 3. Make hooks executable
# =============================================================================
echo -e "${YELLOW}🔑 Making hooks executable...${NC}"

HOOKS_MADE_EXECUTABLE=0

for hook in "${REPO_ROOT}/.githooks"/*; do
    if [ -f "$hook" ]; then
        HOOK_NAME=$(basename "$hook")

        # Check if already executable
        if [ -x "$hook" ]; then
            echo -e "  ${GREEN}✓${NC} ${HOOK_NAME} (already executable)"
        else
            chmod +x "$hook"
            echo -e "  ${GREEN}✓${NC} ${HOOK_NAME} (made executable)"
            HOOKS_MADE_EXECUTABLE=$((HOOKS_MADE_EXECUTABLE + 1))
        fi
    fi
done

if [ $HOOKS_MADE_EXECUTABLE -gt 0 ]; then
    echo -e "${GREEN}Made ${HOOKS_MADE_EXECUTABLE} hook(s) executable${NC}\n"
else
    echo -e "${GREEN}All hooks already executable${NC}\n"
fi

# =============================================================================
# 4. Configure git to use .githooks directory
# =============================================================================
echo -e "${YELLOW}⚙️  Configuring git hooks path...${NC}"

# Get current hooks path
CURRENT_HOOKS_PATH=$(git config --local core.hooksPath 2>/dev/null || echo "")

if [ "$CURRENT_HOOKS_PATH" = ".githooks" ]; then
    echo -e "${GREEN}✓ Git already configured to use .githooks${NC}\n"
else
    if [ -n "$CURRENT_HOOKS_PATH" ]; then
        echo -e "${YELLOW}Current hooks path: ${CURRENT_HOOKS_PATH}${NC}"
        echo -e "${BLUE}Changing to: .githooks${NC}"
    fi

    git config --local core.hooksPath .githooks
    echo -e "${GREEN}✓ Git configured to use .githooks${NC}\n"
fi

# =============================================================================
# 5. List available hooks
# =============================================================================
echo -e "${BOLD}📋 Available hooks:${NC}"

for hook in "${REPO_ROOT}/.githooks"/*; do
    if [ -f "$hook" ]; then
        HOOK_NAME=$(basename "$hook")
        HOOK_DESC=""

        # Extract description from hook file (first non-shebang comment)
        HOOK_DESC=$(grep -m 1 "^# " "$hook" | sed 's/^# //' || echo "")

        if [ -n "$HOOK_DESC" ]; then
            echo -e "  ${BLUE}${HOOK_NAME}${NC}: ${HOOK_DESC}"
        else
            echo -e "  ${BLUE}${HOOK_NAME}${NC}"
        fi
    fi
done

echo ""

# =============================================================================
# 6. Test hooks
# =============================================================================
echo -e "${YELLOW}🧪 Testing hook configuration...${NC}"

# Test if hooks are executable and git can find them
HOOK_TEST_PASSED=true

for hook in pre-commit pre-push; do
    HOOK_PATH="${REPO_ROOT}/.githooks/${hook}"

    if [ -f "$HOOK_PATH" ]; then
        if [ -x "$HOOK_PATH" ]; then
            echo -e "  ${GREEN}✓${NC} ${hook} is ready"
        else
            echo -e "  ${RED}✗${NC} ${hook} is not executable"
            HOOK_TEST_PASSED=false
        fi
    else
        echo -e "  ${YELLOW}⚠${NC} ${hook} not found"
    fi
done

echo ""

if [ "$HOOK_TEST_PASSED" = true ]; then
    echo -e "${GREEN}✓ All hooks configured correctly${NC}\n"
else
    echo -e "${RED}✗ Some hooks have issues${NC}\n"
    exit 1
fi

# =============================================================================
# 7. Provide usage information
# =============================================================================
echo -e "${BOLD}${GREEN}✓ Git hooks setup complete!${NC}\n"

echo -e "${BOLD}📚 Usage:${NC}"
echo -e "  ${BLUE}pre-commit${NC}  - Runs automatically before each commit (fast ~30s)"
echo -e "    → TOML validation, clippy, format check, common issues"
echo -e ""
echo -e "  ${BLUE}pre-push${NC}    - Runs automatically before each push (comprehensive)"
echo -e "    → Full test suite, Weaver validation, integration tests"
echo -e ""

echo -e "${BOLD}⚙️  Configuration:${NC}"
echo -e "  To skip hooks temporarily:"
echo -e "    ${BLUE}git commit --no-verify${NC}  (skip pre-commit)"
echo -e "    ${BLUE}git push --no-verify${NC}    (skip pre-push)"
echo -e ""

echo -e "  To disable hooks:"
echo -e "    ${BLUE}git config --local --unset core.hooksPath${NC}"
echo -e ""

echo -e "  To re-enable hooks:"
echo -e "    ${BLUE}./scripts/setup-git-hooks.sh${NC}"
echo -e ""

echo -e "${BOLD}🎯 Best Practices:${NC}"
echo -e "  • Let pre-commit run - it's fast and catches issues early"
echo -e "  • pre-push runs full validation - ensure you have time"
echo -e "  • Fix issues rather than bypassing with --no-verify"
echo -e "  • Run ${BLUE}cargo test${NC} locally before pushing"
echo -e ""

echo -e "${GREEN}Happy coding! 🚀${NC}\n"

exit 0
