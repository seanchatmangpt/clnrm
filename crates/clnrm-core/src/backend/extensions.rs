//! Backend extensions for enhanced capabilities
//!
//! This module provides extension traits for backend capabilities,
//! allowing backends to expose additional functionality beyond the base trait.

use crate::error::Result;
use crate::backend::{Backend, Cmd, RunResult};
use std::collections::HashMap;
use std::time::Duration;

/// Extension trait for backend capabilities
pub trait BackendExt: Backend {
    /// Get backend capabilities
    fn capabilities(&self) -> BackendCapabilities;
    
    /// Check if backend supports a specific capability
    fn supports_capability(&self, capability: &BackendCapability) -> bool;
    
    /// Get backend metadata
    fn metadata(&self) -> BackendMetadata;
    
    /// Get backend health status
    fn health_status(&self) -> BackendHealthStatus;
    
    /// Get backend performance metrics
    fn performance_metrics(&self) -> BackendPerformanceMetrics;
}

/// Backend capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct BackendCapabilities {
    /// Supported capabilities
    pub capabilities: Vec<BackendCapability>,
    /// Maximum concurrent executions
    pub max_concurrent_executions: Option<usize>,
    /// Supported execution modes
    pub execution_modes: Vec<ExecutionMode>,
    /// Supported resource limits
    pub resource_limits: ResourceLimitsSupport,
}

/// Backend capability types
#[derive(Debug, Clone, PartialEq)]
pub enum BackendCapability {
    /// Hermetic execution support
    HermeticExecution,
    /// Deterministic execution support
    DeterministicExecution,
    /// Resource monitoring support
    ResourceMonitoring,
    /// Network isolation support
    NetworkIsolation,
    /// File system isolation support
    FileSystemIsolation,
    /// Process isolation support
    ProcessIsolation,
    /// Custom capability
    Custom(String),
}

/// Execution modes
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// Synchronous execution
    Synchronous,
    /// Asynchronous execution
    Asynchronous,
    /// Batch execution
    Batch,
    /// Streaming execution
    Streaming,
}

/// Resource limits support
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceLimitsSupport {
    /// CPU limits support
    pub cpu_limits: bool,
    /// Memory limits support
    pub memory_limits: bool,
    /// Disk limits support
    pub disk_limits: bool,
    /// Network limits support
    pub network_limits: bool,
    /// Time limits support
    pub time_limits: bool,
}

/// Backend metadata
#[derive(Debug, Clone)]
pub struct BackendMetadata {
    /// Backend name
    pub name: String,
    /// Backend version
    pub version: String,
    /// Backend description
    pub description: String,
    /// Backend vendor
    pub vendor: String,
    /// Backend configuration
    pub configuration: HashMap<String, String>,
}

/// Backend health status
#[derive(Debug, Clone, PartialEq)]
pub enum BackendHealthStatus {
    /// Backend is healthy
    Healthy,
    /// Backend is degraded
    Degraded,
    /// Backend is unhealthy
    Unhealthy,
    /// Backend health is unknown
    Unknown,
}

/// Backend performance metrics
#[derive(Debug, Clone)]
pub struct BackendPerformanceMetrics {
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Peak execution time
    pub peak_execution_time: Duration,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Success rate
    pub success_rate: f64,
    /// Resource usage
    pub resource_usage: ResourceUsage,
}

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Disk usage in bytes
    pub disk_usage_bytes: u64,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Network bytes received
    pub network_bytes_received: u64,
}

/// Enhanced backend trait with additional functionality
pub trait EnhancedBackend: BackendExt {
    /// Execute command with enhanced options
    fn execute_enhanced(&self, cmd: Cmd, options: ExecutionOptions) -> Result<RunResult>;
    
    /// Execute multiple commands in batch
    fn execute_batch(&self, commands: Vec<Cmd>) -> Result<Vec<RunResult>>;
    
    /// Execute command with streaming output
    fn execute_streaming(&self, cmd: Cmd) -> Result<Box<dyn std::io::Read + Send>>;
    
    /// Get backend statistics
    fn get_statistics(&self) -> BackendStatistics;
    
