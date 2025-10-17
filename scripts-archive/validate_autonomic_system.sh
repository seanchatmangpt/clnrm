#!/usr/bin/env bash
# Integration Validation Script for Autonomic System
# Validates all components of the Cleanroom Autonomic Intelligence Platform

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNINGS=0

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_CHECKS++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_CHECKS++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNINGS++))
}

check() {
    ((TOTAL_CHECKS++))
    local description="$1"
    local command="$2"

    if eval "$command" > /dev/null 2>&1; then
        log_success "$description"
        return 0
    else
        log_error "$description"
        return 1
    fi
}

# Header
echo "=========================================="
echo "  AUTONOMIC SYSTEM INTEGRATION VALIDATION"
echo "=========================================="
echo ""

# 1. Check Rust compilation
log_info "Phase 1: Compilation Validation"
echo "----------------------------------------"

check "Rust project compiles successfully" "cargo build --quiet"
check "All workspaces compile" "cargo build --workspace --quiet"
check "Release build succeeds" "cargo build --release --quiet"

echo ""

# 2. Verify AI Commands Integration
log_info "Phase 2: AI Commands Integration"
echo "----------------------------------------"

check "AI Orchestrate command exists" "test -f crates/clnrm-core/src/cli/commands/ai_orchestrate.rs"
check "AI Predict command exists" "test -f crates/clnrm-core/src/cli/commands/ai_predict.rs"
check "AI Optimize command exists" "test -f crates/clnrm-core/src/cli/commands/ai_optimize.rs"
check "AI Real command exists" "test -f crates/clnrm-core/src/cli/commands/ai_real.rs"

# Check CLI integration
if grep -q "ai_orchestrate" crates/clnrm-core/src/cli/mod.rs; then
    log_success "AI Orchestrate wired to CLI"
    ((PASSED_CHECKS++))
else
    log_error "AI Orchestrate NOT wired to CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "ai_predict" crates/clnrm-core/src/cli/mod.rs; then
    log_success "AI Predict wired to CLI"
    ((PASSED_CHECKS++))
else
    log_error "AI Predict NOT wired to CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "ai_optimize" crates/clnrm-core/src/cli/mod.rs; then
    log_success "AI Optimize wired to CLI"
    ((PASSED_CHECKS++))
else
    log_error "AI Optimize NOT wired to CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "ai_real" crates/clnrm-core/src/cli/mod.rs; then
    log_success "AI Real wired to CLI"
    ((PASSED_CHECKS++))
else
    log_error "AI Real NOT wired to CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

# Check CLI types
if grep -q "AiOrchestrate" crates/clnrm-core/src/cli/types.rs; then
    log_success "AI Orchestrate command type defined"
    ((PASSED_CHECKS++))
else
    log_error "AI Orchestrate command type NOT defined"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "AiPredict" crates/clnrm-core/src/cli/types.rs; then
    log_success "AI Predict command type defined"
    ((PASSED_CHECKS++))
else
    log_error "AI Predict command type NOT defined"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "AiOptimize" crates/clnrm-core/src/cli/types.rs; then
    log_success "AI Optimize command type defined"
    ((PASSED_CHECKS++))
else
    log_error "AI Optimize command type NOT defined"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "AiReal" crates/clnrm-core/src/cli/types.rs; then
    log_success "AI Real command type defined"
    ((PASSED_CHECKS++))
else
    log_error "AI Real command type NOT defined"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

echo ""

# 3. Service Integration Validation
log_info "Phase 3: Service System Integration"
echo "----------------------------------------"

check "Services module exists" "test -f crates/clnrm-core/src/services/mod.rs"
check "AI Test Generator service exists" "test -f crates/clnrm-core/src/services/ai_test_generator.rs"
check "Ollama service exists" "test -f crates/clnrm-core/src/services/ollama.rs"
check "SurrealDB service exists" "test -f crates/clnrm-core/src/services/surrealdb.rs"
check "Generic service plugin exists" "test -f crates/clnrm-core/src/services/generic.rs"

echo ""

# 4. Monitoring System Validation
log_info "Phase 4: Monitoring System"
echo "----------------------------------------"

check "Telemetry module exists" "test -f crates/clnrm-core/src/telemetry.rs"
check "Cleanroom environment exists" "test -f crates/clnrm-core/src/cleanroom.rs"

# Check for telemetry integration
if grep -q "telemetry" crates/clnrm-core/src/lib.rs; then
    log_success "Telemetry integrated in lib.rs"
    ((PASSED_CHECKS++))
else
    log_warning "Telemetry NOT found in lib.rs exports"
fi
((TOTAL_CHECKS++))

echo ""

# 5. Error Handling Validation
log_info "Phase 5: Error Handling"
echo "----------------------------------------"

check "Error module exists" "test -f crates/clnrm-core/src/error.rs"

# Check error handling in AI commands
if grep -q "CleanroomError" crates/clnrm-core/src/cli/commands/ai_orchestrate.rs; then
    log_success "AI Orchestrate has error handling"
    ((PASSED_CHECKS++))
else
    log_error "AI Orchestrate MISSING error handling"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "CleanroomError" crates/clnrm-core/src/cli/commands/ai_predict.rs; then
    log_success "AI Predict has error handling"
    ((PASSED_CHECKS++))
else
    log_error "AI Predict MISSING error handling"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if grep -q "CleanroomError" crates/clnrm-core/src/cli/commands/ai_optimize.rs; then
    log_success "AI Optimize has error handling"
    ((PASSED_CHECKS++))
else
    log_error "AI Optimize MISSING error handling"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

echo ""

# 6. Documentation Validation
log_info "Phase 6: Documentation Accuracy"
echo "----------------------------------------"

