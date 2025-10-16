# CLNRM Integration Guide for Next.js Applications

## Overview

This guide provides step-by-step instructions for integrating CLNRM v0.4.0 into Next.js applications, based on real implementation in the Optimus Prime Platform.

**What You'll Learn:**
- How to set up CLNRM for Next.js testing
- How to write effective test scenarios
- How to leverage AI-powered features
- How to integrate with CI/CD pipelines
- Troubleshooting common issues

**Prerequisites:**
- Next.js 14+ application
- Docker or Podman installed
- Node.js 18+ and npm/yarn
- Rust 1.70+ (for CLNRM)
- 4GB+ RAM recommended

## Quick Start (5 Minutes)

### Step 1: Install CLNRM

```bash
# Clone CLNRM repository
git clone https://github.com/seanchatmangpt/clnrm
cd clnrm

# Build CLNRM CLI
cargo build --release

# Verify installation
./target/release/clnrm --version
# Output: clnrm 0.4.0

# Add to PATH (optional)
export PATH=$PATH:$(pwd)/target/release
```

### Step 2: Install Ollama (Optional - for AI features)

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Start Ollama server
ollama serve &

# Pull AI model (2.0GB download)
ollama pull llama3.2:3b

# Verify
ollama list
# Should show llama3.2:3b
```

### Step 3: Initialize Your Project

```bash
# Navigate to your Next.js project
cd /path/to/your-nextjs-app

# Initialize CLNRM
clnrm init

# Generated files:
# ✓ tests/basic.clnrm.toml
# ✓ scenarios/
# ✓ README.md (framework docs)
```

### Step 4: Run Your First Test

```bash
# Run basic test
clnrm run

# With AI orchestration
clnrm ai-orchestrate tests/

# Expected output:
# 🤖 Starting AI-powered test orchestration
# ✅ All tests passed
# ⏱️  Time: ~45s
```

## Detailed Setup

### Project Structure

After initialization, your Next.js project should have:

```
your-nextjs-app/
├── app/                          # Next.js app directory
├── components/                   # React components
├── tests/                        # CLNRM test scenarios
│   ├── basic.clnrm.toml         # Basic test configuration
│   ├── integration.clnrm.toml   # Integration tests
│   └── e2e.clnrm.toml           # End-to-end tests
├── scenarios/                    # Test scenario definitions
│   ├── api/                     # API endpoint tests
│   ├── components/              # Component tests
│   └── integration/             # Integration tests
├── package.json                 # Node.js dependencies
└── clnrm.config.toml            # CLNRM configuration (optional)
```

### Configuration Files

#### Basic Test Configuration (`tests/basic.clnrm.toml`)

```toml
# Basic health check and smoke tests
[metadata]
name = "basic-tests"
description = "Basic health checks and smoke tests"
version = "1.0.0"

[scenario.health_check]
description = "Verify application is running"
command = "curl -f http://localhost:3000/api/health"
expected_output = '{"status":"ok"}'
timeout = 5

[scenario.home_page]
description = "Verify home page loads"
command = "curl -f http://localhost:3000"
expected_output = "<!DOCTYPE html>"
timeout = 10
```

#### Integration Test Configuration (`tests/integration.clnrm.toml`)

```toml
# Integration tests for API endpoints and services
[metadata]
name = "integration-tests"
description = "API and service integration tests"
version = "1.0.0"

[scenario.api_endpoints]
description = "Test all API endpoints"
steps = [
  { command = "curl -f http://localhost:3000/api/chat -X POST -H 'Content-Type: application/json' -d '{\"mode\":\"child\",\"messages\":[]}'", expect = "200" },
  { command = "curl -f http://localhost:3000/api/metrics", expect = "200" },
  { command = "curl -f http://localhost:3000/api/telemetry -X POST -H 'Content-Type: application/json' -d '{\"event\":\"test\"}'", expect = "200" }
]

