# Integration Test Strategy

## Overview

This document defines the comprehensive integration testing strategy for the CLNRM (Cleanroom Testing Framework) project. Integration tests validate that multiple components work together correctly, including:

- Component-to-component integration
- System-level integration
- Database integration
- External service integration
- Container orchestration
- API contract validation

## Test Pyramid Application

```
         /\
        /E2E\      <- 10-15%: Full system workflows
       /------\
      /Integr. \   <- 25-35%: Multi-component integration
     /----------\
    /   Unit     \ <- 50-65%: Component isolation
   /--------------\
```

## Integration Test Categories

### 1. Component Integration Tests

**Purpose**: Validate interactions between 2-3 related components

**Location**: `tests/integration/components/`

**Examples**:
- Backend + Policy validation
- Command execution + Result aggregation
- Plugin system + Service discovery
- Configuration loader + Backend factory

**Characteristics**:
- Fast execution (< 5s per test)
- Minimal external dependencies
- Use test doubles for expensive operations
- Focus on interface contracts

### 2. System Integration Tests

**Purpose**: Validate end-to-end workflows across the system

**Location**: `tests/integration/system/`

**Examples**:
- Full test execution pipeline
- Container lifecycle management
- Observability data flow
- Multi-backend orchestration

**Characteristics**:
- Moderate execution time (5-30s per test)
- Real Docker containers when needed
- Transactional test data
- Cleanup after each test

### 3. Database Integration Tests

**Purpose**: Validate data persistence and retrieval

**Location**: `tests/integration/database/`

**Examples**:
- SurrealDB integration
- Test result storage
- Configuration persistence
- Plugin metadata management

**Characteristics**:
- Use test database instances
- Transaction rollback for isolation
- Schema validation
- Migration testing

### 4. External Service Integration Tests

**Purpose**: Validate integration with external services

**Location**: `tests/integration/external/`

**Examples**:
- OpenTelemetry collector integration
- Container registry interactions
- API endpoint validation
- Service mesh communication

**Characteristics**:
- Mock external services by default
- Optional real service testing (ignored by default)
- Contract testing approach
- Network isolation

## Test Infrastructure

### Docker Compose Test Environment

**File**: `tests/integration/docker-compose.test.yml`

**Services**:
- SurrealDB (database)
- OpenTelemetry Collector (observability)
- Jaeger (tracing UI)
- Test containers (various images)

**Management**:
```bash
# Start test environment
docker-compose -f tests/integration/docker-compose.test.yml up -d

# Stop test environment
docker-compose -f tests/integration/docker-compose.test.yml down -v

# View logs
docker-compose -f tests/integration/docker-compose.test.yml logs -f
```

### Test Data Management

**Fixtures**: Pre-defined test data sets
- Location: `tests/integration/fixtures/`
- Formats: JSON, TOML, SQL
- Version controlled

**Factories**: Dynamic test data generation
- Location: `tests/integration/factories/`
- Builder pattern implementation
- Randomization with seeds
- Relational data consistency

### Test Isolation Mechanisms

**Database Isolation**:
- Unique database per test
- Transaction rollback
- Cleanup hooks

**Container Isolation**:
- Unique network per test suite
- Volume cleanup
- Port randomization

**Filesystem Isolation**:
- Temporary directories
- Automatic cleanup
- No shared state

## Parallel Execution Strategy

### Test Organization

**Suite Grouping**:
- Fast tests: No Docker, in-memory only
- Medium tests: Docker containers, < 30s
- Slow tests: Complex workflows, > 30s

**Execution Strategy**:
```bash
# Run all integration tests (parallel by default)
cargo test --test '*' -- --test-threads=4

# Run specific suite
cargo test --test component_integration

# Run with Docker
cargo test --test system_integration -- --ignored

# Run single-threaded (debugging)
cargo test --test '*' -- --test-threads=1
```

### Resource Management

**Connection Pools**:
- Shared database connections
- Container reuse when safe
- Rate limiting for external APIs

**Concurrency Control**:
- Test-level locks for shared resources
- Timeouts on all operations
- Graceful degradation

## CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/integration-tests.yml`

**Stages**:
1. Setup (Docker, test environment)
2. Unit tests (fast feedback)
3. Component integration tests (parallel)
4. System integration tests (sequential)
5. Database integration tests (parallel)
6. Cleanup and reporting

**Triggers**:
- Pull requests
- Main branch commits
- Nightly builds (extended tests)

**Artifacts**:
- Test results (JUnit XML)
- Coverage reports
- Trace exports
- Container logs

### Test Selection Strategy

