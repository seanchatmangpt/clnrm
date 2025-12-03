#!/bin/bash
# Test script to generate CLI from RDF template using ggen

set -e

cd "$(dirname "$0")/.."

echo "🔍 Checking ggen installation..."
if ! command -v ggen &> /dev/null; then
    echo "❌ ggen not found. Install with: cargo install ggen"
    exit 1
fi

echo "✅ ggen found: $(ggen --version)"

echo ""
echo "📋 Testing template syntax..."
if ! ggen template lint templates/cli-v2/cli-stack.tmpl 2>&1; then
    echo "⚠️  Template linting failed, but continuing..."
fi

echo ""
echo "🧪 Attempting to generate CLI..."
echo "Command: ggen template generate -t templates/cli-v2/cli-stack.tmpl -r docs/clnrm-cli-v2.ttl -o crates/clnrm-v2-generated-test --force"

# Check if ggen has template generate command
if ggen template --help 2>&1 | grep -q "generate"; then
    ggen template generate \
        -t templates/cli-v2/cli-stack.tmpl \
        -r docs/clnrm-cli-v2.ttl \
        -o crates/clnrm-v2-generated-test \
        --force \
        2>&1 || echo "Generation failed - checking error"
else
    echo "❌ 'ggen template generate' command not available"
    echo "Available template commands:"
    ggen template --help
    exit 1
fi

if [ -d "crates/clnrm-v2-generated-test" ]; then
    echo ""
    echo "✅ Generated files found:"
    find crates/clnrm-v2-generated-test -type f | head -10
else
    echo ""
    echo "⚠️  Output directory not found - generation may have failed"
fi

