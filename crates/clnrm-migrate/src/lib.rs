//! Migration tool for converting testcontainers configs to gVisor
//!
//! This library provides functionality to:
//! - Scan codebases for testcontainers usage
//! - Convert ServiceConfig to GvisorServiceConfig
//! - Validate converted configurations
//! - Generate migration reports

pub mod scanner;
pub mod converter;
pub mod validator;
pub mod reporter;
pub mod types;

pub use scanner::Scanner;
pub use converter::Converter;
pub use validator::Validator;
pub use reporter::Reporter;
pub use types::*;

use anyhow::Result;
use std::path::Path;

/// Main migration engine orchestrating the entire process
pub struct MigrationEngine {
    scanner: Scanner,
    converter: Converter,
    validator: Validator,
    reporter: Reporter,
}

impl MigrationEngine {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new(),
            converter: Converter::new(),
            validator: Validator::new(),
            reporter: Reporter::new(),
        }
    }

    /// Run full migration pipeline
    pub fn migrate(&mut self, root_dir: &Path, output_dir: &Path) -> Result<MigrationReport> {
        tracing::info!("Starting migration from {}", root_dir.display());

        // Step 1: Scan
        let discoveries = self.scanner.scan(root_dir)?;
        tracing::info!("Found {} services", discoveries.len());

        // Step 2: Convert
        let conversions = self.converter.convert_all(&discoveries)?;
        tracing::info!("Converted {} services", conversions.len());

        // Step 3: Validate
        let validation_result = self.validator.validate_all(&conversions)?;
        tracing::info!(
            "Validation complete: {} errors, {} warnings",
            validation_result.errors.len(),
            validation_result.warnings.len()
        );

        // Step 4: Generate report
        let report = self.reporter.generate(&discoveries, &conversions, &validation_result)?;

        // Step 5: Write outputs
        self.reporter.write_report(&report, output_dir)?;
        self.converter.write_configs(&conversions, output_dir)?;

        tracing::info!("Migration complete!");

        Ok(report)
    }
}

impl Default for MigrationEngine {
    fn default() -> Self {
        Self::new()
    }
}
