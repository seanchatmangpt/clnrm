#!/usr/bin/env bash
# ==============================================================================
# check-best-practices.sh
# Core Team Best Practices Validation Script
#
# This script enforces FAANG-level code standards for clnrm:
#   1. No unwrap/expect in production code
#   2. No async trait methods (dyn compatibility)
#   3. Zero clippy warnings
#   4. AAA test pattern
#   5. No false green implementations
#   6. Proper error handling
#   7. Type safety
#
# Usage:
#   ./scripts/check-best-practices.sh           # Run all checks
#   ./scripts/check-best-practices.sh --fix     # Auto-fix issues where possible
#   ./scripts/check-best-practices.sh --ci      # CI mode (strict, no colors)
#
# Exit Codes:
#   0 - All checks passed
#   1 - Critical violations found
#   2 - Warning violations found (non-blocking)
# ==============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors (disabled in CI mode)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
FIX_MODE=false
CI_MODE=false
VERBOSE=false

# Counters
CRITICAL_VIOLATIONS=0
WARNING_VIOLATIONS=0
CHECKS_PASSED=0
CHECKS_TOTAL=0

# ==============================================================================
# Helper Functions
# ==============================================================================

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Core Team Best Practices Validation Script for clnrm

OPTIONS:
    --fix       Auto-fix issues where possible (cargo fmt, cargo fix)
    --ci        CI mode (strict, no colors, fail on warnings)
    --verbose   Show detailed output for each check
    -h, --help  Show this help message

CHECKS PERFORMED:
    1. No unwrap/expect in production code (src/ only)
    2. No async trait methods (dyn compatibility)
    3. Zero clippy warnings with -D warnings
    4. AAA test pattern compliance
    5. No false green Ok(()) stubs
    6. Proper Result<T, CleanroomError> error handling
    7. Type safety verification

EXIT CODES:
    0 - All checks passed
    1 - Critical violations found
    2 - Warning violations found (CI mode only)

EXAMPLES:
    $0                    # Run all checks
    $0 --fix             # Auto-fix formatting and clippy issues
    $0 --ci              # CI mode (strict, fail on warnings)
    $0 --verbose         # Detailed output

EOF
    exit 0
}

log_info() {
    if [ "$CI_MODE" = false ]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    else
        echo "[INFO] $*"
    fi
}

log_success() {
    if [ "$CI_MODE" = false ]; then
        echo -e "${GREEN}[PASS]${NC} $*"
    else
        echo "[PASS] $*"
    fi
}

log_warning() {
    if [ "$CI_MODE" = false ]; then
        echo -e "${YELLOW}[WARN]${NC} $*" >&2
    else
        echo "[WARN] $*" >&2
    fi
}

log_error() {
    if [ "$CI_MODE" = false ]; then
        echo -e "${RED}[FAIL]${NC} $*" >&2
    else
        echo "[FAIL] $*" >&2
    fi
}

check_start() {
    ((CHECKS_TOTAL++))
    log_info "Check $CHECKS_TOTAL: $1"
}

check_pass() {
    ((CHECKS_PASSED++))
    log_success "$1"
}

check_fail() {
    ((CRITICAL_VIOLATIONS++))
    log_error "$1"
}

check_warn() {
    ((WARNING_VIOLATIONS++))
    log_warning "$1"
}

# ==============================================================================
# Check 1: No unwrap/expect in Production Code
# ==============================================================================

check_no_unwrap_in_production() {
    check_start "No unwrap/expect in production code"

    # Search for unwrap/expect in src/ directories (production code)
    local violations=0
    local temp_file=$(mktemp)

    # Find all unwrap/expect in src/ directories
    grep -rn "\.unwrap()\|\.expect(" \
        crates/clnrm/src/ \
        crates/clnrm-core/src/ \
        crates/clnrm-shared/src/ \
        2>/dev/null | \
        grep -v "\.rs:.*//.*unwrap\|\.rs:.*//.*expect" | \
        grep -v "#\[cfg(test)\]" | \
        grep -v "/tests/" > "$temp_file" || true

    violations=$(wc -l < "$temp_file" | tr -d ' ')

    if [ "$violations" -gt 0 ]; then
        check_fail "Found $violations unwrap/expect in production code:"
        if [ "$VERBOSE" = true ]; then
            cat "$temp_file"
        else
            head -10 "$temp_file"
            if [ "$violations" -gt 10 ]; then
                echo "... and $((violations - 10)) more (use --verbose to see all)"
            fi
        fi
        rm "$temp_file"
        return 1
    fi

    rm "$temp_file"
    check_pass "No unwrap/expect found in production code"
    return 0
}

# ==============================================================================
# Check 2: No Async Trait Methods
# ==============================================================================

