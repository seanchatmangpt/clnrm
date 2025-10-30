# Production Deployment

Production deployment covers running clnrm in CI/CD pipelines, optimizing performance, and implementing enterprise-scale patterns for reliable testing in production environments.

## Overview

Production deployment includes:
- **CI/CD Integration** - GitHub Actions, GitLab CI, Jenkins
- **Performance Optimization** - Parallel execution, resource management
- **Enterprise Patterns** - Multi-environment, security, compliance
- **Monitoring and Observability** - Production monitoring, alerting
- **Scaling Strategies** - Horizontal scaling, resource allocation

## CI/CD Integration

### GitHub Actions Integration

Complete GitHub Actions workflow for clnrm:

```yaml
# .github/workflows/clnrm-tests.yml
name: Cleanroom Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Pull test images
      run: clnrm pull tests/

    - name: Run tests
      run: clnrm run tests/ --parallel --workers 4

    - name: Upload results
      uses: actions/upload-artifact@v3
      with:
        name: test-results
        path: |
          *.json
          *.xml
          *.sha256

  performance:
    runs-on: ubuntu-latest
    needs: test
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Run performance tests
      run: clnrm run tests/performance/ --baseline production

    - name: Check for regressions
      run: |
        if clnrm run tests/performance/ --check-regressions; then
          echo "✅ No performance regressions detected"
        else
          echo "❌ Performance regression detected"
          exit 1
        fi
```

### GitLab CI Integration

GitLab CI pipeline configuration:

```yaml
# .gitlab-ci.yml
stages:
  - test
  - performance
  - deploy

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  RUST_BACKTRACE: 1

cache:
  paths:
    - .cargo/
    - target/

test:clnrm:
  stage: test
  image: rust:latest
  script:
    - cargo install --path .
    - clnrm pull tests/
    - clnrm run tests/ --parallel --workers $(nproc)
  artifacts:
    reports:
      junit: test-results.xml
    paths:
      - "*.json"
      - "*.xml"

performance:clnrm:
  stage: performance
  image: rust:latest
  script:
    - cargo install --path .
    - clnrm run tests/performance/ --baseline $CI_COMMIT_REF_NAME
  only:
    - main
    - tags
  artifacts:
    paths:
      - "performance-*.json"

deploy:clnrm:
  stage: deploy
  image: rust:latest
  script:
    - cargo install --path .
    - clnrm run tests/smoke/ --env production
  only:
    - tags
  environment:
    name: production
```

### Jenkins Pipeline Integration

Jenkins pipeline for clnrm:

```groovy
pipeline {
    agent any

    stages {
        stage('Install clnrm') {
            steps {
                sh 'cargo install --path .'
            }
        }

        stage('Pull Images') {
            steps {
                sh 'clnrm pull tests/'
            }
        }

        stage('Run Tests') {
            steps {
                sh 'clnrm run tests/ --parallel --workers 4'
            }
            post {
                always {
                    junit 'test-results.xml'
                    archiveArtifacts artifacts: '*.json,*.xml,*.sha256'
                }
            }
        }

        stage('Performance Tests') {
            when {
                branch 'main'
            }
            steps {
                sh 'clnrm run tests/performance/ --baseline production'
            }
        }

        stage('Deploy Tests') {
            when {
                tag pattern: 'v*', comparator: 'REGEXP'
            }
            steps {
                sh 'clnrm run tests/smoke/ --env production'
            }
        }
    }

    post {
        always {
            cleanWs()
        }
    }
}
```

## Performance Optimization

### Parallel Execution

Configure parallel test execution:

```toml
# Parallel execution configuration
[execution]
parallel = true
workers = 4
timeout_minutes = 30

[execution.strategy]
type = "dependency_aware"
max_concurrent_services = 10

# Service resource allocation
[execution.resources]
max_containers = 50
max_memory_gb = 16
max_cpu_cores = 8

# Worker configuration
[execution.workers]
worker_1 = { cpu_cores = 2, memory_gb = 4 }
worker_2 = { cpu_cores = 2, memory_gb = 4 }
worker_3 = { cpu_cores = 2, memory_gb = 4 }
worker_4 = { cpu_cores = 2, memory_gb = 4 }
```

### Image Pre-pulling

Pre-pull Docker images for faster test execution:

```bash
#!/bin/bash
# pull-test-images.sh

echo "🔄 Pre-pulling Docker images for clnrm tests..."

# Extract images from test files
images=$(grep -r "image.*=" tests/ | grep -o '"[^"]*"' | sort | uniq)

for image in $images; do
    image=$(echo $image | tr -d '"')
    echo "📦 Pulling $image..."
    docker pull $image
done

echo "✅ All images pre-pulled successfully"
```

