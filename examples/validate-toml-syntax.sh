#!/bin/bash
# Simple TOML Syntax Validation
# This script validates TOML files using basic syntax checking

set -e

echo "🔍 Validating TOML Syntax"
echo "========================"

# Function to validate TOML syntax
validate_toml() {
    local file="$1"
    echo "📋 Validating: $file"
    
    # Basic TOML syntax checks
    if [ ! -f "$file" ]; then
        echo "❌ File not found: $file"
        return 1
    fi
    
    # Check for basic TOML structure
    if ! grep -q "^\[" "$file"; then
        echo "❌ No TOML sections found in $file"
        return 1
    fi
    
    # Check for balanced brackets
    local open_brackets=$(grep -o '\[' "$file" | wc -l)
    local close_brackets=$(grep -o '\]' "$file" | wc -l)
    
    if [ "$open_brackets" -ne "$close_brackets" ]; then
        echo "❌ Unbalanced brackets in $file (open: $open_brackets, close: $close_brackets)"
        return 1
    fi
    
    # Check for basic key-value pairs
    if ! grep -q "=" "$file"; then
        echo "❌ No key-value pairs found in $file"
        return 1
    fi
    
    echo "✅ TOML syntax appears valid: $file"
    return 0
}

# Validate all TOML files
TOML_FILES=(
    "framework-self-testing/hermetic-isolation-test.toml"
    "toml-config/regex-validation-demo.toml"
    "toml-config/rich-assertions-demo.toml"
    "framework-self-testing/validate-all-claims.toml"
    "toml-config/simple-toml-demo.toml"
)

VALID_COUNT=0
TOTAL_COUNT=${#TOML_FILES[@]}

for toml_file in "${TOML_FILES[@]}"; do
    if validate_toml "$toml_file"; then
        ((VALID_COUNT++))
    fi
done

echo ""
echo "📊 TOML Validation Results:"
echo "=========================="
echo "✅ Valid files: $VALID_COUNT/$TOTAL_COUNT"

if [ "$VALID_COUNT" -eq "$TOTAL_COUNT" ]; then
    echo "🎉 All TOML files have valid syntax!"
    exit 0
else
    echo "⚠️  Some TOML files have syntax issues"
    exit 1
fi
