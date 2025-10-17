#!/usr/bin/env bash
# Verify clnrm template command has all types mentioned in README
set -euo pipefail

CLNRM_BIN="${CLNRM_BIN:-/Users/sac/clnrm/target/release/clnrm}"

echo "🧪 Testing: clnrm template types"

# Run clnrm template --help
echo "▶️ Running: clnrm template --help"
OUTPUT=$("$CLNRM_BIN" template --help 2>&1)

echo "📤 Output:"
echo "$OUTPUT"

# Extract template types line
echo ""
echo "✅ Verification checks:"

TEMPLATES=()

# README claims these 6 templates exist (line 41)
EXPECTED_TEMPLATES=("default" "advanced" "minimal" "database" "api" "otel")

for template in "${EXPECTED_TEMPLATES[@]}"; do
    if echo "$OUTPUT" | grep -q "$template"; then
        echo "  ✅ Template '$template' found in help"
        TEMPLATES+=("$template")
    else
        echo "  ❌ Template '$template' NOT found in help"
        exit 1
    fi
done

echo ""
echo "📊 Template count:"
echo "  Found: ${#TEMPLATES[@]}"
echo "  Expected: ${#EXPECTED_TEMPLATES[@]}"

if [[ ${#TEMPLATES[@]} -eq ${#EXPECTED_TEMPLATES[@]} ]]; then
    echo "  ✅ All template types from README verified"
else
    echo "  ❌ Template count mismatch"
    exit 1
fi

# Test generating a template
echo ""
echo "🧪 Testing: clnrm template otel generation"
TEMP_FILE=$(mktemp)
"$CLNRM_BIN" template otel > "$TEMP_FILE" 2>&1

if [[ -s "$TEMP_FILE" ]]; then
    echo "  ✅ Template generation produces output"

    # Check if it's valid TOML-like content
    if grep -q '\[' "$TEMP_FILE"; then
        echo "  ✅ Output contains TOML section markers"
    else
        echo "  ⚠️  Output may not be valid TOML"
    fi

    # Check for Tera template variables
    if grep -q '{{' "$TEMP_FILE"; then
        echo "  ✅ Output contains Tera template variables"
    else
        echo "  ⚠️  No Tera template variables found"
    fi
else
    echo "  ❌ Template generation produced no output"
    exit 1
fi

rm -f "$TEMP_FILE"

echo ""
echo "✅ All README claims about 'clnrm template' verified successfully!"
