//! Configuration validator

use crate::types::{ConversionResult, ValidationError, ValidationResult, ValidationWarning};
use anyhow::Result;

pub struct Validator;

impl Validator {
    pub fn new() -> Self {
        Self
    }

    /// Validate all converted configurations
    pub fn validate_all(&self, conversions: &[ConversionResult]) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for conversion in conversions {
            // Validate TOML syntax
            if !conversion.target_config.is_empty() {
                if let Err(e) = toml::from_str::<toml::Value>(&conversion.target_config) {
                    errors.push(ValidationError {
                        service_name: conversion.source.service_name.clone(),
                        error_type: "invalid_toml".to_string(),
                        message: format!("Invalid TOML syntax: {}", e),
                        suggestion: "Review and fix TOML syntax errors".to_string(),
                    });
                }
            }

            // Add conversion warnings
            for warning in &conversion.warnings {
                warnings.push(ValidationWarning {
                    service_name: conversion.source.service_name.clone(),
                    severity: "warning".to_string(),
                    message: warning.clone(),
                });
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
