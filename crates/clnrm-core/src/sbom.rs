//! Software Bill of Materials (SBOM) generation for clnrm v1.5.0
//!
//! This module generates SBOM documents from Cargo.lock, providing transparency
//! about dependencies for security auditing and compliance.
//!
//! # Format
//!
//! The SBOM is generated in SPDX 2.3 JSON format, which is widely supported
//! by security scanning tools and compliance frameworks.
//!
//! # Example
//!
//! ```rust,no_run
//! use clnrm_core::sbom::SbomGenerator;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = SbomGenerator::new()?;
//! let sbom_json = generator.generate_spdx()?;
//! println!("{}", sbom_json);
//! # Ok(())
//! # }
//! ```

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// SPDX 2.3 SBOM document structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxDocument {
    /// SPDX version
    spdx_version: String,
    /// Data license
    data_license: String,
    /// SPDX identifier
    #[serde(rename = "SPDXID")]
    spdx_id: String,
    /// Document name
    name: String,
    /// Document namespace
    document_namespace: String,
    /// Creation info
    creation_info: CreationInfo,
    /// Packages in the SBOM
    packages: Vec<Package>,
}

/// SPDX creation information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreationInfo {
    /// Creation timestamp
    created: String,
    /// Creators
    creators: Vec<String>,
    /// License list version
    license_list_version: String,
}

/// SPDX package information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Package {
    /// SPDX identifier
    #[serde(rename = "SPDXID")]
    spdx_id: String,
    /// Package name
    name: String,
    /// Package version
    version_info: String,
    /// Supplier (crates.io or git)
    supplier: String,
    /// Download location
    download_location: String,
    /// Files analyzed
    files_analyzed: bool,
    /// Checksums
    checksums: Vec<Checksum>,
    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    /// Declared license
    #[serde(skip_serializing_if = "Option::is_none")]
    license_declared: Option<String>,
}

/// SPDX checksum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Checksum {
    /// Algorithm
    algorithm: String,
    /// Checksum value
    checksum_value: String,
}

/// Cargo.lock package entry
#[derive(Debug, Deserialize)]
struct CargoLockPackage {
    name: String,
    version: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    checksum: Option<String>,
}

/// Cargo.lock structure
#[derive(Debug, Deserialize)]
struct CargoLock {
    #[serde(default)]
    package: Vec<CargoLockPackage>,
}

/// SBOM generator
pub struct SbomGenerator {
    /// Path to Cargo.lock
    cargo_lock_path: PathBuf,
}

impl SbomGenerator {
    /// Create a new SBOM generator
    ///
    /// # Errors
    ///
    /// Returns error if Cargo.lock cannot be found
    pub fn new() -> Result<Self> {
        let cargo_lock_path = Self::find_cargo_lock()?;
        Ok(Self { cargo_lock_path })
    }

