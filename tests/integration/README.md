# Integration Tests

This directory contains comprehensive integration tests for the CLNRM (Cleanroom Testing Framework) project.

## Directory Structure

```
tests/integration/
├── README.md                          # This file
├── mod.rs                             # Module exports
├── common/                            # Shared test utilities
│   └── mod.rs                         # Re-exports helpers, factories, fixtures
├── helpers/                           # Test helper utilities
│   └── mod.rs                         # TestContext, guards, utilities
├── factories/                         # Test data builders
│   └── mod.rs                         # Builder patterns for test data
├── fixtures/                          # Pre-defined test data
│   └── mod.rs                         # Static test data sets
├── assertions/                        # Custom assertions
│   └── mod.rs                         # Domain-specific assertion methods
├── component_integration_test.rs      # Component integration tests
├── system_integration_test.rs         # System-level integration tests
├── database_integration_test.rs       # Database integration tests
├── external_service_test.rs           # External service mocks/stubs
├── docker-compose.test.yml            # Test environment infrastructure
├── otel-collector-config.yml          # OpenTelemetry configuration
└── prometheus-config.yml              # Prometheus configuration

## Running Tests

### All Integration Tests

```bash
# Run all integration tests (fast, no Docker required)
cargo test --test '*'

# Run with Docker-dependent tests
cargo test --test '*' -- --ignored

# Run with specific thread count
cargo test --test '*' -- --test-threads=4
```

### Specific Test Suites

```bash
# Component integration tests
cargo test --test component_integration_test

# System integration tests (requires Docker)
cargo test --test system_integration_test -- --ignored

# Database integration tests (requires SurrealDB)
cargo test --test database_integration_test -- --ignored

# External service tests
cargo test --test external_service_test
```

### Test Environment Setup

```bash
# Start test infrastructure
docker-compose -f tests/integration/docker-compose.test.yml up -d

# Check service health
docker-compose -f tests/integration/docker-compose.test.yml ps

# View logs
docker-compose -f tests/integration/docker-compose.test.yml logs -f

# Stop infrastructure
docker-compose -f tests/integration/docker-compose.test.yml down -v
```

## Test Categories

### Component Integration Tests

**Purpose**: Validate interactions between 2-3 related components

**Characteristics**:
- Fast execution (< 5s per test)
- No external dependencies
- Focus on interface contracts
- Use test doubles for expensive operations

**Examples**:
- Backend + Policy validation
- Command execution + Result aggregation
- Configuration loader + Backend factory

### System Integration Tests

**Purpose**: Validate end-to-end workflows

**Characteristics**:
- Moderate execution time (5-30s per test)
- May require Docker containers
- Test complete workflows
- Include cleanup mechanisms

**Examples**:
- Full test execution pipeline
- Container lifecycle management
- Multi-backend orchestration

### Database Integration Tests

**Purpose**: Validate data persistence and retrieval

**Characteristics**:
- Use test database instances
- Transaction-based isolation
- Schema validation
- Migration testing

**Examples**:
- Result storage and retrieval
- Configuration persistence
- Query performance

### External Service Integration Tests

**Purpose**: Validate external service integration

**Characteristics**:
- Use mocks/stubs by default
- Optional real service testing
- Contract validation
- Error handling

**Examples**:
- OpenTelemetry integration
- Container registry interaction
- API endpoint validation

## Test Utilities

### TestContext

Provides isolated test environment:

```rust
let ctx = TestContext::new()?;

// Create test files
ctx.create_file("config.toml", "content")?;

// Read test files
let content = ctx.read_file("config.toml")?;

// Automatic cleanup on drop
```

### Builders (Factories)

Fluent API for test data creation:

```rust
let backend = BackendConfigBuilder::new()
    .name("test-backend")
    .image("alpine")
    .tag("latest")
    .env("KEY", "value")
    .build();

