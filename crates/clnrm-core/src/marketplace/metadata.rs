//! Plugin Metadata Management
//!
//! Defines the structure and validation of plugin metadata including
//! versioning, capabilities, dependencies, and community information.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin category for organization and discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCategory {
    /// Database testing plugins (PostgreSQL, MySQL, MongoDB, etc.)
    Database,
    /// Cache and session management (Redis, Memcached, etc.)
    Cache,
    /// Message queue and streaming (Kafka, RabbitMQ, etc.)
    MessageQueue,
    /// Web and API services (HTTP servers, REST APIs, etc.)
    Web,
    /// AI/ML model testing and validation
    AiMl,
    /// Storage and file systems (S3, MinIO, etc.)
    Storage,
    /// Monitoring and observability
    Observability,
    /// Security and authentication
    Security,
    /// Testing utilities and tools
    Testing,
    /// Custom/Other category
    Custom(String),
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginCategory::Database => write!(f, "database"),
            PluginCategory::Cache => write!(f, "cache"),
            PluginCategory::MessageQueue => write!(f, "message-queue"),
            PluginCategory::Web => write!(f, "web"),
            PluginCategory::AiMl => write!(f, "ai-ml"),
            PluginCategory::Storage => write!(f, "storage"),
            PluginCategory::Observability => write!(f, "observability"),
            PluginCategory::Security => write!(f, "security"),
            PluginCategory::Testing => write!(f, "testing"),
            PluginCategory::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Plugin capability descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapability {
    /// Capability identifier
    pub name: String,
    /// Category this capability belongs to
    pub category: PluginCategory,
    /// Human-readable description
    pub description: String,
    /// Optional configuration schema
    pub config_schema: Option<HashMap<String, String>>,
}

impl PluginCapability {
    pub fn new(
        name: impl Into<String>,
        category: PluginCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            description: description.into(),
            config_schema: None,
        }
    }

    pub fn with_config_schema(mut self, schema: HashMap<String, String>) -> Self {
        self.config_schema = Some(schema);
        self
    }
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency plugin name
    pub name: String,
    /// Version constraint (semver format)
    pub version_constraint: String,
    /// Whether this dependency is optional
    pub optional: bool,
}

impl PluginDependency {
    pub fn new(name: impl Into<String>, version_constraint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_constraint: version_constraint.into(),
            optional: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Community engagement and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    /// Average rating (0-5)
    pub average_rating: f64,
    /// Total number of ratings
    pub rating_count: u32,
    /// Total download count
    pub download_count: u64,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// User reviews
    pub reviews: Vec<String>,
}

impl Default for CommunityInfo {
    fn default() -> Self {
        Self {
            average_rating: 0.0,
            rating_count: 0,
            download_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            reviews: Vec::new(),
        }
    }
}

/// Complete plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin unique identifier
    pub name: String,
    /// Semantic version
    pub version: semver::Version,
    /// Short description
    pub description: String,
    /// Author name or organization
    pub author: String,
    /// License identifier (SPDX format)
    pub license: String,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// Source repository URL
    pub repository: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Search keywords
    pub keywords: Vec<String>,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Minimum cleanroom version required
    pub min_cleanroom_version: semver::Version,
    /// Community information
    pub community: CommunityInfo,
    /// Custom metadata fields
    pub custom_fields: HashMap<String, String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        name: impl Into<String>,
        version: impl AsRef<str>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Result<Self> {
        let version = semver::Version::parse(version.as_ref())
            .map_err(|e| CleanroomError::validation_error(format!("Invalid version: {}", e)))?;

        Ok(Self {
            name: name.into(),
            version,
            description: description.into(),
            author: author.into(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            documentation: None,
            keywords: Vec::new(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            min_cleanroom_version: semver::Version::new(0, 3, 0),
            community: CommunityInfo::default(),
            custom_fields: HashMap::new(),
        })
    }

    /// Get primary category from capabilities
    pub fn primary_category(&self) -> Option<&PluginCategory> {
        self.capabilities.first().map(|c| &c.category)
    }

    /// Check version compatibility
    pub fn is_compatible_with(&self, cleanroom_version: &semver::Version) -> bool {
        cleanroom_version >= &self.min_cleanroom_version
    }

    /// Validate metadata completeness
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(CleanroomError::validation_error(
                "Plugin name cannot be empty",
            ));
        }

        if self.description.is_empty() {
            return Err(CleanroomError::validation_error(
                "Plugin description cannot be empty",
            ));
        }

        if self.author.is_empty() {
            return Err(CleanroomError::validation_error(
                "Plugin author cannot be empty",
            ));
        }

        if self.capabilities.is_empty() {
            return Err(CleanroomError::validation_error(
                "Plugin must have at least one capability",
            ));
        }

        // Validate dependencies
        for dep in &self.dependencies {
            semver::VersionReq::parse(&dep.version_constraint).map_err(|e| {
                CleanroomError::validation_error(format!(
                    "Invalid version constraint for dependency '{}': {}",
                    dep.name, e
                ))
            })?;
        }

        Ok(())
    }

