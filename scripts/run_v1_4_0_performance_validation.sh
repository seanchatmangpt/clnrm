#!/bin/bash
# v1.4.0 Performance Validation Automation Script
#
# This script automates the complete performance validation workflow:
# 1. Collects v1.3.0 baseline metrics
# 2. Runs v1.4.0 comprehensive benchmarks
# 3. Compares results and generates reports
# 4. Validates performance targets are met
#
# Usage:
#   ./scripts/run_v1_4_0_performance_validation.sh [--baseline-only|--validation-only|--full]
#
# Modes:
#   --baseline-only: Only collect v1.3.0 baseline
#   --validation-only: Only run v1.4.0 validation (assumes baseline exists)
#   --full: Complete workflow (default)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BASELINE_DIR="$PROJECT_ROOT/target/baseline_v1_3_0"
VALIDATION_DIR="$PROJECT_ROOT/target/validation_v1_4_0"
REPORT_DIR="$PROJECT_ROOT/docs/performance_reports"

# Create directories
mkdir -p "$BASELINE_DIR" "$VALIDATION_DIR" "$REPORT_DIR"

# Parse arguments
MODE="${1:-full}"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function: Collect v1.3.0 baseline metrics
collect_baseline() {
    log_info "Collecting v1.3.0 baseline metrics..."

    # Save current branch
    CURRENT_BRANCH=$(git branch --show-current)
    log_info "Current branch: $CURRENT_BRANCH"

    # Check if baseline already exists
    if [ -d "$BASELINE_DIR/criterion" ]; then
        log_warning "Baseline already exists at $BASELINE_DIR/criterion"
        read -p "Overwrite existing baseline? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Skipping baseline collection"
            return 0
        fi
    fi

    # Checkout v1.3.0
    log_info "Checking out v1.3.0..."
    git checkout v1.3.0 || {
        log_error "Failed to checkout v1.3.0"
        return 1
    }

    # Clean build
    log_info "Building v1.3.0..."
    cargo clean
    cargo build --release --features otel || {
        log_error "Failed to build v1.3.0"
        git checkout "$CURRENT_BRANCH"
        return 1
    }

    # Run baseline benchmarks
    log_info "Running baseline benchmarks (stress_capacity_benchmarks)..."
    cargo bench --bench stress_capacity_benchmarks -- --save-baseline v1_3_0 || {
        log_error "Baseline benchmarks failed"
        git checkout "$CURRENT_BRANCH"
        return 1
    }

    # Copy baseline results
    log_info "Saving baseline results..."
    cp -r target/criterion "$BASELINE_DIR/"

    # Generate baseline summary
    cat > "$BASELINE_DIR/summary.txt" << EOF
v1.3.0 Baseline Performance Metrics
====================================
Date: $(date)
Commit: $(git rev-parse HEAD)
Rust: $(rustc --version)

Expected Baseline Values (from stress_capacity_benchmarks):
- Throughput: 10-20 tests/sec
- Concurrency: 50-100 concurrent tests
- Container startup: 2-5 seconds
- P95 latency: 5-10 seconds
- Memory: ~200MB baseline

Benchmark Results:
See target/criterion/ for detailed HTML reports
EOF

    log_success "Baseline collection complete"

    # Return to original branch
    git checkout "$CURRENT_BRANCH"
}

# Function: Run v1.4.0 validation benchmarks
run_validation() {
    log_info "Running v1.4.0 performance validation..."

    # Check if on correct branch
    CURRENT_BRANCH=$(git branch --show-current)
    if [[ ! "$CURRENT_BRANCH" =~ v1.4.0|v1_4_0|master ]]; then
        log_warning "Not on v1.4.0 or master branch (current: $CURRENT_BRANCH)"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_error "Aborted"
            return 1
        fi
    fi

    # Clean build
    log_info "Building v1.4.0..."
    cargo clean
    cargo build --release --features otel || {
        log_error "Failed to build v1.4.0"
        log_error "Please fix compilation errors before running benchmarks"
        return 1
    }

    # Run validation benchmarks
    log_info "Running v1.4.0 performance validation benchmarks..."
    log_info "This may take 30-45 minutes..."

    cargo bench --bench v1_4_0_performance_validation || {
        log_error "Validation benchmarks failed"
        return 1
    }

    # Copy validation results
    log_info "Saving validation results..."
    cp -r target/criterion "$VALIDATION_DIR/"

    log_success "Validation benchmarks complete"

    # Run regression tests
    log_info "Running regression validation..."
    cargo bench --bench performance_regression || {
        log_warning "Regression tests failed - manual review required"
    }

    log_success "Performance validation complete"
}

