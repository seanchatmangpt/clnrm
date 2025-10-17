# Add OpenTelemetry Integration - Observability Setup

Add OpenTelemetry (OTEL) integration to clnrm following kgold patterns.

## What This Does

Implements production-grade OpenTelemetry integration:
1. **Create clnrm-otel crate** - OTEL integration crate
2. **Add dependencies** - OpenTelemetry 0.31+ dependencies
3. **Implement telemetry init** - Setup tracers, meters, loggers
4. **Add feature flags** - Optional OTEL features
5. **Configure exporters** - OTLP, stdout, Jaeger
6. **Add instrumentation** - Trace test execution

## Step 1: Create OTEL Crate

```bash
# Create new crate
cd crates
cargo new clnrm-otel --lib
cd ..

# Add to workspace Cargo.toml
# Add "crates/clnrm-otel" to members array
```

## Step 2: Add Dependencies

Add to `crates/clnrm-otel/Cargo.toml`:

```toml
[package]
name = "clnrm-otel"
version = "0.4.0"
edition = "2021"

[dependencies]
# OpenTelemetry SDK 0.31 (baseline standard from kgold)
opentelemetry = { version = "0.31", default-features = false, features = ["trace", "metrics", "logs"] }
opentelemetry_sdk = { version = "0.31", features = ["rt-tokio", "metrics", "trace", "logs"] }
opentelemetry-otlp = { version = "0.31", features = ["metrics", "trace", "logs", "grpc-tonic"] }
opentelemetry-semantic-conventions = "0.31"
opentelemetry-appender-tracing = "0.31"

# Tracing integration
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
tracing-opentelemetry = "0.31"

# Async runtime
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
tonic = { version = "0.12" }

# Error handling
anyhow = "1"
thiserror = "1"

[features]
default = ["otlp-http"]
otlp-http = []
otlp-grpc = []
jaeger = []
stdout = []
```

## Step 3: Implement Telemetry Initialization

Create `crates/clnrm-otel/src/lib.rs`:

```rust
//! OpenTelemetry integration for clnrm
//!
//! Provides production-grade observability following kgold patterns.

use anyhow::Result;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    trace::{self, TracerProvider},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod config;
pub mod init;
pub mod telemetry;

pub use config::OtelConfig;
pub use init::init_otel;

/// Initialize OpenTelemetry with clnrm configuration
///
/// # Examples
///
/// ```
/// use clnrm_otel::{init_otel, OtelConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let config = OtelConfig::from_env()?;
///     let _guard = init_otel(config)?;
///
///     // Your code here
///
///     Ok(())
/// }
/// ```
pub fn setup() -> Result<()> {
    let config = OtelConfig::from_env()?;
    init_otel(config)?;
    Ok(())
}
```

Create `crates/clnrm-otel/src/config.rs`:

```rust
//! OTEL configuration following kgold environment patterns

use anyhow::Result;
use std::env;

/// OpenTelemetry configuration
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// Service name (OTEL_SERVICE_NAME)
    pub service_name: String,

    /// Service version (OTEL_SERVICE_VERSION)
    pub service_version: String,

    /// OTLP endpoint (OTEL_EXPORTER_OTLP_ENDPOINT)
    pub otlp_endpoint: String,

    /// Telemetry enabled (CLNRM_TELEMETRY_ENABLED)
    pub telemetry_enabled: bool,

    /// Export format: stdout, otlp-http, otlp-grpc
    pub exporter: ExportFormat,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Stdout,
    OtlpHttp,
    OtlpGrpc,
    Jaeger,
}

impl OtelConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            service_name: env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "clnrm".to_string()),

            service_version: env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),

            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),

            telemetry_enabled: env::var("CLNRM_TELEMETRY_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            exporter: Self::parse_exporter(&env::var("CLNRM_OTEL_EXPORTER")
                .unwrap_or_else(|_| "stdout".to_string())),
        })
    }

    fn parse_exporter(s: &str) -> ExportFormat {
        match s.to_lowercase().as_str() {
            "stdout" => ExportFormat::Stdout,
            "otlp-http" => ExportFormat::OtlpHttp,
            "otlp-grpc" => ExportFormat::OtlpGrpc,
            "jaeger" => ExportFormat::Jaeger,
            _ => ExportFormat::Stdout,
        }
    }
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "clnrm".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            telemetry_enabled: true,
            exporter: ExportFormat::Stdout,
        }
    }
}
```

Create `crates/clnrm-otel/src/init.rs`:

```rust
//! OTEL initialization following kgold patterns