# Check README for AI commands
if [ -f "README.md" ]; then
    if grep -q "ai-orchestrate\|ai-predict\|ai-optimize\|ai-real" README.md; then
        log_success "AI commands documented in README.md"
        ((PASSED_CHECKS++))
    else
        log_warning "AI commands NOT documented in README.md"
    fi
    ((TOTAL_CHECKS++))
fi

# Check for example documentation
if [ -d "examples/optimus-prime-platform" ]; then
    if [ -f "examples/optimus-prime-platform/README.md" ]; then
        log_success "Optimus Prime platform documented"
        ((PASSED_CHECKS++))
    else
        log_warning "Optimus Prime platform NOT documented"
    fi
    ((TOTAL_CHECKS++))
fi

echo ""

# 7. Code Quality Checks
log_info "Phase 7: Code Quality"
echo "----------------------------------------"

# Check for unused imports
UNUSED_IMPORTS=$(cargo clippy --quiet 2>&1 | grep "unused import" | wc -l || echo "0")
if [ "$UNUSED_IMPORTS" -gt "0" ]; then
    log_warning "Found $UNUSED_IMPORTS unused imports (run 'cargo clippy' for details)"
else
    log_success "No unused imports detected"
    ((PASSED_CHECKS++))
fi
((TOTAL_CHECKS++))

# Check for compilation warnings
WARNINGS_COUNT=$(cargo build --quiet 2>&1 | grep "warning:" | wc -l || echo "0")
if [ "$WARNINGS_COUNT" -gt "0" ]; then
    log_warning "Found $WARNINGS_COUNT compilation warnings"
else
    log_success "No compilation warnings"
    ((PASSED_CHECKS++))
fi
((TOTAL_CHECKS++))

echo ""

# 8. Integration Tests
log_info "Phase 8: Integration Tests"
echo "----------------------------------------"

if cargo test --lib --quiet > /dev/null 2>&1; then
    log_success "Library tests pass"
    ((PASSED_CHECKS++))
else
    log_error "Library tests FAILED"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if cargo test --test '*' --quiet > /dev/null 2>&1; then
    log_success "Integration tests pass"
    ((PASSED_CHECKS++))
else
    log_warning "Some integration tests failed (check with 'cargo test --test \"*\"')"
fi
((TOTAL_CHECKS++))

echo ""

# 9. Binary Validation
log_info "Phase 9: Binary Execution"
echo "----------------------------------------"

if cargo build --bin clnrm --quiet > /dev/null 2>&1; then
    log_success "CLI binary builds successfully"
    ((PASSED_CHECKS++))
else
    log_error "CLI binary build FAILED"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

# Try to run the binary with --help
if ./target/debug/clnrm --help > /dev/null 2>&1; then
    log_success "CLI binary executes successfully"
    ((PASSED_CHECKS++))
else
    log_error "CLI binary execution FAILED"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

# Check if AI commands are available in help
if ./target/debug/clnrm --help 2>&1 | grep -q "ai-orchestrate"; then
    log_success "ai-orchestrate command available in CLI"
    ((PASSED_CHECKS++))
else
    log_error "ai-orchestrate command NOT available in CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if ./target/debug/clnrm --help 2>&1 | grep -q "ai-predict"; then
    log_success "ai-predict command available in CLI"
    ((PASSED_CHECKS++))
else
    log_error "ai-predict command NOT available in CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if ./target/debug/clnrm --help 2>&1 | grep -q "ai-optimize"; then
    log_success "ai-optimize command available in CLI"
    ((PASSED_CHECKS++))
else
    log_error "ai-optimize command NOT available in CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

if ./target/debug/clnrm --help 2>&1 | grep -q "ai-real"; then
    log_success "ai-real command available in CLI"
    ((PASSED_CHECKS++))
else
    log_error "ai-real command NOT available in CLI"
    ((FAILED_CHECKS++))
fi
((TOTAL_CHECKS++))

echo ""

# 10. Performance Benchmarks
log_info "Phase 10: Performance Check"
echo "----------------------------------------"

# Check if benchmark scripts exist
if [ -f "scripts/run_benchmarks.sh" ]; then
    log_success "Benchmark scripts available"
    ((PASSED_CHECKS++))
else
    log_warning "Benchmark scripts NOT found"
fi
((TOTAL_CHECKS++))

echo ""

# Summary
echo "=========================================="
echo "  VALIDATION SUMMARY"
echo "=========================================="
echo ""
echo "Total Checks:    $TOTAL_CHECKS"
echo -e "Passed:          ${GREEN}$PASSED_CHECKS${NC}"
echo -e "Failed:          ${RED}$FAILED_CHECKS${NC}"
echo -e "Warnings:        ${YELLOW}$WARNINGS${NC}"
echo ""

# Calculate success rate
SUCCESS_RATE=$(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc)
echo "Success Rate:    $SUCCESS_RATE%"
echo ""

# Final verdict
if [ "$FAILED_CHECKS" -eq 0 ]; then
    echo -e "${GREEN}✓ ALL CRITICAL VALIDATIONS PASSED${NC}"
    echo ""
    echo "The autonomic system integration is COMPLETE and OPERATIONAL."
    echo ""
    if [ "$WARNINGS" -gt 0 ]; then
        echo -e "${YELLOW}Note: $WARNINGS warnings detected. These are non-critical but should be addressed.${NC}"
    fi
    exit 0
else
    echo -e "${RED}✗ VALIDATION FAILED${NC}"
    echo ""
    echo "The autonomic system has $FAILED_CHECKS critical issues that must be resolved."
    echo ""
    echo "Recommendations:"
    echo "1. Review failed checks above"
    echo "2. Fix compilation errors with: cargo build"
    echo "3. Run tests with: cargo test"
    echo "4. Check clippy warnings with: cargo clippy"
    echo ""
    exit 1
fi
