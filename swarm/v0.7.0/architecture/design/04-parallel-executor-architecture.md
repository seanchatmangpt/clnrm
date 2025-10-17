# Parallel Executor Architecture - v0.7.0

## Overview

Worker pool-based parallel executor for running multiple test scenarios concurrently with container resource limits, priority queuing, and concurrent OTEL span collection.

## Architecture Components

### 1. ParallelExecutor

```rust
// crates/clnrm-core/src/executor/parallel.rs
pub struct ParallelExecutor {
    worker_pool: WorkerPool,
    scenario_queue: Arc<Mutex<PriorityQueue<ScenarioTask>>>,
    resource_manager: Arc<ResourceManager>,
    span_collector: Arc<SpanCollector>,
}

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Number of worker threads
    pub worker_count: usize,

    /// Maximum concurrent containers
    pub max_concurrent_containers: usize,

    /// Per-container memory limit in MB
    pub container_memory_limit_mb: u64,

    /// Per-container CPU limit (0.0-1.0)
    pub container_cpu_limit: f64,

    /// Enable OTEL span collection
    pub collect_spans: bool,

    /// Timeout for individual scenarios (milliseconds)
    pub scenario_timeout_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get(),
            max_concurrent_containers: 10,
            container_memory_limit_mb: 512,
            container_cpu_limit: 1.0,
            collect_spans: true,
            scenario_timeout_ms: 300_000, // 5 minutes
        }
    }
}
```

### 2. Worker Pool

```rust
// crates/clnrm-core/src/executor/worker_pool.rs
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_tx: mpsc::Sender<ScenarioTask>,
    result_rx: mpsc::Receiver<ExecutionResult>,
}

struct Worker {
    id: usize,
    handle: tokio::task::JoinHandle<()>,
}

impl WorkerPool {
    /// Create new worker pool
    pub fn new(config: ExecutorConfig) -> Result<Self> {
        let (task_tx, task_rx) = mpsc::channel::<ScenarioTask>(1000);
        let (result_tx, result_rx) = mpsc::channel::<ExecutionResult>(1000);

        let task_rx = Arc::new(Mutex::new(task_rx));

        let mut workers = Vec::with_capacity(config.worker_count);

        for id in 0..config.worker_count {
            let worker = Self::spawn_worker(
                id,
                task_rx.clone(),
                result_tx.clone(),
                config.clone(),
            )?;
            workers.push(worker);
        }

        Ok(Self {
            workers,
            task_tx,
            result_rx,
        })
    }

    fn spawn_worker(
        id: usize,
        task_rx: Arc<Mutex<mpsc::Receiver<ScenarioTask>>>,
        result_tx: mpsc::Sender<ExecutionResult>,
        config: ExecutorConfig,
    ) -> Result<Worker> {
        let handle = tokio::spawn(async move {
            tracing::info!("Worker {} started", id);

            loop {
                // Receive task from queue
                let task = {
                    let mut rx = task_rx.lock().await;
                    rx.recv().await
                };

                match task {
                    Some(task) => {
                        tracing::debug!("Worker {} executing scenario: {}", id, task.name);

                        let result = Self::execute_scenario(task, &config).await;

                        if result_tx.send(result).await.is_err() {
                            tracing::error!("Worker {} failed to send result", id);
                            break;
                        }
                    }
                    None => {
                        tracing::info!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });

        Ok(Worker { id, handle })
    }

    async fn execute_scenario(
        task: ScenarioTask,
        config: &ExecutorConfig,
    ) -> ExecutionResult {
        let start = Instant::now();

        // Create span for this scenario execution
        let _span = tracing::info_span!(
            "scenario.execute",
            scenario.name = %task.name,
            scenario.priority = task.priority,
        ).entered();

        // Apply resource limits
        let limits = ResourceLimits {
            memory_mb: config.container_memory_limit_mb,
            cpu_limit: config.container_cpu_limit,
        };

        // Execute scenario with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(config.scenario_timeout_ms),
            Self::run_scenario_with_limits(task.clone(), limits),
        ).await;

        let duration = start.elapsed();

        match result {
            Ok(Ok(output)) => ExecutionResult {
                scenario_name: task.name,
                success: true,
                duration,
                output,
                error: None,
            },
            Ok(Err(e)) => ExecutionResult {
                scenario_name: task.name,
                success: false,
                duration,
                output: String::new(),
                error: Some(e.to_string()),
            },
            Err(_) => ExecutionResult {
                scenario_name: task.name,
                success: false,
                duration,
                output: String::new(),
                error: Some(format!(
                    "Scenario timeout after {}ms",
                    config.scenario_timeout_ms
                )),
            },
        }
    }

    async fn run_scenario_with_limits(
        task: ScenarioTask,
        limits: ResourceLimits,
    ) -> Result<String> {
        // Create cleanroom environment with limits
        let mut env = CleanroomEnvironment::new().await?;

        // Apply resource limits to all containers
        env.set_resource_limits(limits)?;

        // Execute scenario
        env.execute_scenario(&task.config).await
    }

    /// Submit task to worker pool
    pub async fn submit(&self, task: ScenarioTask) -> Result<()> {
        self.task_tx.send(task).await
            .map_err(|e| CleanroomError::internal_error(
                format!("Failed to submit task: {}", e)
            ))
    }

    /// Receive execution result
    pub async fn receive_result(&mut self) -> Option<ExecutionResult> {
        self.result_rx.recv().await
    }

    /// Shutdown worker pool gracefully
    pub async fn shutdown(self) -> Result<()> {
        // Drop task sender to signal workers to stop
        drop(self.task_tx);

        // Wait for all workers to finish
        for worker in self.workers {
            worker.handle.await
                .map_err(|e| CleanroomError::internal_error(
                    format!("Worker join failed: {}", e)
                ))?;
        }

        Ok(())
    }
}
```

