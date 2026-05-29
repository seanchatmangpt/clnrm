//! Poka-Yoke Trait Abstractions
//!
//! This module defines trait-based abstractions for poka-yoke mechanisms,
//! following the codebase's Chicago School TDD pattern with trait-based design.
//!
//! # Architecture Pattern
//!
//! All poka-yoke mechanisms follow the same abstraction pattern:
//! 1. **Trait Definition** - Behavioral interface (dyn-compatible, sync methods)
//! 2. **Default Implementation** - Concrete implementation for production
//! 3. **Test Implementation** - For testing (via mockall or manual test configurations)
//!
//! This enables:
//! - **Testability**: Test validators for unit tests
//! - **Extensibility**: Custom validators for specific use cases
//! - **Consistency**: Same pattern as Cache, Backend, Formatter traits

use crate::error::Result;
use std::path::Path;

/// Validator trait for CLI arguments
///
/// Validates CLI arguments before execution to prevent invalid configurations.
/// This is the abstraction layer for CLI argument validation (FM-031, RPN: 280).
///
/// # Chicago TDD Compliance
///
/// - ✅ Sync methods (dyn-compatible)
/// - ✅ Returns Result<T> (proper error handling)
/// - ✅ Mockable for testing
/// - ✅ Single responsibility (CLI validation only)
pub trait CliValidator: Send + Sync {
    /// Validate run command arguments
    ///
    /// # Errors
    ///
    /// Returns error with clear remediation if arguments are invalid
    fn validate_run_args(
        &self,
        parallel: bool,
        jobs: usize,
        watch: bool,
        fail_fast: bool,
        shard: Option<(usize, usize)>,
    ) -> Result<()>;

    /// Validate OTEL configuration arguments
    ///
    /// # Errors
    ///
    /// Returns error if OTEL configuration is invalid
    fn validate_otel_args(
        &self,
        exporter: &str,
        endpoint: Option<String>,
        validate: bool,
    ) -> Result<()>;
}

/// Validator trait for TOML configuration
///
/// Validates TOML content before parsing to catch common errors early.
/// This is the abstraction layer for TOML validation (FM-008, RPN: 180).
///
/// # Chicago TDD Compliance
///
/// - ✅ Sync methods (dyn-compatible)
/// - ✅ Returns Result<T> (proper error handling)
/// - ✅ Mockable for testing
/// - ✅ Single responsibility (TOML validation only)
pub trait TomlValidator: Send + Sync {
    /// Validate TOML content before parsing
    ///
    /// # Errors
    ///
    /// Returns error with clear remediation if TOML has common issues
    fn validate_before_parse(&self, content: &str, file_path: &Path) -> Result<()>;
}

/// Validator trait for telemetry samples
///
/// Detects zero telemetry samples early and provides clear diagnostics.
/// This is the abstraction layer for telemetry validation (FM-013, RPN: 150).
///
/// # Chicago TDD Compliance
///
/// - ✅ Sync methods (dyn-compatible)
/// - ✅ Returns Result<T> (proper error handling)
/// - ✅ Mockable for testing
/// - ✅ Single responsibility (telemetry validation only)
pub trait TelemetryValidator: Send + Sync {
    /// Validate telemetry samples before validation
    ///
    /// # Errors
    ///
    /// Returns error with clear diagnostics if zero samples detected
    fn validate_samples(
        &self,
        sample_count: usize,
        exporter: &str,
        endpoint: Option<String>,
    ) -> Result<()>;
}

/// Timeout calculator trait for adaptive timeouts
///
/// Provides adaptive timeout based on image pull status and system load.
/// This is the abstraction layer for timeout management (FM-002, RPN: 120).
///
/// # Chicago TDD Compliance
///
/// - ✅ Sync methods (dyn-compatible)
/// - ✅ Returns Duration (no errors, pure calculation)
/// - ✅ Mockable for testing
/// - ✅ Single responsibility (timeout calculation only)
pub trait TimeoutCalculator: Send + Sync {
    /// Get timeout based on whether image is cached
    ///
    /// # Arguments
    ///
    /// * `image_cached` - Whether the image is already cached locally
    /// * `system_load` - Current system load (0.0-1.0, higher = more loaded)
    fn get_timeout(&self, image_cached: bool, system_load: f64) -> std::time::Duration;
}

/// Handler trait for pool exhaustion
///
/// Provides clear error messages and backpressure when pool is exhausted.
/// This is the abstraction layer for pool exhaustion handling (FM-005, RPN: 120).
///
/// # Chicago TDD Compliance
///
/// - ✅ Sync methods (dyn-compatible)
/// - ✅ Returns Result<T> (always returns error, but follows pattern)
/// - ✅ Mockable for testing
/// - ✅ Single responsibility (pool exhaustion handling only)
pub trait PoolExhaustionHandler: Send + Sync {
    /// Handle pool exhaustion with clear error message
    ///
    /// # Errors
    ///
    /// Always returns error with actionable remediation
    fn handle_exhaustion(
        &self,
        max_size: usize,
        current_size: usize,
        pending_requests: usize,
    ) -> Result<()>;

    /// Check if pool is approaching exhaustion and warn
    ///
    /// Returns true if exhaustion risk is detected
    fn check_exhaustion_risk(&self, current: usize, max: usize, threshold: f64) -> bool;
}

/// Lock manager trait for container creation
///
/// Prevents race conditions in container creation by using locks per image.
/// This is the abstraction layer for container creation locking (FM-004, RPN: 168).
///
/// # Chicago TDD Compliance
///
/// - ⚠️ Async methods (required for lock acquisition)
/// - ✅ Returns Result<T> (proper error handling)
/// - ✅ Mockable for testing (with async-trait)
/// - ✅ Single responsibility (lock management only)
///
/// Note: This trait uses async methods because lock acquisition is inherently async.
/// For sync contexts, use `tokio::task::block_in_place` to call async methods.
#[async_trait::async_trait]
pub trait ContainerCreationLock: Send + Sync {
    /// Acquire lock for container creation
    ///
    /// Waits until the lock is available, then returns. The lock is held
    /// internally and will be released when this function returns.
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails
    async fn acquire(&self, image: &str) -> Result<()>;
}

// Note: Mock implementations can be added in tests.rs using mockall if needed
// The traits are designed to be mockable for testing purposes
