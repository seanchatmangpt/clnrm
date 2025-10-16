#!/bin/bash
# Verification script for integration testing infrastructure

set -e

echo "🧪 Integration Testing Infrastructure Verification"
echo "=================================================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Docker is available
echo "📦 Checking Docker availability..."
if command -v docker &> /dev/null; then
    if docker ps &> /dev/null; then
        echo -e "${GREEN}✓${NC} Docker is available and running"
    else
        echo -e "${YELLOW}⚠${NC} Docker is installed but not running"
        echo "  Start Docker with: systemctl start docker (Linux) or open -a Docker (macOS)"
    fi
else
    echo -e "${YELLOW}⚠${NC} Docker is not installed"
    echo "  Docker is required for system integration tests"
fi
echo ""

# Check file structure
echo "📁 Checking file structure..."
files=(
    "docs/INTEGRATION_TEST_STRATEGY.md"
    "docs/INTEGRATION_TESTING_COMPLETE.md"
    "tests/integration/mod.rs"
    "tests/integration/helpers/mod.rs"
    "tests/integration/fixtures/mod.rs"
    "tests/integration/factories/mod.rs"
    "tests/integration/assertions/mod.rs"
    "tests/integration/common/mod.rs"
    "tests/integration/component_integration_test.rs"
    "tests/integration/system_integration_test.rs"
    "tests/integration/database_integration_test.rs"
    "tests/integration/external_service_test.rs"
    "tests/integration/docker-compose.test.yml"
    "tests/integration/otel-collector-config.yml"
    "tests/integration/prometheus-config.yml"
    "tests/integration/README.md"
    ".github/workflows/integration-tests.yml"
)

missing_files=()
for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $file"
    else
        echo -e "${RED}✗${NC} $file (missing)"
        missing_files+=("$file")
    fi
done
echo ""

if [ ${#missing_files[@]} -gt 0 ]; then
    echo -e "${RED}Error: ${#missing_files[@]} file(s) missing${NC}"
    exit 1
fi

# Check Rust compilation
echo "🦀 Checking Rust compilation..."
if cargo check --tests 2>&1 | tee /tmp/cargo-check.log; then
    echo -e "${GREEN}✓${NC} Tests compile successfully"
else
    echo -e "${RED}✗${NC} Compilation errors detected"
    echo "See /tmp/cargo-check.log for details"
    exit 1
fi
echo ""

# Run component tests (fast)
echo "🧩 Running component integration tests..."
if cargo test --test component_integration_test 2>&1 | tee /tmp/component-tests.log; then
    echo -e "${GREEN}✓${NC} Component integration tests passed"
else
    echo -e "${RED}✗${NC} Component integration tests failed"
    echo "See /tmp/component-tests.log for details"
    exit 1
fi
echo ""

# Check Docker Compose configuration
echo "🐳 Validating Docker Compose configuration..."
if docker-compose -f tests/integration/docker-compose.test.yml config > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Docker Compose configuration is valid"
else
    echo -e "${RED}✗${NC} Docker Compose configuration has errors"
    exit 1
fi
echo ""

# Summary
echo "=================================================="
echo "✅ Integration Testing Infrastructure Verified"
echo ""
echo "Next steps:"
echo "  1. Start test environment: docker-compose -f tests/integration/docker-compose.test.yml up -d"
echo "  2. Run system tests: cargo test --test system_integration_test -- --ignored"
echo "  3. Run database tests: cargo test --test database_integration_test -- --ignored"
echo "  4. Run all tests: cargo test --test '*' -- --test-threads=4"
echo ""
echo "Documentation:"
echo "  - Strategy: docs/INTEGRATION_TEST_STRATEGY.md"
echo "  - Usage: tests/integration/README.md"
echo "  - Summary: docs/INTEGRATION_TESTING_COMPLETE.md"
echo ""
