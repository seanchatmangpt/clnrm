#!/bin/bash
# Weaver Advisor Validation Runner
# Usage: ./run_validation.sh [test_data_file]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Weaver Advisor Validation"
echo "=============================="
echo ""

# Check if Weaver is installed
if ! command -v weaver &> /dev/null; then
    echo "❌ Weaver not found. Install with: cargo install weaver-cli"
    exit 1
fi

echo "✅ Weaver version: $(weaver --version)"
echo ""

# Validate registry first
echo "📋 Step 1: Validating registry schemas..."
if weaver registry check -r "$PROJECT_ROOT/registry/" 2>&1 | grep -q "✔"; then
    echo "✅ Registry validation passed"
else
    echo "❌ Registry validation failed"
    exit 1
fi
echo ""

# Start Weaver live-check in background
echo "🚀 Step 2: Starting Weaver live-check with custom policies..."
WEAVER_PID=""

# Use provided test data or OTLP
if [ -n "$1" ]; then
    TEST_FILE="$1"
    if [ ! -f "$TEST_FILE" ]; then
        echo "❌ Test file not found: $TEST_FILE"
        exit 1
    fi

    echo "📊 Running live-check on test file: $TEST_FILE"
    weaver registry live-check \
        --registry "$PROJECT_ROOT/registry/" \
        --advice-policies "$SCRIPT_DIR/custom-policies/" \
        --input-source "$TEST_FILE" \
        --input-format json \
        --no-stream \
        --format ansi
else
    echo "🎧 Starting OTLP listener on port 4317..."
    echo "   (Send telemetry to http://localhost:4317)"
    echo ""
    echo "   In another terminal, run:"
    echo "   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo test --features otel"
    echo ""
    echo "   Press Ctrl+C to stop"
    echo ""

    weaver registry live-check \
        --registry "$PROJECT_ROOT/registry/" \
        --advice-policies "$SCRIPT_DIR/custom-policies/" \
        --otlp-grpc-port 4317 \
        --inactivity-timeout 300 \
        --format ansi

    echo ""
    echo "✅ Validation complete"
fi
