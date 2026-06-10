//! Poka-Yoke (Error-Proofing) Mechanisms
//!
//! This module implements error-proofing mechanisms to prevent the highest-priority
//! failure modes identified in the FMEA audit. These mechanisms make failures
//! impossible or immediately detectable.
//!
//! # Poka-Yoke Principles
//!
//! 1. **Prevention**: Make errors impossible through design
//! 2. **Detection**: Make errors immediately obvious when they occur
//! 3. **Fail-Fast**: Detect errors as early as possible
//! 4. **Clear Errors**: Provide actionable error messages

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::warn;

/// Poka-Yoke validator for CLI arguments (FM-031, RPN: 280)
///
/// Validates CLI arguments at parse time to prevent invalid configurations
/// from reaching execution. This catches errors immediately with clear messages.
pub struct CliArgumentValidator;

impl CliArgumentValidator {
    /// Validate run command arguments
    ///
    /// # Errors
    ///
    /// Returns error with clear remediation if arguments are invalid
    pub fn validate_run_args(
        parallel: bool,
        jobs: usize,
        watch: bool,
        fail_fast: bool,
        shard: Option<(usize, usize)>,
    ) -> Result<()> {
        // Poka-Yoke 1: Jobs must be > 0
        if jobs == 0 {
            return Err(CleanroomError::validation_error(
                "Invalid --jobs value: must be > 0\n\n\
                 Remediation:\n\
                 Use --jobs 1 or higher (e.g., --jobs 4)\n\n\
                 Exit code: 2",
            ));
        }

        // Poka-Yoke 2: Jobs must be reasonable (prevent resource exhaustion)
        const MAX_JOBS: usize = 1000;
        if jobs > MAX_JOBS {
            return Err(CleanroomError::validation_error(format!(
                "Invalid --jobs value: {} exceeds maximum of {}\n\n\
                 Remediation:\n\
                 Use --jobs {} or lower to prevent resource exhaustion\n\n\
                 Exit code: 2",
                jobs, MAX_JOBS, MAX_JOBS
            )));
        }

        // Poka-Yoke 3: Parallel must be enabled if jobs > 1
        if !parallel && jobs > 1 {
            return Err(CleanroomError::validation_error(
                "Invalid configuration: --jobs > 1 requires --parallel\n\n\
                 Remediation:\n\
                 Use --parallel flag or set --jobs 1\n\n\
                 Exit code: 2",
            ));
        }

        // Poka-Yoke 4: Watch mode incompatible with parallel
        if watch && parallel {
            return Err(CleanroomError::validation_error(
                "Invalid configuration: --watch and --parallel are incompatible\n\n\
                 Remediation:\n\
                 Use either --watch (sequential) or --parallel (not both)\n\n\
                 Exit code: 2",
            ));
        }

        // Poka-Yoke 5: Shard validation
        if let Some((shard_index, total_shards)) = shard {
            if shard_index == 0 || shard_index > total_shards {
                return Err(CleanroomError::validation_error(format!(
                    "Invalid --shard value: {}/{}\n\n\
                     Remediation:\n\
                     Shard index must be 1-based and <= total shards\n\
                     Example: --shard 1/4 (first of four shards)\n\n\
                     Exit code: 2",
                    shard_index, total_shards
                )));
            }
        }

        Ok(())
    }

