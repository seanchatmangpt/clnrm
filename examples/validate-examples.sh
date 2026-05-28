#!/bin/bash
# Example Validation Script
# Tests that the new examples are properly formatted and can be parsed

set -e

echo "🚀 Validating New Examples"
echo "=========================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
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

# Test TOML files can be parsed
echo ""
echo "📋 Testing TOML files..."

for toml_file in examples/*.toml; do
    if [ -f "$toml_file" ]; then
        echo "  Checking $toml_file..."
        if command -v python3 &> /dev/null; then
            if python3 -c "import tomllib; tomllib.load(open('$toml_file', 'rb'))" 2>/dev/null; then
                print_success "TOML syntax valid: $toml_file"
            else
                print_error "TOML syntax invalid: $toml_file"
            fi
        else
            print_warning "Python3 not available, skipping TOML validation for $toml_file"
        fi
    fi
done

# Test Rust files compile (basic syntax check)
echo ""
echo "📋 Testing Rust files..."

for rs_file in examples/*.rs; do
    if [ -f "$rs_file" ]; then
        echo "  Checking $rs_file..."
        if rustc --emit=dep-info "$rs_file" 2>/dev/null; then
            print_success "Rust syntax valid: $rs_file"
        else
            print_error "Rust syntax invalid: $rs_file"
        fi
    fi
done

# Test shell scripts
echo ""
echo "📋 Testing shell scripts..."

for sh_file in examples/*.sh; do
    if [ -f "$sh_file" ] && [ -x "$sh_file" ]; then
        echo "  Checking $sh_file..."
        if bash -n "$sh_file" 2>/dev/null; then
            print_success "Shell syntax valid: $sh_file"
        else
            print_error "Shell syntax invalid: $sh_file"
        fi
    fi
done

# Check that examples are documented
echo ""
echo "📋 Checking documentation..."

if [ -f "examples/README.md" ]; then
    print_success "Examples README exists"
else
    print_error "Examples README missing"
fi

echo ""
echo "🎉 Validation completed!"
echo ""
echo "💡 Next steps:"
echo "   - Run 'cargo build' to ensure framework compiles"
echo "   - Test examples manually: clnrm run examples/simple-working-test.clnrm.toml"
echo "   - Review examples/README.md for usage instructions"

