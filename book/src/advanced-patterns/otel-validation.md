# OTEL Validation (Legacy)

**⚠️ IMPORTANT: This chapter describes the v1.0.x approach to OTEL validation. For v1.2.1+, see [Weaver Schema Validation](weaver-validation.md) which is now the recommended approach.**

OTEL (OpenTelemetry) validation in v1.0.x used TOML-based span expectations. v1.2.1 replaces this with **Weaver live-check** with health checks for schema-driven validation that prevents false positives.

## Overview

clnrm uses OTEL for:
- **Span validation** - Verify expected spans are created with correct attributes
- **Trace analysis** - Validate trace topology and relationships
- **Temporal constraints** - Ensure proper timing between operations
- **Attribute validation** - Verify span attributes match expectations
- **Graph validation** - Validate parent-child relationships and trace structure

## Basic Span Validation

### Single Span Validation

Validate individual spans with attributes:

```toml
[test.metadata]
name = "basic_span_validation"
description = "Validate basic span creation and attributes"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# OTEL configuration
[otel]
exporter = "stdout"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0

[[steps]]
name = "api_request"
description = "Make API request to generate spans"
command = ["curl", "-f", "http://localhost:80/health"]
expected_output_regex = ".*"

# Expected spans
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = {
    "http.method" = "GET",
    "http.route" = "/health",
    "http.status_code" = "200",
    "http.user_agent" = "curl/.*"
}

[[expect.span]]
name = "api.response"
kind = "server"
attrs.all = {
    "http.status_code" = "200",
    "http.response_size" = "[0-9]+"
}

# Span count validation
[expect.count]
by_kind.server = { min = 2, max = 2 }
by_kind.internal = { min = 1, max = 1 }
```

### Attribute Validation Patterns

Validate span attributes with various patterns:

```toml
[test.metadata]
name = "attribute_validation_patterns"
description = "Test various attribute validation patterns"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[[steps]]
name = "api_request"
command = ["curl", "http://localhost:80/api/users"]

# Exact attribute matching
[[expect.span]]
name = "api.request"
kind = "server"
attrs.exact = {
    "http.method" = "GET",
    "http.route" = "/api/users",
    "http.status_code" = "200"
}

# Pattern matching with regex
[[expect.span]]
name = "api.request"
kind = "server"
attrs.regex = {
    "http.user_agent" = "curl/.*",
    "http.request_size" = "[0-9]+"
}

# Numeric range validation
[[expect.span]]
name = "api.response"
kind = "server"
attrs.numeric = {
    "http.status_code" = { min = 200, max = 299 },
    "http.response_size" = { min = 1, max = 10000 }
}

# Existence validation
[[expect.span]]
name = "api.request"
kind = "server"
attrs.exists = ["http.method", "http.route", "http.status_code"]
```

## Trace Structure Validation

### Parent-Child Relationships

Validate span hierarchy and relationships:

```toml
[test.metadata]
name = "trace_structure_validation"
description = "Validate trace structure and relationships"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[[steps]]
name = "api_db_request"
description = "API request that queries database"
command = ["curl", "http://localhost:80/api/data"]
expected_output_regex = ".*"

# Expected trace structure
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[[expect.span]]
name = "api.request"
kind = "server"
parent = "clnrm.run"

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "db.response"
kind = "client"
parent = "api.request"

# Validate trace graph
[expect.graph]
must_include = [
    ["clnrm.run", "api.request"],
    ["api.request", "db.query"],
    ["api.request", "db.response"]
]

acyclic = true
max_depth = 3

# Temporal constraints
[expect.window]
start_span = "api.request"
end_span = "db.response"
max_duration_ms = 1000
```

### Complex Trace Patterns

Validate complex multi-service trace patterns:

```toml
[test.metadata]
name = "complex_trace_patterns"
description = "Validate complex multi-service trace patterns"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[[steps]]
name = "complex_request"
description = "Complex request involving multiple services"
command = ["curl", "http://localhost:80/api/complex"]
expected_output_regex = ".*"

# Complex trace structure
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[[expect.span]]
name = "api.request"
kind = "server"
parent = "clnrm.run"
attrs.all = { "http.method" = "GET", "http.route" = "/api/complex" }

[[expect.span]]
name = "cache.get"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "external.api_call"
kind = "client"
parent = "api.request"

# Validate complex relationships
[expect.graph]
must_include = [
    ["clnrm.run", "api.request"],
    ["api.request", "cache.get"],
    ["api.request", "db.query"],
    ["api.request", "external.api_call"]
]

# Parallel execution validation
[expect.parallel]
spans = ["cache.get", "db.query"]
max_duration_ms = 500

# Sequential dependencies
[expect.order]
must_precede = [
    ["api.request", "cache.get"],
    ["api.request", "db.query"],
    ["api.request", "external.api_call"],
    ["cache.get", "api.response"],
    ["db.query", "api.response"],
    ["external.api_call", "api.response"]
]
```

## Temporal Validation

### Time Window Validation

Validate timing constraints between spans:

