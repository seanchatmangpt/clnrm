//! Common types for migration tool

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Service discovery result from scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    pub source_file: PathBuf,
    pub service_name: String,
    pub service_type: ServiceType,
    pub line_number: Option<usize>,
    pub raw_config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    SurrealDB,
    GenericContainer,
    CustomImage,
    TestcontainersModule,
}

impl ServiceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SurrealDB => "surrealdb",
            Self::GenericContainer => "generic_container",
            Self::CustomImage => "custom_image",
            Self::TestcontainersModule => "testcontainers_module",
        }
    }
}

/// Conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub source: ServiceDiscovery,
    pub target_config: String,  // TOML representation
    pub warnings: Vec<String>,
    pub manual_steps: Vec<String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub service_name: String,
    pub error_type: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub service_name: String,
    pub severity: String,  // warning, security, performance
    pub message: String,
}

/// Complete migration report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub timestamp: String,
    pub total_services: usize,
    pub converted_services: usize,
    pub validation_errors: usize,
    pub validation_warnings: usize,
    pub services: Vec<ConversionResult>,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}
