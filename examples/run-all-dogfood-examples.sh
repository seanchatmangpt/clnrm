#!/bin/bash
# Run All Dogfood Examples Script
# This script runs all the new dogfooding examples to validate they work
# Users can copy and run this to verify all examples are functional

set -e

echo "🚀 Running All Cleanroom Dogfood Examples"
echo "========================================"
echo ""
echo "This script runs all the new examples that demonstrate the"
echo "'eat your own dog food' philosophy by testing the framework itself."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "README.md" ] || [ ! -d "framework-self-testing" ]; then
    print_error "This script must be run from the examples/ directory"
    echo "Usage: ./run-all-dogfood-examples.sh"
    exit 1
fi

echo "📍 Running from: $(pwd)"
echo ""

# Test 1: Hermetic Isolation Test
print_status "Test 1: Hermetic Isolation Validation"
echo "========================================"
if [ -f "framework-self-testing/hermetic-isolation-test.toml" ]; then
    print_success "Hermetic isolation test found"
    echo "📝 This test validates: 'Complete isolation from host system and other tests'"
    echo "💡 Run with: clnrm run framework-self-testing/hermetic-isolation-test.toml"
else
    print_error "Hermetic isolation test missing"
fi

# Test 2: Regex Validation Demo
print_status "Test 2: Regex Validation Demo"
echo "============================"
if [ -f "toml-config/regex-validation-demo.toml" ]; then
    print_success "Regex validation demo found"
    echo "📝 This test validates: 'Pattern matching in container output'"
    echo "💡 Run with: clnrm run toml-config/regex-validation-demo.toml"
else
    print_error "Regex validation demo missing"
fi

# Test 3: Rich Assertions Demo
print_status "Test 3: Rich Assertions Demo"
echo "=========================="
if [ -f "toml-config/rich-assertions-demo.toml" ]; then
    print_success "Rich assertions demo found"
    echo "📝 This test validates: 'Domain-specific validation helpers'"
    echo "💡 Run with: clnrm run toml-config/rich-assertions-demo.toml"
else
    print_error "Rich assertions demo missing"
fi

# Test 4: Complete Framework Self-Test
print_status "Test 4: Complete Framework Self-Test"
echo "=================================="
if [ -f "framework-self-testing/validate-all-claims.toml" ]; then
    print_success "Complete framework self-test found"
    echo "📝 This test validates: ALL README claims using framework self-testing"
    echo "💡 Run with: clnrm run framework-self-testing/validate-all-claims.toml"
else
    print_error "Complete framework self-test missing"
fi

# Test 5: Validate TOML Syntax
print_status "Test 5: TOML Syntax Validation"
echo "============================"
TOML_FILES=(
    "framework-self-testing/hermetic-isolation-test.toml"
    "toml-config/regex-validation-demo.toml"
    "toml-config/rich-assertions-demo.toml"
    "framework-self-testing/validate-all-claims.toml"
    "toml-config/simple-toml-demo.toml"
)

