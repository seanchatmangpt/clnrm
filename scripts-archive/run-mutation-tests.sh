#!/bin/bash
# Comprehensive Mutation Testing Runner for CLNRM Project
# This script orchestrates mutation testing across Rust and TypeScript components

set -e  # Exit on error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORTS_DIR="$PROJECT_ROOT/docs/mutation-reports"
RUST_REPORT_DIR="$REPORTS_DIR/rust"
TS_REPORT_DIR="$REPORTS_DIR/typescript"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo -e "${BLUE}=== CLNRM Mutation Testing Suite ===${NC}"
echo -e "Project Root: $PROJECT_ROOT"
echo -e "Reports Directory: $REPORTS_DIR"
echo -e "Timestamp: $TIMESTAMP\n"

# Create report directories
mkdir -p "$RUST_REPORT_DIR"
mkdir -p "$TS_REPORT_DIR"

# Function to print status
print_status() {
    echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to check if cargo-mutants is installed
check_cargo_mutants() {
    if ! command -v cargo-mutants &> /dev/null; then
        print_warning "cargo-mutants not found, installing..."
        cargo install cargo-mutants --locked
        print_success "cargo-mutants installed"
    else
        print_success "cargo-mutants is available"
    fi
}

# Function to run Rust mutation tests
run_rust_mutation_tests() {
    print_status "Starting Rust mutation testing..."

    cd "$PROJECT_ROOT"

    # Check if cargo-mutants is installed
    check_cargo_mutants

    # Run mutation tests with configuration
    print_status "Running cargo-mutants..."

    # Create a temporary config if custom one doesn't exist
    if [ ! -f ".cargo-mutants.toml" ] && [ -f "docs/cargo-mutants-config.toml" ]; then
        cp docs/cargo-mutants-config.toml .cargo-mutants.toml
    fi

    # Run mutation tests (this will take a while)
    if cargo mutants --no-shuffle \
        --timeout-multiplier 3.0 \
        --jobs 4 \
        --output "$RUST_REPORT_DIR/report_${TIMESTAMP}" \
        --json "$RUST_REPORT_DIR/report_${TIMESTAMP}.json" \
        2>&1 | tee "$RUST_REPORT_DIR/log_${TIMESTAMP}.txt"; then
        print_success "Rust mutation testing completed"
    else
        print_warning "Rust mutation testing completed with some failures"
    fi

    # Generate summary
    if [ -f "$RUST_REPORT_DIR/report_${TIMESTAMP}.json" ]; then
        print_status "Generating Rust mutation testing summary..."
        echo "Summary will be in: $RUST_REPORT_DIR/summary_${TIMESTAMP}.txt"
    fi
}

# Function to run TypeScript mutation tests
run_typescript_mutation_tests() {
    print_status "Starting TypeScript mutation testing..."

    # Test Optimus Prime Platform
    if [ -d "$PROJECT_ROOT/examples/optimus-prime-platform" ]; then
        print_status "Testing: Optimus Prime Platform"
        cd "$PROJECT_ROOT/examples/optimus-prime-platform"

        # Install Stryker if not present
        if ! npm list @stryker-mutator/core &> /dev/null; then
            print_warning "Installing Stryker..."
            npm install --save-dev @stryker-mutator/core \
                @stryker-mutator/typescript-checker \
                @stryker-mutator/jest-runner
        fi

        # Run Stryker (if tests exist)
        if [ -d "src" ]; then
            print_status "Running Stryker mutation tests..."
            if npx stryker run 2>&1 | tee "$TS_REPORT_DIR/optimus-prime_${TIMESTAMP}.log"; then
                print_success "Optimus Prime Platform mutation testing completed"
            else
                print_warning "Optimus Prime Platform mutation testing had issues"
            fi
        else
            print_warning "No test files found, skipping Stryker"
        fi
    fi

    # Test clnrm-case-study
    if [ -d "$PROJECT_ROOT/examples/clnrm-case-study" ]; then
        print_status "Testing: CLNRM Case Study"
        cd "$PROJECT_ROOT/examples/clnrm-case-study"

        # Similar process as above
        if [ -d "src" ]; then
            print_status "Running Stryker mutation tests..."
            # Note: Only run if stryker.conf.json exists
            if [ -f "stryker.conf.json" ]; then
                npx stryker run 2>&1 | tee "$TS_REPORT_DIR/case-study_${TIMESTAMP}.log" || true
            fi
        fi
    fi
}

# Function to generate comprehensive report
generate_comprehensive_report() {
    print_status "Generating comprehensive mutation testing report..."

    REPORT_FILE="$REPORTS_DIR/comprehensive_report_${TIMESTAMP}.md"

    cat > "$REPORT_FILE" << EOF
# CLNRM Mutation Testing Report
**Generated:** $(date)
**Timestamp:** $TIMESTAMP

## Executive Summary

This report summarizes the mutation testing results for the CLNRM project,
covering both Rust and TypeScript components.

## Rust Mutation Testing Results

### Configuration
- Tool: cargo-mutants
- Timeout Multiplier: 3.0
- Parallel Jobs: 4
- Report Directory: \`$RUST_REPORT_DIR\`

### Results
See detailed results in:
- JSON Report: \`report_${TIMESTAMP}.json\`
- Log File: \`log_${TIMESTAMP}.txt\`

## TypeScript Mutation Testing Results

### Optimus Prime Platform
See detailed results in:
- Log File: \`$TS_REPORT_DIR/optimus-prime_${TIMESTAMP}.log\`

### CLNRM Case Study
See detailed results in:
- Log File: \`$TS_REPORT_DIR/case-study_${TIMESTAMP}.log\`

## Recommendations

### Based on Mutation Testing Results:

1. **Improve Test Coverage**: Focus on untested code paths revealed by surviving mutants
2. **Strengthen Assertions**: Weak assertions allow mutants to survive
3. **Edge Case Testing**: Add tests for boundary conditions
4. **Error Path Testing**: Ensure error handling is thoroughly tested
5. **Integration Tests**: Add integration tests for critical paths

### Mutation Score Targets:

| Component Type | Target Score | Current Score |
|---------------|--------------|---------------|
| Core Modules  | 85%          | TBD           |
| Utilities     | 75%          | TBD           |
| CLI Commands  | 70%          | TBD           |
| Examples      | 60%          | TBD           |

## Next Steps

1. Review surviving mutants to identify test gaps
2. Add tests for uncovered scenarios
3. Increase assertion strength
4. Re-run mutation tests to verify improvements
5. Integrate into CI/CD pipeline

---
*Generated by CLNRM Mutation Testing Suite*
EOF

    print_success "Comprehensive report generated: $REPORT_FILE"
}

# Main execution
main() {
    print_status "Starting comprehensive mutation testing..."

    # Parse command line arguments
    RUN_RUST=true
    RUN_TS=true

    while [[ $# -gt 0 ]]; do
        case $1 in
            --rust-only)
                RUN_TS=false
                shift
                ;;
            --typescript-only)
                RUN_RUST=false
                shift
                ;;
            --help)
                echo "Usage: $0 [--rust-only | --typescript-only | --help]"
                echo ""
                echo "Options:"
                echo "  --rust-only         Run only Rust mutation tests"
                echo "  --typescript-only   Run only TypeScript mutation tests"
                echo "  --help              Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # Run tests based on flags
    if [ "$RUN_RUST" = true ]; then
        run_rust_mutation_tests
    fi

    if [ "$RUN_TS" = true ]; then
        run_typescript_mutation_tests
    fi

    # Generate comprehensive report
    generate_comprehensive_report

    print_success "Mutation testing complete!"
    echo -e "\nReports available in: ${BLUE}$REPORTS_DIR${NC}"
}

# Run main function
main "$@"
