# SDK Resource Attribute Validation Enhancement

## Overview

Enhanced the hermeticity validator to validate SDK-provided resource attributes, enabling verification that OpenTelemetry SDK is properly configured and emitting expected resource attributes.

## Changes Made

### 1. New Violation Types

Added two new violation types to `ViolationType` enum:

- `MissingSdkResourceAttribute` - SDK resource attribute is missing
- `SdkResourceAttributeMismatch` - SDK resource attribute value doesn't match expected

### 2. Enhanced HermeticityExpectation

Added new field `sdk_resource_attrs_must_match`:

```rust
pub struct HermeticityExpectation {
    pub no_external_services: Option<bool>,
    pub resource_attrs_must_match: Option<HashMap<String, String>>,
    pub sdk_resource_attrs_must_match: Option<HashMap<String, String>>, // NEW
    pub span_attrs_forbid_keys: Option<Vec<String>>,
}
```

### 3. New Constructor Method

Added `with_sdk_resource_attrs()` constructor:

```rust
pub fn with_sdk_resource_attrs(attrs: HashMap<String, String>) -> Self
```

### 4. Validation Logic

Added `check_sdk_resource_attributes()` method that:
- Validates SDK-provided resource attributes separately from user attributes
- Provides clear error messages indicating SDK configuration issues
- Supports OTEL attribute value format (stringValue wrapper)

### 5. Integration into Validation Flow

Updated `validate()` method to include SDK resource attribute checking:

```rust
// 3. Validate SDK-provided resource attributes match expected values
if let Some(ref expected_sdk_attrs) = self.sdk_resource_attrs_must_match {
    violations.extend(self.check_sdk_resource_attributes(spans, expected_sdk_attrs));
}
```

## Usage Example

### TOML Configuration

```toml
[expect.hermeticity]
resource_attrs.must_match = { "service.name" = "clnrm", "env" = "test" }
sdk_resource_attrs.must_match = { "telemetry.sdk.language" = "rust" }
```

### Programmatic Usage

```rust
use clnrm_core::validation::hermeticity_validator::HermeticityExpectation;
use std::collections::HashMap;

let mut sdk_attrs = HashMap::new();
sdk_attrs.insert("telemetry.sdk.language".to_string(), "rust".to_string());

let expectation = HermeticityExpectation::with_sdk_resource_attrs(sdk_attrs);

// Validate spans
let result = expectation.validate(&spans);
```

## Test Coverage

Added 8 comprehensive tests demonstrating:

### ✅ Valid Cases

1. **test_sdk_resource_attributes_validation_passes_with_correct_language**
   - Spans with `telemetry.sdk.language=rust` pass validation

2. **test_sdk_resource_attributes_validation_with_multiple_sdk_attrs**
   - Multiple SDK attributes (language, name, version) validated correctly

3. **test_sdk_and_user_resource_attributes_validation_combined**
   - Both user and SDK attributes validated together successfully

4. **test_sdk_resource_attributes_validation_handles_otel_format**
   - OTEL-formatted attributes (with stringValue wrapper) handled correctly

### ❌ Invalid Cases

5. **test_sdk_resource_attributes_validation_fails_when_missing**
   - Missing SDK attributes detected with clear error message
   - Error includes: "SDK may not be properly configured"

6. **test_sdk_resource_attributes_validation_fails_with_wrong_language**
   - Wrong SDK language detected (expected rust, found python)
   - Error includes: "verify SDK configuration"

7. **test_sdk_resource_attributes_can_distinguish_from_user_attrs**
   - Validates SDK and user attributes separately
   - Reports specific SDK attribute errors without conflating with user attributes

8. **test_sdk_resource_attributes_validation_fails_with_empty_spans**
   - Handles edge case of no spans available

## Error Messages

### Missing SDK Attribute

```
Hermeticity validation failed with 1 violation(s):

1. Required SDK resource attribute 'telemetry.sdk.language' is missing (expected 'rust') - SDK may not be properly configured
   Span: test.span
   Span ID: span1
   Attribute: telemetry.sdk.language
   Expected: rust
```

### SDK Attribute Mismatch

```
Hermeticity validation failed with 1 violation(s):

1. SDK resource attribute 'telemetry.sdk.language' mismatch: expected 'rust', found 'python' - verify SDK configuration
   Span: test.span
   Span ID: span1
   Attribute: telemetry.sdk.language
   Expected: rust
   Actual: python
```

## Benefits

1. **SDK Configuration Validation**: Ensures OpenTelemetry SDK is properly configured
2. **Clear Separation**: Distinguishes between user-provided and SDK-provided attributes
3. **Better Debugging**: Specific error messages help identify SDK configuration issues
4. **OTEL Compliance**: Validates that SDK follows OpenTelemetry semantic conventions
5. **Comprehensive Testing**: 8 tests cover all success and failure scenarios

## Standards Compliance

✅ No `.unwrap()` or `.expect()` in production code
✅ Proper `Result<T, CleanroomError>` error handling
✅ AAA pattern tests (Arrange, Act, Assert)
✅ Clear error messages explaining what's wrong
✅ Follows FAANG-level code standards from .cursorrules

## Files Modified

- `/Users/sac/clnrm/crates/clnrm-core/src/validation/hermeticity_validator.rs`
  - Added 2 new violation types
  - Added 1 new struct field
  - Added 1 new constructor method
  - Added 1 new validation method
  - Added 8 comprehensive tests

## Files Created

- `/Users/sac/clnrm/tests/sdk_resource_validation_demo.rs`
  - Standalone demonstration of SDK resource validation
  - 4 integration tests showing real-world usage

## Next Steps

To use this feature in production:

1. Add SDK resource attribute expectations to TOML test definitions
2. Verify SDK configuration by checking for expected `telemetry.sdk.*` attributes
3. Use separate validation for user vs SDK attributes for clarity
4. Monitor SDK attribute violations to detect configuration drift

## Example Integration Test

See `/Users/sac/clnrm/tests/sdk_resource_validation_demo.rs` for complete working examples.
