#!/usr/bin/env bash
#
# Local fuzzing script for development
#
# Usage:
#   ./run_local_fuzz.sh [target] [duration]
#
# Examples:
#   ./run_local_fuzz.sh                          # Run all targets for 60s each
#   ./run_local_fuzz.sh fuzz_toml_parser         # Run single target for 60s
#   ./run_local_fuzz.sh fuzz_toml_parser 300     # Run single target for 5min

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
FUZZ_TARGETS=(
    "fuzz_toml_parser"
    "fuzz_scenario_dsl"
    "fuzz_cli_args"
    "fuzz_error_handling"
    "fuzz_regex_patterns"
)

DEFAULT_DURATION=60
MEMORY_LIMIT=2048  # MB
TIMEOUT=5          # seconds per input

# Parse arguments
TARGET="${1:-all}"
DURATION="${2:-$DEFAULT_DURATION}"

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo -e "${RED}Error: cargo-fuzz is not installed${NC}"
    echo "Install with: cargo install cargo-fuzz"
    exit 1
fi

# Check if nightly toolchain is installed
if ! rustup toolchain list | grep -q nightly; then
    echo -e "${RED}Error: Rust nightly toolchain is not installed${NC}"
    echo "Install with: rustup install nightly"
    exit 1
fi

# Function to run a single fuzz target
run_fuzz_target() {
    local target=$1
    local duration=$2

    echo -e "\n${GREEN}=== Running fuzz target: $target ===${NC}"
    echo "Duration: ${duration}s | Memory limit: ${MEMORY_LIMIT}MB | Timeout: ${TIMEOUT}s"

    # Create corpus directory if it doesn't exist
    mkdir -p "corpus/$target"

    # Run fuzzer
    if cargo +nightly fuzz run "$target" -- \
        -max_total_time="$duration" \
        -rss_limit_mb="$MEMORY_LIMIT" \
        -timeout="$TIMEOUT" \
        -print_final_stats=1; then

        echo -e "${GREEN}✓ $target completed successfully${NC}"

        # Minimize corpus
        echo "Minimizing corpus..."
        cargo +nightly fuzz cmin "$target" 2>/dev/null || true

        return 0
    else
        echo -e "${RED}✗ $target found crashes${NC}"

        # Check for artifacts
        if [ -d "artifacts/$target" ] && [ "$(ls -A "artifacts/$target")" ]; then
            echo -e "${YELLOW}Crash artifacts saved to: artifacts/$target${NC}"
            ls -lh "artifacts/$target"
        fi

        return 1
    fi
}

# Main execution
cd "$(dirname "$0")"

echo -e "${GREEN}Starting fuzzing session${NC}"
echo "Target: $TARGET"
echo "Duration: ${DURATION}s per target"
echo

FAILED_TARGETS=()

if [ "$TARGET" = "all" ]; then
    # Run all targets
    for target in "${FUZZ_TARGETS[@]}"; do
        if ! run_fuzz_target "$target" "$DURATION"; then
            FAILED_TARGETS+=("$target")
        fi
    done
else
    # Run specific target
    if [[ ! " ${FUZZ_TARGETS[@]} " =~ " $TARGET " ]]; then
        echo -e "${RED}Error: Unknown target '$TARGET'${NC}"
        echo "Available targets: ${FUZZ_TARGETS[*]}"
        exit 1
    fi

    if ! run_fuzz_target "$TARGET" "$DURATION"; then
        FAILED_TARGETS+=("$TARGET")
    fi
fi

# Summary
echo -e "\n${GREEN}=== Fuzzing Summary ===${NC}"

if [ ${#FAILED_TARGETS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All targets passed without crashes${NC}"
    exit 0
else
    echo -e "${RED}✗ ${#FAILED_TARGETS[@]} target(s) found crashes:${NC}"
    for target in "${FAILED_TARGETS[@]}"; do
        echo -e "  - ${RED}$target${NC}"
    done

    echo -e "\n${YELLOW}Next steps:${NC}"
    echo "1. Review crash artifacts in artifacts/ directory"
    echo "2. Reproduce crashes with: cargo +nightly fuzz run <target> artifacts/<target>/<crash-file>"
    echo "3. Add regression tests to crash_reproduction_tests.rs"
    echo "4. Fix the underlying issues"

    exit 1
fi
