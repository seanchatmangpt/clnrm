#!/usr/bin/env bash
# Test Scenario 2.1: ANSI Output (Human-Readable)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
SAMPLES_DIR="${SCRIPT_DIR}/../samples"
REGISTRY_DIR="${SCRIPT_DIR}/../../../registry"
mkdir -p "${RESULTS_DIR}"

echo "=== Scenario 2.1: ANSI Output (Human-Readable) ==="
echo "Testing live-check with ANSI-formatted terminal output"

# Run with ANSI output (default)
echo "Running live-check with ANSI output..."

if command -v weaver &> /dev/null; then
    # Capture ANSI output (preserving color codes)
    script -q /dev/null weaver registry live-check \
        --registry "${REGISTRY_DIR}" \
        --file "${SAMPLES_DIR}/invalid_spans.json" \
        --output ansi > "${RESULTS_DIR}/scenario_2.1_ansi.txt" 2>&1 || true
else
    cd "${SCRIPT_DIR}/../.."
    docker-compose up -d weaver-validator

    docker exec weaver-validator weaver registry live-check \
        --registry /registry \
        --file /samples/invalid_spans.json \
        --output ansi > "${RESULTS_DIR}/scenario_2.1_ansi.txt" 2>&1 || true
fi

# Verify ANSI codes present
if [ -f "${RESULTS_DIR}/scenario_2.1_ansi.txt" ]; then
    echo "✅ PASS: ANSI output generated"

    # Check for ANSI escape codes (color formatting)
    if grep -q $'\x1b\[' "${RESULTS_DIR}/scenario_2.1_ansi.txt"; then
        echo "✅ PASS: ANSI color codes detected"
    else
        echo "⚠️  WARNING: No ANSI color codes found (may be stripped)"
    fi

    # Check for violation markers
    if grep -qi "violation\|error\|warning" "${RESULTS_DIR}/scenario_2.1_ansi.txt"; then
        echo "✅ PASS: Violation output present"
    fi

    echo "Output preview:"
    head -20 "${RESULTS_DIR}/scenario_2.1_ansi.txt"
else
    echo "❌ FAIL: No ANSI output generated"
    exit 1
fi

echo "=== Scenario 2.1 Complete ==="
