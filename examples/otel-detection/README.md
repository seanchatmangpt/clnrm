# OTEL Collector Detection Examples

Quick reference for OpenTelemetry Collector detection and validation in testcontainers.

## Examples

### Basic Collector (`basic-collector.clnrm.toml`)
- Single OTEL Collector service
- Endpoint detection (gRPC, HTTP, health check)
- 5 validation steps
- **Use case**: Verify collector is running and endpoints are accessible

**Run**:
```bash
cargo run -- run examples/otel-detection/basic-collector.clnrm.toml
```

### App with Collector (`app-with-collector.clnrm.toml`)
- OTEL Collector + Application service
- Full OTEL configuration for app
- 7 validation steps including telemetry generation
- **Use case**: Test application sending telemetry to collector

**Run**:
```bash
cargo run -- run examples/otel-detection/app-with-collector.clnrm.toml
```

### Advanced Validation (`advanced-validation.clnrm.toml`)
- OTEL Collector with all features enabled
- Two application services (primary + secondary)
- 12 comprehensive validation steps
- Prometheus metrics, zPages, trace context propagation
- **Use case**: Complete observability testing scenario

**Run**:
```bash
cargo run -- run examples/otel-detection/advanced-validation.clnrm.toml
```

## Quick Start

### Minimal TOML Configuration
```toml
[test.metadata]
name = "my_otel_test"

[services.otel_collector]
type = "otel_collector"
plugin = "otel_collector"

[[steps]]
name = "verify"
command = ["echo", "OTEL Collector is running"]
```

### With Application
```toml
[services.otel_collector]
type = "otel_collector"
plugin = "otel_collector"

[services.app]
type = "generic_container"
plugin = "generic_container"
image = "myapp:latest"

[services.app.env]
OTEL_EXPORTER_OTLP_ENDPOINT = "http://otel_collector:4318"
OTEL_SERVICE_NAME = "my-app"
OTEL_TRACES_EXPORTER = "otlp"
```

## Configuration Options

### Environment Variables
```toml
[services.otel_collector.env]
ENABLE_OTLP_GRPC = "true"    # Default: true
ENABLE_OTLP_HTTP = "true"    # Default: true
ENABLE_PROMETHEUS = "false"  # Default: false
ENABLE_ZPAGES = "false"      # Default: false
```

### Available Endpoints
| Endpoint | Port | Metadata Key |
|----------|------|--------------|
| OTLP gRPC | 4317 | `otlp_grpc_endpoint` |
| OTLP HTTP | 4318 | `otlp_http_endpoint` |
| Health Check | 13133 | `health_check_endpoint` |
| Prometheus | 8889 | `prometheus_endpoint` |
| zPages | 55679 | N/A |

## Prerequisites

- Docker running: `docker ps` should work
- clnrm built: `cargo build --release`
- Network access for image pull

## Testing

```bash
# Run all OTEL examples
cargo run -- run examples/otel-detection/*.toml

# Run specific example
cargo run -- run examples/otel-detection/basic-collector.clnrm.toml

# With verbose output
RUST_LOG=debug cargo run -- run examples/otel-detection/basic-collector.clnrm.toml
```

## Expected Output

```
🚀 Executing test: basic_otel_collector_detection
📦 Registered service plugin: otel_collector
✅ Service 'otel_collector' started successfully
  Metadata:
    - otlp_grpc_endpoint: http://127.0.0.1:xxxxx
    - otlp_http_endpoint: http://127.0.0.1:xxxxx
    - health_check_endpoint: http://127.0.0.1:xxxxx
✅ All steps passed
Test Results: 1 passed, 0 failed
```

## Troubleshooting

### Docker not running
```bash
# macOS
open -a Docker

# Linux
sudo systemctl start docker
```

### Image not found
```bash
docker pull otel/opentelemetry-collector:latest
```

### Port conflict
Stop other services using ports 4317, 4318, or 13133

## Documentation

- [Complete Guide](../../docs/OTEL_COLLECTOR_DETECTION.md)
- [Implementation Summary](../../docs/OTEL_DETECTION_IMPLEMENTATION_SUMMARY.md)
- [TOML Reference](../../docs/TOML_REFERENCE.md)

## Support

Issues: https://github.com/seanchatmangpt/clnrm/issues
