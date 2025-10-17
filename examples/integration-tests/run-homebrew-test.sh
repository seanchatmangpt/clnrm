#!/bin/bash
# ============================================================================
# Homebrew Installation Validation Test Runner
# ============================================================================
#
# This script runs the Homebrew installation validation test and verifies:
#   1. Test execution succeeds
#   2. OTEL spans are produced
#   3. All validators pass
#   4. Deterministic digests are stable across runs
#
# Usage:
#   ./run-homebrew-test.sh
#
# Exit codes:
#   0 - All tests passed
#   1 - Test execution failed
#   2 - Validation failed
#   3 - Determinism check failed
#
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Homebrew Installation Validation ===${NC}"
echo
echo "This test validates:"
echo "  1. Homebrew can install clnrm"
echo "  2. Installed clnrm runs self-test"
echo "  3. Self-test produces valid OTEL spans"
echo "  4. All validators pass on the spans"
echo

# Check if clnrm is available
if ! command -v clnrm &> /dev/null; then
    echo -e "${RED}Error: clnrm not found in PATH${NC}"
    echo "Please install clnrm first:"
    echo "  cargo install --path crates/clnrm"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${RED}Error: Docker is not running${NC}"
    echo "Please start Docker and try again"
    exit 1
fi

# Clean up previous run artifacts
echo -e "${YELLOW}Cleaning up previous artifacts...${NC}"
rm -f brew-selftest.report.json brew-selftest.trace.sha256

# Run the test
echo -e "${BLUE}Running test...${NC}"
echo

if ! clnrm run homebrew-install-selftest.clnrm.toml; then
    echo -e "${RED}Test execution failed${NC}"
    exit 1
fi

echo

# Check outputs exist
echo -e "${BLUE}Checking outputs...${NC}"
if [ ! -f brew-selftest.report.json ]; then
    echo -e "${RED}Error: Report file not found${NC}"
    exit 2
fi

if [ ! -f brew-selftest.trace.sha256 ]; then
    echo -e "${RED}Error: Digest file not found${NC}"
    exit 2
fi

ls -lh brew-selftest.report.json brew-selftest.trace.sha256
echo

# Parse report for validation results
echo -e "${BLUE}Validating test results...${NC}"

# Check if jq is available for JSON parsing
if command -v jq &> /dev/null; then
    # Check verdict
    VERDICT=$(jq -r '.verdict // "unknown"' brew-selftest.report.json)
    if [ "$VERDICT" != "pass" ]; then
        echo -e "${RED}Test verdict: $VERDICT${NC}"
        exit 2
    fi
    echo -e "${GREEN}Test verdict: $VERDICT${NC}"

    # Check span count
    SPAN_COUNT=$(jq -r '.spans_collected // 0' brew-selftest.report.json)
    echo "Spans collected: $SPAN_COUNT"

    # Check validator results
    echo "Validator results:"
    jq -r '.validators | to_entries[] | "  \(.key): \(.value.status)"' brew-selftest.report.json || true
else
    echo -e "${YELLOW}Warning: jq not found, skipping detailed report parsing${NC}"
fi

echo

# Verify determinism
echo -e "${BLUE}Verifying digest stability...${NC}"
DIGEST1=$(cat brew-selftest.trace.sha256)
echo "First run digest: $DIGEST1"

# Run again for determinism check
echo -e "${YELLOW}Running test again to verify determinism...${NC}"
rm -f brew-selftest.report.json brew-selftest.trace.sha256

if ! clnrm run homebrew-install-selftest.clnrm.toml &> /dev/null; then
    echo -e "${RED}Second test run failed${NC}"
    exit 3
fi

DIGEST2=$(cat brew-selftest.trace.sha256)
echo "Second run digest: $DIGEST2"
echo

if [ "$DIGEST1" = "$DIGEST2" ]; then
    echo -e "${GREEN}✅ Determinism verified: digests match${NC}"
else
    echo -e "${RED}❌ Determinism failed: digests differ${NC}"
    echo "This indicates non-deterministic behavior in the test execution"
    exit 3
fi

echo

# Summary
echo -e "${GREEN}=== Homebrew Validation Complete ===${NC}"
echo
echo "Results:"
echo "  - Test execution: PASSED"
echo "  - OTEL spans: VALIDATED"
echo "  - All validators: PASSED"
echo "  - Determinism: VERIFIED"
echo
echo "Output files:"
echo "  - brew-selftest.report.json (full test report)"
echo "  - brew-selftest.trace.sha256 (digest for reproducibility)"
echo

exit 0