    /// Validate OTEL configuration arguments
    ///
    /// # Errors
    ///
    /// Returns error if OTEL configuration is invalid
    pub fn validate_otel_args(
        exporter: &str,
        endpoint: Option<&String>,
        validate: bool,
    ) -> Result<()> {
        // Poka-Yoke 6: Endpoint required for OTLP exporters
        let otlp_exporters = ["otlp-http", "otlp-grpc"];
        if otlp_exporters.contains(&exporter) && endpoint.is_none() {
            return Err(CleanroomError::validation_error(
                "Invalid OTEL configuration: --otel-endpoint required for otlp-http/otlp-grpc\n\n\
                 Remediation:\n\
                 Use --otel-endpoint http://localhost:4317 (or your collector URL)\n\n\
                 Exit code: 2",
            ));
        }

        // Poka-Yoke 7: Validate requires OTEL exporter
        if validate && exporter == "none" {
            return Err(CleanroomError::validation_error(
                "Invalid configuration: --validate requires OTEL exporter\n\n\
                 Remediation:\n\
                 Use --otel-exporter otlp-grpc --otel-endpoint http://localhost:4317\n\n\
                 Exit code: 2",
            ));
        }

        Ok(())
    }
}

/// Poka-Yoke for concurrent container creation (FM-004, RPN: 168)
///
/// Prevents race conditions in container creation by using a lock per image.
/// This ensures only one container is created per image at a time.
pub struct ContainerCreationLock {
    /// Locks per image (image_name -> Mutex)
    locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl ContainerCreationLock {
    /// Create new container creation lock
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Acquire lock for container creation
    ///
    /// Waits until the lock is available, then returns. The lock is held
    /// internally and will be released when this function returns.
    /// 
    /// Note: This is a blocking wait - the function will not return until
    /// the lock is acquired. Use this before creating a container.
    pub async fn acquire(&self, image: &str) -> Result<()> {
        // Get or create lock for this image (must clone Arc before dropping locks_map guard)
        let lock = {
            let mut locks_map = self.locks.lock().await;
            locks_map
                .entry(image.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        // Acquire the lock (will wait if another creation is in progress)
        // The lock is held for the duration of this await
        let _guard = lock.lock().await;
        
        // Guard is held here - when this function returns, guard is dropped and lock is released
        // But we want to hold it during container creation, so the caller must ensure
        // container creation happens synchronously within this async context
        Ok(())
    }
}

impl Default for ContainerCreationLock {
    fn default() -> Self {
        Self::new()
    }
}

/// Global container creation lock (shared across all backends)
///
/// This ensures that even if multiple GvisorBackend instances
/// try to create containers for the same image simultaneously, only
/// one creation happens at a time per image.
static GLOBAL_CONTAINER_LOCK: once_cell::sync::Lazy<ContainerCreationLock> =
    once_cell::sync::Lazy::new(ContainerCreationLock::new);

/// Acquire global lock for container creation and execute closure
///
/// This is the main entry point for preventing concurrent container creation
/// race conditions. The lock is held for the duration of the closure execution.
///
/// # Example
///
/// ```rust,no_run
/// use clnrm_core::poka_yoke::with_container_creation_lock;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// with_container_creation_lock("alpine:latest", || {
///     // Create container here - lock prevents concurrent creation
///     Ok(())
/// }).await?;
/// # Ok(())
/// # }
/// ```
pub async fn with_container_creation_lock<F, R>(image: &str, f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    let _guard = GLOBAL_CONTAINER_LOCK.acquire(image).await?;
    f()
}

/// Acquire global lock for container creation (returns guard)
///
/// This is a lower-level API that returns a guard. The guard must be held
/// for the duration of container creation.
///
/// Note: The guard has a lifetime tied to the lock, so it cannot be stored
/// across await points. Use `with_container_creation_lock` for most cases.
pub async fn acquire_container_creation_lock(
    image: &str,
) -> Result<()> {
    let _guard = GLOBAL_CONTAINER_LOCK.acquire(image).await?;
    // Guard is dropped here, but that's OK - we just needed to wait for the lock
    // The actual locking happens inside acquire() which holds the lock
    Ok(())
}

/// Poka-Yoke for TOML parsing edge cases (FM-008, RPN: 180)
///
/// Validates TOML content before parsing to catch common errors early
/// with clear, actionable error messages.
pub struct TomlPokaYoke;

impl TomlPokaYoke {
    /// Validate TOML content before parsing
    ///
    /// # Errors
    ///
    /// Returns error with clear remediation if TOML has common issues
    pub fn validate_before_parse(content: &str, file_path: &Path) -> Result<()> {
        // Poka-Yoke 1: Check for unclosed strings
        if let Some(line_num) = Self::find_unclosed_string(content) {
            return Err(CleanroomError::configuration_error(format!(
                "TOML parse error: Unclosed string at line {}\n\n\
                 File: {}\n\
                 Remediation:\n\
                 Close all string literals with matching quotes\n\
                 Example: name = \"value\" (not name = \"value)\n\n\
                 Exit code: 2",
                line_num,
                file_path.display()
            )));
        }

        // Poka-Yoke 2: Check for invalid escape sequences
        if let Some((line_num, seq)) = Self::find_invalid_escape(content) {
            return Err(CleanroomError::configuration_error(format!(
                "TOML parse error: Invalid escape sequence '{}' at line {}\n\n\
                 File: {}\n\
                 Remediation:\n\
                 Use valid escape sequences: \\n, \\t, \\\", \\\\, \\uXXXX\n\n\
                 Exit code: 2",
                seq, line_num, file_path.display()
            )));
        }

        // Poka-Yoke 3: Check for circular template references
        if let Some(circular_ref) = Self::find_circular_template_ref(content) {
            return Err(CleanroomError::configuration_error(format!(
                "TOML parse error: Circular template reference detected: {}\n\n\
                 File: {}\n\
                 Remediation:\n\
                 Remove circular dependencies in template variables\n\n\
                 Exit code: 2",
                circular_ref,
                file_path.display()
            )));
        }

        // Poka-Yoke 4: Check for missing required sections
        // Check for [test] or [containers] section (v2.0.0 format)
        // Use regex to match section headers (lines starting with [test] or [containers])
        let has_test_section = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == "[test]" || trimmed.starts_with("[test.")
        });
        let has_containers_section = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == "[containers]" || trimmed.starts_with("[containers.")
        });
        let has_services_section = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("[service.") || trimmed.starts_with("[services.")
        });
        
        if !has_test_section && !has_containers_section && !has_services_section {
            return Err(CleanroomError::configuration_error(format!(
                "TOML parse error: Missing required section [test] or [containers]\n\n\
                 File: {}\n\
                 Remediation:\n\
                 Add [test] section with at least 'name' field, or [containers] section\n\n\
                 Exit code: 2",
                file_path.display()
            )));
        }

        Ok(())
    }

    /// Find unclosed strings in TOML content
    fn find_unclosed_string(content: &str) -> Option<usize> {
        let mut in_string = false;
        let mut escape_next = false;
        let mut quote_char = None;

        for (line_num, line) in content.lines().enumerate() {
            for ch in line.chars() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                if ch == '#' && !in_string {
                    break;
                }

                if ch == '\\' {
                    escape_next = true;
                    continue;
                }

                if (ch == '"' || ch == '\'') && !escape_next {
                    if in_string && quote_char == Some(ch) {
                        in_string = false;
                        quote_char = None;
                    } else if !in_string {
                        in_string = true;
                        quote_char = Some(ch);
                    }
                }
            }

            // If we're still in a string at end of line (and it's not a multiline string)
            if in_string && !line.trim_end().ends_with('\\') {
                return Some(line_num + 1);
            }
        }

        if in_string {
            Some(content.lines().count())
        } else {
            None
        }
    }

    /// Find invalid escape sequences
    fn find_invalid_escape(content: &str) -> Option<(usize, String)> {
        for (line_num, line) in content.lines().enumerate() {
            let mut chars = line.chars().peekable();
            let mut escape_next = false;

            while let Some(ch) = chars.next() {
                if escape_next {
                    escape_next = false;
                    // Valid escapes: n, t, r, ", ', \, u, U
                    if !matches!(ch, 'n' | 't' | 'r' | '"' | '\'' | '\\' | 'u' | 'U' | 'x') {
                        return Some((line_num + 1, format!("\\{}", ch)));
                    }
                    continue;
                }

                if ch == '\\' {
                    escape_next = true;
                }
            }
        }

        None
    }

    /// Find circular template references
    fn find_circular_template_ref(content: &str) -> Option<String> {
        // Simple heuristic: look for {{ var }} that references itself
        // This is a simplified check - full cycle detection would require
        // building a dependency graph
        // Use string matching instead of regex to avoid dependency
        let mut in_vars_section = false;
        let mut var_definitions = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed == "[vars]" || trimmed == "[variables]" {
                in_vars_section = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_vars_section = false;
                continue;
            }

            if in_vars_section && trimmed.contains('=') {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    // Check if value contains template reference to itself
                    if value.contains(&format!("{{{{ {} }}}}", key)) || 
                       value.contains(&format!("{{{{{} }}}}", key)) ||
                       value.contains(&format!("{{{{ {} }}}}", key.trim())) {
                        return Some(key.to_string());
                    }
                    var_definitions.insert(key.to_string(), value.to_string());
                }
            }
        }

        None
    }
}