```toml
[test.metadata]
name = "temporal_validation"
description = "Validate temporal constraints between operations"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[[steps]]
name = "timed_operation"
description = "Operation with timing constraints"
command = ["curl", "http://localhost:80/api/timed"]
expected_output_regex = ".*"

# Temporal window validation
[expect.window]
start_span = "api.request"
end_span = "api.response"
min_duration_ms = 100
max_duration_ms = 1000

[expect.window]
start_span = "db.query.start"
end_span = "db.query.end"
max_duration_ms = 500

# Multiple windows
[expect.windows]
api_request = {
    start = "api.request",
    end = "api.response",
    max_ms = 1000
}

db_operation = {
    start = "db.query.start",
    end = "db.query.end",
    max_ms = 500
}
```

### Latency Analysis

Analyze latency patterns in traces:

```toml
[test.metadata]
name = "latency_analysis"
description = "Analyze latency patterns in traces"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[[steps]]
name = "latency_test"
description = "Test operation latency"
command = ["curl", "http://localhost:80/api/slow"]
expected_output_regex = ".*"
timeout_seconds = 10

# Latency analysis
[expect.latency]
spans = ["api.request", "api.response"]
max_p95_ms = 2000
max_p99_ms = 5000

[expect.latency]
spans = ["db.query"]
max_p95_ms = 100
max_p99_ms = 200

# Latency percentiles
[expect.latency.percentiles]
api_total = { p50 = 500, p95 = 1500, p99 = 3000 }
db_query = { p50 = 50, p95 = 100, p99 = 150 }
```

## Error Span Validation

### Error Handling Validation

Validate error spans and error propagation:

```toml
[test.metadata]
name = "error_span_validation"
description = "Validate error handling and error spans"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[[steps]]
name = "error_scenario"
description = "Trigger error scenario"
command = ["curl", "http://localhost:80/api/error"]
expected_exit_code = 1

# Expected error spans
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { "http.method" = "GET", "http.route" = "/api/error" }

[[expect.span]]
name = "api.error"
kind = "internal"
parent = "api.request"
attrs.all = {
    "error.type" = "validation_error",
    "error.message" = "Invalid input",
    "error.code" = "400"
}

[[expect.span]]
name = "api.response"
kind = "server"
parent = "api.request"
attrs.all = { "http.status_code" = "400" }

# Error count validation
[expect.count]
by_kind.internal = { min = 1, max = 1 }
by_name."api.error" = { min = 1, max = 1 }
```

### Error Propagation Patterns

Validate error propagation across services:

```toml
[test.metadata]
name = "error_propagation"
description = "Validate error propagation across services"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[[steps]]
name = "error_propagation_test"
description = "Test error propagation"
command = ["curl", "http://localhost:80/api/db-error"]
expected_exit_code = 1

# Error propagation trace
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[[expect.span]]
name = "api.request"
kind = "server"
parent = "clnrm.run"

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "db.error"
kind = "internal"
parent = "db.query"
attrs.all = { "error.type" = "connection_error" }

[[expect.span]]
name = "api.error"
kind = "internal"
parent = "api.request"
attrs.all = { "error.type" = "database_error" }

# Error propagation validation
[expect.error_propagation]
source_span = "db.error"
destination_spans = ["api.error"]
propagation_type = "error_cause"
```

## Performance Validation

### Performance Span Validation

Validate performance-related spans:

```toml
[test.metadata]
name = "performance_span_validation"
description = "Validate performance-related spans"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[[steps]]
name = "performance_test"
description = "Performance test with metrics"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Performance spans
[[expect.span]]
name = "api.request"
kind = "server"
attrs.performance = {
    "http.request_duration" = { max_ms = 100 },
    "http.response_size" = { min_bytes = 100, max_bytes = 10000 }
}

[[expect.span]]
name = "api.performance"
kind = "internal"
attrs.performance = {
    "throughput_rps" = { min = 800, max = 1200 },
    "error_rate" = { max_percent = 1.0 }
}

# Performance thresholds
[expect.performance]
spans = ["api.request"]
max_p95_latency_ms = 150
max_p99_latency_ms = 300
min_throughput_rps = 800
```

## Custom Span Validators

### Plugin-Based Validation

Create custom span validators for domain-specific validation:

