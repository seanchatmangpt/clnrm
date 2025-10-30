# Error Handling Reference

This chapter provides comprehensive documentation for clnrm's error handling system, including error types, context, and recovery strategies.

## Overview

clnrm uses a structured error handling system based on:

- **`CleanroomError`** - Main error type with context and source information
- **Error categories** - Different error types for different failure modes
- **Error context** - Detailed information about where and why errors occurred
- **Recovery strategies** - How to handle and recover from errors

## Error Types

### CleanroomError Structure

```rust
pub struct CleanroomError {
    pub category: ErrorCategory,
    pub message: String,
    pub context: HashMap<String, String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub span_context: Option<tracing::SpanId>,
}
```

### Error Categories

| Category | Description | Common Causes |
|----------|-------------|---------------|
| `Configuration` | Invalid configuration | Malformed TOML, missing fields |
| `Validation` | Validation failures | Schema violations, constraint failures |
| `Execution` | Runtime execution errors | Command failures, timeouts |
| `Container` | Container-related errors | Docker failures, image issues |
| `Network` | Network-related errors | Connection failures, DNS issues |
| `Plugin` | Plugin-related errors | Plugin initialization, lifecycle failures |
| `Service` | Service-related errors | Service startup, health check failures |
| `OTEL` | OpenTelemetry errors | Span creation, export failures |
| `Template` | Template processing errors | Tera rendering, variable resolution |
| `Internal` | Internal framework errors | Unexpected states, logic errors |

## Error Creation Patterns

### Configuration Errors

```rust
use clnrm_core::error::{CleanroomError, ErrorCategory};

// Configuration error with context
CleanroomError::configuration_error("Invalid TOML configuration")
    .with_context("file", "test.toml")
    .with_context("line", "23")
    .with_context("field", "services.api.image")
    .with_source("Missing required field: image")
```

### Validation Errors

```rust
// Validation error with detailed context
CleanroomError::validation_error("Port number out of range")
    .with_context("field", "services.api.ports")
    .with_context("value", "99999")
    .with_context("constraint", "must be between 1024 and 65535")
```

### Execution Errors

```rust
// Execution error with command context
CleanroomError::execution_error("Command failed")
    .with_context("command", "curl http://localhost:80/health")
    .with_context("exit_code", "7")
    .with_context("stderr", "Failed to connect to localhost:80")
```

### Container Errors

```rust
// Container error with container context
CleanroomError::container_error("Container failed to start")
    .with_context("container_id", "abc123def456")
    .with_context("image", "nginx:alpine")
    .with_context("exit_code", "1")
    .with_context("logs", "nginx: [error] invalid configuration")
```

## Error Context Patterns

### Structured Context

Add structured context to errors:

```rust
fn validate_service_config(config: &ServiceConfig) -> Result<(), CleanroomError> {
    if config.image.is_empty() {
        return Err(CleanroomError::validation_error("Service image is required")
            .with_context("service_name", &config.name)
            .with_context("service_type", &config.service_type)
            .with_context("field", "image")
            .with_context("validation_rule", "non_empty"));
    }

    if config.ports.iter().any(|&p| p < 1024 || p > 65535) {
        return Err(CleanroomError::validation_error("Invalid port number")
            .with_context("service_name", &config.name)
            .with_context("invalid_ports", "80,443,8080")
            .with_context("valid_range", "1024-65535"));
    }

    Ok(())
}
```

### Chained Context

Build context across function calls:

```rust
fn execute_test_step(step: &TestStep) -> Result<(), CleanroomError> {
    let _span = tracing::info_span!("test_step", step_name = step.name);

    self.execute_command(&step.command)
        .map_err(|e| e.with_context("step_name", &step.name)
                    .with_context("step_index", "1")
                    .with_context("test_name", &self.test_name))?;

    Ok(())
}

fn execute_command(&self, command: &[String]) -> Result<(), CleanroomError> {
    let output = std::process::Command::new(&command[0])
        .args(&command[1..])
        .output()
        .map_err(|e| CleanroomError::execution_error("Command execution failed")
            .with_context("command", &command.join(" "))
            .with_context("working_directory", &self.working_directory)
            .with_source(e))?;

    if !output.status.success() {
        return Err(CleanroomError::execution_error("Command returned non-zero exit code")
            .with_context("command", &command.join(" "))
            .with_context("exit_code", &output.status.code().unwrap_or(-1).to_string())
            .with_context("stdout", &String::from_utf8_lossy(&output.stdout))
            .with_context("stderr", &String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}
```

## Error Recovery Strategies

### Retry with Backoff

Implement retry logic with exponential backoff:

```rust
use tokio::time::{sleep, Duration};

async fn execute_with_retry<F, T>(
    operation: F,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T, CleanroomError>
where
    F: Fn() -> Result<T, CleanroomError>,
{
    let mut last_error = None;

    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < max_retries - 1 {
                    let delay = base_delay * 2_u32.pow(attempt);
                    tracing::warn!("Operation failed (attempt {}), retrying in {:?}", attempt + 1, delay);
                    sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

// Usage
let result = execute_with_retry(
    || self.start_container(),
    3,
    Duration::from_secs(1),
).await?;
```

