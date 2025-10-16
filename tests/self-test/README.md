# clnrm Self-Test Suite

Self-testing suite that validates clnrm functionality via OpenTelemetry spans.

## Quick Start

### Prerequisites

- Docker running: `docker ps` should work
- Rust 1.70+
- clnrm built with OTEL features: `cargo build --release --features otel-traces`

### Build and Run

```bash
# 1. Build clnrm Docker image
docker build -t clnrm:test .

# 2. Run self-test
cargo run --release --features otel-traces -- run tests/self-test/clnrm-otel-validation.clnrm.toml
```

## Files

| File | Purpose |
|------|---------|
| `clnrm-otel-validation.clnrm.toml` | Main self-test configuration |
| `inner-test.clnrm.toml` | Test executed inside clnrm container |
| `otel-collector-config.yaml` | OTEL Collector configuration |
| `README.md` | This file |

## How It Works

1. **OTEL Collector** starts and captures telemetry on port 4318
2. **clnrm Container** runs with OTEL instrumentation enabled
3. **Inner Test** executes inside container, emitting spans
4. **Span Validator** reads exported spans and validates:
   - `clnrm.run` span exists (proves execution)
   - `clnrm.test` span exists (proves test ran)
   - `clnrm.service.start` span exists (proves container lifecycle)
   - `clnrm.command.execute` span exists (proves command execution)
   - Span hierarchy is correct (proves orchestration)

## Expected Spans

```
clnrm.run
  └─ clnrm.test
      ├─ clnrm.service.start
      └─ clnrm.command.execute
```

## Validation Strategy

Instead of traditional assertions, we validate that:
1. Expected spans were emitted
2. Spans have correct attributes
3. Span hierarchy reflects proper execution flow
4. Span timestamps show reasonable duration

If these spans exist with correct structure, we've **proven** clnrm works.

## Troubleshooting

### Docker Issues

```bash
# Check Docker is running
docker ps

# Check image exists
docker images | grep clnrm

# Rebuild image
docker build --no-cache -t clnrm:test .
```

### Span Export Issues

```bash
# Check collector logs
docker ps  # Find otel_collector container ID
docker logs <container_id>

# Manually check spans
cat /tmp/clnrm-spans.json
```

### Build Issues

```bash
# Clean and rebuild
cargo clean
cargo build --release --features otel-traces
```

## Documentation

See [CLNRM_SELF_TESTING.md](../../docs/CLNRM_SELF_TESTING.md) for complete documentation.

## Status

✅ **Production Ready** - All components functional and tested