```rust
use crate::cleanroom::{ValidationResult, SpanData, TraceData};
use crate::error::{CleanroomError, Result};

#[derive(Debug)]
pub struct CustomSpanValidator {
    name: String,
    rules: Vec<ValidationRule>,
}

#[derive(Debug)]
pub struct ValidationRule {
    span_name_pattern: String,
    attribute_validators: Vec<AttributeValidator>,
}

#[derive(Debug)]
pub struct AttributeValidator {
    attribute_name: String,
    validator_type: ValidatorType,
}

#[derive(Debug)]
pub enum ValidatorType {
    Regex(String),
    Numeric { min: Option<f64>, max: Option<f64> },
    StringList(Vec<String>),
    Custom(Box<dyn Fn(&str) -> bool + Send + Sync>),
}

impl CustomSpanValidator {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn validate_spans(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        for span in spans {
            for rule in &self.rules {
                if self.span_matches_rule(span, rule) {
                    if let Err(validation_errors) = self.validate_span_attributes(span, rule) {
                        errors.extend(validation_errors);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(ValidationResult::success())
        } else {
            Ok(ValidationResult::failure(errors))
        }
    }

    fn span_matches_rule(&self, span: &SpanData, rule: &ValidationRule) -> bool {
        // Simple pattern matching - in reality, use regex
        span.name.contains(&rule.span_name_pattern)
    }

    fn validate_span_attributes(&self, span: &SpanData, rule: &ValidationRule) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        for attr_validator in &rule.attribute_validators {
            if let Some(value) = span.attributes.get(&attr_validator.attribute_name) {
                if !self.validate_attribute_value(value, &attr_validator.validator_type) {
                    errors.push(format!(
                        "Attribute '{}' failed validation for span '{}'",
                        attr_validator.attribute_name, span.name
                    ));
                }
            }
        }

        Ok(errors)
    }

    fn validate_attribute_value(&self, value: &str, validator: &ValidatorType) -> bool {
        match validator {
            ValidatorType::Regex(pattern) => {
                // Simple regex check - in reality, use proper regex crate
                value.contains(pattern)
            }
            ValidatorType::Numeric { min, max } => {
                if let Ok(num) = value.parse::<f64>() {
                    (min.is_none() || num >= min.unwrap()) &&
                    (max.is_none() || num <= max.unwrap())
                } else {
                    false
                }
            }
            ValidatorType::StringList(valid_values) => {
                valid_values.contains(&value.to_string())
            }
            ValidatorType::Custom(validator_fn) => {
                validator_fn(value)
            }
        }
    }
}
```

## Integration Testing with OTEL

### End-to-End OTEL Validation

Complete e2e test with OTEL validation:

```toml
[test.metadata]
name = "e2e_otel_validation"
description = "End-to-end OTEL validation test"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

# OTEL configuration
[otel]
exporter = "stdout"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0

[otel.resources]
"service.name" = "e2e_test"
"service.version" = "1.0.0"
"env" = "test"

[[steps]]
name = "e2e_test"
description = "Complete e2e test with OTEL validation"
command = ["curl", "http://localhost:80/api/complete"]
expected_output_regex = "success"

# Complete trace validation
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "api.request"
kind = "server"
parent = "clnrm.run"
attrs.all = {
    "http.method" = "GET",
    "http.route" = "/api/complete"
}

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"
attrs.all = {
    "db.system" = "postgresql",
    "db.operation" = "SELECT"
}

[[expect.span]]
name = "cache.get"
kind = "client"
parent = "api.request"
attrs.all = {
    "cache.system" = "redis",
    "cache.operation" = "GET"
}

# Complex trace validation
[expect.graph]
must_include = [
    ["clnrm.run", "api.request"],
    ["api.request", "db.query"],
    ["api.request", "cache.get"]
]

[expect.order]
must_precede = [
    ["api.request", "db.query"],
    ["api.request", "cache.get"],
    ["db.query", "api.response"],
    ["cache.get", "api.response"]
]

# Temporal constraints
[expect.window]
start_span = "api.request"
end_span = "api.response"
max_duration_ms = 2000

[expect.window]
start_span = "db.query"
end_span = "cache.get"
max_duration_ms = 500

# Count validation
[expect.count]
by_kind.server = { min = 1, max = 1 }
by_kind.client = { min = 2, max = 2 }
by_kind.internal = { min = 1, max = 1 }

# Hermeticity validation
[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = {
    "service.name" = "e2e_test",
    "env" = "test"
}
```

## Best Practices

### 1. Start with Simple Validation

```toml
# ✅ Good: Simple, focused validation
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { "http.method" = "GET" }

[expect.count]
by_kind.server = { min = 1, max = 1 }
```

### 2. Use Descriptive Span Names

```toml
# ✅ Good: Descriptive span names
[[expect.span]]
name = "user_authentication.check_credentials"
kind = "internal"
attrs.all = { "user.id" = "12345" }
```

### 3. Validate Complete Traces

```toml
# ✅ Good: Complete trace validation
[expect.graph]
must_include = [
    ["clnrm.run", "api.request"],
    ["api.request", "db.query"],
    ["db.query", "api.response"]
]

[expect.order]
must_precede = [
    ["api.request", "db.query"],
    ["db.query", "api.response"]
]
```

### 4. Set Realistic Timeouts

```toml
# ✅ Good: Realistic timeouts
[expect.window]
start_span = "api.request"
end_span = "api.response"
max_duration_ms = 1000

# ✅ Good: Performance expectations
[expect.performance]
max_p95_latency_ms = 150
max_p99_latency_ms = 300
```

## Next Steps

Now that you understand OTEL validation:

1. **Try the examples**: Run the OTEL validation examples in this chapter
2. **Create custom validators**: Build custom span validators for your domain
3. **Learn performance testing**: Move on to [Performance Testing](performance-testing.md)
4. **Master template system**: Learn about [Template System Mastery](../template-mastery/README.md)

## Further Reading

- [OpenTelemetry Specification](https://opentelemetry.io/docs/)
- [OTEL Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [Plugin Development](../plugin-development/README.md)

