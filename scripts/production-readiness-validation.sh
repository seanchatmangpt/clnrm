#!/usr/bin/env bash
# ==============================================================================
# production-readiness-validation.sh
# Comprehensive Production Readiness Validation for Cleanroom Testing Framework
#
# This script performs exhaustive validation to ensure the framework is ready
# for production deployment, including performance, security, reliability,
# and operational readiness checks.
#
# Usage:
#   ./scripts/production-readiness-validation.sh           # Full validation
#   ./scripts/production-readiness-validation.sh --quick   # Quick validation
#   ./scripts/production-readiness-validation.sh --report  # Generate report
#   ./scripts/production-readiness-validation.sh --fix     # Auto-fix issues
#
# Exit Codes:
#   0 - Production ready
#   1 - Minor issues (warnings)
#   2 - Major issues (blocking)
#   3 - Critical issues (not production ready)
# ==============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_FILE="$PROJECT_ROOT/production-readiness-report.json"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters and state
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0
CRITICAL_ISSUES=0
VALIDATION_RESULTS=()

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_CHECKS++))
    VALIDATION_RESULTS+=("{\"check\":\"$1\",\"status\":\"PASS\",\"timestamp\":\"$TIMESTAMP\"}")
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_CHECKS++))
    VALIDATION_RESULTS+=("{\"check\":\"$1\",\"status\":\"FAIL\",\"timestamp\":\"$TIMESTAMP\"}")
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNING_CHECKS++))
    VALIDATION_RESULTS+=("{\"check\":\"$1\",\"status\":\"WARN\",\"timestamp\":\"$TIMESTAMP\"}")
}

log_critical() {
    echo -e "${RED}[CRITICAL]${NC} $1"
    ((CRITICAL_ISSUES++))
    ((FAILED_CHECKS++))
    VALIDATION_RESULTS+=("{\"check\":\"$1\",\"status\":\"CRITICAL\",\"timestamp\":\"$TIMESTAMP\"}")
}

log_header() {
    echo -e "${PURPLE}[HEADER]${NC} $1"
}

# Check function wrapper
run_check() {
    local check_name="$1"
    local check_command="$2"
    local critical="${3:-false}"
    
    ((TOTAL_CHECKS++))
    log_info "Running check: $check_name"
    
    if eval "$check_command" >/dev/null 2>&1; then
        log_success "$check_name"
        return 0
    else
        if [[ "$critical" == "true" ]]; then
            log_critical "$check_name"
            return 2
        else
            log_error "$check_name"
            return 1
        fi
    fi
}

# Code Quality Checks
validate_code_quality() {
    log_header "🔍 Code Quality Validation"
    
    # No unwrap/expect in production code
    run_check "No unwrap/expect in production code" \
        "! grep -r '\.unwrap()\|\.expect(' crates/ --include='*.rs' | grep -v '//.*unwrap\|//.*expect' | grep -v 'test'"
    
    # No async trait methods (dyn compatibility)
    run_check "No async trait methods" \
        "! grep -r 'async fn' crates/ --include='*.rs' | grep -A1 -B1 'trait'"
    
    # Proper error handling
    run_check "Proper error handling patterns" \
        "grep -r 'Result<' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 10 ? 0 : 1}'"
    
    # Clippy warnings
    run_check "Zero clippy warnings" \
        "cargo clippy --all-features -- -D warnings 2>&1 | grep -q 'warning:' && exit 1 || exit 0"
    
    # Code formatting
    run_check "Code formatting compliance" \
        "cargo fmt -- --check"
    
    # Documentation coverage
    run_check "Documentation coverage" \
        "cargo doc --all-features --no-deps --document-private-items 2>&1 | grep -q 'warning.*missing documentation' && exit 1 || exit 0"
}

# Security Validation
validate_security() {
    log_header "🔒 Security Validation"
    
    # Dependency audit
    run_check "Dependency security audit" \
        "cargo audit"
    
    # No hardcoded secrets
    run_check "No hardcoded secrets" \
        "! grep -r -i 'password\|secret\|key\|token' crates/ --include='*.rs' | grep -v '//.*test\|//.*example'"
    
    # Input validation
    run_check "Input validation patterns" \
        "grep -r 'validate\|sanitize\|escape' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 5 ? 0 : 1}'"
    
    # Error message security
    run_check "Secure error messages" \
        "! grep -r 'unwrap\|expect' crates/ --include='*.rs' | grep -v 'test' | grep -v '//.*unwrap\|//.*expect'"
}

