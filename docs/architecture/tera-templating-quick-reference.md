# Tera Templating Quick Reference

**Companion to**: `tera-templating-architecture.md`
**Purpose**: Quick lookup for template syntax and common patterns
**Status**: DESIGN PHASE

---

## Quick Start

### 1. Create a Template File

```bash
# File: tests/my-test.clnrm.toml.tera
```

```toml
[test.metadata]
name = "my_property_test"

# Generate 100 test scenarios
{% for i in range(end=100) %}
[[steps]]
name = "step_{{ i }}"
command = ["echo", "{{ fake_uuid() }}"]
{% endfor %}
```

### 2. Run the Test

```bash
clnrm run tests/my-test.clnrm.toml.tera
```

The framework automatically detects the `.tera` extension and renders the template before parsing.

---

## Function Cheat Sheet

### Fake Data Generators

```toml
# UUIDs
id = "{{ fake_uuid() }}"                    # Random: 550e8400-e29b-41d4-a716-446655440000
id = "{{ fake_uuid_seeded(seed=42) }}"     # Deterministic

# Names & Emails
name = "{{ fake_name() }}"                  # John Doe
email = "{{ fake_email() }}"                # test_123@example.com

# Timestamps
created = {{ fake_timestamp() }}            # 1729123456 (Unix seconds)
created_ms = {{ fake_timestamp_ms() }}      # 1729123456789 (milliseconds)

# IP Addresses
ip = "{{ fake_ipv4() }}"                    # 192.168.1.42
```

### Random Generators

```toml
# Integers
port = {{ random_int(min=8000, max=9000) }}          # Random port
count = {{ random_int(min=1, max=100) }}             # Random count

# Strings
token = "{{ random_string(length=32) }}"             # Random alphanumeric
password = "{{ random_string(length=16) }}"          # Random password

# Booleans
enabled = {{ random_bool() }}                        # true or false

# Choice from list
log_level = "{{ random_choice(items=['debug', 'info', 'warn', 'error']) }}"
```

### Loops & Ranges

```toml
# Simple range
{% for i in range(end=10) %}
step_{{ i }}                                # Generates step_0 to step_9
{% endfor %}

# Range with start
{% for i in range(start=1, end=5) %}
step_{{ i }}                                # Generates step_1 to step_5
{% endfor %}

# Iterate over list
{% set databases = ['postgres', 'mysql', 'mongodb'] %}
{% for db in databases %}
[services.{{ db }}]
type = "database"
image = "{{ db }}:latest"
{% endfor %}
```

### Filters

```toml
# String transformations
name = "{{ 'hello world' | upper }}"        # HELLO WORLD
name = "{{ 'HELLO WORLD' | lower }}"        # hello world

# Hashing
password_hash = "{{ 'mypassword' | sha256 }}"
token = "{{ 'secret' | base64 }}"
```

### Environment Variables

```toml
[services.db.env]
DB_PASSWORD = "{{ env_var(name='DB_PASSWORD') }}"
API_KEY = "{{ env_var(name='API_KEY') }}"
```

---

## Common Patterns

### Pattern 1: Load Testing (N Concurrent Requests)

```toml
[test.metadata]
name = "load_test"

[services.api]
type = "generic_container"
image = "my-api:latest"

{% for i in range(end=1000) %}
[[steps]]
name = "request_{{ i }}"
command = ["curl", "http://api:8080/endpoint"]
expected_exit_code = 0
{% endfor %}
```

### Pattern 2: Property-Based User Creation

```toml
{% for i in range(end=100) %}
[[steps]]
name = "create_user_{{ i }}"
command = ["api", "create-user",
           "--name", "{{ fake_name() }}",
           "--email", "{{ fake_email() }}",
           "--id", "{{ fake_uuid() }}"]
expected_exit_code = 0
{% endfor %}
```

### Pattern 3: Matrix Testing (All Combinations)

