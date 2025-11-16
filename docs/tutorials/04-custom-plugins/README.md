# Tutorial 4: Custom Plugins (20 minutes)

**⏱ Estimated Time**: 20 minutes
**📋 Prerequisites**: Completed Tutorial 1, basic Rust knowledge
**🎯 Learning Objectives**: Create a custom service plugin to extend clnrm

## What You'll Learn

By the end of this tutorial, you'll:
- ✅ Understand how plugins work
- ✅ Implement the ServicePlugin trait
- ✅ Create a simple custom plugin
- ✅ Register and use your plugin
- ✅ Test your plugin works

---

## The Plugin System: Extensibility Without Hardcoding

clnrm uses plugins to support different services (databases, APIs, LLMs, etc.) without hardcoding them.

### Why Plugins?

**Without plugins** (hardcoded):
```
clnrm binary includes:
  - PostgreSQL support (code)
  - MongoDB support (code)
  - Redis support (code)
  - Cassandra support (code)
  - Your custom DB (code) ← hard to add
```

**With plugins** (extensible):
```
clnrm core (small, fast)
  + PostgreSQL plugin (if needed)
  + MongoDB plugin (if needed)
  + Your custom plugin (easy to add)
```

---

## Step 1: Understand the Plugin Trait (3 minutes)

All plugins implement the `ServicePlugin` trait:

```rust
pub trait ServicePlugin: Send + Sync {
    // Start the service
    fn start(&self) -> Result<ServiceHandle>;

    // Stop the service
    fn stop(&mut self, handle: ServiceHandle) -> Result<()>;

    // Check if service is healthy
    fn health_check(&self, handle: &ServiceHandle) -> Result<bool>;

    // What type of service is this?
    fn service_type(&self) -> &'static str;
}
```

### What Each Method Does

- **`start()`** — Allocate resources, configure, start service (return handle)
- **`stop()`** — Clean up resources, shut down service
- **`health_check()`** — Verify service is still running and responsive
- **`service_type()`** — Return service identifier (e.g., "custom_db")

---

## Step 2: Create a Simple Plugin (8 minutes)

Let's create a "greeting service" plugin. In your project:

```bash
mkdir -p src/plugins
```

Create: `src/plugins/greeting_plugin.rs`

```rust
use clnrm_core::{ServicePlugin, ServiceHandle, CleanroomError, Result};

/// A simple greeting service plugin
pub struct GreetingServicePlugin {
    name: String,
    message: String,
}

impl GreetingServicePlugin {
    /// Create a new greeting service
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
        }
    }
}

impl ServicePlugin for GreetingServicePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        println!("🎉 Greeting service starting: {}", self.name);
        println!("Message: {}", self.message);

        // Create a handle (could store port, PID, connection, etc.)
        let handle = ServiceHandle {
            id: self.name.clone(),
            port: 8080,
            // ... other fields
        };

        Ok(handle)
    }

    fn stop(&mut self, handle: ServiceHandle) -> Result<()> {
        println!("👋 Greeting service stopping: {}", handle.id);
        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<bool> {
        // Check if service is still responsive
        println!("✅ Greeting service is healthy");
        Ok(true)
    }

    fn service_type(&self) -> &'static str {
        "greeting_service"
    }
}
```

### Understanding the Implementation

- **`start()`** — Initialize and return a handle
- **`stop()`** — Cleanup (close connections, stop processes, etc.)
- **`health_check()`** — Verify service responsiveness
- **`service_type()`** — Identifier for matching in TOML

---

## Step 3: Register Your Plugin (4 minutes)

In your plugin registry, add your plugin:

Create: `src/plugins/mod.rs`

```rust
pub mod greeting_plugin;

pub use greeting_plugin::GreetingServicePlugin;

/// Register all available plugins
pub fn register_plugins() -> HashMap<String, Box<dyn ServicePlugin>> {
    let mut plugins: HashMap<String, Box<dyn ServicePlugin>> = HashMap::new();

    // Built-in plugins
    plugins.insert(
        "generic_container".to_string(),
        Box::new(GenericContainerPlugin::new()),
    );

    plugins.insert(
        "surrealdb".to_string(),
        Box::new(SurrealDBPlugin::new()),
    );

    // Your custom plugin
    plugins.insert(
        "greeting_service".to_string(),
        Box::new(GreetingServicePlugin::new(
            "my_greeting",
            "Hello from clnrm!",
        )),
    );

    plugins
}
```

