//! Stress Testing Infrastructure
//!
//! Provides comprehensive stress testing capabilities with:
//! - Combinatorial test generation (permutations of containers × tests × OTEL spans)
//! - Container pool management with pre-allocation
//! - OTEL span stress generation with configurable depth
//! - Parallel execution with resource limits
//! - Graceful degradation and error recovery
//!
//! # Architecture
//!
//! The stress testing infrastructure follows clean architecture principles:
//! - `config`: Configuration structures for stress tests
//! - `permutation`: Combinatorial test generation engine
//! - `pool`: Container pool manager for efficient resource usage
//! - `span_gen`: OTEL span stress generator
//! - `executor`: Parallel test executor with resource management
//! - `metrics`: Metrics collection and reporting
//!
//! # Example
//!
//! ```rust,no_run
//! use clnrm_core::stress_test::{StressTestConfig, StressTestExecutor};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = StressTestConfig::builder()
//!     .with_containers(vec!["alpine:latest", "ubuntu:latest"])
//!     .with_test_count(100)
//!     .with_span_depth(10)
//!     .with_max_containers(20)
//!     .with_max_memory_mb(4096)
//!     .with_concurrency(4)
//!     .build()?;
//!
//! let executor = StressTestExecutor::new(config);
//! let results = executor.run().await?;
//!
//! println!("Completed {} tests with {} failures",
//!     results.total_tests, results.failed_tests);
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod executor;
pub mod metrics;
pub mod permutation;
pub mod pool;
pub mod span_gen;

// Re-export key types
pub use config::{ResourceLimits, StressTestConfig, StressTestConfigBuilder};
pub use executor::{StressTestExecutor, StressTestResult, TestExecution};
pub use metrics::{StressMetrics, StressMetricsCollector};
pub use permutation::{PermutationDimension, PermutationEngine, TestPermutation};
pub use pool::{ContainerPool, ContainerPoolConfig, PooledContainer};
pub use span_gen::{SpanConfig, SpanGenerator, SpanStressProfile};
