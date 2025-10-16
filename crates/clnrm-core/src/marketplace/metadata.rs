//! Plugin Metadata Standard
//!
//! Defines the comprehensive metadata structure for Cleanroom plugins,
//! enabling rich discovery, compatibility checking, and community features.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Semantic version for plugin versioning
pub use semver::Version;

/// Plugin metadata - comprehensive information about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub name: String,
    /// Semantic version
    pub version: Version,
    /// Human-readable description
    pub description: String,
    /// Plugin author/maintainer
    pub author: String,
    /// License information
    pub license: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Source repository URL
    pub repository: Option<String>,
    /// Search keywords and tags
    pub keywords: Vec<String>,
    /// Plugin capabilities and features
    pub capabilities: Vec<PluginCapability>,
    /// Runtime dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Framework compatibility information
    pub compatibility: PluginCompatibility,
    /// Plugin documentation
    pub documentation: PluginDocumentation,
    /// Installation and usage instructions
    pub installation: InstallationInfo,
    /// Community information
    pub community: CommunityInfo,
}

impl PluginMetadata {
    /// Create a new plugin metadata with required fields
    pub fn new(
        name: &str,
        version: &str,
        description: &str,
        author: &str,
    ) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            version: Version::parse(version)?,
            description: description.to_string(),
            author: author.to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: Vec::new(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            compatibility: PluginCompatibility::default(),
            documentation: PluginDocumentation::default(),
            installation: InstallationInfo::default(),
            community: CommunityInfo::default(),
        })
    }

    /// Check if plugin is compatible with given framework version
    pub fn is_compatible_with(&self, framework_version: &Version) -> bool {
        self.compatibility.is_compatible(framework_version)
    }

    /// Get primary capability category
    pub fn primary_category(&self) -> Option<&PluginCategory> {
        self.capabilities.first().map(|cap| &cap.category)
    }

    /// Check if plugin supports a specific capability
    pub fn supports_capability(&self, capability: &PluginCapability) -> bool {
        self.capabilities.iter().any(|cap| cap.name == capability.name)
    }

    /// Validate plugin metadata completeness
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }

        if self.description.trim().is_empty() {
            return Err(CleanroomError::validation_error("Plugin description cannot be empty"));
        }

        if self.author.trim().is_empty() {
            return Err(CleanroomError::validation_error("Plugin author cannot be empty"));
        }

        // Validate dependencies
        for dep in &self.dependencies {
            dep.validate()?;
        }

        Ok(())
    }
}

impl fmt::Display for PluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} v{} by {} - {}",
            self.name, self.version, self.author, self.description
        )
    }
}

/// Plugin capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCapability {
    /// Capability name (e.g., "database", "messaging", "monitoring")
    pub name: String,
    /// Capability category
    pub category: PluginCategory,
    /// Capability description
    pub description: String,
    /// Required for plugin operation
    pub required: bool,
    /// Configuration options for this capability
    pub config_schema: Option<HashMap<String, serde_json::Value>>,
}

impl PluginCapability {
    pub fn new(name: &str, category: PluginCategory, description: &str) -> Self {
        Self {
            name: name.to_string(),
            category,
            description: description.to_string(),
            required: false,
            config_schema: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_config_schema(mut self, schema: HashMap<String, serde_json::Value>) -> Self {
        self.config_schema = Some(schema);
        self
    }
}

/// Plugin capability categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginCategory {
    /// Database services (PostgreSQL, MySQL, MongoDB, etc.)
    Database,
    /// Messaging systems (Kafka, RabbitMQ, Redis PubSub)
    Messaging,
    /// API and web services
    Api,
    /// AI/ML services (Ollama, OpenAI, etc.)
    AiMl,
    /// Monitoring and observability
    Monitoring,
    /// Storage systems (S3, MinIO, etc.)
    Storage,
    /// Container and orchestration
    Container,
    /// Network services (DNS, load balancers, etc.)
    Network,
    /// Security and authentication
    Security,
    /// Development tools (debuggers, profilers)
    Development,
    /// Testing utilities and helpers
    Testing,
    /// Custom or specialized services
    Custom,
}

impl fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginCategory::Database => write!(f, "Database"),
            PluginCategory::Messaging => write!(f, "Messaging"),
            PluginCategory::Api => write!(f, "API"),
            PluginCategory::AiMl => write!(f, "AI/ML"),
            PluginCategory::Monitoring => write!(f, "Monitoring"),
            PluginCategory::Storage => write!(f, "Storage"),
            PluginCategory::Container => write!(f, "Container"),
            PluginCategory::Network => write!(f, "Network"),
            PluginCategory::Security => write!(f, "Security"),
            PluginCategory::Development => write!(f, "Development"),
            PluginCategory::Testing => write!(f, "Testing"),
            PluginCategory::Custom => write!(f, "Custom"),
        }
    }
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency name
    pub name: String,
    /// Version constraint (e.g., "^1.2.0", ">=2.0.0")
    pub version_constraint: String,
    /// Dependency type
    pub dependency_type: DependencyType,
    /// Optional description
    pub description: Option<String>,
}

