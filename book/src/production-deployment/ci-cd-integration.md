# CI/CD Integration

This chapter covers integrating clnrm v2.0.0 into CI/CD pipelines.

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

      - name: Setup Docker
        run: docker --version

      - name: Install clnrm
        run: cargo install clnrm

      - name: Run Tests
        run: clnrm run tests/ --parallel --jobs 4

      - name: Generate Report
        run: clnrm report --format html,junit --output reports/
```

### Advanced Integration with Weaver

```yaml
name: Advanced Testing
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      weaver:
        image: otel/weaver:latest
        ports:
          - 4318:4318

    steps:
      - uses: actions/checkout@v4

      - name: Install clnrm
        run: cargo install clnrm

      - name: Pre-pull Images
        run: clnrm pull tests/

      - name: Run with Weaver Validation
        run: |
          clnrm run tests/ \
            --validate \
            --otel-exporter otlp-http \
            --otel-endpoint http://localhost:4318 \
            --parallel --jobs 4

      - name: Upload Reports
        uses: actions/upload-artifact@v4
        with:
          name: test-reports
          path: reports/
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

## Best Practices

### 1. Use Appropriate Timeouts

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - name: Run Tests
        run: clnrm run tests/ --timeout 25m
```

### 2. Cache Docker Images

```yaml
- name: Cache Docker Images
  uses: actions/cache@v4
  with:
    path: ~/.docker
    key: docker-images-${{ hashFiles('tests/**/*.toml') }}

- name: Pre-pull Images
  run: clnrm pull tests/
```

### 3. Artifact Management

```yaml
- name: Run Tests
  run: clnrm run tests/ --report-junit results.xml

- name: Upload Test Results
  uses: actions/upload-artifact@v4
  if: always()
  with:
    name: test-results
    path: results.xml
```

### 4. Matrix Testing

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        test-suite: [unit, integration, e2e]

    steps:
      - name: Run ${{ matrix.test-suite }} Tests
        run: clnrm run tests/${{ matrix.test-suite }}/ --parallel
```

## Troubleshooting

### Common CI/CD Issues

**Container startup failures:**
```bash
# Increase timeout and add health checks
clnrm run tests/ --timeout 45m
```

**Network connectivity:**
```bash
# Use host networking for CI
[containers.app]
image = "myapp:latest"
network_mode = "host"
```

**Resource constraints:**
```bash
# Reduce parallelism in CI
clnrm run tests/ --jobs 2 --parallel
```

## Enterprise Integration

### Jenkins Pipeline

```groovy
pipeline {
    agent any

    stages {
        stage('Test') {
            steps {
                sh 'clnrm run tests/ --parallel --jobs 4'
            }
            post {
                always {
                    sh 'clnrm report --format junit --output results.xml'
                    junit 'results.xml'
                }
            }
        }
    }
}
```

### GitLab CI

```yaml
test:
  stage: test
  image: docker:latest
  services:
    - docker:dind

  script:
    - docker --version
    - cargo install clnrm
    - clnrm run tests/ --parallel --jobs 4
    - clnrm report --format junit --output results.xml

  artifacts:
    reports:
      junit: results.xml
```

## Performance Optimization

### CI/CD Specific Tuning

```bash
# Optimize for CI environments
clnrm run tests/ \
  --parallel \
  --jobs $(nproc) \
  --fail-fast \
  --disable-telemetry \  # Skip OTEL in fast CI runs
  --timeout 20m
```

### Resource Management

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    container:
      image: docker:latest
      options: --privileged
      volumes:
        - /var/run/docker.sock:/var/run/docker.sock

    resources:
      limits:
        cpu: 2
        memory: 4GB
```

## Monitoring and Alerting

### Test Metrics

```yaml
- name: Run Tests with Metrics
  run: |
    clnrm run tests/ --parallel --jobs 4
    clnrm report --format json --output metrics.json

- name: Send Metrics
  run: |
    # Send to monitoring system
    curl -X POST \
      -H "Content-Type: application/json" \
      -d @metrics.json \
      https://monitoring.example.com/api/metrics
```

### Failure Notifications

```yaml
- name: Notify on Failure
  if: failure()
  run: |
    curl -X POST \
      -H "Content-Type: application/json" \
      -d '{"text": "Tests failed", "channel": "#alerts"}' \
      https://hooks.slack.com/services/...
```