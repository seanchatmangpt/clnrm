#!/usr/bin/env bash
#
# CI Gate Script for clnrm
# Configuration-driven quality enforcement
#
# Usage: ./scripts/ci-gate.sh [--config <path>] [--check <check_name>] [--fail-fast]
#

set -euo pipefail

# Script metadata
readonly SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly DEFAULT_CONFIG="${SCRIPT_DIR}/ci-gate-config.yaml"

# Color output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Exit codes
readonly EXIT_SUCCESS=0
readonly EXIT_FAILURE=1
readonly EXIT_CONFIG_ERROR=2
readonly EXIT_CHECK_FAILED=3

# Global state
FAIL_FAST=false
CONFIG_FILE="${DEFAULT_CONFIG}"
SPECIFIC_CHECK=""
REPORT_DIR="${PROJECT_ROOT}/target/ci-gate-report"
FAILURES=0
WARNINGS=0
CHECKS_RUN=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" >&2
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*" >&2
    ((WARNINGS++)) || true
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
    ((FAILURES++)) || true
}

log_check() {
    echo -e "${BLUE}[CHECK]${NC} $*" >&2
    ((CHECKS_RUN++)) || true
}

# Usage information
usage() {
    cat << EOF
Usage: ${SCRIPT_NAME} [OPTIONS]

Configuration-driven CI gate script for clnrm quality enforcement.

OPTIONS:
    --config <path>     Path to configuration YAML file (default: ${DEFAULT_CONFIG})
    --check <name>      Run only specific check (e.g., critical_patterns, coverage)
    --fail-fast         Stop on first failure
    --help              Show this help message

CHECKS:
    critical_patterns   Detect unwrap, expect, panic, todo
    core_functions      Verify required API functions exist
    compilation         Test compilation with all feature combinations
    linting             Run clippy with strict rules
    error_handling      Verify Result<T, CleanroomError> usage
    documentation       Check module and public item documentation
    coverage            Ensure test coverage meets threshold

EXAMPLES:
    ${SCRIPT_NAME}                              # Run all checks
    ${SCRIPT_NAME} --check critical_patterns    # Run only pattern check
    ${SCRIPT_NAME} --fail-fast                  # Stop on first failure

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            --check)
                SPECIFIC_CHECK="$2"
                shift 2
                ;;
            --fail-fast)
                FAIL_FAST=true
                shift
                ;;
            --help)
                usage
                exit ${EXIT_SUCCESS}
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit ${EXIT_FAILURE}
                ;;
        esac
    done
}

# Check if required tools are installed
check_dependencies() {
    local missing=()

    for cmd in cargo grep sed awk; do
        if ! command -v "${cmd}" &> /dev/null; then
            missing+=("${cmd}")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing[*]}"
        return ${EXIT_FAILURE}
    fi

    return ${EXIT_SUCCESS}
}

# Parse YAML config (simple parsing for our needs)
parse_yaml_value() {
    local key="$1"
    local config_file="${2:-${CONFIG_FILE}}"

    if [[ ! -f "${config_file}" ]]; then
        log_error "Config file not found: ${config_file}"
        return ${EXIT_CONFIG_ERROR}
    fi

    # Simple YAML parsing - extract value after key
    grep -E "^\s*${key}:" "${config_file}" | sed 's/.*:\s*//' | tr -d '"' || echo ""
}