# Function: Compare baseline vs. validation
compare_results() {
    log_info "Comparing v1.3.0 baseline vs. v1.4.0 validation..."

    # Check if both baseline and validation exist
    if [ ! -d "$BASELINE_DIR/criterion" ]; then
        log_error "Baseline not found at $BASELINE_DIR/criterion"
        log_error "Run with --baseline-only first"
        return 1
    fi

    if [ ! -d "$VALIDATION_DIR/criterion" ]; then
        log_error "Validation results not found at $VALIDATION_DIR/criterion"
        log_error "Run with --validation-only first"
        return 1
    fi

    # Generate comparison report
    REPORT_FILE="$REPORT_DIR/v1_4_0_validation_report_$(date +%Y%m%d_%H%M%S).md"

    cat > "$REPORT_FILE" << 'EOF'
# v1.4.0 Performance Validation Report

**Date:** $(date)
**Baseline:** v1.3.0
**Validation:** v1.4.0
**Hardware:** $(sysctl -n machdep.cpu.brand_string) ($(sysctl -n hw.physicalcpu) cores)
**OS:** $(sw_vers -productName) $(sw_vers -productVersion)

---

## Executive Summary

### Performance Targets

| Metric | v1.3.0 Baseline | v1.4.0 Target | v1.4.0 Actual | Status |
|--------|-----------------|---------------|---------------|--------|
| Throughput | 10-20 tests/sec | 100-200 tests/sec | TBD | ⏳ |
| Concurrency | 50-100 tests | 500-1000 tests | TBD | ⏳ |
| Pool Hit Latency | 2500ms | <1ms | TBD | ⏳ |
| P95 Latency | 5000ms | 1500ms | TBD | ⏳ |
| P99 Latency | 8000ms | 2500ms | TBD | ⏳ |
| Memory Overhead | 200MB | <250MB | TBD | ⏳ |
| Pool Hit Rate | N/A | >90% | TBD | ⏳ |

**Overall Status:** ⏳ PENDING MANUAL ANALYSIS

---

## Benchmark Results

### 1. Container Pooling vs. Fresh Creation

**v1.3.0 Fresh Container:**
- Mean: TBD ms/iter
- Std Dev: TBD ms

**v1.4.0 Pooled Container (Warm):**
- Mean: TBD ms/iter
- Std Dev: TBD ms
- **Improvement:** TBD x faster

**v1.4.0 Pooled Container (Realistic 90% hit rate):**
- Mean: TBD ms/iter
- **Improvement:** TBD x faster

**Analysis:** TBD

---

### 2. Throughput Improvement

**v1.3.0 Sequential:**
- Throughput: TBD tests/sec

**v1.4.0 Concurrent (10 threads):**
- Throughput: TBD tests/sec
- **Improvement:** TBD x

**v1.4.0 Concurrent (50 threads):**
- Throughput: TBD tests/sec
- **Improvement:** TBD x

**v1.4.0 Concurrent (100 threads):**
- Throughput: TBD tests/sec
- **Improvement:** TBD x

**v1.4.0 Concurrent (200 threads):**
- Throughput: TBD tests/sec
- **Improvement:** TBD x

**Analysis:** TBD

---

### 3. Concurrency Scaling

| Concurrency | Throughput | Avg Latency | Hit Rate | Status |
|-------------|------------|-------------|----------|--------|
| 50          | TBD        | TBD         | TBD      | TBD    |
| 100         | TBD        | TBD         | TBD      | TBD    |
| 250         | TBD        | TBD         | TBD      | TBD    |
| 500         | TBD        | TBD         | TBD      | TBD    |
| 750         | TBD        | TBD         | TBD      | TBD    |
| 1000        | TBD        | TBD         | TBD      | TBD    |

**Analysis:** TBD

---

### 4. Latency Percentiles

**v1.3.0 Latency Distribution:**
- P50: TBD ms
- P95: TBD ms
- P99: TBD ms

**v1.4.0 Latency Distribution:**
- P50: TBD ms (TBD% reduction)
- P95: TBD ms (TBD% reduction)
- P99: TBD ms (TBD% reduction)

**Analysis:** TBD

---

### 5. Atomic Metrics Performance

**Single-threaded:**
- Operations/sec: TBD

**Multi-threaded (4 threads):**
- Operations/sec: TBD
- Scaling: TBD x

**Multi-threaded (8 threads):**
- Operations/sec: TBD
- Scaling: TBD x

**Multi-threaded (16 threads):**
- Operations/sec: TBD
- Scaling: TBD x

**Multi-threaded (32 threads):**
- Operations/sec: TBD
- Scaling: TBD x

**Analysis:** TBD

---

### 6. Memory Overhead Under Load

| Load | Memory Usage | Increase | Status |
|------|--------------|----------|--------|
| 100  | TBD MB       | TBD MB   | TBD    |
| 500  | TBD MB       | TBD MB   | TBD    |
| 1000 | TBD MB       | TBD MB   | TBD    |

**Analysis:** TBD

---

### 7. Pool Hit Rate Analysis

| Pool Size | Hit Rate | P95 Latency | Recommendation |
|-----------|----------|-------------|----------------|
| 10        | TBD      | TBD         | TBD            |
| 20        | TBD      | TBD         | TBD            |
| 50        | TBD      | TBD         | TBD            |
| 100       | TBD      | TBD         | TBD            |

**Optimal Pool Size:** TBD

**Analysis:** TBD

---

### 8. Full System Integration

**1000 tests with realistic workload:**
- Total duration: TBD seconds
- Throughput: TBD tests/sec
- Pool hit rate: TBD%
- Success rate: TBD%
- Average latency: TBD ms
- P95 latency: TBD ms
- P99 latency: TBD ms
- Memory peak: TBD MB

**Analysis:** TBD

---

## Regression Analysis

**Regression Tests Status:** TBD

**Zero Regressions Validated:** ⏳

| Metric | Baseline | Current | Change | Status |
|--------|----------|---------|--------|--------|
| OTEL Overhead | TBD | TBD | TBD | TBD |
| Memory Usage | TBD | TBD | TBD | TBD |
| Binary Size | TBD | TBD | TBD | TBD |
| Container Startup | TBD | TBD | TBD | TBD |

---

## Performance Targets Validation

### ✅ Achieved Targets

TBD

### ⏳ Partially Achieved Targets

TBD

### ❌ Not Achieved Targets

TBD

---

## Recommendations

### Immediate Actions

TBD

### Optimizations

TBD

### Future Work

TBD

---

## Conclusion

TBD

**Sign-Off:**
- [ ] Performance targets met
- [ ] Zero regressions validated
- [ ] Production ready

**Prepared by:** Agent 12 (Performance Benchmark Engineer)
**Date:** $(date)
EOF

    log_success "Comparison report generated: $REPORT_FILE"

    # Open HTML reports in browser
    log_info "Opening HTML reports..."
    if command -v open &> /dev/null; then
        open "$VALIDATION_DIR/criterion/report/index.html" 2>/dev/null || true
    fi

    log_info "Manual analysis required to complete report"
    log_info "See: $REPORT_FILE"
}

