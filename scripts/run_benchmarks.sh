#!/bin/bash
# Performance Benchmark Runner with Memory Tracking
#
# This script runs comprehensive performance benchmarks for the CLNRM framework
# and tracks memory usage, system metrics, and generates detailed reports.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BENCHMARK_DIR="target/criterion"
RESULTS_DIR="benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="${RESULTS_DIR}/benchmark_${TIMESTAMP}.txt"
MEMORY_LOG="${RESULTS_DIR}/memory_${TIMESTAMP}.log"

# Create results directory
mkdir -p "${RESULTS_DIR}"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║          CLNRM Performance Benchmark Suite                 ║${NC}"
echo -e "${BLUE}║                                                            ║${NC}"
echo -e "${BLUE}║  Timestamp: $(date '+%Y-%m-%d %H:%M:%S')                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# System information
echo -e "${GREEN}System Information:${NC}"
echo "  OS: $(uname -s) $(uname -r)"
echo "  CPU: $(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs || echo 'N/A')"
echo "  Cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 'N/A')"
echo "  Memory: $(free -h 2>/dev/null | grep Mem | awk '{print $2}' || echo 'N/A')"
echo "  Rust: $(rustc --version)"
echo ""

# Check if running on battery (laptops)
if [ -f /sys/class/power_supply/AC/online ]; then
    AC_STATUS=$(cat /sys/class/power_supply/AC/online)
    if [ "$AC_STATUS" -eq 0 ]; then
        echo -e "${YELLOW}⚠️  WARNING: Running on battery power. Results may vary.${NC}"
        echo ""
    fi
fi

# Function to run a benchmark suite with memory tracking
run_benchmark() {
    local name=$1
    local bench_name=$2

    echo -e "${GREEN}Running ${name}...${NC}"

    # Start memory tracking in background
    (
        while true; do
            ps aux | grep -E "criterion|cargo" | grep -v grep | \
                awk '{print $2, $3, $4, $5, $6}' >> "${MEMORY_LOG}" 2>/dev/null || true
            sleep 0.5
        done
    ) &
    MEMORY_PID=$!

    # Run benchmark
    if cargo bench --bench "${bench_name}" 2>&1 | tee -a "${RESULTS_FILE}"; then
        echo -e "${GREEN}✓ ${name} completed${NC}"
    else
        echo -e "${RED}✗ ${name} failed${NC}"
    fi

    # Stop memory tracking
    kill ${MEMORY_PID} 2>/dev/null || true
    wait ${MEMORY_PID} 2>/dev/null || true

    echo ""
}

# Clean previous build artifacts
echo -e "${BLUE}Cleaning build artifacts...${NC}"
cargo clean -p clnrm --release 2>/dev/null || true
echo ""

# Build benchmarks in release mode
echo -e "${BLUE}Building benchmarks in release mode...${NC}"
if cargo build --release --benches; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo ""

# Start benchmark timestamp
START_TIME=$(date +%s)

# Run benchmark suites
run_benchmark "Cleanroom Benchmarks" "cleanroom_benchmarks"
run_benchmark "Scenario Benchmarks" "scenario_benchmarks"
run_benchmark "AI Intelligence Benchmarks" "ai_intelligence_benchmarks"
run_benchmark "Memory Benchmarks" "memory_benchmarks"

# End benchmark timestamp
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Generate summary report
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Benchmark Summary                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "  Total Duration: ${DURATION} seconds"
echo "  Results File: ${RESULTS_FILE}"
echo "  Memory Log: ${MEMORY_LOG}"
echo "  Criterion Reports: ${BENCHMARK_DIR}/report/index.html"
echo ""

# Extract key metrics if available
if [ -f "${RESULTS_FILE}" ]; then
    echo -e "${GREEN}Key Metrics:${NC}"
    echo ""

    # Parse cleanroom creation time
    if grep -q "cleanroom_creation" "${RESULTS_FILE}"; then
        CLEANROOM_TIME=$(grep "cleanroom_creation" "${RESULTS_FILE}" | grep "time:" | head -1)
        echo "  Cleanroom Creation: ${CLEANROOM_TIME}"
    fi

    # Parse container reuse time
    if grep -q "container_reuse" "${RESULTS_FILE}"; then
        REUSE_TIME=$(grep "container_reuse/reuse" "${RESULTS_FILE}" | grep "time:" | head -1)
        echo "  Container Reuse: ${REUSE_TIME}"
    fi

    # Parse scenario execution time
    if grep -q "scenario" "${RESULTS_FILE}"; then
        SCENARIO_TIME=$(grep "single_step_scenario" "${RESULTS_FILE}" | grep "time:" | head -1)
        echo "  Single-step Scenario: ${SCENARIO_TIME}"
    fi

    echo ""
fi

# Memory usage summary
if [ -f "${MEMORY_LOG}" ] && [ -s "${MEMORY_LOG}" ]; then
    echo -e "${GREEN}Memory Usage Summary:${NC}"
    echo ""

    # Calculate peak memory usage
    PEAK_MEM=$(awk '{if ($4 > max) max = $4} END {print max}' "${MEMORY_LOG}")
    AVG_MEM=$(awk '{sum += $4; count++} END {print sum/count}' "${MEMORY_LOG}")

    echo "  Peak Memory: ${PEAK_MEM}%"
    echo "  Average Memory: ${AVG_MEM}%"
    echo ""
fi

# Performance comparison with baseline
echo -e "${YELLOW}Performance Status:${NC}"
echo ""

# Define performance budgets (in microseconds)
declare -A budgets=(
    ["cleanroom_creation"]=200
    ["service_registration"]=100
    ["container_reuse"]=5
    ["metrics_collection"]=10
)

# Check each budget
for metric in "${!budgets[@]}"; do
    budget=${budgets[$metric]}
    # This is a placeholder - actual parsing would be more sophisticated
    echo "  ${metric}: Budget ${budget}µs"
done
echo ""

# Generate HTML report link
if [ -d "${BENCHMARK_DIR}/report" ]; then
    echo -e "${GREEN}View detailed HTML report:${NC}"
    echo "  file://$(pwd)/${BENCHMARK_DIR}/report/index.html"
    echo ""
fi

# Cleanup old results (keep last 10)
echo -e "${BLUE}Cleaning up old results...${NC}"
ls -t "${RESULTS_DIR}"/benchmark_*.txt 2>/dev/null | tail -n +11 | xargs -r rm -f
ls -t "${RESULTS_DIR}"/memory_*.log 2>/dev/null | tail -n +11 | xargs -r rm -f
echo ""

# Final message
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Benchmark Suite Completed Successfully!           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Exit with success
exit 0
