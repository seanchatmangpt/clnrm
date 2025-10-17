#!/bin/bash
# Flamegraph Profiling for v0.7.0 DX Features
#
# This script generates flamegraphs for performance profiling and bottleneck identification.
#
# Requirements:
# - cargo-flamegraph: cargo install flamegraph
# - perf (Linux) or dtrace (macOS)
#
# Usage:
#   ./flamegraph_profiling.sh [target]
#
# Targets:
#   - hot_reload: Profile hot reload workflow
#   - template_render: Profile template rendering
#   - toml_parse: Profile TOML parsing
#   - all: Profile all targets (default)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../../" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/flamegraphs"

mkdir -p "$OUTPUT_DIR"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Flamegraph Profiling for v0.7.0 DX Features              ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

cd "$PROJECT_ROOT"

# Check for flamegraph tool
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "⚠️  cargo-flamegraph not found. Installing..."
    cargo install flamegraph
fi

TARGET="${1:-all}"

# Function to profile with flamegraph
profile_target() {
    local name=$1
    local bench_name=$2

    echo "🔥 Profiling: $name"
    echo "   Benchmark: $bench_name"
    echo "   Output: $OUTPUT_DIR/${name}.svg"

    cargo flamegraph \
        --bench dx_features_benchmarks \
        --output "$OUTPUT_DIR/${name}.svg" \
        -- --bench "$bench_name" \
        > /dev/null 2>&1 || echo "   ⚠️  Profiling completed with warnings"

    if [ -f "$OUTPUT_DIR/${name}.svg" ]; then
        echo "   ✓ Flamegraph generated"
        echo ""
    else
        echo "   ✗ Flamegraph generation failed"
        echo ""
        return 1
    fi
}

# Profile targets based on argument
case "$TARGET" in
    hot_reload)
        profile_target "hot_reload_workflow" "hot_reload_workflow"
        ;;

    template_render)
        profile_target "template_rendering_simple" "template_rendering/simple_template"
        profile_target "template_rendering_complex" "template_rendering/complex_template"
        ;;

    toml_parse)
        profile_target "toml_parsing_simple" "toml_parsing/simple_toml"
        profile_target "toml_parsing_large" "toml_parsing/large_toml"
        ;;

    scalability)
        profile_target "scalability_100_files" "scalability/100_files"
        ;;

    all)
        echo "Profiling all targets..."
        echo ""

        profile_target "hot_reload_workflow" "hot_reload_workflow"
        profile_target "template_rendering_simple" "template_rendering/simple_template"
        profile_target "template_rendering_complex" "template_rendering/complex_template"
        profile_target "toml_parsing_large" "toml_parsing/large_toml"
        profile_target "scalability_100_files" "scalability/100_files"
        ;;

    *)
        echo "❌ Unknown target: $TARGET"
        echo ""
        echo "Available targets:"
        echo "  - hot_reload"
        echo "  - template_render"
        echo "  - toml_parse"
        echo "  - scalability"
        echo "  - all (default)"
        exit 1
        ;;
esac

echo "═══════════════════════════════════════════════════════════"
echo "Flamegraphs generated in: $OUTPUT_DIR"
echo ""
echo "View flamegraphs:"
echo "  - Open .svg files in a web browser"
echo "  - Look for hot paths (wider bars = more time)"
echo "  - Identify optimization opportunities"
echo ""
echo "Common bottlenecks to look for:"
echo "  - Tera template compilation"
echo "  - TOML parsing/deserialization"
echo "  - File I/O operations"
echo "  - String allocations"
echo "  - Regex compilation"
echo "═══════════════════════════════════════════════════════════"
