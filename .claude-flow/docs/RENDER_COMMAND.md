# clnrm render - Template Rendering Command

## Overview

The `clnrm render` command renders Tera templates with variable substitution, enabling dynamic test configuration generation.

## Syntax

```bash
clnrm render [OPTIONS] <TEMPLATE>
```

## Arguments

- `<TEMPLATE>` - Path to the Tera template file (.toml.tera)

## Options

- `--map <KEY=VALUE>` - Variable mapping in key=value format (can be used multiple times)
- `-o, --output <FILE>` - Output file path (default: stdout)
- `--show-vars` - Display resolved variables before rendering

## Variable Resolution Precedence

The template system resolves variables using the following precedence (highest to lowest):

1. **User-provided variables** - Variables passed via `--map` flags
2. **Environment variables** - System environment variables
3. **Default values** - Built-in defaults (e.g., `env=ci`, `svc=default`)

## Examples

### Basic Template Rendering

Render a template with variable substitution:

```bash
clnrm render template.toml.tera --map svc=myapp --map env=prod
```

**Template (`template.toml.tera`):**
```toml
[test.metadata]
name = "{{ svc }}_integration_test"
description = "Test for {{ svc }} in {{ env }}"

[service.{{ svc }}]
plugin = "generic_container"
image = "{{ svc }}:latest"
```

**Output:**
```toml
[test.metadata]
name = "myapp_integration_test"
description = "Test for myapp in prod"

[service.myapp]
plugin = "generic_container"
image = "myapp:latest"
```

### Render to File

Save rendered output to a file:

```bash
clnrm render template.toml.tera \
  --map svc=api \
  --map env=staging \
  -o output.toml
```

### Show Resolved Variables

Display variables before rendering (useful for debugging):

```bash
clnrm render template.toml.tera \
  --map svc=web \
  --map env=dev \
  --show-vars
```

**Output:**
```
INFO  📋 Resolved variables:
INFO    svc = "web"
INFO    env = "dev"

[test.metadata]
name = "web_integration_test"
...
```

### Multiple Variables

Pass multiple variables in a single command:

```bash
clnrm render template.toml.tera \
  --map svc=payment \
  --map env=production \
  --map region=us-east-1 \
  --map version=1.2.3
```

### Using Environment Variables

Template variables can reference environment variables:

```bash
export APP_NAME=myapp
export DEPLOY_ENV=staging

clnrm render template.toml.tera --map svc=$APP_NAME --map env=$DEPLOY_ENV
```

## Template Syntax