    /// Create generator with explicit Cargo.lock path
    pub fn with_path(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(CleanroomError::internal_error(format!(
                "Cargo.lock not found at: {}",
                path.display()
            )));
        }
        Ok(Self {
            cargo_lock_path: path,
        })
    }

    /// Find Cargo.lock in workspace root
    fn find_cargo_lock() -> Result<PathBuf> {
        // Start from current directory and walk up to find Cargo.lock
        let mut current_dir = std::env::current_dir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to get current directory: {}", e))
        })?;

        loop {
            let cargo_lock = current_dir.join("Cargo.lock");
            if cargo_lock.exists() {
                return Ok(cargo_lock);
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => {
                    return Err(CleanroomError::internal_error(
                        "Cargo.lock not found in workspace",
                    ))
                }
            }
        }
    }

    /// Parse Cargo.lock
    fn parse_cargo_lock(&self) -> Result<CargoLock> {
        let content = fs::read_to_string(&self.cargo_lock_path).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to read Cargo.lock: {}", e))
        })?;

        toml::from_str(&content).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to parse Cargo.lock: {}", e))
        })
    }

    /// Generate SPDX 2.3 SBOM
    ///
    /// # Errors
    ///
    /// Returns error if Cargo.lock cannot be parsed or SBOM cannot be serialized
    pub fn generate_spdx(&self) -> Result<String> {
        let cargo_lock = self.parse_cargo_lock()?;

        let document = SpdxDocument {
            spdx_version: "SPDX-2.3".to_string(),
            data_license: "CC0-1.0".to_string(),
            spdx_id: "SPDXRef-DOCUMENT".to_string(),
            name: "clnrm-sbom".to_string(),
            document_namespace: format!(
                "https://github.com/seanchatmangpt/clnrm/sbom-{}",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            ),
            creation_info: CreationInfo {
                created: chrono::Utc::now().to_rfc3339(),
                creators: vec![
                    "Tool: clnrm-sbom-1.5.0".to_string(),
                    "Organization: clnrm".to_string(),
                ],
                license_list_version: "3.21".to_string(),
            },
            packages: cargo_lock
                .package
                .into_iter()
                .enumerate()
                .map(|(idx, pkg)| self.convert_package(pkg, idx))
                .collect(),
        };

        serde_json::to_string_pretty(&document)
            .map_err(|e| CleanroomError::internal_error(format!("Failed to serialize SBOM: {}", e)))
    }

    /// Convert Cargo.lock package to SPDX package
    fn convert_package(&self, pkg: CargoLockPackage, idx: usize) -> Package {
        let download_location = pkg
            .source
            .as_ref()
            .map(|s| {
                if s.starts_with("registry+") {
                    format!("https://crates.io/crates/{}/{}", pkg.name, pkg.version)
                } else if s.starts_with("git+") {
                    s.trim_start_matches("git+").to_string()
                } else {
                    "NOASSERTION".to_string()
                }
            })
            .unwrap_or_else(|| "NOASSERTION".to_string());

        let checksums = pkg
            .checksum
            .map(|checksum| {
                vec![Checksum {
                    algorithm: "SHA256".to_string(),
                    checksum_value: checksum,
                }]
            })
            .unwrap_or_default();

        Package {
            spdx_id: format!("SPDXRef-Package-{}", idx + 1),
            name: pkg.name.clone(),
            version_info: pkg.version,
            supplier: "Organization: crates.io".to_string(),
            download_location,
            files_analyzed: false,
            checksums,
            homepage: Some(format!("https://crates.io/crates/{}", pkg.name)),
            license_declared: None, // Would need to parse Cargo.toml for this
        }
    }

    /// Generate simplified dependency list (human-readable)
    pub fn generate_dependency_list(&self) -> Result<String> {
        let cargo_lock = self.parse_cargo_lock()?;

        let mut output = String::new();
        output.push_str("# clnrm Dependency List\n\n");
        output.push_str(&format!(
            "Generated: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        ));
        output.push_str(&format!(
            "Total dependencies: {}\n\n",
            cargo_lock.package.len()
        ));

        output.push_str("## Dependencies\n\n");

        let mut packages: Vec<_> = cargo_lock.package.into_iter().collect();
        packages.sort_by(|a, b| a.name.cmp(&b.name));

        for pkg in packages {
            output.push_str(&format!("- {} v{}", pkg.name, pkg.version));
            if let Some(source) = pkg.source {
                if source.starts_with("git+") {
                    output.push_str(&format!(" (git: {})", source.trim_start_matches("git+")));
                }
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Get dependency statistics
    pub fn get_stats(&self) -> Result<HashMap<String, usize>> {
        let cargo_lock = self.parse_cargo_lock()?;

        let mut stats = HashMap::new();
        stats.insert("total_dependencies".to_string(), cargo_lock.package.len());

        let registry_deps = cargo_lock
            .package
            .iter()
            .filter(|p| {
                p.source
                    .as_ref()
                    .map(|s| s.starts_with("registry+"))
                    .unwrap_or(false)
            })
            .count();
        stats.insert("registry_dependencies".to_string(), registry_deps);

        let git_deps = cargo_lock
            .package
            .iter()
            .filter(|p| {
                p.source
                    .as_ref()
                    .map(|s| s.starts_with("git+"))
                    .unwrap_or(false)
            })
            .count();
        stats.insert("git_dependencies".to_string(), git_deps);

        let local_deps = cargo_lock
            .package
            .iter()
            .filter(|p| p.source.is_none())
            .count();
        stats.insert("local_dependencies".to_string(), local_deps);

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbom_generator_creation() {
        // Arrange: This should work in CI since workspace has Cargo.lock

        // Act
        let result = SbomGenerator::new();

        // Assert: Either succeeds or fails with proper error
        match result {
            Ok(generator) => {
                assert!(generator.cargo_lock_path.exists());
            }
            Err(e) => {
                // Acceptable in environments without Cargo.lock
                assert!(e.to_string().contains("Cargo.lock"));
            }
        }
    }

    #[test]
    fn test_spdx_document_structure() {
        // Arrange: Create minimal SPDX document

        // Act
        let doc = SpdxDocument {
            spdx_version: "SPDX-2.3".to_string(),
            data_license: "CC0-1.0".to_string(),
            spdx_id: "SPDXRef-DOCUMENT".to_string(),
            name: "test-sbom".to_string(),
            document_namespace: "https://example.com/test".to_string(),
            creation_info: CreationInfo {
                created: "2025-01-01T00:00:00Z".to_string(),
                creators: vec!["Tool: test".to_string()],
                license_list_version: "3.21".to_string(),
            },
            packages: vec![],
        };

        // Assert: Should serialize to valid JSON
        let json = serde_json::to_string(&doc);
        assert!(json.is_ok());
    }
}