let cmd = CommandBuilder::new("echo")
    .arg("hello")
    .env("TEST", "true")
    .build();
```

### Fixtures

Pre-defined test data sets:

```rust
let config = ConfigFixture::default_alpine();
let cmd = CommandFixture::echo_hello();
let result = ResultFixture::successful_execution();
```

### Assertions

Domain-specific assertion helpers:

```rust
// Async assertions
assert_completes_within(future, Duration::from_secs(5), "Should complete quickly").await;

assert_eventually(
    || async { condition_is_true() },
    Duration::from_secs(10),
    "Condition should eventually be true"
).await;

// Duration assertions
assert_duration_approx_eq(actual, expected, tolerance_ms);

// Collection assertions
assert_contains_all(&collection, &expected_items);
assert_contains_none(&collection, &unexpected_items);
```

## Best Practices

### Test Organization

1. **Arrange-Act-Assert Pattern**:
   ```rust
   #[test]
   fn test_example() -> Result<()> {
       // Arrange: Setup
       let ctx = TestContext::new()?;
       let backend = create_test_backend();

       // Act: Execute
       let result = backend.run_command(cmd)?;

       // Assert: Verify
       assert_eq!(result.exit_code, 0);
       Ok(())
   }
   ```

2. **Test Isolation**: Each test should be independent
   ```rust
   // ✅ Good: Isolated
   #[test]
   fn test_1() -> Result<()> {
       let ctx = TestContext::new()?;
       // Test in isolation
       Ok(())
   }

   // ❌ Bad: Shared state
   static mut SHARED: i32 = 0;
   ```

3. **Cleanup**: Use RAII or explicit cleanup
   ```rust
   let _guard = TestGuard::new(|| {
       // Cleanup code
   });
   // Cleanup runs on drop
   ```

### Performance

1. **Parallel Execution**: Tests run in parallel by default
   ```bash
   cargo test -- --test-threads=4
   ```

2. **Fast Tests First**: Quick feedback loop
   - Unit tests: < 100ms
   - Component tests: < 5s
   - System tests: < 30s

3. **Resource Management**:
   - Reuse containers when safe
   - Use connection pools
   - Clean up after tests

### Debugging

1. **View Output**:
   ```bash
   cargo test -- --nocapture
   ```

2. **Run Single Test**:
   ```bash
   cargo test test_name -- --exact
   ```

3. **Enable Logging**:
   ```bash
   RUST_LOG=debug cargo test
   ```

4. **Container Logs**:
   ```bash
   docker-compose -f tests/integration/docker-compose.test.yml logs -f
   ```

## CI/CD Integration

Tests run automatically in GitHub Actions:

- **Pull Requests**: Component and fast system tests
- **Main Branch**: All integration tests
- **Nightly**: Extended tests and stress tests

See `.github/workflows/integration-tests.yml` for details.

## Troubleshooting

### Docker Not Available

```bash
# Check Docker status
docker ps

# Start Docker
systemctl start docker  # Linux
open -a Docker          # macOS
```

### Port Conflicts

```bash
# Find process using port
lsof -i :4317

# Stop conflicting services
docker-compose down -v
```

### Database Connection Failures

```bash
# Check SurrealDB logs
docker logs clnrm-test-surrealdb

# Restart database
docker-compose restart surrealdb
```

### Test Timeouts

```bash
# Increase timeout
cargo test -- --test-threads=1 --nocapture
```

## Contributing

When adding new integration tests:

1. Follow existing patterns and structure
2. Use builders and fixtures for test data
3. Ensure tests are isolated and repeatable
4. Add documentation for complex scenarios
5. Update this README if adding new categories

## Resources

- [Integration Test Strategy](../../docs/INTEGRATION_TEST_STRATEGY.md)
- [Docker Test Environment](../../docs/DOCKER_TEST_ENVIRONMENT.md)
- [Testing Best Practices](../../docs/TESTING_BEST_PRACTICES.md)