```toml
{% set versions = ['1.0', '2.0', '3.0'] %}
{% set platforms = ['linux', 'windows', 'macos'] %}

{% for version in versions %}
{% for platform in platforms %}
[[steps]]
name = "test_{{ version }}_{{ platform }}"
command = ["test", "--version", "{{ version }}", "--platform", "{{ platform }}"]
{% endfor %}
{% endfor %}
```

### Pattern 4: Conditional Logic

```toml
{% set enable_auth = random_bool() %}

[services.app]
type = "generic_container"
image = "app:latest"

{% if enable_auth %}
[services.app.env]
AUTH_ENABLED = "true"
{% else %}
[services.app.env]
AUTH_ENABLED = "false"
{% endif %}
```

### Pattern 5: Deterministic Property Tests

```toml
{% set seed = 42 %}

[test.metadata]
name = "deterministic_test"
description = "Always generates same values with seed {{ seed }}"

{% for i in range(end=50) %}
[[steps]]
name = "step_{{ i }}"
command = ["test", "--id", "{{ fake_uuid_seeded(seed=(seed + i)) }}"]
expected_output_regex = "success"
{% endfor %}
```

### Pattern 6: Chaos Engineering

```toml
{% set chaos_actions = ['kill_process', 'network_delay', 'cpu_stress', 'disk_full'] %}

{% for i in range(end=20) %}
[[steps]]
name = "chaos_{{ i }}"
service = "chaos_engine"
command = ["chaos", "{{ random_choice(items=chaos_actions) }}",
           "--duration", "{{ random_int(min=5, max=30) }}s"]
{% endfor %}
```

---

## Template Debugging

### Preview Rendered Template

```bash
# Render template to stdout (see what gets generated)
clnrm template render tests/my-test.clnrm.toml.tera
```

### Validate Template Syntax

```bash
# Check for syntax errors without executing
clnrm template validate tests/my-test.clnrm.toml.tera
```

### Debug Output

```bash
# Verbose rendering with debug information
clnrm template render --debug tests/my-test.clnrm.toml.tera
```

---

## Error Messages

### Syntax Error

```
Error: Template rendering failed
  ┌─ tests/test.clnrm.toml.tera:15:20
  │
15│ name = "{{ fake_uuid( }}"
  │                     ^ unexpected character
  │
  = help: Check closing parentheses and quotes
```

### Unknown Function

```
Error: Template rendering failed
  ┌─ tests/test.clnrm.toml.tera:10:15
  │
10│ id = "{{ unknown_func() }}"
  │           ^^^^^^^^^^^^ unknown function
  │
  = help: Available functions: fake_uuid, fake_name, fake_email, random_int, ...
  = note: Use `clnrm template --list-functions` for complete list
```

### Missing Argument

```
Error: Template rendering failed
  ┌─ tests/test.clnrm.toml.tera:12:20
  │
12│ port = {{ random_int(min=8000) }}
  │            ^^^^^^^^^^ missing required argument 'max'
  │
  = help: random_int requires both 'min' and 'max' arguments
  = example: {{ random_int(min=1000, max=2000) }}
```

---

## Performance Tips

### 1. Use Static Values When Possible

```toml
# BAD: Generates new UUID every time (slow)
{% for i in range(end=1000) %}
base_id = "{{ fake_uuid() }}"
{% endfor %}

# GOOD: Generate once, reuse (fast)
{% set base_id = fake_uuid() %}
{% for i in range(end=1000) %}
id = "{{ base_id }}_{{ i }}"
{% endfor %}
```

### 2. Limit Loop Iterations

```toml
# For development: Use small iteration count
{% for i in range(end=10) %}
...
{% endfor %}

# For production: Scale up
{% for i in range(end=10000) %}
...
{% endfor %}
```

### 3. Conditional Compilation

```toml
{% if env_var(name='CI') == 'true' %}
  {% set iterations = 1000 %}
{% else %}
  {% set iterations = 10 %}
{% endif %}

{% for i in range(end=iterations) %}
...
{% endfor %}
```

---

## Complete Example: E2E Property-Based Test

**File**: `tests/e2e-property-test.clnrm.toml.tera`

