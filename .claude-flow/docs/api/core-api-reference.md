# Core API Reference

Comprehensive reference for Cleanroom's core API components following 80/20 principles - 80% of the value with 20% of the documentation complexity.

## Table of Contents

1. [Essential Types](#essential-types)
2. [Core Functions](#core-functions)
3. [Result Types](#result-types)
4. [Error Types](#error-types)
5. [Configuration](#configuration)
6. [Usage Patterns](#usage-patterns)
7. [Best Practices](#best-practices)

## Essential Types

### CleanroomEnvironment

The central orchestrator for hermetic testing environments with plugin-based service management.

```rust
use clnrm::{CleanroomEnvironment, ServicePlugin, ServiceHandle, HealthStatus};

// Note: Most methods currently return "not implemented" errors
// This is a framework for building comprehensive testing environments

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create environment (currently returns "not implemented")
    let environment = CleanroomEnvironment::new().await?;

    // Register service plugins (not implemented yet)
    let plugin = Box::new(MockDatabasePlugin::new());
    environment.register_service(plugin).await?;

    // Start services (not implemented yet)
    let handle = environment.start_service("mock_database").await?;

    // Check health (not implemented yet)
    let health = environment.check_health().await;

    Ok(())
}
```

**Current Implementation Status:**
- **Mostly Stubbed**: Most methods return "not implemented" errors
- **Plugin Architecture**: Uses plugin-based service registry
- **Framework Ready**: Infrastructure in place for full implementation

**Key Architecture:**
- `ServicePlugin` trait for extensible service management
- `ServiceRegistry` for managing service instances
- `ServiceHandle` for service lifecycle management
- Plugin-based approach replaces hardcoded containers

## Core Functions

### scenario() (Primary API)

Create and execute multi-step test scenarios - this is the main API for Cleanroom.

```rust
use clnrm::{scenario, Policy, SecurityLevel};

// Create a multi-step scenario (ACTUALLY IMPLEMENTED)
let result = scenario("integration_test")
    .step("setup", ["echo", "setting up test environment"])
    .step("execute", ["echo", "running main test logic"])
    .step("verify", ["echo", "verifying test results"])
    .run()?;

println!("Scenario completed in {}ms", result.duration_ms);
println!("Steps executed: {}", result.steps.len());

// Check individual step results
for step in &result.steps {
    println!("Step '{}': {} ({}ms)",
        step.name,
        if step.success { "PASSED" } else { "FAILED" },
        step.duration_ms
    );
}
```

**Features (ACTUALLY IMPLEMENTED):**
- **Multi-step workflows**: Define complex testing scenarios with multiple steps
- **Error handling**: Automatic rollback and cleanup on step failures
- **Concurrent execution**: Run steps in parallel for improved performance
- **Deterministic execution**: Reproducible results with seeded randomness
- **Comprehensive reporting**: Detailed step-by-step execution results

### Policy System (Fully Implemented)

Security and resource policy configuration with comprehensive validation.

```rust
use clnrm::{Policy, SecurityLevel, SecurityPolicy, ResourcePolicy};

// Create policy with security level (ACTUALLY IMPLEMENTED)
let policy = Policy::with_security_level(SecurityLevel::Standard);

// Custom security policy (ACTUALLY IMPLEMENTED)
let mut policy = Policy::default();
policy.security.allowed_ports = vec![5432, 6379, 8080];
policy.security.blocked_addresses = vec!["127.0.0.1".to_string()];
policy.security.enable_network_isolation = true;

// Resource limits (ACTUALLY IMPLEMENTED)
policy.resources.max_cpu_usage_percent = 80.0;
policy.resources.max_memory_usage_bytes = 1024 * 1024 * 1024; // 1GB

// Policy validation (ACTUALLY IMPLEMENTED)
match policy.validate() {
    Ok(()) => println!("Policy is valid"),
    Err(e) => println!("Policy validation failed: {}", e),
}
```

**Policy Enforcement (ACTUALLY IMPLEMENTED):**
- **Runtime validation**: Policies are validated before execution
- **Operation checking**: Operations are checked against policy constraints
- **Environment variables**: Policies are exported as environment variables
- **Security levels**: Five security levels from Low to Locked

## Result Types

### RunResult (Scenario Results)

Multi-step scenario execution result with comprehensive metadata.

```rust
use clnrm::scenario;

// Execute scenario (ACTUALLY IMPLEMENTED)
let result = scenario("integration_test")
    .step("setup", ["echo", "setting up"])
    .step("execute", ["echo", "running test"])
    .step("verify", ["echo", "verifying results"])
    .run()?;

println!("Exit code: {}", result.exit_code);
println!("Success: {}", result.success());
println!("Stdout: {}", result.stdout);
println!("Stderr: {}", result.stderr);
println!("Duration: {}ms", result.duration_ms);
println!("Backend: {}", result.backend);
println!("Concurrent: {}", result.concurrent);
println!("Step order: {:?}", result.step_order);

// Check individual step results (ACTUALLY IMPLEMENTED)
for step in &result.steps {
    println!("Step '{}': exit_code={}, duration={}ms, success={}",
        step.name, step.exit_code, step.duration_ms, step.success);
}

// Check for specific content in combined output
if result.stdout.contains("expected content") {
    println!("Found expected content in scenario output!");
}
```

**Key Fields (ACTUALLY IMPLEMENTED):**
- `exit_code: i32` - Exit code of the last step (0 = success)
- `stdout: String` - Combined standard output from all steps
- `stderr: String` - Combined standard error from all steps
- `duration_ms: u64` - Total execution time in milliseconds
- `steps: Vec<StepResult>` - Individual step execution results
- `backend: String` - Backend used for execution
- `concurrent: bool` - Whether execution was concurrent
- `step_order: Vec<String>` - Execution order of steps

### StepResult

Individual step execution result within a scenario.

```rust
// Each step result contains detailed information
for step in &result.steps {
    println!("Step: {}", step.name);
    println!("  Exit code: {}", step.exit_code);
    println!("  Success: {}", step.success);
    println!("  Duration: {}ms", step.duration_ms);
    println!("  Start time: {}", step.start_ts);
    println!("  Stdout: {}", step.stdout);
    println!("  Stderr: {}", step.stderr);
    println!("  Source: {}", step.source);
}
```

**StepResult Fields:**
- `name: String` - Step name/label
- `exit_code: i32` - Step exit code
- `stdout: String` - Step standard output
- `stderr: String` - Step standard error
- `duration_ms: u64` - Step execution time
- `start_ts: u64` - Step start timestamp
- `success: bool` - Whether step succeeded
- `source: String` - Source of the step

## Error Types

### CleanroomError

Comprehensive error type with rich context and structured error kinds.

```rust
use clnrm::{scenario, CleanroomError, ErrorKind};

// Scenario execution with error handling (ACTUALLY IMPLEMENTED)
match scenario("test_scenario")
    .step("setup", ["echo", "setup"])
    .step("execute", ["invalid_command"]) // This will fail
    .run()
{
    Ok(result) => {
        println!("Scenario succeeded");
    }
    Err(CleanroomError { kind: ErrorKind::ContainerError, message, .. }) => {
        println!("Container error: {}", message);
    }
    Err(CleanroomError { kind: ErrorKind::PolicyViolation, message, .. }) => {
        println!("Policy violation: {}", message);
    }
    Err(CleanroomError { kind: ErrorKind::ResourceLimitExceeded, message, .. }) => {
        println!("Resource limit exceeded: {}", message);
    }
    Err(CleanroomError { kind: ErrorKind::Timeout, message, .. }) => {
        println!("Timeout error: {}", message);
    }
    Err(e) => {
        println!("Other error: {} ({:?})", e.message, e.kind);
    }
}
```

**Error Structure:**
```rust
pub struct CleanroomError {
    pub kind: ErrorKind,           // Structured error category
    pub message: String,           // Human-readable message
    pub context: Option<String>,   // Additional context
    pub source: Option<String>,    // Source error (if chained)
    pub timestamp: DateTime<Utc>,  // When error occurred
}
```

**Error Categories (ACTUALLY IMPLEMENTED):**
- `ContainerError` - Container lifecycle and execution failures
- `NetworkError` - Network connectivity and access issues
- `ResourceLimitExceeded` - Resource usage limit violations
- `Timeout` - Operation timeout errors
- `ConfigurationError` - Invalid configuration
- `PolicyViolation` - Security policy violations
- `DeterministicError` - Deterministic execution failures
- `ValidationError` - Input validation failures
- `ServiceError` - Service-related errors
- `InternalError` - Internal framework errors

## Usage Patterns

### Scenario Pattern (Primary Usage)

```rust
use clnrm::{scenario, Policy, SecurityLevel};

// Pattern 1: Basic multi-step scenario (ACTUALLY IMPLEMENTED)
let result = scenario("integration_test")
    .step("setup", ["echo", "setting up test environment"])
    .step("execute", ["echo", "running main test logic"])
    .step("verify", ["echo", "verifying test results"])
    .run()?;

assert!(result.success());
assert!(result.stdout.contains("test environment"));

// Pattern 2: Scenario with policy (ACTUALLY IMPLEMENTED)
let policy = Policy::with_security_level(SecurityLevel::Standard);
let result = scenario("secure_test")
    .with_policy(policy)
    .step("secure_setup", ["echo", "secure environment setup"])
    .step("execute", ["echo", "running secure test"])
    .run()?;

// Pattern 3: Concurrent scenario (ACTUALLY IMPLEMENTED)
let result = scenario("concurrent_test")
    .concurrent()  // Enable concurrent execution
    .step("task1", ["echo", "running task 1"])
    .step("task2", ["echo", "running task 2"])
    .step("task3", ["echo", "running task 3"])
    .run()?;
```

### Policy Pattern (Fully Implemented)

```rust
use clnrm::{Policy, SecurityLevel};

// Pattern 1: Security level policy (ACTUALLY IMPLEMENTED)
let policy = Policy::with_security_level(SecurityLevel::Standard);

// Pattern 2: Custom security policy (ACTUALLY IMPLEMENTED)
let mut policy = Policy::default();
policy.security.allowed_ports = vec![5432, 6379, 8080];
policy.security.enable_network_isolation = true;
policy.security.enable_data_redaction = true;

// Pattern 3: Resource limits (ACTUALLY IMPLEMENTED)
policy.resources.max_cpu_usage_percent = 80.0;
policy.resources.max_memory_usage_bytes = 1024 * 1024 * 1024; // 1GB
policy.resources.max_container_count = 10;

// Pattern 4: Execution policies (ACTUALLY IMPLEMENTED)
policy.execution.enable_deterministic_execution = true;
policy.execution.deterministic_seed = Some(42);
policy.execution.enable_parallel_execution = true;
policy.execution.max_parallel_tasks = 4;

// Pattern 5: Policy validation (ACTUALLY IMPLEMENTED)
match policy.validate() {
    Ok(()) => println!("Policy is valid"),
    Err(e) => println!("Policy validation failed: {}", e),
}
```

### Error Handling Pattern (Actually Implemented)

```rust
use clnrm::{scenario, CleanroomError, ErrorKind};

// Pattern 1: Scenario error handling (ACTUALLY IMPLEMENTED)
match scenario("failing_test")
    .step("setup", ["echo", "setup"])
    .step("execute", ["false"]) // This will fail
    .run()
{
    Ok(result) => println!("Scenario succeeded"),
    Err(CleanroomError { kind: ErrorKind::ContainerError, message, .. }) => {
        println!("Container error: {}", message);
        // Handle container-specific errors
    }
    Err(CleanroomError { kind: ErrorKind::PolicyViolation, message, .. }) => {
        println!("Policy violation: {}", message);
        // Handle policy violations
    }
    Err(e) => {
        println!("Other error: {} ({:?})", e.message, e.kind);
        // Handle unexpected errors
    }
}

// Pattern 2: Step-level error analysis (ACTUALLY IMPLEMENTED)
let result = scenario("test_with_errors")
    .step("working_step", ["echo", "this works"])
    .step("failing_step", ["false"])
    .run()?;

for step in &result.steps {
    if !step.success {
        println!("Failed step: {} - {}", step.name, step.stderr);
    }
}
```

### Deterministic Execution Pattern (Actually Implemented)

```rust
use clnrm::scenario;

// Pattern 1: Deterministic scenario (ACTUALLY IMPLEMENTED)
let result1 = scenario("deterministic_test")
    .deterministic(Some(42)) // Fixed seed for reproducible results
    .step("step1", ["echo", "deterministic output"])
    .run()?;

let result2 = scenario("deterministic_test")
    .deterministic(Some(42)) // Same seed = same results
    .step("step1", ["echo", "deterministic output"])
    .run()?;

assert_eq!(result1.stdout, result2.stdout); // Deterministic results

// Pattern 2: Random but deterministic (ACTUALLY IMPLEMENTED)
let result = scenario("random_test")
    .deterministic(Some(123))
    .step("random_output", ["echo", "pseudo-random output"])
    .run()?;

// Same seed always produces same "random" output

## Best Practices

### 1. Scenario Design (80/20)

```rust
// Good: Use descriptive step names and clear separation of concerns
let result = scenario("user_registration_flow")
    .step("setup_database", ["echo", "initializing test database"])
    .step("create_user", ["echo", "creating test user"])
    .step("verify_user", ["echo", "verifying user creation"])
    .step("cleanup", ["echo", "cleaning up test data"])
    .run()?;

// Good: Use concurrent execution for independent steps
let result = scenario("parallel_operations")
    .concurrent()
    .step("service_a", ["echo", "starting service A"])
    .step("service_b", ["echo", "starting service B"])
    .step("service_c", ["echo", "starting service C"])
    .run()?;

// Avoid: Too many steps in a single scenario
// Avoid: Mixing setup, execution, and verification in one step
```

### 2. Policy Configuration (80/20)

```rust
// Good: Start with security levels and customize as needed
let policy = Policy::with_security_level(SecurityLevel::Standard)
    .with_network_isolation(true)  // Enable for production
    .with_resource_limits(80.0, 1024 * 1024 * 1024, 10 * 1024 * 1024 * 1024);

// Good: Configure resource limits based on your workload
policy.resources.max_container_count = 5;  // Limit concurrent containers
policy.resources.max_test_execution_time = Duration::from_secs(300);

// Avoid: Overly restrictive policies that break functionality
// Avoid: No resource limits (resource exhaustion risk)
```

### 3. Error Handling (80/20)

```rust
// Good: Handle specific error types appropriately
match scenario("test").run() {
    Ok(result) => println!("Test passed"),
    Err(CleanroomError { kind: ErrorKind::PolicyViolation, .. }) => {
        println!("Policy violation - check security settings");
        // Adjust policy or disable problematic features
    }
    Err(CleanroomError { kind: ErrorKind::ResourceLimitExceeded, .. }) => {
        println!("Resource limit exceeded - optimize resource usage");
        // Reduce container count or increase limits
    }
    Err(e) => {
        println!("Unexpected error: {}", e.message);
        // Log for investigation
    }
}

// Avoid: Generic error handling that loses context
```

## Quick Reference

### Most Used Types (80% of usage)

1. `scenario()` - Create multi-step test scenarios (PRIMARY API)
2. `Policy` - Security and resource policies
3. `SecurityLevel` - Five security levels (Low to Locked)
4. `RunResult` - Scenario execution results
5. `CleanroomError` - Structured error handling

### Most Used Functions (80% of usage)

1. `scenario().step().run()` - Execute multi-step scenarios
2. `Policy::with_security_level()` - Create security policies
3. `policy.validate()` - Validate policy configuration
4. `result.success()` - Check scenario success
5. `result.steps` - Access individual step results

### Most Used Patterns (80% of usage)

1. **Basic scenario**: `scenario("test").step("name", ["cmd"]).run()?`
2. **Security policy**: `Policy::with_security_level(SecurityLevel::Standard)`
3. **Error handling**: `match scenario().run() { Ok(_) => ..., Err(_) => ... }`
4. **Deterministic execution**: `scenario().deterministic(Some(seed)).run()?`
5. **Concurrent execution**: `scenario().concurrent().run()?`

## Implementation Status

### ✅ **Fully Implemented**
- **Scenario DSL**: Multi-step test orchestration with error handling
- **Policy System**: Complete security and resource policy framework
- **Error Handling**: Structured error types with rich context
- **Deterministic Execution**: Reproducible results with seeded randomness
- **Concurrent Execution**: Parallel step execution for performance

### 🚧 **Framework Ready (Not Implemented)**
- **CleanroomEnvironment**: Plugin-based service registry (infrastructure exists)
- **Service Management**: Extensible service plugin system
- **Container Management**: Testcontainers integration (backend exists)
- **Advanced Observability**: OpenTelemetry integration (framework exists)

### 📋 **Architecture Notes**
- Plugin-based architecture allows for extensible service management
- CleanroomEnvironment provides framework for comprehensive testing environments
- Backend abstraction supports multiple container runtimes
- Service registry pattern enables flexible service composition

## Resources

- **Scenario Guide**: [../guides/scenario-dsl-guide.md](../guides/scenario-dsl-guide.md)
- **Security Guide**: [../guides/security-guide.md](../guides/security-guide.md)
- **Getting Started**: [../guides/getting-started-tutorial.md](../guides/getting-started-tutorial.md)
- **Architecture**: [../architecture-overview.md](../architecture-overview.md)

---

*This core API reference accurately reflects the current implementation. The scenario DSL and policy system are fully functional, while CleanroomEnvironment provides a framework for future enhancements.*
```
