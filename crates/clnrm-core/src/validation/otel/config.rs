//! Configuration types for OpenTelemetry validation
//!
//! This module provides configuration structures for the OTEL validation system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenTelemetry validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelValidationConfig {
    /// Enable span validation
    pub validate_spans: bool,
    /// Enable trace completeness validation
    pub validate_traces: bool,
    /// Enable export validation
    pub validate_exports: bool,
    /// Enable performance overhead validation
    pub validate_performance: bool,
    /// Maximum allowed performance overhead in milliseconds
    pub max_overhead_ms: f64,
    /// Expected span attributes
    pub expected_attributes: HashMap<String, String>,
}

impl Default for OtelValidationConfig {
    fn default() -> Self {
        Self {
            validate_spans: true,
            validate_traces: true,
            validate_exports: false, // Requires external collector
            validate_performance: true,
            max_overhead_ms: 100.0,
            expected_attributes: HashMap::new(),
        }
    }
}