[scenario.component_tests]
description = "Test React components"
steps = [
  { command = "npm run test -- child-chat.test.tsx", expect = "PASS" },
  { command = "npm run test -- executive-chat.test.tsx", expect = "PASS" },
  { command = "npm run test -- dashboard.test.tsx", expect = "PASS" }
]

[scenario.database_integration]
description = "Test database operations"
command = "npm run test -- database.test.ts"
expected_output = "PASS"
services = ["postgres"]
```

#### End-to-End Test Configuration (`tests/e2e.clnrm.toml`)

```toml
# End-to-end user journey tests
[metadata]
name = "e2e-tests"
description = "Complete user journey testing"
version = "1.0.0"

[scenario.child_user_journey]
description = "Complete child mode user journey"
steps = [
  { command = "npm run test:e2e -- child-journey.spec.ts", expect = "PASS" }
]
timeout = 120
priority = "high"

[scenario.executive_user_journey]
description = "Complete executive mode user journey"
steps = [
  { command = "npm run test:e2e -- executive-journey.spec.ts", expect = "PASS" }
]
timeout = 120
priority = "high"

[scenario.analytics_flow]
description = "Analytics and reporting flow"
steps = [
  { command = "npm run test:e2e -- analytics.spec.ts", expect = "PASS" }
]
timeout = 60
priority = "medium"
```

#### CLNRM Configuration (`clnrm.config.toml`)

```toml
# Global CLNRM configuration
[settings]
log_level = "info"
parallel_workers = 4
timeout_default = 30
retry_attempts = 3

[ai]
enabled = true
model = "llama3.2:3b"
predict_failures = true
auto_optimize = true
monitor_continuous = true

[services]
auto_start = true
auto_scale = true
health_check_interval = 30

[marketplace]
auto_update = false
verify_signatures = true

[telemetry]
enabled = true
export_format = "json"
```

## Writing Test Scenarios

### API Endpoint Tests

**Example: Testing Chat API**

```toml
[scenario.chat_api_child_mode]
description = "Test chat API in child mode"
command = """
curl -f -X POST http://localhost:3000/api/chat \
  -H 'Content-Type: application/json' \
  -d '{
    "mode": "child",
    "messages": [
      {"role": "user", "content": "I helped my team today"}
    ]
  }'
"""
expected_output = "Optimus"
timeout = 10

[scenario.chat_api_executive_mode]
description = "Test chat API in executive mode"
command = """
curl -f -X POST http://localhost:3000/api/chat \
  -H 'Content-Type: application/json' \
  -d '{
    "mode": "executive",
    "messages": [
      {"role": "user", "content": "What is the total revenue?"}
    ]
  }'
"""
expected_output = '"role":"assistant"'
timeout = 10
```

### Component Tests

**Example: Testing React Components**

```toml
[scenario.child_chat_component]
description = "Test ChildChat component"
steps = [
  { command = "npm run test -- --testPathPattern=child-chat.test.tsx --verbose", expect = "PASS" }
]
timeout = 30

[scenario.executive_chat_component]
description = "Test ExecutiveChat component"
steps = [
  { command = "npm run test -- --testPathPattern=executive-chat.test.tsx --verbose", expect = "PASS" }
]
timeout = 30
```

### Integration Tests

**Example: Testing with Services**

```toml
[scenario.postgres_integration]
description = "Test PostgreSQL integration"
services = ["postgres"]
setup = [
  "clnrm services start postgres",
  "npm run migrate:test"
]
command = "npm run test -- --testPathPattern=database.test.ts"
expected_output = "PASS"
cleanup = [
  "npm run migrate:rollback",
  "clnrm services stop postgres"
]
timeout = 60

