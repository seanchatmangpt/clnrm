# clnrm Tera Template Examples

This directory contains example templates demonstrating various Tera features in clnrm v0.6.0.

## Examples Overview

### 1. simple-variables.clnrm.toml

**Purpose**: Demonstrates basic variable substitution

**Features**:
- `[template.vars]` section
- `{{ vars.* }}` substitution
- Environment resource configuration

**Use Case**: Simple tests with reusable configuration values

**Run**:
```bash
clnrm run examples/templates/simple-variables.clnrm.toml
```

---

### 2. matrix-expansion.clnrm.toml

**Purpose**: Demonstrates matrix-based scenario generation

**Features**:
- `[template.matrix]` section
- `{% for %}` loops
- Dynamic scenario generation
- Conditional endpoint configuration

**Use Case**: Testing with multiple OTEL exporters or configurations

**Run**:
```bash
clnrm run examples/templates/matrix-expansion.clnrm.toml
```

**Note**: This generates 3 scenarios (stdout, otlp, jaeger) from a single template.

---

### 3. multi-environment.clnrm.toml

**Purpose**: Demonstrates environment-specific configurations

**Features**:
- `{% if %}` conditionals
- `env()` function
- Environment-based OTEL configuration
- Conditional determinism and reporting

**Use Case**: Different behavior for dev/staging/prod environments

**Run**:
```bash
# Development
TEST_ENV=dev clnrm run examples/templates/multi-environment.clnrm.toml

# Staging
TEST_ENV=staging clnrm run examples/templates/multi-environment.clnrm.toml

# Production
TEST_ENV=prod clnrm run examples/templates/multi-environment.clnrm.toml
```

---

### 4. service-mesh.clnrm.toml

**Purpose**: Demonstrates complex multi-service orchestration

**Features**:
- Multiple service definitions via loops
- Service-specific environment variables
- Conditional service enablement
- Per-service span expectations

**Use Case**: Microservices integration testing

**Run**:
```bash
clnrm run examples/templates/service-mesh.clnrm.toml
```

**Note**: Requires Docker/Podman to pull nginx, python, postgres, redis images.

---

### 5. ci-integration.clnrm.toml

**Purpose**: Demonstrates CI/CD pipeline integration

**Features**:
- `env()` function for CI environment variables
- `sha256()` function for unique test IDs
- `now_rfc3339()` function for timestamps
- Conditional determinism (CI vs local)
- Conditional reporting

**Use Case**: Running tests in CI/CD pipelines (GitHub Actions, GitLab CI, etc.)

**Run**:
```bash
# Local run
clnrm run examples/templates/ci-integration.clnrm.toml

# CI simulation
CI=true CI_JOB_ID=12345 GIT_COMMIT=abc123 clnrm run examples/templates/ci-integration.clnrm.toml
```

---

### 6. macros-and-includes.clnrm.toml

**Purpose**: Demonstrates reusable template blocks

**Features**:
- `{% macro %}` definitions
- Macro invocations with parameters
- Inline TOML generation
- Reusable event patterns

**Use Case**: DRY (Don't Repeat Yourself) template design

**Run**:
```bash
clnrm run examples/templates/macros-and-includes.clnrm.toml
```

---

### 7. advanced-validators.clnrm.toml

**Purpose**: Demonstrates new v0.6.0 validators

**Features**:
- `[expect.order]` - Temporal ordering validation
- `[expect.status]` - Span status validation
- Per-pattern status requirements
- Lifecycle phase validation

**Use Case**: Strict temporal and status validation

**Run**:
```bash
clnrm run examples/templates/advanced-validators.clnrm.toml
```

---

## Template Rendering

You can preview rendered templates before execution:

```bash
# Render template to stdout (future feature)
clnrm template render examples/templates/matrix-expansion.clnrm.toml

# Render and save to file (future feature)
clnrm template render examples/templates/matrix-expansion.clnrm.toml --output /tmp/rendered.toml
```

---

## Creating Your Own Templates

### Step 1: Start with a Working TOML

Start with a non-template `.clnrm.toml` file that works.

### Step 2: Identify Repetition

Look for repeated values that could be variables:
```toml
# Before
[[scenario]]
name = "test_1"
run = "clnrm run --otel-exporter otlp"

[[scenario]]
name = "test_2"
run = "clnrm run --otel-exporter otlp"
```

### Step 3: Extract to Variables

```toml
# After
[template.vars]
exporter = "otlp"

[[scenario]]
name = "test_1"
run = "clnrm run --otel-exporter {{ vars.exporter }}"

[[scenario]]
name = "test_2"
run = "clnrm run --otel-exporter {{ vars.exporter }}"
```

### Step 4: Use Loops for Similar Blocks

```toml
[template.matrix]
tests = [1, 2]

{% for i in matrix.tests %}
[[scenario]]
name = "test_{{ i }}"
run = "clnrm run --otel-exporter {{ vars.exporter }}"
{% endfor %}
```

### Step 5: Add Conditionals

```toml
[template.vars]
enable_reporting = true

{% if vars.enable_reporting %}
[report]
json = "results/report.json"
{% endif %}
```

---

## Best Practices

1. **Start Simple**: Use variables before loops and conditionals
2. **Test Incrementally**: Validate templates after each change
3. **Comment Logic**: Use `{# comments #}` to document template logic
4. **Use Macros**: Extract repeated patterns into macros
5. **Validate Output**: Render templates to verify generated TOML

---

## Common Pitfalls

### 1. Mismatched Braces

```toml
# Wrong
name = "{{ vars.name }"

# Correct
name = "{{ vars.name }}"
```

### 2. Undefined Variables

```toml
# Will fail
exporter = "{{ vars.undefined }}"

# Add to template.vars first
[template.vars]
undefined = "value"
```

### 3. TOML Syntax in Template

```toml
# Wrong (invalid TOML after rendering)
{% for i in range(end=3) %}
[[expect.span]]
name = span_{{ i }}  # Missing quotes!
{% endfor %}

# Correct
{% for i in range(end=3) %}
[[expect.span]]
name = "span_{{ i }}"
{% endfor %}
```

---

## Further Reading

- [Tera Template Guide](../../docs/TERA_TEMPLATE_GUIDE.md)
- [TOML Reference](../../docs/TOML_REFERENCE.md)
- [Tera Documentation](https://keats.github.io/tera/docs/)

---

## Contributing Examples

Have a useful template pattern? Contribute by:

1. Creating a new `.clnrm.toml` template in this directory
2. Documenting it in this README
3. Opening a pull request

Examples should be:
- **Self-contained**: Run without external dependencies (or document them)
- **Well-commented**: Explain template logic with `{# comments #}`
- **Demonstrative**: Show a specific feature or pattern clearly
