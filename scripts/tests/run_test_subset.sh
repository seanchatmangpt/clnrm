#!/bin/bash
# run_test_subset.sh
# Run specific subsets of live-check tests for faster iteration

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAIN_SCRIPT="$SCRIPT_DIR/test_live_check_comprehensive.sh"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTION]

Run specific subsets of Weaver live-check tests for faster iteration.

Options:
    --basic         Run basic tests (file, stdin, output formats)
    --advanced      Run advanced tests (timeout, sighup, policies)
    --concurrent    Run concurrent and OTLP tests
    --quick         Run quick smoke tests only
    --all           Run all tests (default)
    --list          List available tests
    -h, --help      Show this help message

Examples:
    $(basename "$0") --basic           # Fast feedback loop
    $(basename "$0") --advanced        # Complex scenarios
    $(basename "$0") --quick           # Fastest validation
    $(basename "$0") --all             # Complete suite

Test Categories:
    Basic (fast):
        - 01_file_json
        - 02_stdin_text
        - 03_json_output
        - 04_ansi_output

    Advanced (medium):
        - 05_inactivity_timeout
        - 06_sighup_stop
        - 07_custom_policies
        - 08_statistics

    Concurrent (slow):
        - 09_concurrent_instances
        - 10_otlp_grpc

    Quick (fastest):
        - 01_file_json
        - 03_json_output
EOF
}

list_tests() {
    echo -e "${BLUE}Available Tests:${NC}"
    echo ""
    echo "  01_file_json               - File input JSON"
    echo "  02_stdin_text              - stdin text input"
    echo "  03_json_output             - JSON output format"
    echo "  04_ansi_output             - ANSI output format"
    echo "  05_inactivity_timeout      - Inactivity timeout"
    echo "  06_sighup_stop             - SIGHUP graceful stop"
    echo "  07_custom_policies         - Custom OPA policies"
    echo "  08_statistics              - Statistics generation"
    echo "  09_concurrent_instances    - Concurrent instances"
    echo "  10_otlp_grpc               - OTLP gRPC input"
    echo ""
}

# Extract and run specific test from main script
run_specific_test() {
    local test_name=$1

    # Source the main script functions
    source "$MAIN_SCRIPT"

    # Run just this test
    check_weaver
    check_registry
    run_test "$test_name" "test_${test_name#*_}"
}

run_basic_tests() {
    echo -e "${BLUE}Running basic tests...${NC}"
    source "$MAIN_SCRIPT"
    check_weaver
    check_registry

    run_test "01_file_json" test_file_json
    run_test "02_stdin_text" test_stdin_text
    run_test "03_json_output" test_json_output
    run_test "04_ansi_output" test_ansi_output

    print_summary
}

run_advanced_tests() {
    echo -e "${BLUE}Running advanced tests...${NC}"
    source "$MAIN_SCRIPT"
    check_weaver
    check_registry

    run_test "05_inactivity_timeout" test_inactivity_timeout
    run_test "06_sighup_stop" test_sighup_stop
    run_test "07_custom_policies" test_custom_policies
    run_test "08_statistics" test_statistics

    print_summary
}

run_concurrent_tests() {
    echo -e "${BLUE}Running concurrent tests...${NC}"
    source "$MAIN_SCRIPT"
    check_weaver
    check_registry

    run_test "09_concurrent_instances" test_concurrent_instances
    run_test "10_otlp_grpc" test_otlp_grpc || SKIPPED+=("10_otlp_grpc")

    print_summary
}

run_quick_tests() {
    echo -e "${BLUE}Running quick smoke tests...${NC}"
    source "$MAIN_SCRIPT"
    check_weaver
    check_registry

    run_test "01_file_json" test_file_json
    run_test "03_json_output" test_json_output

    print_summary
}

print_summary() {
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}Quick Summary${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${GREEN}Passed:  ${#PASSED[@]}${NC}"
    echo -e "${RED}Failed:  ${#FAILED[@]}${NC}"
    echo -e "${YELLOW}Skipped: ${#SKIPPED[@]}${NC}"

    if [ ${#FAILED[@]} -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# Parse arguments
case "${1:-}" in
    --basic)
        run_basic_tests
        ;;
    --advanced)
        run_advanced_tests
        ;;
    --concurrent)
        run_concurrent_tests
        ;;
    --quick)
        run_quick_tests
        ;;
    --all)
        exec "$MAIN_SCRIPT"
        ;;
    --list)
        list_tests
        ;;
    -h|--help)
        usage
        ;;
    "")
        exec "$MAIN_SCRIPT"
        ;;
    *)
        echo "Unknown option: $1"
        echo ""
        usage
        exit 1
        ;;
esac
