//! Template command implementation
//!
//! Handles project generation from templates with various configurations.

use crate::error::{CleanroomError, Result};
use tracing::{debug, info};

/// Generate a basic OTEL template with Tera syntax
pub fn generate_otel_template() -> Result<String> {
    Ok(r#"# clnrm OTEL validation template (v0.6.0)
# This file uses Tera templating syntax

[meta]
name = "{{ vars.name | default(value="otel_validation") }}"
version = "0.6.0"
description = "Telemetry-only validation test"

[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
{% if otel.endpoint %}
endpoint = "{{ otel.endpoint }}"
{% endif %}
sample_ratio = 1.0
resources = { "service.name" = "clnrm", "service.version" = "0.6.0" }

[service.clnrm]
plugin = "generic_container"
image = "{{ vars.image | default(value="alpine:latest") }}"
args = ["sh", "-c", "echo 'Running test'"]
wait_for_span = "clnrm.run"

[[scenario]]
name = "otel_validation"
service = "clnrm"
run = "echo 'Test execution'"

[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[expect.counts]
spans_total = { gte = 1 }
errors_total = { eq = 0 }

{% if vars.deterministic %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
{% endif %}

[report]
json = "{{ vars.report_dir | default(value="reports") }}/report.json"
digest = "{{ vars.report_dir | default(value="reports") }}/digest.sha256"
"#
    .to_string())
}

/// Generate a macro library for common patterns
pub fn generate_macro_library() -> Result<String> {
    Ok(r#"# Tera Macro Library for clnrm v0.6.0
# Reusable template macros for common testing patterns

{% macro container_lifecycle_events() %}
["container.start", "container.exec", "container.stop"]
{% endmacro %}

{% macro otel_standard_resources(service_name, version) %}
{
  "service.name" = "{{ service_name }}",
  "service.version" = "{{ version }}",
  "deployment.environment" = "{{ env(name="ENV") | default(value="test") }}"
}
{% endmacro %}

{% macro span_assertions(prefix, kind) %}
[[expect.span]]
name = "{{ prefix }}.*"
kind = "{{ kind }}"
attrs.all = { "test.framework" = "clnrm" }
{% endmacro %}

{% macro temporal_order(first, second) %}
[expect.order]
must_precede = [["{{ first }}", "{{ second }}"]]
{% endmacro %}

{% macro status_validation(pattern, status) %}
[expect.status]
by_name."{{ pattern }}" = "{{ status }}"
{% endmacro %}

{% macro standard_reports(dir) %}
[report]
json = "{{ dir }}/report.json"
junit = "{{ dir }}/junit.xml"
digest = "{{ dir }}/digest.sha256"
{% endmacro %}
"#.to_string())
}

/// Generate a matrix testing template
pub fn generate_matrix_template() -> Result<String> {
    Ok(r#"# Matrix Testing Template (v0.6.0)
# Demonstrates cross-product testing with Tera loops

[meta]
name = "matrix_test_{{ matrix.os }}_{{ matrix.version }}"
version = "0.6.0"

[vars]
test_count = 3

[matrix]
os = ["alpine", "ubuntu", "debian"]
version = ["3.18", "22.04", "bullseye"]

[otel]
exporter = "stdout"
resources = {
  "service.name" = "matrix-test",
  "test.os" = "{{ matrix.os }}",
  "test.version" = "{{ matrix.version }}"
}

[service.test_runner]
plugin = "generic_container"
image = "{{ matrix.os }}:{{ matrix.version }}"
args = ["sh", "-c", "echo 'Testing on {{ matrix.os }}:{{ matrix.version }}'"]

{% for i in range(end=vars.test_count) %}
[[scenario]]
name = "test_{{ i }}_{{ matrix.os }}"
service = "test_runner"
run = "echo 'Test {{ i }} on {{ matrix.os }}'"
{% endfor %}

[[expect.span]]
name = "test_runner.exec"
kind = "internal"
attrs.all = {
  "container.os" = "{{ matrix.os }}",
  "container.version" = "{{ matrix.version }}"
}

[expect.counts]
spans_total = { gte = {{ vars.test_count }} }

[report]
json = "reports/{{ matrix.os }}_{{ matrix.version }}.json"
digest = "reports/{{ matrix.os }}_{{ matrix.version }}.sha256"
"#.to_string())
}

/// Generate comprehensive validation template showcasing all validators
pub fn generate_full_validation_template() -> Result<String> {
    Ok(r#"# Full Validation Template (v0.6.0)
# Demonstrates all available validators: order, status, count, window, graph, hermeticity

[meta]
name = "{{ vars.test_name | default(value="full_validation") }}"
version = "0.6.0"
description = "Comprehensive validation example using all validators"

[vars]
service_name = "validation_demo"
max_duration_ms = 5000

[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
sample_ratio = 1.0
resources = {
  "service.name" = "{{ vars.service_name }}",
  "service.version" = "0.6.0",
  "test.timestamp" = "{{ now_rfc3339() }}"
}

{% if env(name="OTEL_ENDPOINT") %}
[otel.headers]
"x-api-key" = "{{ env(name="OTEL_API_KEY") | default(value="test-key") }}"
"x-test-id" = "{{ sha256(s=vars.test_name) }}"
{% endif %}

[otel.propagators]
use = ["tracecontext", "baggage"]

[service.main_service]
plugin = "generic_container"
image = "alpine:latest"
args = ["sh", "-c", "sleep 1 && echo 'Service ready'"]
wait_for_span = "main_service.start"

[[scenario]]
name = "startup"
service = "main_service"
run = "echo 'Starting service'"

[[scenario]]
name = "process_request"
service = "main_service"
run = "echo 'Processing request'"

[[scenario]]
name = "shutdown"
service = "main_service"
run = "echo 'Shutting down'"

# ========================================
# SPAN ASSERTIONS
# ========================================
[[expect.span]]
name = "main_service.start"
kind = "server"
attrs.all = { "service.state" = "starting" }

[[expect.span]]
name = "main_service.exec"
kind = "internal"
attrs.any = { "exec.command" = "echo" }

[[expect.span]]
name = "main_service.stop"
kind = "server"
attrs.all = { "service.state" = "stopped" }

# ========================================
# ORDER VALIDATION (temporal ordering)
# ========================================
[expect.order]
must_precede = [
  ["main_service.start", "main_service.exec"],
  ["main_service.exec", "main_service.stop"]
]
must_follow = [
  ["main_service.stop", "main_service.start"]
]

# ========================================
# STATUS VALIDATION (with glob patterns)
# ========================================
[expect.status]
all = "ok"  # All spans must have OK status
by_name."main_service.*" = "ok"  # Glob pattern matching
by_name."error.*" = "error"  # Error spans should have error status

# ========================================
# COUNT VALIDATION
# ========================================
[expect.counts]
spans_total = { gte = 3, lte = 10 }
errors_total = { eq = 0 }
spans_by_kind.internal = { gte = 1 }
spans_by_kind.server = { gte = 2 }

# ========================================
# WINDOW VALIDATION (time-based)
# ========================================
[[expect.window]]
name = "startup_window"
start_span = "main_service.start"
end_span = "main_service.exec"
max_duration_ms = {{ vars.max_duration_ms }}
min_span_count = 1

# ========================================
# GRAPH VALIDATION (trace topology)
# ========================================
[expect.graph]
parent_child = [
  ["main_service.start", "startup"],
  ["main_service.exec", "process_request"],
  ["main_service.stop", "shutdown"]
]
max_depth = 3
must_be_connected = true

# ========================================
# HERMETICITY VALIDATION
# ========================================
[expect.hermeticity]
allow_network = false
allow_filesystem_read = true
allow_filesystem_write = false
allowed_env_vars = ["PATH", "HOME"]
forbidden_syscalls = ["socket", "connect"]

# ========================================
# DETERMINISM CONFIGURATION
# ========================================
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

# ========================================
# RESOURCE LIMITS
# ========================================
[limits]
cpu_millicores = 500
memory_mb = 512

# ========================================
# REPORTING CONFIGURATION
# ========================================
[report]
json = "reports/{{ vars.test_name }}_{{ now_rfc3339() | replace(from=":", to="-") }}.json"
junit = "reports/junit_{{ sha256(s=vars.test_name) | truncate(length=8, end="") }}.xml"
digest = "reports/digest_{{ vars.test_name }}.sha256"
"#.to_string())
}

/// Generate a lifecycle matcher template
pub fn generate_lifecycle_matcher() -> Result<String> {
    Ok(r#"{% macro container_lifecycle_events() %}
["container.start", "container.exec", "container.stop"]
{% endmacro %}"#
        .to_string())
}

/// Generate a deterministic testing template
pub fn generate_deterministic_template() -> Result<String> {
    Ok(r#"# Deterministic Testing Template (v0.6.0)
# Ensures reproducible test results with seeded randomness and frozen clock

[meta]
name = "{{ vars.test_name | default(value="deterministic_test") }}"
version = "0.6.0"
description = "Deterministic test with reproducible results"

[vars]
seed_value = 12345
freeze_time = "2025-01-01T12:00:00Z"

[determinism]
seed = {{ vars.seed_value }}
freeze_clock = "{{ vars.freeze_time }}"

[otel]
exporter = "stdout"
resources = {
  "service.name" = "deterministic-test",
  "test.seed" = "{{ vars.seed_value }}",
  "test.frozen_time" = "{{ vars.freeze_time }}"
}

[service.deterministic_service]
plugin = "generic_container"
image = "alpine:latest"
args = ["sh", "-c", "date && echo $RANDOM"]

[[scenario]]
name = "deterministic_run_1"
service = "deterministic_service"
run = "echo 'Run 1 at {{ now_rfc3339() }}'"

[[scenario]]
name = "deterministic_run_2"
service = "deterministic_service"
run = "echo 'Run 2 at {{ now_rfc3339() }}'"

[[expect.span]]
name = "deterministic_service.*"
kind = "internal"
attrs.all = {
  "test.deterministic" = "true",
  "test.timestamp" = "{{ vars.freeze_time }}"
}

[report]
digest = "reports/deterministic_{{ sha256(s=vars.test_name) }}.sha256"
json = "reports/deterministic_{{ vars.test_name }}.json"

# The digest should be IDENTICAL across multiple runs with same seed and freeze_clock
"#.to_string())
}

/// Generate project from template
pub fn generate_from_template(template: &str, name: Option<&str>) -> Result<()> {
    let project_name = name.unwrap_or("cleanroom-project");

    info!(
        "Generating project from template: {} -> {}",
        template, project_name
    );
    debug!("Template: {}", template);

    // Check if template exists
    let available_templates = ["default", "advanced", "minimal", "database", "api"];
    if !available_templates.contains(&template) {
        return Err(CleanroomError::validation_error(format!(
            "Unknown template '{}'. Available templates: {}",
            template,
            available_templates.join(", ")
        )));
    }

    // Self-test: Create project structure
    let project_dir = std::path::Path::new(project_name);

    if project_dir.exists() {
        return Err(
            CleanroomError::validation_error("Project directory already exists")
                .with_context(format!("Directory: {}", project_name)),
        );
    }

    // Create directory structure
    std::fs::create_dir_all(project_dir)?;
    std::fs::create_dir_all(project_dir.join("tests"))?;
    std::fs::create_dir_all(project_dir.join("scenarios"))?;

    // Generate template-specific content
    match template {
        "default" => generate_default_template(project_dir, project_name)?,
        "advanced" => generate_advanced_template(project_dir, project_name)?,
        "minimal" => generate_minimal_template(project_dir, project_name)?,
        "database" => generate_database_template(project_dir, project_name)?,
        "api" => generate_api_template(project_dir, project_name)?,
        _ => unreachable!(), // Already validated above
    }

    info!("Project generated successfully: {}", project_name);
    Ok(())
}

fn generate_default_template(project_dir: &std::path::Path, project_name: &str) -> Result<()> {
    // Create basic test file
    let test_content = format!(
        r#"# Cleanroom Test Configuration
# Generated by clnrm template default

[test.metadata]
name = "{}"
description = "Default template test"
timeout = "120s"

[services.test_container]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[steps]]
name = "setup"
command = ["echo", "Setting up test environment"]
expected_output_regex = "Setting up test environment"

[[steps]]
name = "test"
command = ["echo", "Running test"]
expected_output_regex = "Running test"

[[steps]]
name = "cleanup"
command = ["echo", "Cleaning up"]
expected_output_regex = "Cleaning up"
"#,
        project_name
    );

    std::fs::write(
        project_dir.join("tests").join("basic.clnrm.toml"),
        test_content,
    )?;

    // Create README
    let readme_content = format!(
        r#"# {} - Cleanroom Test Project

This project uses the cleanroom testing framework for hermetic integration testing.

## Quick Start

```bash
# Run tests
clnrm run

# Validate configuration
clnrm validate tests/

# Show available plugins
clnrm plugins
```

## Project Structure

- `tests/` - Test configuration files
- `scenarios/` - Test scenario definitions
- `README.md` - This file

## Framework Self-Testing

This project demonstrates the cleanroom framework testing itself through the "eat your own dog food" principle.
"#,
        project_name
    );

    std::fs::write(project_dir.join("README.md"), readme_content)?;

    Ok(())
}

fn generate_advanced_template(project_dir: &std::path::Path, project_name: &str) -> Result<()> {
    // Create advanced test file with multiple scenarios
    let test_content = format!(
        r#"# Advanced Cleanroom Test Configuration
# Generated by clnrm template advanced

[test.metadata]
name = "{}"
description = "Advanced integration test with multiple services"
timeout = "300s"

[services.database]
type = "database"
plugin = "postgres"
image = "postgres:15"
env = {{ POSTGRES_PASSWORD = "testpass", POSTGRES_DB = "testdb" }}

[services.api_server]
type = "api"
plugin = "nginx"
image = "nginx:alpine"

[[steps]]
name = "setup_database"
command = ["echo", "Setting up database"]
expected_output_regex = "Setting up database"

[[steps]]
name = "setup_api"
command = ["echo", "Setting up API server"]
expected_output_regex = "Setting up API server"

[[steps]]
name = "run_integration_tests"
command = ["echo", "Running integration tests"]
expected_output_regex = "Running integration tests"

[[steps]]
name = "load_test"
command = ["echo", "Running load tests"]
expected_output_regex = "Running load tests"

[[steps]]
name = "benchmark"
command = ["echo", "Running benchmarks"]
expected_output_regex = "Running benchmarks"

[[steps]]
name = "cleanup"
command = ["echo", "Cleaning up"]
expected_output_regex = "Cleaning up"
"#,
        project_name
    );

    std::fs::write(
        project_dir.join("tests").join("advanced.clnrm.toml"),
        test_content,
    )?;

    // Create README
    let readme_content = format!(
        r#"# {} - Advanced Cleanroom Test Project

This project demonstrates advanced cleanroom testing patterns including:
- Multi-scenario testing
- Concurrent execution
- Service integration
- Performance testing

## Quick Start

```bash
# Run all tests
clnrm run

# Run specific test
clnrm run tests/advanced.toml

# Validate configuration
clnrm validate tests/

# Show available plugins
clnrm plugins
```

## Project Structure

- `tests/` - Test configuration files
- `scenarios/` - Test scenario definitions
- `README.md` - This file

## Advanced Features

- **Concurrent Testing**: Multiple scenarios run in parallel
- **Service Integration**: Database and API server services
- **Performance Testing**: Load testing and benchmarking
- **Policy Enforcement**: High security level with resource limits
"#,
        project_name
    );

    std::fs::write(project_dir.join("README.md"), readme_content)?;

    Ok(())
}

fn generate_minimal_template(project_dir: &std::path::Path, project_name: &str) -> Result<()> {
    // Create minimal test file
    let test_content = format!(
        r#"# Minimal Cleanroom Test Configuration
# Generated by clnrm template minimal

[test.metadata]
name = "{}"
description = "Minimal test"
timeout = "60s"

[services.test_container]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[steps]]
name = "simple_test"
command = ["echo", "Hello from cleanroom!"]
expected_output_regex = "Hello from cleanroom!"
"#,
        project_name
    );

    std::fs::write(
        project_dir.join("tests").join("minimal.clnrm.toml"),
        test_content,
    )?;

    // Create minimal README
    let readme_content = format!(
        r#"# {} - Minimal Cleanroom Test

Simple cleanroom test project.

```bash
clnrm run
```
"#,
        project_name
    );

    std::fs::write(project_dir.join("README.md"), readme_content)?;

    Ok(())
}

fn generate_database_template(project_dir: &std::path::Path, project_name: &str) -> Result<()> {
    // Create database-focused test file
    let test_content = format!(
        r#"# Database Integration Test Configuration
# Generated by clnrm template database

[test.metadata]
name = "{}"
description = "Database integration test"
timeout = "300s"

[services.postgres]
type = "database"
plugin = "postgres"
image = "postgres:15"
env = {{ 
    POSTGRES_PASSWORD = "testpass",
    POSTGRES_DB = "testdb",
    POSTGRES_USER = "testuser"
}}

[services.redis]
type = "cache"
plugin = "redis"
image = "redis:7-alpine"

[[steps]]
name = "setup_database"
command = ["echo", "Setting up database"]
expected_output_regex = "Setting up database"

[[steps]]
name = "create_tables"
command = ["echo", "Creating tables"]
expected_output_regex = "Creating tables"

[[steps]]
name = "insert_data"
command = ["echo", "Inserting test data"]
expected_output_regex = "Inserting test data"

[[steps]]
name = "run_queries"
command = ["echo", "Running test queries"]
expected_output_regex = "Running test queries"

[[steps]]
name = "cleanup"
command = ["echo", "Cleaning up database"]
expected_output_regex = "Cleaning up database"
"#,
        project_name
    );

    std::fs::write(
        project_dir.join("tests").join("database.clnrm.toml"),
        test_content,
    )?;

    // Create README
    let readme_content = format!(
        r#"# {} - Database Integration Test Project

This project focuses on database integration testing with cleanroom.

## Quick Start

```bash
# Run database tests
clnrm run

# Validate configuration
clnrm validate tests/

# Show available plugins
clnrm plugins
```

## Services

- **PostgreSQL**: Main database service
- **Redis**: Caching service

## Test Scenarios

- Database setup and teardown
- Table creation and data insertion
- Query testing and validation
- Cleanup procedures
"#,
        project_name
    );

    std::fs::write(project_dir.join("README.md"), readme_content)?;

    Ok(())
}

fn generate_api_template(project_dir: &std::path::Path, project_name: &str) -> Result<()> {
    // Create API-focused test file
    let test_content = format!(
        r#"# API Integration Test Configuration
# Generated by clnrm template api

[test.metadata]
name = "{}"
description = "API integration test"
timeout = "300s"

[services.api_server]
type = "api"
plugin = "nginx"
image = "nginx:alpine"
env = {{ 
    NGINX_HOST = "0.0.0.0",
    NGINX_PORT = "8080"
}}

[services.database]
type = "database"
plugin = "postgres"
image = "postgres:15"
env = {{ 
    POSTGRES_PASSWORD = "testpass",
    POSTGRES_DB = "apidb"
}}

[[steps]]
name = "start_api"
command = ["echo", "Starting API server"]
expected_output_regex = "Starting API server"

[[steps]]
name = "health_check"
command = ["echo", "Checking API health"]
expected_output_regex = "Checking API health"

[[steps]]
name = "test_endpoints"
command = ["echo", "Testing API endpoints"]
expected_output_regex = "Testing API endpoints"

[[steps]]
name = "load_test"
command = ["echo", "Running load tests"]
expected_output_regex = "Running load tests"

[[steps]]
name = "cleanup"
command = ["echo", "Stopping API server"]
expected_output_regex = "Stopping API server"
"#,
        project_name
    );

    std::fs::write(
        project_dir.join("tests").join("api.clnrm.toml"),
        test_content,
    )?;

    // Create README
    let readme_content = format!(
        r#"# {} - API Integration Test Project

This project focuses on API integration testing with cleanroom.

## Quick Start

```bash
# Run API tests
clnrm run

# Validate configuration
clnrm validate tests/

# Show available plugins
clnrm plugins
```

## Services

- **API Server**: Nginx-based API server
- **Database**: PostgreSQL backend

## Test Scenarios

- API server startup and health checks
- Endpoint testing and validation
- Load testing and performance validation
- Cleanup and teardown procedures
"#,
        project_name
    );

    std::fs::write(project_dir.join("README.md"), readme_content)?;

    Ok(())
}