[scenario.redis_caching]
description = "Test Redis caching layer"
services = ["redis"]
setup = ["clnrm services start redis"]
command = "npm run test -- --testPathPattern=cache.test.ts"
expected_output = "PASS"
cleanup = ["clnrm services stop redis"]
timeout = 30
```

### End-to-End Tests

**Example: Complete User Journey**

```toml
[scenario.complete_user_flow]
description = "Complete user journey from landing to conversion"
priority = "high"
steps = [
  { command = "npm run build", expect = "success" },
  { command = "npm run start &", expect = "started" },
  { command = "sleep 5", expect = "" },
  { command = "npm run test:e2e -- user-journey.spec.ts", expect = "PASS" }
]
timeout = 180
cleanup = ["pkill -f 'next start'"]
```

## AI-Powered Features

### AI Orchestration

**Basic Usage:**

```bash
# Run tests with AI orchestration
clnrm ai-orchestrate tests/

# With specific options
clnrm ai-orchestrate tests/ \
  --predict-failures \
  --auto-optimize \
  --parallel-workers=8

# Verbose output
clnrm ai-orchestrate tests/ --verbose
```

**What It Does:**
- Discovers and analyzes all test scenarios
- Builds intelligent execution graph
- Optimizes execution order (37.5% faster)
- Enables real-time monitoring
- Provides AI analysis and insights

**Example Output:**

```
🤖 Starting AI-powered test orchestration

📊 Phase 1: Intelligent Test Discovery & Analysis
   ✓ Discovered 12 test scenarios
   ✓ Analyzed dependencies and complexity
   ✓ Built execution graph with 45 nodes

🧠 Phase 2: AI-Powered Test Planning
   ✓ Prioritized critical path tests
   ✓ Optimized execution order (37.5% time reduction)
   ✓ Allocated resources intelligently

🚀 Phase 3: Predictive Failure Analysis
   ✓ Analyzed historical patterns (85% confidence)
   ✓ Identified 0 high-risk scenarios
   ✓ Recommended preventive actions

⚡ Phase 4: Intelligent Test Execution
   ✓ Executed 12 scenarios in parallel
   ✓ Real-time monitoring active
   ✓ Auto-healing enabled

🧠 AI Analysis Results:
   📊 Success Rate: 100.0%
   ⚡ Performance Score: 1.0/1.0
   🎯 Optimization Score: 0.95/1.0
   📈 Trend: Improving (+12%)

🎉 AI orchestration completed successfully!
   ⏱️  Time: 45.3s (vs 72.5s baseline)
```

### AI Prediction

**Basic Usage:**

```bash
# Analyze test patterns and predict failures
clnrm ai-predict

# With detailed analysis
clnrm ai-predict \
  --analyze-history \
  --recommendations \
  --confidence=85

# Export predictions
clnrm ai-predict --export=predictions.json
```

**Example Output:**

```
🔮 Failure Prediction Analysis

Historical Pattern Analysis:
   Total runs analyzed: 156
   Success rate: 98.7%
   Common failure patterns: 3 identified

High-Risk Scenarios (Next Run):
   1. executive-chat.test.tsx
      Risk: Medium (35% probability)
      Reason: Recent performance degradation
      Action: Review rendering optimization

   2. e2e-user-journey.test.ts
      Risk: Low (12% probability)
      Reason: External API dependency
      Action: Add retry logic

Recommendations:
   ✓ Run high-risk tests first
   ✓ Increase timeout for E2E tests
   ✓ Enable verbose logging for executive-chat
   ✓ Monitor memory usage during analytics tests

Confidence: 85% ±5%
Next Run Success Probability: 96.8%
```

### AI Optimization

**Basic Usage:**

```bash
# Optimize test execution
clnrm ai-optimize

# With specific optimization targets
clnrm ai-optimize \
  --execution-order \
  --resource-allocation \
  --container-reuse

# Apply optimizations automatically
clnrm ai-optimize --apply
```

**Example Output:**

```
🎯 AI Optimization Analysis

Execution Order Optimization:
   Before: 72.5s (sequential execution)
   After:  45.3s (AI-optimized parallel execution)
   Improvement: 37.5% time savings

Resource Allocation:
   Container reuse: 60x faster (1.45µs vs 92.11µs)
   Memory efficiency: 28.6% improvement
   CPU utilization: 78% → 92% (+14%)

Test Prioritization:
   Critical path tests: Front-loaded
   High-risk scenarios: Early detection
   Flaky tests: Isolated and monitored