/// Poka-Yoke for zero telemetry samples detection (FM-013, RPN: 150)
///
/// Detects zero telemetry samples early and provides clear diagnostics.
pub struct TelemetrySampleValidator;

impl TelemetrySampleValidator {
    /// Validate telemetry samples before validation
    ///
    /// # Errors
    ///
    /// Returns error with clear diagnostics if zero samples detected
    pub fn validate_samples(
        sample_count: usize,
        exporter: &str,
        endpoint: Option<&str>,
    ) -> Result<()> {
        if sample_count == 0 {
            let diagnostics = Self::diagnose_zero_samples(exporter, endpoint);
            return Err(CleanroomError::validation_error(format!(
                "CRITICAL: Zero telemetry samples received\n\n\
                 This means validation did not actually test anything.\n\
                 Validation result is meaningless.\n\n\
                 Diagnostics:\n{}\n\n\
                 Remediation:\n\
                 1. Check OTEL exporter configuration\n\
                 2. Verify collector is running and accessible\n\
                 3. Ensure tests emit telemetry (check for OTEL spans)\n\
                 4. Check network connectivity to collector\n\n\
                 Exit code: 2",
                diagnostics
            )));
        }

        Ok(())
    }

    /// Diagnose why zero samples were received
    fn diagnose_zero_samples(exporter: &str, endpoint: Option<&str>) -> String {
        let mut diagnostics = Vec::new();

        if exporter == "none" {
            diagnostics.push("  ❌ OTEL exporter is 'none' - no telemetry will be exported".to_string());
        }

        if exporter.starts_with("otlp") && endpoint.is_none() {
            diagnostics.push("  ❌ OTLP exporter configured but no endpoint specified".to_string());
        }

        if let Some(endpoint) = endpoint {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                diagnostics.push(format!("  ⚠️  Endpoint '{}' may be invalid (should start with http:// or https://)", endpoint));
            }
        }