    /// Calculate quality score (0-100)
    pub fn quality_score(&self) -> f64 {
        let mut score = 0.0;

        // Documentation presence (25 points)
        if self.homepage.is_some() {
            score += 8.0;
        }
        if self.repository.is_some() {
            score += 8.0;
        }
        if self.documentation.is_some() {
            score += 9.0;
        }

        // Metadata completeness (25 points)
        if !self.keywords.is_empty() {
            score += 8.0;
        }
        if !self.description.is_empty() && self.description.len() > 50 {
            score += 9.0;
        }
        if !self.capabilities.is_empty() {
            score += 8.0;
        }

        // Community engagement (25 points)
        score += (self.community.average_rating / 5.0) * 10.0;
        if self.community.rating_count > 10 {
            score += 8.0;
        }
        if self.community.download_count > 100 {
            score += 7.0;
        }

        // Maintenance (25 points)
        let days_since_update = (chrono::Utc::now() - self.community.updated_at).num_days();
        if days_since_update < 30 {
            score += 13.0;
        } else if days_since_update < 90 {
            score += 9.0;
        } else if days_since_update < 180 {
            score += 5.0;
        }

        if !self.dependencies.is_empty() {
            score += 12.0;
        }

        score.min(100.0)
    }
}

/// Usage statistics for plugin analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUsageStats {
    pub installations: u64,
    pub active_installations: u64,
    pub daily_usage: f64,
    pub peak_usage: u64,
    pub avg_session_duration: f64,
    pub error_rate: f64,
}

impl Default for PluginUsageStats {
    fn default() -> Self {
        Self {
            installations: 0,
            active_installations: 0,
            daily_usage: 0.0,
            peak_usage: 0,
            avg_session_duration: 0.0,
            error_rate: 0.0,
        }
    }
}

/// Performance metrics for plugin evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPerformanceMetrics {
    pub avg_startup_time_ms: f64,
    pub avg_memory_usage_mb: f64,
    pub avg_cpu_usage_percent: f64,
    pub p95_response_time_ms: f64,
    pub reliability_score: f64,
}

impl Default for PluginPerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_startup_time_ms: 0.0,
            avg_memory_usage_mb: 0.0,
            avg_cpu_usage_percent: 0.0,
            p95_response_time_ms: 0.0,
            reliability_score: 100.0,
        }
    }
}

/// Comprehensive plugin statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatistics {
    pub metadata: PluginMetadata,
    pub community: CommunityInfo,
    pub usage_stats: PluginUsageStats,
    pub performance_metrics: PluginPerformanceMetrics,
}

/// Standard capability templates
pub mod standard_capabilities {
    use super::*;

    pub fn database_capability() -> PluginCapability {
        let mut schema = HashMap::new();
        schema.insert("host".to_string(), "string".to_string());
        schema.insert("port".to_string(), "integer".to_string());
        schema.insert("database".to_string(), "string".to_string());

        PluginCapability::new(
            "database",
            PluginCategory::Database,
            "Provides database connectivity and testing capabilities",
        )
        .with_config_schema(schema)
    }

    pub fn cache_capability() -> PluginCapability {
        let mut schema = HashMap::new();
        schema.insert("host".to_string(), "string".to_string());
        schema.insert("port".to_string(), "integer".to_string());

        PluginCapability::new(
            "cache",
            PluginCategory::Cache,
            "Provides caching and session management capabilities",
        )
        .with_config_schema(schema)
    }

    pub fn ai_ml_capability() -> PluginCapability {
        let mut schema = HashMap::new();
        schema.insert("endpoint".to_string(), "string".to_string());
        schema.insert("model".to_string(), "string".to_string());
        schema.insert("api_key".to_string(), "string (optional)".to_string());

        PluginCapability::new(
            "ai_ml",
            PluginCategory::AiMl,
            "Provides AI/ML model testing and validation capabilities",
        )
        .with_config_schema(schema)
    }

    pub fn message_queue_capability() -> PluginCapability {
        let mut schema = HashMap::new();
        schema.insert("broker_url".to_string(), "string".to_string());
        schema.insert("topic".to_string(), "string".to_string());

        PluginCapability::new(
            "message_queue",
            PluginCategory::MessageQueue,
            "Provides message queue and streaming capabilities",
        )
        .with_config_schema(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_creation() -> Result<()> {
        let metadata = PluginMetadata::new("test-plugin", "1.0.0", "A test plugin", "Test Author")?;

        assert_eq!(metadata.name, "test-plugin");
        assert_eq!(metadata.version.to_string(), "1.0.0");
        assert_eq!(metadata.description, "A test plugin");
        assert_eq!(metadata.author, "Test Author");

        Ok(())
    }

    #[test]
    fn test_plugin_validation() -> Result<()> {
        let mut metadata =
            PluginMetadata::new("test-plugin", "1.0.0", "A test plugin", "Test Author")?;

        // Should fail without capabilities
        assert!(metadata.validate().is_err());

        // Add capability and validate again
        metadata
            .capabilities
            .push(standard_capabilities::database_capability());
        assert!(metadata.validate().is_ok());

        Ok(())
    }

    #[test]
    fn test_version_compatibility() -> Result<()> {
        let metadata = PluginMetadata::new("test-plugin", "1.0.0", "A test plugin", "Test Author")?;

        let compatible_version = semver::Version::new(0, 3, 2);
        let incompatible_version = semver::Version::new(0, 2, 0);

        assert!(metadata.is_compatible_with(&compatible_version));
        assert!(!metadata.is_compatible_with(&incompatible_version));

        Ok(())
    }

    #[test]
    fn test_quality_score() -> Result<()> {
        let mut metadata = PluginMetadata::new(
            "test-plugin",
            "1.0.0",
            "A comprehensive test plugin with detailed description",
            "Test Author",
        )?;

        metadata
            .capabilities
            .push(standard_capabilities::database_capability());
        metadata.homepage = Some("https://example.com".to_string());
        metadata.repository = Some("https://github.com/example/plugin".to_string());
        metadata.keywords = vec!["test".to_string(), "plugin".to_string()];

        let score = metadata.quality_score();
        assert!(score > 0.0 && score <= 100.0);

        Ok(())
    }
}
