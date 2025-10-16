# CI/CD Integration Guide for Testing

## Overview

Comprehensive CI/CD integration ensures all test types run automatically on every code change, providing fast feedback and preventing regressions.

## GitHub Actions Workflows

### 1. Standard Test Suite

**File**: `.github/workflows/test.yml`

```yaml
name: Test Suite

on:
  push:
    branches: [main, master, develop]
  pull_request:
    branches: [main, master]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib

      - name: Run doc tests
        run: cargo test --doc

  integration-tests:
    runs-on: ubuntu-latest
    services:
      docker:
        image: docker:dind
        options: --privileged

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Start test services
        run: docker-compose -f tests/integration/docker-compose.test.yml up -d

      - name: Run integration tests
        run: cargo test --test '*' -- --test-threads=1

      - name: Stop test services
        if: always()
        run: docker-compose -f tests/integration/docker-compose.test.yml down -v

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml --output-dir coverage/

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          fail_ci_if_error: true
```

### 2. Property-Based Tests

**File**: `.github/workflows/property-tests.yml`

```yaml
name: Property-Based Tests

on:
  push:
    branches: [main, master]
  pull_request:
  schedule:
    - cron: '0 2 * * *'  # Nightly at 2 AM

jobs:
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable

      - name: Quick property tests (PR)
        if: github.event_name == 'pull_request'
        run: cargo test --test property_tests

      - name: Thorough property tests (main/nightly)
        if: github.event_name != 'pull_request'
        run: PROPTEST_CASES=10000 cargo test --test property_tests

      - name: Upload failure artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: proptest-regressions
          path: proptest-regressions/
```

### 3. Mutation Tests

**File**: `.github/workflows/mutation-tests.yml`

```yaml
name: Mutation Testing

on:
  push:
    branches: [main, master]
  schedule:
    - cron: '0 3 * * 0'  # Weekly on Sunday at 3 AM

jobs:
  mutation-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked

      - name: Run mutation tests
        run: |
          cargo mutants \
            --timeout-multiplier 3.0 \
            --jobs 4 \
            --output mutation-report \
            --json mutation-report.json

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: mutation-report/

      - name: Check mutation score
        run: |
          SCORE=$(jq -r '.mutation_score' mutation-report.json)
          echo "Mutation score: $SCORE"
          if (( $(echo "$SCORE < 0.70" | bc -l) )); then
            echo "Mutation score below threshold (70%)"
            exit 1
          fi
```

### 4. Fuzz Tests

**File**: `.github/workflows/fuzz.yml`

```yaml
name: Fuzz Testing

on:
  schedule:
    - cron: '0 0 * * *'  # Nightly
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - fuzz_toml_parser
          - fuzz_scenario_dsl
          - fuzz_cli_args
          - fuzz_error_handling
          - fuzz_regex_patterns

    steps:
      - uses: actions/checkout@v3

      - name: Install nightly Rust
        uses: dtolnay/rust-toolchain@nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Cache corpus
        uses: actions/cache@v3
        with:
          path: tests/fuzz/corpus/${{ matrix.target }}
          key: fuzz-corpus-${{ matrix.target }}

      - name: Run fuzzer
        working-directory: tests/fuzz
        run: |
          cargo +nightly fuzz run ${{ matrix.target }} \
            -- -max_total_time=600 \
            -timeout=10

      - name: Upload crash artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-crashes-${{ matrix.target }}
          path: tests/fuzz/artifacts/${{ matrix.target }}/

      - name: Upload corpus
        uses: actions/upload-artifact@v3
        with:
          name: corpus-${{ matrix.target }}
          path: tests/fuzz/corpus/${{ matrix.target }}/
```

### 5. Chaos Tests

**File**: `.github/workflows/chaos-tests.yml`

```yaml
name: Chaos Engineering Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Nightly
  workflow_dispatch:

jobs:
  chaos-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable

      - name: Run chaos tests
        run: cargo test --features chaos-testing

      - name: Upload chaos metrics
        uses: actions/upload-artifact@v3
        with:
          name: chaos-metrics
          path: target/chaos-metrics.json
```

## Test Execution Strategy

### Pull Request Tests (Fast Feedback)

- Unit tests
- Integration tests (critical paths)
- Quick property-based tests (256 cases)
- Linting and formatting
- **Target**: < 10 minutes

### Main Branch Tests (Comprehensive)

- All unit tests
- All integration tests
- Property-based tests (10,000 cases)
- Contract tests
- **Target**: < 30 minutes

### Nightly Tests (Exhaustive)

- Mutation tests
- Fuzz tests (10+ minutes per target)
- Chaos engineering tests
- Performance benchmarks
- **Target**: < 2 hours

## Pre-commit Hooks

**File**: `.git/hooks/pre-commit`

```bash
#!/bin/bash

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt -- --check || {
    echo "Error: Code not formatted. Run 'cargo fmt'"
    exit 1
}

# Clippy
echo "Running clippy..."
cargo clippy -- -D warnings || {
    echo "Error: Clippy found issues"
    exit 1
}

# Fast tests
echo "Running fast tests..."
cargo test --lib || {
    echo "Error: Tests failed"
    exit 1
}

echo "Pre-commit checks passed!"
```

## Branch Protection Rules

### Recommended Settings

```yaml
# .github/branch-protection.yml
branches:
  - name: main
    protection:
      required_status_checks:
        strict: true
        contexts:
          - unit-tests
          - integration-tests
          - coverage
      required_pull_request_reviews:
        required_approving_review_count: 1
      enforce_admins: false
      restrictions: null
```

## Test Result Reporting

### JUnit Reports

```yaml
- name: Generate test reports
  run: cargo test -- --format junit > test-results.xml

- name: Publish test results
  uses: EnricoMi/publish-unit-test-result-action@v2
  if: always()
  with:
    files: test-results.xml
```

### Coverage Reports

```yaml
- name: Generate coverage
  run: cargo tarpaulin --out Html --output-dir coverage/

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: ./coverage/cobertura.xml

- name: Publish coverage report
  uses: actions/upload-artifact@v3
  with:
    name: coverage-report
    path: coverage/
```

## Performance Tracking

### Benchmark Comparison

```yaml
- name: Run benchmarks
  run: cargo bench --bench cleanroom_benchmarks -- --save-baseline pr

- name: Compare with main
  run: |
    git fetch origin main
    git checkout origin/main
    cargo bench --bench cleanroom_benchmarks -- --save-baseline main
    git checkout -
    cargo bench --bench cleanroom_benchmarks -- --baseline main
```

## Optimization Tips

### 1. Caching

```yaml
- name: Cache Rust dependencies
  uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### 2. Parallel Execution

```yaml
strategy:
  matrix:
    test-type: [unit, integration, property]
  max-parallel: 3

steps:
  - name: Run ${{ matrix.test-type }} tests
    run: cargo test --test ${{ matrix.test-type }}
```

### 3. Incremental Builds

```yaml
env:
  CARGO_INCREMENTAL: 1
```

## Notifications

### Slack Integration

```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Tests failed on ${{ github.ref }}'
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
