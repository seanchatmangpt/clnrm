# gVisor OTEL Environment Variables Reference

**Version**: 1.0.0
**Agent**: 6 of 10 - OTEL & Service Configuration Migration
**Last Updated**: 2026-01-08

## Quick Reference

### OTEL Collector Endpoints

```bash
# Set all OTEL collector endpoints for gVisor
export OTEL_OTLP_GRPC_ENDPOINT=127.0.0.1:4317
export OTEL_OTLP_HTTP_ENDPOINT=127.0.0.1:4318
export OTEL_HEALTH_CHECK_ENDPOINT=127.0.0.1:13133
export OTEL_JAEGER_ENDPOINT=127.0.0.1:4317
export OTEL_PPROF_ENDPOINT=127.0.0.1:1777
export OTEL_ZPAGES_ENDPOINT=127.0.0.1:55679
export OTEL_LOG_LEVEL=info
```

### Resource Configuration

```bash
# Memory limits for OTEL collector (gVisor overhead considered)
export GOMEMLIMIT=256MiB
```

### Service Discovery

```bash
# Prometheus scrape targets
export OTEL_COLLECTOR_ENDPOINT=127.0.0.1:8889
```

## Complete Environment Variable List

### OTEL Receiver Configuration

| Variable | Default | Purpose | Example |
|----------|---------|---------|---------|
| `OTEL_OTLP_GRPC_ENDPOINT` | `127.0.0.1:4317` | gRPC receiver endpoint | `127.0.0.1:4317` |
| `OTEL_OTLP_HTTP_ENDPOINT` | `127.0.0.1:4318` | HTTP receiver endpoint | `127.0.0.1:4318` |

### OTEL Exporter Configuration

| Variable | Default | Purpose | Example |
|----------|---------|---------|---------|
| `OTEL_JAEGER_ENDPOINT` | `127.0.0.1:4317` | Jaeger backend endpoint | `127.0.0.1:4317` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | Client export endpoint | `http://127.0.0.1:4318` |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` | Export protocol | `http/protobuf`, `grpc` |

### OTEL Health & Monitoring

| Variable | Default | Purpose | Example |
|----------|---------|---------|---------|
| `OTEL_HEALTH_CHECK_ENDPOINT` | `127.0.0.1:13133` | Health check endpoint | `127.0.0.1:13133` |
| `OTEL_PPROF_ENDPOINT` | `127.0.0.1:1777` | pprof profiling endpoint | `127.0.0.1:1777` |
| `OTEL_ZPAGES_ENDPOINT` | `127.0.0.1:55679` | zpages tracing endpoint | `127.0.0.1:55679` |
| `OTEL_LOG_LEVEL` | `info` | Collector log level | `debug`, `info`, `warn`, `error` |

### Service Discovery

| Variable | Default | Purpose | Example |
|----------|---------|---------|---------|
| `OTEL_COLLECTOR_ENDPOINT` | `127.0.0.1:8889` | Prometheus scrape target | `127.0.0.1:8889` |
| `PROMETHEUS_ENDPOINT` | `127.0.0.1:9090` | Prometheus API endpoint | `127.0.0.1:9090` |
| `JAEGER_UI_ENDPOINT` | `127.0.0.1:16686` | Jaeger UI endpoint | `127.0.0.1:16686` |

### Resource Limits

| Variable | Default | Purpose | Example |
|----------|---------|---------|---------|
| `GOMEMLIMIT` | `256MiB` | Go runtime memory limit | `256MiB`, `512MiB` |
| `GOMAXPROCS` | (auto) | Max Go processes | `2`, `4`, `8` |

### Jaeger Configuration

| Variable | Default | Purpose | Example |
|----------|---------|---------|---------|
| `COLLECTOR_OTLP_ENABLED` | `true` | Enable OTLP receiver | `true`, `false` |
| `SPAN_STORAGE_TYPE` | `memory` | Storage backend | `memory`, `badger` |
| `MEMORY_MAX_TRACES` | `50000` | Max traces in memory | `10000`, `50000`, `100000` |

## Usage Examples

### Running OTEL Tests with gVisor

```bash
#!/bin/bash

