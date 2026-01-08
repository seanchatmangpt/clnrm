# CI/CD Integration with gVisor

This chapter covers integrating clnrm v2.0.0 with gVisor backend into CI/CD pipelines. No Docker daemon required!

## GitHub Actions

### Basic Integration

```yaml
name: Integration Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install gVisor Runtime
        run: |
          curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
          echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
          sudo apt-get update
          sudo apt-get install -y runsc skopeo

      - name: Verify gVisor Installation
        run: |
          runsc --version
          skopeo --version

      - name: Run Tests with gVisor
        env:
          CLNRM_BACKEND: gvisor
        run: cargo test --all

      - name: Generate Report
        if: always()
        run: cargo test --all -- --format json > test-results.json
```

### Advanced Integration with Pre-cached Images

```yaml
name: Advanced Testing with Image Cache
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        test-suite: [unit, integration, e2e]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install gVisor
        run: |
          curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
          echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
          sudo apt-get update
          sudo apt-get install -y runsc skopeo

      - name: Cache OCI Images
        uses: actions/cache@v4
        with:
          path: ~/.cache/clnrm
          key: clnrm-images-${{ hashFiles('**/Cargo.toml') }}
          restore-keys: |
            clnrm-images-

      - name: Pre-pull Base Images
        run: |
          mkdir -p ~/.cache/clnrm
          skopeo copy docker://alpine:latest oci://~/.cache/clnrm/alpine:latest
          skopeo copy docker://python:3.11-slim oci://~/.cache/clnrm/python:3.11-slim
          skopeo copy docker://rust:latest oci://~/.cache/clnrm/rust:latest

      - name: Run ${{ matrix.test-suite }} Tests
        env:
          CLNRM_BACKEND: gvisor
          CLNRM_CACHE_DIR: ~/.cache/clnrm
        run: cargo test --${{ matrix.test-suite }}

      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results-${{ matrix.test-suite }}
          path: target/test-results/
```

## v2.0.0 CI/CD Improvements

### Environment Persistence

In v2.0.0, environment variables work correctly across test steps:

```yaml
# This now works in CI/CD
[containers.app]
image = "myapp:latest"
env = {
    "DATABASE_URL" = "postgresql://test:test@localhost:5432/testdb",
    "REDIS_URL" = "redis://localhost:6379"
}

[[steps]]
name = "migrate"
container = "app"
exec = ["./migrate"]

[[steps]]
name = "test"
container = "app"
exec = ["./test"]  # Env vars persist from container definition
```

### Parallel Execution

```yaml
# Optimized for CI/CD
clnrm run tests/ \
  --parallel \
  --jobs $(nproc) \
  --fail-fast \
  --timeout 30m
```

## v2.0.0 gVisor-specific CI/CD Improvements

### No Docker Daemon Required

gVisor runs directly without Docker daemon:

```yaml
# No need for:
# - Docker socket mounting
# - DinD (Docker in Docker)
# - Docker service startup
# - Docker daemon health checks

# Just install runsc and test!
```

### Resource Efficiency

gVisor uses less resources than Docker in CI/CD:

```yaml
- name: Install gVisor
  run: |
    # Lightweight installation
    curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
    sudo apt-get update
    sudo apt-get install -y runsc skopeo

    # No daemon startup needed
    # Tests run immediately
```

## Best Practices for gVisor in CI/CD

### 1. Image Caching

Cache OCI images for faster test runs:

```yaml
- name: Cache OCI Images
  uses: actions/cache@v4
  with:
    path: ~/.cache/clnrm
    key: clnrm-images-${{ hashFiles('**/Cargo.toml') }}
    restore-keys: clnrm-images-

- name: Pre-pull Images
  run: |
    mkdir -p ~/.cache/clnrm
    skopeo copy docker://alpine:latest oci://~/.cache/clnrm/alpine:latest
    skopeo copy docker://python:3.11-slim oci://~/.cache/clnrm/python:3.11-slim
    skopeo copy docker://surrealdb/surrealdb:latest oci://~/.cache/clnrm/surrealdb:latest
```

### 2. Parallel Test Execution

Run tests in parallel with gVisor:

```yaml
- name: Run Tests in Parallel
  env:
    CLNRM_BACKEND: gvisor
  run: cargo test --all -- --test-threads=$(nproc)
```

### 3. Test Organization

Organize tests by type for optimal CI/CD flow:

```yaml
strategy:
  matrix:
    test-type: [unit, integration, e2e]

steps:
  - name: Run ${{ matrix.test-type }} Tests
    env:
      CLNRM_BACKEND: gvisor
    run: |
      case ${{ matrix.test-type }} in
        unit)
          cargo test --lib
          ;;
        integration)
          cargo test --test '*'
          ;;
        e2e)
          cargo test --test 'e2e_*'
          ;;
      esac
```

### 4. Artifact Management

Collect and upload test results:

```yaml
- name: Run Tests
  env:
    CLNRM_BACKEND: gvisor
  run: cargo test --all -- --format json > test-results.json
  continue-on-error: true

- name: Upload Test Results
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: test-results-${{ matrix.test-type }}
    path: test-results.json

- name: Publish Test Report
  if: always()
  uses: dorny/test-reporter@v1
  with:
    name: Test Results (${{ matrix.test-type }})
    path: test-results.json
    reporter: 'json'
```

