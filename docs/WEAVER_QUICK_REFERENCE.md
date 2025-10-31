# Weaver Code Generation - Quick Reference

One-page reference for common Weaver operations in the clnrm project.

## Installation

```bash
cargo install weaver-cli
weaver --version
```

## Generate Code

```bash
# Full generation
weaver registry generate rust \
  --registry registry/ \
  --templates templates/registry/rust/ \
  --output crates/clnrm-core/src/telemetry/generated/

# Or just use cargo build (automatic via build.rs)
cargo build
```

## Verify Setup

```bash
./tests/verify_weaver_setup.sh
```

## File Locations

| Purpose | Path |
|---------|------|
| **Templates** | `templates/registry/rust/*.j2` |
| **Configuration** | `templates/registry/rust/weaver.yaml` |
| **Schemas** | `registry/*.yaml` |
| **Generated Code** | `crates/clnrm-core/src/telemetry/generated/` |
| **Build Script** | `build.rs` |

## Template Files

| File | Generates |
|------|-----------|
| `spans.rs.j2` | Type-safe span builders |
| `metrics.rs.j2` | Metric recorders (feature-gated) |
| `mocks.rs.j2` | Mockall traits for testing |
| `events.rs.j2` | Structured event emitters |

## Common Commands

```bash
# Validate schemas
weaver registry check --registry registry/

# List available templates
ls templates/registry/rust/*.j2

# View generated code
cat crates/clnrm-core/src/telemetry/generated/spans.rs

# Clean generated code
rm -rf crates/clnrm-core/src/telemetry/generated/*.rs

# Regenerate
cargo clean && cargo build
```

## Usage Patterns

### Span Creation
```rust
use clnrm_core::telemetry::generated::spans::TestExecutionSpan;

let span = TestExecutionSpan::new("test", "alpine:latest");
span.set_isolated(true);
let _guard = span.enter();
```

### Metrics Recording
```rust
#[cfg(feature = "otel-metrics")]
use clnrm_core::telemetry::generated::metrics::TestExecutionMetric;

let metric = TestExecutionMetric::new(&meter);
metric.record(duration_ms, "test", "pass");
```

### Mock Testing
```rust
#[cfg(test)]
use clnrm_core::telemetry::generated::mocks::MockTestExecutionSpanTrait;

let mut mock = MockTestExecutionSpanTrait::new();
mock.expect_set_result().times(1).returning(|_| ());
```

## Feature Flags

```bash
# Build with metrics
cargo build --features otel-metrics

# Build without metrics (metrics are no-ops)
cargo build
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `weaver: command not found` | `cargo install weaver-cli` |
| Template syntax error | Check Jinja2 syntax in `*.j2` files |
| Generated code won't compile | Verify type mappings in templates |
| Schemas not found | Create `registry/` directory with YAML files |

## Type Mappings

| OTel Type | Rust Type |
|-----------|-----------|
| `string` | `&str` |
| `int` | `i64` |
| `double` | `f64` |
| `boolean` | `bool` |
| `string[]` | `Vec<String>` |

## Schema Example

```yaml
groups:
  - id: test.execution
    type: span
    brief: "Test execution span"
    attributes:
      - id: test.name
        type: string
        requirement_level: required
      - id: test.isolated
        type: boolean
        requirement_level: optional
```

## CI/CD Integration

```yaml
# .github/workflows/ci.yml
- name: Install Weaver
  run: cargo install weaver-cli

- name: Generate Code
  run: weaver registry generate rust ...

- name: Verify No Changes
  run: git diff --exit-code
```

## Documentation

- **Full Guide**: [WEAVER_CODEGEN_GUIDE.md](WEAVER_CODEGEN_GUIDE.md)
- **Examples**: [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)
- **Template Docs**: [templates/registry/rust/README.md](../templates/registry/rust/README.md)

## Support

- [Weaver GitHub](https://github.com/open-telemetry/weaver)
- [OTel Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [clnrm Issues](https://github.com/seanchatmangpt/clnrm/issues)
