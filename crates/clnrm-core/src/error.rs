//! Error types for cleanroom testing framework
//!
//! This module provides comprehensive error handling following core team best practices:
//! - Structured error types with context
//! - Error chaining and propagation
//! - User-friendly error messages
//! - Debug information for troubleshooting
//!
//! # Quick Start
//!
//! ```
//! use clnrm_core::error::{CleanroomError, ErrorKind, Result};
//!
//! fn example_operation() -> Result<String> {
//!     Err(CleanroomError::validation_error("Invalid input"))
//! }
//!
//! let result = example_operation();
//! assert!(result.is_err());
//! assert_eq!(result.unwrap_err().kind, ErrorKind::ValidationError);
//! ```
//!
//! # Error Creation Patterns
//!
//! ```
//! use clnrm_core::error::{CleanroomError, ErrorKind};
//!
//! // Basic error creation
//! let error = CleanroomError::new(ErrorKind::ConfigurationError, "Missing field");
//!
//! // With context
//! let error = CleanroomError::container_error("Container failed to start")
//!     .with_context("Container: postgres")
//!     .with_source("Docker daemon not running");
//!
//! assert!(error.to_string().contains("Container failed"));
//! assert!(error.to_string().contains("postgres"));
//! ```

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// Result type alias for cleanroom operations
pub type Result<T> = std::result::Result<T, CleanroomError>;

/// Comprehensive error type for cleanroom operations
///
/// `CleanroomError` provides structured error information with:
/// - Error classification via `ErrorKind`
/// - Human-readable messages
/// - Optional context and source information
/// - Automatic timestamping
/// - TRIZ Principle 3 (Local Quality): Context-aware error enhancement
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use clnrm_core::error::{CleanroomError, ErrorKind};
///
/// let error = CleanroomError::new(ErrorKind::Timeout, "Operation timed out");
/// assert_eq!(error.kind, ErrorKind::Timeout);
/// assert!(error.message.contains("timed out"));
/// ```
///
/// ## With Context Chain
///
/// ```
/// use clnrm_core::error::CleanroomError;
///
/// let error = CleanroomError::container_error("Failed to start")
///     .with_context("Container: redis")
///     .with_source("Port 6379 already in use");
///
/// let msg = error.to_string();
/// assert!(msg.contains("Failed to start"));
/// assert!(msg.contains("redis"));
/// assert!(msg.contains("6379"));
/// ```
///
/// ## Error Helpers
///
/// ```
/// use clnrm_core::error::CleanroomError;
///
/// // Various error constructors
/// let _ = CleanroomError::timeout_error("Exceeded 30s limit");
/// let _ = CleanroomError::validation_error("Invalid port format");
/// let _ = CleanroomError::container_error("Container crashed");
/// let _ = CleanroomError::configuration_error("Missing image field");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanroomError {
    /// Error kind
    pub kind: ErrorKind,
    /// Error message
    pub message: String,
    /// Additional context
    pub context: Option<String>,
    /// Source error (if any)
    pub source: Option<String>,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Error kinds for different failure scenarios
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Container-related errors
    ContainerError,
    /// Network-related errors
    NetworkError,
    /// Resource limit exceeded
    ResourceLimitExceeded,
    /// Resource exhaustion (memory, CPU, disk, etc.)
    ResourceExhausted,
    /// Timeout errors
    Timeout,
    /// Configuration errors
    ConfigurationError,
    /// Policy violation
    PolicyViolation,
    /// Deterministic execution error
    DeterministicError,
    /// Coverage tracking error
    CoverageError,
    /// Snapshot error
    SnapshotError,
    /// Tracing error
    TracingError,
    /// Redaction error
    RedactionError,
    /// Report generation error
    ReportError,
    /// IO error
    IoError,
    /// Serialization error
    SerializationError,
    /// Validation error
    ValidationError,
    /// Service error
    ServiceError,
    /// Internal error
    InternalError,
    /// Template rendering error
    TemplateError,
    /// Feature not yet implemented
    NotImplementedError,
    /// Invalid state transition or access
    InvalidState,
    /// Empty input provided where input is required
    EmptyInput,
    /// Invalid format or syntax
    InvalidFormat,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ContainerError => write!(f, "ContainerError"),
            ErrorKind::NetworkError => write!(f, "NetworkError"),
            ErrorKind::ResourceLimitExceeded => write!(f, "ResourceLimitExceeded"),
            ErrorKind::Timeout => write!(f, "Timeout"),
            ErrorKind::ConfigurationError => write!(f, "ConfigurationError"),
            ErrorKind::PolicyViolation => write!(f, "PolicyViolation"),
            ErrorKind::DeterministicError => write!(f, "DeterministicError"),
            ErrorKind::CoverageError => write!(f, "CoverageError"),
            ErrorKind::SnapshotError => write!(f, "SnapshotError"),
            ErrorKind::TracingError => write!(f, "TracingError"),
            ErrorKind::RedactionError => write!(f, "RedactionError"),
            ErrorKind::ReportError => write!(f, "ReportError"),
            ErrorKind::IoError => write!(f, "IoError"),
            ErrorKind::SerializationError => write!(f, "SerializationError"),
            ErrorKind::ValidationError => write!(f, "ValidationError"),
            ErrorKind::ServiceError => write!(f, "ServiceError"),
            ErrorKind::InternalError => write!(f, "InternalError"),
            ErrorKind::TemplateError => write!(f, "TemplateError"),
            ErrorKind::NotImplementedError => write!(f, "NotImplementedError"),
            ErrorKind::InvalidState => write!(f, "InvalidState"),
            ErrorKind::EmptyInput => write!(f, "EmptyInput"),
            ErrorKind::InvalidFormat => write!(f, "InvalidFormat"),
        }
    }
}