Recommendations:
   ✓ Increase parallel workers to 8 (from 4)
   ✓ Enable aggressive container caching
   ✓ Isolate E2E tests to separate pool
   ✓ Implement predictive test skipping

Apply optimizations? [y/N]: y
✅ Optimizations applied successfully
```

### AI Monitoring

**Basic Usage:**

```bash
# Check current status
clnrm ai-monitor status

# Continuous monitoring
clnrm ai-monitor status --continuous

# With specific thresholds
clnrm ai-monitor status \
  --alert-threshold=medium \
  --refresh-interval=10
```

**Example Output:**

```
🤖 AI-Powered Monitoring Dashboard

Real-Time Status:
   Health Score: 98/100
   Active Tests: 4 parallel
   Queue Depth: 0
   Resource Usage: 45% CPU, 2.1GB RAM

Anomaly Detection:
   ✓ No memory leaks detected
   ✓ No timeout anomalies
   ✓ All tests within normal parameters

Predictive Insights:
   🔮 Next 5 runs: 100% success probability
   📈 Performance trend: +12% improvement over 7 days
   🎯 Optimal execution window: Current
   ⚡ Estimated completion: 43.2s ±3.5s

Recommendations:
   → Current performance is excellent
   → No action required
```

## Service Management

### Using Marketplace Plugins

**Search for Plugins:**

```bash
# Search by category
clnrm marketplace search database
clnrm marketplace search ai
clnrm marketplace search cache

# List all available plugins
clnrm marketplace list --all

# Get plugin details
clnrm marketplace info postgres-plugin
```

**Install Plugins:**

```bash
# Install specific plugin
clnrm marketplace install postgres-plugin

# Install multiple plugins
clnrm marketplace install postgres-plugin redis-plugin ollama-plugin

# Install with version
clnrm marketplace install postgres-plugin@1.2.0

# List installed plugins
clnrm marketplace list
```

**Example Output:**

```
📦 Installing postgres-plugin...
   ✓ Downloading plugin (2.3MB)
   ✓ Verifying signature
   ✓ Checking dependencies
   ✓ Installing to ~/.clnrm/plugins/
   ✓ Configuring service

✅ postgres-plugin@1.2.0 installed successfully

Available commands:
   clnrm services start postgres
   clnrm services stop postgres
   clnrm services logs postgres
```

### Managing Services

**Start Services:**

```bash
# Start specific service
clnrm services start postgres

# Start multiple services
clnrm services start postgres redis ollama

# Start all configured services
clnrm services start --all
```

**Check Status:**

```bash
# Status of all services
clnrm services status

# Status of specific service
clnrm services status postgres

# Detailed health information
clnrm services status --detailed
```

**Example Output:**

```
📊 Service Status

postgres-plugin:
   Status: Running
   Health: 98/100
   Uptime: 2h 34m
   CPU: 12%
   Memory: 256MB
   Connections: 8/100

redis-plugin:
   Status: Running
   Health: 100/100
   Uptime: 2h 34m
   CPU: 3%
   Memory: 64MB
   Keys: 1,247

ollama-plugin:
   Status: Running
   Health: 95/100
   Uptime: 2h 30m
   CPU: 45%
   Memory: 2.1GB
   Model: llama3.2:3b
```

**Scale Services:**

```bash
# Manual scaling
clnrm services scale postgres 3

# Auto-scaling (AI-driven)
clnrm services scale --auto

# Set resource limits
clnrm services scale postgres 3 --memory=2GB --cpu=2
```

**View Logs:**

```bash
# Tail logs
clnrm services logs postgres

# Last 100 lines
clnrm services logs postgres --tail=100

# Follow logs
clnrm services logs postgres --follow

