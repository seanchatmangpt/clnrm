# Mock Design Report - v0.6.0 London School TDD Contracts

**Agent**: Mock Designer
**Reporting To**: London TDD Sub-Coordinator
**Date**: 2025-10-16
**Status**: ✅ COMPLETE

## Objective

Design comprehensive mocked interaction contracts for v0.6.0 features following London School TDD methodology to drive outside-in development and specify expected behavior through mock expectations.

## Deliverables

### 1. Template System Mocks (`swarm/tdd/mocks/template_mocks.rs`)

**Mock Traits Defined:**

- **`MockTemplateRenderer`** - Template engine contract
  - `render_file(path)` - Render template file to TOML string
  - `render_str(template, name)` - Render template string directly
  - `set_context(context)` - Set rendering context variables
  - `verify_all_expectations()` - Interaction verification

- **`MockTemplateContext`** - Template context management contract
  - `add_var(key, value)` - Add user-defined variables (vars.*)
  - `add_matrix_param(key, value)` - Add matrix parameters (matrix.*)
  - `add_otel_config(key, value)` - Add OTEL config (otel.*)
  - `to_render_context()` - Convert to rendering context

- **`MockDeterminismConfig`** - Deterministic execution contract
  - `set_seed(seed)` - Set random seed for matrix expansion
  - `set_freeze_clock(timestamp)` - Freeze time for reproducibility
  - `is_deterministic()` - Check if determinism enabled
  - `get_seed()` / `get_freeze_clock()` - Accessor methods

- **`MockTemplateFunctions`** - Custom function registration contract
  - `register_env_function()` - Environment variable access: `{{ env("VAR") }}`
  - `register_now_function(determinism)` - Timestamps: `{{ now_rfc3339() }}`
  - `register_sha256_function()` - Hashing: `{{ sha256("input") }}`
  - `register_toml_function()` - TOML encoding: `{{ toml(data) }}`

**Supporting Types:**

- `TemplateRenderExpectations` - Fluent expectation builder
- `RenderFileExpectation` - File rendering expectations
- `RenderStrExpectation` - String rendering expectations

### 2. Validation System Mocks (`swarm/tdd/mocks/validation_mocks.rs`)

**Mock Traits Defined:**

- **`MockSpanValidator`** - OTEL span validation contract
  - `validate_span(assertion)` - Validate span attributes and properties
  - `validate_span_duration(name, min, max)` - Validate duration constraints
  - `verify_all_expectations()` - Interaction verification

- **`MockTraceValidator`** - OTEL trace validation contract
  - `validate_trace(assertion)` - Validate complete trace with all spans
  - `validate_parent_child_relationships(pairs)` - Validate span hierarchy
  - `validate_trace_completeness(trace_id, count)` - Validate no unexpected spans
  - `verify_all_expectations()` - Interaction verification

- **`MockExportValidator`** - Telemetry export validation contract
  - `validate_export(endpoint)` - Validate telemetry reaches destination
  - `capture_exported_spans(spans)` - Capture spans for verification
  - `verify_span_data_integrity(span)` - Validate data integrity after export
  - `verify_all_expectations()` - Interaction verification

- **`MockPerformanceValidator`** - Performance overhead measurement contract
  - `measure_baseline(operation)` - Measure without telemetry
  - `measure_with_telemetry(operation)` - Measure with telemetry
  - `validate_overhead(baseline, instrumented, max)` - Validate acceptable overhead
  - `verify_all_expectations()` - Interaction verification

**Supporting Types:**

- `ValidationExpectations` - Fluent expectation builder
- `SpanAssertion` / `TraceAssertion` - Assertion configurations
- `SpanValidationResult` / `TraceValidationResult` - Validation results
- `ExportedSpan` - Exported span representation

### 3. Plugin System Mocks (`swarm/tdd/mocks/plugin_mocks.rs`)

**Mock Traits Defined:**

- **`MockServicePlugin`** - Service plugin lifecycle contract
  - `name()` - Get service name identifier
  - `start()` - Start service instance
  - `stop(handle)` - Stop service instance
  - `health_check(handle)` - Check service health status
  - `verify_all_expectations()` - Interaction verification

- **`MockServiceRegistry`** - Service registry management contract
  - `register_plugin(plugin)` - Register new plugin
  - `start_service(name)` - Start registered service
  - `stop_service(name)` - Stop active service
  - `get_service(name)` - Get service handle
  - `list_plugins()` - List registered plugins
  - `list_active_services()` - List running services
  - `verify_all_expectations()` - Interaction verification

- **`MockContainerBackend`** - Container backend operations contract
  - `start_container(image, config)` - Start container from image
  - `execute_command(container, command)` - Execute command in container
  - `stop_container(container)` - Stop and remove container
  - `get_logs(container)` - Get container logs
  - `verify_all_expectations()` - Interaction verification

**Supporting Types:**

- `PluginExpectations` - Fluent expectation builder
- `ServiceHandle` - Service instance identifier with metadata
- `HealthStatus` - Service health status enum
- `ContainerHandle` - Container instance identifier
- `ContainerConfig` - Container configuration
- `ExecutionResult` - Command execution result