### 5. Resource Limits

Configure resource limits for CI/CD environment:

```yaml
- name: Run Tests with Resource Limits
  env:
    CLNRM_BACKEND: gvisor
    CLNRM_MEMORY_LIMIT_MB: 512
    CLNRM_CPU_LIMIT: 2.0
  run: cargo test --all

- name: Monitor Resource Usage
  if: always()
  run: |
    echo "Memory usage:"
    free -h
    echo "Disk usage:"
    du -sh ~/.cache/clnrm
```

## Troubleshooting gVisor in CI/CD

### Issue: gVisor Installation Timeout

**Symptom**: GitHub Actions times out during gVisor installation

**Solution**:
```yaml
- name: Install gVisor (with timeout)
  timeout-minutes: 10
  run: |
    apt-get update || echo "Update failed, continuing..."
    apt-get install -y runsc || apt-get install -y runsc
```

### Issue: Image Pull Timeout

**Symptom**: Tests timeout pulling images in CI

**Solution**:
```yaml
- name: Pre-pull Images with Retry
  run: |
    for image in alpine:latest python:3.11-slim surrealdb/surrealdb:latest; do
      for i in {1..3}; do
        skopeo copy "docker://$image" "oci://~/.cache/clnrm/${image// /}" && break || sleep 30
      done
    done
```

### Issue: Permission Denied

**Symptom**: Tests fail with "Permission denied" running gVisor

**Solution**:
```yaml
- name: Run Tests with gVisor
  run: |
    # gVisor requires privileged context
    sudo cargo test --all

    # Or allow sudo without password for runsc
    echo "$(whoami) ALL=(ALL) NOPASSWD: /usr/bin/runsc" | sudo tee /etc/sudoers.d/gvisor
    cargo test --all
```

### Issue: Too Many Open Files

**Symptom**: "Too many open files" error in CI

**Solution**:
```yaml
- name: Increase File Descriptor Limit
  run: |
    ulimit -n 4096
    cargo test --all
```

### Issue: Out of Disk Space

**Symptom**: CI fails with "No space left on device"

**Solution**:
```yaml
- name: Clear Image Cache
  if: always()
  run: rm -rf ~/.cache/clnrm/*

- name: Monitor Disk Usage
  run: |
    du -sh ~/.cache/clnrm
    df -h
```

## Enterprise Integration

### Jenkins Pipeline with gVisor

```groovy
pipeline {
    agent any

    stages {
        stage('Setup') {
            steps {
                sh '''
                    # Install gVisor
                    curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
                    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
                    sudo apt-get update
                    sudo apt-get install -y runsc skopeo
                '''
            }
        }

        stage('Test') {
            steps {
                sh '''
                    export CLNRM_BACKEND=gvisor
                    cargo test --all
                '''
            }
            post {
                always {
                    junit 'target/**/test-results.xml'
                }
            }
        }
    }
}
```

### GitLab CI with gVisor

```yaml
stages:
  - test

test:gvisor:
  stage: test
  image: ubuntu:22.04

  script:
    - apt-get update
    - apt-get install -y curl gpg apt-transport-https
    - curl -fsSL https://gvisor.dev/archive.key | gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
    - echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | tee /etc/apt/sources.list.d/gvisor.list
    - apt-get update
    - apt-get install -y runsc skopeo rustc cargo
    - export CLNRM_BACKEND=gvisor
    - cargo test --all

  artifacts:
    reports:
      junit: target/**/test-results.xml
    paths:
      - target/
```

## Performance Optimization for CI/CD

### Parallel Testing

Run different test suites in parallel:

```yaml
jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install gVisor
        run: # ... gVisor installation ...
      - name: Run Unit Tests
        run: cargo test --lib

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install gVisor
        run: # ... gVisor installation ...
      - name: Run Integration Tests
        run: cargo test --test '*'

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install gVisor
        run: # ... gVisor installation ...
      - name: Run E2E Tests
        run: cargo test --test 'e2e_*'
```

### Build Caching

```yaml
- name: Cache Cargo
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      target/
    key: cargo-${{ hashFiles('**/Cargo.lock') }}
```

## Monitoring and Alerting

### Test Duration Tracking

```yaml
- name: Track Test Duration
  run: |
    time cargo test --all 2>&1 | tee test-timing.log

- name: Upload Timing Data
  uses: actions/upload-artifact@v4
  with:
    name: test-timing
    path: test-timing.log
```

### Failure Notifications

```yaml
- name: Notify on Failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Tests failed in CI'
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
    fields: repo,message,commit,author
```

## Security in CI/CD

### Secure Credential Handling

```yaml
- name: Run Tests (Secure)
  env:
    CLNRM_BACKEND: gvisor
    REGISTRY_TOKEN: ${{ secrets.REGISTRY_TOKEN }}
  run: cargo test --all
```

### Container Image Verification

```yaml
- name: Verify Image Signatures
  run: |
    # Verify image signatures before using
    skopeo inspect docker://alpine:latest --format='{{.Digest}}'
```