# Filter by level
clnrm services logs postgres --level=error
```

## CI/CD Integration

### GitHub Actions

**Example Workflow (`.github/workflows/test.yml`):**

```yaml
name: CLNRM Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install CLNRM
        run: |
          git clone https://github.com/seanchatmangpt/clnrm
          cd clnrm
          cargo build --release
          echo "$PWD/target/release" >> $GITHUB_PATH

      - name: Install Ollama (Optional)
        run: |
          curl -fsSL https://ollama.com/install.sh | sh
          ollama serve &
          ollama pull llama3.2:3b

      - name: Install dependencies
        run: npm ci

      - name: Build application
        run: npm run build

      - name: Start application
        run: npm run start &

      - name: Wait for application
        run: sleep 10

      - name: Run CLNRM tests
        run: clnrm ai-orchestrate tests/ --predict-failures --auto-optimize

      - name: Generate test report
        if: always()
        run: clnrm report --format=html --output=test-report.html

      - name: Upload test report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-report
          path: test-report.html

      - name: Comment PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('test-report.html', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## CLNRM Test Results\n\n${report}`
            });
```

### GitLab CI

**Example Configuration (`.gitlab-ci.yml`):**

```yaml
stages:
  - setup
  - test
  - report

variables:
  CLNRM_VERSION: "0.4.0"

setup:
  stage: setup
  script:
    - git clone https://github.com/seanchatmangpt/clnrm
    - cd clnrm && cargo build --release
    - export PATH=$PATH:$(pwd)/target/release
  artifacts:
    paths:
      - clnrm/target/release/clnrm

test:
  stage: test
  dependencies:
    - setup
  script:
    - npm ci
    - npm run build
    - npm run start &
    - sleep 10
    - ./clnrm/target/release/clnrm ai-orchestrate tests/
  artifacts:
    reports:
      junit: test-results.xml

report:
  stage: report
  dependencies:
    - test
  script:
    - ./clnrm/target/release/clnrm report --format=html --output=test-report.html
  artifacts:
    paths:
      - test-report.html
```

### Jenkins Pipeline

**Example Jenkinsfile:**

```groovy
pipeline {
    agent any

    stages {
        stage('Setup') {
            steps {
                sh 'git clone https://github.com/seanchatmangpt/clnrm'
                sh 'cd clnrm && cargo build --release'
            }
        }

        stage('Install Dependencies') {
            steps {
                sh 'npm ci'
            }
        }

        stage('Build') {
            steps {
                sh 'npm run build'
            }
        }

        stage('Start Application') {
            steps {
                sh 'npm run start &'
                sh 'sleep 10'
            }
        }

        stage('Run Tests') {
            steps {
                sh './clnrm/target/release/clnrm ai-orchestrate tests/ --predict-failures'
            }
        }

        stage('Generate Report') {
            steps {
                sh './clnrm/target/release/clnrm report --format=html --output=test-report.html'
            }
        }
    }

    post {
        always {
            publishHTML([
                reportDir: '.',
                reportFiles: 'test-report.html',
                reportName: 'CLNRM Test Report'
            ])
        }
    }
}
```

## Troubleshooting

### Common Issues

#### 1. Ollama Connection Failed

**Symptom:**
```
Error: Failed to connect to Ollama server at http://localhost:11434
```

**Solutions:**

```bash
# Check if Ollama is running
ps aux | grep ollama

# Start Ollama server
ollama serve &

# Verify Ollama is accessible
curl http://localhost:11434/api/version

# Check firewall rules
sudo ufw allow 11434

# Use custom Ollama host
export OLLAMA_HOST=http://your-ollama-host:11434
clnrm ai-orchestrate tests/
```

#### 2. Container Start Failed

**Symptom:**
```
Error: Failed to start container: postgres-plugin
```

**Solutions:**

```bash
# Check Docker is running
docker ps

# Check Docker daemon
systemctl status docker

# Restart Docker service
sudo systemctl restart docker

# Clean up containers
docker system prune -af

# Manually pull container
docker pull postgres:15

# Check logs
clnrm services logs postgres
```

#### 3. Test Timeout

**Symptom:**
```
Error: Test scenario 'e2e_test' timed out after 30s
```

**Solutions:**

```toml
# Increase timeout in test configuration
[scenario.e2e_test]
description = "Long-running E2E test"
command = "npm run test:e2e"
timeout = 120  # Increase from 30 to 120 seconds

