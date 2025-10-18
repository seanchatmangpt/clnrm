#!/usr/bin/env bash
# Test the fake scanner by creating test files with known issues
# Version: 1.0

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly TEST_DIR="/tmp/clnrm-fake-scanner-test-$$"

# Color codes
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

log_info() {
    echo -e "${BLUE}[TEST]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*"
}

cleanup() {
    rm -rf "$TEST_DIR"
}

trap cleanup EXIT

# Create test directory structure
setup_test_env() {
    mkdir -p "$TEST_DIR/src"
    mkdir -p "$TEST_DIR/tests"

    # Create Cargo.toml to make it look like a Rust project
    cat > "$TEST_DIR/Cargo.toml" <<EOF
[package]
name = "test-project"
version = "0.1.0"
EOF
}

# Test 1: Should detect unimplemented!
test_unimplemented_detection() {
    log_info "Test 1: Detecting unimplemented! macros"

    cat > "$TEST_DIR/src/lib.rs" <<'EOF'
pub fn incomplete_function() -> Result<(), String> {
    unimplemented!("This should be detected")
}
EOF

    if bash "$SCRIPT_DIR/scan-fakes.sh" "$TEST_DIR" >/dev/null 2>&1; then
        log_error "Test 1 FAILED: Should have detected unimplemented!"
        return 1
    else
        log_success "Test 1 PASSED: Correctly detected unimplemented!"
        return 0
    fi
}

# Test 2: Should detect fake returns
test_fake_returns() {
    log_info "Test 2: Detecting fake return values"

    cat > "$TEST_DIR/src/service.rs" <<'EOF'
pub fn start_service() -> Result<ServiceHandle, Error> {
    Ok(fake_handle())
}
EOF

    if bash "$SCRIPT_DIR/scan-fakes.sh" "$TEST_DIR" >/dev/null 2>&1; then
        log_error "Test 2 FAILED: Should have detected fake return"
        return 1
    else
        log_success "Test 2 PASSED: Correctly detected fake return"
        return 0
    fi
}

# Test 3: Should detect println! in production
test_println_detection() {
    log_info "Test 3: Detecting println! in production code"

    cat > "$TEST_DIR/src/handler.rs" <<'EOF'
pub fn handle_request() -> Result<(), Error> {
    println!("Handling request");
    Ok(())
}
EOF

    if bash "$SCRIPT_DIR/scan-fakes.sh" "$TEST_DIR" >/dev/null 2>&1; then
        log_error "Test 3 FAILED: Should have detected println!"
        return 1
    else
        log_success "Test 3 PASSED: Correctly detected println!"
        return 0
    fi
}

# Test 4: Should pass on clean code
test_clean_code() {
    log_info "Test 4: Passing clean code"

    cat > "$TEST_DIR/src/clean.rs" <<'EOF'
use tracing::info;

pub fn proper_implementation() -> Result<(), Error> {
    info!("Starting operation");
    let result = perform_operation()?;
    Ok(result)
}

fn perform_operation() -> Result<(), Error> {
    Ok(())
}
EOF

    if bash "$SCRIPT_DIR/scan-fakes.sh" "$TEST_DIR" >/dev/null 2>&1; then
        log_success "Test 4 PASSED: Clean code accepted"
        return 0
    else
        log_error "Test 4 FAILED: Should have passed clean code"
        return 1
    fi
}

# Test 5: Should ignore test files
test_ignore_test_files() {
    log_info "Test 5: Ignoring patterns in test files"

    cat > "$TEST_DIR/tests/integration_test.rs" <<'EOF'
#[test]
fn test_something() {
    panic!("Test panic should be ignored");
    let result = something().unwrap();
    println!("Debug output in tests is OK");
}
EOF

    if bash "$SCRIPT_DIR/scan-fakes.sh" "$TEST_DIR" >/dev/null 2>&1; then
        log_success "Test 5 PASSED: Test files correctly ignored"
        return 0
    else
        log_error "Test 5 FAILED: Should ignore test files"
        return 1
    fi
}

# Main test execution
main() {
    local exit_code=0

    echo "=============================================="
    echo "  Fake Scanner Test Suite"
    echo "=============================================="
    echo ""

    setup_test_env

    # Run all tests
    test_unimplemented_detection || exit_code=1
    test_fake_returns || exit_code=1
    test_println_detection || exit_code=1
    test_clean_code || exit_code=1
    test_ignore_test_files || exit_code=1

    echo ""
    echo "=============================================="
    if [[ $exit_code -eq 0 ]]; then
        log_success "All scanner tests PASSED"
    else
        log_error "Some scanner tests FAILED"
    fi
    echo "=============================================="

    return $exit_code
}

main "$@"
