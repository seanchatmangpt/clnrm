# Plugin Lifecycle Management

Understanding plugin lifecycle is crucial for creating robust, production-ready plugins. This chapter covers the complete lifecycle from initialization to cleanup.

## Lifecycle Overview

clnrm plugins follow a specific lifecycle:

```
1. Construction → 2. Start → 3. Health Checks → 4. Stop → 5. Cleanup
```

Each phase has specific responsibilities and best practices.

## Lifecycle Phases

### 1. Construction Phase

**Purpose**: Initialize plugin with configuration and validation.

**Responsibilities**:
- Validate input parameters
- Set up internal state
- Prepare configuration
- Initialize resources

**Example**:
```rust
impl MyPlugin {
    pub fn new(name: &str, image: &str) -> Result<Self> {
        // Validate inputs
        if name.is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }
        
        // Parse and validate image
        let (image_name, image_tag) = Self::parse_image(image)?;
        
        // Initialize plugin state
        Ok(Self {
            name: name.to_string(),
            image: image_name,
            tag: image_tag,
            state: PluginState::Initialized,
            container_id: Arc::new(RwLock::new(None)),
        })
    }
    
    fn parse_image(image: &str) -> Result<(String, String)> {
        if image.is_empty() {
            return Err(CleanroomError::validation_error("Image cannot be empty"));
        }
        
        let (name, tag) = if let Some((name, tag)) = image.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image.to_string(), "latest".to_string())
        };
        
        Ok((name, tag))
    }
}
```

### 2. Start Phase

**Purpose**: Start the service and make it available for testing.

**Responsibilities**:
- Create and start container
- Configure service
- Wait for readiness
- Create service handle
- Update state

**Example**:
```rust
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create span for observability
                let _span = tracing::info_span!("plugin_start", plugin = self.name);
                
                // Validate state
                if !matches!(self.state, PluginState::Initialized) {
                    return Err(CleanroomError::internal_error(
                        "Plugin must be in Initialized state to start"
                    ));
                }
                
                // Start container
                let container = self.start_container().await?;
                
                // Wait for service readiness
                self.wait_for_readiness(&container).await?;
                
                // Create service handle
                let handle = self.create_service_handle(&container).await?;
                
                // Update state
                self.state = PluginState::Running;
                
                tracing::info!("Plugin {} started successfully", self.name);
                Ok(handle)
            })
        })
    }
}
```

### 3. Health Check Phase

**Purpose**: Verify service is healthy and ready to handle requests.

**Responsibilities**:
- Check service availability
- Verify configuration
- Test basic functionality
- Return health status

**Example**:
```rust
impl ServicePlugin for MyPlugin {
    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("plugin_health_check", plugin = self.name);
                
                // Get container ID
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;
                
                // Check if container is running
                if !self.is_container_running(container_id).await? {
                    return Ok(HealthStatus::Unhealthy);
                }
                
                // Perform service-specific health check
                match self.check_service_health(handle).await {
                    Ok(_) => Ok(HealthStatus::Healthy),
                    Err(e) => {
                        tracing::warn!("Health check failed: {}", e);
                        Ok(HealthStatus::Unhealthy)
                    }
                }
            })
        })
    }
}
```

### 4. Stop Phase

**Purpose**: Gracefully stop the service and prepare for cleanup.

**Responsibilities**:
- Stop container
- Save any necessary state
- Update metadata
- Change state

**Example**:
```rust
impl ServicePlugin for MyPlugin {
    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("plugin_stop", plugin = self.name);
                
                // Get container ID
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;
                
                // Stop container
                self.stop_container(container_id).await?;
                
                // Clear container ID
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = None;
                }
                
                // Update state
                self.state = PluginState::Stopped;
                
                tracing::info!("Plugin {} stopped successfully", self.name);
                Ok(())
            })
        })
    }
}
```

### 5. Cleanup Phase

**Purpose**: Clean up resources and prepare for destruction.

**Responsibilities**:
- Deallocate resources
- Clear state
- Log cleanup completion
- Prepare for garbage collection

**Example**:
```rust
impl Drop for MyPlugin {
    fn drop(&mut self) {
        // Ensure cleanup happens
        if matches!(self.state, PluginState::Running) {
            tracing::warn!("Plugin {} dropped while running - forcing cleanup", self.name);
            // Force cleanup if needed
        }
        
        tracing::debug!("Plugin {} cleaned up", self.name);
    }
}
```

## State Management

### Plugin States

