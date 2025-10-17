# Homebrew Installation Validation

## Overview

This integration test validates the complete clnrm installation and execution loop using **OTEL-first validation**. Unlike traditional integration tests that rely on exit codes, this test proves correctness through OpenTelemetry spans.

### What This Test Does

1. **Install**: Homebrew installs clnrm in a fresh container
2. **Execute**: Installed clnrm runs self-test with OTEL tracing enabled
3. **Validate**: OTEL spans prove successful execution
4. **Verify**: All validators pass and digest is recorded

### Why This Matters

Traditional integration tests check exit codes:

```bash
brew install clnrm && clnrm self-test
echo $?  # Just checks exit code (0 or non-zero)
```

**OTEL-first validation requires PROOF** through telemetry:

- **Lifecycle events**: `container.start`, `container.exec`, `container.stop`
- **Span graph structure**: Parent→child edges form proper tree
- **Status codes**: All spans have `status.code = OK`
- **Hermetic execution**: No external services accessed
- **Deterministic digests**: Same inputs → same outputs

## Test Architecture

### Components

```
homebrew-install-selftest.clnrm.toml
├── [determinism] seed=42, freeze_clock
├── [services.brew] homebrew/brew:latest container
├── [steps] Install and run clnrm self-test
└── [expect.*] OTEL span validators

Validators:
├── expect.span → Span attribute validation
├── expect.graph → Parent-child edge validation
├── expect.counts → Span count thresholds
├── expect.status → Status code validation
└── expect.hermeticity → Hermetic attribute enforcement
```

### Validators

#### 1. Span Validator (`expect.span`)

Validates individual span attributes:

```toml
[expect.span]
spans = [
    {
        name = "clnrm.run",
        attributes = {
            "clnrm.version" = { exists = true },
            "test.count" = { gte = 1 },
            "otel.kind" = "internal",
        }
    },
]
```

**Proves**: Specific operations occurred with correct metadata.

#### 2. Graph Validator (`expect.graph`)

Validates span tree structure:

```toml
[expect.graph]
edges = [
    { parent = "clnrm.run", child = "clnrm.test" },
    { parent = "clnrm.test", child = "clnrm.container.start" },
]
acyclic = true
max_depth = 5
```

**Proves**: Operations occurred in correct order with proper causality.

#### 3. Count Validator (`expect.counts`)

Validates span count thresholds:

```toml
[expect.counts]
spans_total = { gte = 2, lte = 200 }
spans_per_service = { "clnrm" = { gte = 2 } }
```

**Proves**: Expected number of operations occurred.

#### 4. Status Validator (`expect.status`)

Validates all operations succeeded:

```toml
[expect.status]
all_ok = true
error_count = 0
```

**Proves**: No errors occurred during execution.

#### 5. Hermeticity Validator (`expect.hermeticity`)

Validates hermetic execution:

```toml
[expect.hermeticity]
no_external_services = true
network_isolation = true
```

**Proves**: Test ran in complete isolation.

## Usage

### Prerequisites

- Docker or Podman running
- clnrm installed: `cargo install --path crates/clnrm`
- Rust 1.70+ (for integration tests)

### Running the Test

```bash
cd examples/integration-tests
./run-homebrew-test.sh
```

### Expected Output

```
=== Homebrew Installation Validation ===

This test validates:
  1. Homebrew can install clnrm
  2. Installed clnrm runs self-test
  3. Self-test produces valid OTEL spans
  4. All validators pass on the spans

Running test...

✅ PASS in 45.2s (spans=12, digest=abc123...)

Validators:
  ✅ expect.span: 2/2 spans valid
  ✅ expect.graph: edges present, acyclic
  ✅ expect.counts: spans=12 (gte=2, lte=200)
  ✅ expect.status: all OK
  ✅ expect.hermeticity: no external services

Verifying digest stability...
First run digest: abc123def456...
Second run digest: abc123def456...
✅ Determinism verified: digests match

=== Homebrew Validation Complete ===
```

## Files Generated

### `brew-selftest.report.json`

Full test report with validation results:

```json
{
  "verdict": "pass",
  "duration_ms": 45234.5,
  "spans_collected": 12,
  "errors_total": 0,
  "validators": {
    "span": { "status": "pass", "spans_validated": 2 },
    "graph": { "status": "pass", "edges_validated": 3 },
    "counts": { "status": "pass", "total": 12 },
    "status": { "status": "pass", "all_ok": true },
    "hermeticity": { "status": "pass", "violations": 0 }
  },
  "spans": [ /* full span data */ ]
}
```

### `brew-selftest.trace.sha256`

Deterministic digest for reproducibility:

```
abc123def456789...  # SHA-256 of normalized span tree
```

## Integration Tests (Rust)

### Location

`crates/clnrm-core/tests/integration/homebrew_validation.rs`

### Running

```bash
# Run with network access (pulls Docker images)
cargo test --test homebrew_validation -- --ignored

# Run just the validator check (no Docker needed)
cargo test test_all_validators_exist
```

