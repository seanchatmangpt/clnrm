//! Migration report generator

use crate::types::{
    ConversionResult, MigrationReport, ServiceDiscovery, ValidationError, ValidationResult,
    ValidationWarning,
};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct Reporter;

impl Reporter {
    pub fn new() -> Self {
        Self
    }

    /// Generate migration report
    pub fn generate(
        &self,
        discoveries: &[ServiceDiscovery],
        conversions: &[ConversionResult],
        validation: &ValidationResult,
    ) -> Result<MigrationReport> {
        Ok(MigrationReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_services: discoveries.len(),
            converted_services: conversions.len(),
            validation_errors: validation.errors.len(),
            validation_warnings: validation.warnings.len(),
            services: conversions.to_vec(),
            errors: validation.errors.clone(),
            warnings: validation.warnings.clone(),
        })
    }

    /// Write report to files (JSON and Markdown)
    pub fn write_report(&self, report: &MigrationReport, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir)?;

        // Write JSON report
        let json_path = output_dir.join("migration-report.json");
        let json = serde_json::to_string_pretty(report)?;
        fs::write(&json_path, json)?;
        tracing::info!("Wrote JSON report to {}", json_path.display());

        // Write Markdown report
        let md_path = output_dir.join("migration-report.md");
        let markdown = self.generate_markdown(report);
        fs::write(&md_path, markdown)?;
        tracing::info!("Wrote Markdown report to {}", md_path.display());

        Ok(())
    }

    fn generate_markdown(&self, report: &MigrationReport) -> String {
        let mut md = format!(
            r#"# gVisor Migration Report

**Generated:** {}

## Summary

- **Total services found:** {}
- **Converted services:** {}
- **Validation errors:** {}
- **Validation warnings:** {}

"#,
            report.timestamp,
            report.total_services,
            report.converted_services,
            report.validation_errors,
            report.validation_warnings,
        );

        // Services table
        md.push_str("## Converted Services\n\n");
        md.push_str("| Service | Type | Status | Warnings |\n");
        md.push_str("|---------|------|--------|----------|\n");

        for service in &report.services {
            let status = if service.target_config.is_empty() {
                "⚠️ Manual"
            } else {
                "✅ Auto"
            };

            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                service.source.service_name,
                service.source.service_type.as_str(),
                status,
                service.warnings.len()
            ));
        }

        md.push_str("\n");

        // Errors
        if !report.errors.is_empty() {
            md.push_str("## Validation Errors\n\n");
            md.push_str("| Service | Type | Message | Suggestion |\n");
            md.push_str("|---------|------|---------|------------|\n");

            for error in &report.errors {
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    error.service_name, error.error_type, error.message, error.suggestion
                ));
            }

            md.push_str("\n");
        }

        // Warnings
        if !report.warnings.is_empty() {
            md.push_str("## Warnings\n\n");

            for warning in &report.warnings {
                md.push_str(&format!(
                    "- **[{}]** {}: {}\n",
                    warning.severity, warning.service_name, warning.message
                ));
            }

            md.push_str("\n");
        }

        // Next steps
        md.push_str(
            r#"## Next Steps

1. Review the generated `gvisor-services.toml` file
2. Address any validation errors or warnings
3. Test migrated services in a development environment
4. Update test configurations to use gVisor backend
5. Enable gVisor as the default backend

## Resources

- [gVisor Documentation](https://gvisor.dev/docs/)
- [Migration Design](GVISOR_MIGRATION_DESIGN.md)
- [Configuration Reference](gvisor-services.toml)
"#,
        );

        md
    }
}

impl Default for Reporter {
    fn default() -> Self {
        Self::new()
    }
}
