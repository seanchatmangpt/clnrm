# Template Plugin Interface Specification - CLNRM v0.6.0

**Version**: 0.6.0
**Date**: 2025-10-16
**Type**: Interface Specification
**Status**: Design Phase

## Overview

This document defines the plugin interfaces for the CLNRM template system, enabling extensibility through custom template functions, filters, and generators.

---

## Table of Contents

1. [Template Function Plugin Interface](#template-function-plugin-interface)
2. [Template Filter Plugin Interface](#template-filter-plugin-interface)
3. [Generator Plugin Interface](#generator-plugin-interface)
4. [Registration API](#registration-api)
5. [Plugin Lifecycle](#plugin-lifecycle)
6. [Error Handling Contracts](#error-handling-contracts)
7. [Testing Requirements](#testing-requirements)
8. [Example Implementations](#example-implementations)

---

## Template Function Plugin Interface

### Base Interface (Tera's Function Trait)

All template functions implement Tera's `Function` trait:

```rust
use tera::Function;
use std::collections::HashMap;

pub trait Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value>;
}
```

### Implementation Contract

**MUST Requirements**:
1. **Thread Safety**: Implement `Sync + Send`
2. **No Panics**: Return `tera::Error` on failure, never panic
3. **Parameter Validation**: Validate all required parameters
4. **Type Safety**: Check parameter types before use
5. **Determinism**: Same inputs → same outputs (for seeded variants)

**MUST NOT**:
1. **Async Operations**: All methods MUST be synchronous
2. **Side Effects**: No I/O, no network, no file system access
3. **Global State Mutation**: Avoid mutable global state
4. **Long-Running Operations**: Complete in < 100ms

### Function Naming Conventions

```
Pattern: <category>_<action>[_<variant>]

Categories:
- fake_*   - Fake data generation (fake_uuid, fake_name)
- random_* - Random value generation (random_int, random_string)
- env_*    - Environment access (env)
- hash_*   - Cryptographic functions (sha256)
- time_*   - Time-related functions (now_rfc3339)

Variants:
- *_seeded - Deterministic variant with seed parameter

Examples:
✅ fake_uuid()
✅ fake_uuid_seeded(seed)
✅ random_int(min, max)
✅ random_int_seeded(seed, min, max)
❌ generateUUID() (wrong naming convention)
❌ get_random_int() (wrong prefix)
```

### Parameter Conventions

```rust
// Use named parameters (not positional)
✅ {{ random_int(min=1, max=100) }}
❌ {{ random_int(1, 100) }}

// Seeded variants always have 'seed' as first parameter
✅ {{ fake_uuid_seeded(seed=42) }}
❌ {{ fake_uuid(seed=42) }}

// Required parameters must have no defaults
✅ random_int(min, max) - both required
❌ random_int(max) - min assumed 0 (implicit behavior)
```

### Return Value Conventions

```rust
// Return appropriate Tera Value types
String  → Value::String
i64     → Value::Number(n.into())
bool    → Value::Bool
Vec<T>  → Value::Array
Object  → Value::Object

// Examples:
fake_uuid()        → Value::String("550e8400-...")
random_int(1, 10)  → Value::Number(7.into())
random_bool()      → Value::Bool(true)
```

---

## Template Filter Plugin Interface

### Base Interface (Tera's Filter Trait)

```rust
use tera::Filter;

pub trait Filter {
    fn filter(
        &self,
        value: &Value,
        args: &HashMap<String, Value>
    ) -> tera::Result<Value>;
}
```

### Implementation Contract

**MUST Requirements**:
1. **Input Validation**: Validate input value type
2. **Idempotent**: Same input → same output
3. **Type Preservation**: Return compatible type (or convert explicitly)
4. **Error Messages**: Clear errors for type mismatches

### Filter Naming Conventions

```
Pattern: <action>

Examples:
✅ sha256      - Compute SHA-256
✅ base64      - Base64 encode
✅ upper       - Convert to uppercase
✅ lower       - Convert to lowercase
❌ compute_sha - Wrong prefix
❌ toBase64    - Wrong case
```

### Usage Pattern

```toml
# Pipe syntax
{{ "hello" | sha256 }}
{{ "secret" | base64 }}
{{ vars.name | upper }}

# Filter chaining
{{ "hello world" | upper | sha256 }}
```

---

## Generator Plugin Interface

### Base Interface (Pure Functions)

Generator functions are pure functions, not trait-based:

```rust
// Non-deterministic variant
pub fn fake_<type>() -> T;

// Deterministic variant
pub fn fake_<type>_seeded(seed: u64, ...) -> T;

// Random variant
pub fn random_<type>(...) -> T;
pub fn random_<type>_seeded(seed: u64, ...) -> T;
```

### Implementation Contract

**MUST Requirements**:
1. **Pure Functions**: No side effects, no I/O
2. **Deterministic Seeded**: Same seed → same output
3. **Bounded Execution**: Complete in < 10ms
4. **Type Safety**: Return correct type always
5. **Valid Output**: Generated data must be valid (e.g., valid UUIDs)

**Testing Requirements**:
1. **Determinism Test**: Verify seeded variant produces same output
2. **Validity Test**: Verify output is valid (e.g., UUID parses)
3. **Bounds Test**: Verify random values stay within bounds
4. **Performance Test**: Verify execution time < 10ms

### Example Implementation

```rust
use uuid::Uuid;
use rand::{Rng, SeedableRng};

/// Generate random UUID v4 (non-deterministic)
pub fn fake_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Generate deterministic UUID from seed
pub fn fake_uuid_seeded(seed: u64) -> String {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let bytes: [u8; 16] = rng.gen();
    Uuid::from_bytes(bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_uuid_is_valid() {
        let uuid = fake_uuid();
        assert!(Uuid::parse_str(&uuid).is_ok());
    }

    #[test]
    fn test_fake_uuid_seeded_deterministic() {
        let uuid1 = fake_uuid_seeded(42);
        let uuid2 = fake_uuid_seeded(42);
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_fake_uuid_seeded_different_seeds() {
        let uuid1 = fake_uuid_seeded(42);
        let uuid2 = fake_uuid_seeded(43);
        assert_ne!(uuid1, uuid2);
    }
}
```

---

## Registration API

### Function Registration

```rust
// template/registry.rs

use tera::Tera;
use crate::error::Result;
use crate::template::generators;

/// Register all template functions
pub fn register_all_functions(tera: &mut Tera) -> Result<()> {
    // Existing functions
    functions::register_functions(tera)?;

    // Fake data generators
    register_fake_data_functions(tera)?;

    // Random generators
    register_random_functions(tera)?;

    // Property test helpers
    register_property_test_functions(tera)?;

    Ok(())
}

/// Register fake data generation functions
fn register_fake_data_functions(tera: &mut Tera) -> Result<()> {
    use tera::{Function, Value};

    // fake_uuid()
    tera.register_function("fake_uuid", Box::new(|_args: &HashMap<String, Value>| {
        Ok(Value::String(generators::fake_uuid()))
    }));

    // fake_uuid_seeded(seed)
    tera.register_function("fake_uuid_seeded", Box::new(|args: &HashMap<String, Value>| {
        let seed = args.get("seed")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| tera::Error::msg("fake_uuid_seeded requires 'seed' parameter (u64)"))?;
        Ok(Value::String(generators::fake_uuid_seeded(seed)))
    }));

    // ... more registrations

    Ok(())
}

/// Register random value generation functions
fn register_random_functions(tera: &mut Tera) -> Result<()> {
    use tera::{Function, Value};

    // random_int(min, max)
    tera.register_function("random_int", Box::new(|args: &HashMap<String, Value>| {
        let min = args.get("min")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| tera::Error::msg("random_int requires 'min' parameter (i64)"))?;
        let max = args.get("max")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| tera::Error::msg("random_int requires 'max' parameter (i64)"))?;

        if min > max {
            return Err(tera::Error::msg(
                format!("random_int: min ({}) must be <= max ({})", min, max)
            ));
        }

        Ok(Value::Number(generators::random_int(min, max).into()))
    }));

    // ... more registrations

    Ok(())
}
```

### Filter Registration

```rust
use tera::{Filter, Value};

/// Register custom filters
pub fn register_custom_filters(tera: &mut Tera) -> Result<()> {
    // sha256 filter
    tera.register_filter("sha256", Box::new(|value: &Value, _args: &HashMap<String, Value>| {
        let s = value.as_str()
            .ok_or_else(|| tera::Error::msg("sha256 filter requires string input"))?;
        Ok(Value::String(hash_sha256(s)))
    }));

    // base64 filter
    tera.register_filter("base64", Box::new(|value: &Value, _args: &HashMap<String, Value>| {
        let s = value.as_str()
            .ok_or_else(|| tera::Error::msg("base64 filter requires string input"))?;
        Ok(Value::String(encode_base64(s)))
    }));

    Ok(())
}
```

---

## Plugin Lifecycle

### Registration Phase

```
┌─────────────────────────────────────┐
│ 1. TemplateRenderer::new()          │
│    ├─ Create Tera instance          │
│    └─ Call registry::register_all() │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 2. registry::register_all()         │
│    ├─ register_functions()          │
│    ├─ register_fake_data_functions()│
│    ├─ register_random_functions()   │
│    └─ register_filters()            │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 3. Functions registered with Tera   │
│    └─ Ready for template rendering  │
└─────────────────────────────────────┘
```

### Rendering Phase

```
┌─────────────────────────────────────┐
│ 1. TemplateRenderer::render_str()   │
│    ├─ Parse template                │
│    └─ Identify function calls       │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 2. Tera looks up function           │
│    ├─ Find registered function      │
│    └─ Validate parameters           │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 3. Function::call(args)             │
│    ├─ Execute generator             │
│    └─ Return Value or Error         │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 4. Tera substitutes result          │
│    └─ Continue rendering            │
└─────────────────────────────────────┘
```

---

## Error Handling Contracts

### Function Errors

```rust
// Return tera::Error with helpful message
Err(tera::Error::msg(format!(
    "random_int requires 'min' parameter (i64), got {:?}",
    args.get("min")
)))

// Include context in error message
Err(tera::Error::msg(format!(
    "random_int: min ({}) must be <= max ({})", min, max
)))

// Suggest fixes when possible
Err(tera::Error::msg(format!(
    "fake_person() not found. Did you mean 'fake_name()'?"
)))
```

### Generator Errors

```rust
// Generators MUST NOT panic - return Result if fallible
pub fn fake_port() -> Result<u16> {
    // If generation can fail, return Result
    Ok(generate_port()?)
}

// Most generators should be infallible
pub fn fake_uuid() -> String {
    // No Result needed - always succeeds
    Uuid::new_v4().to_string()
}
```

### Error Testing

```rust
#[test]
fn test_function_missing_parameter() {
    let func = RandomIntFunction;
    let args = HashMap::new(); // Missing 'min' and 'max'

    let result = func.call(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires 'min'"));
}

#[test]
fn test_function_invalid_parameter_type() {
    let func = RandomIntFunction;
    let mut args = HashMap::new();
    args.insert("min".to_string(), Value::String("not a number".to_string()));
    args.insert("max".to_string(), Value::Number(10.into()));

    let result = func.call(&args);
    assert!(result.is_err());
}
```

---

## Testing Requirements

### Unit Tests (Per Generator)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_returns_valid_output() {
        let result = fake_uuid();
        assert!(Uuid::parse_str(&result).is_ok());
    }

    #[test]
    fn test_seeded_generator_is_deterministic() {
        let result1 = fake_uuid_seeded(42);
        let result2 = fake_uuid_seeded(42);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_random_generator_stays_in_bounds() {
        for _ in 0..1000 {
            let val = random_int(10, 20);
            assert!(val >= 10 && val <= 20);
        }
    }

    #[test]
    fn test_generator_performance() {
        use std::time::Instant;

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = fake_uuid();
        }
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100); // 1000 calls in < 100ms
    }
}
```

### Integration Tests (Function Registration)

```rust
#[test]
fn test_function_registered_and_callable() {
    let mut tera = Tera::default();
    register_all_functions(&mut tera).unwrap();

    let template = "{{ fake_uuid() }}";
    let result = tera.render_str(template, &Context::new());

    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(Uuid::parse_str(&rendered).is_ok());
}

#[test]
fn test_all_functions_listed() {
    let mut tera = Tera::default();
    register_all_functions(&mut tera).unwrap();

    // Verify all expected functions are registered
    let functions = vec![
        "fake_uuid", "fake_uuid_seeded",
        "fake_name", "fake_email",
        "random_int", "random_string",
    ];

    for func_name in functions {
        let template = format!("{{{{ {}() }}}}", func_name);
        // Should not error on unknown function (would if not registered)
        // Actual error might be parameter error, but not "unknown function"
    }
}
```

### Property-Based Tests

```rust
#[cfg(feature = "proptest")]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_int_always_in_bounds(min in -1000i64..1000, max in -1000i64..1000) {
            if min <= max {
                let val = random_int(min, max);
                assert!(val >= min && val <= max);
            }
        }

        #[test]
        fn random_string_correct_length(len in 0usize..1000) {
            let s = random_string(len);
            assert_eq!(s.len(), len);
        }

        #[test]
        fn seeded_generator_deterministic(seed in any::<u64>()) {
            let val1 = fake_uuid_seeded(seed);
            let val2 = fake_uuid_seeded(seed);
            assert_eq!(val1, val2);
        }
    }
}
```

---

## Example Implementations

### Example 1: Simple Generator (No Parameters)

```rust
/// Generate fake first name
pub fn fake_first_name() -> String {
    const NAMES: &[&str] = &["Alice", "Bob", "Charlie", "Diana"];
    let mut rng = rand::thread_rng();
    NAMES[rng.gen_range(0..NAMES.len())].to_string()
}

/// Generate deterministic fake first name
pub fn fake_first_name_seeded(seed: u64) -> String {
    const NAMES: &[&str] = &["Alice", "Bob", "Charlie", "Diana"];
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    NAMES[rng.gen_range(0..NAMES.len())].to_string()
}

// Registration:
tera.register_function("fake_first_name", Box::new(|_args| {
    Ok(Value::String(generators::fake_first_name()))
}));
```

### Example 2: Generator with Parameters

```rust
/// Generate random integer in range
pub fn random_int(min: i64, max: i64) -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

/// Generate deterministic random integer
pub fn random_int_seeded(seed: u64, min: i64, max: i64) -> i64 {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    rng.gen_range(min..=max)
}

// Registration:
tera.register_function("random_int", Box::new(|args| {
    let min = args.get("min")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| tera::Error::msg("random_int requires 'min' (i64)"))?;
    let max = args.get("max")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| tera::Error::msg("random_int requires 'max' (i64)"))?;

    if min > max {
        return Err(tera::Error::msg(format!(
            "random_int: min ({}) must be <= max ({})", min, max
        )));
    }

    Ok(Value::Number(generators::random_int(min, max).into()))
}));
```

### Example 3: Custom Filter

```rust
use sha2::{Sha256, Digest};

