# OpenTelemetry Integration Guide for CLNRM

**Complete Setup Guide for OpenTelemetry Trace Collection and Validation**

This guide walks you through setting up, configuring, and using OpenTelemetry (OTEL) with the Cleanroom Testing Framework (clnrm) for comprehensive trace validation and fake-green test detection.

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Collector Installation](#collector-installation)
5. [Collector Configuration](#collector-configuration)
6. [CLNRM Configuration](#clnrm-configuration)
7. [Complete Workflow](#complete-workflow)
8. [Analyzing Traces](#analyzing-traces)
9. [CI/CD Integration](#cicd-integration)
10. [Troubleshooting](#troubleshooting)
11. [Advanced Topics](#advanced-topics)

---

## Overview

OpenTelemetry integration with clnrm enables:

- **Trace Collection**: Automatic capture of test execution traces
- **Validation**: Verify that operations actually executed (catch fake-green tests)
- **Analysis**: 7 validators to ensure test correctness
- **Observability**: Export traces to Jaeger, DataDog, New Relic, etc.

### What You'll Build

```
┌─────────────┐      ┌──────────────────┐      ┌─────────────┐
│   clnrm     │─────▶│ OTEL Collector   │─────▶│ Traces File │
│  (Tests)    │ HTTP │  (4318/4317)     │ JSON │ spans.json  │
└─────────────┘      └──────────────────┘      └─────────────┘
                              │
                              │ (optional)
                              ▼
                     ┌─────────────────┐
                     │ Jaeger/DataDog  │
                     │  (Visualization)│
                     └─────────────────┘
```

---

## Prerequisites

Before starting, ensure you have:

- **clnrm** installed (via Homebrew or cargo)
- **Docker or Podman** for container-based tests
- **Rust 1.70+** if building from source
- **4GB+ RAM** recommended

### Verify Installation

```bash
# Check clnrm
clnrm --version
# Should show: clnrm 1.0.0 or later

# Check Docker
docker --version
# Should show: Docker version 20.10.0 or later

# Check Rust (optional, for building from source)
rustc --version
# Should show: rustc 1.70.0 or later
```

---

## Quick Start

Get up and running in 5 minutes:

```bash
# 1. Install OpenTelemetry Collector
brew install opentelemetry-collector  # macOS
# OR download binary from https://github.com/open-telemetry/opentelemetry-collector-releases

# 2. Create collector config
cat > otel-collector-config.yaml << 'EOF'
receivers:
  otlp/http:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s

exporters:
  file:
    path: /tmp/clnrm-spans.json

service:
  pipelines:
    traces:
      receivers: [otlp/http]
      processors: [batch]
      exporters: [file]
EOF

# 3. Start collector
otelcol --config otel-collector-config.yaml &

# 4. Run clnrm tests with OTEL
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
clnrm run --features otel tests/

# 5. Analyze traces
clnrm analyze tests/my-test.clnrm.toml --traces /tmp/clnrm-spans.json
```

---

## Collector Installation

### macOS (Homebrew)

```bash
# Install OpenTelemetry Collector
brew install opentelemetry-collector

# Verify installation
otelcol --version
# Should output: otelcol version vX.XX.X
```

### Linux (Binary Release)

```bash
# Download latest release (adjust version as needed)
VERSION=0.91.0
ARCH=linux_amd64  # or linux_arm64

curl -L -o otelcol \
  https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v${VERSION}/otelcol_${ARCH}

# Make executable
chmod +x otelcol

# Move to PATH
sudo mv otelcol /usr/local/bin/

# Verify
otelcol --version
```

### Linux (Package Manager)

#### Debian/Ubuntu

```bash
# Add OpenTelemetry APT repository
wget -q -O- https://apt.datadoghq.com/otel.asc | sudo apt-key add -
echo "deb https://apt.datadoghq.com/ stable otel" | sudo tee /etc/apt/sources.list.d/otel.list

# Install
sudo apt-get update
sudo apt-get install -y opentelemetry-collector

# Verify
otelcol --version
```

#### Red Hat/CentOS/Fedora

```bash
# Add OpenTelemetry YUM repository
sudo tee /etc/yum.repos.d/otel.repo <<EOF
[otel]
name=OpenTelemetry Collector
baseurl=https://yum.datadoghq.com/stable/otel/
enabled=1
gpgcheck=1
gpgkey=https://yum.datadoghq.com/DATADOG_RPM_KEY_CURRENT.public
EOF

# Install
sudo yum install -y opentelemetry-collector

# Verify
otelcol --version
```

### Docker

```bash
# Pull official image
docker pull otel/opentelemetry-collector:latest

# Run collector (with config volume)
docker run -d \
  --name otel-collector \
  -p 4317:4317 \
  -p 4318:4318 \
  -v $(pwd)/otel-collector-config.yaml:/etc/otelcol/config.yaml \
  otel/opentelemetry-collector:latest

# Verify running
docker ps | grep otel-collector
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  otel-collector:
    image: otel/opentelemetry-collector:latest
    container_name: otel-collector
    ports:
      - "4317:4317"  # gRPC
      - "4318:4318"  # HTTP
    volumes:
      - ./otel-collector-config.yaml:/etc/otelcol/config.yaml
      - /tmp:/tmp
    command: ["--config", "/etc/otelcol/config.yaml"]
    restart: unless-stopped
```

```bash
# Start
docker-compose up -d otel-collector

# Check logs
docker-compose logs otel-collector
```

### Verification

Test that the collector is working:

```bash
# Start collector
otelcol --config otel-collector-config.yaml

# In another terminal, test endpoint
curl -v http://localhost:4318/v1/traces
# Should return: 405 Method Not Allowed (means it's listening)

# You should see in collector logs:
# Everything is ready. Begin running and processing data.
```

---

## Collector Configuration

Create a configuration file to define how traces are received, processed, and exported.

### Basic Configuration

Create `otel-collector-config.yaml` in your project root:

```yaml
# otel-collector-config.yaml
# Basic configuration for clnrm trace collection

receivers:
  # OTLP HTTP receiver - primary endpoint for clnrm
  otlp/http:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

  # OTLP gRPC receiver - alternative endpoint
  otlp/grpc:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  # Batch processor for performance
  batch:
    timeout: 1s
    send_batch_size: 100

  # Memory limiter to prevent OOM
  memory_limiter:
    check_interval: 1s
    limit_mib: 512

exporters:
  # File exporter - outputs spans as NDJSON for validation
  file:
    path: /tmp/clnrm-spans.json
    rotation:
      max_megabytes: 10
      max_days: 1
      max_backups: 3
      localtime: true

  # Logging exporter for debugging (optional)
  logging:
    verbosity: detailed

service:
  pipelines:
    traces:
      receivers: [otlp/http, otlp/grpc]
      processors: [memory_limiter, batch]
      exporters: [file, logging]

  telemetry:
    logs:
      level: info
```

### Configuration with Jaeger Visualization

Add Jaeger for visual trace inspection:

```yaml
# otel-collector-config.yaml (with Jaeger)

receivers:
  otlp/http:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s

exporters:
  # File exporter for clnrm validation
  file:
    path: /tmp/clnrm-spans.json

  # Jaeger exporter for visualization
  otlp/jaeger:
    endpoint: localhost:4317
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp/http]
      processors: [batch]
      exporters: [file, otlp/jaeger]
```

Start Jaeger alongside collector:

```bash
# Start Jaeger
docker run -d \
  --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Access Jaeger UI
open http://localhost:16686
```

### Production Configuration

For production environments with DataDog, New Relic, or other backends:

```yaml
# otel-collector-config.yaml (production)

receivers:
  otlp/http:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

  memory_limiter:
    check_interval: 1s
    limit_mib: 2048

  # Resource detection for cloud environments
  resourcedetection:
    detectors: [env, system, docker, ec2, eks]
    timeout: 5s

exporters:
  # File exporter for clnrm validation
  file:
    path: /var/log/otel/spans.json
    rotation:
      max_megabytes: 100
      max_days: 7
      max_backups: 10

  # DataDog exporter
  datadog:
    api:
      key: ${DD_API_KEY}
      site: datadoghq.com
    hostname: ${HOSTNAME}

  # New Relic exporter
  otlphttp/newrelic:
    endpoint: https://otlp.nr-data.net
    headers:
      api-key: ${NEW_RELIC_LICENSE_KEY}

service:
  pipelines:
    traces:
      receivers: [otlp/http]
      processors: [memory_limiter, resourcedetection, batch]
      exporters: [file, datadog, otlphttp/newrelic]
```

---

## CLNRM Configuration

Configure clnrm to emit OpenTelemetry traces during test execution.

### Method 1: Environment Variables (Recommended)

The simplest way to enable OTEL:

```bash
# Required: OTLP endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

# Optional: Service name (default: "clnrm")
export OTEL_SERVICE_NAME=my-service

# Optional: Sampling (default: 1.0 = 100%)
export OTEL_TRACES_SAMPLER=traceidratio
export OTEL_TRACES_SAMPLER_ARG=1.0

# Run tests
clnrm run --features otel tests/
```

### Method 2: Test Configuration File

Configure OTEL in your `.clnrm.toml` test file:

```toml
# tests/my-test.clnrm.toml

[test.metadata]
name = "my_otel_test"
description = "Test with OpenTelemetry validation"

# OTEL configuration
[otel]
enabled = true
service_name = "my-service"
exporter = "otlp_http"
endpoint = "http://localhost:4318"
sample_ratio = 1.0  # 100% sampling

# Optional: Custom OTLP headers for authentication
[otel_headers]
Authorization = "Bearer ${OTEL_API_TOKEN}"
X-Custom-Header = "my-value"

# Service configuration
[services.test_service]
type = "generic_container"
image = "alpine:latest"

# Test steps
[[steps]]
name = "hello_world"
command = ["echo", "Hello, OTEL!"]
service = "test_service"
expected_output_regex = "Hello, OTEL!"
expected_exit_code = 0

# OTEL expectations for validation
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "clnrm.step:hello_world"
attrs.any = ["step.name=hello_world"]
```

### Method 3: Feature Flags

Build and run clnrm with specific OTEL features:

```bash
# Enable all OTEL features
cargo build --release --features otel

# Enable specific features
cargo build --release --features otel-traces
cargo build --release --features otel-metrics,otel-logs

# Run with features
cargo run --features otel -- run tests/
```

### Available OTEL Configuration Options

All configuration options in `.clnrm.toml`:

```toml
[otel]
# Enable/disable OTEL (default: false)
enabled = true

# Service name for traces (default: "clnrm")
service_name = "my-service"

# Deployment environment (default: "dev")
deployment_env = "production"

# Exporter type: otlp_http, otlp_grpc, stdout, stdout_ndjson
exporter = "otlp_http"

# OTLP endpoint (default: http://localhost:4318)
endpoint = "http://collector:4318"

# Sampling ratio: 0.0 to 1.0 (default: 1.0 = 100%)
sample_ratio = 0.1

# Enable local console logging (default: false)
enable_fmt_layer = true

# Custom resource attributes
[otel.resource_attributes]
service.version = "1.0.0"
deployment.environment = "staging"
team = "platform"
region = "us-east-1"
```

---

## Complete Workflow

End-to-end walkthrough of collecting and validating traces.

### Step 1: Start OpenTelemetry Collector

```bash
# Create collector config (see Configuration section above)
cat > otel-collector-config.yaml << 'EOF'
receivers:
  otlp/http:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s

exporters:
  file:
    path: /tmp/clnrm-spans.json

service:
  pipelines:
    traces:
      receivers: [otlp/http]
      processors: [batch]
      exporters: [file]
EOF

# Start collector in background
otelcol --config otel-collector-config.yaml > /tmp/otel-collector.log 2>&1 &
echo $! > /tmp/otel-collector.pid

# Verify collector is running
ps aux | grep otelcol | grep -v grep
# Should show running process

# Test endpoint
curl -v http://localhost:4318/v1/traces 2>&1 | grep "405 Method Not Allowed"
# 405 response means collector is listening correctly
```

### Step 2: Create Test with OTEL Expectations

Create a test file with validation expectations:

```toml
# tests/integration-test.clnrm.toml

[test.metadata]
name = "integration_test"
description = "Integration test with full OTEL validation"
timeout = "60s"

[otel]
enabled = true
service_name = "integration-test"
exporter = "otlp_http"
endpoint = "http://localhost:4318"

# Service configuration
[services.app]
type = "generic_container"
image = "alpine:latest"

# Test steps
[[steps]]
name = "setup"
command = ["echo", "Setting up..."]
service = "app"
expected_exit_code = 0

[[steps]]
name = "execute"
command = ["sh", "-c", "echo 'Running test' && sleep 1 && echo 'Done'"]
service = "app"
expected_exit_code = 0

[[steps]]
name = "teardown"
command = ["echo", "Cleaning up..."]
service = "app"
expected_exit_code = 0

# OTEL Expectations - These validate actual execution
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[[expect.span]]
name = "clnrm.step:setup"

[[expect.span]]
name = "clnrm.step:execute"

[[expect.span]]
name = "clnrm.step:teardown"

# Validate execution graph
[expect.graph]
must_include = [
    ["clnrm.run", "clnrm.step:setup"],
    ["clnrm.run", "clnrm.step:execute"],
    ["clnrm.run", "clnrm.step:teardown"]
]

# Validate span counts
[expect.counts]
spans_total = { gte = 4 }
by_name."clnrm.run" = { eq = 1 }
by_name."clnrm.step:setup" = { eq = 1 }
by_name."clnrm.step:execute" = { eq = 1 }
by_name."clnrm.step:teardown" = { eq = 1 }

# Validate execution order
[expect.order]
must_precede = [
    ["clnrm.step:setup", "clnrm.step:execute"],
    ["clnrm.step:execute", "clnrm.step:teardown"]
]

# Validate status codes
[expect.status]
all = "OK"

# Validate hermeticity (no external services)
[expect.hermeticity]
no_external_services = true
```

### Step 3: Run Test with OTEL Enabled

```bash
# Set OTEL environment variables
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_SERVICE_NAME=clnrm-integration

# Run test with OTEL features
clnrm run --features otel tests/integration-test.clnrm.toml

# Expected output:
# Running test: integration_test
# ✅ Step 'setup' passed
# ✅ Step 'execute' passed
# ✅ Step 'teardown' passed
# ✅ Test 'integration_test' passed (3/3 steps)
# ✅ OTEL spans exported to http://localhost:4318
```

### Step 4: Verify Traces Collected

```bash
# Check that traces were written to file
ls -lh /tmp/clnrm-spans.json
# Should show file with size > 0

# View trace content (requires jq)
cat /tmp/clnrm-spans.json | jq .

# Count spans
cat /tmp/clnrm-spans.json | jq -s 'length'
# Should show 4 or more spans

# List span names
cat /tmp/clnrm-spans.json | jq -r '.[].name' | sort -u
# Should include:
# - clnrm.run
# - clnrm.step:setup
# - clnrm.step:execute
# - clnrm.step:teardown
```

### Step 5: Analyze Traces Against Expectations

```bash
# Run OTEL validation
clnrm analyze tests/integration-test.clnrm.toml \
  --traces /tmp/clnrm-spans.json

# Expected output:
# 📊 OTEL Validation Report
# ========================
#
# Test: integration_test
# Traces: 4 spans, 12 events
#
# Validators:
#   ✅ Span Expectations (4/4 passed)
#   ✅ Graph Structure (all 3 edges present)
#   ✅ Counts (spans_total: 4)
#   ✅ Ordering (all constraints satisfied)
#   ✅ Status (all spans OK)
#   ✅ Hermeticity (no external services detected)
#
# Result: PASS (6/6 validators passed)
# Digest: sha256:abc123... (recorded for reproduction)

# Check exit code
echo $?
# 0 = success, 1 = validation failed
```

### Step 6: Cleanup

```bash
# Stop OTEL collector
kill $(cat /tmp/otel-collector.pid)

# Clean up traces (optional)
rm /tmp/clnrm-spans.json

# Clean up logs
rm /tmp/otel-collector.log /tmp/otel-collector.pid
```

---

## Analyzing Traces

The `clnrm analyze` command validates traces against expectations.

### Basic Analysis

```bash
# Auto-detect traces from artifacts (requires prior run with artifact collection)
clnrm analyze tests/my-test.clnrm.toml

# Specify traces file explicitly
clnrm analyze tests/my-test.clnrm.toml --traces /tmp/clnrm-spans.json

# Analyze all tests in directory
for test in tests/*.clnrm.toml; do
    clnrm analyze "$test" --traces /tmp/clnrm-spans.json
done
```

### Understanding the Report

```
📊 OTEL Validation Report
========================

Test: integration_test
Traces: 10 spans, 25 events

Validators:
  ✅ Span Expectations (3/3 passed)     # All expected spans found
  ✅ Graph Structure (all 2 edges)      # Parent-child relationships correct
  ✅ Counts (spans_total: 10)           # Span counts match expectations
  ✅ Window Containment (all 1 windows) # Temporal containment verified
  ✅ Ordering (all constraints)         # Temporal ordering correct
  ✅ Status (all spans OK)              # No error spans
  ✅ Hermeticity (no external services) # Test isolated

Result: PASS (7/7 validators passed)
Digest: sha256:abc123... (recorded for reproduction)
```

### Exit Codes

- **0** - All validators passed
- **1** - One or more validators failed

Use in CI/CD:

```bash
# Run analysis and fail build on validation errors
if ! clnrm analyze tests/*.clnrm.toml --traces /tmp/clnrm-spans.json; then
    echo "OTEL validation failed!"
    exit 1
fi
```

### Troubleshooting Failed Validations

When validation fails, the report shows the first failing validator:

```
Validators:
  ✅ Span Expectations (3/3 passed)
  ❌ Graph Structure (FAIL: missing edge clnrm.run→clnrm.step:execute)
  ✅ Counts (spans_total: 10)
  ...

Result: FAIL (1/7 validators failed)
```

**Resolution steps:**

1. Check that the expected span exists:
```bash
cat /tmp/clnrm-spans.json | jq '.[] | select(.name=="clnrm.step:execute")'
```

2. Verify parent relationship:
```bash
cat /tmp/clnrm-spans.json | jq -r '.[] | "\(.name) parent=\(.parent_span_id // "none")"'
```

3. Update expectations if behavior changed intentionally

---

## CI/CD Integration

Integrate OTEL validation into continuous integration pipelines.

### GitHub Actions

```yaml
# .github/workflows/test-with-otel.yml
name: Tests with OTEL Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal

      - name: Install clnrm
        run: cargo install clnrm

      - name: Install OpenTelemetry Collector
        run: |
          wget -q https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.91.0/otelcol_linux_amd64
          chmod +x otelcol_linux_amd64
          sudo mv otelcol_linux_amd64 /usr/local/bin/otelcol

      - name: Start OTEL Collector
        run: |
          otelcol --config otel-collector-config.yaml > otel-collector.log 2>&1 &
          echo $! > otel-collector.pid
          sleep 2  # Wait for collector to start

      - name: Run tests with OTEL
        env:
          OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4318
          OTEL_SERVICE_NAME: clnrm-ci
        run: |
          clnrm run --features otel tests/

      - name: Validate OTEL traces
        run: |
          clnrm analyze tests/*.clnrm.toml --traces /tmp/clnrm-spans.json

      - name: Stop OTEL Collector
        if: always()
        run: |
          kill $(cat otel-collector.pid) || true

      - name: Upload traces as artifact
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: otel-traces
          path: /tmp/clnrm-spans.json

      - name: Upload collector logs
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: otel-collector-logs
          path: otel-collector.log
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - setup
  - test
  - validate

variables:
  OTEL_EXPORTER_OTLP_ENDPOINT: "http://localhost:4318"
  OTEL_SERVICE_NAME: "clnrm-ci"

setup:
  stage: setup
  image: rust:latest
  script:
    - cargo install clnrm
    - wget -q https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.91.0/otelcol_linux_amd64
    - chmod +x otelcol_linux_amd64
    - mv otelcol_linux_amd64 /usr/local/bin/otelcol

test:
  stage: test
  image: rust:latest
  before_script:
    - otelcol --config otel-collector-config.yaml > otel-collector.log 2>&1 &
    - sleep 2
  script:
    - clnrm run --features otel tests/
  artifacts:
    paths:
      - /tmp/clnrm-spans.json
      - otel-collector.log
    expire_in: 1 week

validate:
  stage: validate
  image: rust:latest
  dependencies:
    - test
  script:
    - clnrm analyze tests/*.clnrm.toml --traces /tmp/clnrm-spans.json
  allow_failure: false
```

### Jenkins Pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any

    environment {
        OTEL_EXPORTER_OTLP_ENDPOINT = 'http://localhost:4318'
        OTEL_SERVICE_NAME = 'clnrm-jenkins'
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    # Install clnrm
                    cargo install clnrm

                    # Install OTEL collector
                    wget -q https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.91.0/otelcol_linux_amd64
                    chmod +x otelcol_linux_amd64
                    sudo mv otelcol_linux_amd64 /usr/local/bin/otelcol

                    # Start collector
                    otelcol --config otel-collector-config.yaml > otel-collector.log 2>&1 &
                    echo $! > otel-collector.pid
                    sleep 2
                '''
            }
        }

        stage('Test') {
            steps {
                sh 'clnrm run --features otel tests/'
            }
        }

        stage('Validate') {
            steps {
                sh 'clnrm analyze tests/*.clnrm.toml --traces /tmp/clnrm-spans.json'
            }
        }
    }

    post {
        always {
            sh '''
                # Stop collector
                if [ -f otel-collector.pid ]; then
                    kill $(cat otel-collector.pid) || true
                fi
            '''

            archiveArtifacts artifacts: '/tmp/clnrm-spans.json,otel-collector.log', allowEmptyArchive: true
        }
    }
}
```

### CircleCI

```yaml
# .circleci/config.yml
version: 2.1

jobs:
  test-with-otel:
    docker:
      - image: rust:latest
    steps:
      - checkout

      - run:
          name: Install dependencies
          command: |
            cargo install clnrm
            wget -q https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.91.0/otelcol_linux_amd64
            chmod +x otelcol_linux_amd64
            mv otelcol_linux_amd64 /usr/local/bin/otelcol

      - run:
          name: Start OTEL Collector
          command: |
            otelcol --config otel-collector-config.yaml > otel-collector.log 2>&1 &
            echo $! > otel-collector.pid
            sleep 2
          background: true

      - run:
          name: Run tests
          command: clnrm run --features otel tests/
          environment:
            OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4318
            OTEL_SERVICE_NAME: clnrm-circleci

      - run:
          name: Validate traces
          command: clnrm analyze tests/*.clnrm.toml --traces /tmp/clnrm-spans.json

      - store_artifacts:
          path: /tmp/clnrm-spans.json
          destination: traces

      - store_artifacts:
          path: otel-collector.log
          destination: logs

workflows:
  version: 2
  test:
    jobs:
      - test-with-otel
```

---

## Troubleshooting

### Collector Not Receiving Traces

**Symptom:** No spans in output file after running tests

**Solutions:**

```bash
# 1. Check collector is running
ps aux | grep otelcol | grep -v grep
# Should show running process

# 2. Verify OTEL environment variable
echo $OTEL_EXPORTER_OTLP_ENDPOINT
# Should be: http://localhost:4318

# 3. Check collector logs for errors
cat /tmp/otel-collector.log | grep -i error

# 4. Test endpoint with curl
curl -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -d '{}'
# Should return 400 Bad Request (means it's listening)

# 5. Verify file permissions
ls -l /tmp/clnrm-spans.json
chmod 666 /tmp/clnrm-spans.json

# 6. Check firewall (if running in container/VM)
sudo iptables -L -n | grep 4318
```

### Spans Missing from Output

**Symptom:** Some expected spans not in traces file

**Solutions:**

```bash
# 1. Ensure 100% sampling
export OTEL_TRACES_SAMPLER=always_on
# OR in test config:
[otel]
sample_ratio = 1.0

# 2. Wait for batch export
sleep 2  # After test completes

# 3. Reduce batch timeout in collector
processors:
  batch:
    timeout: 100ms  # Shorter timeout

# 4. Enable debug logging
export RUST_LOG=clnrm=debug,opentelemetry=debug
clnrm run --features otel tests/

# 5. Check span creation
cat /tmp/clnrm-spans.json | jq -r '.[].name' | sort -u
```

### Invalid Trace Format

**Symptom:** "Failed to parse spans file" error

**Solutions:**

```bash
# 1. Verify JSON is valid
jq . /tmp/clnrm-spans.json
# Should parse without errors

# 2. Check for NDJSON format (one JSON object per line)
head -1 /tmp/clnrm-spans.json | jq .
# Should show single span

# 3. Verify file exporter configuration
exporters:
  file:
    path: /tmp/clnrm-spans.json
    # Don't use compression

# 4. Check file is not empty
wc -l /tmp/clnrm-spans.json
# Should show lines > 0

# 5. Recreate file
rm /tmp/clnrm-spans.json
# Restart collector
```

### Permission Errors

**Symptom:** "Permission denied" when writing traces

**Solutions:**

```bash
# 1. Check file permissions
sudo chmod 666 /tmp/clnrm-spans.json

# 2. Run collector with appropriate user
# Docker: use --user flag
docker run --user $(id -u):$(id -g) ...

# 3. Use user-writable directory
exporters:
  file:
    path: $HOME/.clnrm/traces/spans.json

mkdir -p $HOME/.clnrm/traces

# 4. Check SELinux (RHEL/CentOS)
getenforce
# If Enforcing, temporarily disable for testing:
sudo setenforce 0
```

### Port Conflicts

**Symptom:** "Address already in use" error

**Solutions:**

```bash
# 1. Check what's using the port
lsof -i :4318
# OR
netstat -tuln | grep 4318

# 2. Kill conflicting process
kill $(lsof -t -i:4318)

# 3. Use different port
receivers:
  otlp/http:
    protocols:
      http:
        endpoint: 0.0.0.0:5318

# Update endpoint in clnrm:
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:5318
```

### Collector Crashes or High Memory Usage

**Symptom:** Collector stops or uses excessive memory

**Solutions:**

```yaml
# 1. Add memory limiter
processors:
  memory_limiter:
    check_interval: 1s
    limit_mib: 512  # Adjust based on available memory

# 2. Reduce batch size
processors:
  batch:
    timeout: 1s
    send_batch_size: 50  # Smaller batches

# 3. Enable resource limits (Docker)
docker run --memory="512m" --cpus="0.5" ...

# 4. Check collector logs
tail -f /tmp/otel-collector.log
```

### Analyze Command Reports "No artifacts found"

**Symptom:** `clnrm analyze` can't find traces

**Solutions:**

```bash
# 1. Specify traces file explicitly
clnrm analyze tests/my-test.clnrm.toml --traces /tmp/clnrm-spans.json

# 2. Check artifact directory structure
ls -R .clnrm/artifacts/

# 3. Ensure tests ran with artifact collection
clnrm run --features otel --artifacts tests/

# 4. Verify scenario names match
# In test config:
[[scenario]]
name = "my_scenario"  # Must match artifact directory name
```

---

## Advanced Topics

### Custom Span Attributes

Add custom attributes to spans for richer validation:

```toml
# In test config
[[expect.span]]
name = "my_operation"
attrs.all = {
  "environment" = "staging",
  "team" = "platform",
  "version" = "1.0.0"
}
```

### Distributed Tracing

Trace across multiple services:

```toml
# Service A configuration
[otel]
service_name = "service-a"

[[steps]]
name = "call_service_b"
command = ["curl", "http://service-b:8080/api"]
service = "service_a"

# Trace context is propagated automatically via W3C Trace Context headers
# Traces will show: service-a → service-b
```

### Performance Profiling

Use OTEL to profile test performance:

```toml
# Enforce performance budgets
[[expect.span]]
name = "critical_operation"
duration_ms = { max = 1000 }  # Fail if > 1 second

[expect.counts]
spans_total = { lte = 50 }  # Prevent span explosion
```

### Multi-Environment Testing

Use templates for environment-specific OTEL config:

```toml
# test.clnrm.toml.tera
[otel]
service_name = "clnrm-{{ env(name='ENV', default='dev') }}"
endpoint = "{{ env(name='OTEL_ENDPOINT', default='http://localhost:4318') }}"
deployment_env = "{{ env(name='ENV', default='dev') }}"

[otel.resource_attributes]
environment = "{{ env(name='ENV', default='dev') }}"
region = "{{ env(name='AWS_REGION', default='us-east-1') }}"
```

### Sampling Strategies

Configure sampling for high-volume tests:

```yaml
# Collector config with tail sampling
processors:
  tail_sampling:
    policies:
      # Always sample errors
      - name: error-traces
        type: status_code
        status_code:
          status_codes: [ERROR]

      # Sample 10% of successful traces
      - name: success-sampling
        type: probabilistic
        probabilistic:
          sampling_percentage: 10

service:
  pipelines:
    traces:
      receivers: [otlp/http]
      processors: [tail_sampling, batch]
      exporters: [file]
```

### Exporting to Multiple Backends

Send traces to multiple destinations:

```yaml
# Collector config with multiple exporters
exporters:
  # File for clnrm validation
  file:
    path: /tmp/clnrm-spans.json

  # Jaeger for visualization
  otlp/jaeger:
    endpoint: jaeger:4317
    tls:
      insecure: true

  # DataDog for monitoring
  datadog:
    api:
      key: ${DD_API_KEY}

service:
  pipelines:
    traces:
      receivers: [otlp/http]
      processors: [batch]
      exporters: [file, otlp/jaeger, datadog]
```

---

## See Also

- [CLI Analyze Reference](CLI_ANALYZE.md) - Complete `analyze` command documentation
- [Fake Green Detection User Guide](FAKE_GREEN_DETECTION_USER_GUIDE.md) - Using OTEL to catch false positives
- [Telemetry Source](/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs) - Implementation details
- [Example Configurations](/Users/sac/clnrm/examples/otel-validation/) - Sample test files

---

## Support

For issues or questions:

- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Documentation**: /Users/sac/clnrm/docs/
- **Examples**: /Users/sac/clnrm/examples/otel-validation/

---

**Note:** This guide is for clnrm v1.0.0+. For earlier versions, some features may not be available.