**PR Tests** (Fast feedback):
- All unit tests
- Component integration tests
- Fast system tests

**Main Branch Tests** (Comprehensive):
- All integration tests
- Performance benchmarks
- Security scans

**Nightly Tests** (Extended):
- Long-running tests
- Stress tests
- Compatibility matrix

## Test Patterns and Best Practices

### AAA Pattern (Arrange-Act-Assert)

```rust
#[test]
fn test_backend_execution() {
    // Arrange: Setup test environment
    let backend = TestcontainerBackend::new("alpine:latest").unwrap();
    let cmd = Cmd::new("echo").arg("test");

    // Act: Execute the operation
    let result = backend.run_cmd(cmd).unwrap();

    // Assert: Verify expectations
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("test"));
}
```

### Test Doubles

**Mocks**: Verify interactions
```rust
let mut mock_backend = MockBackend::new();
mock_backend
    .expect_run_cmd()
    .times(1)
    .returning(|_| Ok(RunResult::default()));
```

**Stubs**: Provide canned responses
```rust
fn stub_database() -> TestDatabase {
    TestDatabase::new()
        .with_preset("test_users", test_users())
        .with_preset("test_projects", test_projects())
}
```

**Fakes**: Working implementations
```rust
struct FakeBackend {
    results: Vec<RunResult>,
}

impl Backend for FakeBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        Ok(self.results.pop().unwrap())
    }
}
```

### Test Helpers

**Common Setup**:
```rust
fn setup_test_environment() -> TestContext {
    TestContext {
        temp_dir: TempDir::new().unwrap(),
        database: start_test_database(),
        backend: TestcontainerBackend::new("alpine").unwrap(),
    }
}
```

**Cleanup**:
```rust
struct TestGuard {
    cleanup: Box<dyn FnOnce()>,
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        (self.cleanup)();
    }
}
```

### Assertions

**Domain-Specific Assertions**:
```rust
trait BackendAssertions {
    fn assert_successful_execution(&self);
    fn assert_hermetic_isolation(&self);
    fn assert_security_policy_applied(&self);
}
```

## Performance Targets

### Execution Times

- Component integration: < 5s per test
- System integration: < 30s per test
- Database integration: < 10s per test
- Full suite: < 5 minutes

### Resource Usage

- Max memory: 2GB per test suite
- Max containers: 10 concurrent
- Max open files: 1024
- Network bandwidth: Unlimited local

## Coverage Goals

### Code Coverage

- Integration test coverage: > 60%
- Combined with unit tests: > 80%
- Critical paths: 100%

### Scenario Coverage

- Happy paths: 100%
- Error conditions: > 80%
- Edge cases: > 70%
- Security scenarios: 100%

## Monitoring and Observability

### Test Instrumentation

**Tracing**:
- Every test generates a trace
- Test setup and teardown spans
- Component interaction spans

**Metrics**:
- Test execution time
- Resource usage
- Flakiness detection
- Success/failure rates

**Logging**:
- Test lifecycle events
- Component interactions
- Error details with context

### Dashboards

**Test Health Dashboard**:
- Pass/fail rates over time
- Flaky test detection
- Execution time trends
- Resource usage patterns

**Integration Point Health**:
- Component interaction success
- External service availability
- Database performance
- Container health

## Troubleshooting Guide

### Common Issues

**Docker Not Available**:
```bash
# Check Docker status
docker ps

# Start Docker Desktop or daemon
open -a Docker  # macOS
sudo systemctl start docker  # Linux
```

**Port Conflicts**:
```bash
# Find process using port
lsof -i :4317

# Kill process
kill -9 <PID>
```

**Database Connection Failures**:
```bash
# Check SurrealDB status
docker logs surrealdb-test

# Restart database
docker-compose restart surrealdb
```

**Test Flakiness**:
- Add explicit waits for async operations
- Increase timeouts for slow operations
- Check for race conditions
- Verify test isolation

## Continuous Improvement

### Metrics to Track

1. **Test Reliability**: Flakiness rate < 1%
2. **Execution Speed**: Consistent execution time
3. **Coverage Trends**: Increasing over time
4. **Bug Detection**: Integration tests finding issues

### Review Cadence

- Weekly: Review flaky tests
- Monthly: Analyze coverage trends
- Quarterly: Strategy effectiveness review
- Annually: Major strategy revisions

## References

- [Testing Best Practices](./TESTING_BEST_PRACTICES.md)
- [Docker Test Environment](./DOCKER_TEST_ENVIRONMENT.md)
- [CI/CD Pipeline](../.github/workflows/integration-tests.yml)
- [Test Data Management](./TEST_DATA_MANAGEMENT.md)