### Caching and Optimization

Configure caching for faster builds:

```toml
# Cache configuration
[cache]
enabled = true
directory = ".clnrm/cache"

[cache.images]
ttl_hours = 24
max_size_gb = 10

[cache.results]
ttl_hours = 168  # 1 week
max_size_gb = 5

[cache.templates]
ttl_hours = 24
max_size_gb = 1

# Optimization settings
[optimization]
image_sharing = true
container_reuse = true
parallel_downloads = true

[optimization.docker]
buildkit = true
parallel_builds = 4
```

## Enterprise Patterns

### Multi-Environment Configuration

Configure tests for multiple environments:

```toml
# Multi-environment test configuration
[environments]
test = {
    image_suffix = "-test",
    resource_limits = { memory_mb = 512, cpu_cores = 1 },
    timeout_minutes = 10
}

staging = {
    image_suffix = "-staging",
    resource_limits = { memory_mb = 1024, cpu_cores = 2 },
    timeout_minutes = 15
}

production = {
    image_suffix = "-prod",
    resource_limits = { memory_mb = 2048, cpu_cores = 4 },
    timeout_minutes = 30
}

# Environment-specific service configuration
[services.api]
type = "generic_container"
image = "myapp{{ env.image_suffix }}:latest"

{% if env.name == "production" %}
ports = [80, 443]
env_vars = { "ENV" = "production", "LOG_LEVEL" = "info" }
{% elif env.name == "staging" %}
ports = [8080]
env_vars = { "ENV" = "staging", "LOG_LEVEL" = "debug" }
{% else %}
ports = [3000]
env_vars = { "ENV" = "test", "LOG_LEVEL" = "trace" }
{% endif %}
```

### Security and Compliance

Implement security and compliance patterns:

```toml
# Security configuration
[security]
enabled = true
scan_images = true
vulnerability_check = true

[security.scanning]
image_scanner = "trivy"
vulnerability_db = "latest"
fail_on_critical = true
fail_on_high = false

[security.secrets]
encrypted_storage = true
rotation_days = 30
audit_logging = true

# Compliance configuration
[compliance]
enabled = true
standards = ["SOC2", "GDPR", "HIPAA"]

[compliance.audit]
log_retention_days = 2555  # 7 years
audit_events = ["test_execution", "data_access", "config_changes"]

[compliance.encryption]
at_rest = true
in_transit = true
key_rotation_days = 90
```

### Resource Management

Manage resources in enterprise environments:

```toml
# Enterprise resource management
[resources]
namespace = "clnrm-prod"
resource_quotas = true

[resources.limits]
max_containers = 100
max_memory_gb = 32
max_cpu_cores = 16
max_storage_gb = 100

[resources.requests]
min_memory_mb = 128
min_cpu_cores = 0.1

# Resource allocation strategies
[resources.allocation]
strategy = "bin_packing"
priority_classes = ["high", "medium", "low"]

[resources.scheduling]
anti_affinity = true
zone_spreading = true
node_selectors = { "dedicated" = "testing" }
```

## Monitoring and Observability

### Production Monitoring

Monitor clnrm in production:

```toml
# Production monitoring configuration
[monitoring]
enabled = true
interval_seconds = 30

[monitoring.metrics]
test_execution_time = true
resource_utilization = true
error_rates = true
performance_trends = true

[monitoring.alerts]
error_rate_threshold = 0.05
response_time_threshold_ms = 5000
resource_threshold_percent = 90

[monitoring.alerts.channels]
email = ["devops@company.com", "qa@company.com"]
slack = ["#testing-alerts"]
pagerduty = ["testing-team"]
```

### OTEL Integration

Comprehensive OTEL setup for production:

```toml
# Production OTEL configuration
[otel]
enabled = true
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") }}"
protocol = "http/protobuf"
sample_ratio = 0.1

[otel.resources]
"service.name" = "clnrm"
"service.version" = "{{ version }}"
"env" = "production"
"deployment.environment" = "prod"

[otel.headers]
"authorization" = "Bearer {{ env(name=\"OTEL_TOKEN\") }}"

[otel.exporters]
otlp = {
    endpoint = "{{ env(name=\"OTEL_ENDPOINT\") }}",
    headers = { "authorization" = "Bearer {{ env(name=\"OTEL_TOKEN\") }}" }
}

jaeger = {
    endpoint = "{{ env(name=\"JAEGER_ENDPOINT\") }}",
    username = "{{ env(name=\"JAEGER_USER\") }}",
    password = "{{ env(name=\"JAEGER_PASSWORD\") }}"
}
```

