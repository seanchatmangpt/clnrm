# Status Validator Implementation

## Overview

The Status Validator validates OTEL span status codes (OK/ERROR/UNSET) with support for glob pattern matching. This enables flexible validation of span statuses across different span name patterns.

## Location

- **Implementation**: `crates/clnrm-core/src/validation/status_validator.rs`
- **Module Export**: `crates/clnrm-core/src/validation/mod.rs`
- **Example**: `crates/clnrm-core/examples/validation/status_validator_demo.rs`

## API Reference

### StatusCode Enum

```rust
pub enum StatusCode {
    Unset,   // Status was not set (default)
    Ok,      // Operation completed successfully
    Error,   // Operation encountered an error
}
```

**Methods:**
- `from_str(s: &str) -> Result<Self>` - Parse from string (case-insensitive)
- `as_str(&self) -> &'static str` - Get string representation

### StatusExpectation Struct

```rust
pub struct StatusExpectation {
    pub all: Option<StatusCode>,              // Expected status for all spans
    pub by_name: HashMap<String, StatusCode>, // Expected status by glob pattern
}
```

**Builder Methods:**
- `new() -> Self` - Create empty expectation
- `with_all(status: StatusCode) -> Self` - Set expected status for all spans
- `with_name_pattern(pattern: String, status: StatusCode) -> Self` - Add pattern-based expectation

**Validation:**
- `validate(&self, spans: &[SpanData]) -> Result<()>` - Validate expectations against spans

## Glob Pattern Support

The validator supports standard glob patterns:

- `*` - Matches any sequence of characters
- `?` - Matches any single character
- `[abc]` - Matches any character in the set
- `[!abc]` - Matches any character not in the set
- `[a-z]` - Matches any character in the range

## Usage Examples

### Example 1: Validate All Spans

```rust
use clnrm_core::validation::{StatusExpectation, StatusCode};

let expectation = StatusExpectation::new()
    .with_all(StatusCode::Ok);

expectation.validate(&spans)?;
```

### Example 2: Pattern-Based Validation

```rust
let expectation = StatusExpectation::new()
    .with_name_pattern("clnrm.test.*".to_string(), StatusCode::Ok)
    .with_name_pattern("external.*".to_string(), StatusCode::Error);

expectation.validate(&spans)?;
```

### Example 3: Multiple Patterns

```rust
let expectation = StatusExpectation::new()
    .with_name_pattern("backend.*".to_string(), StatusCode::Ok)
    .with_name_pattern("frontend.*".to_string(), StatusCode::Ok)
    .with_name_pattern("chaos.*".to_string(), StatusCode::Error)
    .with_name_pattern("monitor.*".to_string(), StatusCode::Unset);

expectation.validate(&spans)?;
```

### Example 4: Wildcard Patterns

```rust
// Match test_1, test_2, test_3, etc.
let expectation = StatusExpectation::new()
    .with_name_pattern("test_?".to_string(), StatusCode::Ok);
```

## Status Attribute Resolution

The validator checks multiple attribute keys for status codes (in order):

1. `otel.status_code` - Standard OTEL attribute
2. `status` - Alternative attribute
3. Defaults to `UNSET` if no status attribute found

## Error Messages

The validator provides clear, actionable error messages:

```
Status validation failed: span 'clnrm.test' has status ERROR but expected OK
Status validation failed: no spans match pattern 'clnrm.*'
Status validation failed: span 'http.request' matching pattern 'http.*' has status OK but expected ERROR
Invalid glob pattern '[invalid': Pattern syntax error at position 8
```

## Integration with TOML Configuration

Status expectations can be defined in `.clnrm.toml` files:

```toml
[validation.status]
# All spans must be OK
all = "OK"

# Pattern-based expectations
[validation.status.by_name]
"clnrm.test.*" = "OK"
"chaos.*" = "ERROR"
"monitor.*" = "UNSET"
```

## Core Team Standards Compliance

✅ **No `.unwrap()` or `.expect()`** - All error handling uses `Result<T, CleanroomError>`

✅ **Clear error messages** - All errors include context (span name, pattern, expected vs actual)

✅ **Comprehensive tests** - 15 unit tests covering all functionality

