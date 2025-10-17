# Mutation Testing Guide for CLNRM

## Table of Contents
1. [Introduction](#introduction)
2. [What is Mutation Testing?](#what-is-mutation-testing)
3. [Setup and Installation](#setup-and-installation)
4. [Running Mutation Tests](#running-mutation-tests)
5. [Interpreting Results](#interpreting-results)
6. [Best Practices](#best-practices)
7. [CI/CD Integration](#cicd-integration)

## Introduction

Mutation testing is an advanced testing technique that evaluates the effectiveness of your test suite by introducing small changes (mutations) to your code and checking if your tests catch these changes.

## What is Mutation Testing?

### Core Concept

Mutation testing works by:
1. **Creating mutants**: Making small, deliberate changes to your code
2. **Running tests**: Executing your test suite against each mutant
3. **Analyzing results**:
   - **Killed mutant**: Tests detected the change (good!)
   - **Survived mutant**: Tests didn't detect the change (test gap!)
   - **Timeout/Error**: Mutant caused infinite loop or crash

### Mutation Operators

#### Arithmetic Operators
```rust
// Original
let result = a + b;

// Mutants
let result = a - b;  // Replace + with -
let result = a * b;  // Replace + with *
let result = a / b;  // Replace + with /
```

#### Logical Operators
```rust
// Original
if x && y { }

// Mutants
if x || y { }  // Replace && with ||
if x { }       // Remove second condition
if y { }       // Remove first condition
```

#### Relational Operators
```rust
// Original
if value > threshold { }

// Mutants
if value >= threshold { }  // Change boundary
if value < threshold { }   // Reverse operator
if value == threshold { }  // Change to equality
```

#### Return Value Mutations
```rust
// Original
fn get_status() -> bool {
    true
}

// Mutant
fn get_status() -> bool {
    false  // Flip boolean return
}
```

## Setup and Installation

### Rust (cargo-mutants)

```bash
# Install cargo-mutants
cargo install cargo-mutants --locked

# Verify installation
cargo mutants --version
```

### TypeScript/JavaScript (Stryker)

```bash
# Install Stryker (per project)
npm install --save-dev @stryker-mutator/core \
    @stryker-mutator/typescript-checker \
    @stryker-mutator/jest-runner

# Verify installation
npx stryker --version
```

## Running Mutation Tests

### Quick Start

```bash
# Run all mutation tests (Rust + TypeScript)
./scripts/run-mutation-tests.sh

# Run Rust tests only
./scripts/run-mutation-tests.sh --rust-only

# Run TypeScript tests only
./scripts/run-mutation-tests.sh --typescript-only
```

### Rust Mutation Testing

```bash
# Basic run
cargo mutants

# With configuration
cargo mutants \
    --timeout-multiplier 3.0 \
    --jobs 4 \
    --output docs/mutation-reports/rust \
    --json docs/mutation-reports/rust/report.json

# Test specific crate
cargo mutants -p clnrm-core

# Test specific file
cargo mutants --file src/backend/testcontainer.rs
```

### TypeScript Mutation Testing

```bash
# Change to project directory
cd examples/optimus-prime-platform

# Run Stryker
npx stryker run

# With specific configuration
npx stryker run --config stryker.conf.json

# Incremental run (only changed files)
npx stryker run --incremental
```

## Interpreting Results

### Mutation Score

```
Mutation Score = (Killed Mutants / Total Mutants) × 100%
```

**Score Interpretation:**
- **90-100%**: Excellent - Very strong test suite
- **80-89%**: Good - Solid coverage with few gaps
- **70-79%**: Acceptable - Room for improvement
- **60-69%**: Weak - Significant test gaps
- **<60%**: Poor - Major test coverage issues

### Example Output Analysis

#### Rust (cargo-mutants)

```
Summary:
  Total mutants: 150
  Killed: 120
  Survived: 25
  Timeout: 5

Mutation Score: 80.0%

Survived mutants:
  src/backend/mod.rs:45: replaced > with >= in Backend::validate
  src/policy.rs:78: removed return value in Policy::check
  ...
```

**What to do:**
1. Review survived mutants
2. Add tests for uncovered scenarios
3. Strengthen existing assertions

#### TypeScript (Stryker)

```html
Mutant killed: 145/180 (80.56%)
Mutant timeout: 5/180 (2.78%)
Mutant survived: 30/180 (16.67%)

High-risk survivors:
  - ArithmeticOperator: src/utils/calculations.ts:23
  - ConditionalExpression: src/components/validator.ts:56
```

## Best Practices

### 1. Start Small
Begin with critical modules rather than the entire codebase:
```bash
# Test one module first
cargo mutants --file src/backend/testcontainer.rs
```

### 2. Focus on High-Value Code
Prioritize mutation testing for:
- Core business logic
- Security-critical code
- Frequently changed modules
- Complex algorithms

### 3. Set Realistic Goals
Don't aim for 100% mutation score:
- **Critical code**: 85-90%
- **Important code**: 75-85%
- **Supporting code**: 65-75%
- **Utility code**: 60-70%

### 4. Review Survivors Strategically

**High Priority Survivors:**
- Boundary condition checks
- Error handling paths
- Security validations
- State transitions

**Low Priority Survivors:**
- Logging statements
- Debug code
- Trivial getters/setters
- Format strings

### 5. Exclude Appropriately

Some code doesn't need mutation testing:
```toml
# .cargo-mutants.toml
exclude_globs = [
    "*/tests/*",
    "*/examples/*",
    "**/main.rs",
    "**/*_test.rs"
]

exclude_re = [
    "fmt",        # Debug formatting
    "clone",      # Simple trait implementations
    "default",    # Default constructors
]
```

### 6. Optimize Performance

```bash
# Use parallel execution
cargo mutants --jobs 4

# Set appropriate timeout
cargo mutants --timeout-multiplier 3.0

# Skip obviously safe code
cargo mutants --skip-calls-unsafe
```

### 7. Iterate and Improve

1. **Run baseline**: Establish current mutation score
2. **Analyze survivors**: Identify test gaps
3. **Add tests**: Cover uncovered scenarios
4. **Re-run**: Verify improvements
5. **Repeat**: Continuous improvement

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Mutation Testing

on:
  pull_request:
    branches: [main, master]
  push:
    branches: [main, master]

jobs:
  mutation-testing-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked

      - name: Run mutation tests
        run: |
          cargo mutants \
            --timeout-multiplier 3.0 \
            --jobs 4 \
            --output mutation-report \
            --json mutation-report.json

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: mutation-report/

      - name: Check mutation score
        run: |
          # Parse JSON and check score
          SCORE=$(jq '.mutation_score' mutation-report.json)
          if (( $(echo "$SCORE < 70" | bc -l) )); then
            echo "Mutation score too low: $SCORE%"
            exit 1
          fi

  mutation-testing-typescript:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: examples/optimus-prime-platform
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: npm ci

      - name: Run Stryker
        run: npx stryker run --reporters json,html

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: stryker-report
          path: examples/optimus-prime-platform/reports/
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run mutation tests on changed files only
CHANGED_RS_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -n "$CHANGED_RS_FILES" ]; then
    echo "Running mutation tests on changed Rust files..."
    for file in $CHANGED_RS_FILES; do
        if [[ ! $file =~ (test|example) ]]; then
            cargo mutants --file "$file" --no-shuffle || exit 1
        fi
    done
fi
```

## Configuration Files Reference

### Rust Configuration (.cargo-mutants.toml)

Located at: `/Users/sac/clnrm/docs/cargo-mutants-config.toml`

Key settings:
- `timeout_multiplier`: How long to wait for tests (default: 3.0)
- `jobs`: Parallel test runners (default: 4)
- `exclude_globs`: Files to skip
- `exclude_re`: Function patterns to skip

### TypeScript Configuration (stryker.conf.json)

Located at: `/Users/sac/clnrm/examples/optimus-prime-platform/stryker.conf.json`

Key settings:
- `mutate`: Files to mutate
- `thresholds`: Score thresholds (high/low/break)
- `timeoutMS`: Test timeout
- `mutator.excludedMutations`: Mutation types to skip

## Troubleshooting

### Common Issues

#### 1. Timeouts
**Problem**: Tests timeout for many mutants
**Solution**: Increase timeout multiplier
```bash
cargo mutants --timeout-multiplier 5.0
```

#### 2. Out of Memory
**Problem**: System runs out of memory
**Solution**: Reduce parallel jobs
```bash
cargo mutants --jobs 2
```

#### 3. False Survivors
**Problem**: Mutants survive due to weak assertions
**Solution**: Use more specific assertions
```rust
// Weak
assert!(result.is_ok());

// Strong
assert_eq!(result.unwrap(), expected_value);
```

#### 4. Too Many Mutants
**Problem**: Mutation testing takes too long
**Solution**:
- Exclude low-value code
- Run incrementally
- Focus on changed files

## Resources

- [cargo-mutants Documentation](https://mutants.rs/)
- [Stryker Documentation](https://stryker-mutator.io/)
- [Mutation Testing Best Practices](https://en.wikipedia.org/wiki/Mutation_testing)

## Support

For issues or questions:
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: https://github.com/seanchatmangpt/clnrm/docs

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