check_no_async_trait_methods() {
    check_start "No async trait methods (dyn compatibility)"

    # Search for "async fn" inside trait definitions
    local violations=0
    local temp_file=$(mktemp)

    # Find trait definitions with async methods
    # This is a heuristic check - looks for "trait" followed by "async fn"
    for file in $(find crates/*/src -name "*.rs" -type f); do
        # Use awk to find async fn within trait blocks
        awk '
            /^[[:space:]]*pub[[:space:]]+trait[[:space:]]|^[[:space:]]*trait[[:space:]]/ { in_trait=1; trait_start=NR }
            in_trait && /^}[[:space:]]*$/ { in_trait=0 }
            in_trait && /async[[:space:]]+fn/ {
                print FILENAME ":" NR ":" $0
                violations++
            }
        ' "$file" >> "$temp_file" 2>/dev/null || true
    done

    violations=$(wc -l < "$temp_file" | tr -d ' ')

    if [ "$violations" -gt 0 ]; then
        check_fail "Found $violations async trait methods (breaks dyn compatibility):"
        cat "$temp_file"
        rm "$temp_file"
        return 1
    fi

    rm "$temp_file"
    check_pass "No async trait methods found (dyn compatible)"
    return 0
}

# ==============================================================================
# Check 3: Zero Clippy Warnings
# ==============================================================================

check_clippy_warnings() {
    check_start "Zero clippy warnings (cargo clippy -- -D warnings)"

    local clippy_output=$(mktemp)

    if [ "$FIX_MODE" = true ]; then
        log_info "Running cargo clippy --fix..."
        cargo clippy --fix --allow-dirty --allow-staged 2>&1 | tee "$clippy_output" || true
    fi

    # Run clippy with -D warnings (treat warnings as errors)
    if cargo clippy -- -D warnings 2>&1 | tee "$clippy_output"; then
        check_pass "Zero clippy warnings"
        rm "$clippy_output"
        return 0
    else
        local warnings=$(grep -c "warning:" "$clippy_output" || echo "0")
        check_fail "Found clippy warnings/errors:"
        if [ "$VERBOSE" = true ]; then
            cat "$clippy_output"
        else
            head -20 "$clippy_output"
            echo "... (use --verbose to see all)"
        fi
        rm "$clippy_output"
        return 1
    fi
}

# ==============================================================================
# Check 4: AAA Test Pattern
# ==============================================================================

check_aaa_test_pattern() {
    check_start "AAA test pattern compliance"

    # Check that test functions have AAA comments
    local violations=0
    local temp_file=$(mktemp)

    # Find test functions without AAA structure
    # This is a heuristic - looks for #[test] or #[tokio::test] without Arrange/Act/Assert
    for file in $(find crates/*/tests crates/*/src -name "*.rs" -type f 2>/dev/null); do
        awk '
            /#\[(tokio::)?test\]/ {
                test_found=1
                test_line=NR
                next
            }
            test_found && /^[[:space:]]*async[[:space:]]+fn|^[[:space:]]*fn/ {
                test_name=$0
                test_found=0
                check_aaa=1
                aaa_found=0
                brace_count=0
                next
            }
            check_aaa && /{/ {
                brace_count++
            }
            check_aaa && /}/ {
                brace_count--
                if (brace_count == 0) {
                    if (aaa_found == 0) {
                        print FILENAME ":" test_line ": Test missing AAA pattern: " test_name
                        violations++
                    }
                    check_aaa=0
                }
            }
            check_aaa && /\/\/[[:space:]]*(Arrange|Act|Assert)/ {
                aaa_found++
            }
        ' "$file" >> "$temp_file" 2>/dev/null || true
    done

    violations=$(wc -l < "$temp_file" | tr -d ' ')

    if [ "$violations" -gt 10 ]; then
        check_warn "Found $violations tests potentially missing AAA pattern (showing first 10):"
        head -10 "$temp_file"
        rm "$temp_file"
        return 0  # Warning only, not blocking
    elif [ "$violations" -gt 0 ]; then
        check_warn "Found $violations tests potentially missing AAA pattern:"
        cat "$temp_file"
        rm "$temp_file"
        return 0  # Warning only, not blocking
    fi

    rm "$temp_file"
    check_pass "AAA test pattern compliance verified"
    return 0
}

# ==============================================================================
# Check 5: No False Green Implementations
# ==============================================================================

check_no_false_green() {
    check_start "No false green Ok(()) stubs"

    # Look for suspicious Ok(()) returns in production code
    local violations=0
    local temp_file=$(mktemp)

    # Find functions that just return Ok(()) without doing real work
    # This is a heuristic - looks for fn returning Result with only Ok(()) body
    for file in $(find crates/*/src -name "*.rs" -type f ! -path "*/tests/*"); do
        # Look for patterns like:
        # fn foo() -> Result<()> { Ok(()) }
        # fn foo() -> Result<()> { todo!() }  (acceptable)
        # fn foo() -> Result<()> { unimplemented!() }  (acceptable)
        grep -n "fn.*->.*Result.*{" "$file" 2>/dev/null | while read -r line; do
            line_num=$(echo "$line" | cut -d: -f1)
            # Check if next few lines are just Ok(()) without any real work
            awk -v start="$line_num" '
                NR >= start && NR <= start+5 {
                    if (/Ok\(\(\)\)/ && !/unimplemented!|todo!|println!|tracing::/) {
                        print FILENAME ":" start ": Suspicious Ok(()) - may be fake green"
                        exit
                    }
                }
            ' "$file" >> "$temp_file" 2>/dev/null || true
        done
    done

    violations=$(wc -l < "$temp_file" | tr -d ' ')

    if [ "$violations" -gt 0 ]; then
        check_warn "Found $violations potentially fake Ok(()) implementations:"
        cat "$temp_file"
        rm "$temp_file"
        return 0  # Warning only, requires manual review
    fi

    rm "$temp_file"
    check_pass "No obvious false green implementations found"
    return 0
}