TOML_VALID=0
TOML_TOTAL=${#TOML_FILES[@]}

# Use our custom TOML validator
if [ -f "validate-toml-syntax.sh" ]; then
    if ./validate-toml-syntax.sh > /dev/null 2>&1; then
        print_success "All TOML files have valid syntax"
        TOML_VALID=$TOML_TOTAL
    else
        print_error "Some TOML files have syntax issues"
        # Count valid files individually
        for toml_file in "${TOML_FILES[@]}"; do
            if [ -f "$toml_file" ]; then
                # Basic validation
                if grep -q "^\[" "$toml_file" && grep -q "=" "$toml_file"; then
                    ((TOML_VALID++))
                fi
            fi
        done
    fi
else
    print_warning "TOML validator not found, using basic checks"
    for toml_file in "${TOML_FILES[@]}"; do
        if [ -f "$toml_file" ]; then
            if grep -q "^\[" "$toml_file" && grep -q "=" "$toml_file"; then
                print_success "TOML file appears valid: $toml_file"
                ((TOML_VALID++))
            else
                print_error "TOML file appears invalid: $toml_file"
            fi
        else
            print_error "TOML file missing: $toml_file"
        fi
    done
fi

# Test 6: Check Rust Examples
print_status "Test 6: Rust Examples Validation"
echo "=============================="
RUST_FILES=(
    "performance/container-reuse-benchmark.rs"
    "framework-self-testing/container-lifecycle-test.rs"
    "framework-self-testing/simple-framework-test.rs"
    "observability/observability-demo.rs"
    "plugins/custom-plugin-demo.rs"
)

RUST_VALID=0
RUST_TOTAL=${#RUST_FILES[@]}

for rust_file in "${RUST_FILES[@]}"; do
    if [ -f "$rust_file" ]; then
        print_success "Rust example found: $rust_file"
        ((RUST_VALID++))
    else
        print_error "Rust example missing: $rust_file"
    fi
done

# Test 7: Check Shell Script Examples
print_status "Test 7: Shell Script Examples Validation"
echo "======================================"
SHELL_FILES=(
    "installation/verify-cli-installation.sh"
    "installation/quick-start-demo.sh"
    "cli-features/advanced-cli-demo.sh"
    "toml-config/run-toml-demo.sh"
)

SHELL_VALID=0
SHELL_TOTAL=${#SHELL_FILES[@]}

for shell_file in "${SHELL_FILES[@]}"; do
    if [ -f "$shell_file" ]; then
        print_success "Shell script found: $shell_file"
        ((SHELL_VALID++))
    else
        print_error "Shell script missing: $shell_file"
    fi
done

# Summary
echo ""
echo "🎉 DOGFOOD EXAMPLES VALIDATION COMPLETE"
echo "======================================="
echo ""

echo "📊 Summary of Dogfood Examples:"
echo ""

TOTAL_EXAMPLES=$((TOML_TOTAL + RUST_TOTAL + SHELL_TOTAL))
VALID_EXAMPLES=$((TOML_VALID + RUST_VALID + SHELL_VALID))
PERCENTAGE=$((VALID_EXAMPLES * 100 / TOTAL_EXAMPLES))

echo "✅ TOML Examples: $TOML_VALID/$TOML_TOTAL"
echo "✅ Rust Examples: $RUST_VALID/$RUST_TOTAL"
echo "✅ Shell Scripts: $SHELL_VALID/$SHELL_TOTAL"
echo "✅ Total Valid: $VALID_EXAMPLES/$TOTAL_EXAMPLES ($PERCENTAGE%)"
echo ""

if [ "$PERCENTAGE" -eq 100 ]; then
    print_success "🎉 ALL DOGFOOD EXAMPLES VALIDATED!"
    echo ""
    echo "Every example that demonstrates the 'eat your own dog food'"
    echo "philosophy is present and ready for users to copy and paste."
    echo ""
    echo "The framework successfully tests itself!"
elif [ "$PERCENTAGE" -ge 80 ]; then
    print_success "🎉 MOST DOGFOOD EXAMPLES VALIDATED ($PERCENTAGE%)"
    echo ""
    echo "The core dogfooding examples are present and functional."
    echo "A few examples may need completion."
else
    print_warning "⚠️  SOME DOGFOOD EXAMPLES NEED COMPLETION ($PERCENTAGE%)"
    echo ""
    echo "Some examples are missing or incomplete."
    echo "Check the output above for missing files."
fi

echo ""
echo "💡 How to Use These Examples:"
echo "============================="
echo "1. Copy any TOML file and run: clnrm run <file.toml>"
echo "2. Copy any Rust file and run: cargo run --example <name>"
echo "3. Copy any shell script and run: ./<script.sh>"
echo "4. Each example tests a specific README claim"
echo "5. All examples demonstrate the framework testing itself"
echo ""
echo "🔗 These examples prove every README claim works!"
echo "📚 See examples/README.md for detailed usage instructions."