# Set gVisor OTEL environment
export OTEL_OTLP_GRPC_ENDPOINT=127.0.0.1:4317
export OTEL_OTLP_HTTP_ENDPOINT=127.0.0.1:4318
export OTEL_HEALTH_CHECK_ENDPOINT=127.0.0.1:13133
export OTEL_JAEGER_ENDPOINT=127.0.0.1:4317
export OTEL_LOG_LEVEL=debug
export GOMEMLIMIT=256MiB

# Start infrastructure
docker-compose -f tests/integration/docker-compose.otel-test.yml up -d

# Run tests
cargo test --test docker_integration -- --test-threads=1

# Cleanup
docker-compose -f tests/integration/docker-compose.otel-test.yml down
```

### Running Weaver Validation with gVisor

```bash
#!/bin/bash

# Set Weaver OTEL environment
export WEAVER_OTLP_GRPC_PORT=4317
export WEAVER_OTLP_HTTP_PORT=4318
export WEAVER_HEALTH_PORT=13133
export WEAVER_METRICS_PORT=8888
export WEAVER_PPROF_PORT=1777
export WEAVER_ZPAGES_PORT=55679

export OTEL_OTLP_GRPC_ENDPOINT=127.0.0.1:4317
export OTEL_OTLP_HTTP_ENDPOINT=127.0.0.1:4318
export OTEL_JAEGER_ENDPOINT=127.0.0.1:4317
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
export OTEL_LOG_LEVEL=info

export GOMEMLIMIT=256MiB
export JAEGER_UI_PORT=16686

# Start Weaver infrastructure
docker-compose -f docker-compose.weaver.yml up -d

# Wait for services to be ready
sleep 10

# Run Weaver validation
cargo test --test weaver_validation

# View traces
echo "Jaeger UI: http://127.0.0.1:16686"

# Cleanup
docker-compose -f docker-compose.weaver.yml down
```

### Running Prometheus Metrics Scrape

```bash
#!/bin/bash

# Configure Prometheus for gVisor
export OTEL_COLLECTOR_ENDPOINT=127.0.0.1:8889
export PROMETHEUS_ENDPOINT=127.0.0.1:9090

# Query Prometheus API
curl "http://127.0.0.1:9090/api/v1/query?query=up"

# Get metric targets
curl "http://127.0.0.1:9090/api/v1/targets"

# Export metrics
curl "http://127.0.0.1:9090/api/v1/query?query={__name__=~'otel.*'}"
```

## Docker vs gVisor Configuration

### Docker (Original)

```bash
# Docker service DNS
export OTEL_JAEGER_ENDPOINT=jaeger:4317
export OTEL_COLLECTOR_ENDPOINT=otel-collector:8889
export OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4318

# Port binding to all interfaces
docker-compose up
```

### gVisor (Migrated)

```bash
# Localhost endpoints for gVisor
export OTEL_JAEGER_ENDPOINT=127.0.0.1:4317
export OTEL_COLLECTOR_ENDPOINT=127.0.0.1:8889
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318

# Explicit localhost port binding
docker-compose -f docker-compose.otel-test.yml up
```

## Configuration Files

### OTEL Collector

**Path**: `/home/user/clnrm/config/otel-collector-config.yaml`

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4317  # Use env var: OTEL_OTLP_GRPC_ENDPOINT
      http:
        endpoint: 127.0.0.1:4318  # Use env var: OTEL_OTLP_HTTP_ENDPOINT

exporters:
  otlp/jaeger:
    endpoint: 127.0.0.1:4317  # Use env var: OTEL_JAEGER_ENDPOINT
    tls:
      insecure: true

extensions:
  health_check:
    endpoint: 127.0.0.1:13133  # Use env var: OTEL_HEALTH_CHECK_ENDPOINT
```

