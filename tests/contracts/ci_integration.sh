#!/bin/bash
# Contract Testing CI/CD Integration Script
# This script runs contract tests and validates schema compliance

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
CONTRACTS_DIR="tests/contracts"
SCHEMAS_DIR="$CONTRACTS_DIR/schemas"
RESULTS_DIR="target/contract-test-results"

echo -e "${GREEN}=== CLNRM Contract Testing CI/CD ===${NC}"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Step 1: Validate JSON schemas
echo -e "${YELLOW}Step 1: Validating JSON schemas...${NC}"
SCHEMA_COUNT=0
SCHEMA_ERRORS=0

for schema in "$SCHEMAS_DIR"/*.json; do
    if [ -f "$schema" ]; then
        SCHEMA_COUNT=$((SCHEMA_COUNT + 1))
        echo "  Validating: $(basename "$schema")"

        # Validate JSON syntax
        if jq empty "$schema" 2>/dev/null; then
            echo -e "    ${GREEN}✓ Valid JSON${NC}"
        else
            echo -e "    ${RED}✗ Invalid JSON${NC}"
            SCHEMA_ERRORS=$((SCHEMA_ERRORS + 1))
        fi
    fi
done

echo ""
if [ $SCHEMA_ERRORS -eq 0 ]; then
    echo -e "${GREEN}✓ All $SCHEMA_COUNT schemas are valid${NC}"
else
    echo -e "${RED}✗ $SCHEMA_ERRORS out of $SCHEMA_COUNT schemas are invalid${NC}"
    exit 1
fi

# Step 2: Run contract tests
echo ""
echo -e "${YELLOW}Step 2: Running contract tests...${NC}"

# Run API contract tests
echo "  Running API contract tests..."
cargo test --test '*' -- --nocapture api_contracts 2>&1 | tee "$RESULTS_DIR/api_contracts.log" || {
    echo -e "${RED}✗ API contract tests failed${NC}"
    exit 1
}

# Run service contract tests
echo "  Running service contract tests..."
cargo test --test '*' -- --nocapture service_contracts 2>&1 | tee "$RESULTS_DIR/service_contracts.log" || {
    echo -e "${RED}✗ Service contract tests failed${NC}"
    exit 1
}

# Run consumer contract tests
echo "  Running consumer contract tests..."
cargo test --test '*' -- --nocapture consumer_contracts 2>&1 | tee "$RESULTS_DIR/consumer_contracts.log" || {
    echo -e "${RED}✗ Consumer contract tests failed${NC}"
    exit 1
}

# Run event contract tests
echo "  Running event contract tests..."
cargo test --test '*' -- --nocapture event_contracts 2>&1 | tee "$RESULTS_DIR/event_contracts.log" || {
    echo -e "${RED}✗ Event contract tests failed${NC}"
    exit 1
}

# Run database contract tests
echo "  Running database contract tests..."
cargo test --test '*' -- --nocapture database_contracts 2>&1 | tee "$RESULTS_DIR/database_contracts.log" || {
    echo -e "${RED}✗ Database contract tests failed${NC}"
    exit 1
}

# Step 3: Generate contract test report
echo ""
echo -e "${YELLOW}Step 3: Generating contract test report...${NC}"

cat > "$RESULTS_DIR/contract_test_report.md" <<EOF
# Contract Test Report

**Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Build:** ${CI_BUILD_ID:-local}

## Summary

| Category | Status |
|----------|--------|
| JSON Schemas | ✓ $SCHEMA_COUNT validated |
| API Contracts | ✓ Passed |
| Service Contracts | ✓ Passed |
| Consumer Contracts | ✓ Passed |
| Event Contracts | ✓ Passed |
| Database Contracts | ✓ Passed |

## Schema Validation

All JSON schemas in \`$SCHEMAS_DIR\` have been validated:

EOF

for schema in "$SCHEMAS_DIR"/*.json; do
    if [ -f "$schema" ]; then
        echo "- ✓ $(basename "$schema")" >> "$RESULTS_DIR/contract_test_report.md"
    fi
done

cat >> "$RESULTS_DIR/contract_test_report.md" <<EOF

## Contract Test Categories

### 1. API Contracts
Tests verify that API endpoints comply with their defined contracts including:
- Request/response schemas
- HTTP status codes
- Error handling
- Data validation

### 2. Service Contracts
Tests verify that service plugins comply with the ServicePlugin trait:
- start() capability
- stop() capability
- health_check() capability
- Lifecycle management

### 3. Consumer Contracts
Tests verify consumer-driven contracts between modules:
- Backend-Cleanroom interaction
- Service Registry contracts
- Capability Registry contracts
- Plugin interface compliance

### 4. Event Contracts
Tests verify async event contracts:
- Service lifecycle events
- Container lifecycle events
- Test execution events
- Capability events

### 5. Database Contracts
Tests verify database schema contracts:
- Table definitions
- Column constraints
- Indexes
- Foreign keys
- Migrations

## Test Logs

Detailed test logs are available in:
EOF

for log in "$RESULTS_DIR"/*.log; do
    if [ -f "$log" ]; then
        echo "- \`$(basename "$log")\`" >> "$RESULTS_DIR/contract_test_report.md"
    fi
done

echo ""
echo -e "${GREEN}✓ Contract test report generated: $RESULTS_DIR/contract_test_report.md${NC}"

# Step 4: Check for contract breaking changes
echo ""
echo -e "${YELLOW}Step 4: Checking for contract breaking changes...${NC}"

# This would compare current contracts with baseline
# For now, we just check that all schemas exist
REQUIRED_SCHEMAS=(
    "service_plugin_contract.json"
    "backend_capabilities_contract.json"
    "cleanroom_api_contract.json"
    "database_schema_contract.json"
)

MISSING_SCHEMAS=0
for schema in "${REQUIRED_SCHEMAS[@]}"; do
    if [ ! -f "$SCHEMAS_DIR/$schema" ]; then
        echo -e "${RED}✗ Required schema missing: $schema${NC}"
        MISSING_SCHEMAS=$((MISSING_SCHEMAS + 1))
    fi
done

if [ $MISSING_SCHEMAS -eq 0 ]; then
    echo -e "${GREEN}✓ All required schemas present${NC}"
else
    echo -e "${RED}✗ $MISSING_SCHEMAS required schemas missing${NC}"
    exit 1
fi

# Step 5: Publish contract test results
echo ""
echo -e "${YELLOW}Step 5: Publishing contract test results...${NC}"

# In CI, this would publish to artifact storage
if [ -n "${CI:-}" ]; then
    echo "  Publishing to CI artifacts..."
    # Example: aws s3 cp "$RESULTS_DIR" s3://bucket/contract-tests/ --recursive
else
    echo "  Running locally, results saved to: $RESULTS_DIR"
fi

echo ""
echo -e "${GREEN}=== Contract Testing Complete ===${NC}"
echo ""
echo "Summary:"
echo "  - Schemas validated: $SCHEMA_COUNT"
echo "  - Contract test suites: 5"
echo "  - Report: $RESULTS_DIR/contract_test_report.md"
echo ""
echo -e "${GREEN}✓ All contract tests passed!${NC}"

exit 0
