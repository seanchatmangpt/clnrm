//! Parallel stress test executor
//!
//! Executes stress tests in parallel with resource management and error recovery.

use super::config::StressTestConfig;
use super::metrics::StressMetricsCollector;
use super::permutation::{PermutationEngine, TestPermutation};
use super::pool::{ContainerPool, ContainerPoolConfig};
use super::span_gen::SpanGenerator;
use crate::backend::{Backend, Cmd};
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;
use tracing::{error, info, warn};

/// Result of a stress test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    /// Total number of tests executed
    pub total_tests: usize,

    /// Number of successful tests
    pub passed_tests: usize,

    /// Number of failed tests
    pub failed_tests: usize,

    /// Number of skipped tests (due to resource limits)
    pub skipped_tests: usize,

    /// Total execution time (ms)
    pub total_duration_ms: u64,

    /// Average test execution time (ms)
    pub avg_test_duration_ms: f64,

    /// Peak container pool utilization (%)
    pub peak_pool_utilization: f64,

    /// Total OTEL spans generated
    pub total_spans_generated: usize,

    /// Individual test executions
    pub executions: Vec<TestExecution>,

    /// Error summary
    pub errors: Vec<String>,
}

impl StressTestResult {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed_tests == 0 && self.total_tests > 0
    }
}

/// Single test execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    /// Permutation ID
    pub permutation_id: String,

    /// Container image used
    pub container: String,

    /// Test iteration number
    pub iteration: usize,

    /// Span depth for this test
    pub span_depth: usize,

    /// Execution status
    pub status: ExecutionStatus,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Number of spans generated
    pub spans_generated: usize,

    /// Error message if failed
    pub error: Option<String>,
}

/// Execution status for a test
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Test passed successfully
    Passed,
    /// Test failed
    Failed,
    /// Test was skipped
    Skipped,
    /// Test timed out
    Timeout,
}

/// Stress test executor
pub struct StressTestExecutor {
    config: StressTestConfig,
    pool: Arc<ContainerPool>,
    metrics: Arc<RwLock<StressMetricsCollector>>,
    semaphore: Arc<Semaphore>,
}

impl StressTestExecutor {
    /// Create a new stress test executor
    pub fn new(config: StressTestConfig) -> Self {
        let pool_config = ContainerPoolConfig {
            max_size: config.limits.max_containers,
            startup_timeout: config.limits.container_startup_timeout,
            cleanup_timeout: config.limits.pool_cleanup_timeout,
            memory_limit: Some(config.limits.max_memory_mb / config.limits.max_containers as u64),
            cpu_limit: config.limits.max_cpu_cores.map(|c| c / config.limits.max_containers as f64),
        };

        let pool = Arc::new(ContainerPool::new(pool_config));
        let metrics = Arc::new(RwLock::new(StressMetricsCollector::new()));
        let semaphore = Arc::new(Semaphore::new(config.concurrency));

        Self {
            config,
            pool,
            metrics,
            semaphore,
        }
    }

    /// Run stress tests
    pub async fn run(&self) -> Result<StressTestResult> {
        info!("Starting stress test execution");
        let start_time = Instant::now();

        // Generate test permutations
        let engine = PermutationEngine::new(
            self.config.containers.clone(),
            self.config.test_count,
            self.config.span_depth,
        );

        let permutations = engine.generate()?;
        let total_tests = permutations.len();

        info!(
            "Generated {} test permutations across {} containers",
            total_tests,
            self.config.containers.len()
        );

        // Pre-allocate containers
        for container in &self.config.containers {
            let count = (self.config.limits.max_containers / self.config.containers.len()).max(1);
            if let Err(e) = self.pool.pre_allocate(container, count).await {
                warn!("Failed to pre-allocate containers for {}: {}", container, e);
            }
        }

        // Execute tests in parallel
        let executions = self.execute_parallel(permutations).await?;

        // Cleanup pool
        if let Err(e) = self.pool.cleanup().await {
            error!("Failed to cleanup container pool: {}", e);
        }

        // Calculate results
        let total_duration_ms = start_time.elapsed().as_millis() as u64;

        let passed_tests = executions.iter().filter(|e| e.status == ExecutionStatus::Passed).count();
        let failed_tests = executions.iter().filter(|e| e.status == ExecutionStatus::Failed).count();
        let skipped_tests = executions.iter().filter(|e| e.status == ExecutionStatus::Skipped).count();

        let avg_test_duration_ms = if !executions.is_empty() {
            executions.iter().map(|e| e.duration_ms).sum::<u64>() as f64 / executions.len() as f64
        } else {
            0.0
        };

        let total_spans_generated = executions.iter().map(|e| e.spans_generated).sum();

        let metrics_data = self.metrics.read().await;
        let peak_pool_utilization = metrics_data.peak_pool_utilization();

        let errors = executions
            .iter()
            .filter_map(|e| e.error.clone())
            .collect();

        Ok(StressTestResult {
            total_tests: executions.len(),
            passed_tests,
            failed_tests,
            skipped_tests,
            total_duration_ms,
            avg_test_duration_ms,
            peak_pool_utilization,
            total_spans_generated,
            executions,
            errors,
        })
    }