### Prometheus

**Path**: `/home/user/clnrm/tests/integration/prometheus-config.yml`

```yaml
scrape_configs:
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['127.0.0.1:8889']  # Use env var: OTEL_COLLECTOR_ENDPOINT
```

### Docker Compose

**Path**: `/home/user/clnrm/docker-compose.weaver.yml`

```yaml
services:
  otel-collector:
    ports:
      - "127.0.0.1:4317:4317"  # Explicit localhost binding
    environment:
      - OTEL_OTLP_GRPC_ENDPOINT=127.0.0.1:4317
      - OTEL_OTLP_HTTP_ENDPOINT=127.0.0.1:4318
      - OTEL_JAEGER_ENDPOINT=127.0.0.1:4317
      - GOMEMLIMIT=256MiB
```

## Troubleshooting

### Connection Refused Error

**Problem**: `Connection refused` when connecting to OTEL collector

**Solution**:
```bash
# Verify OTEL collector is running
docker ps | grep otel-collector

# Verify endpoint is correct
export OTEL_OTLP_GRPC_ENDPOINT=127.0.0.1:4317  # Not 0.0.0.0!

# Test connectivity
curl http://127.0.0.1:13133/healthz
```

### Health Check Failed

**Problem**: Health check endpoint not responding

**Solution**:
```bash
# Check OTEL collector logs
docker logs clnrm-otel-collector

# Verify health check endpoint
export OTEL_HEALTH_CHECK_ENDPOINT=127.0.0.1:13133

# Manual health check
curl -v http://127.0.0.1:13133/healthz
```

### Jaeger Not Receiving Traces

**Problem**: Traces not appearing in Jaeger UI

**Solution**:
```bash
# Verify Jaeger is running
docker ps | grep jaeger

# Verify OTEL exporter endpoint
export OTEL_JAEGER_ENDPOINT=127.0.0.1:4317  # Not jaeger:4317!

# Check Jaeger logs
docker logs clnrm-jaeger

# Query Jaeger API
curl http://127.0.0.1:16686/api/services
```

### Memory Errors in gVisor

**Problem**: Out of memory errors in gVisor containers

**Solution**:
```bash
# Increase memory limits in docker-compose
# For OTEL collector: 768M (256M collector + 512M gVisor overhead)
# For Jaeger: 512M (256M Jaeger + 256M gVisor overhead)

# Set Go memory limit
export GOMEMLIMIT=512MiB  # Increase if needed

# Restart services
docker-compose -f docker-compose.otel-test.yml restart
```

## Summary Table

| Component | Docker Default | gVisor Default | Purpose |
|-----------|---|---|---|
| OTLP gRPC | `0.0.0.0:4317` | `127.0.0.1:4317` | Telemetry ingestion |
| OTLP HTTP | `0.0.0.0:4318` | `127.0.0.1:4318` | Telemetry ingestion |
| Jaeger gRPC | `jaeger:4317` | `127.0.0.1:4317` | Trace export |
| Metrics | `0.0.0.0:8888` | `127.0.0.1:8888` | OTEL metrics |
| Health | `0.0.0.0:13133` | `127.0.0.1:13133` | Readiness checks |
| Prometheus | `localhost:9090` | `127.0.0.1:9090` | Metrics query |
| Jaeger UI | `0.0.0.0:16686` | `127.0.0.1:16686` | Trace visualization |

## Related Documentation

- [GVISOR_OTEL_MIGRATION_GUIDE.md](GVISOR_OTEL_MIGRATION_GUIDE.md) - Detailed migration guide
- [GVISOR_QUICK_REFERENCE.md](GVISOR_QUICK_REFERENCE.md) - Quick start guide
- [docs/design/gvisor-otel-integration.md](design/gvisor-otel-integration.md) - Design document
- [registry/core/gvisor_container.yaml](../registry/core/gvisor_container.yaml) - Weaver schema
