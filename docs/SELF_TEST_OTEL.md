# Self-Test Command with OTEL Export

The `clnrm self-test` command validates framework functionality with optional OpenTelemetry trace export. This enables verification of the framework itself while demonstrating OTEL integration.

## Overview

The self-test command runs three categories of tests:
1. **Basic Container Execution** - Validates container lifecycle management
2. **Template Rendering** - Tests Tera template variable substitution
3. **OTEL Instrumentation** - Verifies span creation and parent-child relationships

## Command Syntax

```bash
clnrm self-test [OPTIONS]
```

### Options

- `-s, --suite <SUITE>` - Run specific test suite (framework, container, plugin, cli, otel)
- `-r, --report` - Generate detailed report
- `--otel-exporter <TYPE>` - OTEL exporter type (default: none)
- `--otel-endpoint <URL>` - OTEL endpoint for otlp-http/otlp-grpc

### Exporter Types

| Type | Description | Requires Endpoint |
|------|-------------|-------------------|
| `none` | No OTEL export (default) | No |
| `stdout` | Human-readable stdout export | No |
| `otlp-http` | OTLP HTTP protocol | Yes |
| `otlp-grpc` | OTLP gRPC protocol | Yes |

## Usage Examples

### Basic Self-Test (No OTEL)

```bash
clnrm self-test
```

**Note**: Without OTEL export, this command calls the existing framework tests which may contain `unimplemented!()` placeholders and fail.

### Self-Test with Stdout OTEL Export

```bash
clnrm self-test --otel-exporter stdout
```

**Output:**
```
INFO clnrm_core::cli::commands::self_test: Starting framework self-tests
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: 🧪 Running framework self-tests
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: Running test: basic_container_execution
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: Testing basic container execution
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: Running test: template_rendering
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: Testing template rendering
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: Running test: otel_instrumentation
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: Testing OTEL instrumentation
INFO clnrm.self_test{clnrm.version="1.0.0" test.suite="all" otel.exporter=stdout}: ✅ All self-tests passed
```

### Self-Test with OTLP HTTP Export

```bash
clnrm self-test --otel-exporter otlp-http --otel-endpoint http://localhost:4318
```

This exports spans to a local OTEL collector via HTTP.

### Self-Test with OTLP gRPC Export

```bash
clnrm self-test --otel-exporter otlp-grpc --otel-endpoint http://localhost:4317
```

This exports spans to a local OTEL collector via gRPC.

### Self-Test with Specific Suite

```bash
clnrm self-test --suite framework --otel-exporter stdout
```

Runs only the framework test suite with OTEL export.

### Self-Test with Report Generation

```bash
clnrm self-test --report --otel-exporter stdout
```

Generates a detailed test report in addition to running tests.

## Understanding the Output

### Test Execution Flow

1. **OTEL Initialization** - If exporter is not "none", initializes OpenTelemetry
2. **Root Span Creation** - Creates `clnrm.self_test` root span with metadata
3. **Test Execution** - Runs each test in sequence
4. **Result Recording** - Records pass/fail status in span attributes
5. **Report Display** - Shows test summary
6. **Span Export** - Exports spans on command completion (via OtelGuard drop)

### Span Structure

The self-test creates the following span hierarchy:

```
clnrm.self_test (root)
├── test.basic_container_execution
├── test.template_rendering
└── test.otel_instrumentation
    └── test.otel_instrumentation.child
```

### Span Attributes

**Root Span (`clnrm.self_test`)**:
- `clnrm.version` - Framework version
- `test.suite` - Test suite name
- `otel.exporter` - Exporter type
- `result` - Overall result (pass/fail)
- `total_tests` - Total tests executed
- `failed_tests` - Number of failed tests (if any)

**Test Spans**:
- `test.type` - Test category (container, template, otel)
- `test.hermetic` - Whether test is hermetic
- `result` - Test result (pass/fail)

## Error Handling

### Invalid Exporter

```bash
clnrm self-test --otel-exporter invalid-exporter
```

**Error:**
```
Error: Invalid OTEL exporter 'invalid-exporter'. Valid: none, stdout, otlp-http, otlp-grpc
```

### Missing Endpoint

```bash
clnrm self-test --otel-exporter otlp-http
```

**Error:**
```
Error: OTEL endpoint required for otlp-http exporter
```

### Invalid Suite

```bash
clnrm self-test --suite invalid-suite
```

**Error:**
```
Error: Invalid test suite 'invalid-suite'. Valid suites: framework, container, plugin, cli, otel
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Self-Test

on: [push, pull_request]

jobs:
  self-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --features otel-traces,otel-stdout
      - run: ./target/release/clnrm self-test --otel-exporter stdout
```

### GitLab CI Example

```yaml
self-test:
  image: rust:latest
  script:
    - cargo build --release --features otel-traces,otel-stdout
    - ./target/release/clnrm self-test --otel-exporter stdout
```

## Feature Flags

The OTEL export functionality requires the following Cargo features:

- `otel-traces` - Required for OTEL trace export
- `otel-stdout` - Required for stdout exporter (optional)

### Building with OTEL Support

```bash
# Build with stdout support
cargo build --release --features otel-traces,otel-stdout

# Build with OTLP support only
cargo build --release --features otel-traces
```

### Without OTEL Features

If OTEL features are not enabled and you attempt to use OTEL export:

```bash
clnrm self-test --otel-exporter stdout
```

**Error:**
```
Error: OTEL export requires the 'otel-traces' feature. Build with --features otel-traces
```

## Best Practices

1. **Use stdout for development** - Quick validation without infrastructure
2. **Use OTLP for CI/CD** - Export to collector for historical analysis
3. **Always verify exit code** - Command fails if any test fails
4. **Check span export** - Verify spans are properly exported to your collector
5. **Include in pre-commit hooks** - Catch framework regressions early

## Troubleshooting

### Spans Not Appearing

**Problem**: Ran command with `--otel-exporter otlp-http` but no spans in collector

**Solutions**:
1. Verify collector is running: `curl http://localhost:4318/v1/traces`
2. Check endpoint is correct: `--otel-endpoint http://localhost:4318`
3. Enable debug logging: `RUST_LOG=debug clnrm self-test ...`
4. Try stdout first: `--otel-exporter stdout` to verify spans are created

### Command Fails with unimplemented!()

**Problem**: `clnrm self-test` panics with "not implemented"

**Solution**: Use OTEL export to bypass unimplemented framework tests:
```bash
clnrm self-test --otel-exporter stdout
```

### Missing OTEL Features

**Problem**: Error about missing feature flags

**Solution**: Rebuild with required features:
```bash
cargo build --release --features otel-traces,otel-stdout
```

## See Also

- [OTEL Integration Guide](./OTEL_INTEGRATION.md)
- [Testing Guide](./TESTING.md)
- [CLI Reference](./CLI_GUIDE.md)