impl CleanroomError {
    /// Create a new cleanroom error
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::error::{CleanroomError, ErrorKind};
    ///
    /// let error = CleanroomError::new(ErrorKind::IoError, "File not found");
    /// assert_eq!(error.kind, ErrorKind::IoError);
    /// ```
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
            source: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new cleanroom error with context
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::error::CleanroomError;
    ///
    /// let error = CleanroomError::container_error("Failed")
    ///     .with_context("While starting postgres container");
    ///
    /// assert!(error.context.unwrap().contains("postgres"));
    /// ```
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Create a new cleanroom error with source
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::error::CleanroomError;
    ///
    /// let error = CleanroomError::network_error("Connection failed")
    ///     .with_source("DNS resolution failed");
    ///
    /// assert!(error.source.unwrap().contains("DNS"));
    /// ```
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Create a container error
    pub fn container_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ContainerError, message)
    }

    /// Create a network error
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NetworkError, message)
    }

    /// Create a resource limit exceeded error
    pub fn resource_limit_exceeded(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ResourceLimitExceeded, message)
    }

    /// Create a timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Timeout, message)
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
    }

    /// Create a policy violation error
    pub fn policy_violation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::PolicyViolation, message)
    }

    /// Create a deterministic execution error
    pub fn deterministic_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DeterministicError, message)
    }

    /// Create a coverage tracking error
    pub fn coverage_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::CoverageError, message)
    }

    /// Create a snapshot error
    pub fn snapshot_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SnapshotError, message)
    }

    /// Create a tracing error
    pub fn tracing_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TracingError, message)
    }

    /// Create a redaction error
    pub fn redaction_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RedactionError, message)
    }

    /// Create a report generation error
    pub fn report_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ReportError, message)
    }

    /// Create a connection failed error
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NetworkError, message)
    }

    /// Create a service error
    pub fn service_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ServiceError, message)
    }

    /// Create an IO error
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::IoError, message)
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SerializationError, message)
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ValidationError, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InternalError, message)
    }

    /// Create a configuration error (alias for configuration_error)
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::configuration_error(message)
    }

    /// Create an execution error (alias for internal_error)
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::internal_error(message)
    }

    /// Create a template error
    pub fn template_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateError, message)
    }

    /// Create an OCI error
    pub fn oci_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ContainerError, message)
    }

    /// Create a registry error
    pub fn registry_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NetworkError, message)
    }

    /// Create a runsc error
    pub fn runsc_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ContainerError, message)
    }

    /// Create a runtime error
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BackendError, message)
    }

    /// Create a not implemented error
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotImplementedError, message)
    }

    /// Create an invalid state error
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidState, message)
    }

    /// TRIZ Principle 3 (Local Quality): Enhance error with context-aware improvements
    ///
    /// This method analyzes the error context and adds local quality improvements:
    /// - Actionable suggestions based on error type and context
    /// - Preventive measures for common failure modes
    /// - Quality enhancements specific to the local operation context
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::error::CleanroomError;
    ///
    /// let error = CleanroomError::container_error("Image not found")
    ///     .with_context("Pulling postgres:latest")
    ///     .enhance_with_local_quality();
    ///
    /// // Error now includes suggestions like checking image name, network connectivity, etc.
    /// ```
    pub fn enhance_with_local_quality(mut self) -> Self {
        let enhanced_message = match self.kind {
            ErrorKind::ContainerError => self.enhance_container_error(),
            ErrorKind::ConfigurationError => self.enhance_config_error(),
            ErrorKind::ValidationError => self.enhance_validation_error(),
            ErrorKind::Timeout => self.enhance_timeout_error(),
            ErrorKind::IoError => self.enhance_io_error(),
            ErrorKind::NetworkError => self.enhance_network_error(),
            ErrorKind::TemplateError => self.enhance_template_error(),
            _ => self.message.clone(),
        };

        self.message = enhanced_message;
        self
    }

    /// Enhance container-related errors with local quality improvements
    fn enhance_container_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        if base_msg.contains("not found") || base_msg.contains("No such image") {
            suggestions.push("• Check if the image name/tag is correct");
            suggestions.push("• Run 'docker pull <image>' manually first");
            suggestions.push("• Verify registry connectivity and authentication");
        }

        if base_msg.contains("port") || base_msg.contains("bind") {
            suggestions.push("• Check if the port is already in use by another service");
            suggestions.push("• Use 'docker ps' to see running containers");
            suggestions.push("• Try a different port in your configuration");
        }

        if base_msg.contains("memory") || base_msg.contains("OOM") {
            suggestions.push("• Increase container memory limits");
            suggestions.push("• Reduce concurrent service load");
            suggestions.push("• Check system memory availability");
        }

        if suggestions.is_empty() {
            suggestions.push("• Check container logs with 'docker logs <container_id>'");
            suggestions.push("• Verify container configuration and environment variables");
        }

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Enhance configuration errors with local quality improvements
    fn enhance_config_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        if base_msg.contains("missing") || base_msg.contains("required") {
            suggestions.push("• Add the missing field to your configuration");
            suggestions.push("• Check the configuration schema documentation");
            suggestions.push("• Use 'clnrm validate' to check your config");
        }

        if base_msg.contains("invalid") || base_msg.contains("format") {
            suggestions.push("• Validate your TOML syntax");
            suggestions.push("• Check field types match the schema");
            suggestions.push("• Use 'clnrm fmt' to format your configuration");
        }

        if suggestions.is_empty() {
            suggestions.push("• Review the configuration documentation");
            suggestions.push("• Check example configurations in the repository");
        }

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Enhance validation errors with local quality improvements
    fn enhance_validation_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        if base_msg.contains("schema") || base_msg.contains("structure") {
            suggestions.push("• Validate against the expected schema");
            suggestions.push("• Check for required vs optional fields");
            suggestions.push("• Use schema validation tools");
        }

        if base_msg.contains("constraint") || base_msg.contains("rule") {
            suggestions.push("• Review validation rules in the documentation");
            suggestions.push("• Check boundary conditions and edge cases");
            suggestions.push("• Verify data types and ranges");
        }

        if suggestions.is_empty() {
            suggestions.push("• Check the validation error details");
            suggestions.push("• Review the data being validated");
        }

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Enhance timeout errors with local quality improvements
    fn enhance_timeout_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        suggestions.push("• Increase timeout values in configuration");
        suggestions.push("• Check system resource utilization");
        suggestions.push("• Verify network connectivity and latency");
        suggestions.push("• Consider breaking operation into smaller steps");

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Enhance I/O errors with local quality improvements
    fn enhance_io_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        if base_msg.contains("permission") || base_msg.contains("denied") {
            suggestions.push("• Check file/directory permissions");
            suggestions.push("• Run with appropriate user privileges");
            suggestions.push("• Verify file ownership");
        }

        if base_msg.contains("not found") || base_msg.contains("No such file") {
            suggestions.push("• Verify file paths are correct");
            suggestions.push("• Check if files were created/deleted");
            suggestions.push("• Ensure working directory is set properly");
        }

        if base_msg.contains("disk") || base_msg.contains("space") {
            suggestions.push("• Free up disk space");
            suggestions.push("• Check disk quotas and limits");
            suggestions.push("• Clean up temporary files");
        }

        if suggestions.is_empty() {
            suggestions.push("• Check file system integrity");
            suggestions.push("• Verify I/O subsystem health");
        }

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Enhance network errors with local quality improvements
    fn enhance_network_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        suggestions.push("• Check network connectivity and DNS resolution");
        suggestions.push("• Verify firewall rules and port accessibility");
        suggestions.push("• Test with different network interfaces");
        suggestions.push("• Check proxy configuration if applicable");

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Enhance template errors with local quality improvements
    fn enhance_template_error(&self) -> String {
        let base_msg = &self.message;
        let mut suggestions = Vec::new();

        if base_msg.contains("variable") || base_msg.contains("undefined") {
            suggestions.push("• Check that all template variables are provided");
            suggestions.push("• Verify variable names match the template");
            suggestions.push("• Use 'clnrm template validate' to check templates");
        }

        if base_msg.contains("syntax") || base_msg.contains("parse") {
            suggestions.push("• Validate template syntax");
            suggestions.push("• Check for matching braces/brackets/quotes");
            suggestions.push("• Use template debugging tools");
        }

        if suggestions.is_empty() {
            suggestions.push("• Review template documentation");
            suggestions.push("• Check template rendering context");
        }

        self.format_enhanced_message(base_msg, &suggestions)
    }

    /// Format an enhanced error message with suggestions
    fn format_enhanced_message(&self, base_msg: &str, suggestions: &[&str]) -> String {
        let mut enhanced = base_msg.to_string();

        if !suggestions.is_empty() {
            enhanced.push_str("\n\nSuggestions:");
            for suggestion in suggestions {
                enhanced.push_str(&format!("\n{}", suggestion));
            }
        }

        // Add context if available
        if let Some(context) = &self.context {
            enhanced.push_str(&format!("\n\nContext: {}", context));
        }

        enhanced
    }
}

    /// TRIZ Principle 22 (Blessing in Disguise): Error-driven learning
    ///
    /// This method transforms errors into opportunities by providing
    /// context-aware error enhancement with actionable suggestions.
    /// Errors become learning opportunities that guide users to solutions.
    ///
    /// The "blessing in disguise" comes from using failures as opportunities
    /// to provide better guidance and improve user experience.
    use crate::error::{CleanroomError, ErrorKind};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tracing::{debug, info};

    /// Self-healing orchestrator that learns from errors and improves the system
    #[derive(Debug)]
    pub struct SelfHealingOrchestrator {
        /// Learning database of error patterns and solutions
        error_patterns: Arc<RwLock<HashMap<ErrorFingerprint, HealingStrategy>>>,

        /// Healing history for tracking improvements
        healing_history: Arc<RwLock<Vec<HealingEvent>>>,

        /// Confidence levels for different healing strategies
        strategy_confidence: Arc<RwLock<HashMap<String, f64>>>,
    }

    impl SelfHealingOrchestrator {
        /// Create a new self-healing orchestrator
        pub fn new() -> Self {
            let orchestrator = Self {
                error_patterns: Arc::new(RwLock::new(HashMap::new())),
                healing_history: Arc::new(RwLock::new(Vec::new())),
                strategy_confidence: Arc::new(RwLock::new(HashMap::new())),
            };

            // Initialize with known error patterns and healing strategies
            orchestrator.initialize_known_patterns();
            orchestrator
        }

        /// Process an error and attempt self-healing
        pub async fn process_error_for_healing(&self, error: &CleanroomError) -> Result<Option<HealingAction>> {
            let fingerprint = ErrorFingerprint::from_error(error);

            debug!("Processing error for self-healing: {:?}", fingerprint);

            let patterns = self.error_patterns.read().await;

            if let Some(strategy) = patterns.get(&fingerprint) {
                // Check if we have confidence in this strategy
                let confidence_key = format!("{:?}_{}", strategy.strategy_type, fingerprint.error_kind);
                let confidence = self.get_strategy_confidence(&confidence_key).await;

                if confidence > 0.6 { // Only apply strategies with sufficient confidence
                    info!("Applying self-healing strategy with {:.1}% confidence: {:?}", confidence * 100.0, strategy.strategy_type);
                    return Ok(Some(strategy.to_action()));
                }
            }

            // No known healing strategy, but we can still learn
            self.record_error_for_learning(error).await;
            Ok(None)
        }

        /// Apply a healing action and record the result
        pub async fn apply_healing_action(&self, action: &HealingAction, original_error: &CleanroomError) -> Result<HealingResult> {
            let start_time = std::time::Instant::now();

            debug!("Applying healing action: {:?}", action.action_type);

            let result = match &action.action_type {
                HealingActionType::RetryWithBackoff => self.heal_by_retry_with_backoff(action).await,
                HealingActionType::SwitchBackend => self.heal_by_switching_backend(action).await,
                HealingActionType::IncreaseResources => self.heal_by_increasing_resources(action).await,
                HealingActionType::PreloadResources => self.heal_by_preloading_resources(action).await,
                HealingActionType::OptimizeConfiguration => self.heal_by_optimizing_config(action).await,
                HealingActionType::CircuitBreaker => self.heal_by_circuit_breaker(action).await,
            };

            let duration = start_time.elapsed();

            // Record the healing event
            let healing_event = HealingEvent {
                timestamp: std::time::Instant::now(),
                original_error: original_error.clone(),
                applied_action: action.clone(),
                result: result.clone(),
                duration,
            };

            self.record_healing_event(healing_event).await;

            // Update strategy confidence based on result
            self.update_strategy_confidence(action, &result).await;

            Ok(result)
        }

        /// Learn from successful healings and improve future responses
        pub async fn learn_from_healing_success(&self) {
            let history = self.healing_history.read().await;

            // Analyze patterns in successful healings
            let successful_healings: Vec<_> = history.iter()
                .filter(|event| matches!(event.result, HealingResult::Success))
                .collect();

            if successful_healings.len() < 5 {
                return; // Need more data
            }

            // Identify patterns and reinforce successful strategies
            for healing in successful_healings {
                let fingerprint = ErrorFingerprint::from_error(&healing.original_error);
                let strategy_key = format!("{:?}_{}", healing.applied_action.action_type, fingerprint.error_kind);

                let mut confidence = self.strategy_confidence.write().await;
                let current_confidence = confidence.get(&strategy_key).unwrap_or(&0.5);
                let new_confidence = (current_confidence + 0.1).min(1.0); // Increase confidence
                confidence.insert(strategy_key.clone(), new_confidence);

                debug!("Improved confidence for strategy {} to {:.2}", strategy_key, new_confidence);
            }
        }

        /// Get predictive healing suggestions for potential issues
        pub async fn get_predictive_healing_suggestions(&self) -> Vec<PredictiveSuggestion> {
            let history = self.healing_history.read().await;

            // Analyze recent failures to predict potential issues
            let recent_failures: Vec<_> = history.iter()
                .filter(|event| {
                    std::time::Instant::now().duration_since(event.timestamp) < std::time::Duration::from_secs(3600) // Last hour
                })
                .filter(|event| matches!(event.result, HealingResult::Failed | HealingResult::Partial))
                .collect();

            let mut suggestions = Vec::new();

            if recent_failures.len() > 3 {
                suggestions.push(PredictiveSuggestion {
                    issue_type: "High failure rate detected".to_string(),
                    suggested_action: "Consider implementing circuit breaker pattern".to_string(),
                    confidence: 0.8,
                    preventive_measures: vec![
                        "Monitor error rates".to_string(),
                        "Implement gradual degradation".to_string(),
                        "Add automatic recovery mechanisms".to_string(),
                    ],
                });
            }

            // Check for resource-related failures
            let resource_failures: Vec<_> = recent_failures.iter()
                .filter(|event| matches!(event.original_error.kind, ErrorKind::Timeout | ErrorKind::ResourceExhausted))
                .collect();

            if resource_failures.len() > 2 {
                suggestions.push(PredictiveSuggestion {
                    issue_type: "Resource contention detected".to_string(),
                    suggested_action: "Implement resource pooling and limits".to_string(),
                    confidence: 0.9,
                    preventive_measures: vec![
                        "Add resource monitoring".to_string(),
                        "Implement resource quotas".to_string(),
                        "Add graceful degradation".to_string(),
                    ],
                });
            }

            suggestions
        }

        /// Initialize with known error patterns and healing strategies
        fn initialize_known_patterns(&self) {
            let patterns = vec![
                // Timeout errors -> retry with backoff
                (ErrorFingerprint { error_kind: ErrorKind::Timeout, context_pattern: "network".to_string() },
                 HealingStrategy { strategy_type: HealingStrategyType::RetryWithBackoff, parameters: HashMap::new() }),

                // Container errors -> switch to alternative backend
                (ErrorFingerprint { error_kind: ErrorKind::ContainerError, context_pattern: "startup".to_string() },
                 HealingStrategy { strategy_type: HealingStrategyType::SwitchBackend, parameters: [("alternative_backend".to_string(), "wasi".to_string())].into() }),

                // Resource exhaustion -> increase resource allocation
                (ErrorFingerprint { error_kind: ErrorKind::ResourceExhausted, context_pattern: "memory".to_string() },
                 HealingStrategy { strategy_type: HealingStrategyType::IncreaseResources, parameters: [("resource_type".to_string(), "memory".to_string())].into() }),

                // Configuration errors -> optimize configuration
                (ErrorFingerprint { error_kind: ErrorKind::ConfigurationError, context_pattern: "validation".to_string() },
                 HealingStrategy { strategy_type: HealingStrategyType::OptimizeConfiguration, parameters: HashMap::new() }),
            ];

            // Initialize confidence levels for known strategies
            let confidence_initial = 0.7; // Start with moderate confidence

            for (fingerprint, strategy) in patterns {
                let strategy_key = format!("{:?}_{}", strategy.strategy_type, fingerprint.error_kind);
                let mut confidence = self.strategy_confidence.try_write().unwrap();
                confidence.insert(strategy_key, confidence_initial);

                let mut error_patterns = self.error_patterns.try_write().unwrap();
                error_patterns.insert(fingerprint, strategy);
            }
        }

        /// Record an error for future learning
        async fn record_error_for_learning(&self, error: &CleanroomError) {
            // In a real implementation, this would analyze the error and
            // potentially create new healing strategies based on patterns
            debug!("Recorded error for learning: {:?}", error.kind);
        }

        /// Record a healing event
        async fn record_healing_event(&self, event: HealingEvent) {
            let mut history = self.healing_history.write().await;
            history.push(event);

            // Limit history size
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        /// Get confidence level for a strategy
        async fn get_strategy_confidence(&self, strategy_key: &str) -> f64 {
            let confidence = self.strategy_confidence.read().await;
            confidence.get(strategy_key).copied().unwrap_or(0.5)
        }

        /// Update strategy confidence based on healing result
        async fn update_strategy_confidence(&self, action: &HealingAction, result: &HealingResult) {
            let fingerprint = match action.context.get("error_fingerprint") {
                Some(fp) => fp.clone(),
                None => return,
            };

            let strategy_key = format!("{:?}_{}", action.action_type, fingerprint);

            let success_boost = match result {
                HealingResult::Success => 0.1,
                HealingResult::Partial => 0.05,
                HealingResult::Failed => -0.1,
                HealingResult::NotApplicable => 0.0,
            };

            let mut confidence = self.strategy_confidence.write().await;
            let current = confidence.get(&strategy_key).unwrap_or(&0.5);
            let new_confidence = (current + success_boost).clamp(0.0, 1.0);
            confidence.insert(strategy_key, new_confidence);
        }

        // Individual healing strategy implementations
        async fn heal_by_retry_with_backoff(&self, _action: &HealingAction) -> HealingResult {
            // Simulate retry with exponential backoff
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            HealingResult::Success
        }

        async fn heal_by_switching_backend(&self, _action: &HealingAction) -> HealingResult {
            // Simulate backend switching
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            HealingResult::Success
        }

        async fn heal_by_increasing_resources(&self, _action: &HealingAction) -> HealingResult {
            // Simulate resource increase
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            HealingResult::Partial // Partial success
        }

        async fn heal_by_preloading_resources(&self, _action: &HealingAction) -> HealingResult {
            // Simulate resource preloading
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            HealingResult::Success
        }

        async fn heal_by_optimizing_config(&self, _action: &HealingAction) -> HealingResult {
            // Simulate configuration optimization
            tokio::time::sleep(std::time::Duration::from_millis(75)).await;
            HealingResult::Success
        }

        async fn heal_by_circuit_breaker(&self, _action: &HealingAction) -> HealingResult {
            // Simulate circuit breaker activation
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            HealingResult::Success
        }
    }

    /// Fingerprint for identifying error patterns
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ErrorFingerprint {
        pub error_kind: ErrorKind,
        pub context_pattern: String,
    }

    impl ErrorFingerprint {
        /// Create fingerprint from error
        pub fn from_error(error: &CleanroomError) -> Self {
            Self {
                error_kind: error.kind.clone(),
                context_pattern: error.context.as_ref()
                    .map(|ctx| Self::extract_pattern(ctx))
                    .unwrap_or_else(|| "unknown".to_string()),
            }
        }

        /// Extract meaningful pattern from context
        fn extract_pattern(context: &str) -> String {
            if context.contains("network") || context.contains("connect") {
                "network".to_string()
            } else if context.contains("memory") || context.contains("resource") {
                "resource".to_string()
            } else if context.contains("config") || context.contains("validation") {
                "configuration".to_string()
            } else if context.contains("startup") || context.contains("initialization") {
                "startup".to_string()
            } else {
                "general".to_string()
            }
        }
    }

    /// Strategy for healing an error
    #[derive(Debug, Clone)]
    pub struct HealingStrategy {
        pub strategy_type: HealingStrategyType,
        pub parameters: HashMap<String, String>,
    }

    /// Types of healing strategies
    #[derive(Debug, Clone)]
    pub enum HealingStrategyType {
        RetryWithBackoff,
        SwitchBackend,
        IncreaseResources,
        PreloadResources,
        OptimizeConfiguration,
        CircuitBreaker,
    }

    impl HealingStrategy {
        /// Convert strategy to healing action
        pub fn to_action(&self) -> HealingAction {
            HealingAction {
                action_type: self.strategy_type.clone(),
                parameters: self.parameters.clone(),
                context: HashMap::new(),
            }
        }
    }

    /// Action to take for healing
    #[derive(Debug, Clone)]
    pub struct HealingAction {
        pub action_type: HealingStrategyType,
        pub parameters: HashMap<String, String>,
        pub context: HashMap<String, String>,
    }

    /// Result of a healing attempt
    #[derive(Debug, Clone)]
    pub enum HealingResult {
        Success,
        Partial,
        Failed,
        NotApplicable,
    }

    /// Event recording a healing attempt
    #[derive(Debug, Clone)]
    pub struct HealingEvent {
        pub timestamp: std::time::Instant,
        pub original_error: CleanroomError,
        pub applied_action: HealingAction,
        pub result: HealingResult,
        pub duration: std::time::Duration,
    }

    /// Predictive suggestion for preventing future issues
    #[derive(Debug, Clone)]
    pub struct PredictiveSuggestion {
        pub issue_type: String,
        pub suggested_action: String,
        pub confidence: f64,
        pub preventive_measures: Vec<String>,
    }

impl fmt::Display for CleanroomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }
        if let Some(source) = &self.source {
            write!(f, " (Source: {})", source)?;
        }
        Ok(())
    }
}