Define clear states for your plugin:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Initialized,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl PluginState {
    pub fn can_start(&self) -> bool {
        matches!(self, PluginState::Initialized)
    }
    
    pub fn can_stop(&self) -> bool {
        matches!(self, PluginState::Running)
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self, PluginState::Running)
    }
}
```

### State Transitions

Implement safe state transitions:

```rust
impl MyPlugin {
    fn transition_to(&mut self, new_state: PluginState) -> Result<()> {
        let old_state = self.state.clone();
        
        // Validate transition
        match (&old_state, &new_state) {
            (PluginState::Initialized, PluginState::Starting) => Ok(()),
            (PluginState::Starting, PluginState::Running) => Ok(()),
            (PluginState::Running, PluginState::Stopping) => Ok(()),
            (PluginState::Stopping, PluginState::Stopped) => Ok(()),
            (PluginState::Stopped, PluginState::Initialized) => Ok(()),
            (_, PluginState::Failed) => Ok(()), // Can transition to Failed from any state
            _ => Err(CleanroomError::internal_error(
                &format!("Invalid state transition: {:?} -> {:?}", old_state, new_state)
            ))
        }?;
        
        tracing::debug!("State transition: {:?} -> {:?}", old_state, new_state);
        self.state = new_state;
        Ok(())
    }
}
```

## Error Handling in Lifecycle

### Error Types

Define specific error types for lifecycle operations:

```rust
#[derive(Debug)]
pub enum PluginLifecycleError {
    InvalidState(String),
    ContainerStartFailed(String),
    HealthCheckFailed(String),
    ContainerStopFailed(String),
    ResourceAllocationFailed(String),
}

