#!/bin/bash
# Schema Registry Validation Script
# This script validates the clnrm schema registry using Weaver

set -e

echo "=================================================="
echo "CLNRM Schema Registry Validation"
echo "=================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if weaver is installed
if ! command -v weaver &> /dev/null; then
    echo -e "${RED}ERROR: weaver not found${NC}"
    echo "Install with: cargo install weaver-cli"
    echo "See: https://github.com/open-telemetry/weaver"
    exit 1
fi

echo -e "${GREEN}✓ Weaver found${NC}"
echo ""

# Change to registry directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Validating registry at: $SCRIPT_DIR"
echo ""

# Run weaver registry check
echo "=================================================="
echo "Running weaver registry check..."
echo "=================================================="
echo ""

if weaver registry check -r .; then
    echo ""
    echo -e "${GREEN}✓✓✓ VALIDATION PASSED ✓✓✓${NC}"
    echo ""
    echo "Schema registry is valid and ready for use."
    echo ""

    # Count schemas
    echo "Schema Statistics:"
    echo "  - Core schemas:   $(find core -name '*.yaml' | wc -l | tr -d ' ')"
    echo "  - Metric schemas: $(find metrics -name '*.yaml' | wc -l | tr -d ' ')"
    echo "  - Event schemas:  $(find events -name '*.yaml' | wc -l | tr -d ' ')"
    echo ""

    # Count groups
    SPAN_COUNT=$(grep -h "type: span" core/*.yaml | wc -l | tr -d ' ')
    METRIC_COUNT=$(grep -h "type: metric" metrics/*.yaml | wc -l | tr -d ' ')
    EVENT_COUNT=$(grep -h "type: event" events/*.yaml | wc -l | tr -d ' ')

    echo "Schema Coverage:"
    echo "  - Spans:   $SPAN_COUNT"
    echo "  - Metrics: $METRIC_COUNT"
    echo "  - Events:  $EVENT_COUNT"
    echo ""

    echo "Next Steps:"
    echo "  1. Implement instrumentation (Instrumentation Engineer)"
    echo "  2. Create validation tests (Test Engineer)"
    echo "  3. Setup CI/CD validation (DevOps Agent)"
    echo ""

    exit 0
else
    echo ""
    echo -e "${RED}✗✗✗ VALIDATION FAILED ✗✗✗${NC}"
    echo ""
    echo "Schema registry has errors. See output above for details."
    echo ""
    exit 1
fi