    /// Execute permutations in parallel
    async fn execute_parallel(&self, permutations: Vec<TestPermutation>) -> Result<Vec<TestExecution>> {
        let mut join_set = JoinSet::new();
        let executions = Arc::new(RwLock::new(Vec::new()));

        for perm in permutations {
            // Acquire semaphore permit to limit concurrency
            let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                CleanroomError::internal_error(format!("Failed to acquire execution permit: {}", e))
            })?;

            let pool = self.pool.clone();
            let metrics = self.metrics.clone();
            let executions_clone = executions.clone();
            let config = self.config.clone();

            join_set.spawn(async move {
                let result = Self::execute_single(perm.clone(), pool, metrics, config).await;

                let mut execs = executions_clone.write().await;
                execs.push(result);

                drop(permit); // Release semaphore
            });
        }

        // Wait for all tasks to complete
        while join_set.join_next().await.is_some() {}

        let final_executions = executions.read().await.clone();
        Ok(final_executions)
    }

    /// Execute a single test permutation
    async fn execute_single(
        perm: TestPermutation,
        pool: Arc<ContainerPool>,
        metrics: Arc<RwLock<StressMetricsCollector>>,
        config: StressTestConfig,
    ) -> TestExecution {
        let start_time = Instant::now();

        // Acquire container from pool
        let container = match pool.acquire(&perm.container).await {
            Ok(c) => c,
            Err(e) => {
                if config.graceful_degradation {
                    warn!("Skipping test due to resource exhaustion: {}", e);
                    return TestExecution {
                        permutation_id: perm.id,
                        container: perm.container,
                        iteration: perm.iteration,
                        span_depth: perm.span_depth,
                        status: ExecutionStatus::Skipped,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        spans_generated: 0,
                        error: Some(format!("Resource exhaustion: {}", e)),
                    };
                } else {
                    return TestExecution {
                        permutation_id: perm.id,
                        container: perm.container,
                        iteration: perm.iteration,
                        span_depth: perm.span_depth,
                        status: ExecutionStatus::Failed,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        spans_generated: 0,
                        error: Some(format!("Failed to acquire container: {}", e)),
                    };
                }
            }
        };

        // Generate OTEL spans
        let mut span_gen = SpanGenerator::new(super::span_gen::SpanConfig {
            max_depth: perm.span_depth,
            spans_per_level: 2,
            add_attributes: true,
            attributes_per_span: 5,
            add_events: true,
            events_per_span: 2,
        });

        let span_stats = match span_gen.generate(&perm.id, &perm.container) {
            Ok(stats) => stats,
            Err(e) => {
                return TestExecution {
                    permutation_id: perm.id,
                    container: perm.container,
                    iteration: perm.iteration,
                    span_depth: perm.span_depth,
                    status: ExecutionStatus::Failed,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    spans_generated: 0,
                    error: Some(format!("Failed to generate spans: {}", e)),
                };
            }
        };

        // Execute test command in container
        let cmd = Cmd::new("echo")
            .arg(format!("Stress test: {}", perm.id));

        let exec_result = container.backend.run_cmd(cmd);

        // Release container back to pool
        let _ = pool.release(&container.id).await;

        // Record metrics
        {
            let mut metrics_guard = metrics.write().await;
            metrics_guard.record_test_execution(start_time.elapsed());

            let pool_stats = pool.stats().await;
            metrics_guard.record_pool_utilization(pool_stats.utilization());
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match exec_result {
            Ok(_) => TestExecution {
                permutation_id: perm.id,
                container: perm.container,
                iteration: perm.iteration,
                span_depth: perm.span_depth,
                status: ExecutionStatus::Passed,
                duration_ms,
                spans_generated: span_stats.total_spans,
                error: None,
            },
            Err(e) => TestExecution {
                permutation_id: perm.id,
                container: perm.container,
                iteration: perm.iteration,
                span_depth: perm.span_depth,
                status: ExecutionStatus::Failed,
                duration_ms,
                spans_generated: span_stats.total_spans,
                error: Some(e.to_string()),
            },
        }
    }
}
