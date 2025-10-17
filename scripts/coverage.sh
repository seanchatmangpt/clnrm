#!/usr/bin/env bash
# Code Coverage Script
# Generates coverage reports and checks thresholds for KGold project

set -euo pipefail

# Timeout for blocking cargo commands per repo rules
TIMEOUT=${TIMEOUT:-10s}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Cleanroom Code Coverage Analysis${NC}"
echo "======================================"

# Require cargo-llvm-cov
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo -e "${RED}❌ cargo-llvm-cov not found. Install with: cargo install cargo-llvm-cov --locked${NC}"
    exit 1
fi

# Require jq for JSON processing
if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ jq not found. Install with: brew install jq (macOS) or apt-get install jq (Ubuntu)${NC}"
    exit 1
fi

# Generate coverage report
echo -e "${BLUE}📊 Generating coverage report...${NC}"
if ! timeout "$TIMEOUT" cargo llvm-cov --workspace --all-features --tests --lcov --output-path coverage.lcov; then
    echo -e "${RED}❌ Coverage report generation failed or timed out${NC}"
    exit 1
fi

# Generate JSON report for analysis
echo -e "${BLUE}📈 Generating detailed coverage analysis...${NC}"
if ! timeout "$TIMEOUT" cargo llvm-cov --workspace --all-features --tests --json --output-path coverage.json; then
    echo -e "${RED}❌ Coverage JSON generation failed or timed out${NC}"
    exit 1
fi

# Check coverage thresholds
echo -e "${BLUE}🎯 Checking coverage thresholds...${NC}"

# Critical files that must have 80%+ coverage
CRITICAL_FILES=(
    "lib.rs"
    "api.rs"
    "validators.rs"
    "init.rs"
    "types.rs"
)

# High priority files that should have 70%+ coverage
HIGH_PRIORITY_FILES=(
    "mttr.rs"
    "advanced_mttr.rs"
    "metrics.rs"
    "validation.rs"
    "correlation.rs"
)

# Medium priority files that should have 60%+ coverage
MEDIUM_PRIORITY_FILES=(
    "governance.rs"
    "security.rs"
    "service_lifecycle.rs"
    "synthetic_monitoring.rs"
    "golden_signals.rs"
    "slo.rs"
)

# Function to check file coverage
check_file_coverage() {
    local file_pattern="$1"
    local threshold="$2"
    local priority="$3"
    
    echo -e "${BLUE}Checking $priority files (threshold: ${threshold}%)...${NC}"
    
    # Extract coverage for files matching the pattern
    local files_below_threshold=""
    if [[ -f "coverage.json" ]]; then
        files_below_threshold=$(jq -r --arg pattern "$file_pattern" --arg threshold "$threshold" '
            .files[] | 
            select(.filename | contains($pattern)) | 
            select(.summary.lines.percent < ($threshold | tonumber)) | 
            "\(.filename): \(.summary.lines.percent)%"
        ' coverage.json 2>/dev/null || echo "")
    fi
    
    if [ -n "$files_below_threshold" ]; then
        echo -e "${RED}❌ Files below ${threshold}% threshold:${NC}"
        echo "$files_below_threshold"
        return 1
    else
        echo -e "${GREEN}✅ All $priority files meet ${threshold}% threshold${NC}"
        return 0
    fi
}

# Check critical files
critical_failed=false
for file in "${CRITICAL_FILES[@]}"; do
    if ! check_file_coverage "$file" "80" "critical"; then
        critical_failed=true
    fi
done

# Check high priority files
high_priority_failed=false
for file in "${HIGH_PRIORITY_FILES[@]}"; do
    if ! check_file_coverage "$file" "70" "high priority"; then
        high_priority_failed=true
    fi
done

# Check medium priority files
medium_priority_failed=false
for file in "${MEDIUM_PRIORITY_FILES[@]}"; do
    if ! check_file_coverage "$file" "60" "medium priority"; then
        medium_priority_failed=true
    fi
done

# Overall coverage summary
echo -e "${BLUE}📊 Overall Coverage Summary${NC}"
echo "=========================="

# Extract overall coverage
overall_coverage="0"
if [[ -f "coverage.json" ]]; then
    overall_coverage=$(jq -r '.summary.lines.percent' coverage.json 2>/dev/null || echo "0")
fi
echo -e "Overall Coverage: ${GREEN}${overall_coverage}%${NC}"

# File count summary
total_files="0"
if [[ -f "coverage.json" ]]; then
    total_files=$(jq -r '.files | length' coverage.json 2>/dev/null || echo "0")
fi
echo -e "Total Files Analyzed: ${BLUE}${total_files}${NC}"

# Coverage distribution
echo -e "${BLUE}Coverage Distribution:${NC}"
if [[ -f "coverage.json" ]]; then
    jq -r '
        .files[] | 
        select(.summary.lines.percent >= 80) | 
        .filename
    ' coverage.json 2>/dev/null | wc -l | xargs echo "Files with 80%+ coverage:"
    jq -r '
        .files[] | 
        select(.summary.lines.percent >= 70 and .summary.lines.percent < 80) | 
        .filename
    ' coverage.json 2>/dev/null | wc -l | xargs echo "Files with 70-79% coverage:"
    jq -r '
        .files[] | 
        select(.summary.lines.percent >= 60 and .summary.lines.percent < 70) | 
        .filename
    ' coverage.json 2>/dev/null | wc -l | xargs echo "Files with 60-69% coverage:"
    jq -r '
        .files[] | 
        select(.summary.lines.percent < 60) | 
        .filename
    ' coverage.json 2>/dev/null | wc -l | xargs echo "Files below 60% coverage:"

    # Show files with lowest coverage
    echo -e "${BLUE}Files with Lowest Coverage:${NC}"
    jq -r '
        .files[] | 
        select(.summary.lines.percent < 80) | 
        "\(.summary.lines.percent | floor)% - \(.filename)"
    ' coverage.json 2>/dev/null | sort -n | head -10
else
    echo "No coverage data available"
fi

# Final result
echo ""
if [ "$critical_failed" = true ]; then
    echo -e "${RED}❌ CRITICAL FILES BELOW 80% THRESHOLD${NC}"
    echo -e "${RED}   This is a blocking issue. Please add tests to critical files.${NC}"
    exit 1
elif [ "$high_priority_failed" = true ]; then
    echo -e "${YELLOW}⚠️  HIGH PRIORITY FILES BELOW 70% THRESHOLD${NC}"
    echo -e "${YELLOW}   Consider adding more tests to high priority files.${NC}"
    exit 1
elif [ "$medium_priority_failed" = true ]; then
    echo -e "${YELLOW}⚠️  MEDIUM PRIORITY FILES BELOW 60% THRESHOLD${NC}"
    echo -e "${YELLOW}   Consider adding tests to medium priority files.${NC}"
    exit 0
else
    echo -e "${GREEN}✅ ALL FILES MEET COVERAGE THRESHOLDS${NC}"
    echo -e "${GREEN}   Excellent coverage across all priority levels!${NC}"
    exit 0
fi
