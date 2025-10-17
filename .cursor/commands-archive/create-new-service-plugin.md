# Create New Service Plugin

Guide for implementing a new service plugin following clnrm's plugin architecture and Core Team Standards.

## Plugin Architecture

Service plugins implement the `ServicePlugin` trait to provide hermetic container-based services for testing.

## Steps to Create a Plugin

### 1. Define Your Service

Determine:
- **Service type**: Database, cache, message queue, API, etc.
- **Docker image**: Official image to use
- **Configuration**: Required environment variables, ports, volumes
- **Health check**: How to verify service is ready
- **Cleanup**: What resources need cleanup

### 2. Create Plugin File

Create `crates/clnrm-core/src/services/your_service.rs`:

```rust
use crate::cleanroom::{ServicePlugin, ServiceHandle};
use crate::error::{CleanroomError, Result};
use async_trait::async_trait;

pub struct YourServicePlugin {
    name: String,
    image: String,
    // Service-specific configuration
}

impl YourServicePlugin {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: "your-service:latest".to_string(),
        }
    }

    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }
}

#[async_trait]
impl ServicePlugin for YourServicePlugin {
    fn service_type(&self) -> &str {
        "your_service"
    }

    async fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for dyn compatibility
        let handle = tokio::task::block_in_place(|| {
            // 1. Configure container
            // 2. Set environment variables
            // 3. Map ports
            // 4. Start container
            // 5. Return ServiceHandle
        });

        handle.map_err(|e| CleanroomError::service_error(
            format!("Failed to start {}: {}", self.name, e)
        ))
    }

    async fn stop(&self, handle: &ServiceHandle) -> Result<()> {
        // Cleanup resources
        Ok(())
    }

    async fn health_check(&self, handle: &ServiceHandle) -> Result<bool> {
        // Check if service is ready
        // Examples:
        // - TCP port check
        // - HTTP health endpoint
        // - Database connection test
        Ok(true)
    }
}
```

### 3. Core Team Standards Compliance

**CRITICAL**: Follow these standards:

#### Error Handling
```rust
// ❌ WRONG - will panic
let result = operation().unwrap();

// ✅ CORRECT - proper error handling
let result = operation().map_err(|e| {
    CleanroomError::service_error(format!("Operation failed: {}", e))
})?;
```

#### Async/Sync Rules
```rust
// ❌ WRONG - breaks dyn compatibility
#[async_trait]
pub trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>; // FORBIDDEN
}

// ✅ CORRECT - use sync methods with block_in_place
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>; // OK
    // Use tokio::task::block_in_place internally for async ops
}
```

#### No False Positives
```rust
// ❌ WRONG - lying about success
pub fn health_check(&self) -> Result<bool> {
    println!("Health check executed");
    Ok(true)  // Didn't actually check!
}

// ✅ CORRECT - honest about incompleteness
pub fn health_check(&self) -> Result<bool> {
    unimplemented!("health_check: needs actual health verification")
}
```

### 4. Register Plugin

Add to `crates/clnrm-core/src/services/mod.rs`:

```rust
pub mod your_service;
pub use your_service::YourServicePlugin;
```

### 5. Create Integration Test

Create `crates/clnrm-core/tests/integration/your_service_test.rs`:

```rust
use clnrm_core::{CleanroomEnvironment, YourServicePlugin};
use clnrm_core::error::Result;

#[tokio::test]
async fn test_your_service_starts_successfully() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(YourServicePlugin::new("test-service"));

    // Act
    env.register_service(plugin).await?;
    let handle = env.start_service("test-service").await?;

    // Assert
    assert!(handle.is_running());
    Ok(())
}

#[tokio::test]
async fn test_your_service_health_check_passes() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(YourServicePlugin::new("test-service"));
    env.register_service(plugin).await?;
    let handle = env.start_service("test-service").await?;

    // Act
    let healthy = env.check_service_health(&handle).await?;

    // Assert
    assert!(healthy, "Service should be healthy after startup");
    Ok(())
}
```

### 6. Add TOML Configuration Support

Create example in `tests/your-service-example.clnrm.toml`:

```toml
[test.metadata]
name = "your_service_integration_test"
description = "Test YourService plugin functionality"

[services.your_service]
type = "your_service"
image = "your-service:latest"
environment = { KEY = "value" }
ports = { "8080" = "8080" }

[[steps]]
name = "verify_service"
command = ["curl", "http://localhost:8080/health"]
expected_output_regex = "OK"
service = "your_service"
```

### 7. Update Documentation

Add to `docs/CLI_GUIDE.md`:

```markdown
### YourService Plugin

Provides YourService integration for testing.

**Usage**:
```rust
let plugin = YourServicePlugin::new("my-service")
    .with_image("your-service:2.0")
    .with_port(8080);
```

**Configuration**:
- Image: `your-service:latest` (default)
- Port: 8080
- Health check: HTTP GET `/health`
```

### 8. Add to CLI Plugins List

Update `src/cli/commands/plugins.rs`:

```rust
println!("- your_service - YourService database/cache/etc.");
```

## Testing Checklist

- [ ] Unit tests pass: `cargo test -p clnrm-core --lib`
- [ ] Integration tests pass: `cargo test --test integration`
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] Proper `Result<T, CleanroomError>` error handling
- [ ] AAA pattern in all tests
- [ ] Health check actually verifies service readiness
- [ ] Cleanup properly handles container removal
- [ ] Documentation updated
- [ ] TOML example provided
- [ ] Self-test validates plugin: `clnrm self-test`

## Quality Gates

Before merging, run:

```bash
# Quality gates
bash scripts/ci-gate.sh

# Fake code scanner
bash scripts/scan-fakes.sh

# Clippy
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

## Examples to Reference

Study existing plugins:
- **Generic**: `src/services/generic.rs` - Simple container wrapper
- **SurrealDB**: `src/services/surrealdb.rs` - Database with health check
- **Ollama**: `src/services/ollama.rs` - LLM service with model loading
- **Chaos**: `src/services/chaos_engine.rs` - Advanced service manipulation

## Common Patterns

### Port Mapping
```rust
use testcontainers::core::WaitFor;

container
    .with_mapped_port(8080, 8080)
    .with_wait_for(WaitFor::message_on_stdout("ready"))
```

### Environment Variables
```rust
container.with_env_var("DATABASE_URL", "postgres://...")
```

### Health Check with Retry
```rust
for attempt in 0..30 {
    if tcp_check(&handle.port).await? {
        return Ok(true);
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
}
Ok(false)
```

## Documentation

- Plugin architecture: `README.md`
- Error handling: `.cursorrules`
- Testing guide: `docs/TESTING.md`
- TOML reference: `docs/TOML_REFERENCE.md`
