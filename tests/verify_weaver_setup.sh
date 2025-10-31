#!/bin/bash
# Verification script for Weaver setup
# Checks that all necessary files and structure are in place

set -e

echo "🔍 Verifying Weaver code generation setup..."

# Check template directory structure
echo "✓ Checking template directory..."
test -d templates/registry/rust || { echo "❌ Template directory missing"; exit 1; }
test -f templates/registry/rust/weaver.yaml || { echo "❌ weaver.yaml missing"; exit 1; }
test -f templates/registry/rust/spans.rs.j2 || { echo "❌ spans.rs.j2 missing"; exit 1; }
test -f templates/registry/rust/metrics.rs.j2 || { echo "❌ metrics.rs.j2 missing"; exit 1; }
test -f templates/registry/rust/mocks.rs.j2 || { echo "❌ mocks.rs.j2 missing"; exit 1; }
test -f templates/registry/rust/events.rs.j2 || { echo "❌ events.rs.j2 missing"; exit 1; }

# Check generated code directory
echo "✓ Checking generated code directory..."
test -d crates/clnrm-core/src/telemetry/generated || { echo "❌ Generated directory missing"; exit 1; }
test -f crates/clnrm-core/src/telemetry/generated/mod.rs || { echo "❌ generated/mod.rs missing"; exit 1; }

# Check build script
echo "✓ Checking build script..."
test -f build.rs || { echo "❌ build.rs missing"; exit 1; }
grep -q "weaver" build.rs && grep -q "registry" build.rs || { echo "❌ build.rs missing weaver generation"; exit 1; }

# Check integration in telemetry.rs
echo "✓ Checking telemetry.rs integration..."
grep -q "pub mod generated;" crates/clnrm-core/src/telemetry.rs || { echo "❌ telemetry.rs missing generated module"; exit 1; }

# Check documentation
echo "✓ Checking documentation..."
test -f docs/WEAVER_CODEGEN_GUIDE.md || { echo "❌ WEAVER_CODEGEN_GUIDE.md missing"; exit 1; }
test -f docs/USAGE_EXAMPLES.md || { echo "❌ USAGE_EXAMPLES.md missing"; exit 1; }
test -f docs/GENERATOR_CODER_STATUS.md || { echo "❌ GENERATOR_CODER_STATUS.md missing"; exit 1; }

# Check if weaver is installed (optional)
if command -v weaver &> /dev/null; then
    echo "✓ Weaver CLI installed: $(weaver --version)"
else
    echo "⚠️  Weaver CLI not installed (optional for development)"
    echo "   Install with: cargo install weaver-cli"
fi

# Check for registry directory (will be created by Schema-Architect)
if test -d registry/; then
    echo "✓ Registry directory exists (schemas available)"
else
    echo "⏳ Registry directory not yet created (waiting on Schema-Architect)"
fi

echo ""
echo "✅ Weaver setup verification complete!"
echo ""
echo "📋 Status:"
echo "   ✓ Templates ready"
echo "   ✓ Build infrastructure in place"
echo "   ✓ Documentation complete"
echo "   ✓ Integration configured"
if test -d registry/; then
    echo "   ✓ Schemas available - ready to generate"
else
    echo "   ⏳ Schemas pending - waiting on Schema-Architect"
fi
echo ""
echo "🚀 Next steps:"
if test -d registry/; then
    echo "   Run: weaver registry generate rust \\"
    echo "        --registry registry/ \\"
    echo "        --templates templates/registry/rust/ \\"
    echo "        --output crates/clnrm-core/src/telemetry/generated/"
else
    echo "   Wait for Schema-Architect to create semantic convention schemas"
    echo "   Once schemas available, code generation will run automatically via build.rs"
fi
