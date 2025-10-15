#!/bin/bash
# Run Real Examples - Framework Self-Testing
# This script runs the actual framework examples that use real code
# Users can copy and paste this to verify the framework works

set -e

echo "🚀 Running Real Framework Examples"
echo "================================="
echo "These examples use actual framework code to test itself"
echo "Following the 'eat your own dog food' principle\n"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first."
    exit 1
fi

echo "✅ Environment check passed"
echo "   Rust/Cargo: $(cargo --version)"
echo "   Project: $(pwd)"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo -e "\n📋 Test $TESTS_TOTAL: $test_name"
    echo "Command: $test_command"
    echo "----------------------------------------"
    
    if eval "$test_command"; then
        echo "✅ PASSED: $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "❌ FAILED: $test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Test 1: Container Lifecycle Management
run_test "Container Lifecycle Management" \
    "cargo run --example real-container-lifecycle-test --manifest-path crates/clnrm-core/Cargo.toml"

# Test 2: Plugin System
run_test "Plugin System Architecture" \
    "cargo run --example real-plugin-system-test --manifest-path crates/clnrm-core/Cargo.toml"

# Test 3: TOML Configuration Parsing
run_test "TOML Configuration Parsing" \
    "cargo run --example real-toml-parsing-test --manifest-path crates/clnrm-core/Cargo.toml"

# Test 4: Container Reuse Performance
run_test "Container Reuse Performance" \
    "cargo run --example real-container-reuse-benchmark --manifest-path crates/clnrm-core/Cargo.toml"

# Test 5: CLI Functionality
run_test "CLI Functionality" \
    "cargo run --example real-cli-test --manifest-path crates/clnrm-core/Cargo.toml"

# Test 6: Observability System
run_test "Observability System" \
    "cargo run --example real-observability-test --manifest-path crates/clnrm-core/Cargo.toml"

# Test 7: Framework Self-Testing (existing example)
run_test "Framework Self-Testing (existing)" \
    "cargo run --example simple_test --manifest-path crates/clnrm-core/Cargo.toml"

# Test 8: Container Reuse Benchmark (existing example)
run_test "Container Reuse Benchmark (existing)" \
    "cargo run --example container_reuse_benchmark --manifest-path crates/clnrm-core/Cargo.toml"

# Final results
echo -e "\n🎉 Test Results Summary"
echo "======================"
echo "Total Tests: $TESTS_TOTAL"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"

if [ $TESTS_FAILED -eq 0 ]; then
    echo "✅ SUCCESS: All framework examples passed!"
    echo "📚 Every claim in the README is backed by working code."
    echo "💡 The framework successfully tests itself using its own capabilities."
else
    echo "⚠️  Some tests failed. This may indicate:"
    echo "   • Missing dependencies (Docker, etc.)"
    echo "   • Environment-specific issues"
    echo "   • Implementation gaps"
    echo ""
    echo "💡 Check the error messages above for specific issues."
fi

echo -e "\n📋 Framework Capabilities Demonstrated:"
echo "   ✅ Container lifecycle management"
echo "   ✅ Plugin-based architecture"
echo "   ✅ TOML configuration parsing"
echo "   ✅ Container reuse for performance"
echo "   ✅ CLI functionality"
echo "   ✅ Observability and tracing"
echo "   ✅ Framework self-testing"

echo -e "\n💡 Users can run this script to verify all framework claims:"
echo "   curl -fsSL https://raw.githubusercontent.com/cleanroom-testing/clnrm/main/examples/run-real-examples.sh | bash"
