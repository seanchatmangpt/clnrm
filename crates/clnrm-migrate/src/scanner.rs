//! Code scanner for discovering testcontainers usage

use crate::types::{ServiceDiscovery, ServiceType};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct Scanner;

impl Scanner {
    pub fn new() -> Self {
        Self
    }

    /// Scan directory for testcontainers service definitions
    pub fn scan(&self, root_dir: &Path) -> Result<Vec<ServiceDiscovery>> {
        let mut discoveries = Vec::new();

        // Scan .clnrm.toml files
        discoveries.extend(self.scan_toml_files(root_dir)?);

        // Scan Rust source files
        discoveries.extend(self.scan_rust_files(root_dir)?);

        Ok(discoveries)
    }

    fn scan_toml_files(&self, root_dir: &Path) -> Result<Vec<ServiceDiscovery>> {
        let mut discoveries = Vec::new();
        let pattern = format!("{}/**/*.clnrm.toml", root_dir.display());

        for entry in glob::glob(&pattern)? {
            let path = entry?;
            let content = fs::read_to_string(&path)?;

            // Parse TOML and extract services
            if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                if let Some(services) = value.get("services") {
                    if let Some(table) = services.as_table() {
                        for (name, config) in table {
                            discoveries.push(ServiceDiscovery {
                                source_file: path.clone(),
                                service_name: name.clone(),
                                service_type: self.detect_type(config),
                                line_number: None,
                                raw_config: toml::to_string(config).unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }

        Ok(discoveries)
    }

    fn scan_rust_files(&self, root_dir: &Path) -> Result<Vec<ServiceDiscovery>> {
        let mut discoveries = Vec::new();
        let pattern = format!("{}/crates/*/src/**/*.rs", root_dir.display());

        for entry in glob::glob(&pattern)? {
            let path = entry?;
            let content = fs::read_to_string(&path)?;

            // Look for testcontainers usage
            if content.contains("testcontainers::") || content.contains("ServicePlugin") {
                // Simple pattern matching (could be enhanced with syn parsing)
                if let Some(discovery) = self.extract_from_rust(&path, &content) {
                    discoveries.push(discovery);
                }
            }
        }

        Ok(discoveries)
    }

    fn detect_type(&self, config: &toml::Value) -> ServiceType {
        if let Some(plugin) = config.get("type").or_else(|| config.get("plugin")) {
            if let Some(plugin_str) = plugin.as_str() {
                match plugin_str {
                    "surrealdb" => return ServiceType::SurrealDB,
                    "generic_container" => return ServiceType::GenericContainer,
                    _ => {}
                }
            }
        }

        ServiceType::CustomImage
    }

    fn extract_from_rust(&self, _path: &Path, _content: &str) -> Option<ServiceDiscovery> {
        // Simplified - would use syn crate for proper AST parsing
        None
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}