impl From<PluginLifecycleError> for CleanroomError {
    fn from(err: PluginLifecycleError) -> Self {
        match err {
            PluginLifecycleError::InvalidState(msg) => {
                CleanroomError::internal_error(&format!("Invalid state: {}", msg))
            }
            PluginLifecycleError::ContainerStartFailed(msg) => {
                CleanroomError::container_error(&format!("Container start failed: {}", msg))
            }
            PluginLifecycleError::HealthCheckFailed(msg) => {
                CleanroomError::service_error(&format!("Health check failed: {}", msg))
            }
            PluginLifecycleError::ContainerStopFailed(msg) => {
                CleanroomError::container_error(&format!("Container stop failed: {}", msg))
            }
            PluginLifecycleError::ResourceAllocationFailed(msg) => {
                CleanroomError::internal_error(&format!("Resource allocation failed: {}", msg))
            }
        }
    }
}
```

### Error Recovery

Implement error recovery strategies:

```rust
impl MyPlugin {
    async fn start_with_retry(&self, max_retries: u32) -> Result<ServiceHandle> {
        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            tracing::info!("Starting plugin {} (attempt {}/{})", self.name, attempt, max_retries);
            
            match self.start_container().await {
                Ok(container) => {
                    tracing::info!("Plugin {} started successfully on attempt {}", self.name, attempt);
                    return self.create_service_handle(&container).await;
                }
                Err(e) => {
                    last_error = Some(e);
                    tracing::warn!("Attempt {} failed: {}", attempt, last_error.as_ref().unwrap());
                    
                    if attempt < max_retries {
                        let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1));
                        tracing::info!("Retrying in {:?}", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}
```

## Resource Management

### Resource Tracking

Track resources throughout the lifecycle:

```rust
#[derive(Debug)]
pub struct ResourceTracker {
    allocated_resources: Arc<RwLock<Vec<String>>>,
    resource_limits: HashMap<String, u32>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            allocated_resources: Arc::new(RwLock::new(Vec::new())),
            resource_limits: HashMap::new(),
        }
    }
    
    pub async fn allocate_resource(&self, resource_type: &str) -> Result<String> {
        // Check limits
        if let Some(limit) = self.resource_limits.get(resource_type) {
            let current_count = self.count_resources_of_type(resource_type).await;
            if current_count >= *limit {
                return Err(CleanroomError::internal_error(
                    &format!("Resource limit exceeded for {}", resource_type)
                ));
            }
        }
        
        // Allocate resource
        let resource_id = format!("{}_{}", resource_type, uuid::Uuid::new_v4());
        
        {
            let mut resources = self.allocated_resources.write().await;
            resources.push(resource_id.clone());
        }
        
        tracing::debug!("Allocated resource: {}", resource_id);
        Ok(resource_id)
    }
    
    pub async fn deallocate_resource(&self, resource_id: &str) -> Result<()> {
        {
            let mut resources = self.allocated_resources.write().await;
            resources.retain(|id| id != resource_id);
        }
        
        tracing::debug!("Deallocated resource: {}", resource_id);
        Ok(())
    }
    
    async fn count_resources_of_type(&self, resource_type: &str) -> usize {
        let resources = self.allocated_resources.read().await;
        resources.iter()
            .filter(|id| id.starts_with(resource_type))
            .count()
    }
}
```

### Cleanup on Failure

Ensure cleanup happens even on failure:

```rust
impl MyPlugin {
    async fn start_with_cleanup(&self) -> Result<ServiceHandle> {
        let mut allocated_resources = Vec::new();
        
        // Allocate resources
        let container_id = self.resource_tracker.allocate_resource("container").await?;
        allocated_resources.push(container_id.clone());
        
        let port = self.resource_tracker.allocate_resource("port").await?;
        allocated_resources.push(port.clone());
        
        // Start container
        match self.start_container_with_resources(&container_id, &port).await {
            Ok(container) => {
                // Success - create handle
                self.create_service_handle(&container).await
            }
            Err(e) => {
                // Failure - cleanup resources
                tracing::warn!("Container start failed, cleaning up resources: {}", e);
                
                for resource in allocated_resources {
                    if let Err(cleanup_err) = self.resource_tracker.deallocate_resource(&resource).await {
                        tracing::error!("Failed to cleanup resource {}: {}", resource, cleanup_err);
                    }
                }
                
                Err(e)
            }
        }
    }
}
```

## Testing Lifecycle

### Lifecycle Tests

Test each phase of the lifecycle:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_lifecycle_complete() -> Result<(), CleanroomError> {
        // Construction
        let plugin = MyPlugin::new("test-plugin", "alpine:latest")?;
        assert_eq!(plugin.state, PluginState::Initialized);
        
        // Start
        let handle = plugin.start()?;
        assert_eq!(plugin.state, PluginState::Running);
        assert!(handle.metadata.contains_key("container_id"));
        
        // Health check
        let health = plugin.health_check(&handle)?;
        assert_eq!(health, HealthStatus::Healthy);
        
        // Stop
        plugin.stop(handle)?;
        assert_eq!(plugin.state, PluginState::Stopped);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_plugin_state_transitions() -> Result<(), CleanroomError> {
        let mut plugin = MyPlugin::new("test-plugin", "alpine:latest")?;
        
        // Valid transitions
        assert!(plugin.transition_to(PluginState::Starting).is_ok());
        assert!(plugin.transition_to(PluginState::Running).is_ok());
        assert!(plugin.transition_to(PluginState::Stopping).is_ok());
        assert!(plugin.transition_to(PluginState::Stopped).is_ok());
        
        // Invalid transition
        assert!(plugin.transition_to(PluginState::Running).is_err());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_plugin_error_recovery() -> Result<(), CleanroomError> {
        let plugin = MyPlugin::new("test-plugin", "invalid-image")?;
        
        // Should retry and eventually fail
        let result = plugin.start_with_retry(3).await;
        assert!(result.is_err());
        
        Ok(())
    }
}
```

## Best Practices

### 1. Always Validate State

```rust
fn start(&self) -> Result<ServiceHandle> {
    if !self.state.can_start() {
        return Err(CleanroomError::internal_error(
            &format!("Cannot start plugin in state: {:?}", self.state)
        ));
    }
    
    // ... start logic
}
```

### 2. Use Spans for Observability

```rust
fn start(&self) -> Result<ServiceHandle> {
    let _span = tracing::info_span!("plugin_start", plugin = self.name);
    // ... start logic
}
```

### 3. Implement Graceful Shutdown

```rust
fn stop(&self, handle: ServiceHandle) -> Result<()> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // Send shutdown signal
            self.send_shutdown_signal().await?;
            
            // Wait for graceful shutdown
            self.wait_for_shutdown().await?;
            
            // Force stop if needed
            self.force_stop().await?;
            
            Ok(())
        })
    })
}
```

### 4. Track Resources

```rust
impl Drop for MyPlugin {
    fn drop(&mut self) {
        if self.state.is_active() {
            tracing::warn!("Plugin {} dropped while active - forcing cleanup", self.name);
            // Force cleanup
        }
    }
}
```

## Next Steps

Now that you understand plugin lifecycle:

1. **Implement lifecycle management**: Add state tracking to your plugins
2. **Add error recovery**: Implement retry logic and graceful degradation
3. **See real examples**: Check out [Plugin Examples](examples.md)
4. **Learn advanced patterns**: Move on to [Advanced Testing Patterns](../advanced-patterns/README.md)

## Further Reading

- [Plugin Architecture Design](../docs/architecture/plugin_system.md)
- [Error Handling Reference](reference/error-handling.md)
- [Core Team Standards](../CLAUDE.md)