### Circuit Breaker Pattern

Implement circuit breaker for failing operations:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

#[derive(Debug)]
enum CircuitBreakerState {
    Closed { failures: u32 },
    Open { next_attempt: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, operation: F) -> Result<T, CleanroomError>
    where
        F: FnOnce() -> Result<T, CleanroomError>,
    {
        // Check if circuit is open
        {
            let state = self.state.read().await;
            match state {
                CircuitBreakerState::Open { next_attempt } => {
                    if Instant::now() < *next_attempt {
                        return Err(CleanroomError::service_error("Circuit breaker is open"));
                    }
                }
                _ => {}
            }
        }

        // Execute operation
        match operation() {
            Ok(result) => {
                // Reset failure count on success
                let mut state = self.state.write().await;
                if let CircuitBreakerState::Closed { .. } = *state {
                    // Success - reset failures
                }
                Ok(result)
            }
            Err(e) => {
                // Increment failure count
                let mut state = self.state.write().await;
                match *state {
                    CircuitBreakerState::Closed { mut failures } => {
                        failures += 1;
                        if failures >= self.failure_threshold {
                            *state = CircuitBreakerState::Open {
                                next_attempt: Instant::now() + self.recovery_timeout,
                            };
                        } else {
                            *state = CircuitBreakerState::Closed { failures };
                        }
                    }
                    _ => {}
                }
                Err(e)
            }
        }
    }
}
```

### Graceful Degradation

Handle errors with graceful degradation:

```rust
async fn execute_with_fallback(
    &self,
    primary: impl FnOnce() -> Result<(), CleanroomError>,
    fallback: impl FnOnce() -> Result<(), CleanroomError>,
) -> Result<(), CleanroomError> {
    match primary() {
        Ok(()) => Ok(()),
        Err(primary_error) => {
            tracing::warn!("Primary operation failed: {}", primary_error);

            match fallback() {
                Ok(()) => {
                    tracing::info!("Fallback operation succeeded");
                    Ok(())
                }
                Err(fallback_error) => {
                    tracing::error!("Both primary and fallback operations failed");
                    Err(CleanroomError::execution_error("All execution strategies failed")
                        .with_context("primary_error", &primary_error.to_string())
                        .with_context("fallback_error", &fallback_error.to_string()))
                }
            }
        }
    }
}

// Usage
self.execute_with_fallback(
    || self.execute_in_container(),
    || self.execute_on_host(),
).await?;
```

## Error Propagation

### Proper Error Propagation

Propagate errors with additional context:

```rust
impl TestExecutor {
    pub async fn run_test(&self, test: &TestConfig) -> Result<TestResult, CleanroomError> {
        let _span = tracing::info_span!("test_execution", test_name = test.metadata.name);

        // Validate test configuration
        self.validate_test_config(test)
            .map_err(|e| e.with_context("test_name", &test.metadata.name))?;

        // Execute test steps
        let mut results = Vec::new();
        for (i, step) in test.steps.iter().enumerate() {
            let result = self.execute_step(step).await
                .map_err(|e| e.with_context("test_name", &test.metadata.name)
                            .with_context("step_index", &(i + 1).to_string())
                            .with_context("step_name", &step.name))?;
            results.push(result);
        }

        Ok(TestResult { results })
    }
}
```

### Error Aggregation

Aggregate multiple errors:

```rust
async fn execute_parallel_steps(&self, steps: &[TestStep]) -> Result<Vec<StepResult>, CleanroomError> {
    let tasks: Vec<_> = steps.iter()
        .enumerate()
        .map(|(i, step)| async move {
            self.execute_step(step).await
                .map_err(|e| (i, e))
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    let mut step_results = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(result) => step_results.push(result),
            Err((index, error)) => {
                errors.push((index, error));
            }
        }
    }

    if errors.is_empty() {
        Ok(step_results)
    } else {
        // Aggregate errors with context
        let mut aggregated_error = CleanroomError::execution_error("Multiple steps failed");
        for (index, error) in errors {
            aggregated_error = aggregated_error.with_context(
                &format!("step_{}", index),
                &error.to_string()
            );
        }
        Err(aggregated_error)
    }
}
```

## Error Logging and Monitoring

### Structured Error Logging

Log errors with structured information:

```rust
impl TestExecutor {
    fn log_error(&self, error: &CleanroomError, context: &str) {
        tracing::error!(
            error.category = %error.category,
            error.message = %error.message,
            error.context = ?error.context,
            error.timestamp = %error.timestamp,
            operation = context,
            "Test execution error"
        );

        if let Some(source) = &error.source {
            tracing::debug!(
                error.source = %source,
                "Error source details"
            );
        }
    }
}
```

### Error Metrics

Track error metrics for monitoring:

```rust
use prometheus::{Counter, Histogram};