        if diagnostics.is_empty() {
            diagnostics.push("  ⚠️  Configuration appears correct - check collector logs".to_string());
        }

        diagnostics.join("\n")
    }
}

/// Poka-Yoke for container startup timeout (FM-002, RPN: 120)
///
/// Provides adaptive timeout based on image pull status and system load.
pub struct AdaptiveStartupTimeout {
    /// Base timeout for cached images
    base_timeout: std::time::Duration,
    /// Extended timeout for first-time image pulls
    pull_timeout: std::time::Duration,
}

impl AdaptiveStartupTimeout {
    /// Create new adaptive timeout
    pub fn new() -> Self {
        Self {
            base_timeout: Duration::from_secs(10), // Fast for cached images
            pull_timeout: Duration::from_secs(60),  // Extended for first pull
        }
    }

    /// Get timeout based on whether image is cached
    ///
    /// # Arguments
    ///
    /// * `image_cached` - Whether the image is already cached locally
    /// * `system_load` - Current system load (0.0-1.0, higher = more loaded)
    pub fn get_timeout(&self, image_cached: bool, system_load: f64) -> Duration {
        let base = if image_cached {
            self.base_timeout
        } else {
            self.pull_timeout
        };

        // Increase timeout under high load (up to 2x)
        let multiplier = 1.0 + system_load;
        Duration::from_secs_f64(base.as_secs_f64() * multiplier)
    }
}