The render command uses [Tera](https://tera.netlify.app/) template syntax.

### Variable Substitution

```toml
name = "{{ variable_name }}"
```

### Conditionals

```toml
{% if env == "production" %}
replicas = 3
{% else %}
replicas = 1
{% endif %}
```

### Loops

```toml
{% for port in ports %}
expose = {{ port }}
{% endfor %}
```

### Filters

```toml
name = "{{ svc | upper }}_test"
path = "{{ file_path | replace(from="/", to="-") }}"
```

### Comments

```toml
{# This is a comment and won't appear in output #}
```

## Macro Library

The render command includes a built-in macro library for common TOML patterns. Import macros with:

```toml
{% import "_macros.toml.tera" as m %}
```

### Available Macros

#### `m::service(name, image, args=[], env={})`

Generate service configuration:

```toml
{% import "_macros.toml.tera" as m %}
{{ m::service("postgres", "postgres:15", env={"POSTGRES_PASSWORD": "test"}) }}
```

**Generates:**
```toml
[service.postgres]
plugin = "generic_container"
image = "postgres:15"
env.POSTGRES_PASSWORD = "test"
```

#### `m::scenario(name, service, command, expect_success=true)`

Generate test scenario:

```toml
{{ m::scenario("health_check", "api", "curl localhost:8080/health") }}
```

**Generates:**
```toml
[[scenario]]
name = "health_check"
service = "api"
run = "curl localhost:8080/health"
expect_success = true
```

#### `m::span(name, parent=None, attrs={})`

Generate OpenTelemetry span expectation:

```toml
{{ m::span("http.request", parent="test.root", attrs={"http.method": "GET"}) }}
```

**Generates:**
```toml
[[expect.span]]
name = "http.request"
parent = "test.root"
attrs.all = { "http.method" = "GET" }
```

## Error Handling

### Invalid Mapping Format

```bash
clnrm render template.toml.tera --map invalid_no_equals
```

**Error:**
```
ERROR Command failed: ValidationError: Invalid variable mapping: 'invalid_no_equals' (expected key=value format)
```

**Solution:** Use proper `key=value` format:
```bash
clnrm render template.toml.tera --map key=value
```

### Missing Template File

```bash
clnrm render nonexistent.toml.tera --map svc=test
```

**Error:**
```
ERROR Command failed: ConfigError: Failed to read template file: No such file or directory
```

**Solution:** Verify the template file path exists.

### Template Syntax Error

**Template with error:**
```toml
name = "{{ unclosed"
```

**Error:**
```
ERROR Command failed: TemplateError: Template rendering failed in 'template.toml.tera': ...
```

**Solution:** Fix template syntax errors (missing `}}`, incorrect conditionals, etc.)

## Use Cases

### 1. Dynamic Test Generation

Generate test configurations for multiple environments:

```bash
for env in dev staging prod; do
  clnrm render test.toml.tera \
    --map env=$env \
    -o "tests/test-$env.toml"
done
```

### 2. Matrix Testing

Generate test combinations:

```bash
for svc in api worker scheduler; do
  for env in dev prod; do
    clnrm render matrix.toml.tera \
      --map svc=$svc \
      --map env=$env \
      -o "tests/${svc}-${env}.toml"
  done
done
```

### 3. CI/CD Pipeline Integration

Generate environment-specific tests in CI:

```yaml
# .github/workflows/test.yml
- name: Generate test configuration
  run: |
    clnrm render tests/template.toml.tera \
      --map env=${{ matrix.environment }} \
      --map version=${{ github.sha }} \
      -o tests/generated.toml

- name: Run tests
  run: clnrm run tests/generated.toml
```

### 4. Configuration Validation

Preview rendered output before saving:

```bash
clnrm render template.toml.tera --map svc=api --map env=prod | less
```

### 5. Debugging Templates

Use `--show-vars` to verify variable resolution:

```bash
clnrm render template.toml.tera \
  --map svc=test \
  --show-vars \
  | clnrm validate -
```

## Best Practices

1. **Use Descriptive Variable Names**
   ```bash
   # Good
   --map service_name=api --map deployment_env=production

   # Avoid
   --map s=a --map e=p
   ```

2. **Validate Rendered Output**
   ```bash
   clnrm render template.toml.tera --map svc=api -o output.toml
   clnrm validate output.toml
   ```

3. **Use Templates for Reusable Patterns**
   - Create template libraries for common service configurations
   - Share templates across team members
   - Version control templates alongside tests

4. **Document Required Variables**
   Add comments to templates listing required variables:
   ```toml
   {# Required variables: svc, env, region #}
   [test.metadata]
   name = "{{ svc }}_test"
   ```

5. **Test Templates**
   Create test scripts to verify template rendering:
   ```bash
   #!/bin/bash
   clnrm render template.toml.tera --map svc=test --map env=ci > output.toml
   clnrm validate output.toml
   ```

## Integration with Other Commands

### With `clnrm run`

Render and execute in one pipeline:

```bash
clnrm render template.toml.tera --map svc=api -o test.toml && \
clnrm run test.toml
```

### With `clnrm validate`

Validate rendered templates:

```bash
clnrm render template.toml.tera --map svc=api | clnrm validate -
```

### With `clnrm fmt`

Format rendered output:

```bash
clnrm render template.toml.tera --map svc=api -o test.toml && \
clnrm fmt test.toml
```

## See Also

- [Template System Documentation](TEMPLATE_SYSTEM.md)
- [TOML Reference](TOML_REFERENCE.md)
- [CLI Guide](CLI_GUIDE.md)
- [Tera Template Engine](https://tera.netlify.app/)

## Version History

- **v0.7.0** - Initial implementation of `clnrm render` command
- **v1.0.0** - Added `--show-vars` flag and enhanced error messages
