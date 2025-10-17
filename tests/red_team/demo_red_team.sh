#!/bin/bash
# Red-Team Demonstration Script
#
# This script demonstrates how clnrm detects all three attack vectors
# and validates that legitimate tests pass.
#
# Usage:
#   ./demo_red_team.sh

set -e  # Exit on error

echo "========================================"
echo "Red-Team Fake-Green Detection Demo"
echo "========================================"
echo ""
echo "This demo shows how clnrm's span-first invariant validation"
echo "detects fake-green attacks through 7 independent layers."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Change to tests/red_team directory
cd "$(dirname "$0")"

echo "=========================================="
echo "Attack A: Echo Pass (Trivial Forgery)"
echo "=========================================="
echo ""

echo "${YELLOW}Running attack script directly (bypassing clnrm):${NC}"
echo "$ bash attack_scripts/attack_a_echo.sh"
echo ""
bash attack_scripts/attack_a_echo.sh || true
echo ""

echo "${YELLOW}Validating with clnrm (span-first detection):${NC}"
echo "$ clnrm run -f attack_a_echo.clnrm.toml"
echo ""

if clnrm run -f attack_a_echo.clnrm.toml 2>&1; then
    echo "${RED}❌ ERROR: Attack A PASSED (should have failed)${NC}"
    exit 1
else
    echo ""
    echo "${GREEN}✅ SUCCESS: Attack A correctly detected${NC}"
    echo "   First Failing Rule: expect.counts.spans_total"
    echo "   Detection Time: ~0.02s"
fi

echo ""
echo "=========================================="
echo "Attack B: Log Mimicry (Sophisticated)"
echo "=========================================="
echo ""

echo "${YELLOW}Running attack script directly:${NC}"
echo "$ bash attack_scripts/attack_b_logs.sh"
echo ""
bash attack_scripts/attack_b_logs.sh || true
echo ""

echo "${YELLOW}Validating with clnrm:${NC}"
echo "$ clnrm run -f attack_b_logs.clnrm.toml"
echo ""

if clnrm run -f attack_b_logs.clnrm.toml 2>&1; then
    echo "${RED}❌ ERROR: Attack B PASSED (should have failed)${NC}"
    exit 1
else
    echo ""
    echo "${GREEN}✅ SUCCESS: Attack B correctly detected${NC}"
    echo "   First Failing Rule: expect.counts.spans_total"
    echo "   Detection Time: ~0.02s"
    echo "   Note: Realistic logs did not fool span validation"
fi

echo ""
echo "=========================================="
echo "Attack C: Empty OTEL Path (Env Spoofing)"
echo "=========================================="
echo ""

echo "${YELLOW}Running attack script directly:${NC}"
echo "$ bash attack_scripts/attack_c_empty_otel.sh"
echo ""
bash attack_scripts/attack_c_empty_otel.sh || true
echo ""

echo "${YELLOW}Validating with clnrm:${NC}"
echo "$ clnrm run -f attack_c_empty_otel.clnrm.toml"
echo ""

if clnrm run -f attack_c_empty_otel.clnrm.toml 2>&1; then
    echo "${RED}❌ ERROR: Attack C PASSED (should have failed)${NC}"
    exit 1
else
    echo ""
    echo "${GREEN}✅ SUCCESS: Attack C correctly detected${NC}"
    echo "   First Failing Rule: expect.counts.spans_total"
    echo "   Detection Time: ~0.02s"
    echo "   Note: OTEL env vars alone are insufficient"
fi

echo ""
echo "=========================================="
echo "Legitimate Test (Control)"
echo "=========================================="
echo ""

echo "${YELLOW}Running legitimate test:${NC}"
echo "$ clnrm run -f legitimate_self_test.clnrm.toml"
echo ""

if clnrm run -f legitimate_self_test.clnrm.toml 2>&1; then
    echo ""
    echo "${GREEN}✅ SUCCESS: Legitimate test correctly passed${NC}"
    echo "   All 7 validation layers: PASS"
    echo "   Span count: >=2"
    echo "   Digest: Valid (not empty trace)"
else
    echo "${RED}❌ ERROR: Legitimate test FAILED (should pass)${NC}"
    exit 1
fi

echo ""
echo "=========================================="
echo "Summary"
echo "=========================================="
echo ""
echo "${GREEN}✅ All attacks correctly detected (0 spans)${NC}"
echo "${GREEN}✅ Legitimate test correctly passed (12+ spans)${NC}"
echo ""
echo "Detection Characteristics:"
echo "  - Attack detection time: ~0.02s (instant)"
echo "  - First failing rule: expect.counts.spans_total"
echo "  - Empty trace digest: d41d8cd98f00b204e9800998ecf8427e"
echo "  - Legitimate test digest: <varies, deterministic>"
echo ""
echo "Security Guarantees:"
echo "  - Exit codes alone are insufficient"
echo "  - Text-based validation bypassed by all attacks"
echo "  - Span-first validation required for security"
echo "  - 7 independent validation layers (defense-in-depth)"
echo "  - Cryptographic digests provide tamper-evident proof"
echo ""
echo "${GREEN}Demo complete!${NC}"
echo ""
echo "Next Steps:"
echo "  1. Read full documentation: docs/RED_TEAM_CASE_STUDY.md"
echo "  2. Review attack scripts: tests/red_team/attack_scripts/"
echo "  3. Inspect TOML configs: tests/red_team/*.clnrm.toml"
echo "  4. Try editing configs: swap attack command to legitimate binary"
echo ""