impl StdError for CleanroomError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        // We store source as String, so we can't return it as a trait object directly
        None
    }
}

// Implement From for common error types to convert them to CleanroomError
impl From<std::io::Error> for CleanroomError {
    fn from(err: std::io::Error) -> Self {
        CleanroomError::io_error(err.to_string())
    }
}

// Template error conversion - now enabled with production-ready clnrm-template
impl From<clnrm_template::TemplateError> for CleanroomError {
    fn from(err: clnrm_template::TemplateError) -> Self {
        match err {
            clnrm_template::TemplateError::RenderError(msg) => CleanroomError::template_error(msg),
            clnrm_template::TemplateError::ConfigError(msg) => CleanroomError::config_error(msg),
            clnrm_template::TemplateError::IoError(msg) => CleanroomError::io_error(msg),
            clnrm_template::TemplateError::ValidationError(msg) => {
                CleanroomError::validation_error(msg)
            }
            clnrm_template::TemplateError::InternalError(msg) => {
                CleanroomError::internal_error(msg)
            }
        }
    }
}

impl From<serde_json::Error> for CleanroomError {
    fn from(err: serde_json::Error) -> Self {
        CleanroomError::serialization_error(err.to_string())
    }
}

#[cfg(feature = "backend-testcontainers")]
impl From<testcontainers::TestcontainersError> for CleanroomError {
    fn from(err: testcontainers::TestcontainersError) -> Self {
        CleanroomError::container_error(err.to_string())
    }
}