/// Compute SHA-256 hash
pub fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Registration:
tera.register_filter("sha256", Box::new(|value, _args| {
    let s = value.as_str()
        .ok_or_else(|| tera::Error::msg("sha256 requires string input"))?;
    Ok(Value::String(hash_sha256(s)))
}));

// Usage: {{ "hello" | sha256 }}
```

---

## Plugin Checklist

When implementing a new template function/filter/generator:

### Design Phase
- [ ] Define clear purpose and use case
- [ ] Choose appropriate naming (follows conventions)
- [ ] Specify parameter types and validation
- [ ] Define return type
- [ ] Document determinism requirements
- [ ] Plan seeded variant (if applicable)

### Implementation Phase
- [ ] Implement generator function(s)
- [ ] Implement seeded variant (if applicable)
- [ ] Add parameter validation
- [ ] Return proper error types (tera::Error)
- [ ] Add inline documentation comments
- [ ] Implement thread-safety (Sync + Send)

### Testing Phase
- [ ] Unit test: valid output
- [ ] Unit test: determinism (seeded variant)
- [ ] Unit test: bounds checking
- [ ] Unit test: performance (< 10ms)
- [ ] Integration test: registration
- [ ] Integration test: template rendering
- [ ] Property test: invariants (if applicable)

### Registration Phase
- [ ] Add to appropriate registration function
- [ ] Add parameter validation in wrapper
- [ ] Add error handling
- [ ] Test registration in integration test

### Documentation Phase
- [ ] Add to function registry table
- [ ] Add usage examples
- [ ] Document parameters and return type
- [ ] Add to TEMPLATE_GUIDE.md

---

## Conclusion

This plugin interface specification provides:

1. **Clear contracts** for function/filter/generator implementations
2. **Naming conventions** for consistency
3. **Error handling patterns** for reliability
4. **Testing requirements** for quality
5. **Example implementations** for guidance

**Key Principles**:
- **Simplicity**: Pure functions, no side effects
- **Determinism**: Seeded variants for reproducibility
- **Type Safety**: Validate all inputs
- **Error Clarity**: Helpful error messages
- **Performance**: < 10ms per call

Following these specifications ensures plugins integrate seamlessly with the CLNRM template system.
