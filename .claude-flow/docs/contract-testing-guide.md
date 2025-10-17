# CLNRM Contract Testing Guide

## Overview

Contract testing ensures that different components of the CLNRM system interact correctly by validating the contracts (interfaces) between them. This guide covers all aspects of contract testing in CLNRM.

## Table of Contents

1. [Introduction to Contract Testing](#introduction-to-contract-testing)
2. [Contract Types](#contract-types)
3. [Schema Definitions](#schema-definitions)
4. [Writing Contract Tests](#writing-contract-tests)
5. [CI/CD Integration](#cicd-integration)
6. [Best Practices](#best-practices)

## Introduction to Contract Testing

Contract testing validates that:
- **Providers** implement their contracts correctly
- **Consumers** use provider contracts as specified
- Changes don't break existing contracts (backward compatibility)

### Benefits

- **Early Detection**: Catch integration issues before deployment
- **Independent Development**: Teams can work on different components independently
- **Documentation**: Contracts serve as living documentation
- **Confidence**: Safe refactoring with contract validation

## Contract Types

### 1. API Contracts

Validate HTTP/REST API endpoints and responses.

**Location**: `tests/contracts/api_contracts.rs`

**Schema**: `tests/contracts/schemas/cleanroom_api_contract.json`

**Example**:
```rust
#[test]
fn test_start_service_contract() {
    let request = json!({
        "service_name": "test_service"
    });

    // Validate request structure
    assert!(request.get("service_name").is_some());
}
```

### 2. Service Plugin Contracts

Validate that service plugins comply with the `ServicePlugin` trait.

**Location**: `tests/contracts/service_contracts.rs`

**Schema**: `tests/contracts/schemas/service_plugin_contract.json`

**Required Capabilities**:
- `start()` - Returns `ServiceHandle`
- `stop()` - Graceful shutdown
- `health_check()` - Returns `HealthStatus`

**Example**:
```rust
#[test]
fn test_generic_container_plugin_contract() {
    let plugin_contract = ServicePluginContract {
        name: "generic_container".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: "generic_container".to_string(),
        // ... capabilities, lifecycle, health_check
    };

    let serialized = serde_json::to_value(&plugin_contract).unwrap();
    assert!(serialized.get("capabilities").is_some());
}
```

### 3. Consumer-Driven Contracts

Validate inter-module communication based on consumer expectations.

**Location**: `tests/contracts/consumer_contracts.rs`

**Examples**:
- **Backend-Cleanroom**: Command execution contracts
- **Service Registry**: Service lifecycle contracts
- **Capability Registry**: Capability validation contracts

**Example**:
```rust
#[test]
fn test_cleanroom_expects_backend_run_cmd_contract() {
    // Consumer: CleanroomEnvironment
    // Provider: Backend

    let request = BackendCommandRequest {
        command: vec!["sh".to_string(), "-c".to_string(), "echo test".to_string()],
        env: env_map,
    };

    assert!(!request.command.is_empty());
}
```

### 4. Event Contracts

Validate async event-driven communication.

**Location**: `tests/contracts/event_contracts.rs`

**Event Categories**:
- Service lifecycle events
- Container lifecycle events
- Test execution events
- Capability events

**Event Envelope Structure**:
```rust
struct EventEnvelope {
    event_id: String,
    event_type: String,
    event_version: String,
    timestamp: String,
    source: String,
    correlation_id: Option<String>,
    payload: serde_json::Value,
    metadata: HashMap<String, String>,
}
```

**Example**:
```rust
#[test]
fn test_service_started_event_contract() {
    let event = EventEnvelope {
        event_type: "service.started".to_string(),
        payload: serde_json::to_value(&payload).unwrap(),
        // ...
    };

    assert_eq!(event.event_type, "service.started");
}
```

### 5. Database Schema Contracts

Validate database schema definitions and migrations.

**Location**: `tests/contracts/database_contracts.rs`

**Schema**: `tests/contracts/schemas/database_schema_contract.json`

**Example**:
```rust
#[test]
fn test_test_results_table_schema_contract() {
    let table = Table {
        name: "test_results".to_string(),
        columns: vec![...],
        primary_key: vec!["id".to_string()],
        indexes: Some(vec![...]),
    };

    // Validate table structure
    assert!(!table.primary_key.is_empty());
}
```

## Schema Definitions

### JSON Schema Files

All contract schemas are defined in `tests/contracts/schemas/`:

1. **service_plugin_contract.json** - Service plugin interface
2. **backend_capabilities_contract.json** - Backend capabilities API
3. **cleanroom_api_contract.json** - Cleanroom environment API
4. **database_schema_contract.json** - Database schema definitions

### Schema Structure

Each schema includes:
- **Required fields**: Must be present
- **Optional fields**: May be present
- **Data types**: String, number, boolean, object, array
- **Constraints**: Min/max length, patterns, enums
- **Descriptions**: Field documentation

**Example Schema**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Service Plugin Contract",
  "type": "object",
  "required": ["name", "version", "plugin_type"],
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9_]*$",
      "minLength": 3,
      "maxLength": 50
    }
  }
}
```

## Writing Contract Tests

### Step 1: Define the Contract

Create or update the JSON schema in `tests/contracts/schemas/`.

### Step 2: Write the Test

Create a test in the appropriate file:

```rust
#[cfg(test)]
mod my_contract_tests {
    use super::*;

    #[test]
    fn test_my_contract() {
        // Arrange: Create test data
        let data = MyStruct {
            field: "value".to_string(),
        };

        // Act: Serialize to JSON
        let serialized = serde_json::to_value(&data).unwrap();

        // Assert: Validate contract
        assert!(serialized.get("field").is_some());
    }
}
```

### Step 3: Validate Against Schema

Use the `SchemaValidator`:

```rust
use super::schema_validator::SchemaValidator;

#[test]
fn test_with_schema_validation() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    let data = json!({
        "name": "test",
        "version": "1.0.0"
    });

    let result = validator.validate_value("my_contract.json", &data);
    assert!(result.is_ok());
}
```

### Step 4: Run Tests

```bash
# Run all contract tests
cargo test --test '*' -- contract

# Run specific contract test suite
cargo test --test '*' -- api_contracts
cargo test --test '*' -- service_contracts
cargo test --test '*' -- consumer_contracts
cargo test --test '*' -- event_contracts
cargo test --test '*' -- database_contracts
```

## CI/CD Integration

### GitHub Actions Workflow

Contract tests run automatically on:
- Push to `master` or `main`
- Pull requests
- Schema file changes

**Workflow File**: `.github/workflows/contract-tests.yml`

### CI Steps

1. **Schema Validation**: Validate all JSON schemas
2. **Contract Tests**: Run all contract test suites
3. **Report Generation**: Create test report
4. **Breaking Change Detection**: Check for contract changes in PRs
5. **Coverage Analysis**: Calculate contract coverage percentage

### Running Locally

```bash
# Run CI integration script
bash tests/contracts/ci_integration.sh

# Results saved to: target/contract-test-results/
```

### CI Outputs

- **Test Logs**: Detailed test execution logs
- **Test Report**: Markdown summary of results
- **Coverage Badge**: Contract coverage percentage
- **PR Comments**: Automated feedback on pull requests

## Best Practices

### 1. Version Your Contracts

Use semantic versioning for contracts:
- **Major**: Breaking changes (incompatible)
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes

```json
{
  "event_version": "1.0.0"
}
```

### 2. Document Breaking Changes

When updating contracts:
1. Document what changed
2. Provide migration guide
3. Update version numbers
4. Add deprecation warnings

### 3. Test Both Success and Failure Cases

```rust
#[test]
fn test_success_case() {
    let response = json!({"success": true});
    assert!(response.get("success").unwrap().as_bool().unwrap());
}

#[test]
fn test_error_case() {
    let response = json!({
        "success": false,
        "error": "Service not found"
    });
    assert!(!response.get("success").unwrap().as_bool().unwrap());
    assert!(response.get("error").is_some());
}
```

### 4. Use Correlation IDs

For event contracts, always include correlation IDs for tracing:

```rust
let event = EventEnvelope {
    correlation_id: Some("session-123".to_string()),
    // ...
};
```

### 5. Validate Enum Values

Always validate enum values against allowed options:

```rust
let status = "Healthy";
assert!(
    status == "Healthy" || status == "Unhealthy" || status == "Unknown",
    "Invalid health status"
);
```

### 6. Test Required vs Optional Fields

```rust
// Required fields
assert!(data.get("required_field").is_some());

// Optional fields
if let Some(optional) = data.get("optional_field") {
    // Validate if present
}
```

### 7. Backward Compatibility

When adding new fields:
- Make them optional
- Provide default values
- Don't remove existing fields

### 8. Consumer-Driven Approach

Let consumers define what they need:
1. Consumer writes expectation test
2. Provider implements to satisfy
3. Contract serves as agreement

## Integration with CLNRM

### Service Plugins

All service plugins must implement the contract defined in `service_plugin_contract.json`:

```rust
impl ServicePlugin for MyPlugin {
    fn name(&self) -> &str { ... }
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> { ... }
    fn stop(&self, handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> { ... }
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus { ... }
}
```

### API Endpoints

All API endpoints must comply with `cleanroom_api_contract.json`:
- Request/response schemas
- Error handling
- Status codes

### Events

All async events must use `EventEnvelope` structure and comply with event contracts.

## Troubleshooting

### Schema Validation Fails

```bash
# Validate schema manually
jq empty tests/contracts/schemas/my_contract.json
```

### Contract Test Fails

1. Check the error message
2. Validate data structure matches schema
3. Verify required fields are present
4. Check data types match expectations

### Breaking Changes Detected

1. Review changes in PR
2. Update version numbers
3. Provide migration path
4. Update documentation

## Resources

- **JSON Schema**: https://json-schema.org/
- **Consumer-Driven Contracts**: https://martinfowler.com/articles/consumerDrivenContracts.html
- **Pact**: https://docs.pact.io/ (for more advanced contract testing)

## Contributing

When adding new contracts:
1. Define JSON schema
2. Write tests
3. Update this guide
4. Run CI locally before submitting PR

## Questions?

- Check existing contracts in `tests/contracts/`
- Review test examples
- Consult team lead for complex contracts
