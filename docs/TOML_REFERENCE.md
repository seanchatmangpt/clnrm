# TOML Configuration Reference for Cleanroom v1.0.0

Complete reference for writing Cleanroom v1.0.0 test configurations in flat TOML format with no-prefix variables.

> **🎯 Key Innovation:** No-prefix variables with Rust-based precedence resolution. Templates use clean `{{ svc }}`, `{{ endpoint }}` syntax with variables resolved in Rust: template vars → ENV → defaults.

## Schema Overview

Cleanroom v1.0.0 uses a **flat TOML schema** focused on OTEL-based validation:

### **Required Sections**
- `[meta]` - Test metadata
- `[otel]` - OpenTelemetry configuration
- `[service.<id>]` - Service definition (e.g., `[service.clnrm]`)
- `[[scenario]]` - Test scenario definition

### **Optional Sections**
- `[[expect.span]]` - Span validation rules
- `[expect.graph]` - Graph relationship validation
- `[expect.status]` - Status code validation
- `[expect.hermeticity]` - Isolation validation
- `[determinism]` - Deterministic testing configuration
- `[report]` - Report generation settings

### **Authoring-Only Section**
- `[vars]` - Template variables for documentation (ignored at runtime)

## Variable Resolution

Variables are resolved in **Rust** with clear precedence:

1. **Template variables** (highest priority) - Define in `[vars]` section
2. **Environment variables** - `$SERVICE_NAME`, `$OTEL_ENDPOINT`, etc.
3. **Defaults** (lowest priority) - Built-in fallback values

**Available Variables:**
- `svc` - Service name (default: "clnrm")
- `env` - Environment (default: "ci")
- `endpoint` - OTEL endpoint (default: "http://localhost:4318")
- `exporter` - OTEL exporter (default: "otlp")
- `image` - Container image (default: "registry/clnrm:1.0.0")
- `freeze_clock` - Deterministic time (default: "2025-01-01T00:00:00Z")
- `token` - OTEL auth token (default: "")

## Required Sections

### [meta] - Test Metadata

```toml
[meta]
name = "{{ svc }}_otel_proof"        # Required: Test identifier (uses resolved variable)
version = "1.0"                     # Required: Test version
description = "Telemetry-only"      # Required: Human-readable description
```

### [otel] - OpenTelemetry Configuration

```toml
[otel]
exporter = "{{ exporter }}"         # Required: "stdout" or "otlp"
endpoint = "{{ endpoint }}"         # Required: OTLP endpoint URL
protocol = "http/protobuf"          # Required: Protocol specification
sample_ratio = 1.0                  # Required: Sampling ratio (0.0-1.0)

# Optional: Service resources
resources = {
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}

# Optional: Authentication headers
[otel.headers]
{% if token != "" %}
Authorization = "Bearer {{ token }}"
{% endif %}
```

### [service.<id>] - Service Definition

```toml
[service.clnrm]                     # Required: Service identifier
plugin = "generic_container"        # Required: Plugin type
image = "{{ image }}"               # Required: Container image
args = [                            # Required: Command arguments
  "self-test",
  "--otel-exporter", "{{ exporter }}",
  "--otel-endpoint", "{{ endpoint }}"
]
env = {                             # Required: Environment variables
  "OTEL_TRACES_EXPORTER" = "{{ exporter }}",
  "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}"
}
wait_for_span = "clnrm.run"         # Required: Wait for this span before starting
```

### [[scenario]] - Test Scenario

```toml
[[scenario]]                        # Required: At least one scenario
name = "otel_only_proof"            # Required: Scenario identifier
service = "clnrm"                   # Required: Service to execute
run = "clnrm run --otel-exporter {{ exporter }} --otel-endpoint {{ endpoint }}"
artifacts.collect = ["spans:default"]  # Required: Collect telemetry spans

## Optional Sections

### [[expect.span]] - Span Validation

```toml
[[expect.span]]
name = "clnrm.run"                  # Required: Span name to validate
kind = "internal"                   # Required: Span kind
attrs.all = { "result" = "pass" }   # Required: Attributes that must match

# Optional: Parent-child relationships
[[expect.span]]
name = "clnrm.step:hello_world"
parent = "clnrm.run"                # Must be child of parent span
kind = "internal"
events.any = [                      # Must contain at least one of these events
  "container.start",
  "container.exec",
  "container.stop"
]

# Optional: Duration constraints
[[expect.span]]
name = "database.query"
kind = "client"
duration_ms = { min = 10, max = 5000 }  # Duration must be in range
```

### [expect.graph] - Graph Validation

```toml
[expect.graph]
must_include = [                    # Required: These parent-child relationships must exist
  ["clnrm.run", "clnrm.step:hello_world"]
]
acyclic = true                      # Required: Graph must be acyclic

# Optional: Relationships that must NOT exist
must_not_cross = [
  ["service.a", "service.b"]        # These spans must not both be ancestors of the same span
]
```

### [expect.status] - Status Validation

```toml
[expect.status]
all = "OK"                          # Required: All spans must have OK status

# Optional: Status validation by span name pattern
by_name."api.*" = "OK"              # Spans matching pattern must have OK status
by_name."error.*" = "ERROR"         # Error spans must have ERROR status
```

### [expect.hermeticity] - Isolation Validation

```toml
[expect.hermeticity]
no_external_services = true         # Required: No external service calls allowed
resource_attrs.must_match = {       # Required: Resource attributes must match
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}

# Optional: Forbidden attribute keys
span_attrs.forbid_keys = [          # These attribute keys must not appear
  "secret.api_key",
  "internal.database_password"
]
```

### [determinism] - Deterministic Testing

```toml
[determinism]
seed = 42                           # Required: Random seed for reproducibility
freeze_clock = "{{ freeze_clock }}" # Required: Frozen timestamp for time-based operations
```

### [report] - Report Generation

```toml
[report]
json = "report.json"                # Optional: Generate JSON report
junit = "junit.xml"                 # Optional: Generate JUnit XML for CI/CD
digest = "trace.sha256"             # Optional: Generate SHA-256 digest of normalized trace
```

## Authoring-Only Section

### [vars] - Template Variables (Documentation Only)

```toml
[vars]                              # Ignored at runtime - for documentation only
svc = "{{ svc }}"                   # Shows resolved service name
env = "{{ env }}"                   # Shows resolved environment
endpoint = "{{ endpoint }}"         # Shows resolved OTEL endpoint
exporter = "{{ exporter }}"         # Shows resolved OTEL exporter
freeze_clock = "{{ freeze_clock }}" # Shows frozen timestamp
image = "{{ image }}"               # Shows container image
```

This section helps authors understand what values will be used but is ignored during execution.

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