✅ **AAA pattern** - All tests follow Arrange-Act-Assert

✅ **No false positives** - No `Ok(())` stubs, honest error reporting

✅ **Documentation** - All public APIs documented with examples

✅ **Glob dependency** - Added `glob = "0.3"` to Cargo.toml

## Test Coverage

The implementation includes 15 comprehensive tests:

1. `test_status_code_from_str_valid` - Valid status code parsing
2. `test_status_code_from_str_case_insensitive` - Case-insensitive parsing
3. `test_status_code_from_str_invalid` - Invalid status code error
4. `test_all_status_ok` - All spans OK validation
5. `test_all_status_fails` - All spans validation failure
6. `test_glob_pattern_match` - Glob pattern matching
7. `test_glob_pattern_mismatch` - Pattern mismatch error
8. `test_glob_pattern_no_matches` - No matching spans error
9. `test_invalid_glob_pattern` - Invalid pattern syntax error
10. `test_multiple_patterns` - Multiple pattern validation
11. `test_wildcard_patterns` - Wildcard (?) pattern matching
12. `test_default_unset_status` - Default UNSET status
13. `test_alternative_status_attribute` - Alternative status attribute
14. `test_combining_all_and_pattern` - Combined all + pattern validation
15. `test_status_code_as_str` - String conversion

All tests pass with zero failures.

## Performance Considerations

- **Efficient pattern matching** - Uses optimized `glob` crate
- **Single-pass validation** - Validates all patterns in one iteration
- **Lazy evaluation** - Stops on first validation error
- **Memory efficient** - Borrows spans, no cloning

## Running the Demo

```bash
cargo run --example status-validator-demo -p clnrm-core
```

Output:
```
=== Status Validator Demo ===

Example 1: All spans OK
✓ All spans have OK status

Example 2: Glob pattern matching
✓ All patterns matched expected status

Example 3: Wildcard patterns with ?
✓ All wildcard matches have OK status

Example 4: Multiple patterns
✓ All multi-pattern validations passed

Example 5: Error case - status mismatch
✗ Expected error: ValidationError: Status validation failed: span 'clnrm.fail' has status ERROR but expected OK

Example 6: Error case - no matches for pattern
✗ Expected error: ValidationError: Status validation failed: no spans match pattern 'clnrm.*'

Example 7: Complex real-world scenario
✓ Complex scenario validated successfully

=== Demo Complete ===
```

## Future Enhancements

Potential improvements for future versions:

1. **Regex patterns** - Support regex in addition to glob patterns
2. **Status transitions** - Validate status changes over time
3. **Conditional validation** - Status expectations based on other attributes
4. **Aggregation** - Validate percentage of spans with specific status
5. **Custom status codes** - Support for framework-specific status codes

## Related Validators

- **SpanValidator** - Core span validation functionality
- **CountExpectation** - Validate span counts
- **WindowExpectation** - Time window validation
- **GraphExpectation** - Span hierarchy validation
- **OrderExpectation** - Span ordering validation

## Dependencies

```toml
glob = "0.3"  # Glob pattern matching
serde = "1.0" # Serialization
serde_json = "1.0" # JSON handling
```

## Integration Example

Complete integration with validation orchestrator:

```rust
use clnrm_core::validation::{
    SpanValidator, StatusExpectation, StatusCode,
    PrdExpectations, ValidationReport
};

// Load spans from OTEL collector
let validator = SpanValidator::from_file("spans.json")?;

// Define status expectations
let status_expectation = StatusExpectation::new()
    .with_name_pattern("clnrm.*".to_string(), StatusCode::Ok)
    .with_name_pattern("chaos.*".to_string(), StatusCode::Error);

// Validate
status_expectation.validate(validator.spans())?;

// Or integrate with PRD expectations
let prd = PrdExpectations {
    status: Some(status_expectation),
    // ... other expectations
};

let report = prd.validate(validator.spans())?;
```

## Conclusion

The Status Validator provides a robust, production-ready solution for validating OTEL span status codes with flexible glob pattern matching. It follows all core team standards, includes comprehensive tests, and integrates seamlessly with the clnrm validation framework.