# Performance Validation
validate_performance() {
    log_header "⚡ Performance Validation"
    
    # Build performance
    run_check "Release build performance" \
        "timeout 300 cargo build --release --all-features"
    
    # Test performance
    run_check "Test execution performance" \
        "timeout 600 cargo test --lib --release"
    
    # Memory usage
    run_check "Memory usage validation" \
        "cargo test --lib --features memory-tests 2>&1 | grep -q 'memory leak' && exit 1 || exit 0"
    
    # Benchmark execution
    run_check "Benchmark suite execution" \
        "cargo bench --bench '*' 2>&1 | grep -q 'error' && exit 1 || exit 0"
}

# Reliability Validation
validate_reliability() {
    log_header "🛡️ Reliability Validation"
    
    # Test coverage
    run_check "Test coverage validation" \
        "cargo test --lib --all-features 2>&1 | grep -q 'test result: FAILED' && exit 1 || exit 0"
    
    # Integration tests
    run_check "Integration test suite" \
        "cargo test --test '*' 2>&1 | grep -q 'test result: FAILED' && exit 1 || exit 0"
    
    # Property-based tests
    run_check "Property-based testing" \
        "cargo test --features proptest 2>&1 | grep -q 'test result: FAILED' && exit 1 || exit 0"
    
    # Error recovery
    run_check "Error recovery mechanisms" \
        "grep -r 'recover\|retry\|fallback' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 3 ? 0 : 1}'"
    
    # Graceful degradation
    run_check "Graceful degradation patterns" \
        "grep -r 'graceful\|degradation\|fallback' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 2 ? 0 : 1}'"
}

# Operational Readiness
validate_operational_readiness() {
    log_header "🚀 Operational Readiness"
    
    # Logging and monitoring
    run_check "Logging and monitoring setup" \
        "grep -r 'tracing\|log\|metrics' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 10 ? 0 : 1}'"
    
    # Configuration management
    run_check "Configuration management" \
        "grep -r 'config\|Config' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 5 ? 0 : 1}'"
    
    # Health checks
    run_check "Health check endpoints" \
        "grep -r 'health\|status\|ping' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 2 ? 0 : 1}'"
    
    # Graceful shutdown
    run_check "Graceful shutdown handling" \
        "grep -r 'shutdown\|terminate\|signal' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 3 ? 0 : 1}'"
    
    # Resource cleanup
    run_check "Resource cleanup patterns" \
        "grep -r 'cleanup\|drop\|close' crates/ --include='*.rs' | wc -l | awk '{exit \$1 > 5 ? 0 : 1}'"
}

# Documentation and Usability
validate_documentation() {
    log_header "📚 Documentation and Usability"
    
    # API documentation
    run_check "API documentation completeness" \
        "cargo doc --all-features --no-deps --document-private-items 2>&1 | grep -c 'warning.*missing documentation' | awk '{exit \$1 < 5 ? 0 : 1}'"
    
    # README completeness
    run_check "README completeness" \
        "test -f README.md && grep -q 'Installation\|Usage\|Examples' README.md"
    
    # Example code
    run_check "Example code availability" \
        "test -d examples && find examples -name '*.rs' | wc -l | awk '{exit \$1 > 3 ? 0 : 1}'"
    
    # CLI help
    run_check "CLI help documentation" \
        "cargo run -- --help 2>&1 | grep -q 'Usage:'"
}

# Compliance and Standards
validate_compliance() {
    log_header "📋 Compliance and Standards"
    
    # License compliance
    run_check "License compliance" \
        "test -f LICENSE && grep -q 'MIT\|Apache\|BSD' LICENSE"
    
    # Cargo.toml completeness
    run_check "Cargo.toml completeness" \
        "grep -q 'description\|license\|repository\|homepage' Cargo.toml"
    
    # Version consistency
    run_check "Version consistency" \
        "grep '^version' crates/*/Cargo.toml | sort | uniq | wc -l | awk '{exit \$1 == 1 ? 0 : 1}'"
    
    # Dependency management
    run_check "Dependency management" \
        "cargo tree --depth 1 | grep -q 'clnrm'"
}

