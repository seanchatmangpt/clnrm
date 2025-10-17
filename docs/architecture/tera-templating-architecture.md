# Tera Templating Architecture for Property-Based Testing in CLNRM

**Document Type**: System Architecture Design
**Version**: 1.0.0
**Date**: 2025-10-16
**Status**: DESIGN PHASE - NOT IMPLEMENTED
**Author**: System Architect

## Executive Summary

This document defines a complete Tera templating system that enables property-based testing and fake data generation in `.clnrm.toml` files. The design maintains backward compatibility while adding powerful generative capabilities for creating large-scale test suites with randomized, property-based scenarios.

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Diagram](#architecture-diagram)
3. [Integration Points](#integration-points)
4. [Custom Tera Functions & Filters](#custom-tera-functions--filters)
5. [Template TOML Examples](#template-toml-examples)
6. [Implementation Plan](#implementation-plan)
7. [Error Handling Strategy](#error-handling-strategy)
8. [Testing Strategy](#testing-strategy)
9. [Backward Compatibility](#backward-compatibility)
10. [Performance Considerations](#performance-considerations)
11. [Security Considerations](#security-considerations)

---

## System Overview

### Goals

1. **Property-Based Testing**: Generate hundreds/thousands of test scenarios with varying parameters
2. **Fake Data Generation**: Built-in functions for UUIDs, names, emails, timestamps, etc.
3. **TOML Template Support**: Pre-process TOML with Tera before parsing
4. **Backward Compatibility**: Non-templated TOML files work unchanged
5. **Clean Integration**: Minimal changes to existing config parsing pipeline

### Non-Goals

- Runtime template rendering (templates render at config load time only)
- Dynamic template variables from test execution
- Template inheritance beyond Tera's built-in features
- Template debugging UI (CLI only)

### Key Design Principles

1. **Render Before Parse**: Templates render to TOML text, then parse normally
2. **Fail Fast**: Template errors caught before TOML parsing
3. **Deterministic**: Same template renders identically with fixed seeds
4. **Composable**: Mix templating with static TOML seamlessly

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                     TOML File Loading Pipeline                      │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│ User creates     │
│ .clnrm.toml.tera │  ← New: Template files with .tera extension
│ file             │
└────────┬─────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│ Step 1: File Detection                                     │
│ ────────────────────────────────────────────────────────── │
│ load_config_from_file(path)                               │
│   ├─ Check extension:                                      │
│   │   ├─ .toml → Skip template rendering                  │
│   │   └─ .tera or .toml.tera → Render template            │
│   └─ Read file contents                                    │
└────────────────────┬───────────────────────────────────────┘
                     │
                     ▼
         ┌─────────────────────┐
         │ Is template file?   │
         └───────┬─────────────┘
                 │
    ┌────────────┼────────────┐
    │ NO         │ YES        │
    ▼            ▼            │
┌───────┐   ┌────────────────────────────────────────────┐
│ TOML  │   │ Step 2: Template Rendering                 │
│ Text  │   │ ────────────────────────────────────────── │
│       │   │ config::template::render_template()        │
│       │   │   ├─ Initialize Tera engine                │
│       │   │   ├─ Register custom functions              │
│       │   │   │   ├─ fake_*() generators                │
│       │   │   │   ├─ random_*() functions               │
│       │   │   │   └─ property_test() macro              │
│       │   │   ├─ Register custom filters                │
│       │   │   ├─ Render template to String              │
│       │   │   └─ Handle template errors → CleanroomError│
│       │   └────────────────────┬───────────────────────┘
└───┬───┘                        │
    │                            ▼
    │                   ┌────────────────┐
    │                   │ Rendered TOML  │
    │                   │ Text           │
    │                   └────────┬───────┘
    │                            │
    └────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────────────────────┐
│ Step 3: TOML Parsing (UNCHANGED)                          │
│ ────────────────────────────────────────────────────────── │
│ parse_toml_config(content)                                │
│   ├─ Parse with toml crate                                │
│   ├─ Deserialize to TestConfig                            │
│   └─ Return TestConfig or CleanroomError                  │
└────────────────────┬───────────────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────────────────────┐
│ Step 4: Validation (UNCHANGED)                            │
│ ────────────────────────────────────────────────────────── │
│ config.validate()                                          │
│   ├─ Validate metadata                                     │
│   ├─ Validate services                                     │
│   ├─ Validate steps                                        │
│   └─ Return Ok(()) or CleanroomError                      │
└────────────────────┬───────────────────────────────────────┘
                     │
                     ▼
              ┌──────────────┐
              │ TestConfig   │
              │ (Ready)      │
              └──────────────┘
```

### Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   New Module Structure                       │
└─────────────────────────────────────────────────────────────┘

src/config/
├── mod.rs                    # Existing: TestConfig, load_config_from_file
│   └── Modified: Add template rendering step
├── template.rs               # NEW: Tera rendering engine
│   ├── render_template(content: &str) -> Result<String>
│   ├── init_tera_engine() -> Tera
│   ├── register_custom_functions(tera: &mut Tera)
│   └── register_custom_filters(tera: &mut Tera)
└── fake_data.rs              # NEW: Fake data generators
    ├── fake_uuid() -> String
    ├── fake_name() -> String
    ├── fake_email() -> String
    ├── fake_timestamp() -> i64
    ├── random_int(min, max) -> i64
    ├── random_string(length) -> String
    └── random_bool() -> bool
```

---

## Integration Points

### 1. Modified: `config::load_config_from_file()`

**Current Implementation** (lines 681-689 in config.rs):

```rust
pub fn load_config_from_file(path: &std::path::Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    let config = parse_toml_config(&content)?;
    config.validate()?;

    Ok(config)
}
```

**Proposed Implementation**:

```rust
pub fn load_config_from_file(path: &std::path::Path) -> Result<TestConfig> {
    // Read raw file contents
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Check if template rendering is needed
    let rendered_content = if is_template_file(path) {
        // NEW: Render template before parsing
        template::render_template(&content)
            .map_err(|e| CleanroomError::config_error(format!("Template rendering failed: {}", e)))?
    } else {
        // No template rendering needed
        content
    };

    // Parse TOML (unchanged)
    let config = parse_toml_config(&rendered_content)?;

    // Validate (unchanged)
    config.validate()?;

    Ok(config)
}

fn is_template_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "tera" || path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".toml.tera"))
            .unwrap_or(false))
        .unwrap_or(false)
}
```

### 2. New Module: `config::template`

**Location**: `/Users/sac/clnrm/crates/clnrm-core/src/config/template.rs`

**Responsibilities**:
- Initialize Tera engine with custom functions
- Render template strings to TOML
- Error handling for template errors

**Key Functions**:

```rust
use crate::error::{CleanroomError, Result};
use tera::{Tera, Context};

/// Render a Tera template string to TOML
pub fn render_template(template_content: &str) -> Result<String> {
    let mut tera = init_tera_engine()?;

    // Create empty context (templates are self-contained)
    let context = Context::new();

    // Render template
    tera.render_str(template_content, &context)
        .map_err(|e| CleanroomError::config_error(
            format!("Tera template rendering failed: {}", e)
        ))
}

/// Initialize Tera engine with custom functions and filters
fn init_tera_engine() -> Result<Tera> {
    let mut tera = Tera::default();

    // Register custom functions
    register_custom_functions(&mut tera)?;

    // Register custom filters
    register_custom_filters(&mut tera)?;

    Ok(tera)
}
```

### 3. New Module: `config::fake_data`

**Location**: `/Users/sac/clnrm/crates/clnrm-core/src/config/fake_data.rs`

**Responsibilities**:
- Generate fake data (UUIDs, names, emails, etc.)
- Deterministic random generation with seeds
- Property-based test helpers

---

## Custom Tera Functions & Filters

### Category 1: Fake Data Generators

#### `fake_uuid()`
**Returns**: UUID v4 string
**Example**: `"550e8400-e29b-41d4-a716-446655440000"`
**Determinism**: New UUID per call (use `fake_uuid_seeded()` for determinism)

```rust
use uuid::Uuid;

pub fn fake_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn fake_uuid_seeded(seed: u64) -> String {
    // Use seed for deterministic UUID generation
    use rand::{SeedableRng, Rng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let bytes: [u8; 16] = rng.gen();
    Uuid::from_bytes(bytes).to_string()
}
```

**Template Usage**:
```toml
[[steps]]
name = "test_{{ fake_uuid() }}"
command = ["echo", "{{ fake_uuid() }}"]
```

#### `fake_name()`
**Returns**: Random full name
**Example**: `"John Doe"`, `"Jane Smith"`

```rust
const FIRST_NAMES: &[&str] = &["John", "Jane", "Alice", "Bob", "Charlie", "Diana", "Emma", "Frank"];
const LAST_NAMES: &[&str] = &["Doe", "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller"];

pub fn fake_name() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let first = FIRST_NAMES[rng.gen_range(0..FIRST_NAMES.len())];
    let last = LAST_NAMES[rng.gen_range(0..LAST_NAMES.len())];
    format!("{} {}", first, last)
}
```

**Template Usage**:
```toml
[services.user_service.env]
TEST_USER = "{{ fake_name() }}"
```

#### `fake_email()`
**Returns**: Random email address
**Example**: `"test_abc123@example.com"`

```rust
pub fn fake_email() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen();
    format!("test_{}@example.com", id)
}
```

#### `fake_timestamp()`
**Returns**: Unix timestamp (seconds since epoch)
**Example**: `1729123456`

```rust
pub fn fake_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn fake_timestamp_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
```

#### `fake_ipv4()`
**Returns**: Random IPv4 address
**Example**: `"192.168.1.42"`

```rust
pub fn fake_ipv4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{}.{}.{}.{}",
        rng.gen_range(1..255),
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        rng.gen_range(1..255)
    )
}
```

### Category 2: Random Value Generators

#### `random_int(min, max)`
**Arguments**: `min: i64, max: i64`
**Returns**: Random integer in range [min, max]

```rust
pub fn random_int(min: i64, max: i64) -> i64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}
```

**Template Usage**:
```toml
[[steps]]
name = "load_test_{{ random_int(1, 1000) }}"
expected_exit_code = {{ random_int(0, 0) }}
```

#### `random_string(length)`
**Arguments**: `length: usize`
**Returns**: Random alphanumeric string

```rust
pub fn random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

#### `random_bool()`
**Returns**: Random boolean

```rust
pub fn random_bool() -> bool {
    use rand::Rng;
    rand::thread_rng().gen()
}
```

#### `random_choice(items)`
**Arguments**: `items: Vec<String>`
**Returns**: Random item from list

```rust
pub fn random_choice(items: Vec<String>) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    items[rng.gen_range(0..items.len())].clone()
}
```

**Template Usage**:
```toml
[services.app.env]
LOG_LEVEL = "{{ random_choice(items=['debug', 'info', 'warn', 'error']) }}"
```

### Category 3: Property Test Helpers

#### `property_range(start, end)`
**Returns**: Array of integers from start to end
**Example**: `property_range(0, 5)` → `[0, 1, 2, 3, 4, 5]`

```rust
pub fn property_range(start: i64, end: i64) -> Vec<i64> {
    (start..=end).collect()
}
```

**Template Usage**:
```toml
{% for i in property_range(0, 100) %}
[[steps]]
name = "property_test_{{ i }}"
command = ["echo", "iteration_{{ i }}"]
{% endfor %}
```

### Category 4: Custom Filters

#### `upper` (Tera built-in)
**Example**: `{{ "hello" | upper }}` → `"HELLO"`

#### `lower` (Tera built-in)
**Example**: `{{ "HELLO" | lower }}` → `"hello"`

#### `sha256` (Custom)
**Returns**: SHA-256 hash of input

```rust
use sha2::{Sha256, Digest};

pub fn filter_sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**Template Usage**:
```toml
[services.auth.env]
PASSWORD_HASH = "{{ 'my_password' | sha256 }}"
```

#### `base64` (Custom)
**Returns**: Base64 encoding of input

```rust
use base64::{Engine as _, engine::general_purpose};

pub fn filter_base64(value: &str) -> String {
    general_purpose::STANDARD.encode(value.as_bytes())
}
```

---

## Template TOML Examples

### Example 1: Property-Based Load Testing (100 Scenarios)

**File**: `tests/load-test.clnrm.toml.tera`

```toml
# Property-based load test - 100 concurrent requests
[test.metadata]
name = "load_test_property_based"
description = "Property-based load test with 100 random scenarios"

[services.api]
type = "generic_container"
plugin = "generic_container"
image = "my-api:latest"
env = { PORT = "8080" }

# Generate 100 test scenarios with random data
{% for i in range(end=100) %}
[[steps]]
name = "request_{{ i }}_{{ fake_uuid() }}"
service = "api"
command = ["curl", "-X", "POST", "http://localhost:8080/api/users",
           "-H", "Content-Type: application/json",
           "-d", '{"name":"{{ fake_name() }}","email":"{{ fake_email() }}","id":"{{ fake_uuid() }}"}']
expected_exit_code = 0
expected_output_regex = "20[01]"
{% endfor %}

[assertions]
total_requests = 100
all_requests_successful = true
```

### Example 2: Randomized Chaos Testing

**File**: `tests/chaos-random.clnrm.toml.tera`

```toml
[test.metadata]
name = "chaos_random_failures"
description = "Chaos testing with random failure injection"

[services.app]
type = "generic_container"
plugin = "generic_container"
image = "app:test"

[services.chaos]
type = "generic_container"
plugin = "chaos_engine"
image = "chaos:latest"

# Random chaos scenarios
{% for i in range(end=50) %}
[[steps]]
name = "chaos_{{ i }}"
service = "chaos"
{% set action = random_choice(items=['kill_process', 'network_delay', 'cpu_stress', 'memory_pressure']) %}
command = ["chaos", "{{ action }}", "--duration", "{{ random_int(5, 30) }}s"]
expected_exit_code = 0
{% endfor %}
```

### Example 3: Multi-Database Property Tests

**File**: `tests/db-property-test.clnrm.toml.tera`

```toml
[test.metadata]
name = "database_property_tests"
description = "Property-based tests for database operations"

{% set databases = ['postgres', 'mysql', 'mongodb'] %}
{% for db in databases %}
[services.{{ db }}]
type = "generic_container"
plugin = "generic_container"
image = "{{ db }}:latest"
env = { DATABASE = "test_{{ fake_uuid() }}" }
{% endfor %}

# Test each database with 20 random operations
{% for db in databases %}
{% for i in range(end=20) %}
[[steps]]
name = "{{ db }}_insert_{{ i }}"
service = "{{ db }}"
command = ["db-cli", "insert",
           "--table", "users",
           "--data", '{"id":"{{ fake_uuid() }}","name":"{{ fake_name() }}","email":"{{ fake_email() }}","created":"{{ fake_timestamp() }}"}']
expected_exit_code = 0
{% endfor %}
{% endfor %}
```

### Example 4: Seeded Deterministic Tests

**File**: `tests/deterministic-property.clnrm.toml.tera`

```toml
# Deterministic property-based test with seed
{% set seed = 42 %}

[test.metadata]
name = "deterministic_property_test"
description = "Reproducible property-based test with seed {{ seed }}"

[services.app]
type = "generic_container"
plugin = "generic_container"
image = "app:latest"
env = { SEED = "{{ seed }}" }

# Same random values every run with fixed seed
{% for i in range(end=10) %}
[[steps]]
name = "step_{{ i }}"
command = ["test", "--seed", "{{ seed }}", "--iteration", "{{ i }}"]
# Seeded random values are reproducible
expected_output_regex = "result_{{ fake_uuid_seeded(seed=(seed + i)) }}"
{% endfor %}
```

### Example 5: Matrix Testing (Combinatorial)

**File**: `tests/matrix-test.clnrm.toml.tera`

```toml
[test.metadata]
name = "matrix_combinatorial_test"
description = "Test all combinations of versions and platforms"

{% set versions = ['1.0', '1.1', '2.0'] %}
{% set platforms = ['linux/amd64', 'linux/arm64'] %}
{% set languages = ['en', 'es', 'fr'] %}

# Generate service for each combination
{% for version in versions %}
{% for platform in platforms %}
[services.app_{{ version | replace(from='.', to='_') }}_{{ platform | replace(from='/', to='_') }}]
type = "generic_container"
plugin = "generic_container"
image = "app:{{ version }}"
env = { PLATFORM = "{{ platform }}" }
{% endfor %}
{% endfor %}

# Test each combination
{% for version in versions %}
{% for platform in platforms %}
{% for lang in languages %}
[[steps]]
name = "test_{{ version }}_{{ platform }}_{{ lang }}"
service = "app_{{ version | replace(from='.', to='_') }}_{{ platform | replace(from='/', to='_') }}"
command = ["app", "test", "--lang", "{{ lang }}"]
expected_exit_code = 0
{% endfor %}
{% endfor %}
{% endfor %}

[assertions]
total_combinations = {{ versions | length * platforms | length * languages | length }}
```

---

## Implementation Plan

### Phase 1: Dependencies & Foundation (1-2 days)

**Task 1.1**: Add Tera dependency to `Cargo.toml`

```toml
# Add to crates/clnrm-core/Cargo.toml [dependencies]
tera = "1.19"
base64 = "0.21"
sha2 = "0.10"
```

**Task 1.2**: Create module structure

```bash
# Create new modules
touch crates/clnrm-core/src/config/template.rs
touch crates/clnrm-core/src/config/fake_data.rs
```

**Task 1.3**: Update `config/mod.rs` to include new modules

```rust
// Add to crates/clnrm-core/src/config/mod.rs
pub mod template;
pub mod fake_data;
```

### Phase 2: Fake Data Generators (2-3 days)

**Task 2.1**: Implement all fake data generators in `fake_data.rs`

Priority order:
1. `fake_uuid()` - Most common use case
2. `fake_name()` - User testing
3. `fake_email()` - User testing
4. `fake_timestamp()` - Time-based tests
5. `random_int()`, `random_string()` - General purpose
6. `fake_ipv4()` - Network testing
7. `random_bool()`, `random_choice()` - Conditional logic

**Task 2.2**: Unit tests for all generators

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_uuid_format() {
        let uuid = fake_uuid();
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_random_int_range() {
        for _ in 0..100 {
            let val = random_int(10, 20);
            assert!(val >= 10 && val <= 20);
        }
    }
}
```

### Phase 3: Tera Integration (3-4 days)

**Task 3.1**: Implement `template::render_template()`

**Task 3.2**: Register custom functions

```rust
fn register_custom_functions(tera: &mut Tera) -> Result<()> {
    use tera::Function;

    // fake_uuid
    tera.register_function("fake_uuid",
        Box::new(|_args: &HashMap<String, tera::Value>| {
            Ok(tera::Value::String(fake_data::fake_uuid()))
        })
    );

    // fake_name
    tera.register_function("fake_name",
        Box::new(|_args: &HashMap<String, tera::Value>| {
            Ok(tera::Value::String(fake_data::fake_name()))
        })
    );

    // random_int
    tera.register_function("random_int",
        Box::new(|args: &HashMap<String, tera::Value>| {
            let min = args.get("min")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| tera::Error::msg("random_int requires 'min' argument"))?;
            let max = args.get("max")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| tera::Error::msg("random_int requires 'max' argument"))?;
            Ok(tera::Value::Number(fake_data::random_int(min, max).into()))
        })
    );

    // ... register remaining functions

    Ok(())
}
```

**Task 3.3**: Register custom filters

```rust
fn register_custom_filters(tera: &mut Tera) -> Result<()> {
    use tera::Filter;

    // sha256 filter
    tera.register_filter("sha256",
        Box::new(|value: &tera::Value, _args: &HashMap<String, tera::Value>| {
            let s = value.as_str()
                .ok_or_else(|| tera::Error::msg("sha256 requires string input"))?;
            Ok(tera::Value::String(fake_data::filter_sha256(s)))
        })
    );

    // base64 filter
    tera.register_filter("base64",
        Box::new(|value: &tera::Value, _args: &HashMap<String, tera::Value>| {
            let s = value.as_str()
                .ok_or_else(|| tera::Error::msg("base64 requires string input"))?;
            Ok(tera::Value::String(fake_data::filter_base64(s)))
        })
    );

    Ok(())
}
```

### Phase 4: Config Integration (2 days)

**Task 4.1**: Modify `load_config_from_file()` to detect and render templates

**Task 4.2**: Add file extension checking logic

**Task 4.3**: Error handling for template rendering failures

### Phase 5: Testing & Documentation (3-4 days)

**Task 5.1**: Unit tests for template rendering

```rust
#[test]
fn test_render_simple_template() {
    let template = r#"
[test.metadata]
name = "{{ fake_name() }}"
"#;
    let rendered = render_template(template).unwrap();
    assert!(rendered.contains("name = "));
}
```

**Task 5.2**: Integration tests with full TOML templates

**Task 5.3**: Property-based tests for generators

```rust
#[cfg(feature = "proptest")]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_int_stays_in_bounds(min in 0i64..1000, max in 1000i64..10000) {
            let val = random_int(min, max);
            assert!(val >= min && val <= max);
        }
    }
}
```

**Task 5.4**: Update documentation

- Add `/Users/sac/clnrm/docs/TEMPLATE_GUIDE.md`
- Update `docs/TOML_REFERENCE.md` with template section
- Add examples to `examples/templating/`

### Phase 6: CLI & Developer Experience (2 days)

**Task 6.1**: Add `clnrm template render` command

```bash
# Render template to stdout (debugging)
clnrm template render tests/my-test.clnrm.toml.tera

# Render and save
clnrm template render tests/my-test.clnrm.toml.tera --output rendered.toml
```

**Task 6.2**: Add template validation

```bash
# Validate template syntax without execution
clnrm template validate tests/my-test.clnrm.toml.tera
```

**Task 6.3**: Add debugging output

```bash
# Verbose rendering with debug info
clnrm template render --debug tests/my-test.clnrm.toml.tera
```

### Total Timeline: 13-17 days (2.5-3.5 weeks)

---

## Error Handling Strategy

### Error Types

Add new error variant to `ErrorKind` in `error.rs`:

```rust
pub enum ErrorKind {
    // ... existing variants

    /// Template rendering error
    TemplateError,
}
```

Add helper function:

```rust
impl CleanroomError {
    /// Create a template error
    pub fn template_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TemplateError, message)
    }
}
```

### Error Scenarios

#### 1. Template Syntax Error

```rust
// Template: {{ fake_uuid(  # Missing closing parenthesis
Err(CleanroomError::template_error(
    "Template syntax error at line 5: unclosed function call"
))
```

#### 2. Unknown Function

```rust
// Template: {{ unknown_function() }}
Err(CleanroomError::template_error(
    "Unknown template function 'unknown_function'. Available: fake_uuid, fake_name, ..."
))
```

#### 3. Invalid Function Arguments

```rust
// Template: {{ random_int(10) }}  # Missing 'max' argument
Err(CleanroomError::template_error(
    "random_int requires both 'min' and 'max' arguments"
))
```

#### 4. TOML Parse Error After Rendering

```rust
// Rendered TOML is invalid
Err(CleanroomError::config_error(
    "TOML parse error after template rendering: duplicate key 'name'"
))
```

### Error Context

All template errors include:
- Line number in template
- Offending template expression
- Suggested fix (when possible)
- Rendered output (on request via `--debug`)

Example:

```
Error: Template rendering failed
  ┌─ tests/load-test.clnrm.toml.tera:15:20
  │
15│ name = "{{ unknown_func() }}"
  │             ^^^^^^^^^^^^ unknown function
  │
  = help: Available functions: fake_uuid, fake_name, fake_email, random_int, random_string
  = note: Use `clnrm template --list-functions` to see all available functions
```

---

## Testing Strategy

### Unit Tests

**Location**: `crates/clnrm-core/src/config/fake_data.rs`, `template.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test each fake data generator
    #[test]
    fn test_fake_uuid_is_valid_uuid() {
        let uuid = fake_uuid();
        assert!(uuid::Uuid::parse_str(&uuid).is_ok());
    }

    #[test]
    fn test_fake_email_has_at_sign() {
        let email = fake_email();
        assert!(email.contains('@'));
        assert!(email.ends_with("@example.com"));
    }

    // Test random generators stay in bounds
    #[test]
    fn test_random_int_bounds() {
        for _ in 0..1000 {
            let val = random_int(5, 10);
            assert!(val >= 5 && val <= 10);
        }
    }

    // Test template rendering
    #[test]
    fn test_render_simple_function() {
        let template = "{{ fake_uuid() }}";
        let rendered = render_template(template).unwrap();
        assert!(uuid::Uuid::parse_str(&rendered).is_ok());
    }

    #[test]
    fn test_render_loop() {
        let template = r#"
{% for i in range(end=3) %}
step_{{ i }}
{% endfor %}
"#;
        let rendered = render_template(template).unwrap();
        assert!(rendered.contains("step_0"));
        assert!(rendered.contains("step_1"));
        assert!(rendered.contains("step_2"));
    }
}
```

### Integration Tests

**Location**: `crates/clnrm-core/tests/template_integration.rs`

```rust
#[tokio::test]
async fn test_load_template_toml_file() {
    // Create template file
    let template = r#"
[test.metadata]
name = "{{ fake_name() }}"

[[steps]]
name = "step_1"
command = ["echo", "{{ fake_uuid() }}"]
"#;

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), template).unwrap();

    // Load config (should render template)
    let config = load_config_from_file(temp_file.path()).unwrap();

    // Verify rendered values
    assert!(!config.test.metadata.name.is_empty());
    assert_eq!(config.steps.len(), 1);
}