use crate::config::{ExportFormat, OtelConfig};
use anyhow::Result;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{trace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize OpenTelemetry tracing, metrics, and logging
///
/// Returns a guard that should be kept alive for the duration of the program.
pub fn init_otel(config: OtelConfig) -> Result<OtelGuard> {
    if !config.telemetry_enabled {
        return Ok(OtelGuard::disabled());
    }

    // Create resource with service metadata
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", config.service_version.clone()),
    ]);

    // Create tracer provider based on exporter
    let tracer_provider = match config.exporter {
        ExportFormat::Stdout => {
            trace::TracerProvider::builder()
                .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
                .with_resource(resource)
                .build()
        }
        ExportFormat::OtlpHttp => {
            // Implement OTLP HTTP exporter
            unimplemented!("OTLP HTTP exporter - see kgold implementation")
        }
        ExportFormat::OtlpGrpc => {
            // Implement OTLP gRPC exporter
            unimplemented!("OTLP gRPC exporter - see kgold implementation")
        }
        ExportFormat::Jaeger => {
            // Implement Jaeger exporter
            unimplemented!("Jaeger exporter - see kgold implementation")
        }
    };

    // Set global tracer provider
    global::set_tracer_provider(tracer_provider.clone());

    // Initialize tracing subscriber with OTEL layer
    let telemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer("clnrm"));

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(OtelGuard {
        _tracer_provider: Some(tracer_provider),
    })
}

/// Guard that ensures OTEL is properly shut down
pub struct OtelGuard {
    _tracer_provider: Option<trace::TracerProvider>,
}

impl OtelGuard {
    fn disabled() -> Self {
        Self {
            _tracer_provider: None,
        }
    }
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(provider) = self._tracer_provider.take() {
            if let Err(e) = provider.shutdown() {
                eprintln!("Error shutting down OTEL: {}", e);
            }
        }
        global::shutdown_tracer_provider();
    }
}
```

## Step 4: Add to clnrm-core

Update `crates/clnrm-core/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...

# OpenTelemetry (optional feature)
clnrm-otel = { path = "../clnrm-otel", optional = true }

[features]
default = []
otel = ["clnrm-otel"]
```

## Step 5: Instrument Test Execution

Add to `crates/clnrm-core/src/cleanroom.rs`:

```rust
#[cfg(feature = "otel")]
use tracing::{info, span, Level};

impl CleanroomEnvironment {
    pub async fn execute_test(&self, test_config: &TestConfig) -> Result<TestResult> {
        #[cfg(feature = "otel")]
        let _span = span!(Level::INFO, "test_execution",
            test_name = %test_config.name
        ).entered();

        #[cfg(feature = "otel")]
        info!("Starting test execution: {}", test_config.name);

        // ... existing test execution logic ...

        #[cfg(feature = "otel")]
        info!("Test completed: {}", test_config.name);

        Ok(result)
    }
}
```

## Step 6: Test OTEL Integration

```bash
# Build with OTEL feature
cargo build --features otel

# Run self-test with OTEL
RUST_LOG=info cargo run --features otel -- self-test --suite otel --otel-exporter stdout

# Should output spans to stdout
```

## Step 7: Add to CI

Update `.github/workflows/ci.yml`:

```yaml
- name: Test with OTEL
  run: |
    cargo test --features otel
    cargo run --features otel -- self-test --suite otel --otel-exporter stdout
```

## Environment Variables

Following kgold patterns:

```bash
# Standard OTEL variables
export OTEL_SERVICE_NAME=clnrm
export OTEL_SERVICE_VERSION=0.4.0
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# clnrm-specific
export CLNRM_TELEMETRY_ENABLED=true
export CLNRM_OTEL_EXPORTER=stdout  # or otlp-http, otlp-grpc, jaeger

# Logging
export RUST_LOG=info
```

## When to Use
- Adding observability to clnrm
- Following kgold OTEL patterns
- Implementing production telemetry
- Debugging test execution with traces
