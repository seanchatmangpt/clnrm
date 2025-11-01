#!/usr/bin/env bash
# Performance profiling script for clnrm v1.4.0+
# Agent 8: Performance Profiler

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROFILE_DIR="$PROJECT_ROOT/target/profiling"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Usage
usage() {
    cat <<EOF
Performance Profiling Script for clnrm

USAGE:
    ./scripts/profile_performance.sh [OPTIONS] <PROFILE_TYPE>

PROFILE TYPES:
    cpu              CPU profiling with flamegraph
    memory           Memory allocation profiling
    benchmarks       Run all benchmarks with timing
    otel-bottleneck  Profile OTEL span emission bottleneck
    container-pool   Profile container pool hot paths
    all              Run all profiling types

OPTIONS:
    -h, --help       Show this help message
    -o, --output     Output directory (default: target/profiling)
    -b, --bench      Specific benchmark to profile
    -d, --duration   Profile duration in seconds (default: 30)

EXAMPLES:
    # Profile CPU hot paths
    ./scripts/profile_performance.sh cpu

    # Profile OTEL bottleneck specifically
    ./scripts/profile_performance.sh otel-bottleneck

    # Profile memory allocations
    ./scripts/profile_performance.sh memory

    # Run all benchmarks
    ./scripts/profile_performance.sh benchmarks

REQUIREMENTS:
    - Rust 1.70+
    - flamegraph: cargo install flamegraph
    - (Linux) perf: apt-get install linux-tools-generic
    - (macOS) Xcode Command Line Tools

OUTPUTS:
    Profiling data saved to: target/profiling/
    - flamegraph.svg      - CPU flamegraph
    - allocations.txt     - Memory allocation analysis
    - benchmark_results/  - Criterion benchmark results

EOF
    exit 0
}

# Check dependencies
check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"

    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}ERROR: cargo not found. Install Rust: https://rustup.rs/${NC}"
        exit 1
    fi

    if ! cargo flamegraph --help &> /dev/null 2>&1; then
        echo -e "${YELLOW}WARNING: flamegraph not installed. Install with: cargo install flamegraph${NC}"
        FLAMEGRAPH_AVAILABLE=false
    else
        FLAMEGRAPH_AVAILABLE=true
    fi

    if [[ "$OSTYPE" == "darwin"* ]]; then
        if ! command -v xcrun &> /dev/null; then
            echo -e "${YELLOW}WARNING: Xcode tools not available. Some profiling features disabled.${NC}"
        fi
        OS="macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if ! command -v perf &> /dev/null; then
            echo -e "${YELLOW}WARNING: perf not installed. Install with: sudo apt-get install linux-tools-generic${NC}"
        fi
        OS="linux"
    else
        echo -e "${YELLOW}WARNING: Unsupported OS. Some features may not work.${NC}"
        OS="unknown"
    fi

    echo -e "${GREEN}✓ Dependencies checked${NC}"
}

# Setup profiling directory
setup() {
    mkdir -p "$PROFILE_DIR"
    mkdir -p "$PROFILE_DIR/benchmark_results"
    mkdir -p "$PROFILE_DIR/flamegraphs"
    echo -e "${GREEN}✓ Created profiling directory: $PROFILE_DIR${NC}"
}

# Build release binary
build_release() {
    echo -e "${BLUE}Building release binary with profiling symbols...${NC}"
    cd "$PROJECT_ROOT"

    RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release --features otel

    echo -e "${GREEN}✓ Release binary built with profiling symbols${NC}"
}

# CPU profiling with flamegraph
profile_cpu() {
    echo -e "${BLUE}Profiling CPU with flamegraph...${NC}"

    if [[ "$FLAMEGRAPH_AVAILABLE" != "true" ]]; then
        echo -e "${RED}ERROR: flamegraph not installed. Run: cargo install flamegraph${NC}"
        exit 1
    fi

    cd "$PROJECT_ROOT"

    # Profile container pool hot paths
    echo -e "${YELLOW}Profiling container pool benchmarks...${NC}"
    cargo flamegraph \
        --output="$PROFILE_DIR/flamegraphs/container_pool.svg" \
        --bench stress_capacity_benchmarks \
        -- --bench "incremental_container_load/containers/100"

    echo -e "${GREEN}✓ Flamegraph saved: $PROFILE_DIR/flamegraphs/container_pool.svg${NC}"

    # Profile OTEL span emission
    echo -e "${YELLOW}Profiling OTEL span emission...${NC}"
    cargo flamegraph \
        --output="$PROFILE_DIR/flamegraphs/otel_spans.svg" \
        --bench stress_capacity_benchmarks \
        -- --bench "otel_span_capacity/spans/1000"

    echo -e "${GREEN}✓ Flamegraph saved: $PROFILE_DIR/flamegraphs/otel_spans.svg${NC}"
}