#[tokio::test]
async fn test_property_based_template() {
    let template = r#"
[test.metadata]
name = "property_test"

{% for i in range(end=100) %}
[[steps]]
name = "step_{{ i }}"
command = ["echo", "{{ i }}"]
{% endfor %}
"#;

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), template).unwrap();

    let config = load_config_from_file(temp_file.path()).unwrap();

    // Verify 100 steps generated
    assert_eq!(config.steps.len(), 100);
}
```

### Property-Based Tests

**Location**: `crates/clnrm-core/tests/property_tests.rs`

```rust
#[cfg(feature = "proptest")]
mod template_property_tests {
    use proptest::prelude::*;
    use clnrm_core::config::fake_data::*;

    proptest! {
        #[test]
        fn random_int_never_panics(min in -1000i64..1000, max in -1000i64..1000) {
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
        fn fake_uuid_always_valid(seed in any::<u64>()) {
            let uuid_str = fake_uuid_seeded(seed);
            assert!(uuid::Uuid::parse_str(&uuid_str).is_ok());
        }
    }
}
```

### End-to-End Tests

**Location**: `examples/templating/e2e-template-test.clnrm.toml.tera`

Full working example that:
1. Generates 50 test scenarios
2. Uses all template functions
3. Validates execution with OTEL
4. Runs via `cargo run -- run examples/templating/e2e-template-test.clnrm.toml.tera`

---

## Backward Compatibility

### Compatibility Matrix

| File Type | Extension | Behavior | Backward Compatible |
|-----------|-----------|----------|---------------------|
| Static TOML | `.toml` | Parse directly, no template rendering | ✅ Yes - unchanged |
| Template TOML | `.toml.tera` | Render template, then parse | ✅ N/A (new feature) |
| Template TOML | `.tera` | Render template, then parse | ✅ N/A (new feature) |

### Migration Path

**Existing TOML files**: No changes needed. Continue working as-is.

**New template files**: Use `.tera` or `.toml.tera` extension.

**No breaking changes**: All existing `.clnrm.toml` files work without modification.

### Deprecation Policy

No deprecations. Both static and templated TOML supported indefinitely.

---

## Performance Considerations

### Template Rendering Cost

**Baseline**: Rendering empty template: ~0.1ms
**Small template** (10 functions): ~1ms
**Large template** (1000 iterations): ~50-100ms

**Optimization**: Template rendering happens once at config load time, not per test execution.

### Caching Strategy

```rust
// Optional: Cache rendered templates
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref TEMPLATE_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn render_template_cached(template: &str, cache_key: &str) -> Result<String> {
    let mut cache = TEMPLATE_CACHE.lock().unwrap();

    if let Some(cached) = cache.get(cache_key) {
        return Ok(cached.clone());
    }

    let rendered = render_template(template)?;
    cache.insert(cache_key.to_string(), rendered.clone());

    Ok(rendered)
}
```

### Memory Usage

**Large templates** (10,000 steps): ~5-10MB memory during rendering
**Post-rendering**: Memory freed after TOML parsing

**Mitigation**: Use streaming/chunked rendering for massive templates (future optimization).

---

## Security Considerations

### Template Injection Prevention

**Risk**: User-provided template strings could execute arbitrary code.

**Mitigation**:
1. **No `include` or `import`**: Disable Tera's file inclusion features
2. **No shell execution**: Functions only return strings/numbers
3. **Sandboxed functions**: All custom functions pure (no I/O, no network)
4. **No user input in templates**: Templates are source-controlled files

```rust
fn init_tera_engine() -> Result<Tera> {
    let mut tera = Tera::default();

    // Disable dangerous features
    tera.autoescape_on(vec![]);  // No HTML escaping needed for TOML

    // Only register safe functions
    register_custom_functions(&mut tera)?;

    Ok(tera)
}
```

### Determinism vs Security

**Random generation**: Uses `rand::thread_rng()` which is cryptographically secure but not deterministic.

**Seeded generation**: Use `fake_uuid_seeded(seed)` for reproducible tests.

**Recommendation**:
- CI/production: Use seeded functions for determinism
- Local development: Use non-seeded for variety

### Secrets in Templates

**Risk**: Hardcoded secrets in template files.

**Mitigation**:
1. **Lint warnings**: Detect patterns like `password = "..."`
2. **Environment variable injection**: `{{ env_var(name='SECRET_KEY') }}`
3. **Documentation**: Warn against committing secrets

```rust
pub fn env_var(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_|
        CleanroomError::template_error(format!("Environment variable '{}' not found", name))
    )
}
```

**Template usage**:
```toml
[services.db.env]
DB_PASSWORD = "{{ env_var(name='DB_PASSWORD') }}"
```

---

## Future Enhancements (Out of Scope for v1)

1. **Template Inheritance**: Extend Tera to support TOML-specific base templates
2. **Dynamic Context**: Pass runtime variables into templates
3. **Template Debugger**: Interactive step-through of template rendering
4. **Template Marketplace**: Share reusable template snippets
5. **Visual Template Builder**: GUI for creating templates
6. **Constraint Solvers**: Generate tests that satisfy complex predicates

---

## Acceptance Criteria

### Definition of Done for Implementation

- [ ] All fake data generators implemented and tested
- [ ] Tera integration complete with custom functions/filters
- [ ] `load_config_from_file()` modified to support template rendering
- [ ] File extension detection (`.tera`, `.toml.tera`) working
- [ ] All error scenarios handled with helpful messages
- [ ] Unit tests for all functions (>90% coverage)
- [ ] Integration tests with full TOML templates
- [ ] Property-based tests for random generators
- [ ] End-to-end test with 100+ generated scenarios
- [ ] Documentation: `TEMPLATE_GUIDE.md`, examples, API docs
- [ ] CLI commands: `template render`, `template validate`
- [ ] No breaking changes to existing TOML files
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes all tests
- [ ] Framework self-test validates templating feature

---

## Appendix A: Complete Function Registry

| Function | Arguments | Returns | Example |
|----------|-----------|---------|---------|
| `fake_uuid()` | None | String | `"550e8400-..."` |
| `fake_uuid_seeded(seed)` | `seed: u64` | String | `"550e8400-..."` |
| `fake_name()` | None | String | `"John Doe"` |
| `fake_email()` | None | String | `"test@example.com"` |
| `fake_timestamp()` | None | i64 | `1729123456` |
| `fake_timestamp_ms()` | None | i64 | `1729123456789` |
| `fake_ipv4()` | None | String | `"192.168.1.42"` |
| `random_int(min, max)` | `min: i64, max: i64` | i64 | `42` |
| `random_string(length)` | `length: usize` | String | `"a3kJ9z"` |
| `random_bool()` | None | bool | `true` |
| `random_choice(items)` | `items: Vec<String>` | String | `"option1"` |
| `property_range(start, end)` | `start: i64, end: i64` | Vec<i64> | `[0,1,2,3]` |
| `env_var(name)` | `name: String` | String | `"secret123"` |

| Filter | Input | Returns | Example |
|--------|-------|---------|---------|
| `upper` | String | String | `"HELLO"` |
| `lower` | String | String | `"hello"` |
| `sha256` | String | String | `"abc123..."` |
| `base64` | String | String | `"SGVsbG8="` |

---

## Appendix B: File Structure After Implementation

```
crates/clnrm-core/src/
├── config/
│   ├── mod.rs              # Modified: Add template rendering step
│   ├── template.rs         # NEW: Tera rendering engine
│   └── fake_data.rs        # NEW: Fake data generators
├── error.rs                # Modified: Add TemplateError variant
└── lib.rs                  # Modified: Export template module