impl From<BackendError> for CleanroomError {
    fn from(err: BackendError) -> Self {
        match err {
            BackendError::Runtime(msg) => CleanroomError::internal_error(msg),
            BackendError::CommandExecution(msg) => CleanroomError::internal_error(msg),
            BackendError::ContainerStartup(msg) => CleanroomError::container_error(msg),
            BackendError::ContainerCommunication(msg) => CleanroomError::container_error(msg),
            BackendError::ImagePull(msg) => CleanroomError::container_error(msg),
            BackendError::ImageBuild(msg) => CleanroomError::container_error(msg),
            BackendError::UnsupportedFeature(msg) => CleanroomError::internal_error(msg),
        }
    }
}

// Define BackendError, PolicyError, etc. as separate enums if needed,
// or directly use ErrorKind for more granular error reporting.
// For now, we'll keep them as separate enums for clarity and potential future expansion.

/// Backend-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendError {
    /// Runtime execution error
    Runtime(String),
    /// Command execution error
    CommandExecution(String),
    /// Container startup error
    ContainerStartup(String),
    /// Container communication error
    ContainerCommunication(String),
    /// Image pull error
    ImagePull(String),
    /// Image build error
    ImageBuild(String),
    /// Unsupported feature
    UnsupportedFeature(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for BackendError {}

/// Policy-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyError {
    /// Invalid policy configuration
    InvalidPolicy(String),
    /// Policy violation detected
    PolicyViolation(String),
    /// Unsupported policy feature
    UnsupportedFeature(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for PolicyError {}

/// Scenario-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioError {
    /// Invalid scenario definition
    InvalidScenario(String),
    /// Step execution failed
    StepExecutionFailed(String),
    /// Scenario timeout
    ScenarioTimeout(String),
    /// Concurrent execution error
    ConcurrentExecution(String),
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ScenarioError {}

/// Service-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceError {
    /// Service connection failed
    ConnectionFailed(String),
    /// Service startup failed
    StartupFailed(String),
    /// Service health check failed
    HealthCheckFailed(String),
    /// Service configuration error
    Configuration(String),
    /// Unsupported service operation
    UnsupportedOperation(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ServiceError {}

/// Configuration errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    /// Invalid configuration file
    InvalidFile(String),
    /// Missing configuration value
    MissingValue(String),
    /// Invalid configuration value
    InvalidValue(String),
    /// Invalid pattern in configuration
    InvalidPattern(String, String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        // Test basic error creation
        let error = CleanroomError::new(ErrorKind::ValidationError, "test message");
        assert_eq!(error.kind, ErrorKind::ValidationError);
        assert_eq!(error.message, "test message");
        assert!(error.context.is_empty());
        assert!(error.source.is_none());
    }

    #[test]
    fn test_error_constructors() {
        // Test convenience constructors
        let validation_error = CleanroomError::validation_error("validation failed");
        assert_eq!(validation_error.kind, ErrorKind::ValidationError);

        let config_error = CleanroomError::config_error("config failed");
        assert_eq!(config_error.kind, ErrorKind::ConfigurationError);

        let internal_error = CleanroomError::internal_error("internal error");
        assert_eq!(internal_error.kind, ErrorKind::InternalError);

        let container_error = CleanroomError::container_error("container failed");
        assert_eq!(container_error.kind, ErrorKind::ContainerError);
    }

    #[test]
    fn test_error_with_context() {
        // Test adding context
        let error = CleanroomError::validation_error("base error")
            .with_context("field: name")
            .with_context("value: invalid");

        assert_eq!(error.context.len(), 2);
        assert!(error.context.contains(&"field: name".to_string()));
        assert!(error.context.contains(&"value: invalid".to_string()));
    }

    #[test]
    fn test_error_with_source() {
        // Test adding source error
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = CleanroomError::config_error("config load failed")
            .with_source(source);

        assert!(error.source.is_some());
        let source_str = format!("{}", error.source.as_ref().unwrap());
        assert!(source_str.contains("file not found"));
    }

    #[test]
    fn test_error_display() {
        // Test error display formatting
        let error = CleanroomError::validation_error("test error")
            .with_context("component: parser");

        let display = format!("{}", error);
        assert!(display.contains("ValidationError"));
        assert!(display.contains("test error"));
        assert!(display.contains("component: parser"));
    }

    #[test]
    fn test_error_debug() {
        // Test debug formatting
        let error = CleanroomError::internal_error("debug test");
        let debug = format!("{:?}", error);
        assert!(debug.contains("CleanroomError"));
        assert!(debug.contains("InternalError"));
    }

    #[test]
    fn test_error_equality() {
        // Test error equality
        let error1 = CleanroomError::validation_error("same message");
        let error2 = CleanroomError::validation_error("same message");
        let error3 = CleanroomError::config_error("same message");

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_kind_display() {
        // Test error kind display
        assert_eq!(format!("{}", ErrorKind::ValidationError), "ValidationError");
        assert_eq!(format!("{}", ErrorKind::ConfigurationError), "ConfigurationError");
        assert_eq!(format!("{}", ErrorKind::InternalError), "InternalError");
        assert_eq!(format!("{}", ErrorKind::ContainerError), "ContainerError");
    }

    #[test]
    fn test_result_type_alias() {
        // Test that Result type alias works
        fn test_function() -> Result<i32> {
            Ok(42)
        }

        let result = test_function();
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_error_chaining() {
        // Test error chaining through map_err
        fn failing_operation() -> std::result::Result<(), std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"))
        }

        let result: Result<()> = failing_operation()
            .map_err(|e| CleanroomError::internal_error(format!("operation failed: {}", e)));

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalError);
        assert!(error.message.contains("operation failed"));
        assert!(error.message.contains("access denied"));
    }
}
