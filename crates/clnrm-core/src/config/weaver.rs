//! Weaver live-check configuration
//!
//! Configuration for Weaver live-checking support in TOML files.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Weaver live-check configuration (v1.3.0)
///
/// Enables Weaver validation for test execution when present in TOML.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WeaverConfig {
    /// Enable Weaver live-checking
    /// Default: true if [weaver] section is present
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Registry path for Weaver schemas
    /// Default: "registry" (relative to installation)
    #[serde(default = "default_registry_path")]
    pub registry_path: String,

    /// OTLP gRPC port (0 = auto-discover)
    /// Default: 0 (auto-discover)
    #[serde(default)]
    pub otlp_port: u16,

    /// Admin port for control interface (0 = auto-discover)
    /// Default: 0 (auto-discover)
    #[serde(default)]
    pub admin_port: u16,

    /// Output directory for validation reports
    /// Default: "./validation_output"
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Enable streaming output (real-time feedback)
    /// Default: false
    #[serde(default)]
    pub stream: bool,

    /// Fail fast on first violation
    /// Default: false
    #[serde(default)]
    pub fail_fast: bool,
}

fn default_true() -> bool {
    true
}

fn default_registry_path() -> String {
    "registry".to_string()
}

fn default_output_dir() -> String {
    "./validation_output".to_string()
}

impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            registry_path: "registry".to_string(),
            otlp_port: 0,
            admin_port: 0,
            output_dir: "./validation_output".to_string(),
            stream: false,
            fail_fast: false,
        }
    }
}

impl WeaverConfig {
    /// Convert to telemetry::WeaverConfig
    pub fn to_telemetry_config(&self) -> Result<crate::telemetry::weaver_controller::WeaverConfig> {
        use crate::telemetry::weaver_controller::WeaverConfig as TelemetryWeaverConfig;

        // Resolve registry path (can be relative or absolute)
        let registry_path = if self.registry_path.starts_with('/') || self.registry_path.starts_with("~/") {
            // Absolute path or home directory
            PathBuf::from(self.registry_path.replace("~/", &format!("{}/", std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))))
        } else {
            // Relative path - resolve from installation directory or current directory
            // Note: Actual resolution happens in run command
            PathBuf::from(&self.registry_path)
        };

        Ok(TelemetryWeaverConfig {
            registry_path,
            otlp_port: self.otlp_port,
            admin_port: self.admin_port,
            output_dir: PathBuf::from(&self.output_dir),
            stream: self.stream,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.otlp_port > 0 && self.otlp_port < 1024 {
            return Err(CleanroomError::validation_error(
                "OTLP port must be >= 1024 or 0 for auto-discovery"
            ));
        }

        if self.admin_port > 0 && self.admin_port < 1024 {
            return Err(CleanroomError::validation_error(
                "Admin port must be >= 1024 or 0 for auto-discovery"
            ));
        }

        if self.otlp_port == self.admin_port && self.otlp_port > 0 {
            return Err(CleanroomError::validation_error(
                "OTLP port and admin port must be different"
            ));
        }

        Ok(())
    }
}

