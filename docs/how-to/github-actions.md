# How-To: Integrate with GitHub Actions

**Problem**: Want to run clnrm tests automatically on every push/PR
**Solution**: Add workflow file to `.github/workflows/`

## Quick Answer

```yaml
# .github/workflows/test.yml
name: clnrm Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install clnrm
        run: |
          brew tap seanchatmangpt/clnrm
          brew install clnrm

      - name: Run tests
        run: |
          CLNRM_ENABLE_POOLING=1 \
          clnrm run --parallel --jobs 8
```

## Complete Workflow (Production Ready)

```yaml
# .github/workflows/test.yml
name: Integration Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Set up Docker
        if: matrix.os == 'ubuntu-latest'
        uses: docker-practice/actions-setup-docker@master

      - name: Install clnrm (macOS)
        if: matrix.os == 'macos-latest'
        run: |
          brew tap seanchatmangpt/clnrm
          brew install clnrm

      - name: Install clnrm (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          curl -L https://github.com/seanchatmangpt/clnrm/releases/download/v1.4.1/clnrm-x86_64-unknown-linux-gnu > /tmp/clnrm
          chmod +x /tmp/clnrm
          sudo mv /tmp/clnrm /usr/local/bin/clnrm

      - name: Run integration tests
        run: |
          CLNRM_ENABLE_POOLING=1 \
          clnrm run --parallel --jobs 8

      - name: Generate test report
        if: always()
        run: |
          clnrm run --output junit > test-results.xml

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test-results.xml
```

## With Test Reports

```yaml
      - name: Run tests with JUnit output
        run: |
          CLNRM_ENABLE_POOLING=1 \
          clnrm run --output junit > test-results.xml

      - name: Publish test results
        if: always()
        uses: EnricoMi/publish-unit-test-result-action@v2
        with:
          files: test-results.xml
          check_name: Test Results
```

## With Matrix Testing

Test multiple configurations:

```yaml
strategy:
  matrix:
    config:
      - { pool_size: 2, jobs: 2 }
      - { pool_size: 10, jobs: 8 }

steps:
  - name: Run tests
    env:
      CLNRM_POOL_SIZE: ${{ matrix.config.pool_size }}
    run: |
      CLNRM_ENABLE_POOLING=1 \
      clnrm run --parallel --jobs ${{ matrix.config.jobs }}
```

## Conditional Execution

```yaml
# Only run on certain events
on:
  push:
    paths:
      - 'tests/**'
      - 'src/**'
      - '.github/workflows/**'

# Skip CI with git commit message
# To skip: add "[skip ci]" to commit message
```

## Debugging Failed Tests

```yaml
      - name: Run tests (verbose)
        if: failure()
        run: |
          CLNRM_ENABLE_POOLING=1 \
          clnrm run --verbose --fail-fast

      - name: Upload logs
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-logs
          path: .clnrm/logs/
```

## Best Practices

### 1. Cache Dependencies

```yaml
      - name: Cache Docker images
        uses: docker/setup-buildx-action@v2
```

### 2. Fail Fast for Quick Feedback

```yaml
      - name: Run tests (stop on first failure)
        run: |
          CLNRM_ENABLE_POOLING=1 \
          clnrm run -x  # -x = fail-fast
```

### 3. Parallel + Pooling

```yaml
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16
```

### 4. Different Strategies for PR vs Push

```yaml
      - name: Run full test suite (main branch)
        if: github.ref == 'refs/heads/main'
        run: CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16

      - name: Run quick tests (PR)
        if: github.ref != 'refs/heads/main'
        run: CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 8 -x
```

## Troubleshooting

### Error: Docker not available

Use docker-in-docker action:
```yaml
      - name: Set up Docker
        uses: docker/setup-buildx-action@v2
```

### Error: Command not found (clnrm)

Ensure clnrm is installed before running:
```yaml
      - name: Install clnrm
        run: |
          brew tap seanchatmangpt/clnrm
          brew install clnrm
          clnrm --version
```

### Tests timeout

Increase timeout and reduce jobs:
```yaml
      - name: Run tests (with timeout)
        timeout-minutes: 30
        run: CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 4
```

## See Also

- [How-To: Parallel Execution](./parallel-execution.md)
- [How-To: Container Pooling Setup](./container-pooling-setup.md)
- [How-To: Test Reporting](./test-reporting.md)
