# CLNRM Contract Testing

## Quick Start

Contract testing for the CLNRM (Cleanroom Testing Framework) ensures that all components interact correctly by validating their contracts.

### Running Tests

```bash
# Run all contract tests
cargo test --test contract_tests

# Run specific suite
cargo test --test contract_tests -- api_contracts
cargo test --test contract_tests -- service_contracts
cargo test --test contract_tests -- consumer_contracts
cargo test --test contract_tests -- event_contracts
cargo test --test contract_tests -- database_contracts

# Run CI integration
bash tests/contracts/ci_integration.sh
```

## Structure

```
tests/contracts/
├── README.md                      # This file
├── mod.rs                         # Module exports
├── schema_validator.rs            # Schema validation utilities
├── api_contracts.rs               # API contract tests
├── service_contracts.rs           # Service plugin contract tests
├── consumer_contracts.rs          # Consumer-driven contract tests
├── event_contracts.rs             # Event contract tests
├── database_contracts.rs          # Database schema contract tests
├── ci_integration.sh              # CI/CD integration script
└── schemas/                       # JSON schema definitions
    ├── service_plugin_contract.json
    ├── backend_capabilities_contract.json
    ├── cleanroom_api_contract.json
    └── database_schema_contract.json
```

## Contract Types

### 1. API Contracts
- **File**: `api_contracts.rs`
- **Schema**: `schemas/cleanroom_api_contract.json`
- **Purpose**: Validate CleanroomEnvironment and Backend Capabilities APIs
- **Tests**: 10+ test cases

### 2. Service Contracts
- **File**: `service_contracts.rs`
- **Schema**: `schemas/service_plugin_contract.json`
- **Purpose**: Validate service plugin implementations
- **Tests**: 10+ test cases covering multiple plugin types

### 3. Consumer Contracts
- **File**: `consumer_contracts.rs`
- **Purpose**: Validate inter-module communication contracts
- **Tests**: 10+ test cases for different consumer-provider pairs

### 4. Event Contracts
- **File**: `event_contracts.rs`
- **Purpose**: Validate async event-driven communication
- **Tests**: 12+ event types validated

### 5. Database Contracts
- **File**: `database_contracts.rs`
- **Schema**: `schemas/database_schema_contract.json`
- **Purpose**: Validate database schema definitions
- **Tests**: 5+ table schema validations

## Key Concepts

### Contract Definition
A contract defines the expected interface between components:
- Request/response structures
- Data types and constraints
- Error handling
- Behavior expectations

### Consumer-Driven Contracts
Consumers define what they need from providers:
1. Consumer writes test expressing expectations
2. Provider implements to satisfy contract
3. Both parties verify against contract

### Schema Validation
All contracts are backed by JSON schemas:
- Type safety
- Data validation
- Documentation
- Breaking change detection

## Examples

### API Contract Test
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

### Service Plugin Contract Test
```rust
#[test]
fn test_generic_container_plugin_contract() {
    let plugin_contract = ServicePluginContract {
        name: "generic_container".to_string(),
        version: "1.0.0".to_string(),
        // ... full contract definition
    };

    // Validate contract compliance
    assert!(plugin_contract.capabilities.start.is_some());
}
```

### Consumer Contract Test
```rust
#[test]
fn test_cleanroom_expects_backend_run_cmd_contract() {
    // Consumer: CleanroomEnvironment
    // Provider: Backend

    let request = BackendCommandRequest {
        command: vec!["echo".to_string(), "test".to_string()],
    };

    assert!(!request.command.is_empty());
}
```

## CI/CD Integration

### GitHub Actions
Contract tests run on:
- Every push to master/main
- All pull requests
- Schema file changes

See: `.github/workflows/contract-tests.yml`

### Local CI Script
Run the full CI pipeline locally:
```bash
bash tests/contracts/ci_integration.sh
```

Results saved to: `target/contract-test-results/`

## Best Practices

1. **Version Contracts**: Use semantic versioning
2. **Backward Compatibility**: Make new fields optional
3. **Document Changes**: Update docs when contracts change
4. **Test Both Sides**: Test success and error cases
5. **Use Correlation IDs**: For event tracing
6. **Validate Enums**: Always check enum values
7. **Consumer-Driven**: Let consumers define needs

## Documentation

- **Full Guide**: `docs/contract-testing-guide.md`
- **Implementation Summary**: `docs/contract-testing-summary.md`
- **CI Workflow**: `.github/workflows/contract-tests.yml`

## Statistics

- **Test Suites**: 5
- **Test Cases**: 50+
- **JSON Schemas**: 4
- **Contract Definitions**: 20+
- **Event Types**: 12+
- **Lines of Code**: 2000+

## Contributing

When adding new contracts:

1. Define JSON schema in `schemas/`
2. Write tests in appropriate file
3. Update this README
4. Run tests locally
5. Submit PR with documentation

## Support

For questions or issues:
- Review existing contracts
- Check documentation in `docs/`
- Consult with team lead

## License

Same as CLNRM project
