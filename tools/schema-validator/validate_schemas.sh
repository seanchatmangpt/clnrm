#!/usr/bin/env bash
# Schema Validation Script
# Runs all schema validation checks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REGISTRY_DIR="$PROJECT_ROOT/registry"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "==============================================="
echo "CLNRM Schema Validation"
echo "==============================================="
echo ""

# Track overall status
VALIDATION_FAILED=0

# Function to print status
print_status() {
    local status=$1
    local message=$2

    if [ "$status" -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $message"
    else
        echo -e "${RED}✗${NC} $message"
        VALIDATION_FAILED=1
    fi
}

# 1. Check Weaver is installed
echo "Checking prerequisites..."
if ! command -v weaver &> /dev/null; then
    echo -e "${YELLOW}⚠${NC} Weaver not found, installing..."

    # Detect OS
    OS="$(uname -s)"
    case "${OS}" in
        Linux*)     PLATFORM=linux;;
        Darwin*)    PLATFORM=macos;;
        *)          echo "Unsupported platform: ${OS}"; exit 1;;
    esac

    # Download weaver
    WEAVER_VERSION="v0.10.0"
    WEAVER_URL="https://github.com/open-telemetry/weaver/releases/download/${WEAVER_VERSION}/weaver-${PLATFORM}-x86_64"

    curl -L "$WEAVER_URL" -o /tmp/weaver
    chmod +x /tmp/weaver
    sudo mv /tmp/weaver /usr/local/bin/weaver

    print_status 0 "Weaver installed"
else
    print_status 0 "Weaver found: $(weaver --version)"
fi

echo ""
echo "==============================================="
echo "1. Schema Syntax Validation"
echo "==============================================="

# Run weaver registry check
if weaver registry check --registry "$REGISTRY_DIR"; then
    print_status 0 "Schema syntax valid"
else
    print_status 1 "Schema syntax errors found"
fi

echo ""
echo "==============================================="
echo "2. Schema Completeness Check"
echo "==============================================="

# Check all required schemas exist
REQUIRED_SCHEMAS=(
    "span.clnrm.test_execution"
    "span.clnrm.container_lifecycle"
    "span.clnrm.plugin_execution"
    "span.clnrm.service_command"
    "metric.clnrm.test.duration"
    "metric.clnrm.test.count"
    "metric.clnrm.container.count"
    "metric.clnrm.container.lifetime"
    "metric.clnrm.isolation.score"
    "event.clnrm.test.started"
    "event.clnrm.test.completed"
    "event.clnrm.test.failed"
    "event.clnrm.container.leaked"
    "event.clnrm.isolation.violation"
)

MISSING_SCHEMAS=0

for schema_id in "${REQUIRED_SCHEMAS[@]}"; do
    # Search for schema in YAML files
    if grep -r "id: $schema_id" "$REGISTRY_DIR" &> /dev/null; then
        echo -e "${GREEN}✓${NC} Found: $schema_id"
    else
        echo -e "${RED}✗${NC} Missing: $schema_id"
        MISSING_SCHEMAS=$((MISSING_SCHEMAS + 1))
        VALIDATION_FAILED=1
    fi
done

if [ "$MISSING_SCHEMAS" -eq 0 ]; then
    print_status 0 "All required schemas present"
else
    print_status 1 "$MISSING_SCHEMAS required schemas missing"
fi

echo ""
echo "==============================================="
echo "3. Critical Attributes Check"
echo "==============================================="

# Function to check attribute in schema
check_attribute() {
    local schema_file=$1
    local attr_name=$2
    local requirement=$3

    # Check if attribute exists and get its requirement level
    if grep -q "id: $attr_name" "$schema_file"; then
        if grep -A 4 "id: $attr_name" "$schema_file" | grep -q "requirement_level: $requirement"; then
            echo -e "${GREEN}✓${NC} $attr_name is $requirement"
            return 0
        else
            # Get actual requirement level
            actual=$(grep -A 4 "id: $attr_name" "$schema_file" | grep "requirement_level:" | head -1 | awk '{print $2}')
            echo -e "${RED}✗${NC} $attr_name is '$actual', should be '$requirement'"
            VALIDATION_FAILED=1
            return 1
        fi
    else
        echo -e "${RED}✗${NC} $attr_name not found in schema"
        VALIDATION_FAILED=1
        return 1
    fi
}

# Check test_execution schema
TEST_EXEC_FILE="$REGISTRY_DIR/core/test_execution.yaml"
if [ -f "$TEST_EXEC_FILE" ]; then
    echo "Checking test_execution schema..."
    check_attribute "$TEST_EXEC_FILE" "container.id" "required"
    check_attribute "$TEST_EXEC_FILE" "test.isolated" "required"
    check_attribute "$TEST_EXEC_FILE" "test.result" "required"
    check_attribute "$TEST_EXEC_FILE" "test.duration_ms" "required"
    check_attribute "$TEST_EXEC_FILE" "test.cleanup_performed" "required"