# ==============================================================================
# Check 6: Proper Error Handling
# ==============================================================================

check_error_handling() {
    check_start "Proper Result<T, CleanroomError> error handling"

    # Check that functions use proper error types
    local violations=0
    local temp_file=$(mktemp)

    # Find Result types not using CleanroomError
    for file in $(find crates/clnrm-core/src -name "*.rs" -type f ! -path "*/tests/*"); do
        # Look for Result<T, E> where E is not CleanroomError
        grep -n "Result<.*,.*>" "$file" 2>/dev/null | \
            grep -v "Result<.*CleanroomError>" | \
            grep -v "Result<.*Box<dyn.*Error" | \
            grep -v "Result<.*std::io::Error" | \
            grep -v "^[[:space:]]*///" | \
            grep -v "^[[:space:]]*//" >> "$temp_file" || true
    done

    violations=$(wc -l < "$temp_file" | tr -d ' ')

    if [ "$violations" -gt 5 ]; then
        check_warn "Found $violations uses of non-CleanroomError Result types (showing first 5):"
        head -5 "$temp_file"
        rm "$temp_file"
        return 0  # Warning only, some cases are acceptable
    elif [ "$violations" -gt 0 ]; then
        check_warn "Found $violations uses of non-CleanroomError Result types:"
        cat "$temp_file"
        rm "$temp_file"
        return 0  # Warning only
    fi

    rm "$temp_file"
    check_pass "Proper error handling verified"
    return 0
}

# ==============================================================================
# Check 7: Formatting
# ==============================================================================

check_formatting() {
    check_start "Code formatting (cargo fmt)"

    if [ "$FIX_MODE" = true ]; then
        log_info "Running cargo fmt..."
        cargo fmt
        check_pass "Code formatted"
        return 0
    fi

    if cargo fmt -- --check 2>&1 >/dev/null; then
        check_pass "Code formatting correct"
        return 0
    else
        check_fail "Code formatting issues found. Run: cargo fmt"
        return 1
    fi
}

# ==============================================================================
# Main Execution
# ==============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --fix)
                FIX_MODE=true
                shift
                ;;
            --ci)
                CI_MODE=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                usage
                ;;
            *)
                echo "Unknown option: $1"
                usage
                ;;
        esac
    done

    log_info "Starting Core Team Best Practices Check"
    log_info "Project: clnrm"
    log_info "Mode: $([ "$CI_MODE" = true ] && echo "CI" || echo "Local")"
    echo ""

    # Run all checks
    check_formatting || true
    check_no_unwrap_in_production || true
    check_no_async_trait_methods || true
    check_clippy_warnings || true
    check_aaa_test_pattern || true
    check_no_false_green || true
    check_error_handling || true

    # Print summary
    echo ""
    log_info "==================== SUMMARY ===================="
    log_info "Total Checks: $CHECKS_TOTAL"
    log_success "Passed: $CHECKS_PASSED"

    if [ "$WARNING_VIOLATIONS" -gt 0 ]; then
        log_warning "Warnings: $WARNING_VIOLATIONS"
    fi

    if [ "$CRITICAL_VIOLATIONS" -gt 0 ]; then
        log_error "Critical Failures: $CRITICAL_VIOLATIONS"
    fi

    echo ""

    # Determine exit code
    if [ "$CRITICAL_VIOLATIONS" -gt 0 ]; then
        log_error "❌ CRITICAL VIOLATIONS FOUND - FAILED"
        exit 1
    elif [ "$CI_MODE" = true ] && [ "$WARNING_VIOLATIONS" -gt 0 ]; then
        log_error "❌ WARNINGS IN CI MODE - FAILED"
        exit 2
    else
        log_success "✅ ALL CHECKS PASSED"
        exit 0
    fi
}

# Run main
main "$@"