# OTEL bottleneck profiling
profile_otel_bottleneck() {
    echo -e "${BLUE}Profiling OTEL span emission bottleneck...${NC}"

    cd "$PROJECT_ROOT"

    # Run focused benchmark with detailed output
    echo -e "${YELLOW}Running OTEL span capacity benchmarks...${NC}"
    cargo bench --bench stress_capacity_benchmarks -- \
        --bench "otel_span_capacity" \
        --verbose \
        | tee "$PROFILE_DIR/otel_bottleneck_analysis.txt"

    if [[ "$FLAMEGRAPH_AVAILABLE" == "true" ]]; then
        echo -e "${YELLOW}Generating flamegraph for 1K spans...${NC}"
        cargo flamegraph \
            --output="$PROFILE_DIR/flamegraphs/otel_1k_spans.svg" \
            --bench stress_capacity_benchmarks \
            -- --bench "otel_span_capacity/spans/1000"

        echo -e "${YELLOW}Generating flamegraph for 10K spans...${NC}"
        cargo flamegraph \
            --output="$PROFILE_DIR/flamegraphs/otel_10k_spans.svg" \
            --bench stress_capacity_benchmarks \
            -- --bench "otel_span_capacity/spans/10000"
    fi

    echo -e "${GREEN}✓ OTEL bottleneck profiling complete${NC}"
    echo -e "${BLUE}Compare flamegraphs:${NC}"
    echo -e "  1K spans:  $PROFILE_DIR/flamegraphs/otel_1k_spans.svg"
    echo -e "  10K spans: $PROFILE_DIR/flamegraphs/otel_10k_spans.svg"
    echo -e "${BLUE}Look for functions with increasing % time at higher span counts${NC}"
}

# Memory profiling
profile_memory() {
    echo -e "${BLUE}Profiling memory allocations...${NC}"

    cd "$PROJECT_ROOT"

    if [[ "$OS" == "linux" ]]; then
        echo -e "${YELLOW}Using valgrind massif...${NC}"
        valgrind --tool=massif \
            --massif-out-file="$PROFILE_DIR/massif.out" \
            target/release/clnrm run tests/

        ms_print "$PROFILE_DIR/massif.out" > "$PROFILE_DIR/memory_profile.txt"
        echo -e "${GREEN}✓ Memory profile saved: $PROFILE_DIR/memory_profile.txt${NC}"

    elif [[ "$OS" == "macos" ]]; then
        echo -e "${YELLOW}Using Instruments allocations template...${NC}"
        xcrun xctrace record \
            --template 'Allocations' \
            --output "$PROFILE_DIR/allocations.trace" \
            --launch -- target/release/clnrm run tests/

        echo -e "${GREEN}✓ Allocations trace saved: $PROFILE_DIR/allocations.trace${NC}"
        echo -e "${BLUE}Open with: open $PROFILE_DIR/allocations.trace${NC}"
    else
        echo -e "${RED}Memory profiling not supported on this platform${NC}"
    fi
}

# Container pool profiling
profile_container_pool() {
    echo -e "${BLUE}Profiling container pool performance...${NC}"

    cd "$PROJECT_ROOT"

    # Run container pool benchmarks
    echo -e "${YELLOW}Running incremental container load benchmarks...${NC}"
    cargo bench --bench stress_capacity_benchmarks -- \
        --bench "incremental_container_load" \
        | tee "$PROFILE_DIR/container_pool_benchmarks.txt"

    # Generate flamegraph for 100 containers (typical load)
    if [[ "$FLAMEGRAPH_AVAILABLE" == "true" ]]; then
        echo -e "${YELLOW}Generating flamegraph for container pool (100 containers)...${NC}"
        cargo flamegraph \
            --output="$PROFILE_DIR/flamegraphs/pool_100_containers.svg" \
            --bench stress_capacity_benchmarks \
            -- --bench "incremental_container_load/containers/100"
    fi

    echo -e "${GREEN}✓ Container pool profiling complete${NC}"
}

# Run all benchmarks
run_benchmarks() {
    echo -e "${BLUE}Running all benchmarks...${NC}"

    cd "$PROJECT_ROOT"

    # Hot reload benchmarks
    echo -e "${YELLOW}Running hot reload benchmarks...${NC}"
    cargo bench --bench hot_reload_critical_path \
        | tee "$PROFILE_DIR/benchmark_results/hot_reload.txt"

    # Stress capacity benchmarks
    echo -e "${YELLOW}Running stress capacity benchmarks (this may take 5-10 minutes)...${NC}"
    cargo bench --bench stress_capacity_benchmarks \
        | tee "$PROFILE_DIR/benchmark_results/stress_capacity.txt"

    echo -e "${GREEN}✓ All benchmarks complete${NC}"
    echo -e "${BLUE}Results saved to: $PROFILE_DIR/benchmark_results/${NC}"
}

# Main profiling logic
main() {
    PROFILE_TYPE="${1:-}"

    if [[ "$PROFILE_TYPE" == "-h" || "$PROFILE_TYPE" == "--help" ]]; then
        usage
    fi

    if [[ -z "$PROFILE_TYPE" ]]; then
        echo -e "${RED}ERROR: Profile type required${NC}"
        usage
    fi

    check_dependencies
    setup
    build_release

    case "$PROFILE_TYPE" in
        cpu)
            profile_cpu
            ;;
        memory)
            profile_memory
            ;;
        benchmarks)
            run_benchmarks
            ;;
        otel-bottleneck)
            profile_otel_bottleneck
            ;;
        container-pool)
            profile_container_pool
            ;;
        all)
            run_benchmarks
            profile_cpu
            profile_otel_bottleneck
            profile_container_pool
            # profile_memory  # Commented out - takes very long
            ;;
        *)
            echo -e "${RED}ERROR: Unknown profile type: $PROFILE_TYPE${NC}"
            usage
            ;;
    esac

    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ Profiling complete!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Profiling data location: $PROFILE_DIR${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo -e "  1. Review flamegraphs: open $PROFILE_DIR/flamegraphs/*.svg"
    echo -e "  2. Analyze benchmark results: cat $PROFILE_DIR/benchmark_results/*.txt"
    echo -e "  3. See full report: $PROJECT_ROOT/docs/PERFORMANCE_PROFILING_REPORT.md"
    echo ""
}

# Run main
main "$@"