### 4. Mock Coordinator (`swarm/tdd/mocks/mod.rs`)

**Centralized Coordination:**

- `MockCoordinator` - Unified expectation management for complex workflows
  - `template_expectations()` - Access template expectations
  - `validation_expectations()` - Access validation expectations
  - `plugin_expectations()` - Access plugin expectations
  - `verify_all()` - Verify all expectations across all systems

**Module Organization:**

- Clean exports of all mock traits and types
- Comprehensive documentation with usage examples
- Integration with test framework

### 5. Documentation (`swarm/tdd/mocks/README.md`)

**Complete Guide Including:**

- London School TDD principles overview
- Detailed usage examples for each mock category
- Integration patterns with test framework
- Design principles and standards compliance
- Next steps for implementation

## London School TDD Principles Applied

### 1. Outside-In Development

All mocks are designed to start from high-level acceptance tests:

```rust
#[test]
fn test_config_rendering_workflow() {
    // Arrange - Define expected interactions
    let mut expectations = TemplateRenderExpectations::new();
    expectations.expect_render_file("/path/to/config.toml")
        .times(1)
        .returns(Ok("[test]\nname = \"value\"".to_string()));

    // Act - Execute system under test
    let result = system_under_test.render_config();

    // Assert - Verify interactions occurred
    assert!(result.is_ok());
    expectations.verify()?;
}
```

### 2. Mock-Driven Design

Mocks define clear contracts between collaborating objects:

- **Template System**: Renderer → Context → Functions → Determinism
- **Validation System**: Validator → Span/Trace Assertions → Export → Performance
- **Plugin System**: Registry → Plugins → Backend → Containers

### 3. Behavior Verification

Focus on HOW objects interact, not WHAT they contain:

```rust
expectations.expect_validate_span("test.operation")
    .times(1)
    .returns(Ok(SpanValidationResult::passed()));
```

### 4. Contract-First Development

Interfaces established through mock expectations before implementation:

- All traits define expected method signatures
- Expectations specify call counts and return values
- Verification ensures all interactions occurred

## Core Team Standards Compliance

All mock contracts follow established standards:

✅ **No `.unwrap()` or `.expect()`** - Use `Result<T, String>` consistently
✅ **Sync methods only** - Maintain `dyn` trait compatibility
✅ **Clear contracts** - Explicit expected behaviors documented
✅ **Verification support** - All mocks support interaction verification
✅ **AAA pattern** - Arrange, Act, Assert in all examples
✅ **Proper error handling** - Result types with meaningful messages

## File Locations

```
/Users/sac/clnrm/swarm/tdd/mocks/
├── mod.rs                    # Module exports and MockCoordinator
├── template_mocks.rs         # Template system mock contracts
├── validation_mocks.rs       # OTEL validation mock contracts
├── plugin_mocks.rs           # Plugin system mock contracts
└── README.md                 # Comprehensive documentation
```

## Usage Example: Complete Workflow

```rust
#[test]
fn test_complete_test_execution_workflow() {
    // Arrange
    let mut coordinator = MockCoordinator::new();

    // Define template rendering expectations
    coordinator.template_expectations()
        .expect_render_file("/tests/config.toml")
        .returns(Ok("[test]\nname = \"integration\"".to_string()));

    // Define validation expectations
    coordinator.validation_expectations()
        .expect_validate_span("test.execution")
        .returns(Ok(SpanValidationResult::passed()));

    // Define plugin expectations
    coordinator.plugin_expectations()
        .expect_start()
        .returns(Ok(ServiceHandle::new("container-123")));

    // Act
    let result = test_runner.execute_complete_workflow();

    // Assert
    assert!(result.is_ok());
    coordinator.verify_all()?;
}
```

## Next Steps for Implementation

1. **Concrete Mock Implementations**
   - Create struct implementations of all mock traits
   - Implement expectation tracking and verification
   - Add builder patterns for fluent API

2. **Integration with Test Framework**
   - Connect mocks to actual test execution
   - Implement in-memory span exporters for validation
   - Add mock OTLP collector for export testing

3. **Test Coverage**
   - Unit tests for each mock implementation
   - Integration tests demonstrating mock usage
   - Acceptance tests for complete workflows

4. **Documentation**
   - Comprehensive usage examples
   - Best practices guide
   - Migration guide from existing tests

## Benefits of This Design

1. **Clear Contracts** - Every interaction is explicitly defined
2. **Testability** - Components can be tested in isolation
3. **Flexibility** - Easy to change implementations without affecting contracts
4. **Documentation** - Mocks serve as executable specifications
5. **Confidence** - Verification ensures all expected interactions occur

## Conclusion

The mock contracts provide a solid foundation for London School TDD development of v0.6.0 features. They define clear interaction patterns, support outside-in development, and maintain compliance with core team standards.

The next phase will implement concrete mock types and integrate them with the actual test framework to enable true behavior-driven development.

---

**Mock Designer**
London TDD Sub-Coordinator Team
