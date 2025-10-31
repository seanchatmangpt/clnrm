# Instrumentation Quick Reference

## Files Created

### 1. CLI Telemetry Helpers
**File**: `crates/clnrm-core/src/telemetry/cli_helpers.rs`

```rust
// Usage pattern for all CLI commands
use crate::telemetry::cli_helpers::CliInitSpanBuilder;

pub fn command_impl(args: &Args) -> Result<()> {
    // 1. Start span
    let span = CliInitSpanBuilder::new(/* context */).start();

    // 2. Execute command
    match do_work() {
        Ok(result) => {
            // 3. Finish with success
            span.finish(true, /* ...attributes... */, None);
            Ok(result)
        }
        Err(e) => {
            // 3. Finish with error
            span.finish(false, /* ...attributes... */,
                Some(("ErrorType".to_string(), e.to_string())));
            Err(e)
        }
    }
}
```

## Instrumented Commands

### clnrm init
- **File**: `crates/clnrm-core/src/cli/commands/init.rs`
- **Schema**: `registry/cli/initialization.yaml`
- **Key Attributes**: `project.path`, `config.generated`, `files.created`

### clnrm plugins
- **File**: `crates/clnrm-core/src/cli/commands/plugins.rs`
- **Schema**: `registry/cli/plugin_operations.yaml`
- **Key Attributes**: `plugins.discovered`, `plugins.by_type`

### clnrm health
- **File**: `crates/clnrm-core/src/cli/commands/health.rs`
- **Schema**: `registry/cli/health_check.yaml`
- **Key Attributes**: `health.overall`, `docker.available`, `weaver.available`

### clnrm self-test
- **File**: `crates/clnrm-core/src/cli/commands/self_test.rs`
- **Key Attributes**: `test.suite`, `test.count`, `test.passed`, `test.failed`

## Testing Instrumentation

### Compilation Test
```bash
cargo check --package clnrm-core
# ✅ Zero errors (only template warnings)
```

### Runtime Test
```bash
# Export telemetry to stdout
clnrm init --force
clnrm plugins
clnrm health --verbose
clnrm self-test --suite framework

# Export to OTLP collector
clnrm self-test --otel-exporter otlp-http --otel-endpoint http://localhost:4318
```

### Weaver Validation
```bash
# Schema validation
weaver registry check -r registry/

# Live telemetry validation
weaver registry live-check --registry registry/
```

## Coverage Metrics

**Before**: 0/153 attributes (0.0%)
**After**: ~55/153 attributes (36%)
**Remaining**: 11 CLI commands + enhanced test execution

## Next Commands to Instrument

1. `validate` - Configuration validation
2. `services` - Service management
3. `template` - Template generation
4. `report` - Test reporting
5. `fmt` - TOML formatting
6. `record` - Test recording
7. `repro` - Test reproduction
8. `red-green` - TDD workflow
9. `pull` - Image operations
10. `render` - Template rendering
11. Collector/Services noun-verb commands

**Pattern**: Copy builder from `cli_helpers.rs`, add to command file