    /// Reset backend statistics
    fn reset_statistics(&mut self);
    
    /// Configure backend
    fn configure(&mut self, config: BackendConfiguration) -> Result<()>;
}

/// Execution options for enhanced backends
#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    /// Timeout for execution
    pub timeout: Option<Duration>,
    /// Resource limits
    pub resource_limits: Option<ResourceLimits>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Working directory
    pub working_directory: Option<std::path::PathBuf>,
    /// Execution mode
    pub execution_mode: ExecutionMode,
    /// Enable resource monitoring
    pub enable_resource_monitoring: bool,
    /// Enable output streaming
    pub enable_output_streaming: bool,
}

/// Resource limits for execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage
    pub max_cpu_usage_percent: f64,
    /// Maximum memory usage in bytes
    pub max_memory_usage_bytes: u64,
    /// Maximum disk usage in bytes
    pub max_disk_usage_bytes: u64,
    /// Maximum network bandwidth in bytes per second
    pub max_network_bandwidth_bytes_per_sec: u64,
    /// Maximum execution time
    pub max_execution_time: Duration,
}

/// Backend configuration
#[derive(Debug, Clone)]
pub struct BackendConfiguration {
    /// Configuration parameters
    pub parameters: HashMap<String, String>,
    /// Feature flags
    pub feature_flags: HashMap<String, bool>,
    /// Resource limits
    pub resource_limits: Option<ResourceLimits>,
    /// Performance tuning
    pub performance_tuning: Option<PerformanceTuning>,
}

/// Performance tuning options
#[derive(Debug, Clone)]
pub struct PerformanceTuning {
    /// Enable performance optimizations
    pub enable_optimizations: bool,
    /// Cache size
    pub cache_size: Option<usize>,
    /// Connection pool size
    pub connection_pool_size: Option<usize>,
    /// Batch size
    pub batch_size: Option<usize>,
}

/// Backend statistics
#[derive(Debug, Clone)]
pub struct BackendStatistics {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Peak execution time
    pub peak_execution_time: Duration,
    /// Total resource usage
    pub total_resource_usage: ResourceUsage,
    /// Backend uptime
    pub uptime: Duration,
    /// Last execution time
    pub last_execution_time: Option<std::time::Instant>,
}

/// Backend capability checker
pub struct BackendCapabilityChecker;

impl BackendCapabilityChecker {
    /// Check if backend supports all required capabilities
    pub fn check_capabilities(
        backend: &dyn BackendExt,
        required_capabilities: &[BackendCapability],
    ) -> Result<()> {
        for capability in required_capabilities {
            if !backend.supports_capability(capability) {
                return Err(crate::error::CleanroomError::internal_error(&format!(
                    "Backend does not support required capability: {:?}",
                    capability
                )));
            }
        }
        Ok(())
    }
    
    /// Get missing capabilities
    pub fn get_missing_capabilities(
        backend: &dyn BackendExt,
        required_capabilities: &[BackendCapability],
    ) -> Vec<BackendCapability> {
        required_capabilities
            .iter()
            .filter(|capability| !backend.supports_capability(capability))
            .cloned()
            .collect()
    }
    
    /// Check if backend meets minimum requirements
    pub fn meets_minimum_requirements(backend: &dyn BackendExt) -> bool {
        let required_capabilities = vec![
            BackendCapability::HermeticExecution,
            BackendCapability::ResourceMonitoring,
        ];
        
        required_capabilities
            .iter()
            .all(|capability| backend.supports_capability(capability))
    }
}

/// Backend performance analyzer
pub struct BackendPerformanceAnalyzer;

impl BackendPerformanceAnalyzer {
    /// Analyze backend performance
    pub fn analyze_performance(backend: &dyn BackendExt) -> PerformanceAnalysis {
        let metrics = backend.performance_metrics();
        
        PerformanceAnalysis {
            overall_score: Self::calculate_overall_score(&metrics),
            performance_trend: Self::analyze_performance_trend(&metrics),
            resource_efficiency: Self::analyze_resource_efficiency(&metrics),
            recommendations: Self::generate_recommendations(&metrics),
        }
    }
    
