#!/usr/bin/env bash
# Production Validation Script for Weaver Live-Check Integration
#
# This script executes comprehensive production validation tests across
# all categories: performance, reliability, security, deployment, and integration.
#
# Usage:
#   ./scripts/production_validation.sh [category]
#
# Categories:
#   all          - Run all validation tests (default)
#   performance  - Performance and load tests
#   reliability  - Crash recovery and failure tests
#   security     - Security and data protection tests
#   deployment   - Platform deployment tests
#   integration  - End-to-end integration tests
#   quick        - Quick smoke test (subset of tests)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REGISTRY_PATH="${REGISTRY_PATH:-registry/}"
OUTPUT_DIR="${OUTPUT_DIR:-./validation_output/production}"
CATEGORY="${1:-all}"
PARALLEL_JOBS="${PARALLEL_JOBS:-4}"

# Prerequisites check
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

    local missing=()

    if ! command -v weaver &> /dev/null; then
        missing+=("weaver")
    fi

    if ! command -v cargo &> /dev/null; then
        missing+=("cargo")
    fi

    if ! command -v docker &> /dev/null; then
        missing+=("docker")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing prerequisites: ${missing[*]}${NC}"
        echo "Install missing tools and try again."
        exit 1
    fi

    echo -e "${GREEN}✅ All prerequisites found${NC}"
}

# Verify Weaver installation
verify_weaver() {
    echo -e "${BLUE}🔧 Verifying Weaver installation...${NC}"

    if ! weaver --version &> /dev/null; then
        echo -e "${RED}❌ Weaver not properly installed${NC}"
        exit 1
    fi

    local version=$(weaver --version 2>&1 | head -n1)
    echo -e "${GREEN}✅ Weaver version: ${version}${NC}"
}

# Schema validation (baseline)
validate_schema() {
    echo -e "${BLUE}📋 Validating schema registry...${NC}"

    if ! weaver registry check --registry "$REGISTRY_PATH"; then
        echo -e "${RED}❌ Schema validation failed${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Schema validation passed${NC}"
}

# Performance tests
run_performance_tests() {
    echo -e "${BLUE}⚡ Running performance validation tests...${NC}"

    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        performance
}

# Reliability tests
run_reliability_tests() {
    echo -e "${BLUE}🔥 Running reliability validation tests...${NC}"

    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        reliability
}

# Security tests
run_security_tests() {
    echo -e "${BLUE}🔐 Running security validation tests...${NC}"

    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        security
}

# Deployment tests
run_deployment_tests() {
    echo -e "${BLUE}🚀 Running deployment validation tests...${NC}"

    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        deployment
}

# Integration tests
run_integration_tests() {
    echo -e "${BLUE}🔗 Running integration validation tests...${NC}"

    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        integration
}

# Quick smoke test
run_quick_validation() {
    echo -e "${BLUE}💨 Running quick smoke tests...${NC}"

    # Run a subset of fast tests from each category
    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        test_weaver_config_defaults \
        test_validation_report_default \
        test_validation_status_serialization
}

# Benchmark suite
run_benchmarks() {
    echo -e "${BLUE}📊 Running performance benchmarks...${NC}"

    cargo test --test production_validation \
        --features otel \
        -- \
        --ignored \
        --test-threads=1 \
        benchmark
}

# Generate validation report
generate_report() {
    echo -e "${BLUE}📄 Generating validation report...${NC}"

    local report_file="$OUTPUT_DIR/production_validation_report.md"
    mkdir -p "$OUTPUT_DIR"

    cat > "$report_file" << EOF
# Production Validation Report

**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Category:** $CATEGORY
**Registry:** $REGISTRY_PATH

## Test Execution Summary

EOF

    if [ -f "$OUTPUT_DIR/test_results.json" ]; then
        # Parse test results and add to report
        echo "Test results found in $OUTPUT_DIR/test_results.json"
    fi

    echo -e "${GREEN}✅ Report generated: $report_file${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Production Validation Suite - clnrm     ║${NC}"
    echo -e "${BLUE}║  Weaver Live-Check Integration           ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
    echo ""

    check_prerequisites
    verify_weaver
    validate_schema

    echo ""
    echo -e "${BLUE}Category: ${YELLOW}${CATEGORY}${NC}"
    echo ""

    # Create output directory
    mkdir -p "$OUTPUT_DIR"

    case "$CATEGORY" in
        all)
            run_performance_tests
            run_reliability_tests
            run_security_tests
            run_deployment_tests
            run_integration_tests
            ;;
        performance)
            run_performance_tests
            ;;
        reliability)
            run_reliability_tests
            ;;
        security)
            run_security_tests
            ;;
        deployment)
            run_deployment_tests
            ;;
        integration)
            run_integration_tests
            ;;
        quick)
            run_quick_validation
            ;;
        benchmark)
            run_benchmarks
            ;;
        *)
            echo -e "${RED}❌ Unknown category: $CATEGORY${NC}"
            echo "Valid categories: all, performance, reliability, security, deployment, integration, quick, benchmark"
            exit 1
            ;;
    esac

    generate_report

    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Production Validation Complete ✅        ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
}

# Trap errors
trap 'echo -e "${RED}❌ Validation failed with error${NC}"; exit 1' ERR

# Run main
main
