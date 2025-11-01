//! Container pool manager
//!
//! Manages a pool of pre-allocated containers for efficient stress testing.

use crate::backend::TestcontainerBackend;
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, RwLock};
use tracing::{debug, info, warn};

/// Configuration for container pool
#[derive(Debug, Clone)]
pub struct ContainerPoolConfig {
    /// Maximum number of containers in pool
    pub max_size: usize,

    /// Container startup timeout
    pub startup_timeout: Duration,

    /// Pool cleanup timeout
    pub cleanup_timeout: Duration,

    /// Memory limit per container (MB)
    pub memory_limit: Option<u64>,

    /// CPU limit per container
    pub cpu_limit: Option<f64>,
}

impl Default for ContainerPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            startup_timeout: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(60),
            memory_limit: None,
            cpu_limit: None,
        }
    }
}

/// A pooled container instance
#[derive(Debug, Clone)]
pub struct PooledContainer {
    /// Container image
    pub image: String,

    /// Unique container ID
    pub id: String,

    /// Backend for this container
    pub backend: Arc<TestcontainerBackend>,

    /// Whether container is currently in use
    pub in_use: bool,
}

impl PooledContainer {
    /// Create a new pooled container
    fn new(image: String, backend: TestcontainerBackend) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            image,
            id,
            backend: Arc::new(backend),
            in_use: false,
        }
    }

    /// Mark container as in use
    fn acquire(&mut self) {
        self.in_use = true;
    }

    /// Mark container as available
    fn release(&mut self) {
        self.in_use = false;
    }
}

/// Container pool manager
#[derive(Debug)]
pub struct ContainerPool {
    /// Pool configuration
    config: ContainerPoolConfig,

    /// Containers in the pool, keyed by image name
    pools: Arc<RwLock<HashMap<String, Vec<PooledContainer>>>>,

    /// Semaphore to limit concurrent container allocation
    semaphore: Arc<Semaphore>,

    /// Total containers currently allocated
    allocated_count: Arc<RwLock<usize>>,
}

impl ContainerPool {
    /// Create a new container pool
    pub fn new(config: ContainerPoolConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_size)),
            config,
            pools: Arc::new(RwLock::new(HashMap::new())),
            allocated_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Pre-allocate containers for an image
    ///
    /// # Errors
    ///
    /// Returns error if container creation fails
    pub async fn pre_allocate(&self, image: &str, count: usize) -> Result<()> {
        info!("Pre-allocating {} containers for image: {}", count, image);

        let mut containers = Vec::new();

        for i in 0..count {
            // Acquire semaphore permit
            let _permit = self.semaphore.acquire().await.map_err(|e| {
                CleanroomError::internal_error(format!("Failed to acquire semaphore: {}", e))
            })?;

            // Check if we've hit the pool limit
            let current_count = *self.allocated_count.read().await;
            if current_count >= self.config.max_size {
                warn!(
                    "Reached pool size limit ({}), stopping pre-allocation",
                    self.config.max_size
                );
                break;
            }

            // Create backend
            let mut backend = TestcontainerBackend::new(image)?
                .with_startup_timeout(self.config.startup_timeout);

            if let Some(mem_limit) = self.config.memory_limit {
                backend = backend.with_memory_limit(mem_limit);
            }

            if let Some(cpu_limit) = self.config.cpu_limit {
                backend = backend.with_cpu_limit(cpu_limit);
            }

            let container = PooledContainer::new(image.to_string(), backend);
            containers.push(container);

            // Update allocated count
            let mut count_guard = self.allocated_count.write().await;
            *count_guard += 1;

            debug!("Pre-allocated container {}/{}", i + 1, count);
        }

        // Add to pool
        let mut pools = self.pools.write().await;
        pools
            .entry(image.to_string())
            .or_insert_with(Vec::new)
            .extend(containers);

        info!("Pre-allocation complete for image: {}", image);
        Ok(())
    }

    /// Acquire a container from the pool
    ///
    /// # Errors
    ///
    /// Returns error if no containers are available or creation fails
    pub async fn acquire(&self, image: &str) -> Result<PooledContainer> {
        // Try to get from pool first
        {
            let mut pools = self.pools.write().await;
            if let Some(containers) = pools.get_mut(image) {
                if let Some(container) = containers.iter_mut().find(|c| !c.in_use) {
                    container.acquire();
                    debug!("Acquired container from pool: {}", container.id);
                    return Ok(container.clone());
                }
            }
        }

        // No available container, create new one if under limit
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire semaphore: {}", e))
        })?;

        let current_count = *self.allocated_count.read().await;
        if current_count >= self.config.max_size {
            return Err(CleanroomError::resource_limit_exceeded(format!(
                "Container pool exhausted (max: {})",
                self.config.max_size
            )));
        }

        // Create new container
        let mut backend = TestcontainerBackend::new(image)?
            .with_startup_timeout(self.config.startup_timeout);

        if let Some(mem_limit) = self.config.memory_limit {
            backend = backend.with_memory_limit(mem_limit);
        }

        if let Some(cpu_limit) = self.config.cpu_limit {
            backend = backend.with_cpu_limit(cpu_limit);
        }

        let mut container = PooledContainer::new(image.to_string(), backend);
        container.acquire();

        // Add to pool
        {
            let mut pools = self.pools.write().await;
            pools
                .entry(image.to_string())
                .or_insert_with(Vec::new)
                .push(container.clone());

            let mut count_guard = self.allocated_count.write().await;
            *count_guard += 1;
        }

        info!("Created new container: {}", container.id);
        Ok(container)
    }

    /// Release a container back to the pool
    pub async fn release(&self, container_id: &str) -> Result<()> {
        let mut pools = self.pools.write().await;

        for containers in pools.values_mut() {
            if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
                container.release();
                debug!("Released container back to pool: {}", container_id);
                return Ok(());
            }
        }

        Err(CleanroomError::internal_error(format!(
            "Container not found in pool: {}",
            container_id
        )))
    }

    /// Cleanup all containers in the pool
    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up container pool");

        let mut pools = self.pools.write().await;
        pools.clear();

        let mut count_guard = self.allocated_count.write().await;
        *count_guard = 0;

        info!("Container pool cleanup complete");
        Ok(())
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let pools = self.pools.read().await;
        let allocated = *self.allocated_count.read().await;

        let mut in_use = 0;
        let mut available = 0;

        for containers in pools.values() {
            for container in containers {
                if container.in_use {
                    in_use += 1;
                } else {
                    available += 1;
                }
            }
        }

        PoolStats {
            total_allocated: allocated,
            in_use,
            available,
            max_size: self.config.max_size,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total containers allocated
    pub total_allocated: usize,

    /// Containers currently in use
    pub in_use: usize,

    /// Containers available for use
    pub available: usize,

    /// Maximum pool size
    pub max_size: usize,
}

impl PoolStats {
    /// Get utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.total_allocated as f64 / self.max_size as f64) * 100.0
        }
    }
}