## Scaling Strategies

### Horizontal Scaling

Scale clnrm horizontally:

```toml
# Horizontal scaling configuration
[scaling]
enabled = true
min_workers = 2
max_workers = 20
target_cpu_percent = 70

[scaling.auto_scaling]
enabled = true
cpu_threshold_percent = 70
scale_up_cooldown_minutes = 5
scale_down_cooldown_minutes = 10

[scaling.load_balancing]
strategy = "round_robin"
health_check_interval_seconds = 30
unhealthy_threshold = 3
```

### Resource Allocation

Allocate resources efficiently:

```toml
# Resource allocation for scaling
[resources.allocation]
strategy = "resource_aware"
priority_weights = {
    "cpu" = 0.4,
    "memory" = 0.4,
    "network" = 0.2
}

[resources.pools]
cpu_intensive = {
    cpu_cores = 4,
    memory_mb = 2048,
    max_concurrent = 2
}

memory_intensive = {
    cpu_cores = 2,
    memory_mb = 8192,
    max_concurrent = 1
}

network_intensive = {
    cpu_cores = 2,
    memory_mb = 1024,
    network_priority = "high",
    max_concurrent = 4
}
```

## Best Practices

### 1. Use Environment-Specific Configuration

```toml
# ✅ Good: Environment-specific configuration
{% if env == "production" %}
[resources]
memory_limit_gb = 8
cpu_limit_cores = 4
{% else %}
[resources]
memory_limit_gb = 2
cpu_limit_cores = 1
{% endif %}
```

### 2. Implement Proper Resource Limits

```toml
# ✅ Good: Proper resource limits
[services.api]
type = "generic_container"
image = "nginx:alpine"
memory_limit = "512Mi"
cpu_limit = "500m"
```

### 3. Use Monitoring and Alerting

```toml
# ✅ Good: Monitoring and alerting
[monitoring]
enabled = true
alerts = true

[monitoring.alerts]
error_rate_threshold = 0.05
response_time_threshold_ms = 5000
```

### 4. Implement Security Measures

```toml
# ✅ Good: Security measures
[security]
enabled = true
scan_images = true
encrypted_storage = true
```

## Common Patterns

### Production CI/CD Pipeline

```yaml
# Complete production pipeline
name: Production Testing Pipeline

on:
  push:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Security scan
      run: clnrm run tests/security/

  performance-baseline:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Update performance baseline
      run: clnrm run tests/performance/ --update-baseline production

  integration-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Run integration tests
      run: clnrm run tests/integration/ --parallel

  smoke-tests:
    runs-on: ubuntu-latest
    needs: [security-scan, integration-tests]
    steps:
    - uses: actions/checkout@v4
    - name: Run smoke tests
      run: clnrm run tests/smoke/ --env production
```

### Enterprise Multi-Environment Setup

```toml
# Enterprise multi-environment configuration
[environments]
development = {
    namespace = "clnrm-dev",
    resource_limits = { memory_gb = 2, cpu_cores = 1 },
    image_registry = "dev-registry.company.com"
}

staging = {
    namespace = "clnrm-staging",
    resource_limits = { memory_gb = 4, cpu_cores = 2 },
    image_registry = "staging-registry.company.com"
}

production = {
    namespace = "clnrm-prod",
    resource_limits = { memory_gb = 8, cpu_cores = 4 },
    image_registry = "prod-registry.company.com"
}

# Environment-specific service configuration
{% for env_name, env_config in environments %}
[test.{{ env_name }}.metadata]
name = "{{ env_name }}_integration_test"

[services.api]
image = "{{ env_config.image_registry }}/api:latest"
namespace = "{{ env_config.namespace }}"

{% if env_name == "production" %}
replicas = 3
health_check_interval_seconds = 10
{% else %}
replicas = 1
health_check_interval_seconds = 30
{% endif %}

{% endfor %}
```

## Next Steps

Now that you understand production deployment:

1. **Implement CI/CD**: Set up GitHub Actions or GitLab CI for your project
2. **Configure monitoring**: Set up OTEL and alerting for your test environment
3. **Scale appropriately**: Configure resource allocation for your needs
4. **Learn reference documentation**: Move on to [Reference](../reference/README.md)

## Further Reading

- [CI/CD Best Practices](https://martinfowler.com/articles/continuousIntegration.html)
- [Kubernetes Testing Patterns](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
- [Enterprise Testing Strategies](https://www.infoq.com/articles/enterprise-testing-strategies/)