---

## Step 4: Use Your Plugin in a Test (3 minutes)

In your TOML test file, reference your plugin:

```toml
[meta]
name = "greeting_service_test"
description = "Test the greeting service plugin"

[service.greeting]
plugin = "greeting_service"     # Your plugin name!
```

The clnrm orchestrator will:
1. Look for plugin named "greeting_service"
2. Find it in the registry
3. Call `start()` to initialize
4. Use the returned handle for scenarios
5. Call `stop()` and `health_check()` as needed

---

## Step 5: A More Realistic Example (2 minutes)

Here's a plugin for a custom database:

```rust
pub struct CustomDBPlugin {
    host: String,
    port: u16,
}

impl ServicePlugin for CustomDBPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Create container from custom DB image
        let container = docker.create_container(
            "my-custom-db:latest",
            &self.host,
            self.port,
        )?;

        // Wait for it to be ready
        self.wait_for_ready(&container)?;

        Ok(ServiceHandle {
            id: container.id,
            port: self.port,
        })
    }

    fn stop(&mut self, handle: ServiceHandle) -> Result<()> {
        // Stop and remove container
        docker.stop_container(&handle.id)?;
        docker.remove_container(&handle.id)?;
        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<bool> {
        // Connect and verify responsive
        let client = connect_to_db(&handle.id, self.port)?;
        client.ping()
    }

    fn service_type(&self) -> &'static str {
        "custom_db"
    }
}
```

---

## Step 6: Test Your Plugin (Built-in Test) (0 minutes)

clnrm includes built-in plugin testing:

```bash
# List all available plugins
clnrm plugins

# Output:
# Available Plugins:
#   - generic_container
#   - surrealdb
#   - ollama
#   - greeting_service      ← Your plugin!
```

---

## Plugin Lifecycle

When clnrm runs a test with your plugin:

```
Test File (.clnrm.toml)
    ↓
clnrm reads [service.greeting]
    ↓
Looks up plugin: "greeting_service"
    ↓
Finds GreetingServicePlugin in registry
    ↓
Calls: plugin.start()
    ↓
Runs scenarios with returned handle
    ↓
Calls: plugin.health_check() (periodic)
    ↓
Runs next scenario
    ↓
All scenarios done
    ↓
Calls: plugin.stop()
    ↓
Test complete
```

---

## Key Concepts

### ServicePlugin Trait
- **Required** — Implement start/stop/health_check/service_type
- **Synchronous** — No async methods (use block_in_place for async operations)
- **Send + Sync** — Must be thread-safe

### ServiceHandle
- Created by `start()`
- Contains service metadata (ID, port, credentials, etc.)
- Passed to scenarios and health checks
- Destroyed by `stop()`

### Plugin Registration
- Plugins registered in plugin registry
- Registry returns plugins by name
- TOML references plugins by `plugin = "name"`

---

## Common Patterns

### Pattern 1: Wrapping a Docker Image
```rust
impl ServicePlugin for MyDBPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Create Docker container
        // Wait for readiness (port open, health check passes, etc.)
        // Return handle with connection info
    }
}
```

### Pattern 2: Wrapping a Process
```rust
impl ServicePlugin for MyServicePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Start local process (not Docker)
        // Store PID in handle
        // Return handle
    }
}
```

### Pattern 3: Mocking
```rust
impl ServicePlugin for MockPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Don't start anything
        // Return fake handle for testing
    }
}
```

---

## Summary

You now know:
- ✅ **Plugin architecture** — Extensibility pattern
- ✅ **ServicePlugin trait** — Interface all plugins implement
- ✅ **Creating plugins** — Implement start/stop/health_check
- ✅ **Registering plugins** — Add to plugin registry
- ✅ **Using plugins** — Reference in TOML tests

---

## Next Steps

### Want to add observability?
→ [Tutorial 5: OTEL Integration](../05-otel-integration/)

### Want to understand the plugin system deeply?
→ [Explanation: Plugin System](../../explanation/plugin-system.md)

### Want to create advanced plugins?
→ [How-To: Plugin Development](../../how-to/plugin-development.md)

### Want practical examples?
→ [How-To: Custom Service Testing](../../how-to/custom-service-testing.md)

---

**Congratulations!** You can now extend clnrm with custom services! 🔧

Next: [Tutorial 5: OTEL Integration](../05-otel-integration/)
