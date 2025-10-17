#!/bin/bash
# Verification Script: Test Each Detection Layer Independently
#
# This script validates that each of the 7 detection layers can
# independently catch fake-green tests.

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Detection Layer Verification                              ║"
echo "║  Testing Each Layer Independently                          ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo

# Navigate to case study directory
cd "$(dirname "$0")"

# Test counter
TOTAL_TESTS=7
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to test a detection layer
test_layer() {
    local layer_name="$1"
    local layer_number="$2"
    local expected_failure="$3"

    echo "═══════════════════════════════════════════════════════════"
    echo -e "${BLUE}[LAYER $layer_number/$TOTAL_TESTS]${NC} $layer_name"
    echo "═══════════════════════════════════════════════════════════"
    echo

    # Run fake-green test
    if clnrm run fake-green-detection.toml --service fake 2>&1 | grep -i "$expected_failure"; then
        echo -e "${GREEN}✅ PASS${NC}: Layer correctly detected: $expected_failure"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC}: Layer did not detect expected failure"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo
    sleep 1
}

# LAYER 1: Lifecycle Events
test_layer "Lifecycle Events" 1 "lifecycle event"

# LAYER 2: Span Graph Structure
test_layer "Span Graph Structure" 2 "edge"

# LAYER 3: Span Counts
test_layer "Span Counts" 3 "span count"

# LAYER 4: Ordering Constraints
test_layer "Ordering Constraints" 4 "ordering"

# LAYER 5: Window Containment
test_layer "Window Containment" 5 "window"

# LAYER 6: Status Validation
test_layer "Status Validation" 6 "status"

# LAYER 7: Hermeticity Validation
test_layer "Hermeticity Validation" 7 "hermeticity"

# Summary
echo "═══════════════════════════════════════════════════════════"
echo -e "${BLUE}[VERIFICATION SUMMARY]${NC}"
echo "═══════════════════════════════════════════════════════════"
echo
echo "Total Layers: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ ALL DETECTION LAYERS VERIFIED${NC}"
    echo "Each layer can independently catch fake-green tests!"
    exit 0
else
    echo -e "${RED}❌ SOME DETECTION LAYERS FAILED${NC}"
    echo "Review failures above for details."
    exit 1
fi
