#!/bin/bash
# Test script for clnrm v2 CLI generation
# Tests the complete generation workflow: template -> RDF -> multi-file output

set -e

cd "$(dirname "$0")/.."

echo "🧪 Testing clnrm v2 CLI generation workflow..."
echo ""

# Check if ggen is available
if ! command -v ggen &> /dev/null; then
    echo "❌ ggen not found. Building ggen..."
    cd /Users/sac/ggen
    cargo build --release --bin ggen
    export PATH="/Users/sac/ggen/target/release:$PATH"
    cd /Users/sac/clnrm
fi

echo "✅ ggen found: $(ggen --version 2>&1 | head -1)"
echo ""

# Test 1: Generate CLI from RDF
echo "📋 Test 1: Generate CLI from RDF template"
echo "Command: ggen template generate -t templates/cli-v2/cli-stack.tmpl -r docs/clnrm-cli-v2.ttl -o crates/clnrm-v2-generated -f"

ggen template generate \
    -t templates/cli-v2/cli-stack.tmpl \
    -r docs/clnrm-cli-v2.ttl \
    -o crates/clnrm-v2-generated \
    -f \
    2>&1

if [ $? -eq 0 ]; then
    echo "✅ Test 1 passed: CLI generation succeeded"
    echo ""
    
    # Check generated files
    echo "📁 Generated files:"
    find crates/clnrm-v2-generated -type f | head -10
    echo ""
    
    # Count files
    file_count=$(find crates/clnrm-v2-generated -type f | wc -l | tr -d ' ')
    echo "📊 Total files generated: $file_count"
else
    echo "❌ Test 1 failed: CLI generation failed"
    exit 1
fi

echo ""

# Test 2: Generate Weaver config
echo "📋 Test 2: Generate Weaver config from RDF"
echo "Command: ggen template generate -t ~/ggen/templates/clnrm/weaver-config.tmpl -r docs/clnrm-cli-v2.ttl -o weaver.toml -f"

ggen template generate \
    -t /Users/sac/ggen/templates/clnrm/weaver-config.tmpl \
    -r docs/clnrm-cli-v2.ttl \
    -o weaver.toml \
    -f \
    2>&1

if [ $? -eq 0 ]; then
    echo "✅ Test 2 passed: Weaver config generation succeeded"
    if [ -f "weaver.toml" ]; then
        echo "   📄 Generated: weaver.toml"
    fi
else
    echo "⚠️  Test 2 warning: Weaver config generation failed (template may need adjustments)"
fi

echo ""

# Test 3: Generate Weaver registry
echo "📋 Test 3: Generate Weaver registry from RDF"
echo "Command: ggen template generate -t ~/ggen/templates/clnrm/weaver-registry.tmpl -r docs/clnrm-cli-v2.ttl -o registry -f"

ggen template generate \
    -t /Users/sac/ggen/templates/clnrm/weaver-registry.tmpl \
    -r docs/clnrm-cli-v2.ttl \
    -o registry \
    -f \
    2>&1

if [ $? -eq 0 ]; then
    echo "✅ Test 3 passed: Weaver registry generation succeeded"
    if [ -d "registry" ]; then
        echo "   📁 Generated: registry/"
        find registry -type f | head -10
    fi
else
    echo "⚠️  Test 3 warning: Weaver registry generation failed (template may need adjustments)"
fi

echo ""
echo "✨ All tests completed!"
echo ""
echo "Next steps:"
echo "  1. Review generated CLI in: crates/clnrm-v2-generated/"
echo "  2. Verify Weaver config: weaver.toml"
echo "  3. Verify Weaver registry: registry/"
echo "  4. Test compilation: cd crates/clnrm-v2-generated && cargo build"

