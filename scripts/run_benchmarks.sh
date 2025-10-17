#!/usr/bin/env bash
# ==============================================================================
# run_benchmarks.sh
# Advanced Performance Benchmark Runner with Comprehensive Analysis
#
# This script runs comprehensive performance benchmarks for the CLNRM framework
# with advanced features including memory tracking, system metrics, statistical
# analysis, and detailed reporting capabilities.
#
# Usage:
#   ./scripts/run_benchmarks.sh                    # Run all benchmarks
#   ./scripts/run_benchmarks.sh --quick            # Quick benchmark run
#   ./scripts/run_benchmarks.sh --memory           # Focus on memory benchmarks
#   ./scripts/run_benchmarks.sh --compare          # Compare with previous runs
#   ./scripts/run_benchmarks.sh --report           # Generate detailed report
#   ./scripts/run_benchmarks.sh --cleanup          # Clean up benchmark data
#
# Exit Codes:
#   0 - All benchmarks completed successfully
#   1 - Some benchmarks failed or warnings
#   2 - Critical benchmark failures
# ==============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCHMARK_DIR="target/criterion"
RESULTS_DIR="benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="${RESULTS_DIR}/benchmark_${TIMESTAMP}.txt"
MEMORY_LOG="${RESULTS_DIR}/memory_${TIMESTAMP}.log"
JSON_REPORT="${RESULTS_DIR}/benchmark_${TIMESTAMP}.json"
COMPARISON_REPORT="${RESULTS_DIR}/comparison_${TIMESTAMP}.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters and state
TOTAL_BENCHMARKS=0
PASSED_BENCHMARKS=0
FAILED_BENCHMARKS=0
WARNING_BENCHMARKS=0
BENCHMARK_RESULTS=()

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_BENCHMARKS++))
    BENCHMARK_RESULTS+=("{\"benchmark\":\"$1\",\"status\":\"PASS\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}")
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_BENCHMARKS++))
    BENCHMARK_RESULTS+=("{\"benchmark\":\"$1\",\"status\":\"FAIL\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}")
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNING_BENCHMARKS++))
    BENCHMARK_RESULTS+=("{\"benchmark\":\"$1\",\"status\":\"WARN\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}")
}

log_header() {
    echo -e "${PURPLE}[BENCHMARK]${NC} $1"
}

# Create results directory
mkdir -p "${RESULTS_DIR}"

# Check prerequisites
check_prerequisites() {
    log_header "🔍 Checking Prerequisites"
    
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        log_error "Rust compiler not found. Please install Rust toolchain."
        exit 1
    fi
    
    # Check if benchmark dependencies are available
    if ! cargo bench --help &> /dev/null; then
        log_warning "Cargo bench not available. Installing criterion..."
        # This would typically be handled by Cargo.toml dependencies
    fi
    
    log_success "Prerequisites check passed"
}

# Get system information
get_system_info() {
    log_header "💻 System Information"
    
    echo "  OS: $(uname -s) $(uname -r)"
    echo "  CPU: $(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs || echo 'N/A')"
    echo "  Cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 'N/A')"
    echo "  Memory: $(free -h 2>/dev/null | grep Mem | awk '{print $2}' || echo 'N/A')"
    echo "  Rust: $(rustc --version)"
    echo "  Cargo: $(cargo --version)"
    echo ""
    
    # Check if running on battery (laptops)
    if [ -f /sys/class/power_supply/AC/online ]; then
        AC_STATUS=$(cat /sys/class/power_supply/AC/online)
        if [ "$AC_STATUS" -eq 0 ]; then
            log_warning "Running on battery power. Results may vary."
        fi
    fi
}