# Function: Print usage
usage() {
    cat << EOF
v1.4.0 Performance Validation Script

Usage:
    $0 [OPTIONS]

Options:
    --baseline-only     Collect v1.3.0 baseline metrics only
    --validation-only   Run v1.4.0 validation benchmarks only
    --full              Complete workflow (default)
    --compare           Compare baseline vs. validation results
    --help              Show this help message

Examples:
    # Full workflow (collect baseline + run validation + compare)
    $0 --full

    # Collect baseline only
    $0 --baseline-only

    # Run validation only (assumes baseline exists)
    $0 --validation-only

    # Compare existing results
    $0 --compare

Output:
    Baseline:   $BASELINE_DIR/
    Validation: $VALIDATION_DIR/
    Reports:    $REPORT_DIR/
EOF
}

# Main execution
main() {
    log_info "v1.4.0 Performance Validation Automation"
    log_info "=========================================="

    case "$MODE" in
        --baseline-only)
            collect_baseline
            ;;
        --validation-only)
            run_validation
            ;;
        --compare)
            compare_results
            ;;
        --full)
            collect_baseline && run_validation && compare_results
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown mode: $MODE"
            usage
            exit 1
            ;;
    esac

    log_success "Complete!"
    log_info "Next steps:"
    log_info "  1. Review HTML reports: open $VALIDATION_DIR/criterion/report/index.html"
    log_info "  2. Complete analysis in: $REPORT_DIR/v1_4_0_validation_report_*.md"
    log_info "  3. Validate all performance targets met"
    log_info "  4. Sign off on production readiness"
}

# Run main
cd "$PROJECT_ROOT"
main
