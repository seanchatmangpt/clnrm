/// Generator Configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::Result;

/// Generator configuration (mirrors ggen.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    pub project: ProjectConfig,
    pub generation: GenerationConfig,
    pub rdf: RdfConfig,
    pub templates: TemplatesConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub ontology_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub output_dir: PathBuf,
    pub incremental: bool,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfConfig {
    pub formats: Vec<String>,
    pub default_format: String,
    pub base_uri: String,
    pub strict_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    pub enable_caching: bool,
    pub auto_reload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub formatting: String,
    pub line_length: usize,
    pub indent: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "clnrm".to_string(),
                version: "2.0.0".to_string(),
                description: "Cleanroom Testing Framework".to_string(),
                authors: vec!["Sean Chatman <seanchatmangpt@gmail.com>".to_string()],
                license: "MIT".to_string(),
            },
            generation: GenerationConfig {
                ontology_dir: PathBuf::from("schema/"),
                templates_dir: PathBuf::from("templates/"),
                output_dir: PathBuf::from("generated/"),
                incremental: true,
                overwrite: false,
            },
            rdf: RdfConfig {
                formats: vec!["turtle".to_string()],
                default_format: "turtle".to_string(),
                base_uri: "https://clnrm.io/ontology/".to_string(),
                strict_validation: false,
            },
            templates: TemplatesConfig {
                enable_caching: true,
                auto_reload: true,
            },
            output: OutputConfig {
                formatting: "default".to_string(),
                line_length: 100,
                indent: 2,
            },
        }
    }
}

impl GeneratorConfig {
    pub async fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_toml(&content)
    }

    pub fn from_toml(content: &str) -> Result<Self> {
        match toml::from_str(content) {
            Ok(config) => Ok(config),
            Err(e) => Err(crate::error::GgenError::ConfigError(format!(
                "Failed to parse config: {}",
                e
            ))),
        }
    }

    pub fn to_toml(&self) -> Result<String> {
        match toml::to_string_pretty(self) {
            Ok(s) => Ok(s),
            Err(e) => Err(crate::error::GgenError::ConfigError(format!(
                "Failed to serialize config: {}",
                e
            ))),
        }
    }

    /// Get full paths for ontology and template directories
    pub fn get_ontology_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if self.generation.ontology_dir.exists() {
            paths.push(self.generation.ontology_dir.clone());
        }
        paths
    }

    pub fn get_template_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if self.generation.templates_dir.exists() {
            paths.push(self.generation.templates_dir.clone());
        }
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GeneratorConfig::default();
        assert_eq!(config.project.name, "clnrm");
        assert_eq!(config.project.version, "2.0.0");
    }

    #[test]
    fn test_config_serialization() {
        let config = GeneratorConfig::default();
        let toml = config.to_toml().unwrap();
        assert!(toml.contains("name = \"clnrm\""));
    }

    #[tokio::test]
    async fn test_config_default_paths() {
        let config = GeneratorConfig::default();
        let paths = config.get_ontology_paths();
        // Will only return if dirs exist, which they do in our setup
        assert!(!paths.is_empty());
    }
}