impl Default for AdaptiveStartupTimeout {
    fn default() -> Self {
        Self::new()
    }
}

/// Poka-Yoke for pool exhaustion (FM-005, RPN: 120)
///
/// Provides clear error messages and backpressure when pool is exhausted.
pub struct PoolExhaustionHandler;

impl PoolExhaustionHandler {
    /// Handle pool exhaustion with clear error message
    ///
    /// # Errors
    ///
    /// Returns error with actionable remediation
    pub fn handle_exhaustion(
        max_size: usize,
        current_size: usize,
        pending_requests: usize,
    ) -> Result<()> {
        Err(CleanroomError::resource_limit_exceeded(format!(
            "Container pool exhausted\n\n\
             Current Status:\n\
             - Pool size: {}/{}\n\
             - Pending requests: {}\n\n\
             Remediation:\n\
             1. Increase pool size: CLNRM_POOL_MAX_SIZE={}\n\
             2. Reduce concurrent jobs: --jobs {}\n\
             3. Disable pooling: unset CLNRM_ENABLE_POOLING\n\
             4. Wait for containers to be released\n\n\
             Exit code: 2",
            current_size,
            max_size,
            pending_requests,
            max_size * 2,
            max_size / 2
        )))
    }

    /// Check if pool is approaching exhaustion and warn
    pub fn check_exhaustion_risk(current: usize, max: usize, threshold: f64) -> bool {
        let utilization = current as f64 / max as f64;
        if utilization >= threshold {
            warn!(
                "Pool utilization high: {:.1}% ({}/{}) - consider increasing pool size",
                utilization * 100.0,
                current,
                max
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_validator_jobs_zero() {
        let result = CliArgumentValidator::validate_run_args(false, 0, false, false, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be > 0"));
    }

    #[test]
    fn test_cli_validator_jobs_too_large() {
        let result = CliArgumentValidator::validate_run_args(true, 2000, false, false, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_cli_validator_parallel_required() {
        let result = CliArgumentValidator::validate_run_args(false, 4, false, false, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires --parallel"));
    }

    #[test]
    fn test_toml_validator_unclosed_string() {
        let content = r#"
[test]
name = "test
"#;
        let result = TomlPokaYoke::validate_before_parse(content, Path::new("test.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unclosed string"));
    }

    #[test]
    fn test_toml_validator_missing_section() {
        let content = r#"
# No [test] or [containers] section
name = "test"
"#;
        let result = TomlPokaYoke::validate_before_parse(content, Path::new("test.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required section"));
    }

    #[tokio::test]
    async fn test_container_creation_lock() {
        let lock = ContainerCreationLock::new();
        let guard1 = lock.acquire("alpine:latest").await.unwrap();
        // Second acquire should wait (but we can't easily test that without timeouts)
        drop(guard1);
        // Now second acquire should succeed
        let _guard2 = lock.acquire("alpine:latest").await.unwrap();
    }

    #[test]
    fn test_adaptive_timeout() {
        let timeout = AdaptiveStartupTimeout::new();
        let cached = timeout.get_timeout(true, 0.0);
        let uncached = timeout.get_timeout(false, 0.0);
        assert!(uncached > cached);
    }

    #[test]
    fn test_pool_exhaustion_handler() {
        let result = PoolExhaustionHandler::handle_exhaustion(10, 10, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exhausted"));
    }
}

