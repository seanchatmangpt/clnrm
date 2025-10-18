#!/usr/bin/env bash
# Cleanroom Testing Framework - Fake Implementation Scanner
# Adapted from KCura's scan_fakes.sh for clnrm patterns
# Detects stub implementations, unimplemented code, and fake success returns
# Version: 1.0

set -euo pipefail

# Script metadata
readonly SCRIPT_NAME="scan-fakes"
readonly SCRIPT_VERSION="1.0"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
readonly SCRIPT_TIMEOUT="10s"
readonly MAX_RETRIES=2

# Color codes for output
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" >&2
}

# Input validation
validate_input() {
    local root_dir="${1:-.}"

    # Sanitize input path
    root_dir="$(cd "$root_dir" && pwd 2>/dev/null || echo "$root_dir")"

    if [[ ! -d "$root_dir" ]]; then
        log_error "Input validation failed: '$root_dir' is not a valid directory"
        return 1
    fi

    # Check for path traversal attempts
    if [[ "$root_dir" =~ \.\. ]] && [[ "$root_dir" != *"$PROJECT_ROOT"* ]]; then
        log_error "Input validation failed: Path traversal detected in '$root_dir'"
        return 1
    fi

    echo "$root_dir"
    return 0
}

# Check if ripgrep is available
check_dependencies() {
    if ! command -v rg >/dev/null 2>&1; then
        log_error "ripgrep (rg) not found - required for pattern scanning"
        log_info "Install ripgrep: https://github.com/BurntSushi/ripgrep#installation"
        return 1
    fi
    return 0
}

# Safe pattern matching with timeout and detailed results
scan_pattern_safely() {
    local pattern="$1"
    local description="$2"
    local root_dir="$3"
    local timeout="$4"
    local exclusions=("$@")
    exclusions=("${exclusions[@]:4}")

    # Validate pattern
    if [[ -z "$pattern" ]]; then
        log_warn "Empty pattern provided for $description"
        return 0
    fi

    log_info "Scanning: $description"

    # Build exclusion arguments
    local exclude_args=()
    for excl in "${exclusions[@]}"; do
        exclude_args+=(--glob "!${excl}")
    done

    # Default exclusions
    exclude_args+=(
        --glob '!scripts/**'
        --glob '!target/**'
        --glob '!.git/**'
        --glob '!*.md'
        --glob '!Cargo.lock'
    )

    local retry_count=0
    local found_issues=false

    while [[ $retry_count -lt $MAX_RETRIES ]]; do
        # Capture matches with file locations
        local matches
        if matches=$(timeout "$timeout" rg -n "$pattern" "$root_dir" \
            --type rust \
            "${exclude_args[@]}" \
            --no-heading \
            --color never \
            2>/dev/null || true); then

            if [[ -n "$matches" ]]; then
                found_issues=true
                log_error "Fake pattern detected: $description"
                echo "----------------------------------------"
                echo "$matches"
                echo "----------------------------------------"
                return 1
            fi
        else
            # Check if timeout occurred
            if [[ $? -eq 124 ]]; then
                log_warn "Pattern scan timed out: $description"
                retry_count=$((retry_count + 1))
                if [[ $retry_count -lt $MAX_RETRIES ]]; then
                    log_info "Retrying... (attempt $((retry_count + 1))/$MAX_RETRIES)"
                    sleep 1
                    continue
                fi
            fi
            break
        fi
        break
    done

    if [[ "$found_issues" == "false" ]]; then
        log_success "Clean: $description"
    fi

    return 0
}

