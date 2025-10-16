# Contract Testing Guide

## Overview

Contract testing ensures that services can communicate with each other by validating API contracts between providers and consumers. This prevents integration failures caused by breaking changes.

## Core Concepts

### Provider and Consumer

- **Provider**: Service that exposes an API
- **Consumer**: Service that calls the provider's API
- **Contract**: Agreement on request/response format

### Contract Testing Flow

```
1. Consumer defines expected interactions (contract)
2. Provider implements and validates against contract
3. Both sides test independently
4. Integration guaranteed without end-to-end tests
```

## Implementation

### Consumer Contract Tests

**Location**: `crates/clnrm-core/tests/contract/consumer/`

```rust
use serde_json::json;

#[tokio::test]
async fn test_container_api_contract() {
    // Define expected request/response
    let request = ContainerRequest {
        image: "alpine:latest",
        command: vec!["echo", "test"],
    };

    let expected_response = ContainerResponse {
        exit_code: 0,
        stdout: "test\n",
        stderr: "",
    };

    // Record interaction for provider validation
    record_contract_interaction(
        "container-api",
        "run-command",
        request,
        expected_response,
    ).await?;
}
```

### Provider Contract Tests

**Location**: `crates/clnrm-core/tests/contract/provider/`

```rust
#[tokio::test]
async fn test_provider_honors_contract() {
    // Load recorded interactions
    let interactions = load_contract_interactions("container-api").await?;

    // Verify provider satisfies each interaction
    for interaction in interactions {
        let response = execute_provider_request(interaction.request).await?;
        assert_contract_match(response, interaction.expected_response);
    }
}
```

## Schema Validation

### JSON Schema

```rust
use jsonschema::JSONSchema;

#[test]
fn test_api_schema_validation() {
    let schema = json!({
        "type": "object",
        "properties": {
            "image": {"type": "string"},
            "command": {
                "type": "array",
                "items": {"type": "string"}
            }
        },
        "required": ["image", "command"]
    });

    let compiled = JSONSchema::compile(&schema).unwrap();

    // Valid request
    let valid = json!({"image": "alpine", "command": ["echo"]});
    assert!(compiled.is_valid(&valid));

    // Invalid request
    let invalid = json!({"image": 123});
    assert!(!compiled.is_valid(&invalid));
}
```

### OpenAPI/Swagger

```yaml
# openapi.yaml
openapi: 3.0.0
info:
  title: CLNRM API
  version: 1.0.0

paths:
  /containers/run:
    post:
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ContainerRequest'
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ContainerResponse'

components:
  schemas:
    ContainerRequest:
      type: object
      required:
        - image
        - command
      properties:
        image:
          type: string
        command:
          type: array
          items:
            type: string
```

## Breaking Change Detection

### Version Compatibility

```rust
use semver::Version;

#[test]
fn test_backward_compatibility() {
    let current_version = Version::parse("1.2.0").unwrap();
    let previous_version = Version::parse("1.1.0").unwrap();

    // Load schemas
    let current_schema = load_api_schema(&current_version)?;
    let previous_schema = load_api_schema(&previous_version)?;

    // Verify backward compatibility
    let breaking_changes = detect_breaking_changes(
        &previous_schema,
        &current_schema
    );

    assert!(breaking_changes.is_empty(),
        "Breaking changes detected: {:?}", breaking_changes);
}
```

### Contract Evolution

```rust
#[derive(Debug)]
enum SchemaChange {
    AddedField { field: String, optional: bool },
    RemovedField { field: String },
    ChangedFieldType { field: String, old_type: String, new_type: String },
    AddedRequirement { field: String },
}

fn is_breaking_change(change: &SchemaChange) -> bool {
    match change {
        SchemaChange::AddedField { optional: true, .. } => false,  // Non-breaking
        SchemaChange::AddedField { optional: false, .. } => true,  // Breaking
        SchemaChange::RemovedField { .. } => true,                 // Breaking
        SchemaChange::ChangedFieldType { .. } => true,             // Breaking
        SchemaChange::AddedRequirement { .. } => true,             // Breaking
    }
}
```

## Running Contract Tests

```bash
# Run consumer contract tests
cargo test --test contract_consumer

# Run provider contract tests
cargo test --test contract_provider

# Generate contract documentation
cargo run --example generate-contract-docs

# Validate contracts
cargo run --example validate-contracts
```

## Best Practices

1. **Test both sides**: Consumer and provider tests
2. **Version contracts**: Track contract versions
3. **Fail on breaking changes**: CI/CD validation
4. **Document contracts**: OpenAPI specs
5. **Use schemas**: JSON Schema validation

## CI/CD Integration

```yaml
# .github/workflows/contracts.yml
name: Contract Tests

on: [push, pull_request]

jobs:
  contract-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Consumer contract tests
        run: cargo test --test contract_consumer

      - name: Provider contract tests
        run: cargo test --test contract_provider

      - name: Validate schemas
        run: cargo run --example validate-contracts

      - name: Check for breaking changes
        run: |
          cargo run --example detect-breaking-changes
          if [ $? -ne 0 ]; then
            echo "Breaking changes detected!"
            exit 1
          fi
```

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