### 3. Priority Queue

```rust
// crates/clnrm-core/src/executor/priority_queue.rs
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct ScenarioTask {
    pub name: String,
    pub config: ScenarioConfig,
    pub priority: Priority,
    pub submitted_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High = 3,
    Normal = 2,
    Low = 1,
}

impl Ord for ScenarioTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        match (self.priority as u8).cmp(&(other.priority as u8)) {
            Ordering::Equal => {
                // Same priority: FIFO (earlier submitted first)
                self.submitted_at.cmp(&other.submitted_at)
            }
            other => other,
        }
    }
}

impl PartialOrd for ScenarioTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct PriorityQueue<T> {
    heap: BinaryHeap<T>,
}

impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.heap.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
```

### 4. Resource Manager

```rust
// crates/clnrm-core/src/executor/resource_manager.rs
pub struct ResourceManager {
    max_containers: usize,
    active_containers: Arc<AtomicUsize>,
    memory_tracker: Arc<Mutex<MemoryTracker>>,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub memory_mb: u64,
    pub cpu_limit: f64,
}

impl ResourceManager {
    pub fn new(max_containers: usize) -> Self {
        Self {
            max_containers,
            active_containers: Arc::new(AtomicUsize::new(0)),
            memory_tracker: Arc::new(Mutex::new(MemoryTracker::new())),
        }
    }

    /// Acquire container slot (blocks if limit reached)
    pub async fn acquire_container_slot(&self) -> Result<ContainerSlot> {
        loop {
            let current = self.active_containers.load(Ordering::SeqCst);

            if current < self.max_containers {
                // Try to increment
                if self.active_containers.compare_exchange(
                    current,
                    current + 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ).is_ok() {
                    return Ok(ContainerSlot {
                        manager: self.active_containers.clone(),
                    });
                }
            } else {
                // Wait for a slot to free up
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    /// Get current container count
    pub fn active_count(&self) -> usize {
        self.active_containers.load(Ordering::SeqCst)
    }
}

pub struct ContainerSlot {
    manager: Arc<AtomicUsize>,
}

impl Drop for ContainerSlot {
    fn drop(&mut self) {
        // Release slot
        self.manager.fetch_sub(1, Ordering::SeqCst);
    }
}

struct MemoryTracker {
    allocated_mb: HashMap<String, u64>,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            allocated_mb: HashMap::new(),
        }
    }

    fn allocate(&mut self, container_id: String, memory_mb: u64) {
        self.allocated_mb.insert(container_id, memory_mb);
    }

    fn deallocate(&mut self, container_id: &str) {
        self.allocated_mb.remove(container_id);
    }

    fn total_allocated(&self) -> u64 {
        self.allocated_mb.values().sum()
    }
}
```

### 5. Span Collector

Concurrent OTEL span collection from parallel executions.

```rust
// crates/clnrm-core/src/executor/span_collector.rs
#[cfg(feature = "otel-traces")]
pub struct SpanCollector {
    spans: Arc<Mutex<Vec<CollectedSpan>>>,
}

#[cfg(feature = "otel-traces")]
#[derive(Debug, Clone)]
pub struct CollectedSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub scenario_name: String,
}

#[cfg(feature = "otel-traces")]
impl SpanCollector {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Collect span from scenario execution
    pub async fn collect_span(&self, span: CollectedSpan) -> Result<()> {
        let mut spans = self.spans.lock().await;
        spans.push(span);
        Ok(())
    }

    /// Get all collected spans
    pub async fn get_spans(&self) -> Vec<CollectedSpan> {
        let spans = self.spans.lock().await;
        spans.clone()
    }

    /// Get spans for specific scenario
    pub async fn get_scenario_spans(&self, scenario_name: &str) -> Vec<CollectedSpan> {
        let spans = self.spans.lock().await;
        spans.iter()
            .filter(|s| s.scenario_name == scenario_name)
            .cloned()
            .collect()
    }

    /// Clear all collected spans
    pub async fn clear(&self) {
        let mut spans = self.spans.lock().await;
        spans.clear();
    }
}
```

## Data Flow

