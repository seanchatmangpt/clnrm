# Run Framework Self-Test

Execute clnrm's comprehensive self-test suite to validate the framework using its own testing capabilities ("eating our own dogfood").

## Critical: Use Homebrew Installation

**ALWAYS use the Homebrew-installed binary for validation**, not `cargo run`:

```bash
# ✅ CORRECT - Production binary
clnrm self-test

# ❌ WRONG - Development build
cargo run -- self-test
```

## Why Dogfooding Matters

The framework validates itself using the production installation path. This ensures:
- Real-world usage patterns are tested
- Installation issues are caught early
- Production binary matches development behavior
- Framework can test itself hermetically

## Running Self-Tests

### Quick Self-Test
```bash
# Run all self-tests
clnrm self-test

# Run specific test suite
clnrm self-test --suite basic
clnrm self-test --suite container
clnrm self-test --suite service
clnrm self-test --suite otel
```

### With OTEL Validation
```bash
# Run OTEL self-tests with stdout exporter
clnrm self-test --suite otel --otel-exporter stdout

# Run with OTLP HTTP exporter
clnrm self-test --suite otel --otel-exporter http://localhost:4318
```

### Verbose Output
```bash
# Show detailed test execution
clnrm self-test --verbose

# Show container logs
clnrm self-test --verbose --show-logs
```

## Self-Test Suites

The framework includes several self-test suites:

### 1. Basic Suite
- Framework initialization
- Configuration loading
- Environment setup
- Basic validation

### 2. Container Suite
- Container creation and lifecycle
- Command execution
- Output capture
- Cleanup verification

### 3. Service Suite
- Service plugin registration
- Service start/stop
- Health checks
- Service discovery

### 4. OTEL Suite (with `--features otel`)
- Telemetry initialization
- Trace emission
- Metric collection
- Log export

## Installation for Testing

```bash
# Build and install from source
cargo build --release --features otel
brew uninstall clnrm  # if already installed
brew install --build-from-source .

# Verify installation
which clnrm
# Should show: /opt/homebrew/bin/clnrm (or /usr/local/bin/clnrm)

clnrm --version
```

## Integration Tests (Cargo)

While self-tests use the production binary, integration tests use cargo:

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test integration_otel
cargo test --test container_isolation_test
cargo test --test v1_compliance_comprehensive

# Run with OTEL features
cargo test --features otel --test integration_otel
```

## Unit Tests

```bash
# Run unit tests only
cargo test --lib

# Run specific crate unit tests
cargo test -p clnrm-core --lib
cargo test -p clnrm-shared --lib

# Run with all features
cargo test --all-features --lib
```

## Property-Based Tests (Future)

```bash
# Run property tests (160K+ generated cases)
cargo test --features proptest

# Run specific property test
cargo test --features proptest test_toml_parsing_properties
```

## Expected Results

Self-tests should complete with:
```
✅ All tests passed (X/X)
⏱️  Total time: ~30-60 seconds
📊 Container operations: Y
🔍 Services tested: Z
```

## Troubleshooting

### Self-Test Fails
1. Verify Homebrew installation: `which clnrm`
2. Check Docker is running: `docker ps`
3. Ensure no port conflicts
4. Review logs with `--verbose`

### Installation Issues
```bash
# Reinstall from source
brew uninstall clnrm
cargo clean
cargo build --release --features otel
brew install --build-from-source .
```

### Docker Issues
```bash
# Check Docker daemon
docker info

# Verify testcontainers works
docker run --rm alpine:latest echo "test"
```

## CI Integration

Self-tests run in CI using the production binary:

```yaml
- name: Install clnrm
  run: brew install --build-from-source .

- name: Run self-tests
  run: clnrm self-test --verbose
```

## Documentation

See `docs/TESTING.md` and `README.md` for complete testing documentation.