pub struct ErrorMetrics {
    errors_total: Counter,
    error_latency: Histogram,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            errors_total: Counter::new("errors_total", "Total number of errors")
                .unwrap(),
            error_latency: Histogram::new("error_latency_seconds", "Error handling latency")
                .unwrap(),
        }
    }

    pub fn record_error(&self, error: &CleanroomError) {
        self.errors_total
            .with_label_values(&[&error.category.to_string()])
            .inc();

        // Record error handling latency
        let start = std::time::Instant::now();
        // Error handling logic
        let duration = start.elapsed();
        self.error_latency.observe(duration.as_secs_f64());
    }
}
```

## Error Testing

### Testing Error Conditions

Test error handling in your code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_handling() -> Result<(), CleanroomError> {
        let executor = TestExecutor::new();

        // Test configuration error
        let invalid_config = TestConfig {
            metadata: TestMetadata {
                name: "".to_string(), // Invalid: empty name
                description: "Test description".to_string(),
                version: "1.0.0".to_string(),
            },
            services: HashMap::new(),
            steps: vec![],
        };

        let result = executor.run_test(&invalid_config).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.category, ErrorCategory::Validation);
        assert!(error.message.contains("name"));
        assert!(error.context.contains_key("field"));

        Ok(())
    }

    #[tokio::test]
    async fn test_error_recovery() -> Result<(), CleanroomError> {
        let executor = TestExecutor::new();

        // Test retry logic
        let result = executor.execute_with_retry(
            || async {
                // Simulate flaky operation that fails twice then succeeds
                static mut CALL_COUNT: u32 = 0;
                unsafe {
                    CALL_COUNT += 1;
                    if CALL_COUNT <= 2 {
                        Err(CleanroomError::execution_error("Temporary failure"))
                    } else {
                        Ok(())
                    }
                }
            },
            3,
            Duration::from_millis(100),
        ).await;

        assert!(result.is_ok());
        Ok(())
    }
}
```

## Best Practices

### 1. Use Proper Error Types

```rust
// ✅ Good: Use appropriate error types
if config.name.is_empty() {
    return Err(CleanroomError::validation_error("Test name is required"));
}

if port < 1024 || port > 65535 {
    return Err(CleanroomError::validation_error("Invalid port range"));
}
```

### 2. Add Rich Context

```rust
// ✅ Good: Rich error context
Err(CleanroomError::container_error("Container failed to start")
    .with_context("container_id", container_id)
    .with_context("image", image_name)
    .with_context("command", command.join(" "))
    .with_context("exit_code", exit_code.to_string()))
```

### 3. Implement Recovery Strategies

```rust
// ✅ Good: Recovery strategies
self.execute_with_retry(operation, 3, Duration::from_secs(1)).await
    .or_else(|_| self.execute_with_fallback(primary, fallback)).await
```

### 4. Log Errors Appropriately

```rust
// ✅ Good: Appropriate error logging
tracing::error!(
    error.category = %error.category,
    error.message = %error.message,
    "Operation failed"
);
```

## Common Error Patterns

### Configuration Error Pattern

```rust
fn validate_config(config: &TestConfig) -> Result<(), CleanroomError> {
    // Validate metadata
    if config.metadata.name.is_empty() {
        return Err(CleanroomError::validation_error("Test name is required")
            .with_context("section", "test.metadata")
            .with_context("field", "name")
            .with_context("validation_rule", "non_empty"));
    }

    // Validate services
    for (name, service) in &config.services {
        if service.image.is_empty() {
            return Err(CleanroomError::validation_error("Service image is required")
                .with_context("section", "services")
                .with_context("service_name", name)
                .with_context("field", "image"));
        }
    }

    Ok(())
}
```

### Execution Error Pattern

```rust
async fn execute_step(&self, step: &TestStep) -> Result<StepResult, CleanroomError> {
    let start_time = Instant::now();

    let output = self.execute_command(&step.command).await
        .map_err(|e| e.with_context("step_name", &step.name)
                    .with_context("command", &step.command.join(" ")))?;

    let duration = start_time.elapsed();

    Ok(StepResult {
        name: step.name.clone(),
        success: true,
        output,
        duration,
    })
}
```

### Plugin Error Pattern

```rust
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        self.start_container()
            .map_err(|e| CleanroomError::plugin_error("Plugin start failed")
                .with_context("plugin_name", &self.name)
                .with_context("plugin_version", &self.version)
                .with_source(e))
    }
}
```

## Next Steps

Now that you understand error handling:

1. **Implement error handling**: Add proper error handling to your plugins and tests
2. **Test error scenarios**: Write tests for error conditions
3. **Monitor errors**: Set up error monitoring and alerting
4. **Review other chapters**: Check the other chapters for advanced usage patterns

## Further Reading

- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Structured Error Handling](https://nick.groenen.me/posts/structured-error-handling/)
- [Error Context Patterns](https://www.lpalmieri.com/posts/error-context/)