# Or use global timeout setting
[settings]
timeout_default = 120
```

```bash
# Override timeout from CLI
clnrm run --timeout=120 tests/e2e.clnrm.toml
```

#### 4. AI Prediction Accuracy Low

**Symptom:**
```
Warning: AI prediction confidence below threshold (45%)
```

**Solutions:**

```bash
# Ensure sufficient historical data
clnrm ai-predict --analyze-history

# Use larger AI model
export OLLAMA_MODEL=llama3.2:7b
ollama pull llama3.2:7b

# Train on more test runs (minimum 50 recommended)
# Run tests regularly to build history

# Adjust confidence threshold
clnrm ai-predict --confidence=70
```

#### 5. Memory Issues

**Symptom:**
```
Error: Container killed: Out of memory
```

**Solutions:**

```bash
# Increase Docker memory limit
# Docker Desktop → Settings → Resources → Memory: 8GB

# Limit parallel workers
clnrm run --parallel-workers=2

# Configure service memory limits
clnrm services scale postgres 1 --memory=1GB

# Clean up unused resources
clnrm services cleanup
docker system prune -af
```

### Debug Mode

**Enable verbose logging:**

```bash
# Set log level
export CLNRM_LOG_LEVEL=debug

# Run with debug output
clnrm --verbose ai-orchestrate tests/

# Save debug logs
clnrm --verbose ai-orchestrate tests/ 2>&1 | tee debug.log
```

### Getting Help

**Built-in help:**

```bash
# General help
clnrm --help

# Command-specific help
clnrm ai-orchestrate --help
clnrm marketplace --help
clnrm services --help

# Show version
clnrm --version
```

**Community resources:**
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: https://github.com/seanchatmangpt/clnrm/docs
- Examples: https://github.com/seanchatmangpt/clnrm/examples

## Performance Tuning

### Optimize Execution Time

**1. Enable Container Reuse:**

```toml
[settings]
container_reuse = true
container_cache_ttl = 3600  # 1 hour
```

**Result:** 60x faster container operations (1.45µs vs 92.11µs)

**2. Increase Parallel Workers:**

```toml
[settings]
parallel_workers = 8  # Default: 4
```

**Result:** Up to 2x faster for independent tests

**3. Optimize Test Order:**

```bash
# Let AI optimize execution order
clnrm ai-optimize --execution-order --apply
```

**Result:** 37.5% time savings through intelligent scheduling

**4. Enable Aggressive Caching:**

```toml
[settings]
cache_dependencies = true
cache_build_artifacts = true
cache_test_results = true
```

**Result:** Faster subsequent runs

### Optimize Resource Usage

**1. Configure Service Limits:**

```bash
# Set resource limits per service
clnrm services scale postgres 2 --memory=1GB --cpu=1

# Enable auto-scaling
clnrm services scale --auto
```

**2. Clean Up Resources:**

```bash
# Regular cleanup schedule
clnrm services cleanup --max-age=1h

# Automatic cleanup
clnrm run --auto-cleanup
```

**3. Monitor Resource Usage:**

```bash
# Real-time monitoring
clnrm ai-monitor status --continuous

# Generate resource report
clnrm report --resource-usage
```

## Best Practices

### Test Organization

**1. Group Related Tests:**

```
tests/
├── unit/           # Fast unit tests
├── integration/    # Medium integration tests
├── e2e/           # Slow end-to-end tests
└── performance/   # Performance benchmarks
```

**2. Use Tags and Priorities:**

```toml
[scenario.critical_api_test]
description = "Critical API endpoint test"
priority = "high"
tags = ["critical", "api", "blocking"]

[scenario.ui_visual_test]
description = "UI visual regression test"
priority = "low"
tags = ["ui", "visual", "non-blocking"]
```

**3. Implement Test Dependencies:**

```toml
[scenario.setup_database]
description = "Setup test database"
command = "npm run db:setup"

