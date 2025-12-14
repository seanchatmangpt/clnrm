//! Global poka-yoke validator instances
//!
//! This module provides shared, global instances of poka-yoke validators
//! that can be used throughout the codebase. These are the default
//! implementations used in production.

use crate::poka_yoke::impls::*;
use crate::poka_yoke::traits::*;
use once_cell::sync::Lazy;

/// Global CLI validator instance
///
/// This is the default validator used for CLI argument validation.
/// It can be replaced with a custom validator for testing or special cases.
pub static CLI_VALIDATOR: Lazy<DefaultCliValidator> = Lazy::new(DefaultCliValidator::default);

/// Global TOML validator instance
///
/// This is the default validator used for TOML configuration validation.
pub static TOML_VALIDATOR: Lazy<DefaultTomlValidator> = Lazy::new(DefaultTomlValidator::default);

/// Global telemetry validator instance
///
/// This is the default validator used for telemetry sample validation.
pub static TELEMETRY_VALIDATOR: Lazy<DefaultTelemetryValidator> =
    Lazy::new(DefaultTelemetryValidator::default);

/// Global timeout calculator instance
///
/// This is the default calculator used for adaptive timeout calculation.
pub static TIMEOUT_CALCULATOR: Lazy<DefaultTimeoutCalculator> =
    Lazy::new(DefaultTimeoutCalculator::default);

/// Global pool exhaustion handler instance
///
/// This is the default handler used for pool exhaustion errors.
pub static POOL_EXHAUSTION_HANDLER: Lazy<DefaultPoolExhaustionHandler> =
    Lazy::new(DefaultPoolExhaustionHandler::default);

/// Global container creation lock instance
///
/// This is the default lock used for preventing concurrent container creation.
pub static CONTAINER_CREATION_LOCK: Lazy<DefaultContainerCreationLock> =
    Lazy::new(DefaultContainerCreationLock::default);

/// Convenience functions that use global validators
///
/// These functions provide a simple API for using the global validators
/// without needing to access the static instances directly.
/// Validate CLI arguments using global validator
pub fn validate_cli_args(
    parallel: bool,
    jobs: usize,
    watch: bool,
    fail_fast: bool,
    shard: Option<(usize, usize)>,
) -> crate::error::Result<()> {
    CLI_VALIDATOR.validate_run_args(parallel, jobs, watch, fail_fast, shard)
}

/// Validate OTEL arguments using global validator
pub fn validate_otel_args(
    exporter: &str,
    endpoint: Option<&str>,
    validate: bool,
) -> crate::error::Result<()> {
    CLI_VALIDATOR.validate_otel_args(exporter, endpoint.map(|s| s.to_string()), validate)
}

/// Validate TOML content using global validator
pub fn validate_toml(content: &str, file_path: &std::path::Path) -> crate::error::Result<()> {
    TOML_VALIDATOR.validate_before_parse(content, file_path)
}

/// Validate telemetry samples using global validator
pub fn validate_telemetry_samples(
    sample_count: usize,
    exporter: &str,
    endpoint: Option<&str>,
) -> crate::error::Result<()> {
    TELEMETRY_VALIDATOR.validate_samples(sample_count, exporter, endpoint.map(|s| s.to_string()))
}

/// Get adaptive timeout using global calculator
pub fn get_adaptive_timeout(image_cached: bool, system_load: f64) -> std::time::Duration {
    TIMEOUT_CALCULATOR.get_timeout(image_cached, system_load)
}

/// Handle pool exhaustion using global handler
pub fn handle_pool_exhaustion(
    max_size: usize,
    current_size: usize,
    pending_requests: usize,
) -> crate::error::Result<()> {
    POOL_EXHAUSTION_HANDLER.handle_exhaustion(max_size, current_size, pending_requests)
}

/// Acquire container creation lock using global lock
pub async fn acquire_container_creation_lock(image: &str) -> crate::error::Result<()> {
    CONTAINER_CREATION_LOCK.acquire(image).await
}
