//! Intelligent Service Manager - AI-driven service lifecycle management
//!
//! Provides autonomous service management with:
//! - Auto-scaling based on load prediction
//! - Resource optimization and pooling
//! - Service health prediction
//! - Cost optimization recommendations

use crate::cleanroom::{HealthStatus, ServiceHandle};
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Service metrics for tracking resource usage and performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Service identifier
    pub service_id: String,
    /// Service name
    pub service_name: String,
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// Memory usage in MB
    pub memory_usage: f64,
    /// Network I/O in MB/s
    pub network_io: f64,
    /// Number of active connections
    pub active_connections: u32,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Average response time in milliseconds
    pub response_time_ms: f64,
    /// Error rate (0-1)
    pub error_rate: f64,
    /// Timestamp when metrics were collected
    pub timestamp: u64,
}

impl ServiceMetrics {
    /// Create new service metrics
    pub fn new(service_id: String, service_name: String) -> Self {
        Self {
            service_id,
            service_name,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_io: 0.0,
            active_connections: 0,
            request_rate: 0.0,
            response_time_ms: 0.0,
            error_rate: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Calculate health score (0-100)
    pub fn health_score(&self) -> f64 {
        let cpu_score = (100.0 - self.cpu_usage).max(0.0);
        let memory_score = (100.0 - (self.memory_usage / 10.24)).max(0.0); // Assuming 1GB max
        let error_score = (1.0 - self.error_rate) * 100.0;
        let response_score = (1000.0 / (self.response_time_ms + 1.0)).min(100.0);

        cpu_score * 0.3 + memory_score * 0.3 + error_score * 0.2 + response_score * 0.2
    }
}

/// Historical metrics for load prediction
#[derive(Debug, Clone)]
pub struct MetricsHistory {
    /// Service identifier
    pub service_id: String,
    /// Historical metrics (up to 1000 entries)
    pub history: Vec<ServiceMetrics>,
    /// Maximum history size
    max_size: usize,
}

impl MetricsHistory {
    /// Create new metrics history
    pub fn new(service_id: String) -> Self {
        Self {
            service_id,
            history: Vec::new(),
            max_size: 1000,
        }
    }

    /// Add metrics to history
    pub fn add_metrics(&mut self, metrics: ServiceMetrics) {
        self.history.push(metrics);
        if self.history.len() > self.max_size {
            self.history.remove(0);
        }
    }

    /// Get average metrics over last N entries
    pub fn average_metrics(&self, last_n: usize) -> Option<ServiceMetrics> {
        if self.history.is_empty() {
            return None;
        }

        let n = last_n.min(self.history.len());
        let slice = &self.history[self.history.len() - n..];

        let avg_cpu = slice.iter().map(|m| m.cpu_usage).sum::<f64>() / n as f64;
        let avg_memory = slice.iter().map(|m| m.memory_usage).sum::<f64>() / n as f64;
        let avg_network = slice.iter().map(|m| m.network_io).sum::<f64>() / n as f64;
        let avg_connections = slice.iter().map(|m| m.active_connections).sum::<u32>() / n as u32;
        let avg_request_rate = slice.iter().map(|m| m.request_rate).sum::<f64>() / n as f64;
        let avg_response_time = slice.iter().map(|m| m.response_time_ms).sum::<f64>() / n as f64;
        let avg_error_rate = slice.iter().map(|m| m.error_rate).sum::<f64>() / n as f64;

        Some(ServiceMetrics {
            service_id: self.service_id.clone(),
            service_name: self.history.last()?.service_name.clone(),
            cpu_usage: avg_cpu,
            memory_usage: avg_memory,
            network_io: avg_network,
            active_connections: avg_connections,
            request_rate: avg_request_rate,
            response_time_ms: avg_response_time,
            error_rate: avg_error_rate,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Predict future load using simple moving average
    pub fn predict_load(&self, horizon_minutes: u32) -> Option<ServiceMetrics> {
        if self.history.len() < 3 {
            return None;
        }

        // Use exponential moving average for better prediction
        let alpha = 0.3; // Smoothing factor
        let mut ema_cpu = self.history[0].cpu_usage;
        let mut ema_memory = self.history[0].memory_usage;
        let mut ema_request_rate = self.history[0].request_rate;

        for metrics in &self.history[1..] {
            ema_cpu = alpha * metrics.cpu_usage + (1.0 - alpha) * ema_cpu;
            ema_memory = alpha * metrics.memory_usage + (1.0 - alpha) * ema_memory;
            ema_request_rate = alpha * metrics.request_rate + (1.0 - alpha) * ema_request_rate;
        }

        // Apply trend factor based on recent growth
        let recent = self.average_metrics(5)?;
        let older = self.average_metrics(20)?;

        let cpu_trend = (recent.cpu_usage - older.cpu_usage) / older.cpu_usage.max(1.0);
        let memory_trend = (recent.memory_usage - older.memory_usage) / older.memory_usage.max(1.0);
        let request_trend =
            (recent.request_rate - older.request_rate) / older.request_rate.max(1.0);

        Some(ServiceMetrics {
            service_id: self.service_id.clone(),
            service_name: recent.service_name.clone(),
            cpu_usage: (ema_cpu * (1.0 + cpu_trend)).min(100.0),
            memory_usage: ema_memory * (1.0 + memory_trend),
            network_io: recent.network_io,
            active_connections: recent.active_connections,
            request_rate: ema_request_rate * (1.0 + request_trend),
            response_time_ms: recent.response_time_ms,
            error_rate: recent.error_rate,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + (horizon_minutes as u64 * 60),
        })
    }
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScaleConfig {
    /// Minimum number of instances
    pub min_instances: u32,
    /// Maximum number of instances
    pub max_instances: u32,
    /// CPU threshold for scaling up (percentage)
    pub cpu_scale_up_threshold: f64,
    /// CPU threshold for scaling down (percentage)
    pub cpu_scale_down_threshold: f64,
    /// Memory threshold for scaling up (MB)
    pub memory_scale_up_threshold: f64,
    /// Memory threshold for scaling down (MB)
    pub memory_scale_down_threshold: f64,
    /// Request rate threshold for scaling up (req/s)
    pub request_rate_scale_up_threshold: f64,
    /// Cool-down period between scaling actions (seconds)
    pub cooldown_seconds: u64,
}

impl Default for AutoScaleConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            cpu_scale_up_threshold: 70.0,
            cpu_scale_down_threshold: 30.0,
            memory_scale_up_threshold: 512.0,
            memory_scale_down_threshold: 128.0,
            request_rate_scale_up_threshold: 100.0,
            cooldown_seconds: 60,
        }
    }
}

/// Scaling decision
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingAction {
    /// Scale up by N instances
    ScaleUp(u32),
    /// Scale down by N instances
    ScaleDown(u32),
    /// No scaling needed
    NoAction,
}

/// Resource pool for service reuse
#[derive(Debug, Clone)]
pub struct ResourcePool {
    /// Service name
    pub service_name: String,
    /// Available service instances
    pub available: Vec<ServiceHandle>,
    /// In-use service instances
    pub in_use: Vec<ServiceHandle>,
    /// Maximum pool size
    pub max_size: usize,
}

impl ResourcePool {
    /// Create new resource pool
    pub fn new(service_name: String, max_size: usize) -> Self {
        Self {
            service_name,
            available: Vec::new(),
            in_use: Vec::new(),
            max_size,
        }
    }

    /// Acquire a service from pool
    pub fn acquire(&mut self) -> Option<ServiceHandle> {
        if let Some(handle) = self.available.pop() {
            self.in_use.push(handle.clone());
            Some(handle)
        } else {
            None
        }
    }

    /// Release a service back to pool
    pub fn release(&mut self, handle: ServiceHandle) {
        if let Some(pos) = self.in_use.iter().position(|h| h.id == handle.id) {
            self.in_use.remove(pos);
            if self.available.len() < self.max_size {
                self.available.push(handle);
            }
        }
    }

    /// Get pool utilization (0-1)
    pub fn utilization(&self) -> f64 {
        let total = self.available.len() + self.in_use.len();
        if total == 0 {
            0.0
        } else {
            self.in_use.len() as f64 / total as f64
        }
    }
}

/// Cost optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecommendation {
    /// Service identifier
    pub service_id: String,
    /// Service name
    pub service_name: String,
    /// Recommendation type
    pub recommendation_type: String,
    /// Description
    pub description: String,
    /// Estimated savings (percentage)
    pub estimated_savings: f64,
    /// Priority (1-5, 5 being highest)
    pub priority: u32,
}

/// Intelligent Service Manager
pub struct ServiceManager {
    /// Service metrics history
    metrics_history: HashMap<String, MetricsHistory>,
    /// Auto-scaling configurations
    auto_scale_configs: HashMap<String, AutoScaleConfig>,
    /// Current service instances count
    pub service_instances: HashMap<String, u32>,
    /// Last scaling action timestamp
    last_scaling_action: HashMap<String, u64>,
    /// Resource pools
    pub resource_pools: HashMap<String, ResourcePool>,
}

impl ServiceManager {
    /// Create new service manager
    pub fn new() -> Self {
        Self {
            metrics_history: HashMap::new(),
            auto_scale_configs: HashMap::new(),
            service_instances: HashMap::new(),
            last_scaling_action: HashMap::new(),
            resource_pools: HashMap::new(),
        }
    }

    /// Record service metrics
    pub fn record_metrics(&mut self, metrics: ServiceMetrics) {
        let service_id = metrics.service_id.clone();

        self.metrics_history
            .entry(service_id)
            .or_insert_with(|| MetricsHistory::new(metrics.service_id.clone()))
            .add_metrics(metrics);
    }

    /// Set auto-scaling configuration for a service
    pub fn set_auto_scale_config(&mut self, service_id: String, config: AutoScaleConfig) {
        self.auto_scale_configs.insert(service_id, config);
    }

    /// Predict service load
    pub fn predict_load(&self, service_id: &str, horizon_minutes: u32) -> Option<ServiceMetrics> {
        self.metrics_history
            .get(service_id)
            .and_then(|history| history.predict_load(horizon_minutes))
    }

    /// Determine scaling action based on metrics and prediction
    pub fn determine_scaling_action(&mut self, service_id: &str) -> Result<ScalingAction> {
        let config = self.auto_scale_configs.get(service_id).ok_or_else(|| {
            CleanroomError::internal_error("No auto-scale config found")
                .with_context(format!("Service: {}", service_id))
        })?;

        // Check cooldown period
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(&last_action) = self.last_scaling_action.get(service_id) {
            if now - last_action < config.cooldown_seconds {
                debug!(
                    "Scaling action in cooldown period for service: {}",
                    service_id
                );
                return Ok(ScalingAction::NoAction);
            }
        }

        // Get current and predicted metrics
        let history = self.metrics_history.get(service_id).ok_or_else(|| {
            CleanroomError::internal_error("No metrics history found")
                .with_context(format!("Service: {}", service_id))
        })?;

        let current = history
            .average_metrics(5)
            .ok_or_else(|| CleanroomError::internal_error("Insufficient metrics data"))?;

        let predicted = history.predict_load(5).unwrap_or_else(|| current.clone());

        let current_instances = *self.service_instances.get(service_id).unwrap_or(&1);

        // Determine if scaling is needed based on current + predicted metrics
        let max_cpu = current.cpu_usage.max(predicted.cpu_usage);
        let max_memory = current.memory_usage.max(predicted.memory_usage);
        let max_request_rate = current.request_rate.max(predicted.request_rate);

        if max_cpu > config.cpu_scale_up_threshold
            || max_memory > config.memory_scale_up_threshold
            || max_request_rate > config.request_rate_scale_up_threshold
        {
            if current_instances < config.max_instances {
                let scale_up = ((max_cpu / config.cpu_scale_up_threshold).ceil() as u32)
                    .min(config.max_instances - current_instances);

                self.last_scaling_action.insert(service_id.to_string(), now);
                info!(
                    "Scaling up service {} by {} instances",
                    service_id, scale_up
                );
                return Ok(ScalingAction::ScaleUp(scale_up));
            }
        } else if max_cpu < config.cpu_scale_down_threshold
            && max_memory < config.memory_scale_down_threshold
            && current_instances > config.min_instances
        {
            let scale_down = 1.min(current_instances - config.min_instances);
            self.last_scaling_action.insert(service_id.to_string(), now);
            info!(
                "Scaling down service {} by {} instances",
                service_id, scale_down
            );
            return Ok(ScalingAction::ScaleDown(scale_down));
        }

        Ok(ScalingAction::NoAction)
    }

    /// Update service instance count
    pub fn update_instance_count(&mut self, service_id: String, count: u32) {
        self.service_instances.insert(service_id, count);
    }

    /// Predict service health
    pub fn predict_service_health(&self, service_id: &str) -> Result<HealthStatus> {
        let history = self.metrics_history.get(service_id).ok_or_else(|| {
            CleanroomError::internal_error("No metrics history found")
                .with_context(format!("Service: {}", service_id))
        })?;

        let predicted = history.predict_load(5);

        if let Some(metrics) = predicted {
            let health_score = metrics.health_score();

            if health_score > 70.0 {
                Ok(HealthStatus::Healthy)
            } else if health_score > 40.0 {
                warn!(
                    "Service {} predicted to be degraded (score: {})",
                    service_id, health_score
                );
                Ok(HealthStatus::Unknown)
            } else {
                warn!(
                    "Service {} predicted to be unhealthy (score: {})",
                    service_id, health_score
                );
                Ok(HealthStatus::Unhealthy)
            }
        } else {
            Ok(HealthStatus::Unknown)
        }
    }

    /// Get or create resource pool for a service
    pub fn get_or_create_pool(
        &mut self,
        service_name: String,
        max_size: usize,
    ) -> &mut ResourcePool {
        self.resource_pools
            .entry(service_name.clone())
            .or_insert_with(|| ResourcePool::new(service_name, max_size))
    }

    /// Generate cost optimization recommendations
    pub fn generate_cost_recommendations(&self, service_id: &str) -> Vec<CostRecommendation> {
        let mut recommendations = Vec::new();

        if let Some(history) = self.metrics_history.get(service_id) {
            if let Some(avg_metrics) = history.average_metrics(100) {
                let service_name = avg_metrics.service_name.clone();

                // Check for over-provisioning
                if avg_metrics.cpu_usage < 20.0 && avg_metrics.memory_usage < 100.0 {
                    recommendations.push(CostRecommendation {
                        service_id: service_id.to_string(),
                        service_name: service_name.clone(),
                        recommendation_type: "Downsize".to_string(),
                        description: "Service is significantly under-utilized. Consider reducing instance size.".to_string(),
                        estimated_savings: 30.0,
                        priority: 5,
                    });
                }

                // Check for high error rate
                if avg_metrics.error_rate > 0.05 {
                    recommendations.push(CostRecommendation {
                        service_id: service_id.to_string(),
                        service_name: service_name.clone(),
                        recommendation_type: "Optimize".to_string(),
                        description:
                            "High error rate detected. Investigate and fix to reduce retry costs."
                                .to_string(),
                        estimated_savings: 15.0,
                        priority: 4,
                    });
                }

                // Check for resource pooling opportunity
                if let Some(pool) = self.resource_pools.get(&service_name) {
                    if pool.utilization() < 0.5 && pool.available.len() > 2 {
                        recommendations.push(CostRecommendation {
                            service_id: service_id.to_string(),
                            service_name: service_name.clone(),
                            recommendation_type: "Pool Optimization".to_string(),
                            description:
                                "Resource pool has low utilization. Consider reducing pool size."
                                    .to_string(),
                            estimated_savings: 20.0,
                            priority: 3,
                        });
                    }
                }

                // Check for consistent low load
                if avg_metrics.request_rate < 10.0 {
                    recommendations.push(CostRecommendation {
                        service_id: service_id.to_string(),
                        service_name,
                        recommendation_type: "Serverless Migration".to_string(),
                        description:
                            "Low consistent load. Consider migrating to serverless architecture."
                                .to_string(),
                        estimated_savings: 40.0,
                        priority: 4,
                    });
                }
            }
        }

        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert(
            "total_services".to_string(),
            serde_json::json!(self.metrics_history.len()),
        );

        summary.insert(
            "total_instances".to_string(),
            serde_json::json!(self.service_instances.values().sum::<u32>()),
        );

        summary.insert(
            "total_pools".to_string(),
            serde_json::json!(self.resource_pools.len()),
        );

        let avg_utilization: f64 = self
            .resource_pools
            .values()
            .map(|p| p.utilization())
            .sum::<f64>()
            / self.resource_pools.len().max(1) as f64;

        summary.insert(
            "avg_pool_utilization".to_string(),
            serde_json::json!(format!("{:.1}%", avg_utilization * 100.0)),
        );

        summary
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_metrics_health_score() {
        let mut metrics = ServiceMetrics::new("test-1".to_string(), "test-service".to_string());

        metrics.cpu_usage = 50.0;
        metrics.memory_usage = 256.0;
        metrics.error_rate = 0.01;
        metrics.response_time_ms = 100.0;

        let score = metrics.health_score();
        assert!(score > 50.0 && score < 100.0);
    }

    #[test]
    fn test_metrics_history_average() -> Result<()> {
        let mut history = MetricsHistory::new("test-1".to_string());

        for i in 0..10 {
            let mut metrics = ServiceMetrics::new("test-1".to_string(), "test-service".to_string());
            metrics.cpu_usage = i as f64 * 10.0;
            history.add_metrics(metrics);
        }

        let avg = history.average_metrics(5)
            .ok_or_else(|| CleanroomError::internal_error("No metrics available for average calculation"))?;
        assert!((avg.cpu_usage - 70.0).abs() < 1.0);
        Ok(())
    }

    #[test]
    fn test_load_prediction() -> Result<()> {
        let mut history = MetricsHistory::new("test-1".to_string());

        for i in 0..20 {
            let mut metrics = ServiceMetrics::new("test-1".to_string(), "test-service".to_string());
            metrics.cpu_usage = 50.0 + (i as f64 * 2.0);
            metrics.request_rate = 10.0 + (i as f64);
            history.add_metrics(metrics);
        }

        let predicted = history.predict_load(5);
        assert!(predicted.is_some());
        let pred = predicted
            .ok_or_else(|| CleanroomError::internal_error("Load prediction failed"))?;
        assert!(pred.cpu_usage > 50.0);
        Ok(())
    }

    #[test]
    fn test_auto_scale_decision() -> Result<()> {
        let mut manager = ServiceManager::new();
        let service_id = "test-1";

        manager.set_auto_scale_config(service_id.to_string(), AutoScaleConfig::default());
        manager.update_instance_count(service_id.to_string(), 2);

        // Add metrics indicating high load
        for _ in 0..10 {
            let mut metrics =
                ServiceMetrics::new(service_id.to_string(), "test-service".to_string());
            metrics.cpu_usage = 85.0;
            metrics.memory_usage = 600.0;
            manager.record_metrics(metrics);
        }

        let action = manager.determine_scaling_action(service_id)?;
        assert!(matches!(action, ScalingAction::ScaleUp(_)));
        Ok(())
    }

    #[test]
    fn test_resource_pool() -> Result<()> {
        let mut pool = ResourcePool::new("test-service".to_string(), 5);

        let handle = ServiceHandle {
            id: "handle-1".to_string(),
            service_name: "test-service".to_string(),
            metadata: HashMap::new(),
        };

        pool.available.push(handle.clone());

        let acquired = pool.acquire();
        assert!(acquired.is_some());
        assert_eq!(pool.in_use.len(), 1);
        assert_eq!(pool.available.len(), 0);

        pool.release(acquired
            .ok_or_else(|| CleanroomError::internal_error("Failed to acquire resource from pool"))?);
        assert_eq!(pool.available.len(), 1);
        assert_eq!(pool.in_use.len(), 0);
        Ok(())
    }

    #[test]
    fn test_cost_recommendations() {
        let mut manager = ServiceManager::new();
        let service_id = "test-1";

        // Add metrics indicating under-utilization
        for _ in 0..100 {
            let mut metrics =
                ServiceMetrics::new(service_id.to_string(), "test-service".to_string());
            metrics.cpu_usage = 10.0;
            metrics.memory_usage = 50.0;
            metrics.request_rate = 5.0;
            manager.record_metrics(metrics);
        }

        let recommendations = manager.generate_cost_recommendations(service_id);
        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .any(|r| r.recommendation_type == "Downsize"));
    }

    #[test]
    fn test_health_prediction() -> Result<()> {
        let mut manager = ServiceManager::new();
        let service_id = "test-1";

        for _ in 0..20 {
            let mut metrics =
                ServiceMetrics::new(service_id.to_string(), "test-service".to_string());
            metrics.cpu_usage = 30.0;
            metrics.memory_usage = 200.0;
            metrics.error_rate = 0.001;
            manager.record_metrics(metrics);
        }

        let health = manager.predict_service_health(service_id)?;
        assert_eq!(health, HealthStatus::Healthy);
        Ok(())
    }
}