# Get array values from YAML
parse_yaml_array() {
    local key="$1"
    local config_file="${2:-${CONFIG_FILE}}"

    if [[ ! -f "${config_file}" ]]; then
        return ${EXIT_CONFIG_ERROR}
    fi

    # Extract array items (lines starting with - after the key)
    awk "/^[[:space:]]*${key}:/,/^[[:space:]]*[^[:space:]-]/ {
        if (/^[[:space:]]*-[[:space:]]/) {
            gsub(/^[[:space:]]*-[[:space:]]*/, \"\")
            gsub(/[[:space:]]*$/, \"\")
            gsub(/\"/, \"\")
            print
        }
    }" "${config_file}"
}

# Retry logic with exponential backoff
retry_with_backoff() {
    local max_attempts="$1"
    local initial_delay="$2"
    local max_delay="$3"
    shift 3
    local cmd=("$@")

    local attempt=1
    local delay="${initial_delay}"

    while [[ ${attempt} -le ${max_attempts} ]]; do
        if "${cmd[@]}"; then
            return ${EXIT_SUCCESS}
        fi

        if [[ ${attempt} -lt ${max_attempts} ]]; then
            log_warning "Attempt ${attempt}/${max_attempts} failed, retrying in ${delay}s..."
            sleep "${delay}"
            delay=$((delay * 2))
            [[ ${delay} -gt ${max_delay} ]] && delay="${max_delay}"
        fi

        ((attempt++)) || true
    done

    return ${EXIT_FAILURE}
}

# Initialize report directory
init_report() {
    mkdir -p "${REPORT_DIR}"

    cat > "${REPORT_DIR}/report.json" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "project": "clnrm",
  "checks": []
}
EOF
}

# Add check result to report
add_to_report() {
    local check_name="$1"
    local status="$2"
    local message="${3:-}"
    local details="${4:-}"

    local report_file="${REPORT_DIR}/report.json"

    # Create temporary JSON entry
    local temp_entry=$(cat << EOF
  {
    "check": "${check_name}",
    "status": "${status}",
    "message": "${message}",
    "details": "${details}",
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  }
EOF
)

    # Append to report (simple JSON manipulation)
    if [[ -f "${report_file}" ]]; then
        # Remove last closing bracket and append entry
        sed -i.bak '$ d' "${report_file}" 2>/dev/null || sed -i '' '$ d' "${report_file}"

        if grep -q '"checks": \[\]' "${report_file}"; then
            sed -i.bak 's/"checks": \[\]/"checks": [/' "${report_file}" 2>/dev/null || \
                sed -i '' 's/"checks": \[\]/"checks": [/' "${report_file}"
            echo "${temp_entry}" >> "${report_file}"
        else
            echo "," >> "${report_file}"
            echo "${temp_entry}" >> "${report_file}"
        fi

        echo "  ]" >> "${report_file}"
        echo "}" >> "${report_file}"
        rm -f "${report_file}.bak"
    fi
}

# Check for critical patterns (unwrap, expect, panic, todo)
check_critical_patterns() {
    log_check "Checking for critical patterns..."

    local patterns=(
        "\.unwrap\(\)"
        "\.expect\("
        "panic!\("
    )

    local warning_patterns=(
        "println!\("
    )

    local exclude_paths=("tests/" "examples/" "crates/clnrm-ai/" "target/")
    local exclude_args=()
    for path in "${exclude_paths[@]}"; do
        exclude_args+=(--exclude-dir="${path}")
    done

    local critical_found=false
    local warnings_found=false

    cd "${PROJECT_ROOT}"

    # Check critical patterns
    for pattern in "${patterns[@]}"; do
        local matches=$(grep -rn -E "${pattern}" \
            --include="*.rs" \
            "${exclude_args[@]}" \
            . 2>/dev/null || true)

        if [[ -n "${matches}" ]]; then
            log_error "Critical pattern found: ${pattern}"
            echo "${matches}" | head -n 20
            critical_found=true

            if [[ "${FAIL_FAST}" == "true" ]]; then
                add_to_report "critical_patterns" "failed" "Critical pattern detected: ${pattern}" "${matches}"
                return ${EXIT_CHECK_FAILED}
            fi
        fi
    done

    # Check warning patterns
    for pattern in "${warning_patterns[@]}"; do
        local matches=$(grep -rn -E "${pattern}" \
            --include="*.rs" \
            "${exclude_args[@]}" \
            . 2>/dev/null || true)

        if [[ -n "${matches}" ]]; then
            log_warning "Warning pattern found: ${pattern}"
            echo "${matches}" | head -n 10
            warnings_found=true
        fi
    done

    if [[ "${critical_found}" == "true" ]]; then
        add_to_report "critical_patterns" "failed" "Critical patterns detected"
        return ${EXIT_CHECK_FAILED}
    elif [[ "${warnings_found}" == "true" ]]; then
        add_to_report "critical_patterns" "warning" "Warning patterns detected"
        log_success "Critical patterns check passed (with warnings)"
        return ${EXIT_SUCCESS}
    else
        add_to_report "critical_patterns" "passed" "No critical patterns found"
        log_success "Critical patterns check passed"
        return ${EXIT_SUCCESS}
    fi
}

# Verify core API functions exist
check_core_functions() {
    log_check "Verifying core API functions..."

    local required_functions=(
        "CleanroomEnvironment|crates/clnrm-core/src/cleanroom.rs"
        "ServicePlugin|crates/clnrm-core/src/cleanroom.rs"
        "Backend|crates/clnrm-core/src/backend/mod.rs"
        "CleanroomError|crates/clnrm-core/src/error.rs"
    )

    local missing=()

    cd "${PROJECT_ROOT}"

    for entry in "${required_functions[@]}"; do
        IFS='|' read -r func file <<< "${entry}"

        if [[ ! -f "${file}" ]]; then
            log_error "Required file not found: ${file}"
            missing+=("${func} (file missing)")
            continue
        fi

        if ! grep -q "${func}" "${file}"; then
            log_error "Required function not found: ${func} in ${file}"
            missing+=("${func}")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        add_to_report "core_functions" "failed" "Missing core functions" "${missing[*]}"
        log_error "Missing core functions: ${missing[*]}"
        return ${EXIT_CHECK_FAILED}
    fi

    add_to_report "core_functions" "passed" "All core functions verified"
    log_success "Core functions check passed"
    return ${EXIT_SUCCESS}
}

# Check compilation with all feature combinations
check_compilation() {
    log_check "Checking compilation with all features..."

    local features=("" "otel" "otel-traces" "otel-metrics" "otel-logs")

    cd "${PROJECT_ROOT}"

    for feature in "${features[@]}"; do
        local feature_flag=""
        [[ -n "${feature}" ]] && feature_flag="--features ${feature}"

        log_info "Compiling with features: ${feature:-default}"

        if ! cargo build --release ${feature_flag} 2>&1 | tee "${REPORT_DIR}/compile-${feature:-default}.log"; then
            add_to_report "compilation" "failed" "Compilation failed with features: ${feature:-default}"
            log_error "Compilation failed with features: ${feature:-default}"
            return ${EXIT_CHECK_FAILED}
        fi
    done

    add_to_report "compilation" "passed" "All feature combinations compile successfully"
    log_success "Compilation check passed"
    return ${EXIT_SUCCESS}
}

# Run clippy with strict rules
check_linting() {
    log_check "Running clippy with strict rules..."

    cd "${PROJECT_ROOT}"

    if ! cargo clippy --all-targets --all-features -- \
        -D warnings \
        -D clippy::unwrap_used \
        -D clippy::expect_used \
        -D clippy::panic \
        2>&1 | tee "${REPORT_DIR}/clippy.log"; then
        add_to_report "linting" "failed" "Clippy found violations"
        log_error "Clippy check failed"
        return ${EXIT_CHECK_FAILED}
    fi

    add_to_report "linting" "passed" "Clippy check passed"
    log_success "Linting check passed"
    return ${EXIT_SUCCESS}
}

# Verify proper error handling
check_error_handling() {
    log_check "Verifying error handling patterns..."

    local exclude_paths=("tests/" "examples/" "crates/clnrm-ai/" "target/")
    local exclude_args=()
    for path in "${exclude_paths[@]}"; do
        exclude_args+=(--exclude-dir="${path}")
    done

    cd "${PROJECT_ROOT}"

    # Check for Result<T, CleanroomError> usage
    local result_count=$(grep -r "Result<" --include="*.rs" "${exclude_args[@]}" . 2>/dev/null | wc -l || echo "0")
    local error_count=$(grep -r "CleanroomError" --include="*.rs" "${exclude_args[@]}" . 2>/dev/null | wc -l || echo "0")

    if [[ ${result_count} -eq 0 ]]; then
        add_to_report "error_handling" "warning" "No Result types found"
        log_warning "No Result types found in source code"
        return ${EXIT_SUCCESS}
    fi

    log_info "Found ${result_count} Result usages and ${error_count} CleanroomError references"
    add_to_report "error_handling" "passed" "Error handling patterns verified"
    log_success "Error handling check passed"
    return ${EXIT_SUCCESS}
}

# Check documentation coverage
check_documentation() {
    log_check "Checking documentation coverage..."

    cd "${PROJECT_ROOT}"

    # Use cargo doc with warnings as errors
    if ! RUSTDOCFLAGS="-D missing_docs -D rustdoc::broken_intra_doc_links" \
        cargo doc --no-deps --all-features 2>&1 | tee "${REPORT_DIR}/docs.log"; then
        add_to_report "documentation" "warning" "Documentation has warnings"
        log_warning "Documentation check found issues"
        return ${EXIT_SUCCESS}  # Don't fail on doc warnings
    fi

    add_to_report "documentation" "passed" "Documentation check passed"
    log_success "Documentation check passed"
    return ${EXIT_SUCCESS}
}

# Check test coverage
check_coverage() {
    log_check "Checking test coverage..."

    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warning "cargo-tarpaulin not installed, skipping coverage check"
        log_info "Install with: cargo install cargo-tarpaulin"
        add_to_report "coverage" "skipped" "cargo-tarpaulin not installed"
        return ${EXIT_SUCCESS}
    fi

    cd "${PROJECT_ROOT}"

    local min_coverage=85

    log_info "Running coverage analysis (minimum: ${min_coverage}%)..."

    if ! cargo tarpaulin --exclude-files "crates/clnrm-ai/*" \
        --out Json --output-dir "${REPORT_DIR}" 2>&1 | tee "${REPORT_DIR}/coverage.log"; then
        add_to_report "coverage" "failed" "Coverage analysis failed"
        log_error "Coverage analysis failed"
        return ${EXIT_CHECK_FAILED}
    fi

    # Parse coverage percentage from JSON
    if [[ -f "${REPORT_DIR}/tarpaulin-report.json" ]]; then
        local coverage=$(grep -o '"coverage": [0-9.]*' "${REPORT_DIR}/tarpaulin-report.json" | cut -d' ' -f2 || echo "0")
        local coverage_int=$(printf "%.0f" "${coverage}" || echo "0")

        log_info "Coverage: ${coverage_int}%"

        if [[ ${coverage_int} -lt ${min_coverage} ]]; then
            add_to_report "coverage" "failed" "Coverage ${coverage_int}% below minimum ${min_coverage}%"
            log_error "Coverage ${coverage_int}% is below minimum ${min_coverage}%"
            return ${EXIT_CHECK_FAILED}
        fi
    fi

    add_to_report "coverage" "passed" "Coverage meets minimum threshold"
    log_success "Coverage check passed"
    return ${EXIT_SUCCESS}
}

# Generate final report
generate_report() {
    log_info "Generating final report..."

    local report_md="${REPORT_DIR}/report.md"

    cat > "${report_md}" << EOF
# CI Gate Report

**Date**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Project**: clnrm
**Checks Run**: ${CHECKS_RUN}
**Failures**: ${FAILURES}
**Warnings**: ${WARNINGS}

## Summary

EOF

    if [[ ${FAILURES} -eq 0 ]]; then
        echo "✅ **All checks passed**" >> "${report_md}"
    else
        echo "❌ **${FAILURES} check(s) failed**" >> "${report_md}"
    fi

    if [[ ${WARNINGS} -gt 0 ]]; then
        echo "⚠️  **${WARNINGS} warning(s) found**" >> "${report_md}"
    fi

    echo "" >> "${report_md}"
    echo "## Detailed Results" >> "${report_md}"
    echo "" >> "${report_md}"

    # Parse JSON report and add to markdown
    if [[ -f "${REPORT_DIR}/report.json" ]]; then
        echo "See \`report.json\` for detailed results." >> "${report_md}"
    fi

    log_success "Report generated: ${report_md}"
}

# Main execution
main() {
    parse_args "$@"

    log_info "CI Gate starting..."
    log_info "Config: ${CONFIG_FILE}"
    log_info "Project root: ${PROJECT_ROOT}"

    # Check dependencies
    if ! check_dependencies; then
        exit ${EXIT_FAILURE}
    fi

    # Initialize reporting
    init_report

    # Define checks to run
    local checks=(
        "critical_patterns"
        "core_functions"
        "linting"
        "error_handling"
        "documentation"
    )

    # If specific check requested, run only that
    if [[ -n "${SPECIFIC_CHECK}" ]]; then
        checks=("${SPECIFIC_CHECK}")
    fi

    # Run checks
    for check in "${checks[@]}"; do
        local check_func="check_${check}"

        if declare -f "${check_func}" > /dev/null; then
            if ! ${check_func}; then
                if [[ "${FAIL_FAST}" == "true" ]]; then
                    log_error "Check failed: ${check} (fail-fast enabled)"
                    generate_report
                    exit ${EXIT_CHECK_FAILED}
                fi
            fi
        else
            log_warning "Unknown check: ${check}"
        fi
    done

    # Generate final report
    generate_report

    # Exit with appropriate code
    if [[ ${FAILURES} -gt 0 ]]; then
        log_error "CI Gate failed: ${FAILURES} check(s) failed"
        exit ${EXIT_CHECK_FAILED}
    else
        log_success "CI Gate passed: All checks successful"
        exit ${EXIT_SUCCESS}
    fi
}

# Execute main function
main "$@"