crates/clnrm-core/tests/
├── template_integration.rs # NEW: Integration tests
└── property_tests.rs       # Modified: Add template property tests

examples/
└── templating/
    ├── load-test.clnrm.toml.tera
    ├── chaos-random.clnrm.toml.tera
    ├── db-property-test.clnrm.toml.tera
    └── deterministic-property.clnrm.toml.tera

docs/
├── TEMPLATE_GUIDE.md       # NEW: Complete templating guide
├── TOML_REFERENCE.md       # Modified: Add templating section
└── architecture/
    └── tera-templating-architecture.md  # THIS DOCUMENT

Cargo.toml                  # Modified: Add tera dependency
```

---

## Conclusion

This architecture provides a complete, production-ready design for Tera templating in CLNRM. The implementation:

1. **Maintains backward compatibility** - existing TOML files unchanged
2. **Integrates cleanly** - minimal changes to config parsing pipeline
3. **Provides powerful features** - property-based testing, fake data, loops
4. **Handles errors gracefully** - clear error messages with context
5. **Follows CLNRM standards** - no `.unwrap()`, proper `Result<T>` usage
6. **Thoroughly tested** - unit, integration, property-based, and E2E tests

**Next Steps**:
1. Review this architecture document
2. Get stakeholder approval
3. Begin implementation following the phased plan
4. Iterate based on testing feedback

**Estimated Completion**: 2.5-3.5 weeks for full implementation and testing.

---

**Document Status**: ✅ READY FOR REVIEW
**Implementation Status**: 🔴 NOT STARTED - DESIGN ONLY