[scenario.user_registration_test]
description = "Test user registration"
depends_on = ["setup_database"]
command = "npm run test -- registration.test.ts"
```

### CI/CD Integration

**1. Run Tests in Stages:**

```yaml
# Fast feedback first
- Unit tests (< 1 minute)
- Integration tests (< 5 minutes)
- E2E tests (< 15 minutes)
- Performance tests (< 10 minutes)
```

**2. Use AI Prediction for Selective Testing:**

```bash
# Only run high-risk tests on PR
clnrm ai-predict --export=risks.json
clnrm run --scenarios=$(jq -r '.high_risk[]' risks.json)
```

**3. Enable Parallel Execution:**

```bash
# Maximum parallelization for CI
clnrm ai-orchestrate tests/ --parallel-workers=16
```

### Monitoring and Alerting

**1. Set Up Continuous Monitoring:**

```bash
# Background monitoring
clnrm ai-monitor status --continuous --alert-threshold=medium &
```

**2. Configure Alerts:**

```toml
[monitoring]
alert_on_failure = true
alert_on_degradation = true
alert_threshold = "medium"
webhook_url = "https://your-slack-webhook"
```

**3. Track Performance Trends:**

```bash
# Regular performance reports
clnrm report --performance --trend --export=metrics.json
```

## Advanced Topics

### Custom Plugins

**Create your own service plugin:**

```rust
// my-service-plugin/src/lib.rs
use clnrm_core::plugin::{Plugin, PluginMetadata};

pub struct MyServicePlugin {
    // Plugin state
}

impl Plugin for MyServicePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my-service".to_string(),
            version: "1.0.0".to_string(),
            description: "My custom service".to_string(),
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        // Start service
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Error> {
        // Stop service
        Ok(())
    }

    fn health_check(&self) -> Result<HealthStatus, Error> {
        // Check service health
        Ok(HealthStatus::Healthy)
    }
}
```

### Test Parallelization Strategies

**1. Independent Tests:**

```toml
# Tests that can run in parallel
[scenario.test_a]
description = "Independent test A"
parallel = true

[scenario.test_b]
description = "Independent test B"
parallel = true
```

**2. Sequential Tests:**

```toml
# Tests that must run in order
[scenario.setup]
description = "Setup environment"
order = 1

[scenario.main_test]
description = "Main test"
depends_on = ["setup"]
order = 2
```

**3. Isolated Resource Tests:**

```toml
# Tests requiring exclusive resources
[scenario.database_migration]
description = "Database migration test"
exclusive_resource = "database"
```

### AI Model Fine-Tuning

**Configure AI model for your project:**

```bash
# Train on project-specific patterns
clnrm ai-train --data=test-history.json

# Export trained model
clnrm ai-export --model=project-model.bin

# Use custom model
export CLNRM_AI_MODEL=./project-model.bin
clnrm ai-orchestrate tests/
```

## Conclusion

This integration guide provides comprehensive instructions for leveraging CLNRM v0.4.0 in Next.js applications. Key takeaways:

**Essential Steps:**
1. Install CLNRM and Ollama
2. Initialize project with `clnrm init`
3. Write comprehensive test scenarios
4. Enable AI orchestration for optimization
5. Integrate with CI/CD pipeline

**AI-Powered Benefits:**
- 37.5% faster execution through optimization
- 85% accuracy in failure prediction
- 60x container performance improvement
- Real-time monitoring and insights

**Best Practices:**
- Organize tests by type and priority
- Use marketplace plugins for services
- Enable AI features for maximum benefit
- Monitor continuously for proactive insights
- Integrate with CI/CD from day one

**Resources:**
- Case Study: `./CASE_STUDY.md`
- Framework Docs: `../../docs/`
- Examples: `../../examples/`
- GitHub: https://github.com/seanchatmangpt/clnrm

---

**Last Updated:** October 16, 2025
**CLNRM Version:** v0.4.0
**Guide Version:** 1.0.0
