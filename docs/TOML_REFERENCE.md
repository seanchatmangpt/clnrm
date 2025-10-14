# TOML Configuration Reference

Complete reference for writing Cleanroom test configurations in TOML format.

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

# Optional configuration
[services.my_database.config]
port = 8080                         # Port mapping
environment = { DEBUG = "true" }    # Environment variables
volumes = { "/host/path" = "/container/path" }  # Volume mounts
```

### Built-in Service Types

- **`generic_container`** - Basic container execution
- **`api`** - API service testing
- **`database`** - Database service testing
- **`cache`** - Cache service testing
- **`message_queue`** - Message queue testing

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
