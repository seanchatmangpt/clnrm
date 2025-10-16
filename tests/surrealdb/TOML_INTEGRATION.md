# TOML-Based SurrealDB Service Integration

## Overview

The clnrm framework now supports **complete service management directly from `.clnrm.toml` files**. This means you can define SurrealDB (and other services) in your test configuration, and the framework will automatically:

1. Parse the service configuration
2. Create the appropriate plugin instance (e.g., `SurrealDbPlugin`)
3. Start the container before running test steps
4. Stop the container after test completion (even on failure)

## How It Works

### Configuration Flow

```
.clnrm.toml file
      ↓
[services.my_db] section parsed
      ↓
ServiceConfig struct created
      ↓
load_services_from_config() called
      ↓
Match on plugin type "surrealdb"
      ↓
SurrealDbPlugin::with_credentials(username, password)
      ↓
Plugin registered with CleanroomEnvironment
      ↓
Service started before test steps
      ↓
ServiceHandle available for test execution
      ↓
Service stopped after test completion
```

### TOML Syntax

```toml
[services.service_name]
type = "surrealdb"              # Service type
plugin = "surrealdb"            # Plugin to use
username = "root"               # Optional: defaults to "root"
password = "root"               # Optional: defaults to "root"
strict = false                  # Optional: defaults to false
```

## Supported Services

| Plugin | Type | Configuration Options |
|--------|------|----------------------|
| **SurrealDB** | `surrealdb` | `username`, `password`, `strict` |
| **Generic Container** | `generic_container` | `image`, `env`, `ports`, `volumes` |
| **Ollama** | `ollama` | `OLLAMA_ENDPOINT`, `OLLAMA_MODEL` via env |
| **vLLM** | `vllm` | `VLLM_ENDPOINT`, `VLLM_MODEL` via env |
| **TGI** | `tgi` | `TGI_ENDPOINT`, `TGI_MODEL` via env |
| **Chaos Engine** | `chaos_engine` | Uses default chaos config |

## Examples

### Example 1: Basic SurrealDB Service

```toml
[test.metadata]
name = "basic_surrealdb_test"

[services.my_db]
type = "surrealdb"
plugin = "surrealdb"
username = "root"
password = "root"

[[steps]]
name = "verify_running"
command = ["echo", "Database is ready"]
```

### Example 2: Custom Credentials

```toml
[services.prod_db]
type = "surrealdb"
plugin = "surrealdb"
username = "admin"
password = "secure_password_123"
strict = true
```

### Example 3: Multiple Services

```toml
# Database service
[services.database]
type = "surrealdb"
plugin = "surrealdb"

# Application service
[services.app]
type = "generic_container"
plugin = "generic_container"
image = "myapp:latest"

[services.app.env]
DATABASE_URL = "ws://127.0.0.1:8000"
```

### Example 4: With Volumes (Generic Container)

```toml
[services.data_processor]
type = "generic_container"
plugin = "generic_container"
image = "alpine:latest"
ports = [8080]

[[services.data_processor.volumes]]
host_path = "/tmp/test-data"
container_path = "/data"
read_only = false

[services.data_processor.env]
DATA_PATH = "/data"
```

## Running Tests

```bash
# Run a single TOML test with managed services
cargo run -- run tests/surrealdb/toml-managed.clnrm.toml

# Run all SurrealDB TOML tests
cargo run -- run tests/surrealdb/*.toml

# Run example demos
cargo run -- run examples/surrealdb-integration-demo.clnrm.toml
cargo run -- run examples/multi-service-demo.clnrm.toml
```

## Implementation Details

### Service Loading (`run.rs`)

The `load_services_from_config()` function in `cli/commands/run.rs` handles service instantiation:

```rust
async fn load_services_from_config(
    env: &CleanroomEnvironment,
    services: &HashMap<String, ServiceConfig>,
) -> Result<HashMap<String, ServiceHandle>> {
    let mut service_handles = HashMap::new();

    for (service_name, service_config) in services {
        // Create plugin based on type
        let plugin: Box<dyn ServicePlugin> = match service_config.plugin.as_str() {
            "surrealdb" => {
                let username = service_config.username.as_deref().unwrap_or("root");
                let password = service_config.password.as_deref().unwrap_or("root");
                let strict = service_config.strict.unwrap_or(false);

                let mut plugin = SurrealDbPlugin::with_credentials(username, password);
                plugin = plugin.with_strict(strict);
                Box::new(plugin)
            }
            // ... other plugin types ...
        };

        // Register and start service
        env.register_service(plugin).await?;
        let handle = env.start_service(service_name).await?;
        service_handles.insert(service_name.clone(), handle);
    }

    Ok(service_handles)
}
```

### Service Lifecycle

1. **Before test steps**: `load_services_from_config()` is called
2. **During test execution**: Services are running and available
3. **After test steps**: Services are stopped in reverse order
4. **On failure**: Services are still cleaned up properly

### Configuration Schema (`config.rs`)

```rust
pub struct ServiceConfig {
    pub r#type: String,
    pub plugin: String,
    pub image: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<u16>>,
    pub volumes: Option<Vec<VolumeConfig>>,
    pub health_check: Option<HealthCheckConfig>,

    // SurrealDB-specific options
    pub username: Option<String>,
    pub password: Option<String>,
    pub strict: Option<bool>,
}
```

## Service Metadata

When a service starts, it provides metadata that can be accessed by test steps:

```rust
pub struct ServiceHandle {
    pub id: String,
    pub service_name: String,
    pub metadata: HashMap<String, String>,
}
```

### SurrealDB Metadata

```rust
metadata = {
    "host": "127.0.0.1",
    "port": "8000",
    "username": "root",
    "connection_string": "ws://127.0.0.1:8000",
    "database_type": "surrealdb"
}
```

## Best Practices

1. **Service Names**: Use descriptive names like `app_database`, `cache_server`, `message_queue`
2. **Credentials**: Use test credentials in TOML, production credentials via environment variables
3. **Resource Cleanup**: Services are automatically stopped, no manual cleanup needed
4. **Isolation**: Each test gets fresh service instances (hermetic isolation)
5. **Multiple Services**: Define as many services as needed in the `[services]` section

## Error Handling

The framework provides clear error messages for common issues:

```
❌ Unknown service plugin: invalid_plugin
✅ Service 'my_db' started successfully (handle: uuid-123)
⚠️  Failed to stop service 'my_db': connection timeout
```

## Troubleshooting

### Service Won't Start

- Check Docker is running: `docker ps`
- Verify image name is correct
- Check port conflicts: `lsof -i :8000`
- Review service logs

### Plugin Not Found

Ensure plugin name matches exactly:
- ✅ `plugin = "surrealdb"`
- ❌ `plugin = "SurrealDB"`
- ❌ `plugin = "surreal"`

### Credentials Not Working

- Check username/password in TOML
- Verify SurrealDB version supports authentication
- Try default credentials: `root`/`root`

## Next Steps

1. Review working examples in `examples/` directory
2. Run the TOML tests in `tests/surrealdb/`
3. Create your own TOML-based tests
4. Explore multi-service configurations

## Related Documentation

- [SurrealDB Test Suite](./README.md)
- [TOML Reference](../../docs/TOML_REFERENCE.md)
- [Plugin Development](../../docs/PLUGIN_DEVELOPMENT.md)
- [Service Architecture](../../docs/SERVICE_ARCHITECTURE.md)

---

**Status**: ✅ Production Ready
**Version**: clnrm v0.4.0
**Last Updated**: 2024-10-16
