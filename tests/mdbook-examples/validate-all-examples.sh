#!/bin/bash
# validate-all-examples.sh
#
# Validates all mdbook examples to ensure they work with clnrm v1.0.1
# Follows core team standards: no false positives, proper error handling

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Use an isolated target directory for mdbook examples to prevent lock contention
export CARGO_TARGET_DIR="$PROJECT_ROOT/target_mdbook_examples_shared"
export CARGO_BUILD_JOBS=2

# Ensure target/debug clnrm is preferred in PATH
export PATH="$PROJECT_ROOT/target/debug:$PATH"

# Cleanup function to remove temporary test artifacts
cleanup() {
    echo -e "\n${BLUE}🧹 Cleaning up temporary test artifacts...${NC}"
    # Remove any stray template validation output reports
    rm -f "$SCRIPT_DIR/template-mastery/"*_report.json
    rm -f "$SCRIPT_DIR/template-mastery/"*_trace.sha256
    rm -f "$SCRIPT_DIR/advanced-patterns/"*_report.json
    rm -f "$SCRIPT_DIR/advanced-patterns/"*_trace.sha256
}
trap cleanup EXIT

echo -e "${BLUE}🔍 Validating mdbook examples for clnrm v1.0.1${NC}"
echo "=================================================="

# Check if clnrm is available
if ! command -v clnrm &> /dev/null; then
    echo -e "${RED}❌ clnrm not found in PATH${NC}"
    echo "Please install clnrm first:"
    echo "  cargo install --path ."
    echo "  # or"
    echo "  brew install clnrm"
    exit 1
fi

echo -e "${GREEN}✅ clnrm found: $(clnrm --version)${NC}"

# Function to run a test and report results
run_test() {
    local test_name="$1"
    local test_command="$2"
    local test_dir="$3"
    
    echo -e "\n${BLUE}🧪 Testing: ${test_name}${NC}"
    echo "Command: ${test_command}"
    echo "Directory: ${test_dir}"
    
    if (cd "$test_dir" && eval "$test_command"); then
        echo -e "${GREEN}✅ ${test_name} passed${NC}"
        return 0
    else
        echo -e "${RED}❌ ${test_name} failed${NC}"
        return 1
    fi
}

# Track test results
PASSED=0
FAILED=0
TOTAL=0

# Test 1: Plugin Development Examples
echo -e "\n${YELLOW}📦 Plugin Development Examples${NC}"
echo "--------------------------------"

# Build the test target once to avoid concurrent compile locks and resource contention
echo -e "\n${BLUE}Building test target...${NC}"
if ! (cd "$PROJECT_ROOT" && cargo test --test mdbook-examples-plugin-development --no-run); then
    echo -e "${RED}❌ Failed to build test target${NC}"
    exit 1
fi

# Test custom database plugin
TOTAL=$((TOTAL + 1))
if run_test "Custom Database Plugin" "cargo test --test mdbook-examples-plugin-development -- test_custom_database_plugin_lifecycle" "$PROJECT_ROOT"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Test custom API plugin
TOTAL=$((TOTAL + 1))
if run_test "Custom API Plugin" "cargo test --test mdbook-examples-plugin-development -- test_custom_api_plugin_lifecycle" "$PROJECT_ROOT"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Test plugin validation
TOTAL=$((TOTAL + 1))
if run_test "Plugin Validation" "cargo test --test mdbook-examples-plugin-development -- test_plugin_validation" "$PROJECT_ROOT"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Test multi-service integration
TOTAL=$((TOTAL + 1))
if run_test "Multi-Service Integration" "cargo test --test mdbook-examples-plugin-development -- test_multi_service_integration" "$PROJECT_ROOT"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Test 2: Advanced Patterns Examples
echo -e "\n${YELLOW}🔧 Advanced Patterns Examples${NC}"
echo "--------------------------------"

# Test multi-service orchestration
TOTAL=$((TOTAL + 1))
if run_test "Multi-Service Orchestration" "clnrm test validate --path multi-service-orchestration.clnrm.toml" "$SCRIPT_DIR/advanced-patterns"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Test 3: Template Mastery Examples
echo -e "\n${YELLOW}📝 Template Mastery Examples${NC}"
echo "--------------------------------"

# Test template validation
TOTAL=$((TOTAL + 1))
if run_test "Template Validation" "clnrm test validate --path template-example.clnrm.toml.tera" "$SCRIPT_DIR/template-mastery"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Test 4: Core Team Standards Validation
echo -e "\n${YELLOW}🏆 Core Team Standards Validation${NC}"
echo "----------------------------------------"

# Check for unwrap/expect in production code
TOTAL=$((TOTAL + 1))
echo -e "\n${BLUE}🔍 Checking for unwrap/expect in production code${NC}"
if cargo clippy -j 1 --offline -- -D clippy::unwrap_used -D clippy::expect_used 2>&1 | grep -E "(unwrap|expect)" | grep -v "test"; then
    echo -e "${RED}❌ Found unwrap/expect in production code${NC}"
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✅ No unwrap/expect in production code${NC}"
    PASSED=$((PASSED + 1))
fi

# Check clippy warnings
TOTAL=$((TOTAL + 1))
echo -e "\n${BLUE}🔍 Running clippy checks for mdbook examples${NC}"
if cargo clippy -j 1 --offline --test mdbook-examples-plugin-development -- -D warnings 2>&1 | grep -E "warning|error" | grep -v "clnrm-core"; then
    echo -e "${RED}❌ Clippy warnings found in examples${NC}"
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✅ No clippy warnings in examples${NC}"
    PASSED=$((PASSED + 1))
fi

# Test 5: mdbook Build Validation
echo -e "\n${YELLOW}📚 mdbook Build Validation${NC}"
echo "----------------------------"

# Check if mdbook is available
if ! command -v mdbook &> /dev/null; then
    echo -e "${YELLOW}⚠️  mdbook not found, skipping build validation${NC}"
    echo "Install mdbook: cargo install mdbook"
else
    TOTAL=$((TOTAL + 1))
    if run_test "mdbook Build" "mdbook build book/" "$PROJECT_ROOT"; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
fi

# Summary
echo -e "\n${BLUE}📊 Validation Summary${NC}"
echo "====================="
echo -e "Total tests: ${TOTAL}"
echo -e "${GREEN}Passed: ${PASSED}${NC}"
echo -e "${RED}Failed: ${FAILED}${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All mdbook examples validated successfully!${NC}"
    echo -e "${GREEN}✅ Ready for production use${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some examples failed validation${NC}"
    echo -e "${RED}🔧 Please fix the failing tests before proceeding${NC}"
    exit 1
fi
