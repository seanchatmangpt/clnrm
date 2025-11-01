#!/bin/bash
# Error Handling Validation Script - Agent 4
# Validates that all production code is free of unwrap/expect

set -e

echo "============================================"
echo "Error Handling Validation - Agent 4"
echo "============================================"
echo

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED=0

echo "Phase 1: Production Code Scan"
echo "---------------------------------------------"

# Scan production code excluding tests and safe patterns
echo -n "Scanning clnrm-core/src for unwrap/expect... "
PRODUCTION_ISSUES=$(find crates/clnrm-core/src -name "*.rs" -type f \
    -not -path "*/tests/*" \
    -not -name "*test*.rs" \
    -exec grep -H -n "\.unwrap()\|\.expect(" {} \; 2>/dev/null | \
    grep -v "unwrap_or" | \
    grep -v "#\[tokio::test\]" | \
    grep -v "#\[cfg(test)\]" | \
    wc -l | tr -d ' ')

if [ "$PRODUCTION_ISSUES" = "0" ]; then
    echo -e "${GREEN}CLEAN ✅${NC}"
else
    echo -e "${RED}FOUND $PRODUCTION_ISSUES issues ❌${NC}"
    FAILED=1
fi

echo
echo "Phase 2: Critical Files Verification"
echo "---------------------------------------------"

# Check specific critical files
CRITICAL_FILES=(
    "crates/clnrm-core/src/determinism/ports.rs"
    "crates/clnrm-core/src/backend/pool.rs"
)

for file in "${CRITICAL_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -n "$(basename $file): "
        ISSUES=$(grep -n "\.unwrap()\|\.expect(" "$file" 2>/dev/null | \
            grep -v unwrap_or | \
            grep -v "#\[tokio::test\]" | \
            grep -v "#\[cfg(test)\]" | \
            wc -l | tr -d ' ')

        if [ "$ISSUES" = "0" ]; then
            echo -e "${GREEN}CLEAN ✅${NC}"
        else
            echo -e "${RED}$ISSUES issues ❌${NC}"
            grep -n "\.unwrap()\|\.expect(" "$file" | grep -v unwrap_or | head -5
            FAILED=1
        fi
    else
        echo -e "$(basename $file): ${RED}NOT FOUND ❌${NC}"
        FAILED=1
    fi
done

echo
echo "Phase 3: Typestate Pattern Files (Informational)"
echo "---------------------------------------------"

# orchestrator.rs has typestate expects (type system guarantees)
echo -n "orchestrator.rs (typestate): "
TYPESTATE_COUNT=$(grep -c "\.expect(" crates/clnrm-core/src/telemetry/live_check/orchestrator.rs 2>/dev/null || echo "0")
if [ "$TYPESTATE_COUNT" -gt "0" ]; then
    echo -e "${YELLOW}$TYPESTATE_COUNT typestate expects (safe) ⚠️${NC}"
else
    echo -e "${GREEN}CLEAN ✅${NC}"
fi

echo
echo "Phase 4: Build & Test Validation"
echo "---------------------------------------------"

# Try to build clnrm-core
echo -n "Building clnrm-core... "
if cargo build --lib -p clnrm-core --quiet 2>/dev/null; then
    echo -e "${GREEN}SUCCESS ✅${NC}"
else
    echo -e "${RED}FAILED ❌${NC}"
    echo "  (Run 'cargo build --lib -p clnrm-core' for details)"
    FAILED=1
fi

# Try to run tests
echo -n "Testing clnrm-core... "
if cargo test --lib -p clnrm-core --quiet 2>/dev/null; then
    echo -e "${GREEN}SUCCESS ✅${NC}"
else
    echo -e "${RED}FAILED ❌${NC}"
    echo "  (Run 'cargo test --lib -p clnrm-core' for details)"
    FAILED=1
fi

# Try clippy
echo -n "Running clippy... "
if cargo clippy --lib -p clnrm-core --all-features --quiet -- -D warnings 2>/dev/null; then
    echo -e "${GREEN}SUCCESS ✅${NC}"
else
    echo -e "${RED}FAILED ❌${NC}"
    echo "  (Run 'cargo clippy --lib -p clnrm-core --all-features -- -D warnings' for details)"
    FAILED=1
fi

echo
echo "============================================"
echo "Validation Summary"
echo "============================================"
echo

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ ALL CHECKS PASSED${NC}"
    echo
    echo "Production code is panic-safe and ready for v1.4.1 release."
    echo
    exit 0
else
    echo -e "${RED}❌ VALIDATION FAILED${NC}"
    echo
    echo "Issues found. Review output above for details."
    echo "See docs/AGENT_4_ERROR_HANDLING_VALIDATION_REPORT.md for full analysis."
    echo
    exit 1
fi
