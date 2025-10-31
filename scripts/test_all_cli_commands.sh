#!/bin/bash
# Comprehensive CLI Reality Check - Test ALL 23 commands

set -e

BINARY="./target/release/clnrm"
RESULTS_FILE="/tmp/cli_test_results.txt"
TEMP_DIR="/tmp/clnrm_test_$$"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

mkdir -p "$TEMP_DIR"
cd "$TEMP_DIR"

echo "=== CLI Reality Check - Testing All 23 Commands ===" > "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Helper function to test a command
test_command() {
    local cmd_name="$1"
    local cmd_args="$2"
    local check_otel="$3"

    echo "Testing: $cmd_name" >&2

    # Test if command works
    local works="NO"
    local has_otel_flags="NO"
    local emits_telemetry="NO"
    local blockers=""

    # Check if --help works
    if $BINARY $cmd_args --help &>/dev/null; then
        works="YES"

        # Check if OTEL flags exist
        if $BINARY $cmd_args --help 2>&1 | grep -q "otel-exporter"; then
            has_otel_flags="YES"
        fi
    else
        blockers="--help failed"
    fi

    # Try to run the command if it should work
    if [ "$works" = "YES" ] && [ "$check_otel" = "true" ]; then
        # Try with OTEL exporter
        local output
        output=$($BINARY $cmd_args --otel-exporter stdout 2>&1 || true)

        # Check for actual telemetry emission
        if echo "$output" | grep -qi "span\|trace\|SpanData"; then
            emits_telemetry="YES"
        fi

        # Check for errors
        if echo "$output" | grep -qi "error\|failed"; then
            if [ -z "$blockers" ]; then
                blockers=$(echo "$output" | grep -i "error\|failed" | head -1)
            fi
        fi
    fi

    # Output result
    printf "| %-20s | %-8s | %-15s | %-17s | %-40s |\n" \
        "$cmd_name" "$works" "$has_otel_flags" "$emits_telemetry" "$blockers" >> "$RESULTS_FILE"
}

# Write table header
echo "| Command              | Works?   | Has OTEL Flags? | Emits Telemetry? | Blockers                                |" >> "$RESULTS_FILE"
echo "|----------------------|----------|-----------------|------------------|-----------------------------------------|" >> "$RESULTS_FILE"

# Test each command
test_command "run" "run" "false"  # Need test file
test_command "self-test" "self-test" "true"
test_command "init" "init" "false"
test_command "template list" "template list" "false"
test_command "template show" "template show basic" "false"
test_command "validate" "validate" "false"  # Need test file
test_command "plugins" "plugins" "false"
test_command "health" "health" "false"
test_command "services list" "services list" "false"
test_command "services start" "services start" "false"
test_command "services stop" "services stop" "false"
test_command "services status" "services status" "false"
test_command "collector list" "collector list" "false"
test_command "collector start" "collector start" "false"
test_command "collector stop" "collector stop" "false"
test_command "report" "report" "false"  # Need test file
test_command "diff" "diff" "false"  # Need trace files
test_command "spans" "spans" "false"  # Need trace data
test_command "graph" "graph" "false"  # Need trace data
test_command "analyze" "analyze" "false"  # Need test file
test_command "dev" "dev" "false"  # Need test file
test_command "dry-run" "dry-run" "false"  # Need test file
test_command "fmt" "fmt" "false"  # Need test file
test_command "lint" "lint" "false"  # Need test file
test_command "record" "record" "false"  # Need test file
test_command "repro" "repro" "false"  # Need test file
test_command "red-green" "red-green" "false"  # Need test file
test_command "pull" "pull alpine:latest" "false"
test_command "render" "render" "false"  # Need template

cat "$RESULTS_FILE"