impl PluginDependency {
    pub fn new(name: &str, version_constraint: &str) -> Self {
        Self {
            name: name.to_string(),
            version_constraint: version_constraint.to_string(),
            dependency_type: DependencyType::Runtime,
            description: None,
        }
    }

    pub fn optional(mut self) -> Self {
        self.dependency_type = DependencyType::Optional;
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Dependency name cannot be empty"));
        }

        if self.version_constraint.trim().is_empty() {
            return Err(CleanroomError::validation_error("Dependency version constraint cannot be empty"));
        }

        Ok(())
    }
}

/// Types of plugin dependencies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Required for runtime operation
    Runtime,
    /// Required for development/compilation
    Development,
    /// Optional dependency
    Optional,
    /// Peer dependency (same version as another plugin)
    Peer,
}

/// Framework compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCompatibility {
    /// Minimum framework version required
    pub min_framework_version: Version,
    /// Maximum framework version supported
    pub max_framework_version: Option<Version>,
    /// Supported operating systems
    pub supported_platforms: Vec<String>,
    /// Required CPU architectures
    pub supported_architectures: Vec<String>,
    /// Required runtime features
    pub required_features: Vec<String>,
}

impl Default for PluginCompatibility {
    fn default() -> Self {
        Self {
            min_framework_version: Version::new(0, 1, 0),
            max_framework_version: None,
            supported_platforms: vec!["linux".to_string(), "macos".to_string(), "windows".to_string()],
            supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
            required_features: Vec::new(),
        }
    }
}

impl PluginCompatibility {
    pub fn is_compatible(&self, framework_version: &Version) -> bool {
        if *framework_version < self.min_framework_version {
            return false;
        }

        if let Some(max_version) = &self.max_framework_version {
            if *framework_version > *max_version {
                return false;
            }
        }

        true
    }
}

/// Plugin documentation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginDocumentation {
    /// README content (markdown)
    pub readme: String,
    /// Usage examples
    pub examples: Vec<UsageExample>,
    /// API reference
    pub api_reference: Option<String>,
    /// Changelog
    pub changelog: Option<String>,
    /// Troubleshooting guide
    pub troubleshooting: Option<String>,
}

impl PluginDocumentation {
    pub fn with_readme(mut self, readme: &str) -> Self {
        self.readme = readme.to_string();
        self
    }

    pub fn add_example(mut self, example: UsageExample) -> Self {
        self.examples.push(example);
        self
    }
}

/// Usage example for plugin documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    /// Example title
    pub title: String,
    /// Example description
    pub description: String,
    /// TOML configuration snippet
    pub config: String,
    /// Expected output or behavior
    pub expected_output: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl UsageExample {
    pub fn new(title: &str, description: &str, config: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            config: config.to_string(),
            expected_output: None,
            tags: Vec::new(),
        }
    }

    pub fn with_output(mut self, output: &str) -> Self {
        self.expected_output = Some(output.to_string());
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Installation and setup information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallationInfo {
    /// Installation instructions
    pub instructions: String,
    /// Pre-installation requirements
    pub prerequisites: Vec<String>,
    /// Post-installation steps
    pub post_install: Vec<String>,
    /// Configuration examples
    pub config_examples: Vec<String>,
}

impl InstallationInfo {
    pub fn with_instructions(mut self, instructions: &str) -> Self {
        self.instructions = instructions.to_string();
        self
    }

    pub fn add_prerequisite(mut self, prerequisite: &str) -> Self {
        self.prerequisites.push(prerequisite.to_string());
        self
    }

    pub fn add_post_install_step(mut self, step: &str) -> Self {
        self.post_install.push(step.to_string());
        self
    }
}

/// Community information and statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunityInfo {
    /// Average rating (1-5 stars)
    pub average_rating: f32,
    /// Total number of ratings
    pub rating_count: u32,
    /// Total downloads
    pub download_count: u64,
    /// Creation date
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update date
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// GitHub stars (if applicable)
    pub github_stars: Option<u32>,
    /// Open issues count
    pub open_issues: Option<u32>,
    /// Last commit date
    pub last_commit: Option<chrono::DateTime<chrono::Utc>>,
}