### Tests

#### `test_homebrew_installation_via_otel_spans`

End-to-end test running actual Homebrew container:

```rust
#[tokio::test]
#[ignore] // Requires Docker and network
async fn test_homebrew_installation_via_otel_spans() -> Result<()> {
    let config = load_config("examples/integration-tests/homebrew-install-selftest.clnrm.toml")?;
    let result = run_test(&config).await?;

    assert_eq!(result.verdict, "pass");
    assert!(result.spans_collected >= 2);
    assert_eq!(result.errors_total, 0);

    Ok(())
}
```

#### `test_all_validators_exist`

Compile-time verification of validator modules:

```rust
#[tokio::test]
async fn test_all_validators_exist() -> Result<()> {
    use clnrm_core::validation::*;

    // These compile if validators exist
    let _span_validator = SpanValidator::new();
    let _graph_validator = GraphValidator::new();
    let _count_validator = CountValidator::new();
    let _status_validator = StatusValidator::new();
    let _hermeticity_validator = HermeticityValidator::new();

    Ok(())
}
```

## CI/CD Integration

### GitHub Actions Example

See `.github-workflow-example.yml`:

```yaml
name: Homebrew Installation Validation

on: [push, pull_request]

jobs:
  homebrew-validation:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install clnrm
        run: cargo install --path crates/clnrm

      - name: Run Homebrew validation test
        run: |
          cd examples/integration-tests
          ./run-homebrew-test.sh

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: homebrew-validation-results
          path: |
            brew-selftest.report.json
            brew-selftest.trace.sha256
```

## Determinism

### Configuration

```toml
[determinism]
seed = 42                           # Fixed random seed
freeze_clock = "2025-01-01T00:00:00Z"  # Fixed timestamp
```

### Guarantees

With determinism enabled:

1. **Same inputs → same outputs**: Identical span trees
2. **Same digest**: SHA-256 of normalized spans matches
3. **Same structure**: Graph topology preserved
4. **Same attributes**: Span metadata identical

### Why It Matters

Deterministic tests enable:

- **Reproducibility**: Same test, same result, every time
- **Debugging**: Isolate non-deterministic bugs
- **Comparison**: Diff outputs across versions
- **Caching**: Skip identical test runs

## Troubleshooting

### Test Fails: "Docker not running"

```bash
# Start Docker
systemctl start docker  # Linux
open -a Docker         # macOS
```

### Test Fails: "clnrm not found"

```bash
# Install clnrm
cargo install --path crates/clnrm

# Verify installation
which clnrm
clnrm --version
```

### Test Fails: "Validator returned error"

Check validator logs in report:

```bash
jq '.validators' brew-selftest.report.json
```

### Determinism Fails: "Digests differ"

Possible causes:

1. **Timestamps included**: Set `include_timestamps = false`
2. **Non-deterministic attributes**: Check span attributes
3. **Container ordering**: Ensure fixed execution order

## Advanced Features

### Custom Validators

Add custom validators to TOML:

```toml
[expect.custom]
validator = "my_custom_validator"
config = { threshold = 100 }
```

Implement in Rust:

```rust
pub struct MyCustomValidator {
    config: CustomConfig,
}

impl Validator for MyCustomValidator {
    fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        // Custom validation logic
    }
}
```

### Multi-Stage Validation

Chain multiple tests:

```bash
./run-homebrew-test.sh
./run-upgrade-test.sh
./run-uninstall-test.sh
```

### Performance Benchmarking

Record performance metrics:

```toml
[expect.performance]
max_duration_ms = 60000
max_memory_mb = 512
```

## Best Practices

### 1. OTEL-First Validation

Always validate via OTEL spans, not exit codes:

```rust
// ❌ WRONG - exit code only
assert_eq!(status.code(), 0);

// ✅ CORRECT - OTEL proof
assert!(spans.iter().any(|s| s.name == "clnrm.run" && s.status == OK));
```

### 2. Comprehensive Validators

Use all validator types:

- **span**: Individual operation validation
- **graph**: Causality validation
- **counts**: Volume validation
- **status**: Success validation
- **hermeticity**: Isolation validation

### 3. Deterministic Configuration

Always set determinism:

```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
```

### 4. Artifact Preservation

Save test artifacts for debugging:

```bash
brew-selftest.report.json  # Full report
brew-selftest.trace.sha256 # Digest
```

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [TOML Reference](../../docs/TOML_REFERENCE.md)
- [Validation Guide](../../docs/VALIDATION.md)
- [Determinism Guide](../../docs/DETERMINISM.md)

## Support

- **Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Discussions**: https://github.com/seanchatmangpt/clnrm/discussions
- **Documentation**: https://clnrm.dev/docs

---

**Key Insight**: This test demonstrates the power of OTEL-first validation. Traditional tests say "it worked" (exit code 0). OTEL-first tests say "here's the complete proof of exactly what happened" (spans, traces, metrics).
