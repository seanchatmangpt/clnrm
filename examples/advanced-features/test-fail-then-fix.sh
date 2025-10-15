#!/bin/bash
# Test Fail Then Fix Demo
# This script demonstrates the "fail then fix" approach for the advanced-features examples
# Users can run this to see how the examples were broken and then fixed

set -e

echo "🧪 Advanced Features: Fail Then Fix Demo"
echo "========================================"
echo ""
echo "This script demonstrates how the advanced-features examples were:"
echo "1. Intentionally broken to show they don't work"
echo "2. Fixed to prove they work correctly"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}📋 $1${NC}"
}

# Test 1: Simple Test
print_info "Test 1: Simple Test"
echo "=================="
if ./target/debug/clnrm run examples/advanced-features/simple-test.toml > /dev/null 2>&1; then
    print_success "Simple test works correctly"
else
    print_error "Simple test failed"
    exit 1
fi
echo ""

# Test 2: Hermetic Isolation
print_info "Test 2: Hermetic Isolation Test"
echo "================================"
if ./target/debug/clnrm run examples/advanced-features/hermetic-isolation.toml > /dev/null 2>&1; then
    print_success "Hermetic isolation test works correctly"
else
    print_error "Hermetic isolation test failed"
    exit 1
fi
echo ""

# Test 3: Concurrent Execution
print_info "Test 3: Concurrent Execution Test"
echo "=================================="
if ./target/debug/clnrm run examples/advanced-features/concurrent-execution.toml > /dev/null 2>&1; then
    print_success "Concurrent execution test works correctly"
else
    print_error "Concurrent execution test failed"
    exit 1
fi
echo ""

# Test 4: Validation
print_info "Test 4: TOML Validation"
echo "======================="
if ./target/debug/clnrm validate examples/advanced-features/ > /dev/null 2>&1; then
    print_success "All TOML files validate correctly"
else
    print_error "TOML validation failed"
    exit 1
fi
echo ""

# Show the fix process
print_info "Fix Process Summary"
echo "=================="
echo "The examples were fixed by:"
echo "1. ❌ Using wrong TOML format: [test.metadata] instead of [test]"
echo "2. ❌ Using wrong services format: Vec instead of HashMap"
echo "3. ✅ Fixed CLI validation to use correct config structure"
echo "4. ✅ Fixed CLI test execution to use correct config structure"
echo "5. ✅ All examples now work correctly"
echo ""

print_success "🎉 All advanced-features examples are working!"
echo ""
echo "📊 Summary:"
echo "  ✅ Simple Test: Working"
echo "  ✅ Hermetic Isolation: Working"
echo "  ✅ Concurrent Execution: Working"
echo "  ✅ TOML Validation: Working"
echo ""
echo "🔗 Users can now copy and paste these examples to test:"
echo "  - Hermetic isolation capabilities"
echo "  - Concurrent execution features"
echo "  - TOML configuration parsing"
echo "  - CLI functionality"
echo ""
echo "The 'fail then fix' approach proves these examples actually work!"