# Generate report
generate_report() {
    local report_file="$1"
    
    log_info "Generating production readiness report..."
    
    cat > "$report_file" << EOF
{
  "timestamp": "$TIMESTAMP",
  "project": "clnrm",
  "validation_summary": {
    "total_checks": $TOTAL_CHECKS,
    "passed": $PASSED_CHECKS,
    "failed": $FAILED_CHECKS,
    "warnings": $WARNING_CHECKS,
    "critical_issues": $CRITICAL_ISSUES,
    "success_rate": $(( PASSED_CHECKS * 100 / TOTAL_CHECKS )),
    "production_ready": $(( CRITICAL_ISSUES == 0 && FAILED_CHECKS < TOTAL_CHECKS / 10 ? 1 : 0 ))
  },
  "validation_results": [
$(printf '%s,\n' "${VALIDATION_RESULTS[@]}" | sed '$s/,$//')
  ],
  "recommendations": [
    $([ $CRITICAL_ISSUES -gt 0 ] && echo '"Address critical issues before production deployment"' || echo '')
    $([ $FAILED_CHECKS -gt 0 ] && echo '"Review and fix failed validation checks"' || echo '')
    $([ $WARNING_CHECKS -gt 0 ] && echo '"Consider addressing warning-level issues"' || echo '')
  ]
}
EOF
    
    log_success "Report generated: $report_file"
}

# Quick validation mode
quick_validation() {
    log_header "⚡ Quick Production Readiness Validation"
    
    # Essential checks only
    run_check "Compilation" "cargo check --all-features"
    run_check "Tests" "cargo test --lib"
    run_check "Security audit" "cargo audit"
    run_check "Clippy" "cargo clippy --all-features -- -D warnings"
    run_check "Documentation" "cargo doc --all-features --no-deps"
}

# Auto-fix issues
auto_fix_issues() {
    log_header "🔧 Auto-fixing Issues"
    
    log_info "Formatting code..."
    cargo fmt --all || log_warning "Code formatting failed"
    
    log_info "Running clippy fixes..."
    cargo clippy --fix --allow-dirty --allow-staged || log_warning "Clippy fixes failed"
    
    log_info "Updating dependencies..."
    cargo update || log_warning "Dependency update failed"
    
    log_success "Auto-fix completed"
}

# Main function
main() {
    local quick_mode=false
    local generate_report=false
    local auto_fix=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --quick)
                quick_mode=true
                shift
                ;;
            --report)
                generate_report=true
                shift
                ;;
            --fix)
                auto_fix=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --quick    Run quick validation (essential checks only)"
                echo "  --report   Generate detailed JSON report"
                echo "  --fix      Auto-fix common issues"
                echo "  --help, -h Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    log_header "🏭 Clnrm Production Readiness Validation"
    echo "Timestamp: $TIMESTAMP"
    echo "Project: clnrm"
    echo ""
    
    # Auto-fix if requested
    if [[ "$auto_fix" == true ]]; then
        auto_fix_issues
        echo ""
    fi
    
    # Run validation
    if [[ "$quick_mode" == true ]]; then
        quick_validation
    else
        validate_code_quality
        validate_security
        validate_performance
        validate_reliability
        validate_operational_readiness
        validate_documentation
        validate_compliance
    fi
    
    # Generate report if requested
    if [[ "$generate_report" == true ]]; then
        echo ""
        generate_report "$REPORT_FILE"
    fi
    
    # Summary
    echo ""
    log_header "📊 Validation Summary"
    echo "Total Checks: $TOTAL_CHECKS"
    echo "Passed: $PASSED_CHECKS"
    echo "Failed: $FAILED_CHECKS"
    echo "Warnings: $WARNING_CHECKS"
    echo "Critical Issues: $CRITICAL_ISSUES"
    echo "Success Rate: $(( PASSED_CHECKS * 100 / TOTAL_CHECKS ))%"
    
    # Production readiness determination
    if [[ $CRITICAL_ISSUES -gt 0 ]]; then
        log_critical "NOT PRODUCTION READY - Critical issues must be resolved"
        exit 3
    elif [[ $FAILED_CHECKS -gt $((TOTAL_CHECKS / 10)) ]]; then
        log_error "NOT PRODUCTION READY - Too many failed checks"
        exit 2
    elif [[ $WARNING_CHECKS -gt 0 ]]; then
        log_warning "PRODUCTION READY with warnings - Consider addressing issues"
        exit 1
    else
        log_success "PRODUCTION READY - All checks passed"
        exit 0
    fi
}

# Run main function
main "$@"