fi

# Check container_lifecycle schema
CONTAINER_FILE="$REGISTRY_DIR/core/container_lifecycle.yaml"
if [ -f "$CONTAINER_FILE" ]; then
    echo ""
    echo "Checking container_lifecycle schema..."
    check_attribute "$CONTAINER_FILE" "container.id" "required"
    check_attribute "$CONTAINER_FILE" "container.created_at" "required"
    check_attribute "$CONTAINER_FILE" "container.destroyed_at" "required"
    check_attribute "$CONTAINER_FILE" "container.state" "required"
    check_attribute "$CONTAINER_FILE" "cleanup.success" "required"
fi

# Check plugin_execution schema
PLUGIN_FILE="$REGISTRY_DIR/core/plugin_system.yaml"
if [ -f "$PLUGIN_FILE" ]; then
    echo ""
    echo "Checking plugin_execution schema..."
    check_attribute "$PLUGIN_FILE" "plugin.name" "required"
    check_attribute "$PLUGIN_FILE" "plugin.state" "required"
    check_attribute "$PLUGIN_FILE" "container.id" "required"
    check_attribute "$PLUGIN_FILE" "plugin.health_check.performed" "required"
    check_attribute "$PLUGIN_FILE" "plugin.health_check.passed" "required"
fi

echo ""
echo "==============================================="
echo "4. Enum Validation"
echo "==============================================="

# Check test.result is enum with correct values
if grep -A 20 "id: test.result" "$TEST_EXEC_FILE" | grep -q "allow_custom_values: false"; then
    print_status 0 "test.result is strict enum"
else
    print_status 1 "test.result should have allow_custom_values: false"
fi

# Check container.state is enum
if grep -A 20 "id: container.state" "$CONTAINER_FILE" | grep -q "allow_custom_values: false"; then
    print_status 0 "container.state is strict enum"
else
    print_status 1 "container.state should have allow_custom_values: false"
fi

# Check plugin.state is enum
if grep -A 20 "id: plugin.state" "$PLUGIN_FILE" | grep -q "allow_custom_values: false"; then
    print_status 0 "plugin.state is strict enum"
else
    print_status 1 "plugin.state should have allow_custom_values: false"
fi

echo ""
echo "==============================================="
echo "5. Stability Check"
echo "==============================================="

# Check all core schemas are marked stable
CORE_SCHEMAS=$(find "$REGISTRY_DIR/core" -name "*.yaml")
UNSTABLE_SCHEMAS=0

for schema_file in $CORE_SCHEMAS; do
    if grep -q "stability: stable" "$schema_file"; then
        echo -e "${GREEN}✓${NC} $(basename "$schema_file") is stable"
    else
        echo -e "${YELLOW}⚠${NC} $(basename "$schema_file") is not marked stable"
        # Don't fail validation, but warn
    fi
done

echo ""
echo "==============================================="
echo "6. False Positive Risk Check"
echo "==============================================="

# Check for optional attributes that should be required
echo "Checking for false positive risks..."

RISKS=0

# Check if container.id is ever optional (should never be!)
if grep -r "id: container.id" "$REGISTRY_DIR" | xargs grep -l "recommended\|optional" &> /dev/null; then
    echo -e "${RED}✗${NC} container.id found as optional/recommended (should be required!)"
    RISKS=$((RISKS + 1))
    VALIDATION_FAILED=1
fi

# Check if test.isolated is ever optional
if grep -r "id: test.isolated" "$REGISTRY_DIR" | xargs grep -l "recommended\|optional" &> /dev/null; then
    echo -e "${RED}✗${NC} test.isolated found as optional/recommended (should be required!)"
    RISKS=$((RISKS + 1))
    VALIDATION_FAILED=1
fi

# Check if any critical attributes use arbitrary string types
if grep -A 2 "id: test.result" "$REGISTRY_DIR/core/test_execution.yaml" | grep -q "type: string$"; then
    echo -e "${RED}✗${NC} test.result is string type (should be enum!)"
    RISKS=$((RISKS + 1))
    VALIDATION_FAILED=1
fi

if [ "$RISKS" -eq 0 ]; then
    print_status 0 "No false positive risks detected"
else
    print_status 1 "$RISKS false positive risks found"
fi

echo ""
echo "==============================================="
echo "Validation Summary"
echo "==============================================="
echo ""

if [ "$VALIDATION_FAILED" -eq 0 ]; then
    echo -e "${GREEN}✓ All validations passed${NC}"
    echo ""
    echo "Schemas are:"
    echo "  - Syntactically valid"
    echo "  - Complete (all behaviors covered)"
    echo "  - Correct (types and requirements appropriate)"
    echo "  - Safe (no false positive risks)"
    exit 0
else
    echo -e "${RED}✗ Validation failed${NC}"
    echo ""
    echo "Fix the issues above before committing schema changes."
    exit 1
fi
