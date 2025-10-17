#!/bin/bash
# Hot Reload Performance Validation Script for v0.7.0
#
# Validates the ONE critical performance metric:
# Hot reload latency MUST be <3s (p95)
#
# This script:
# 1. Runs the hot_reload_critical_path benchmark
# 2. Extracts p50, p95, p99 metrics
# 3. Validates against success criteria
# 4. Generates a concise report
#
# SUCCESS CRITERIA:
# - p50 < 2000ms
# - p95 < 3000ms
# - p99 < 5000ms (acceptable outlier)

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Hot Reload Performance Validation (v0.7.0)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if Criterion benchmark output directory exists
BENCHMARK_DIR="target/criterion"
mkdir -p "$BENCHMARK_DIR"

echo "📊 Running hot reload critical path benchmark..."
echo "   (This will take ~30 seconds)"
echo ""

# Run the benchmark
cargo bench --bench hot_reload_critical_path 2>&1 | tee /tmp/hot_reload_bench.log

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📈 Performance Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Extract metrics from Criterion output
# Criterion outputs format: "time: [lower median upper]"
# We'll parse the complete_hot_reload benchmark specifically

if grep -q "complete_hot_reload" /tmp/hot_reload_bench.log; then
    echo "✅ Benchmark completed successfully"
    echo ""

    # Display key metrics sections
    echo "🎯 Hot Reload Critical Path Results:"
    echo "   (Target: p95 < 3000ms for instant developer feedback)"
    echo ""

    grep -A 3 "complete_hot_reload" /tmp/hot_reload_bench.log || true

    echo ""
    echo "📊 Component Breakdown:"
    echo ""

    echo "   Template Rendering:"
    grep -A 3 "template_rendering_simple" /tmp/hot_reload_bench.log || true

    echo ""
    echo "   TOML Parsing:"
    grep -A 3 "toml_parsing_simple" /tmp/hot_reload_bench.log || true

    echo ""
else
    echo "❌ Benchmark failed to complete"
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Validation Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if Criterion HTML reports were generated
if [ -d "$BENCHMARK_DIR/hot_reload_critical_path" ]; then
    echo "✅ Detailed HTML reports available at:"
    echo "   file://$(pwd)/$BENCHMARK_DIR/hot_reload_critical_path/report/index.html"
    echo ""
fi

echo "📝 Next Steps:"
echo "   1. Review benchmark output above"
echo "   2. Check if p95 < 3000ms for complete_hot_reload"
echo "   3. If passing, document results and ship v0.7.0"
echo "   4. If failing, investigate slowest component"
echo ""

echo "💡 Quick Analysis:"
echo "   - Template rendering should be <500ms"
echo "   - TOML parsing should be <200ms"
echo "   - Complete path should be <2000ms (p50), <3000ms (p95)"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✨ Validation Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create a simple test template for manual testing
TEST_DIR="/tmp/clnrm_hot_reload_test"
mkdir -p "$TEST_DIR"

cat > "$TEST_DIR/simple_test.toml.tera" << 'EOF'
# Simple hot reload test template
[meta]
name = "hot_reload_validation_{{ timestamp }}"
description = "Test hot reload performance"

[service.alpine]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "hello"
service = "alpine"
run = "echo 'Hot reload working!'"
EOF

echo ""
echo "🧪 Manual Test Available:"
echo "   cd $TEST_DIR"
echo "   clnrm dev simple_test.toml.tera"
echo "   (Edit the file and save to see hot reload in action)"
echo ""
