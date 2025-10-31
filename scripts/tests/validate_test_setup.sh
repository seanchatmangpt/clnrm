#!/bin/bash
# validate_test_setup.sh
# Quick validation that test suite is properly configured

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Validating Live-Check Test Suite Setup...${NC}"
echo ""

ERRORS=0
WARNINGS=0

# Check 1: Weaver installed
echo -n "Checking weaver installation... "
if command -v weaver &> /dev/null; then
    VERSION=$(weaver --version 2>&1 || echo "unknown")
    echo -e "${GREEN}✓${NC} Found: $VERSION"
else
    echo -e "${RED}✗${NC} weaver not found"
    echo "  Install: https://github.com/open-telemetry/weaver"
    ERRORS=$((ERRORS + 1))
fi

# Check 2: Registry exists
echo -n "Checking registry directory... "
if [ -d "$SCRIPT_DIR/../../registry" ]; then
    SCHEMA_COUNT=$(find "$SCRIPT_DIR/../../registry" -name "*.yaml" -o -name "*.yml" | wc -l)
    echo -e "${GREEN}✓${NC} Found ($SCHEMA_COUNT schemas)"
else
    echo -e "${RED}✗${NC} Not found"
    ERRORS=$((ERRORS + 1))
fi

# Check 3: Output directory writable
echo -n "Checking output directory... "
OUTPUT_DIR="$SCRIPT_DIR/../../validation_output/live_check_tests"
if mkdir -p "$OUTPUT_DIR" 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Writable: $OUTPUT_DIR"
    rmdir "$OUTPUT_DIR" 2>/dev/null || true
else
    echo -e "${RED}✗${NC} Not writable"
    ERRORS=$((ERRORS + 1))
fi

# Check 4: Test scripts executable
echo -n "Checking test scripts... "
if [ -x "$SCRIPT_DIR/test_live_check_comprehensive.sh" ] && \
   [ -x "$SCRIPT_DIR/run_test_subset.sh" ]; then
    echo -e "${GREEN}✓${NC} Executable"
else
    echo -e "${YELLOW}⚠${NC} Not executable, fixing..."
    chmod +x "$SCRIPT_DIR/test_live_check_comprehensive.sh"
    chmod +x "$SCRIPT_DIR/run_test_subset.sh"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 5: jq available (for JSON parsing)
echo -n "Checking jq (JSON processor)... "
if command -v jq &> /dev/null; then
    echo -e "${GREEN}✓${NC} Available"
else
    echo -e "${YELLOW}⚠${NC} Not found (optional, but recommended)"
    echo "  Install: brew install jq"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 6: Cargo available (for OTLP test)
echo -n "Checking cargo (for OTLP test)... "
if command -v cargo &> /dev/null; then
    VERSION=$(cargo --version | cut -d' ' -f2)
    echo -e "${GREEN}✓${NC} Found: $VERSION"
else
    echo -e "${YELLOW}⚠${NC} Not found (OTLP test will be skipped)"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 7: Port availability for tests
echo -n "Checking test ports (4320-4322, 4330-4332)... "
PORTS_IN_USE=()
for port in 4320 4321 4322 4330 4331 4332; do
    if lsof -i ":$port" &> /dev/null; then
        PORTS_IN_USE+=($port)
    fi
done

if [ ${#PORTS_IN_USE[@]} -eq 0 ]; then
    echo -e "${GREEN}✓${NC} All ports available"
else
    echo -e "${YELLOW}⚠${NC} Some ports in use: ${PORTS_IN_USE[*]}"
    echo "  Tests using these ports may fail"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 8: Validate registry with weaver
if command -v weaver &> /dev/null && [ -d "$SCRIPT_DIR/../../registry" ]; then
    echo -n "Validating registry schemas... "
    if weaver registry check -r "$SCRIPT_DIR/../../registry" &> /dev/null; then
        echo -e "${GREEN}✓${NC} Valid"
    else
        echo -e "${RED}✗${NC} Invalid schemas"
        echo "  Run: weaver registry check -r registry/"
        ERRORS=$((ERRORS + 1))
    fi
fi

echo ""
echo "========================================"
echo "Validation Summary"
echo "========================================"

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✓ Perfect setup! Ready to run tests.${NC}"
    echo ""
    echo "Quick start:"
    echo "  ./scripts/tests/test_live_check_comprehensive.sh"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠ Setup OK with $WARNINGS warning(s)${NC}"
    echo "  Tests will run, but some may be skipped."
    echo ""
    echo "Quick start:"
    echo "  ./scripts/tests/test_live_check_comprehensive.sh"
    exit 0
else
    echo -e "${RED}✗ Setup incomplete: $ERRORS error(s), $WARNINGS warning(s)${NC}"
    echo "  Fix errors before running tests."
    exit 1
fi
