# OTEL Validation Orchestrator

## Overview

The validation orchestrator (`orchestrator.rs`) provides a unified interface to run all OTEL PRD validation checks and generate comprehensive reports.

## Architecture

### Core Components

1. **PrdExpectations** - Aggregates all validation expectations
2. **ValidationReport** - Collects passes and failures from all validators
3. **Individual Validators** - Specialized validators for different aspects

### Validation Modules

| Module | Purpose | Key Features |
|--------|---------|--------------|
| `span_validator` | Basic span assertions | Existence, count, attributes, hierarchy |
| `graph_validator` | Topology validation | Parent-child relationships, cycles |
| `count_validator` | Cardinality checks | Exact, min, max counts by name |
| `window_validator` | Temporal containment | Span time boundaries |
| `hermeticity_validator` | Isolation checks | No external services, resource attributes |

## Usage

### Basic Example

```rust
use clnrm_core::validation::{
    PrdExpectations,
    GraphExpectation,
    CountExpectation,
    count_validator::CountBound,
    HermeticityExpectation,
    SpanValidator,
};

// Load spans from OTEL collector
let validator = SpanValidator::from_file("spans.json")?;
let spans = validator.spans();

// Configure expectations
let graph = GraphExpectation::new(vec![
    ("clnrm.run".to_string(), "clnrm.test".to_string()),
]);

let counts = CountExpectation::new()
    .with_name_count("clnrm.run".to_string(), CountBound::eq(1))
    .with_name_count("clnrm.test".to_string(), CountBound::gte(1));

let hermeticity = HermeticityExpectation::no_external_services();

// Create orchestrator
let expectations = PrdExpectations::new()
    .with_graph(graph)
    .with_counts(counts)
    .with_hermeticity(hermeticity);

// Run all validations
let report = expectations.validate_all(spans)?;

// Check results
if report.is_success() {
    println!("✓ All validations passed!");
} else {
    eprintln!("✗ Validation failed:\n{}", report.summary());
}
```

### Strict Mode

Fail on first error:

```rust
// Returns Ok(()) if all pass, Err with details if any fail
expectations.validate_strict(spans)?;
```

## Validation Order

The orchestrator runs validations in a specific order:

1. **Graph Topology** - Validates span relationships are correct
2. **Span Counts** - Ensures expected spans exist in right quantities
3. **Temporal Windows** - Verifies timing and containment
4. **Hermeticity** - Confirms test isolation

This order ensures foundational checks pass before more complex validations.

## ValidationReport API

```rust
// Query methods
report.is_success() -> bool
report.pass_count() -> usize
report.failure_count() -> usize
report.passes() -> &[String]
report.failures() -> &[(String, String)]
report.first_error() -> Option<&str>

// Human-readable output
report.summary() -> String
```

### Example Report Output

```
✗ 3 passed, 2 failed
  - graph_topology: Expected edge 'root' -> 'missing_child' not found in span graph
  - span_counts: Span count mismatch for 'test.span': expected exactly 2, found 1
```

## Integration with TOML Configuration

The orchestrator is designed to work with `.clnrm.toml` test definitions:

```toml
[expect.graph]
must_include = [
    ["clnrm.run", "clnrm.test"],
    ["clnrm.test", "container.exec"],
]

[expect.counts]
by_name.clnrm_run = { eq = 1 }
by_name.clnrm_test = { gte = 1 }

[expect.hermeticity]
no_external_services = true
resource_attrs_must_match = { "service.name" = "clnrm", "env" = "test" }
```

## Testing

All validators include comprehensive unit tests:

```bash
# Run orchestrator tests
cargo test -p clnrm-core --lib orchestrator

# Run all validation tests
cargo test -p clnrm-core --lib validation::

# Results: 96 tests passed
```

## Implementation Details

### Error Handling

- All validators return `Result<(), CleanroomError>`
- Errors include detailed context (span names, expected/actual values)
- Orchestrator collects all errors, doesn't stop at first failure

### Performance

- Single-pass validation
- Efficient span lookups via HashMap indices
- Minimal allocations

### Extensibility

Adding new validators:

1. Create validator module in `src/validation/`
2. Implement validation logic with clear error messages
3. Add expectation type to `PrdExpectations`
4. Update `validate_all()` to include new validator
5. Export types in `validation/mod.rs`

## Related Documentation

- [OTEL PRD](../OTEL-PRD.md) - Complete requirements
- [Span Validator](../crates/clnrm-core/src/validation/span_validator.rs) - Span assertions
- [Testing Guide](./TESTING.md) - Framework testing approach

## Status

- **Implementation**: Complete ✅
- **Tests**: 96 tests passing ✅
- **Integration**: Ready for TOML configuration ⏳
- **Documentation**: Complete ✅