    /// Calculate overall performance score
    fn calculate_overall_score(metrics: &BackendPerformanceMetrics) -> f64 {
        let success_rate_score = metrics.success_rate * 100.0;
        let efficiency_score = if metrics.avg_execution_time.as_millis() > 0 {
            100.0 / (metrics.avg_execution_time.as_millis() as f64 / 1000.0)
        } else {
            0.0
        };
        
        (success_rate_score + efficiency_score) / 2.0
    }
    
    /// Analyze performance trend
    fn analyze_performance_trend(_metrics: &BackendPerformanceMetrics) -> PerformanceTrend {
        // Simplified trend analysis - in real implementation, this would
        // analyze historical data to determine trend direction
        PerformanceTrend::Stable
    }
    
    /// Analyze resource efficiency
    fn analyze_resource_efficiency(metrics: &BackendPerformanceMetrics) -> ResourceEfficiency {
        ResourceEfficiency {
            cpu_efficiency: if metrics.resource_usage.cpu_usage_percent > 0.0 {
                100.0 / metrics.resource_usage.cpu_usage_percent
            } else {
                0.0
            },
            memory_efficiency: if metrics.resource_usage.memory_usage_bytes > 0 {
                100.0 / (metrics.resource_usage.memory_usage_bytes as f64 / (1024.0 * 1024.0))
            } else {
                0.0
            },
            overall_efficiency: 0.0, // Would be calculated based on all resources
        }
    }
    
    /// Generate performance recommendations
    fn generate_recommendations(metrics: &BackendPerformanceMetrics) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        
        if metrics.success_rate < 0.95 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Reliability,
                priority: RecommendationPriority::High,
                description: "Success rate is below 95%. Consider investigating failure causes.".to_string(),
            });
        }
        
        if metrics.avg_execution_time > Duration::from_secs(30) {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::Medium,
                description: "Average execution time is high. Consider optimization.".to_string(),
            });
        }
        
        if metrics.resource_usage.cpu_usage_percent > 80.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::ResourceUsage,
                priority: RecommendationPriority::High,
                description: "High CPU usage detected. Consider scaling or optimization.".to_string(),
            });
        }
        
        recommendations
    }
}

/// Performance analysis result
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// Overall performance score
    pub overall_score: f64,
    /// Performance trend
    pub performance_trend: PerformanceTrend,
    /// Resource efficiency
    pub resource_efficiency: ResourceEfficiency,
    /// Performance recommendations
    pub recommendations: Vec<PerformanceRecommendation>,
}

/// Performance trend
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    /// Performance is improving
    Improving,
    /// Performance is stable
    Stable,
    /// Performance is degrading
    Degrading,
    /// Performance trend is unknown
    Unknown,
}

/// Resource efficiency metrics
#[derive(Debug, Clone)]
pub struct ResourceEfficiency {
    /// CPU efficiency score
    pub cpu_efficiency: f64,
    /// Memory efficiency score
    pub memory_efficiency: f64,
    /// Overall efficiency score
    pub overall_efficiency: f64,
}

/// Performance recommendation
#[derive(Debug, Clone)]
pub struct PerformanceRecommendation {
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Recommendation description
    pub description: String,
}

/// Recommendation categories
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationCategory {
    /// Performance-related recommendations
    Performance,
    /// Reliability-related recommendations
    Reliability,
    /// Resource usage recommendations
    ResourceUsage,
    /// Security recommendations
    Security,
    /// General recommendations
    General,
}

/// Recommendation priorities
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Convenience function to check backend capabilities
pub fn check_backend_capabilities(
    backend: &dyn BackendExt,
    required_capabilities: &[BackendCapability],
) -> Result<()> {
    BackendCapabilityChecker::check_capabilities(backend, required_capabilities)
}

/// Convenience function to analyze backend performance
pub fn analyze_backend_performance(backend: &dyn BackendExt) -> PerformanceAnalysis {
    BackendPerformanceAnalyzer::analyze_performance(backend)
}