```toml
# E2E property-based test with multiple services and random data
{% set seed = 42 %}
{% set num_users = 100 %}
{% set num_requests_per_user = 10 %}

[test.metadata]
name = "e2e_property_based_test"
description = "Property-based E2E test with {{ num_users }} users and {{ num_requests_per_user }} requests each"

# Database service
[services.db]
type = "generic_container"
plugin = "generic_container"
image = "postgres:15"
env = { POSTGRES_PASSWORD = "{{ env_var(name='DB_PASSWORD') }}" }

# API service
[services.api]
type = "generic_container"
plugin = "generic_container"
image = "my-api:latest"
env = { DATABASE_URL = "postgresql://postgres@db:5432/testdb" }

# Step 1: Initialize database
[[steps]]
name = "init_database"
service = "db"
command = ["psql", "-c", "CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR, email VARCHAR);"]
expected_exit_code = 0

# Step 2: Create users
{% for i in range(end=num_users) %}
[[steps]]
name = "create_user_{{ i }}"
service = "api"
command = ["curl", "-X", "POST", "http://api:8080/users",
           "-H", "Content-Type: application/json",
           "-d", '{"id":"{{ fake_uuid_seeded(seed=(seed + i)) }}","name":"{{ fake_name() }}","email":"{{ fake_email() }}"}']
expected_exit_code = 0
expected_output_regex = "201|200"
{% endfor %}

# Step 3: Random requests for each user
{% for user_id in range(end=num_users) %}
{% for req_id in range(end=num_requests_per_user) %}
[[steps]]
name = "request_user{{ user_id }}_req{{ req_id }}"
service = "api"
command = ["curl", "http://api:8080/users/{{ fake_uuid_seeded(seed=(seed + user_id)) }}"]
expected_exit_code = 0
expected_output_regex = "name"
{% endfor %}
{% endfor %}

# Step 4: Cleanup
[[steps]]
name = "cleanup_database"
service = "db"
command = ["psql", "-c", "DROP TABLE users;"]
expected_exit_code = 0

[assertions]
total_users_created = {{ num_users }}
total_requests = {{ num_users * num_requests_per_user }}
all_requests_successful = true

# OTEL validation
[otel_validation]
enabled = true
validate_spans = true

[otel_validation.expect_counts]
spans_total = { gte = {{ num_users + num_users * num_requests_per_user }} }
errors_total = { eq = 0 }
```

**Run it**:

```bash
clnrm run tests/e2e-property-test.clnrm.toml.tera
```

**Expected output**:
- Creates 100 users with deterministic UUIDs
- Executes 1,000 requests (100 users × 10 requests)
- Validates all operations via OTEL spans
- Total execution time: ~30-60 seconds

---

## Migration Guide

### Converting Existing TOML to Template

**Before** (`tests/my-test.clnrm.toml`):

```toml
[test.metadata]
name = "manual_test"

[[steps]]
name = "step_1"
command = ["echo", "test"]

[[steps]]
name = "step_2"
command = ["echo", "test"]

[[steps]]
name = "step_3"
command = ["echo", "test"]
```

**After** (`tests/my-test.clnrm.toml.tera`):

```toml
[test.metadata]
name = "automated_test"

{% for i in range(start=1, end=3) %}
[[steps]]
name = "step_{{ i }}"
command = ["echo", "test"]
{% endfor %}
```

**Benefits**:
- Reduced duplication: 12 lines → 8 lines
- Easier to scale: Change `end=3` to `end=100`
- Parametric: Add `{{ fake_uuid() }}` for unique IDs

---

## Best Practices

### 1. Use Descriptive Variable Names

```toml
# GOOD
{% set num_load_test_iterations = 1000 %}
{% set api_base_url = "http://api:8080" %}

# BAD
{% set n = 1000 %}
{% set url = "http://api:8080" %}
```

### 2. Comment Template Logic

```toml
# Generate load test with random user data
{% for i in range(end=100) %}
  # Each user gets deterministic UUID for traceability
  id = "{{ fake_uuid_seeded(seed=(42 + i)) }}"
{% endfor %}
```

