# Window Validator Implementation Summary

## Overview

Implemented temporal window validation for OpenTelemetry spans per PRD section "Expectations: Temporal Windows" (lines 115-121).

## Implementation

**File**: `/Users/sac/clnrm/crates/clnrm-core/src/validation/window_validator.rs`

### Core Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowExpectation {
    /// Name of the outer (parent) span that should contain children
    pub outer: String,
    /// Names of child spans that must be temporally contained
    pub contains: Vec<String>,
}
```

### Validation Logic

The validator enforces strict temporal containment:

```
outer.start_time <= child.start_time AND child.end_time <= outer.end_time
```

**Key Features**:

1. **Missing Timestamp Handling**: Returns clear errors if any timestamp is `None`
2. **Strict Containment**: Uses `<=` for boundaries (allows exact equality)
3. **Multiple Children**: Validates all children in `contains` list
4. **Clear Error Messages**: Includes actual timestamp values in errors
5. **Nanosecond Precision**: Works with Unix nanosecond timestamps

### Public API

```rust
impl WindowExpectation {
    pub fn new(outer: impl Into<String>, contains: Vec<String>) -> Self;
    pub fn validate(&self, spans: &[SpanData]) -> Result<()>;
}

pub struct WindowValidator;

impl WindowValidator {
    pub fn validate_windows(
        expectations: &[WindowExpectation],
        spans: &[SpanData],
    ) -> Result<()>;
}
```

### Module Integration

Added to validation module exports in `/Users/sac/clnrm/crates/clnrm-core/src/validation/mod.rs`:

```rust
pub mod window_validator;
pub use window_validator::{WindowExpectation, WindowValidator};
```

## Test Coverage

**20 comprehensive tests** covering:

### Basic Functionality
- ✅ Valid temporal containment
- ✅ Multiple children all valid
- ✅ Empty contains list (edge case)

### Error Cases
- ✅ Child starts before parent
- ✅ Child ends after parent
- ✅ Outer span not found
- ✅ Child span not found
- ✅ Missing timestamps (outer start/end, child start/end)

### Boundary Conditions
- ✅ Exact boundary containment (start equals)
- ✅ Exact boundary containment (end equals)
- ✅ Exact boundary containment (both equal)
- ✅ Off-by-one nanosecond violation

### Multiple Expectations
- ✅ Multiple children, one invalid
- ✅ WindowValidator with multiple expectations
- ✅ WindowValidator multiple expectations, one fails

### Precision
- ✅ Nanosecond precision with realistic timestamps

## Test Results

```
running 20 tests
test validation::window_validator::tests::test_child_ends_after_parent ... ok
test validation::window_validator::tests::test_child_span_missing_end_time ... ok
test validation::window_validator::tests::test_child_span_missing_start_time ... ok
test validation::window_validator::tests::test_child_span_not_found ... ok
test validation::window_validator::tests::test_child_starts_before_parent ... ok
test validation::window_validator::tests::test_empty_contains_list ... ok
test validation::window_validator::tests::test_exact_boundary_containment_both_equal ... ok
test validation::window_validator::tests::test_exact_boundary_containment_end_equals ... ok
test validation::window_validator::tests::test_exact_boundary_containment_start_equals ... ok
test validation::window_validator::tests::test_multiple_children_all_valid ... ok
test validation::window_validator::tests::test_multiple_children_one_invalid ... ok
test validation::window_validator::tests::test_nanosecond_precision ... ok
test validation::window_validator::tests::test_off_by_one_nanosecond_violation ... ok
test validation::window_validator::tests::test_outer_span_missing_end_time ... ok
test validation::window_validator::tests::test_outer_span_missing_start_time ... ok
test validation::window_validator::tests::test_outer_span_not_found ... ok
test validation::window_validator::tests::test_valid_temporal_containment ... ok
test validation::window_validator::tests::test_window_expectation_new ... ok
test validation::window_validator::tests::test_window_validator_multiple_expectations ... ok
test validation::window_validator::tests::test_window_validator_multiple_expectations_one_fails ... ok

test result: ok. 20 passed; 0 failed; 0 ignored
```

## Example Usage

### TOML Configuration

```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.step:hello_world", "clnrm.step:setup"]
```

### Rust Code

```rust
use clnrm_core::validation::{WindowExpectation, WindowValidator, SpanValidator};

// Load spans from OTEL collector output
let validator = SpanValidator::from_file("spans.json")?;
let spans = validator.spans();

// Create window expectation
let expectation = WindowExpectation::new(
    "clnrm.run",
    vec![
        "clnrm.step:hello_world".to_string(),
        "clnrm.step:setup".to_string(),
    ],
);

// Validate
expectation.validate(spans)?;

// Or validate multiple windows
let expectations = vec![expectation];
WindowValidator::validate_windows(&expectations, spans)?;
```

## Error Message Examples

### Child Starts Before Parent
```
Window validation failed: child span 'child_a' started before outer span 'root'
(child_start: 1000, outer_start: 2000)
```

### Child Ends After Parent
```
Window validation failed: child span 'child_a' ended after outer span 'root'
(child_end: 5001, outer_end: 5000)
```

### Missing Timestamps
```
Window validation failed: span 'root' missing start_time_unix_nano
```

### Span Not Found
```
Window validation failed: span 'nonexistent_child' not found in trace
```

## Compliance with PRD

✅ **Section 3.6 "Expectations: Temporal Windows"** (lines 115-121)
- Implements exact TOML schema: `outer` and `contains` fields
- Validates temporal containment per section 7.6: "`outer`'s [start,end] strictly contains listed spans"
- Handles missing timestamps gracefully
- Provides clear validation error messages

## Code Quality

✅ **FAANG-Level Standards**:
- No `.unwrap()` or `.expect()` in production code
- All functions return `Result<T, CleanroomError>`
- AAA pattern in tests (Arrange, Act, Assert)
- Descriptive test names explaining what is being tested
- Comprehensive error messages with context
- No false positives (`unimplemented!()` would be used if incomplete)

## Files Modified

1. **Created**: `/Users/sac/clnrm/crates/clnrm-core/src/validation/window_validator.rs` (562 lines)
2. **Updated**: `/Users/sac/clnrm/crates/clnrm-core/src/validation/mod.rs` (added window_validator exports)

## Integration Status

The window validator is:
- ✅ Fully implemented
- ✅ Comprehensively tested (20 tests)
- ✅ Integrated into validation module
- ✅ PRD-compliant
- ✅ Production-ready

Ready for integration into the orchestrator validation pipeline alongside:
- `span_validator` - Span-level assertions
- `graph_validator` - Graph topology validation
- `count_validator` - Cardinality validation
- `hermeticity_validator` - Isolation validation
