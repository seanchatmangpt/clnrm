# StdoutSpanParser Implementation

## Overview

The `StdoutSpanParser` is a production-ready parser for extracting OpenTelemetry spans from container stdout. It handles mixed content (logs, debug output, spans) and provides robust error handling.

## Location

- **Implementation**: `crates/clnrm-core/src/otel/stdout_parser.rs`
- **Integration Tests**: `crates/clnrm-core/tests/integration_stdout_parser.rs`
- **Module Export**: `crates/clnrm-core/src/otel/mod.rs`

## Features

### ✅ Core Functionality

1. **Parse JSON Lines**: Extracts OTEL span JSON from mixed stdout
2. **Validate Structure**: Ensures required fields are present (`name`, `trace_id`, `span_id`)
3. **Handle Mixed Content**: Silently ignores non-JSON lines (logs, output, etc.)
4. **Error Handling**: Skips malformed spans with warnings logged via `tracing`
5. **OTEL Format Support**: Compatible with OTEL stdout exporter format

### ✅ Supported Fields

- **Required**: `name`, `trace_id`, `span_id`
- **Optional**: `parent_span_id`, `start_time_unix_nano`, `end_time_unix_nano`
- **Advanced**: `kind` (string or integer), `attributes` (object), `events` (array of strings or objects)

## Usage

```rust
use clnrm_core::otel::StdoutSpanParser;

// Container stdout with mixed content
let stdout = r#"
[2024-01-15T10:30:45Z] INFO Starting test execution
{"name":"clnrm.run","trace_id":"abc123","span_id":"root001","parent_span_id":null,"attributes":{"result":"pass"}}
[2024-01-15T10:30:46Z] INFO Container created
{"name":"clnrm.step:setup","trace_id":"abc123","span_id":"step001","parent_span_id":"root001","events":["container.start"]}
Test completed successfully
"#;

// Parse spans (ignores log lines)
let spans = StdoutSpanParser::parse(stdout)?;
assert_eq!(spans.len(), 2);

// Access parsed span data
assert_eq!(spans[0].name, "clnrm.run");
assert_eq!(spans[0].trace_id, "abc123");
assert_eq!(spans[1].parent_span_id, Some("root001".to_string()));
```

## Test Coverage

### Unit Tests (14 tests)

1. `test_parse_single_span_from_stdout` - Single span extraction
2. `test_parse_mixed_stdout_with_logs` - Mixed content handling
3. `test_parse_empty_stdout` - Empty input handling
4. `test_parse_malformed_json_logs_warning` - Malformed JSON handling
5. `test_parse_span_with_attributes` - Attribute parsing
6. `test_parse_span_with_events` - Event array parsing
7. `test_parse_span_with_event_objects` - Event object parsing
8. `test_parse_span_with_timestamps` - Timestamp parsing
9. `test_parse_span_with_kind_string` - Span kind (string) parsing
10. `test_parse_span_with_kind_int` - Span kind (integer) parsing
11. `test_parse_multiple_spans_same_trace` - Multiple spans per trace
12. `test_parse_json_non_span_ignored` - Non-span JSON filtering
13. `test_parse_whitespace_lines_ignored` - Whitespace handling
14. `test_parse_missing_required_field_logs_warning` - Required field validation

### Integration Tests (8 tests)

1. `test_parse_realistic_container_output` - Realistic container output scenario
2. `test_parse_otel_stdout_exporter_format` - OTEL stdout exporter compatibility
3. `test_parse_with_debug_output_and_errors` - Mixed debug/error output
4. `test_parse_with_json_logs_that_are_not_spans` - Structured JSON log filtering
5. `test_parse_empty_and_whitespace_only_output` - Edge case handling
6. `test_parse_with_unicode_and_special_characters` - Unicode support
7. `test_parse_large_batch_of_spans` - Performance with 100+ spans
8. `test_parse_with_malformed_json_mixed_in` - Robustness with malformed data

## Code Quality

### ✅ Core Team Standards Compliance

- **No `.unwrap()` or `.expect()`**: All error handling uses `Result` types
- **Sync functions**: Parser is `dyn` compatible (no async trait methods)
- **AAA pattern tests**: All tests follow Arrange-Act-Assert pattern
- **Descriptive names**: Function and variable names explain intent
- **Zero clippy warnings**: No clippy issues for `stdout_parser` module

### Error Handling Strategy

```rust
// Malformed JSON → Log warning, continue parsing
if let Ok(value) = serde_json::from_str::<Value>(line) {
    if Self::is_span_like(&value) {
        match Self::parse_span(&value) {
            Ok(span) => spans.push(span),
            Err(e) => {
                tracing::warn!(
                    line = line_num + 1,
                    error = %e,
                    "Failed to parse span-like JSON object"
                );
            }
        }
    }
}
```

## Integration Points

### 1. OTEL Module

```rust
// Re-exported from otel module
use clnrm_core::otel::StdoutSpanParser;
```

### 2. Span Validation

```rust
use clnrm_core::otel::StdoutSpanParser;
use clnrm_core::validation::span_validator::SpanValidator;

// Parse spans from container stdout
let spans_data = StdoutSpanParser::parse(stdout)?;

// Create validator from parsed spans (convert to JSON)
let json = serde_json::to_string(&spans_data)?;
let validator = SpanValidator::from_json(&json)?;

// Validate expectations
let result = validator.validate_expectations(&expectations)?;
```

## Performance Characteristics

- **Time Complexity**: O(n) where n is number of lines in stdout
- **Space Complexity**: O(m) where m is number of valid spans found
- **Tested Scale**: 100+ spans parsed in < 1ms

## Example Output Formats

### OTEL Stdout Exporter Format

```json
{"name":"http.request","trace_id":"trace123","span_id":"span123","parent_span_id":null,"kind":2,"attributes":{"http.method":"GET","http.url":"/api/test"},"start_time_unix_nano":"1700000000000000","end_time_unix_nano":"1700000001000000"}
```

### Clnrm Format

```json
{"name":"clnrm.run","trace_id":"abc123","span_id":"root001","parent_span_id":null,"attributes":{"test.name":"integration_test","result":"pass"},"events":["container.start","container.exec"]}
```

## Future Enhancements

Potential improvements (not currently required):

1. **Streaming Parser**: Parse spans incrementally as stdout arrives
2. **Schema Validation**: Validate against OTEL JSON schema
3. **Performance Metrics**: Track parsing performance with OTEL metrics
4. **Compression Support**: Handle compressed span data

## References

- **OTEL Specification**: https://opentelemetry.io/docs/specs/otel/trace/sdk/
- **Span Data Structure**: `crates/clnrm-core/src/validation/span_validator.rs`
- **OTEL Validators**: `crates/clnrm-core/src/otel/validators/`