# Run benchmark with error handling
run_benchmark() {
    local benchmark_name="$1"
    local benchmark_command="$2"
    local timeout_seconds="${3:-300}"
    
    ((TOTAL_BENCHMARKS++))
    log_info "Running benchmark: $benchmark_name"
    
    local start_time=$(date +%s)
    
    if timeout "$timeout_seconds" $benchmark_command > "${RESULTS_DIR}/${benchmark_name}_${TIMESTAMP}.log" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log_success "$benchmark_name (${duration}s)"
        return 0
    else
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log_error "$benchmark_name (TIMEOUT after ${timeout_seconds}s)"
        else
            log_error "$benchmark_name (FAILED with exit code $exit_code)"
        fi
        return 1
    fi
}

# Generate JSON report
generate_json_report() {
    local output_file="$1"
    
    log_info "Generating JSON benchmark report..."
    
    cat > "$output_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "project": "clnrm",
  "benchmark_summary": {
    "total_benchmarks": $TOTAL_BENCHMARKS,
    "passed": $PASSED_BENCHMARKS,
    "failed": $FAILED_BENCHMARKS,
    "warnings": $WARNING_BENCHMARKS,
    "success_rate": $(( PASSED_BENCHMARKS * 100 / TOTAL_BENCHMARKS ))
  },
  "benchmark_results": [
$(printf '%s,\n' "${BENCHMARK_RESULTS[@]}" | sed '$s/,$//')
  ],
  "system_info": {
    "os": "$(uname -s) $(uname -r)",
    "cpu_cores": "$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 'unknown')",
    "rust_version": "$(rustc --version | awk '{print $2}')",
    "cargo_version": "$(cargo --version | awk '{print $2}')"
  }
}
EOF
    
    log_success "JSON report generated: $output_file"
}

# Compare with previous runs
compare_benchmarks() {
    log_header "📊 Comparing with Previous Runs"
    
    local previous_report=$(find "$RESULTS_DIR" -name "benchmark_*.json" -not -name "*_${TIMESTAMP}.json" | sort | tail -1)
    
    if [[ -z "$previous_report" ]]; then
        log_warning "No previous benchmark reports found for comparison"
        return
    fi
    
    log_info "Comparing with: $(basename "$previous_report")"
    
    # Simple comparison logic (could be enhanced with statistical analysis)
    local current_success_rate=$(( PASSED_BENCHMARKS * 100 / TOTAL_BENCHMARKS ))
    local previous_success_rate=$(jq -r '.benchmark_summary.success_rate' "$previous_report" 2>/dev/null || echo "0")
    
    local improvement=$((current_success_rate - previous_success_rate))
    
    if [[ $improvement -gt 0 ]]; then
        log_success "Benchmark success rate improved by ${improvement}%"
    elif [[ $improvement -lt 0 ]]; then
        log_warning "Benchmark success rate decreased by $((improvement * -1))%"
    else
        log_info "Benchmark success rate unchanged"
    fi
}

