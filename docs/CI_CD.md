# CI/CD Integration with gVisor

Complete guide for integrating clnrm with gVisor backend in CI/CD pipelines.

**Target Audience**: DevOps engineers, CI/CD architects
**Time Required**: 30-60 minutes
**Prerequisites**: gVisor knowledge (see [SETUP.md](SETUP.md))

## Table of Contents

1. [Overview](#overview)
2. [GitHub Actions](#github-actions)
3. [Jenkins](#jenkins)
4. [GitLab CI](#gitlab-ci)
5. [Other CI Systems](#other-ci-systems)
6. [Best Practices](#best-practices)
7. [Optimization](#optimization)

---

## Overview

### gVisor Advantages in CI/CD

| Feature | Docker | gVisor |
|---------|--------|--------|
| **Daemon Required** | Yes (complex) | No (simple) |
| **Privileged Mode** | Yes (security risk) | No |
| **DinD Needed** | Yes | No |
| **Resource Usage** | High | Low |
| **Startup Time** | Slow | Fast |
| **Hermetic** | Partial | Complete |

### CI/CD Architecture

```
CI Pipeline
    ↓
Install gVisor (5 min)
    ↓
Cache OCI Images (optional)
    ↓
Run Tests with gVisor
    ├── Unit Tests (fast)
    ├── Integration Tests (medium)
    └── E2E Tests (slow)
    ↓
Collect Results
    ↓
Publish Reports
```

---

## GitHub Actions

### Minimal Setup

```yaml
name: Test
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

      - name: Install gVisor
        run: |
          curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
          echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
          sudo apt-get update
          sudo apt-get install -y runsc skopeo

      - name: Run Tests
        env:
          CLNRM_BACKEND: gvisor
        run: cargo test --all
```

### With Image Caching

```yaml
name: Test with Cache
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

      - name: Install gVisor
        run: |
          curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
          echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
          sudo apt-get update
          sudo apt-get install -y runsc skopeo

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/
            target/
          key: cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache OCI Images
        uses: actions/cache@v4
        with:
          path: ~/.cache/clnrm
          key: clnrm-images-${{ hashFiles('**/Cargo.toml') }}
          restore-keys: clnrm-images-

      - name: Pre-pull Images
        run: |
          mkdir -p ~/.cache/clnrm
          skopeo copy docker://alpine:latest oci://~/.cache/clnrm/alpine:latest || true
          skopeo copy docker://python:3.11-slim oci://~/.cache/clnrm/python:3.11-slim || true

      - name: Run Tests
        env:
          CLNRM_BACKEND: gvisor
          CLNRM_CACHE_DIR: ~/.cache/clnrm
        run: cargo test --all

      - name: Upload Coverage
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: target/test-results/
```

### Matrix Testing

```yaml
name: Matrix Tests
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

      - name: Cache Images
        uses: actions/cache@v4
        with:
          path: ~/.cache/clnrm
          key: clnrm-images-${{ hashFiles('**/Cargo.toml') }}

      - name: Run ${{ matrix.test-suite }} Tests
        env:
          CLNRM_BACKEND: gvisor
        run: |
          case ${{ matrix.test-suite }} in
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

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: results-${{ matrix.test-suite }}
          path: target/
```

---

## Jenkins

### Declarative Pipeline

```groovy
pipeline {
    agent any

    options {
        timeout(time: 1, unit: 'HOURS')
        timestamps()
        buildDiscarder(logRotator(numToKeepStr: '10'))
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    # Install gVisor
                    curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
                    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
                    sudo apt-get update
                    sudo apt-get install -y runsc skopeo

                    # Verify installation
                    runsc --version
                '''
            }
        }

        stage('Unit Tests') {
            steps {
                sh '''
                    export CLNRM_BACKEND=gvisor
                    cargo test --lib
                '''
            }
        }

        stage('Integration Tests') {
            steps {
                sh '''
                    export CLNRM_BACKEND=gvisor
                    cargo test --test '*'
                '''
            }
        }

        stage('E2E Tests') {
            steps {
                sh '''
                    export CLNRM_BACKEND=gvisor
                    cargo test --test 'e2e_*'
                '''
            }
        }
    }

    post {
        always {
            junit 'target/**/test-results.xml'
            publishHTML([
                reportDir: 'target/coverage',
                reportFiles: 'index.html',
                reportName: 'Coverage Report'
            ])
        }
        failure {
            echo 'Tests failed!'
            sh 'sudo runsc --root /var/run/runsc list || true'
        }
    }
}
```

### Scripted Pipeline

```groovy
node {
    try {
        stage('Setup') {
            checkout scm

            sh '''
                curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
                echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
                sudo apt-get update
                sudo apt-get install -y runsc skopeo
            '''
        }

        stage('Test') {
            sh '''
                export CLNRM_BACKEND=gvisor
                cargo test --all
            '''
        }

        stage('Report') {
            junit 'target/**/test-results.xml'
        }

    } finally {
        // Cleanup
        sh '''
            sudo runsc --root /var/run/runsc delete -force $(sudo runsc --root /var/run/runsc list -quiet) || true
        '''
    }
}
```

---

## GitLab CI

### Basic Configuration

```yaml
image: ubuntu:22.04

stages:
  - test

variables:
  CLNRM_BACKEND: gvisor

before_script:
  - apt-get update
  - apt-get install -y curl gpg apt-transport-https rustc cargo
  - curl -fsSL https://gvisor.dev/archive.key | gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
  - echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | tee /etc/apt/sources.list.d/gvisor.list
  - apt-get update
  - apt-get install -y runsc skopeo

test:unit:
  stage: test
  script:
    - cargo test --lib
  artifacts:
    reports:
      junit: target/*/test-*.xml

test:integration:
  stage: test
  script:
    - cargo test --test '*'
  artifacts:
    reports:
      junit: target/*/test-*.xml

test:e2e:
  stage: test
  script:
    - cargo test --test 'e2e_*'
  artifacts:
    reports:
      junit: target/*/test-*.xml

cache:
  paths:
    - target/
    - ~/.cargo/
    - ~/.cache/clnrm/
  key: cargo-${{ ci_pipeline_id }}
```

---

## Other CI Systems

### CircleCI

```yaml
version: 2.1

jobs:
  test:
    machine:
      image: ubuntu-2204:current
    steps:
      - checkout
      - run:
          name: Install gVisor
          command: |
            curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
            echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
            sudo apt-get update
            sudo apt-get install -y runsc skopeo
      - run:
          name: Run Tests
          command: |
            export CLNRM_BACKEND=gvisor
            cargo test --all
      - store_test_results:
          path: target/

workflows:
  test:
    jobs:
      - test
```

### Travis CI

```yaml
language: rust
rust:
  - stable

before_install:
  - curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
  - echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
  - sudo apt-get update
  - sudo apt-get install -y runsc skopeo

env:
  - CLNRM_BACKEND=gvisor

script:
  - cargo test --all

after_success:
  - |
    if [ "$TRAVIS_PULL_REQUEST" = "false" ]; then
      echo "Running on master branch"
    fi
```

---

## Best Practices

### 1. Fail Fast

```yaml
- name: Quick Unit Tests
  run: cargo test --lib

- name: Integration Tests
  if: success()
  run: cargo test --test '*'

- name: E2E Tests
  if: success()
  run: cargo test --test 'e2e_*'
```

### 2. Parallel Execution

```yaml
strategy:
  matrix:
    test-suite: [unit, integration, e2e]
  max-parallel: 3

steps:
  - name: Run Tests
    run: |
      case ${{ matrix.test-suite }} in
        unit) cargo test --lib ;;
        integration) cargo test --test '*' ;;
        e2e) cargo test --test 'e2e_*' ;;
      esac
```

### 3. Clean Containers

```yaml
- name: Cleanup Containers
  if: always()
  run: |
    sudo runsc --root /var/run/runsc delete -force $(sudo runsc --root /var/run/runsc list -quiet) || true
```

### 4. Resource Management

```yaml
- name: Monitor Resources
  if: always()
  run: |
    echo "=== Disk Usage ==="
    du -sh ~/.cache/clnrm || true
    echo "=== Memory Usage ==="
    free -h
```

---

## Optimization

### Image Caching Strategy

```yaml
- name: Cache OCI Images
  uses: actions/cache@v4
  with:
    path: ~/.cache/clnrm
    key: clnrm-images-${{ hashFiles('**/Cargo.toml') }}
    restore-keys: |
      clnrm-images-

- name: Warm Up Cache
  run: |
    mkdir -p ~/.cache/clnrm
    # Pre-pull critical images
    for image in alpine:latest python:3.11-slim surrealdb/surrealdb:latest; do
      skopeo copy "docker://$image" "oci://~/.cache/clnrm/${image// /}" || true
    done
```

### Parallelization

```yaml
- name: Run Tests in Parallel
  env:
    CLNRM_BACKEND: gvisor
  run: cargo test --all -- --test-threads=$(nproc)
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
    restore-keys: cargo-
```

---

## Troubleshooting

### gVisor Installation Fails

```yaml
- name: Install gVisor (Retry)
  run: |
    for i in {1..3}; do
      apt-get update && apt-get install -y runsc && break || sleep 30
    done
```

### Image Pull Timeout

```yaml
- name: Pre-pull Images
  run: |
    for image in alpine:latest python:3.11-slim; do
      for i in {1..3}; do
        skopeo copy "docker://$image" "oci://~/.cache/clnrm/${image// /}" && break || sleep 30
      done
    done
```

### Stuck Containers

```yaml
- name: Kill Stuck Containers
  if: always()
  run: |
    sudo pkill -f runsc || true
    sudo runsc --root /var/run/runsc delete -force --all || true
```

---

## Security

### Secret Management

```yaml
- name: Run Tests
  env:
    REGISTRY_TOKEN: ${{ secrets.REGISTRY_TOKEN }}
  run: cargo test --all
```

### Signed Artifacts

```yaml
- name: Sign Test Results
  run: |
    gpg --sign test-results.xml
```

---

## Monitoring

### Performance Metrics

```yaml
- name: Collect Metrics
  run: |
    /usr/bin/time -v cargo test --all 2>&1 | tee test-metrics.log

- name: Upload Metrics
  uses: actions/upload-artifact@v4
  with:
    name: metrics
    path: test-metrics.log
```

---

**For more information, see**:
- [SETUP.md](SETUP.md) - Installation guide
- [TESTING.md](TESTING.md) - Testing guide
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development setup

