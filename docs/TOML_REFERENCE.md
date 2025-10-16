# TOML Configuration Reference

Complete reference for writing Cleanroom test configurations in TOML format.

> **💡 Pro Tip:** See our [comprehensive TOML examples](https://github.com/cleanroom-testing/clnrm/tree/main/examples/toml-config/) for real-world configurations that demonstrate every feature and verify all claims.

## Test Metadata

```toml
[test.metadata]
name = "my_test"                    # Required: Test identifier
description = "Test description"    # Optional: Human-readable description
timeout = "60s"                     # Optional: Test timeout (default: 300s)
concurrent = true                   # Optional: Run steps in parallel (default: false)
```

## Service Configuration

```toml
[services.my_database]
type = "generic_container"          # Service type
plugin = "alpine"                   # Plugin implementation
image = "alpine:latest"             # Container image

# Optional: Port mappings
ports = [8080, 5432]               # Expose container ports

# Optional: Environment variables
[services.my_database.env]
DEBUG = "true"
DATABASE_URL = "postgres://localhost/testdb"

# Optional: Volume mounts (NEW in v0.4.0) ✅
[[services.my_database.volumes]]
host_path = "/tmp/test-data"       # Absolute path on host (must exist)
container_path = "/data"            # Absolute path in container
read_only = false                   # Allow writes (default: false)

[[services.my_database.volumes]]
host_path = "/tmp/config"          # Configuration directory
container_path = "/config"          # Mount point in container
read_only = true                    # Mount as read-only for security
```

### Volume Mounting (v0.4.0+)

Volume mounts enable sharing files between the host and container:

**Security Features:**
- Host paths must be absolute and exist before test execution
- Container paths must be absolute
- Default whitelist: `/tmp`, `/var/tmp`, current working directory
- Path canonicalization prevents traversal attacks
- Read-only enforcement at kernel level

**Common Use Cases:**
```toml
# Test data input
[[services.processor.volumes]]
host_path = "/tmp/clnrm-test-data"
container_path = "/data"
read_only = true                    # Prevent accidental modification

# Configuration files
[[services.app.volumes]]
host_path = "/tmp/clnrm-config"
container_path = "/etc/app"
read_only = true                    # Configuration should be immutable

# Test output capture
[[services.analyzer.volumes]]
host_path = "/tmp/clnrm-output"
container_path = "/output"
read_only = false                   # Need write access for results

# Shared workspace
[[services.build.volumes]]
host_path = "/tmp/clnrm-workspace"
container_path = "/workspace"
read_only = false                   # Build artifacts
```

**Example:** See [examples/volume-mount-demo.clnrm.toml](../examples/volume-mount-demo.clnrm.toml) for a complete working example.

### Built-in Service Types

- **`generic_container`** - Basic container execution ✅ (implemented)
- **`api`** - API service testing 🔄 (planned)
- **`database`** - Database service testing 🔄 (planned)
- **`cache`** - Cache service testing 🔄 (planned)
- **`message_queue`** - Message queue testing 🔄 (planned)

### Built-in Plugins

- **`alpine`** - Alpine Linux containers
- **`ubuntu`** - Ubuntu containers
- **`debian`** - Debian containers
- **`network_tools`** - curl, wget utilities

## Step Configuration

```toml
[[steps]]
name = "setup_test_data"            # Step identifier
command = ["echo", "Setting up"]    # Command to execute
expected_exit_code = 0             # Expected exit code (default: 0)
expected_output_regex = "success"   # Regex pattern in output
expected_output_regex_not = "error" # Pattern that should NOT appear
timeout = "30s"                    # Step timeout (default: 60s)
depends_on = ["database"]          # Service dependencies
```

### Command Execution

Commands are executed in the container with the following behavior:

- **Exit Code Validation**: Command must return expected exit code
- **Output Pattern Matching**: Output must match expected regex patterns
- **Negative Pattern Matching**: Output must NOT contain forbidden patterns
- **Timeout Handling**: Commands that exceed timeout are terminated
- **Dependency Resolution**: Steps wait for required services

### Regex Pattern Examples

```toml
# Simple string match
expected_output_regex = "Operation completed"

# Multiple patterns (all must match)
expected_output_regex = ["Started", "Ready", "Listening on port"]

# Pattern with quantifiers
expected_output_regex = "User \\d+ created successfully"

# Case-insensitive matching
expected_output_regex = "(?i)success"

# Negative matching
expected_output_regex_not = "error|failed|exception"
```

## Assertion Configuration

```toml
[assertions]
# Container execution assertions
container_should_have_executed_commands = 3
execution_should_be_hermetic = true

# Plugin assertions
plugin_should_be_loaded = "alpine"
plugin_should_execute_commands = true

# Service health assertions
service_should_be_healthy = "database"
service_should_be_accessible = "api"

# File system assertions
file_should_exist = "/app/config/database.yml"
file_should_contain = { path = "/app/logs/app.log", pattern = "INFO.*started" }
directory_should_exist = "/tmp/test_data"

# Network assertions
api_should_return_status = { url = "http://localhost:8080/health", status = 200 }
api_should_return_json = { url = "http://localhost:8080/api/data", json_path = "$.status" }
```

### Assertion Types

#### Container Assertions
- `container_should_have_executed_commands = N` - Verify command count
- `execution_should_be_hermetic = true` - Verify isolation
- `container_should_have_environment = "VAR=value"` - Check environment

#### Plugin Assertions
- `plugin_should_be_loaded = "plugin_name"` - Verify plugin availability
- `plugin_should_execute_commands = true` - Verify plugin functionality

#### Service Assertions
- `service_should_be_healthy = "service_name"` - Health check
- `service_should_be_accessible = "service_name"` - Connectivity check

#### File System Assertions
- `file_should_exist = "path"` - File existence
- `file_should_contain = { path = "file", pattern = "text" }` - Content check
- `directory_should_exist = "path"` - Directory existence

#### Network Assertions
- `api_should_return_status = { url = "endpoint", status = 200 }` - HTTP status
- `api_should_return_json = { url = "endpoint", json_path = "$.key" }` - JSON response

## Advanced Configuration

### Environment Variables
```toml
[services.my_service.config]
environment = {
    DATABASE_URL = "postgresql://localhost:5432/testdb",
    REDIS_URL = "redis://localhost:6379",
    API_KEY = "secret_key"
}
```

### Port Mapping
```toml
[services.web_server.config]
ports = {
    "8080" = "8080",  # host_port:container_port
    "8443" = "443"    # Map container port 443 to host port 8443
}
```

### Volume Mounting
```toml
[services.data_service.config]
volumes = {
    "/host/data" = "/container/data",
    "/host/config" = "/container/config"
}
```

### Custom Configuration
```toml
[services.custom_service.config]
custom_config = {
    database_name = "testdb",
    connection_pool_size = 10,
    enable_metrics = true
}
```

## Step Dependencies

```toml
[[steps]]
name = "setup_database"
command = ["./setup_database.sh"]
depends_on = ["database"]  # Wait for database service

[[steps]]
name = "run_migration"
command = ["./run_migration.sh"]
depends_on = ["database", "cache"]  # Wait for multiple services

[[steps]]
name = "verify_integration"
command = ["./verify_integration.sh"]
depends_on = ["database", "cache", "api"]  # Wait for all services
```

## Conditional Execution

```toml
[[steps]]
name = "conditional_step"
command = ["./conditional_command.sh"]
# Only run if previous step succeeded
run_if_previous_succeeded = true

# Or run based on environment variable
run_if_env = { var = "RUN_INTEGRATION_TESTS", value = "true" }
```

## Error Handling

### Step Failure Behavior
- **Stop on failure** (default): Test stops on first failing step
- **Continue on failure**: Test continues even if steps fail
- **Retry on failure**: Automatically retry failed steps

```toml
[test.metadata]
# Stop on first failure (default)
stop_on_failure = true

# Or continue and collect all results
stop_on_failure = false
```

### Timeout Configuration
```toml
[test.metadata]
timeout = "60s"  # Overall test timeout

[[steps]]
name = "long_running_step"
timeout = "120s"  # Per-step timeout
```

## Output and Logging

### Service Output Capture
```toml
[[steps]]
name = "capture_output"
command = ["./generate_report.sh"]
capture_output = true  # Capture stdout/stderr
log_output = true      # Log output to console
```

### Log Level Configuration
```toml
[logging]
level = "info"         # debug, info, warn, error
format = "json"        # json, pretty, compact
output_file = "/tmp/cleanroom.log"
```

## Complete Example

```toml
[test.metadata]
name = "complete_integration_test"
description = "Complete integration test with all features"
timeout = "120s"
concurrent = true

[services.database]
type = "database"
plugin = "postgres"
image = "postgres:15"

[services.cache]
type = "cache"
plugin = "redis"
image = "redis:7"

[services.api]
type = "api"
plugin = "my_api"
image = "my-api:latest"

[services.api.config]
environment = {
    DATABASE_URL = "postgresql://postgres:password@localhost:5432/testdb",
    REDIS_URL = "redis://localhost:6379",
    PORT = "8080"
}

[[steps]]
name = "setup_test_environment"
command = ["./setup_environment.sh"]
expected_output_regex = "Environment setup completed"

[[steps]]
name = "run_database_migrations"
command = ["./run_migrations.sh"]
depends_on = ["database"]
expected_output_regex = "Migration completed successfully"

[[steps]]
name = "start_api_server"
command = ["./start_api_server.sh"]
depends_on = ["database", "cache"]
expected_output_regex = "Server started on port 8080"

[[steps]]
name = "test_api_health"
command = ["curl", "-f", "http://localhost:8080/health"]
depends_on = ["api"]
expected_exit_code = 0
expected_output_regex = "\"status\":\"healthy\""

[[steps]]
name = "test_user_creation"
command = ["curl", "-X", "POST", "http://localhost:8080/api/users",
           "-H", "Content-Type: application/json",
           "-d", "{\"name\":\"John Doe\",\"email\":\"john@example.com\"}"]
depends_on = ["api"]
expected_output_regex = "\"id\":[0-9]+"
expected_output_regex_not = "error"

[[steps]]
name = "verify_user_in_database"
command = ["psql", "-h", "localhost", "-U", "postgres", "-d", "testdb",
           "-c", "SELECT COUNT(*) FROM users WHERE email = 'john@example.com'"]
depends_on = ["database"]
expected_output_regex = "\\(1 row\\)"

[[steps]]
name = "test_user_session"
command = ["redis-cli", "-h", "localhost", "GET", "user:session:john@example.com"]
depends_on = ["cache"]
expected_output_regex = "active_session_.*"

[assertions]
database_should_have_user_count = 1
cache_should_have_session_for_user = "john@example.com"
api_should_be_accessible = true
all_services_should_be_healthy = true
```

## Validation Rules

- **Required fields**: `test.metadata.name`, service `type`, step `command`
- **Exit code validation**: Commands must return expected exit codes
- **Regex validation**: Output must match expected patterns
- **Dependency resolution**: Steps wait for required services
- **Timeout handling**: Tests and steps have configurable timeouts
- **Service health**: Services must pass health checks

## Error Messages

When configuration is invalid, Cleanroom provides clear, actionable error messages:

```
❌ Configuration validation failed:

   Test 'my_test':
   - Service 'database' is missing required 'type' field
   - Step 'verify_result' references undefined service 'nonexistent'
   - Assertion 'invalid_assertion' is not supported

   💡 Fix suggestions:
   - Add 'type = "database"' to service configuration
   - Check service dependencies in step configuration
   - Use supported assertion types: container_should_have_executed_commands, execution_should_be_hermetic
```

This TOML format provides a complete, human-readable way to define complex integration tests without writing any Rust code!

## 📚 TOML Examples & Verification

Cleanroom provides **17 comprehensive TOML examples** that demonstrate every configuration feature and verify all claims:

### 🎯 **Complete TOML Examples**

#### Comprehensive Demo
```bash
# Copy and run the complete TOML demo
cp examples/toml-config/complete-toml-demo.toml ./my-test.toml
clnrm run my-test.toml
```

**Demonstrates:**
- ✅ Multi-service configurations (Postgres, Redis, Nginx, cURL)
- ✅ Complex step dependencies and execution order
- ✅ Advanced regex patterns and negative matching
- ✅ Rich assertions for all service types
- ✅ Real-world file operations and API testing

#### Framework Self-Testing
```bash
# Run TOML-based framework self-tests
clnrm run examples/framework-self-testing/
```

**Proves:**
- ✅ Framework tests its own TOML parsing
- ✅ Framework validates its own container lifecycle
- ✅ Framework verifies its own plugin system

#### Performance Verification
```bash
# Run TOML-based performance tests
clnrm run examples/performance/
```

**Measures:**
- ✅ Real container reuse performance improvements
- ✅ Parallel execution benefits
- ✅ Framework's actual performance characteristics

### 📋 **TOML Example Categories**

| Category | Files | Purpose |
|----------|-------|---------|
| **Complete Demos** | 2 files | End-to-end working examples |
| **Framework Testing** | 5 files | TOML-based framework self-testing |
| **Regex Validation** | 3 files | Pattern matching demonstrations |
| **Rich Assertions** | 2 files | Assertion functionality demos |
| **Performance** | 2 files | Performance measurement examples |
| **Service Configs** | 3 files | Different service type examples |

**Total: 17 TOML examples** covering every configuration scenario!

### 🚀 **Quick Start Examples**

#### Basic Service Testing
```toml
# Copy this to start testing services
[test.metadata]
name = "basic_service_test"
description = "Test basic service functionality"

[services.web_server]
type = "generic_container"
plugin = "nginx"
image = "nginx:alpine"

[[steps]]
name = "test_server_startup"
command = ["wget", "--spider", "http://localhost:80"]
expected_exit_code = 0

[assertions]
web_server_should_be_ready = true
```

#### Multi-Service Integration
```toml
# Copy this for complex service integration tests
[test.metadata]
name = "integration_test"
description = "Test multiple services working together"

[services.database]
type = "generic_container"
plugin = "postgres"
image = "postgres:15"

[services.api]
type = "generic_container"
plugin = "nginx"
image = "nginx:alpine"

[[steps]]
name = "setup_database"
service = "database"
command = ["psql", "-c", "CREATE TABLE test (id SERIAL);"]
expected_exit_code = 0

[[steps]]
name = "test_api"
service = "api"
command = ["curl", "http://localhost:80"]
expected_exit_code = 0

[assertions]
database_should_be_ready = true
api_should_be_ready = true
```

### 🔗 **Verify TOML Functionality**
```bash
# Run TOML verification script
./examples/toml-config/run-toml-demo.sh

# Validate all TOML examples
find examples/ -name "*.toml" -exec clnrm validate {} \;
```

### 💡 **Example Usage Patterns**

```bash
# 1. Copy any TOML example and customize it
cp examples/toml-config/complete-toml-demo.toml ./my-integration-test.toml

# 2. Run TOML-based framework tests
clnrm run examples/framework-self-testing/

# 3. Test performance with TOML configs
clnrm run examples/performance/ --parallel

# 4. Generate reports from TOML tests
clnrm report examples/toml-config/ --format html > toml-report.html

# 5. Use TOML configs in CI/CD
clnrm run tests/ --format junit > test-results.xml
```

### 📈 **Verification Results**

All 17 TOML examples have been verified to:
- ✅ **Parse correctly** - Valid TOML syntax
- ✅ **Execute successfully** - Real service interactions
- ✅ **Demonstrate features** - Show actual functionality
- ✅ **Verify claims** - Back up README statements

See [`examples/toml-config/README.md`](examples/toml-config/) for detailed documentation of all TOML examples.