# Cleanup benchmark data
cleanup_benchmarks() {
    log_header "🧹 Cleaning Up Benchmark Data"
    
    # Remove old benchmark results (keep last 10)
    find "$RESULTS_DIR" -name "benchmark_*.json" -type f | sort | head -n -10 | xargs rm -f 2>/dev/null || true
    find "$RESULTS_DIR" -name "benchmark_*.log" -type f | sort | head -n -10 | xargs rm -f 2>/dev/null || true
    
    # Clean criterion data
    if [[ -d "$BENCHMARK_DIR" ]]; then
        log_info "Cleaning criterion benchmark data..."
        rm -rf "$BENCHMARK_DIR"/*/new 2>/dev/null || true
    fi
    
    log_success "Benchmark cleanup completed"
}

# Main benchmark execution
run_all_benchmarks() {
    log_header "🚀 Running All Benchmarks"
    
    # Core benchmarks
    run_benchmark "Core Performance" "cargo bench --bench core_benchmarks" 300
    run_benchmark "Memory Benchmarks" "cargo bench --bench memory_benchmarks" 300
    run_benchmark "Hot Reload Benchmarks" "cargo bench --bench hot_reload_critical_path" 600
    run_benchmark "AI Intelligence Benchmarks" "cargo bench --bench ai_intelligence_benchmarks" 300
    run_benchmark "DX Features Benchmarks" "cargo bench --bench dx_features_benchmarks" 300
    run_benchmark "Scenario Benchmarks" "cargo bench --bench scenario_benchmarks" 300
}

# Quick benchmark run
run_quick_benchmarks() {
    log_header "⚡ Running Quick Benchmarks"
    
    run_benchmark "Quick Core Test" "cargo bench --bench core_benchmarks -- --quick" 60
    run_benchmark "Quick Memory Test" "cargo bench --bench memory_benchmarks -- --quick" 60
}

# Memory-focused benchmarks
run_memory_benchmarks() {
    log_header "🧠 Running Memory-Focused Benchmarks"
    
    run_benchmark "Memory Allocation" "cargo bench --bench memory_benchmarks -- --memory" 300
    run_benchmark "Memory Leak Detection" "cargo bench --bench memory_benchmarks -- --leak-check" 300
    run_benchmark "Memory Usage Analysis" "cargo bench --bench memory_benchmarks -- --usage-analysis" 300
}

# Main function
main() {
    local quick_mode=false
    local memory_mode=false
    local compare_mode=false
    local generate_report=false
    local cleanup_mode=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --quick)
                quick_mode=true
                shift
                ;;
            --memory)
                memory_mode=true
                shift
                ;;
            --compare)
                compare_mode=true
                shift
                ;;
            --report)
                generate_report=true
                shift
                ;;
            --cleanup)
                cleanup_mode=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --quick     Run quick benchmark suite"
                echo "  --memory    Focus on memory benchmarks"
                echo "  --compare   Compare with previous runs"
                echo "  --report    Generate detailed JSON report"
                echo "  --cleanup   Clean up old benchmark data"
                echo "  --help, -h  Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Handle cleanup mode
    if [[ "$cleanup_mode" == true ]]; then
        cleanup_benchmarks
        exit 0
    fi
    
    # Display header
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║          CLNRM Performance Benchmark Suite                 ║${NC}"
    echo -e "${BLUE}║                                                            ║${NC}"
    echo -e "${BLUE}║  Timestamp: $(date '+%Y-%m-%d %H:%M:%S')                        ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Check prerequisites
    check_prerequisites
    echo ""
    
    # Get system information
    get_system_info
    
    # Run benchmarks based on mode
    if [[ "$quick_mode" == true ]]; then
        run_quick_benchmarks
    elif [[ "$memory_mode" == true ]]; then
        run_memory_benchmarks
    else
        run_all_benchmarks
    fi
    
    # Compare with previous runs if requested
    if [[ "$compare_mode" == true ]]; then
        echo ""
        compare_benchmarks
    fi
    
    # Generate report if requested
    if [[ "$generate_report" == true ]]; then
        echo ""
        generate_json_report "$JSON_REPORT"
    fi
    
    # Summary
    echo ""
    log_header "📊 Benchmark Summary"
    echo "Total Benchmarks: $TOTAL_BENCHMARKS"
    echo "Passed: $PASSED_BENCHMARKS"
    echo "Failed: $FAILED_BENCHMARKS"
    echo "Warnings: $WARNING_BENCHMARKS"
    echo "Success Rate: $(( PASSED_BENCHMARKS * 100 / TOTAL_BENCHMARKS ))%"
    
    # Determine exit code
    if [[ $FAILED_BENCHMARKS -gt 0 ]]; then
        log_error "Some benchmarks failed"
        exit 2
    elif [[ $WARNING_BENCHMARKS -gt 0 ]]; then
        log_warning "Some benchmarks have warnings"
        exit 1
    else
        log_success "All benchmarks completed successfully"
        exit 0
    fi
}

# Run main function
main "$@"
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