# Check for specific clnrm anti-patterns
check_clnrm_antipatterns() {
    local root_dir="$1"
    local issues_found=0

    log_info "Checking clnrm-specific anti-patterns..."

    # Check for println! in production code (should use tracing)
    log_info "Checking for println! in production code..."
    local println_matches
    if println_matches=$(rg -n 'println!' "$root_dir" \
        --type rust \
        --glob '!crates/clnrm-core/examples/**' \
        --glob '!crates/clnrm-core/tests/**' \
        --glob '!crates/clnrm/tests/**' \
        --glob '!**/test*.rs' \
        --glob '!target/**' \
        --glob '!scripts/**' \
        --no-heading \
        --color never \
        2>/dev/null || true); then

        if [[ -n "$println_matches" ]]; then
            log_error "println! found in production code (use tracing instead)"
            echo "----------------------------------------"
            echo "$println_matches"
            echo "----------------------------------------"
            issues_found=1
        else
            log_success "Clean: No println! in production code"
        fi
    fi

    # Check for fake ServiceHandle returns
    log_info "Checking for fake ServiceHandle implementations..."
    if rg -n 'ServiceHandle::fake|ServiceHandle::default' "$root_dir" \
        --type rust \
        --glob '!crates/clnrm-core/tests/**' \
        --glob '!**/test*.rs' \
        --glob '!target/**' \
        --no-heading \
        2>/dev/null; then
        log_error "Fake ServiceHandle implementation found"
        issues_found=1
    else
        log_success "Clean: No fake ServiceHandle implementations"
    fi

    # Check for empty Ok(()) stubs
    log_info "Checking for stub Ok(()) implementations..."
    local stub_pattern='^\s*(?:pub\s+)?(?:async\s+)?fn\s+\w+[^{]*\{\s*(?://.*\n\s*)*Ok\(\(\)\)\s*\}|^\s*(?:pub\s+)?(?:async\s+)?fn\s+\w+[^{]*\{\s*(?://.*\n\s*)*println!.*\n\s*Ok\(\(\)\)\s*\}'
    if rg -U -n "$stub_pattern" "$root_dir" \
        --type rust \
        --glob '!crates/clnrm-core/examples/**' \
        --glob '!crates/clnrm-core/tests/**' \
        --glob '!**/test*.rs' \
        --glob '!target/**' \
        --no-heading \
        2>/dev/null; then
        log_error "Stub Ok(()) implementation found"
        issues_found=1
    else
        log_success "Clean: No stub Ok(()) implementations"
    fi

    return $issues_found
}

# Main execution
main() {
    local root_dir="${1:-$PROJECT_ROOT}"
    local exit_code=0

    log_info "Starting Cleanroom Fake Scanner v$SCRIPT_VERSION"
    log_info "Project root: $root_dir"

    # Validate dependencies
    if ! check_dependencies; then
        exit 1
    fi

    # Validate and sanitize input
    if ! VALIDATED_ROOT="$(validate_input "$root_dir")"; then
        log_error "Input validation failed for directory: $root_dir"
        exit 1
    fi

    log_info "Scanning directory: $VALIDATED_ROOT"
    echo ""

    # Define patterns to detect
    # Pattern 1: unimplemented!, todo!, panic! macros
    if ! scan_pattern_safely \
        '\b(unimplemented!|todo!|panic!)' \
        "Unimplemented/TODO/panic macros" \
        "$VALIDATED_ROOT" \
        "$SCRIPT_TIMEOUT"; then
        exit_code=1
    fi

    # Pattern 2: Fake return values (dummy, fake, stub, placeholder, mock)
    if ! scan_pattern_safely \
        '\b(dummy|fake|stub|placeholder)' \
        "Fake/dummy/stub return values" \
        "$VALIDATED_ROOT" \
        "$SCRIPT_TIMEOUT"; then
        exit_code=1
    fi

    # Pattern 3: Hardcoded/canned/mock responses
    if ! scan_pattern_safely \
        '(hardcoded|canned).*(?:response|result|value)(?!.*test)' \
        "Hardcoded/canned responses" \
        "$VALIDATED_ROOT" \
        "$SCRIPT_TIMEOUT"; then
        exit_code=1
    fi

    # Pattern 4: .unwrap() and .expect() in production code
    if ! scan_pattern_safely \
        '\.(unwrap|expect)\(' \
        "unwrap/expect in production code" \
        "$VALIDATED_ROOT" \
        "$SCRIPT_TIMEOUT" \
        'crates/clnrm-core/examples/**' \
        'crates/clnrm-core/tests/**' \
        'crates/clnrm/tests/**' \
        '**/test*.rs'; then
        exit_code=1
    fi

    # Check clnrm-specific anti-patterns
    if ! check_clnrm_antipatterns "$VALIDATED_ROOT"; then
        exit_code=1
    fi

    echo ""

    # Summary
    if [[ $exit_code -eq 0 ]]; then
        log_success "Scan completed successfully - no fake patterns detected"
        log_success "All production code paths validated"
    else
        log_error "Scan completed with issues - fake patterns or anti-patterns detected"
        log_error "Please fix issues before merging to production"
    fi

    return $exit_code
}

# Execute with error handling
if ! main "$@"; then
    log_error "Fake implementation scan FAILED"
    exit 1
fi

log_success "Fake implementation scan PASSED"
exit 0
