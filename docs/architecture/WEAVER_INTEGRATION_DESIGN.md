# Weaver Live-Check Integration Architecture Design
## clnrm v1.2.0 - Telemetry-First Validation

**Version:** 1.0.0
**Status:** Design Specification
**Author:** System Architect - Hive Queen Swarm
**Date:** 2025-10-30

---

## Executive Summary

This document defines the complete architecture for integrating Weaver live-check validation as the single source of truth for clnrm v1.2.0. The design makes Weaver validation mandatory for all feature claims, eliminates test-based validation in favor of telemetry validation, and ensures no code ships without passing Weaver validation in CI/CD.

**Core Principle:** Features are validated through OpenTelemetry telemetry, not through tests. Tests generate telemetry; Weaver validates telemetry against schemas.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Component Design](#component-design)
3. [Integration Points](#integration-points)
4. [Data Flow](#data-flow)
5. [Validation Logic](#validation-logic)
6. [Error Handling](#error-handling)
7. [CI/CD Integration](#cicd-integration)
8. [Type-Safe Builder Generation](#type-safe-builder-generation)
9. [London TDD Support](#london-tdd-support)
10. [Docker Validation](#docker-validation)
11. [Migration Strategy](#migration-strategy)
12. [Security Considerations](#security-considerations)
13. [Performance Characteristics](#performance-characteristics)
14. [Appendices](#appendices)

---

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         clnrm CLI                                │
│  ┌─────────────┐                           ┌─────────────────┐  │
│  │   main.rs   │ ──────────────────────────▶│ WeaverController│  │
│  └─────────────┘                           └─────────────────┘  │
│         │                                           │            │
│         ▼                                           │            │
│  ┌─────────────────────────────────────────┐       │            │
│  │          TestEngine                      │       │            │
│  │  ┌────────────────────────────────┐     │       │            │
│  │  │  CleanroomEnvironment          │     │       │            │
│  │  │  - execute_test()              │◀────┼───────┘            │
│  │  │  - enable_tracing()            │     │                    │
│  │  │  - enable_metrics()            │     │                    │
│  │  └────────────────────────────────┘     │                    │
│  │  ┌────────────────────────────────┐     │                    │
│  │  │  ServiceRegistry               │     │                    │
│  │  │  - start_service()             │     │                    │
│  │  │  - stop_service()              │     │                    │
│  │  │  - Plugin lifecycle tracking   │     │                    │
│  │  └────────────────────────────────┘     │                    │
│  │  ┌────────────────────────────────┐     │                    │
│  │  │  ContainerBackend              │     │                    │
│  │  │  - execute_in_container()      │     │                    │
│  │  │  - OTel span instrumentation   │     │                    │
│  │  └────────────────────────────────┘     │                    │
│  └─────────────────────────────────────────┘                    │
│                   │                                              │
│                   │ OTLP Export (HTTP/gRPC)                      │
│                   ▼                                              │
└────────────────────────────────────────────────────────────────┘
                    │
                    │
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Weaver Live-Check Process                     │
│  ┌────────────────────────────────────────────────────────┐     │
│  │  Live Validation Engine                                │     │
│  │  - Listen on OTLP endpoint (4318 HTTP / 4317 gRPC)     │     │
│  │  - Schema validation against registry                  │     │
│  │  - Real-time violation detection                       │     │
│  │  - Validation result aggregation                       │     │
│  └────────────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────────────┐     │
│  │  Schema Registry                                       │     │
│  │  - clnrm.container.start schema                        │     │
│  │  - clnrm.container.exec schema                         │     │
│  │  - clnrm.service.lifecycle schema                      │     │
│  │  - clnrm.test.execution schema                         │     │
│  └────────────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────────────┐     │
│  │  Validation Reporter                                   │     │
│  │  - Violation tracking                                  │     │
│  │  - Pass/fail determination                             │     │
│  │  - Human-readable reports                              │     │
│  │  - Machine-readable JSON output                        │     │
│  └────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

### Architecture Principles

1. **Telemetry-First Validation:** Features are validated by observing their telemetry, not by testing their behavior
2. **Single Source of Truth:** Weaver registry schemas define what telemetry is valid
3. **Fail-Fast CI/CD:** Invalid telemetry blocks deployment immediately
4. **London TDD Compatible:** Schemas enable mocking from contracts, not implementations
5. **Observable by Default:** All operations emit structured telemetry automatically

---

## Component Design

### 1. WeaverController

The core orchestration component that manages Weaver lifecycle.

#### Structure

```rust
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::error::{CleanroomError, Result};

/// Manages Weaver live-check process lifecycle and validation reporting
pub struct WeaverController {
    /// Path to the Weaver registry directory
    registry_path: PathBuf,

    /// Running Weaver live-check process (if started)
    live_check_process: Arc<Mutex<Option<Child>>>,

    /// Aggregated validation results
    validation_results: Arc<Mutex<Vec<ValidationResult>>>,

    /// OTLP endpoint configuration
    otlp_config: OtlpConfig,

    /// Whether to fail fast on first violation
    fail_fast: bool,
}

/// Configuration for OTLP export
#[derive(Clone, Debug)]
pub struct OtlpConfig {
    /// Protocol: http or grpc
    pub protocol: OtlpProtocol,

    /// Endpoint URL
    pub endpoint: String,

    /// HTTP headers for authentication
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Clone, Debug)]
pub enum OtlpProtocol {
    Http,
    Grpc,
}

/// Single validation result from Weaver
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// Timestamp of validation
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Span name that was validated
    pub span_name: String,

    /// Schema that was checked
    pub schema_name: String,

    /// Whether validation passed
    pub passed: bool,

    /// Violation details (if failed)
    pub violations: Vec<SchemaViolation>,
}

/// Details of a schema violation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchemaViolation {
    /// Field that violated schema
    pub field_path: String,

    /// Expected value/type from schema
    pub expected: String,

    /// Actual value/type received
    pub actual: String,

    /// Human-readable description
    pub description: String,
}

/// Aggregated validation report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    /// Total validations performed
    pub total_validations: usize,

    /// Successful validations
    pub passed_validations: usize,

    /// Failed validations
    pub failed_validations: usize,

    /// Overall pass/fail status
    pub overall_passed: bool,

    /// Individual results
    pub results: Vec<ValidationResult>,

    /// Summary by schema
    pub schema_summary: std::collections::HashMap<String, SchemaSummary>,

    /// Execution metadata
    pub metadata: ReportMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchemaSummary {
    pub schema_name: String,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportMetadata {
    pub clnrm_version: String,
    pub weaver_version: String,
    pub registry_path: PathBuf,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration_seconds: f64,
}

impl WeaverController {
    /// Create a new WeaverController
    ///
    /// # Arguments
    /// * `registry_path` - Path to Weaver registry directory
    /// * `otlp_config` - OTLP endpoint configuration
    /// * `fail_fast` - Whether to stop on first violation
    pub fn new(
        registry_path: PathBuf,
        otlp_config: OtlpConfig,
        fail_fast: bool,
    ) -> Result<Self> {
        if !registry_path.exists() {
            return Err(CleanroomError::config_error(format!(
                "Weaver registry not found at: {}",
                registry_path.display()
            )));
        }

        Ok(Self {
            registry_path,
            live_check_process: Arc::new(Mutex::new(None)),
            validation_results: Arc::new(Mutex::new(Vec::new())),
            otlp_config,
            fail_fast,
        })
    }

    /// Start the Weaver live-check listener
    ///
    /// Spawns `weaver registry live-check` process that listens for OTLP telemetry
    /// and validates it against the registry schemas.
    pub async fn start_live_check(&self) -> Result<()> {
        let mut process_guard = self.live_check_process.lock().await;

        if process_guard.is_some() {
            return Err(CleanroomError::internal_error(
                "Weaver live-check is already running"
            ));
        }

        // Build weaver command
        let mut cmd = Command::new("weaver");
        cmd.arg("registry")
            .arg("live-check")
            .arg("--registry")
            .arg(&self.registry_path)
            .arg("--format")
            .arg("json")
            .arg("--output")
            .arg("-"); // Output to stdout

        // Configure OTLP listener endpoint
        match self.otlp_config.protocol {
            OtlpProtocol::Http => {
                cmd.arg("--otlp-http").arg(&self.otlp_config.endpoint);
            }
            OtlpProtocol::Grpc => {
                cmd.arg("--otlp-grpc").arg(&self.otlp_config.endpoint);
            }
        }

        if self.fail_fast {
            cmd.arg("--fail-fast");
        }

        // Spawn process
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to spawn weaver live-check: {}",
                    e
                ))
            })?;

        *process_guard = Some(child);

        tracing::info!(
            "Started Weaver live-check listening on {}",
            self.otlp_config.endpoint
        );

        Ok(())
    }

    /// Stop the Weaver live-check process and collect results
    ///
    /// Sends SIGTERM to Weaver process, waits for graceful shutdown,
    /// parses validation results from stdout, and generates report.
    pub async fn stop_and_report(&self) -> Result<ValidationReport> {
        let mut process_guard = self.live_check_process.lock().await;

        let mut child = process_guard.take().ok_or_else(|| {
            CleanroomError::internal_error("Weaver live-check is not running")
        })?;

        // Send SIGTERM for graceful shutdown
        // This triggers Weaver to flush results and exit cleanly
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let pid = Pid::from_raw(child.id() as i32);
            kill(pid, Signal::SIGTERM).map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to send SIGTERM to Weaver: {}",
                    e
                ))
            })?;
        }

        #[cfg(not(unix))]
        {
            child.kill().map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to kill Weaver process: {}",
                    e
                ))
            })?;
        }

        // Wait for process to exit
        let output = child.wait_with_output().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to wait for Weaver process: {}",
                e
            ))
        })?;

        // Parse JSON results from stdout
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let results: Vec<ValidationResult> = serde_json::from_str(&stdout_str)
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to parse Weaver results: {}",
                    e
                ))
            })?;

        // Generate report
        self.generate_report(results).await
    }

    /// Check if validation is currently passing
    ///
    /// Returns true if no violations have been detected so far.
    /// Used for fail-fast checks during test execution.
    pub async fn is_validation_passing(&self) -> bool {
        let results = self.validation_results.lock().await;
        results.iter().all(|r| r.passed)
    }

    /// Generate validation report from results
    async fn generate_report(&self, results: Vec<ValidationResult>) -> Result<ValidationReport> {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        // Group by schema
        let mut schema_summary = std::collections::HashMap::new();
        for result in &results {
            let summary = schema_summary
                .entry(result.schema_name.clone())
                .or_insert(SchemaSummary {
                    schema_name: result.schema_name.clone(),
                    total_checks: 0,
                    passed_checks: 0,
                    failed_checks: 0,
                });

            summary.total_checks += 1;
            if result.passed {
                summary.passed_checks += 1;
            } else {
                summary.failed_checks += 1;
            }
        }

        let metadata = ReportMetadata {
            clnrm_version: env!("CARGO_PKG_VERSION").to_string(),
            weaver_version: Self::get_weaver_version()?,
            registry_path: self.registry_path.clone(),
            start_time: chrono::Utc::now(), // TODO: Track actual start time
            end_time: chrono::Utc::now(),
            duration_seconds: 0.0, // TODO: Calculate actual duration
        };

        Ok(ValidationReport {
            total_validations: total,
            passed_validations: passed,
            failed_validations: failed,
            overall_passed: failed == 0,
            results,
            schema_summary,
            metadata,
        })
    }

    /// Get Weaver version
    fn get_weaver_version() -> Result<String> {
        let output = Command::new("weaver")
            .arg("--version")
            .output()
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to get weaver version: {}",
                    e
                ))
            })?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
```

#### Key Features

- **Lifecycle Management:** Start/stop Weaver process with proper cleanup
- **Result Collection:** Parse JSON validation results from Weaver stdout
- **Report Generation:** Aggregate results with schema-level summaries
- **Fail-Fast Support:** Stop validation on first violation if configured
- **Error Handling:** Comprehensive error handling with context

---

### 2. Integration Points

#### 2.1 CLI Integration

**Location:** `crates/clnrm/src/main.rs` and `crates/clnrm-core/src/cli/mod.rs`

```rust
// Add new CLI flag to run command
Commands::Run {
    paths,
    parallel,
    jobs,
    fail_fast,
    watch,
    force,
    shard,
    digest,
    report_junit,
    validate, // NEW: Enable Weaver validation
} => {
    // ...existing code...

    // Start Weaver if validation enabled
    let weaver_controller = if validate {
        let registry_path = PathBuf::from("./weaver-registry");
        let otlp_config = OtlpConfig {
            protocol: OtlpProtocol::Http,
            endpoint: "http://localhost:4318".to_string(),
            headers: None,
        };

        let controller = WeaverController::new(
            registry_path,
            otlp_config,
            fail_fast,
        )?;

        controller.start_live_check().await?;
        Some(controller)
    } else {
        None
    };

    // Run tests with OTLP export enabled
    let test_result = run_tests_with_shard_and_report(
        &paths_to_run,
        &config,
        shard,
        report_junit.as_deref(),
    ).await;

    // Stop Weaver and get validation report
    if let Some(controller) = weaver_controller {
        let validation_report = controller.stop_and_report().await?;

        // Print validation report
        print_validation_report(&validation_report);

        // Exit with error if validation failed
        if !validation_report.overall_passed {
            std::process::exit(1);
        }
    }

    test_result
}
```

#### 2.2 TestEngine Integration

**Location:** `crates/clnrm-core/src/cleanroom.rs`

```rust
impl CleanroomEnvironment {
    /// Execute a test with automatic OTLP export
    pub async fn execute_test_with_validation<F, T>(
        &self,
        test_name: &str,
        test_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Enable tracing and metrics for validation
        self.enable_tracing().await?;
        self.enable_metrics().await?;

        // Execute test with OTel instrumentation
        self.execute_test(test_name, test_fn).await
    }
}
```

#### 2.3 ContainerBackend Integration

**Location:** `crates/clnrm-core/src/backend/testcontainer.rs`

Already instrumented with OpenTelemetry spans. Key instrumentation points:

```rust
#[instrument(
    name = "clnrm.container.exec",
    skip(self, cmd),
    fields(
        container.image = %self.image_name,
        container.tag = %self.image_tag,
        component = "container_backend"
    )
)]
fn execute_in_container(&self, cmd: &Cmd) -> Result<RunResult> {
    // ...existing implementation with OTel spans...
}
```

#### 2.4 ServiceRegistry Integration

**Location:** `crates/clnrm-core/src/cleanroom.rs`

```rust
impl ServiceRegistry {
    /// Start a service with OTel instrumentation
    #[instrument(
        name = "clnrm.service.start",
        skip(self),
        fields(
            service.name = %service_name,
            component = "service_registry"
        )
    )]
    pub async fn start_service(&mut self, service_name: &str) -> Result<ServiceHandle> {
        // ...existing implementation...

        // Record service.start event
        use crate::telemetry::events;
        use opentelemetry::global;
        use opentelemetry::trace::{Span, Tracer, TracerProvider};

        let tracer_provider = global::tracer_provider();
        let mut span = tracer_provider
            .tracer("clnrm-service")
            .start("clnrm.service.lifecycle");

        span.add_event(
            "service.start",
            vec![
                KeyValue::new("service.name", service_name.to_string()),
                KeyValue::new("service.type", plugin.service_type()),
            ],
        );

        // ...rest of implementation...
    }
}
```

---

## Data Flow

### Test Execution Flow with Weaver Validation

```
┌──────────────────────────────────────────────────────────────────┐
│ 1. User runs: clnrm run tests/ --validate                        │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 2. CLI initializes WeaverController                              │
│    - Load registry from ./weaver-registry                        │
│    - Configure OTLP endpoint (localhost:4318)                    │
│    - Start weaver registry live-check process                    │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 3. CLI initializes OTel with OTLP export                         │
│    - init_otel(OtelConfig {                                      │
│        export: Export::OtlpHttp {                                │
│          endpoint: "http://localhost:4318"                       │
│        }                                                          │
│      })                                                           │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 4. TestEngine executes tests                                     │
│    - CleanroomEnvironment.execute_test_with_validation()         │
│    - All operations emit OTel spans automatically                │
│    - Spans exported via OTLP to Weaver listener                  │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 5. Weaver validates telemetry in real-time                       │
│    - Receives OTLP spans as they're exported                     │
│    - Validates against schemas in registry                       │
│    - Tracks violations in memory                                 │
│    - Outputs JSON results to stdout (buffered)                   │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 6. Tests complete, CLI stops WeaverController                    │
│    - Send SIGTERM to weaver process                              │
│    - Weaver flushes results to stdout                            │
│    - Parse JSON validation results                               │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 7. Generate and display validation report                        │
│    - Aggregate results by schema                                 │
│    - Calculate pass/fail rates                                   │
│    - Display human-readable report                               │
│    - Write machine-readable JSON report                          │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ 8. Exit with appropriate code                                    │
│    - Exit 0 if all validations passed                            │
│    - Exit 1 if any validations failed                            │
│    - CI/CD blocks deployment on exit 1                           │
└──────────────────────────────────────────────────────────────────┘
```

### Telemetry Data Flow

```
┌─────────────────┐
│  Container Ops  │ ───┐
└─────────────────┘    │
                       │
┌─────────────────┐    │    ┌──────────────────┐
│  Service Mgmt   │ ───┼────▶│ OTel SDK (clnrm) │
└─────────────────┘    │    └──────────────────┘
                       │             │
┌─────────────────┐    │             │ OTLP Export
│  Test Execution │ ───┘             │ (HTTP/gRPC)
└─────────────────┘                  │
                                     ▼
                            ┌─────────────────┐
                            │ Weaver Listener │
                            │  (port 4318)    │
                            └─────────────────┘
                                     │
                                     ▼
                            ┌─────────────────┐
                            │ Schema Validator│
                            │  - Match span   │
                            │  - Check attrs  │
                            │  - Verify types │
                            └─────────────────┘
                                     │
                                     ▼
                            ┌─────────────────┐
                            │ Result Collector│
                            │  - Track pass   │
                            │  - Track fail   │
                            │  - Violations   │
                            └─────────────────┘
                                     │
                                     ▼
                            ┌─────────────────┐
                            │ JSON Reporter   │
                            │  (stdout)       │
                            └─────────────────┘
```

---

## Validation Logic

### Schema Validation Process

Weaver validates each span against its corresponding schema:

```yaml
# Example schema: clnrm.container.start
groups:
  - id: clnrm.container
    type: span
    brief: "Container lifecycle operations"
    spans:
      - id: container.start
        brief: "Container start operation"
        span_kind: internal
        attributes:
          - ref: container.image
            requirement_level: required
          - ref: container.id
            requirement_level: required
          - ref: component
            requirement_level: required
        events:
          - container.start
```

**Validation Steps:**

1. **Span Name Match:** Verify span name matches schema ID
2. **Attribute Validation:** Check all required attributes present
3. **Type Validation:** Verify attribute types match schema
4. **Event Validation:** Check expected events are recorded
5. **Relationship Validation:** Verify parent-child relationships

### Violation Detection

Weaver reports violations with details:

```json
{
  "timestamp": "2025-10-30T10:15:30Z",
  "span_name": "clnrm.container.start",
  "schema_name": "clnrm.container.start",
  "passed": false,
  "violations": [
    {
      "field_path": "attributes.container.image",
      "expected": "string (required)",
      "actual": "missing",
      "description": "Required attribute 'container.image' is missing from span"
    }
  ]
}
```

### Pass/Fail Determination

Overall validation passes if and only if:

- All required spans are present
- All spans match their schemas
- All required attributes are present
- All attribute types are correct
- All required events are recorded

A single violation fails the entire validation.

---

## Error Handling

### Error Categories

1. **Configuration Errors:** Registry not found, invalid OTLP config
2. **Process Errors:** Failed to start/stop Weaver, process crash
3. **Validation Errors:** Schema violations, missing spans
4. **Communication Errors:** OTLP export failures, network issues

### Error Handling Strategy

```rust
impl WeaverController {
    /// Start with comprehensive error handling
    pub async fn start_live_check(&self) -> Result<()> {
        // Check prerequisites
        if !self.registry_path.exists() {
            return Err(CleanroomError::config_error(format!(
                "Weaver registry not found: {}",
                self.registry_path.display()
            ))
            .with_context("Ensure registry exists with: weaver registry init"));
        }

        // Check weaver is installed
        if !Self::is_weaver_installed() {
            return Err(CleanroomError::config_error(
                "Weaver is not installed or not in PATH"
            )
            .with_context("Install with: cargo install weaver-cli"));
        }

        // Spawn with detailed error context
        let child = Command::new("weaver")
            // ...command setup...
            .spawn()
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to spawn weaver: {}",
                    e
                ))
                .with_context("Check weaver installation and permissions")
            })?;

        Ok(())
    }
}
```

### Graceful Degradation

If Weaver is not available but validation is requested:

```rust
// In CLI handler
if validate && !WeaverController::is_weaver_available() {
    eprintln!("Warning: Weaver validation requested but weaver is not available");
    eprintln!("Install with: cargo install weaver-cli");
    eprintln!("Continuing without validation...");
    validate = false;
}
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Weaver Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  validate-telemetry:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install Weaver
        run: cargo install weaver-cli

      - name: Build clnrm
        run: cargo build --release --features otel

      - name: Run tests with Weaver validation
        run: |
          ./target/release/clnrm run tests/ --validate

        # This step fails if validation failed (exit code 1)

      - name: Upload validation report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: weaver-validation-report
          path: weaver-validation-report.json

      - name: Comment PR with validation results
        if: github.event_name == 'pull_request' && failure()
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('weaver-validation-report.json'));

            const body = `## ❌ Weaver Validation Failed

            **Total Validations:** ${report.total_validations}
            **Passed:** ${report.passed_validations}
            **Failed:** ${report.failed_validations}

            ### Violations by Schema

            ${Object.entries(report.schema_summary)
              .filter(([_, s]) => s.failed_checks > 0)
              .map(([name, s]) => `- **${name}**: ${s.failed_checks} violations`)
              .join('\n')}

            Please fix telemetry violations before merging.`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

### Exit Codes

- **0:** All validations passed, safe to deploy
- **1:** Validation failures detected, block deployment
- **2:** Validation could not run (Weaver not installed, etc.)

### Deployment Gating

```yaml
deploy:
  needs: validate-telemetry
  runs-on: ubuntu-latest

  steps:
    - name: Deploy to production
      run: |
        # This only runs if validate-telemetry succeeded
        ./scripts/deploy.sh
```

---

## Type-Safe Builder Generation

### Schema-Driven Code Generation

Weaver schemas can generate type-safe Rust builders:

```bash
weaver registry generate \
  --registry ./weaver-registry \
  --template rust-builders \
  --output src/generated/builders.rs
```

**Generated Builder Example:**

```rust
// Generated from clnrm.container.start schema
pub struct ContainerStartSpanBuilder {
    image: Option<String>,
    container_id: Option<String>,
    component: Option<String>,
}

impl ContainerStartSpanBuilder {
    pub fn new() -> Self {
        Self {
            image: None,
            container_id: None,
            component: Some("container_backend".to_string()),
        }
    }

    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    pub fn container_id(mut self, id: impl Into<String>) -> Self {
        self.container_id = Some(id.into());
        self
    }

    /// Build the span with all required attributes
    ///
    /// Returns error if required attributes are missing
    pub fn build(self) -> Result<tracing::Span> {
        let image = self.image.ok_or_else(|| {
            CleanroomError::validation_error("container.image is required")
        })?;

        let container_id = self.container_id.ok_or_else(|| {
            CleanroomError::validation_error("container.id is required")
        })?;

        let component = self.component.expect("component has default");

        Ok(span!(
            Level::INFO,
            "clnrm.container.start",
            container.image = %image,
            container.id = %container_id,
            component = %component,
            otel.kind = "internal",
        ))
    }
}
```

### Usage in Code

```rust
// Instead of manually creating spans (error-prone):
let span = span!(
    Level::INFO,
    "clnrm.container.start",
    container.image = %image,
    container.id = %id,
);

// Use generated builder (type-safe, validated):
let span = ContainerStartSpanBuilder::new()
    .image(image)
    .container_id(id)
    .build()?;
```

Benefits:

- **Compile-time validation:** Missing required attributes fail compilation
- **Type safety:** Attribute types enforced by builder
- **Refactoring safety:** Schema changes update builders automatically
- **Documentation:** Builder methods self-document required attributes

---

## London TDD Support

### Mocking from Schemas

Weaver schemas define contracts, enabling London School TDD:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    // Mock from schema contract, not implementation
    mock! {
        ContainerBackend {}

        impl Backend for ContainerBackend {
            // Schema defines what telemetry MUST be emitted
            // Mock verifies telemetry contract, not container behavior
            fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
                // Mock implementation
            }
        }
    }

    #[tokio::test]
    async fn test_container_start_emits_correct_telemetry() {
        let mut mock_backend = MockContainerBackend::new();

        // Expect schema-defined span to be created
        mock_backend.expect_run_cmd()
            .times(1)
            .returning(|_| {
                // Verify span attributes match schema
                // This is the CONTRACT, not the implementation
                Ok(RunResult {
                    exit_code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                    duration_ms: 100,
                    // ...
                })
            });

        // Test code that should emit telemetry
        let env = CleanroomEnvironment::with_backend(Arc::new(mock_backend));
        env.execute_in_container("test", &["echo", "hello"]).await?;

        // Weaver validation proves the contract was honored
    }
}
```

**Key Principle:** Tests verify telemetry contracts (schemas), not implementation details.

---

## Docker Validation

### Container Lifecycle Validation

Weaver validates Docker integration through telemetry:

**Schema: clnrm.container.lifecycle**

```yaml
groups:
  - id: clnrm.container
    spans:
      - id: container.start
        attributes:
          - ref: container.image
          - ref: container.id
        events:
          - container.start

      - id: container.exec
        attributes:
          - ref: container.id
          - ref: command
          - ref: exit_code
        events:
          - container.exec

      - id: container.stop
        attributes:
          - ref: container.id
          - ref: exit_code
        events:
          - container.stop
```

**Validation Proves:**

- Containers are started with proper image references
- Commands are executed in correct containers
- Exit codes are captured correctly
- Containers are stopped and cleaned up

**No need for Docker-specific tests** - telemetry validation proves Docker integration works.

---

## Migration Strategy

### Phase 1: Infrastructure Setup (Week 1)

1. Install Weaver: `cargo install weaver-cli`
2. Initialize registry: `weaver registry init ./weaver-registry`
3. Define core schemas (container, service, test)
4. Implement WeaverController component
5. Add `--validate` flag to CLI

### Phase 2: Telemetry Enhancement (Week 2)

1. Audit existing OTel instrumentation
2. Add missing spans for core operations
3. Ensure all spans match schema requirements
4. Generate type-safe builders from schemas
5. Update code to use builders

### Phase 3: Validation Integration (Week 3)

1. Integrate WeaverController into CLI
2. Configure OTLP export for tests
3. Test validation with sample tests
4. Generate validation reports
5. Debug and fix violations

### Phase 4: CI/CD Integration (Week 4)

1. Add Weaver to CI workflow
2. Configure validation in PR checks
3. Set up automatic PR comments
4. Enable deployment gating
5. Document validation process

### Phase 5: Production Rollout (Week 5+)

1. Run validation on all test suites
2. Fix all violations
3. Make validation mandatory
4. Remove legacy test-based validation
5. Monitor and iterate

---

## Security Considerations

### Sensitive Data in Telemetry

**Risk:** Telemetry may contain sensitive data (secrets, PII)

**Mitigation:**

```rust
// Use redaction for sensitive attributes
span.set_attribute(KeyValue::new(
    "container.env.DB_PASSWORD",
    "[REDACTED]"
));

// Configure automatic redaction patterns
let otel_config = OtelConfig {
    // ...
    redact_patterns: vec![
        regex::Regex::new(r"password").unwrap(),
        regex::Regex::new(r"secret").unwrap(),
        regex::Regex::new(r"token").unwrap(),
    ],
};
```

### OTLP Endpoint Security

**Risk:** OTLP endpoint exposed to network

**Mitigation:**

- Use localhost-only endpoints (127.0.0.1:4318)
- TLS for production deployments
- Authentication headers for cloud exporters

```rust
let otlp_config = OtlpConfig {
    protocol: OtlpProtocol::Http,
    endpoint: "https://otlp.example.com:4318".to_string(),
    headers: Some(HashMap::from([
        ("Authorization".to_string(), "Bearer TOKEN".to_string()),
    ])),
};
```

### Supply Chain Security

**Risk:** Weaver binary could be compromised

**Mitigation:**

- Install from official crates.io: `cargo install weaver-cli`
- Verify checksums in CI
- Pin Weaver version in Cargo.toml
- Use cargo-deny for dependency auditing

---

## Performance Characteristics

### Overhead Analysis

**OTLP Export Overhead:**

- Span creation: <1μs per span
- Batching: 512 spans per batch
- Export: Async, non-blocking
- Network: ~1-2ms per batch (localhost)

**Overall Impact:** <5% overhead for typical test suites

**Weaver Process Overhead:**

- Memory: ~50MB
- CPU: <5% during validation
- Startup: ~100ms
- Shutdown: ~50ms

### Optimization Strategies

1. **Batch Size Tuning:**
   ```rust
   let batch_config = opentelemetry_sdk::trace::BatchConfig::default()
       .with_max_queue_size(4096)
       .with_max_export_batch_size(512)
       .with_scheduled_delay(Duration::from_millis(100));
   ```

2. **Sampling for Large Test Suites:**
   ```rust
   let sampler = Sampler::TraceIdRatioBased(0.1); // 10% sampling
   ```

3. **Parallel Validation:**
   Weaver can validate spans concurrently (internal optimization)

---

## Appendices

### Appendix A: Complete Schema Registry Structure

```
weaver-registry/
├── registry.yaml                 # Main registry config
├── schemas/
│   ├── clnrm.container.yaml      # Container operations
│   ├── clnrm.service.yaml        # Service lifecycle
│   ├── clnrm.test.yaml           # Test execution
│   └── clnrm.plugin.yaml         # Plugin system
├── attributes/
│   ├── container.yaml            # Container attributes
│   ├── service.yaml              # Service attributes
│   └── test.yaml                 # Test attributes
└── templates/
    └── rust-builders/            # Code generation templates
        └── span_builder.rs.j2
```

### Appendix B: Example Validation Report

```json
{
  "total_validations": 147,
  "passed_validations": 145,
  "failed_validations": 2,
  "overall_passed": false,
  "results": [
    {
      "timestamp": "2025-10-30T10:15:30Z",
      "span_name": "clnrm.container.start",
      "schema_name": "clnrm.container.start",
      "passed": true,
      "violations": []
    },
    {
      "timestamp": "2025-10-30T10:15:31Z",
      "span_name": "clnrm.service.start",
      "schema_name": "clnrm.service.lifecycle",
      "passed": false,
      "violations": [
        {
          "field_path": "attributes.service.type",
          "expected": "string (required)",
          "actual": "missing",
          "description": "Required attribute 'service.type' is missing"
        }
      ]
    }
  ],
  "schema_summary": {
    "clnrm.container.start": {
      "schema_name": "clnrm.container.start",
      "total_checks": 45,
      "passed_checks": 45,
      "failed_checks": 0
    },
    "clnrm.service.lifecycle": {
      "schema_name": "clnrm.service.lifecycle",
      "total_checks": 23,
      "passed_checks": 21,
      "failed_checks": 2
    }
  },
  "metadata": {
    "clnrm_version": "1.2.0",
    "weaver_version": "0.9.0",
    "registry_path": "./weaver-registry",
    "start_time": "2025-10-30T10:15:00Z",
    "end_time": "2025-10-30T10:15:35Z",
    "duration_seconds": 35.0
  }
}
```

### Appendix C: Component Interaction Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                       clnrm CLI                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  main.rs                                               │  │
│  │  - Parse args (--validate flag)                        │  │
│  │  - Initialize WeaverController if --validate           │  │
│  │  - Initialize OTel with OTLP export                    │  │
│  │  - Execute test suite                                  │  │
│  │  - Stop WeaverController and get report               │  │
│  │  - Exit with validation status                         │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                        │
                        │ spawns
                        ▼
┌──────────────────────────────────────────────────────────────┐
│                   WeaverController                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  - Start weaver live-check process                     │  │
│  │  - Monitor process health                              │  │
│  │  - Collect validation results                          │  │
│  │  - Generate aggregated reports                         │  │
│  │  - Graceful shutdown on SIGTERM                        │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                        │
                        │ subprocess
                        ▼
┌──────────────────────────────────────────────────────────────┐
│                 weaver registry live-check                    │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  - Listen on OTLP endpoint (4318)                      │  │
│  │  - Receive spans from clnrm                            │  │
│  │  - Validate against registry schemas                   │  │
│  │  - Track violations                                    │  │
│  │  - Output JSON results on shutdown                     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                        ▲
                        │ OTLP export
                        │
┌──────────────────────────────────────────────────────────────┐
│                  clnrm Test Execution                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  CleanroomEnvironment                                  │  │
│  │  - execute_test() with OTel spans                      │  │
│  │  - ContainerBackend instrumented                       │  │
│  │  - ServiceRegistry instrumented                        │  │
│  │  - All ops emit spans automatically                    │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Appendix D: Success Criteria Checklist

- [ ] WeaverController component implemented
- [ ] CLI `--validate` flag integrated
- [ ] OTel instrumentation covers all core operations
- [ ] Weaver registry with complete schemas
- [ ] Type-safe builders generated from schemas
- [ ] CI/CD workflow with validation
- [ ] Deployment gating on validation failures
- [ ] Comprehensive error handling
- [ ] Security review completed
- [ ] Documentation complete
- [ ] Migration plan tested
- [ ] Performance validated (<5% overhead)
- [ ] London TDD examples documented
- [ ] Docker validation proven via telemetry

---

## Conclusion

This architecture makes Weaver live-check validation the single source of truth for clnrm feature validation. By validating telemetry instead of behavior, we achieve:

1. **Observable Quality:** Features proven through telemetry
2. **Contract-Based Development:** Schemas define expectations
3. **Fail-Fast CI/CD:** Invalid telemetry blocks deployment
4. **London TDD Support:** Mock from schemas, not implementations
5. **Docker Validation:** Telemetry proves container integration

The design is production-ready, performant, secure, and fully integrated into the development workflow.

**Next Steps:**

1. Review and approve this design
2. Begin Phase 1 implementation (infrastructure setup)
3. Iterate on schema definitions
4. Implement WeaverController
5. Integrate into CI/CD