```
Test Suite
    ↓
Parse Scenarios
    ↓
Create ScenarioTasks (with priority)
    ↓
Priority Queue
    ↓
┌─────────────────────────────────────┐
│ Worker Pool (N workers)             │
│  ┌──────────┐  ┌──────────┐        │
│  │ Worker 1 │  │ Worker 2 │  ...   │
│  └────┬─────┘  └────┬─────┘        │
│       │             │               │
│       ▼             ▼               │
│  Container     Container            │
│  (Limited)     (Limited)            │
└───────┬─────────────┬───────────────┘
        │             │
        ▼             ▼
    Span Collector (Concurrent)
        │
        ▼
    Execution Results
```

## Performance Optimizations

### 1. Work Stealing

```rust
impl WorkerPool {
    /// Enable work stealing between workers
    pub fn enable_work_stealing(&mut self) {
        // Workers can steal tasks from each other's local queues
        // when their own queue is empty
        unimplemented!("Work stealing optimization")
    }
}
```

### 2. Container Pooling

```rust
pub struct ContainerPool {
    idle_containers: Arc<Mutex<Vec<Container>>>,
    max_pool_size: usize,
}

impl ContainerPool {
    /// Reuse containers across scenarios
    pub async fn acquire_or_create(&self, image: &str) -> Result<Container> {
        let mut pool = self.idle_containers.lock().await;

        // Try to reuse existing container
        if let Some(container) = pool.pop() {
            if container.image() == image {
                return Ok(container);
            }
        }

        // Create new container
        Container::new(image).await
    }

    /// Return container to pool
    pub async fn release(&self, container: Container) -> Result<()> {
        let mut pool = self.idle_containers.lock().await;

        if pool.len() < self.max_pool_size {
            pool.push(container);
        } else {
            // Pool full, drop container
            drop(container);
        }

        Ok(())
    }
}
```

### 3. Adaptive Worker Scaling

```rust
impl WorkerPool {
    /// Dynamically adjust worker count based on queue depth
    pub async fn auto_scale(&mut self, queue_depth: usize) -> Result<()> {
        let target_workers = if queue_depth > 100 {
            self.workers.len() * 2 // Scale up
        } else if queue_depth < 10 && self.workers.len() > 4 {
            self.workers.len() / 2 // Scale down
        } else {
            return Ok(()); // No change
        };

        let target_workers = target_workers.clamp(4, 32);

        if target_workers > self.workers.len() {
            // Spawn more workers
            self.scale_up(target_workers - self.workers.len()).await?;
        } else if target_workers < self.workers.len() {
            // Remove workers
            self.scale_down(self.workers.len() - target_workers).await?;
        }

        Ok(())
    }
}
```

## CLI Integration

```bash
# Parallel execution (default)
clnrm run --parallel tests/

# Custom worker count
clnrm run --workers 8 tests/

# Container resource limits
clnrm run --memory 1024 --cpu 2.0 tests/

# Priority execution
clnrm run --priority high tests/critical/
clnrm run --priority low tests/optional/

# Monitor execution
clnrm run --show-workers tests/
```

## Configuration

```toml
# .clnrm/config.toml
[executor]
enabled = true
worker_count = 8
max_concurrent_containers = 20

[executor.limits]
container_memory_mb = 512
container_cpu_limit = 1.0

[executor.priorities]
critical = "high"
integration = "normal"
smoke = "low"
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_execution_completes() -> Result<()> {
        // Arrange
        let config = ExecutorConfig::default();
        let executor = ParallelExecutor::new(config)?;

        let tasks = (0..10).map(|i| ScenarioTask {
            name: format!("scenario_{}", i),
            config: create_test_scenario(),
            priority: Priority::Normal,
            submitted_at: Instant::now(),
        }).collect::<Vec<_>>();

        // Act
        let results = executor.execute_all(tasks).await?;

        // Assert
        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|r| r.success));

        Ok(())
    }

    #[tokio::test]
    async fn test_resource_limits_enforced() -> Result<()> {
        // Arrange
        let config = ExecutorConfig {
            max_concurrent_containers: 2,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config)?;

        // Act
        let slot1 = executor.resource_manager.acquire_container_slot().await?;
        let slot2 = executor.resource_manager.acquire_container_slot().await?;

        // Assert
        assert_eq!(executor.resource_manager.active_count(), 2);

        // Acquiring 3rd should block (test with timeout)
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            executor.resource_manager.acquire_container_slot()
        ).await;

        assert!(result.is_err()); // Timeout because slot not available

        drop(slot1); // Release one slot
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now should succeed
        let slot3 = executor.resource_manager.acquire_container_slot().await?;
        assert_eq!(executor.resource_manager.active_count(), 2);

        Ok(())
    }
}
```

## Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
num_cpus = "1.16"
futures = "0.3"
tracing = "0.1"
```

## Future Enhancements

1. **GPU Support**: Allocate GPU resources for AI workloads
2. **Distributed Execution**: Run scenarios across multiple machines
3. **Smart Scheduling**: ML-based task priority prediction
4. **Cost Optimization**: Minimize cloud resource costs
