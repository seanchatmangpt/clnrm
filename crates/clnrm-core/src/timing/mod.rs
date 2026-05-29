//! μ-Kernel Timing & τ Validation (Phase 4)
//!
//! End-to-end timing validation framework that bridges OTEL observability
//! with μ-kernel timing guarantees.
//!
//! ## Overview
//!
//! The timing framework validates execution timing across multiple layers:
//!
//! - **Latency Bands**: Hot (sub-ms), Warm (ms-range), Cold (seconds-range)
//! - **OTEL Spans**: Observability data from OpenTelemetry instrumentation
//! - **μ-Kernel Receipts**: Low-level timing proofs from μ-kernel execution
//! - **Cross-Validation**: Ensure OTEL and μ-kernel measurements agree
//!
//! ## Timing Model
//!
//! The framework defines three latency bands:
//!
//! ### Hot Path (τ ≤ 8)
//!
//! Sub-millisecond operations with tight timing guarantees:
//! - Fast path lookups
//! - In-memory operations
//! - Critical section execution
//! - Target: microsecond-level precision
//!
//! ### Warm Path
//!
//! Millisecond-range orchestration:
//! - RPC calls
//! - Database queries
//! - Service-to-service communication
//! - Target: no human-perceivable latency
//!
//! ### Cold Path
//!
//! Seconds-range provisioning:
//! - Container startup
//! - Resource allocation
//! - Network provisioning
//! - Target: bounded but user expects delay
//!
//! ## Usage
//!
//! ```rust,no_run
//! use clnrm_core::timing::{TimingValidator, OtelSpan};
//! use clnrm_core::capabilities::LatencyBand;
//! use std::time::Duration;
//! use std::collections::HashMap;
//!
//! # fn example() -> clnrm_core::error::Result<()> {
//! // Create validator with timing constraints
//! let mut validator = TimingValidator::new();
//! validator.add_constraint("fast_lookup", LatencyBand::Hot {
//!     max_duration: Duration::from_micros(500),
//! });
//! validator.add_constraint("db_query", LatencyBand::Warm {
//!     max_ms: 100,
//! });
//!
//! // Validate OTEL spans
//! let spans = vec![
//!     OtelSpan {
//!         name: "fast_lookup".to_string(),
//!         span_id: "span1".to_string(),
//!         trace_id: "trace1".to_string(),
//!         duration: Duration::from_micros(300),
//!         start_time_nanos: 1000,
//!         end_time_nanos: 301000,
//!         attributes: HashMap::new(),
//!     }
//! ];
//!
//! let footprint = validator.validate_spans(&spans, None)?;
//!
//! // Check for violations
//! validator.validate_no_violations(&footprint)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## μ-Kernel Integration
//!
//! When μ-kernel timing receipts are available, the validator performs
//! cross-layer validation to ensure observability matches low-level execution:
//!
//! ```rust,no_run
//! use clnrm_core::timing::{TimingValidator, OtelSpan, MuKernelReceipt};
//! # use std::time::Duration;
//! # use std::collections::HashMap;
//! # fn example() -> clnrm_core::error::Result<()> {
//! # let validator = TimingValidator::new();
//! # let spans = vec![];
//!
//! // μ-kernel receipts from instrumented services
//! let mu_receipts = vec![
//!     MuKernelReceipt {
//!         operation_id: "fast_lookup".to_string(),
//!         cycles: 100,
//!         timestamp_nanos: 300000,
//!         tau_expected: Some(200),
//!         metadata: HashMap::new(),
//!     }
//! ];
//!
//! // Cross-validate OTEL spans with μ-kernel receipts
//! let footprint = validator.validate_spans(&spans, Some(&mu_receipts))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Future Work
//!
//! The μ-kernel receipt format is currently a EXAMPLE-ONLY: placeholder. Once the μ-kernel
//! specification is finalized, this module will be updated to:
//!
//! - Parse actual μ-kernel receipt format
//! - Map μ-kernel operation IDs to OTEL span names
//! - Extract timing receipts from containers under test
//! - Define precise τ units (CPU cycles, μ-kernel cycles, etc.)

pub mod validator;

// Re-export commonly used types
pub use validator::{MuKernelReceipt, OtelSpan, TimingValidator};