impl CommunityInfo {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            average_rating: 0.0,
            rating_count: 0,
            download_count: 0,
            created_at: now,
            updated_at: now,
            github_stars: None,
            open_issues: None,
            last_commit: None,
        }
    }

    pub fn add_rating(&mut self, rating: u8) {
        let total_rating = self.average_rating * self.rating_count as f32;
        self.rating_count += 1;
        self.average_rating = (total_rating + rating as f32) / self.rating_count as f32;
    }

    pub fn increment_downloads(&mut self) {
        self.download_count += 1;
    }
}

/// Plugin statistics for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatistics {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Community information
    pub community: CommunityInfo,
    /// Usage statistics
    pub usage_stats: UsageStatistics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

impl PluginStatistics {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            community: CommunityInfo::new(),
            usage_stats: UsageStatistics::default(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

/// Usage statistics for plugins
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStatistics {
    /// Total installations
    pub installations: u64,
    /// Active installations (last 30 days)
    pub active_installations: u64,
    /// Average daily usage
    pub daily_usage: f32,
    /// Peak concurrent usage
    pub peak_usage: u64,
    /// Average session duration (seconds)
    pub avg_session_duration: f32,
    /// Error rate percentage
    pub error_rate: f32,
}

impl UsageStatistics {
    pub fn record_usage(&mut self, duration_seconds: f32, had_error: bool) {
        self.daily_usage += 1.0;
        self.avg_session_duration = (self.avg_session_duration + duration_seconds) / 2.0;

        if had_error {
            self.error_rate += 1.0;
        }
    }
}

/// Performance metrics for plugins
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// Average startup time (milliseconds)
    pub avg_startup_time_ms: f32,
    /// Average memory usage (MB)
    pub avg_memory_usage_mb: f32,
    /// Average CPU usage percentage
    pub avg_cpu_usage_percent: f32,
    /// 95th percentile response time (ms)
    pub p95_response_time_ms: f32,
    /// Plugin reliability score (0-100)
    pub reliability_score: f32,
}

impl PerformanceMetrics {
    pub fn record_startup(&mut self, startup_time_ms: f32) {
        self.avg_startup_time_ms = (self.avg_startup_time_ms + startup_time_ms) / 2.0;
    }

    pub fn record_performance(&mut self, memory_mb: f32, cpu_percent: f32, response_time_ms: f32) {
        self.avg_memory_usage_mb = (self.avg_memory_usage_mb + memory_mb) / 2.0;
        self.avg_cpu_usage_percent = (self.avg_cpu_usage_percent + cpu_percent) / 2.0;

        // Update P95 response time (simple rolling average)
        self.p95_response_time_ms = (self.p95_response_time_ms + response_time_ms) / 2.0;
    }
}

/// Standard plugin capabilities for easy categorization
pub mod standard_capabilities {
    use super::*;

    pub fn database_capability() -> PluginCapability {
        PluginCapability::new(
            "database",
            PluginCategory::Database,
            "Provides database connectivity and operations",
        )
    }

    pub fn messaging_capability() -> PluginCapability {
        PluginCapability::new(
            "messaging",
            PluginCategory::Messaging,
            "Provides message queue and event streaming capabilities",
        )
    }

    pub fn api_capability() -> PluginCapability {
        PluginCapability::new(
            "api",
            PluginCategory::Api,
            "Provides HTTP API server and client capabilities",
        )
    }

    pub fn ai_ml_capability() -> PluginCapability {
        PluginCapability::new(
            "ai_ml",
            PluginCategory::AiMl,
            "Provides AI/ML model serving and inference capabilities",
        )
    }

    pub fn monitoring_capability() -> PluginCapability {
        PluginCapability::new(
            "monitoring",
            PluginCategory::Monitoring,
            "Provides observability, metrics, and logging capabilities",
        )
    }

    pub fn storage_capability() -> PluginCapability {
        PluginCapability::new(
            "storage",
            PluginCategory::Storage,
            "Provides object storage, file systems, and data persistence",
        )
    }
}
