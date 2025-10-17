# Validate OTEL Integration

Verify that OpenTelemetry integration actually emits traces, metrics, and logs to a real OTEL collector.

## Why This Matters

It's not enough to just **enable** OTEL features - we must **verify** that:
- Traces are actually exported to the collector
- Spans have proper parent-child relationships
- Metrics are recorded and exported
- The collector receives and processes telemetry
- Service name and attributes are correct

## Running OTEL Validation

### Quick Validation (Script)

```bash
# Run validation script
bash scripts/validate-otel.sh

# The script will:
# 1. Start OTEL collector in Docker
# 2. Run OTEL-enabled tests
# 3. Verify collector received traces
# 4. Generate validation report
```

### Full Integration Tests

```bash
# Start OTEL infrastructure
docker-compose -f tests/integration/docker-compose.otel-test.yml up -d

# Wait for collector to be healthy
curl http://localhost:13133/health

# Run OTEL validation tests
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
  cargo test --features otel --test otel_validation_integration -- --ignored

# Check collector metrics
curl http://localhost:8888/metrics | grep otelcol_receiver_accepted_spans

# View traces in Jaeger UI
open http://localhost:16686

# Cleanup
docker-compose -f tests/integration/docker-compose.otel-test.yml down -v
```

## Test Coverage

The validation suite includes:

1. **`test_otel_traces_are_emitted_to_collector()`**
   - Verifies spans are exported via OTLP HTTP
   - Checks collector received traces
   - Validates trace structure

2. **`test_otel_metrics_are_recorded_and_exported()`**
   - Verifies metrics emission
   - Checks metric values are accurate
   - Validates metric export

3. **`test_span_relationships_preserved_in_export()`**
   - Verifies parent-child span relationships
   - Checks span attributes
   - Validates trace context propagation

4. **`test_collector_health_check()`**
   - Verifies collector is reachable
   - Checks all endpoints (gRPC 4317, HTTP 4318, health 13133)
   - Validates collector configuration

## OTEL Infrastructure

The docker-compose setup includes:

- **OTEL Collector** (OTLP receiver, logging exporter)
- **Jaeger** (trace visualization on :16686)
- **Prometheus** (metrics collection on :9090)

### Endpoints

- OTLP gRPC: `localhost:4317`
- OTLP HTTP: `localhost:4318`
- Health check: `localhost:13133`
- zpages (debug): `localhost:55679`
- Jaeger UI: `localhost:16686`
- Prometheus: `localhost:9090`

## Debugging

### View Collector Logs
```bash
docker-compose -f tests/integration/docker-compose.otel-test.yml logs otel-collector
```

### Check zpages (Trace Debug)
```bash
# View active traces
curl http://localhost:55679/debug/tracez

# View active spans
curl http://localhost:55679/debug/spanprocessorz
```

### Query Jaeger
```bash
# Find traces by service name
curl 'http://localhost:16686/api/traces?service=clnrm'
```

### Check Collector Metrics
```bash
# Verify spans received
curl http://localhost:8888/metrics | grep otelcol_receiver_accepted_spans

# Verify spans exported
curl http://localhost:8888/metrics | grep otelcol_exporter_sent_spans
```

## Common Issues

### Collector Not Starting
- Check Docker is running
- Verify ports 4317, 4318, 13133 are available
- Check collector config syntax

### No Traces Received
- Verify `OTEL_EXPORTER_OTLP_ENDPOINT` is set
- Check network connectivity to collector
- Ensure OTEL features are enabled (`--features otel`)
- Review collector logs for errors

### Tests Fail
- Ensure collector is healthy before running tests
- Check test output for specific failures
- Verify collector configuration matches test expectations

## CI Integration

The OTEL validation job runs automatically in `.github/workflows/integration-tests.yml`:

```yaml
otel-validation:
  runs-on: ubuntu-latest
  steps:
    - name: Start OTEL Collector
    - name: Wait for healthy
    - name: Run OTEL tests
    - name: Verify traces received
    - name: Upload logs on failure
```

## Documentation

See `/docs/implementation/otel-validation-testing.md` for complete technical details.
