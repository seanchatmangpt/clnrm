#!/bin/bash
# Run comprehensive OTEL telemetry and Weaver performance benchmarks

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  OTEL Telemetry & Weaver Performance Benchmark Suite     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

if ! command -v criterion &> /dev/null; then
    echo -e "${YELLOW}Note: criterion CLI not installed (optional)${NC}"
fi

# Create output directory
BENCHMARK_DIR="target/benchmark_results"
mkdir -p "$BENCHMARK_DIR"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_FILE="$BENCHMARK_DIR/telemetry_perf_${TIMESTAMP}.txt"

echo -e "${GREEN}✓ Prerequisites checked${NC}"
echo

# Build benchmarks
echo -e "${YELLOW}Building benchmarks...${NC}"
cargo build --release --benches
echo -e "${GREEN}✓ Build complete${NC}"
echo

# Run telemetry performance benchmarks
echo -e "${BLUE}Running telemetry performance benchmarks...${NC}"
echo "This may take 5-10 minutes depending on your system."
echo

cargo bench --bench telemetry_performance -- --output-format bencher | tee "$OUTPUT_FILE"

# Run cleanroom benchmarks for comparison
echo -e "${BLUE}Running cleanroom environment benchmarks...${NC}"
cargo bench --bench cleanroom_benchmarks -- --output-format bencher | tee -a "$OUTPUT_FILE"

# Generate HTML reports
echo -e "${YELLOW}Generating HTML reports...${NC}"
REPORT_DIR="target/criterion"
if [ -d "$REPORT_DIR" ]; then
    echo -e "${GREEN}✓ Criterion reports available at: file://${PWD}/${REPORT_DIR}/report/index.html${NC}"
fi

# Summary
echo
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Benchmark Results Summary                                 ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo

# Extract key metrics from output
if grep -q "container_startup/without_otel" "$OUTPUT_FILE" 2>/dev/null; then
    echo -e "${GREEN}Container Startup Benchmarks:${NC}"
    grep "container_startup" "$OUTPUT_FILE" | head -4
    echo
fi

if grep -q "otlp_export" "$OUTPUT_FILE" 2>/dev/null; then
    echo -e "${GREEN}OTLP Export Benchmarks:${NC}"
    grep "otlp_export" "$OUTPUT_FILE" | head -4
    echo
fi

if grep -q "weaver_validation" "$OUTPUT_FILE" 2>/dev/null; then
    echo -e "${GREEN}Weaver Validation Benchmarks:${NC}"
    grep "weaver_validation" "$OUTPUT_FILE" | head -5
    echo
fi

echo -e "${YELLOW}Full results saved to: ${OUTPUT_FILE}${NC}"
echo -e "${YELLOW}HTML reports: file://${PWD}/${REPORT_DIR}/report/index.html${NC}"

# Performance analysis
echo
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Performance Analysis                                      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo

cat << 'EOF'
Key Metrics to Analyze:
-----------------------

1. Container Startup Overhead:
   - Compare "without_otel" vs "with_full_telemetry"
   - Target: <15% overhead acceptable
   - Action: If >25%, enable sampling

2. OTLP Export Latency:
   - Check export times for different payload sizes
   - Target: <5ms for batches of 100 items
   - Action: If >10ms, enable batching/compression

3. Weaver Validation Overhead:
   - Check validation time per item
   - Target: <10µs per telemetry item
   - Action: If >50µs, implement caching

4. Memory Overhead:
   - Estimate based on item counts
   - Target: <100MB for typical workload
   - Action: If >200MB, reduce sampling rate

5. Concurrent Performance:
   - Check scaling with multiple containers
   - Target: Linear scaling up to 10 containers
   - Action: If sub-linear, optimize locking

6. End-to-End Pipeline:
   - Full pipeline latency per test
   - Target: <50ms overhead per test
   - Action: If >100ms, move export to background

Optimization Recommendations:
----------------------------
Priority 1 (Implement First):
  • Adaptive sampling (60-80% overhead reduction)
  • Batch OTLP exports (30-50% export reduction)
  • Async telemetry export (eliminates blocking)

Priority 2 (Medium Term):
  • Schema lookup caching (40-60% validation speedup)
  • Selective instrumentation (20-40% volume reduction)

Priority 3 (Long Term):
  • OTLP compression (50-70% bandwidth reduction)
  • Parallel Weaver validation (2-3x throughput)

EOF

echo
echo -e "${GREEN}✓ Benchmark suite complete!${NC}"
echo