### 3. Separate Template Configuration

```toml
# Configuration section at top
{% set num_users = 100 %}
{% set seed = 42 %}
{% set enable_chaos = false %}

# Template logic below
[test.metadata]
name = "test_with_{{ num_users }}_users"
...
```

### 4. Use Seeded Functions for CI

```toml
# Development: Random values
{% if env_var(name='CI') != 'true' %}
id = "{{ fake_uuid() }}"
{% else %}
# CI: Deterministic values
id = "{{ fake_uuid_seeded(seed=42) }}"
{% endif %}
```

### 5. Validate Generated TOML

```bash
# Always preview before running
clnrm template render tests/my-test.clnrm.toml.tera | clnrm validate -
```

---

## Advanced Techniques

### Macro Definitions

```toml
{% macro create_user_step(id, name, email) %}
[[steps]]
name = "create_user_{{ id }}"
command = ["api", "create-user",
           "--name", "{{ name }}",
           "--email", "{{ email }}"]
expected_exit_code = 0
{% endmacro %}

# Use macro
{% for i in range(end=10) %}
{{ create_user_step(id=i, name=fake_name(), email=fake_email()) }}
{% endfor %}
```

### Nested Loops

```toml
{% set regions = ['us-east', 'us-west', 'eu-central'] %}
{% set services = ['api', 'db', 'cache'] %}

{% for region in regions %}
{% for service in services %}
[services.{{ service }}_{{ region }}]
type = "generic_container"
image = "{{ service }}:latest"
env = { REGION = "{{ region }}" }
{% endfor %}
{% endfor %}
```

### Conditional Services

```toml
{% set enable_monitoring = true %}

{% if enable_monitoring %}
[services.prometheus]
type = "generic_container"
image = "prometheus:latest"

[services.grafana]
type = "generic_container"
image = "grafana:latest"
{% endif %}
```

---

## Troubleshooting

### Problem: Template renders but TOML is invalid

**Solution**: Use `clnrm template render` to inspect output

```bash
clnrm template render tests/my-test.clnrm.toml.tera > /tmp/rendered.toml
clnrm validate /tmp/rendered.toml
```

### Problem: Random values change every run

**Solution**: Use seeded functions with fixed seed

```toml
{% set seed = 42 %}
id = "{{ fake_uuid_seeded(seed=seed) }}"
```

### Problem: Too many test scenarios generated

**Solution**: Use environment variable to control iterations

```toml
{% set max_iterations = env_var(name='MAX_ITERATIONS') | default(value='10') | int %}
{% for i in range(end=max_iterations) %}
...
{% endfor %}
```

---

## CLI Reference

### Template Commands (After Implementation)

```bash
# Render template to stdout
clnrm template render <file.tera>

# Render and save
clnrm template render <file.tera> --output <output.toml>

# Validate template syntax
clnrm template validate <file.tera>

# List available functions
clnrm template --list-functions

# Debug rendering
clnrm template render --debug <file.tera>

# Render with custom seed
clnrm template render --seed 42 <file.tera>
```

### Standard Test Execution

```bash
# Run template file (auto-renders)
clnrm run tests/my-test.clnrm.toml.tera

# Run static TOML (no rendering)
clnrm run tests/my-test.clnrm.toml
```

---

## Implementation Status

**Current Status**: 🔴 NOT IMPLEMENTED (Design Phase)

**Tracking Issue**: TBD

**Estimated Completion**: 2.5-3.5 weeks

**Dependencies**:
- `tera = "1.19"`
- `base64 = "0.21"`
- `sha2 = "0.10"`

---

## Related Documentation

- **Full Architecture**: `/Users/sac/clnrm/docs/architecture/tera-templating-architecture.md`
- **TOML Reference**: `/Users/sac/clnrm/docs/TOML_REFERENCE.md`
- **Property Testing**: `/Users/sac/clnrm/docs/testing/property-testing-guide.md`

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
**Status**: ✅ READY FOR REVIEW
