//! Default implementations of poka-yoke traits
//!
//! This module provides concrete implementations of all poka-yoke traits
//! for production use. These are the default validators used throughout
//! the codebase.

use crate::error::{CleanroomError, Result};
use crate::poka_yoke::traits::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::warn;

/// Default CLI argument validator implementation
///
/// Validates CLI arguments at parse time to prevent invalid configurations.
#[derive(Debug, Clone, Default)]
pub struct DefaultCliValidator;

impl CliValidator for DefaultCliValidator {
    fn validate_run_args(
        &self,
        parallel: bool,
        jobs: usize,
        watch: bool,
        _fail_fast: bool,
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

    fn validate_otel_args(
        &self,
        exporter: &str,
        endpoint: Option<String>,
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

/// Default TOML validator implementation
///
/// Validates TOML content before parsing to catch common errors early.
#[derive(Debug, Clone, Default)]
pub struct DefaultTomlValidator;

impl TomlValidator for DefaultTomlValidator {
    fn validate_before_parse(&self, content: &str, file_path: &Path) -> Result<()> {
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
                seq,
                line_num,
                file_path.display()
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
}

impl DefaultTomlValidator {
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

            #[allow(clippy::while_let_on_iterator)]
            while let Some(ch) = chars.next() {
                if escape_next {
                    escape_next = false;
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
                    if value.contains(&format!("{{{{ {} }}}}", key))
                        || value.contains(&format!("{{{{{} }}}}", key))
                        || value.contains(&format!("{{{{ {} }}}}", key.trim()))
                    {
                        return Some(key.to_string());
                    }
                    var_definitions.insert(key.to_string(), value.to_string());
                }
            }
        }

        None
    }
}

/// Default telemetry validator implementation
///
/// Detects zero telemetry samples early and provides clear diagnostics.
#[derive(Debug, Clone, Default)]
pub struct DefaultTelemetryValidator;

impl TelemetryValidator for DefaultTelemetryValidator {
    fn validate_samples(
        &self,
        sample_count: usize,
        exporter: &str,
        endpoint: Option<String>,
    ) -> Result<()> {
        if sample_count == 0 {
            let diagnostics = Self::diagnose_zero_samples(exporter, endpoint.as_deref());
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
}

impl DefaultTelemetryValidator {
    /// Diagnose why zero samples were received
    fn diagnose_zero_samples(exporter: &str, endpoint: Option<&str>) -> String {
        let mut diagnostics = Vec::new();

        if exporter == "none" {
            diagnostics
                .push("  ❌ OTEL exporter is 'none' - no telemetry will be exported".to_string());
        }

        if exporter.starts_with("otlp") && endpoint.is_none() {
            diagnostics.push("  ❌ OTLP exporter configured but no endpoint specified".to_string());
        }

        if let Some(endpoint) = endpoint {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                diagnostics.push(format!(
                    "  ⚠️  Endpoint '{}' may be invalid (should start with http:// or https://)",
                    endpoint
                ));
            }
        }

        if diagnostics.is_empty() {
            diagnostics
                .push("  ⚠️  Configuration appears correct - check collector logs".to_string());
        }

        diagnostics.join("\n")
    }
}

/// Default timeout calculator implementation
///
/// Provides adaptive timeout based on image pull status and system load.
#[derive(Debug, Clone)]
pub struct DefaultTimeoutCalculator {
    /// Base timeout for cached images
    base_timeout: Duration,
    /// Extended timeout for first-time image pulls
    pull_timeout: Duration,
}

impl DefaultTimeoutCalculator {
    /// Create new adaptive timeout calculator
    pub fn new() -> Self {
        Self {
            base_timeout: Duration::from_secs(10), // Fast for cached images
            pull_timeout: Duration::from_secs(60), // Extended for first pull
        }
    }
}

impl Default for DefaultTimeoutCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeoutCalculator for DefaultTimeoutCalculator {
    fn get_timeout(&self, image_cached: bool, system_load: f64) -> Duration {
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

/// Default pool exhaustion handler implementation
///
/// Provides clear error messages and backpressure when pool is exhausted.
#[derive(Debug, Clone, Default)]
pub struct DefaultPoolExhaustionHandler;

impl PoolExhaustionHandler for DefaultPoolExhaustionHandler {
    fn handle_exhaustion(
        &self,
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

    fn check_exhaustion_risk(&self, current: usize, max: usize, threshold: f64) -> bool {
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

/// Default container creation lock implementation
///
/// Prevents race conditions in container creation by using locks per image.
#[derive(Debug)]
pub struct DefaultContainerCreationLock {
    /// Locks per image (image_name -> Mutex)
    locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl DefaultContainerCreationLock {
    /// Create new container creation lock
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for DefaultContainerCreationLock {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ContainerCreationLock for DefaultContainerCreationLock {
    async fn acquire(&self, image: &str) -> Result<()> {
        let lock = {
            let mut locks_map = self.locks.lock().await;
            locks_map
                .entry(image.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        let _guard = lock.lock().await;
        Ok(())
    }
}
