# clnrm v1.5.1: 80/20 Tera Template Support Plan

**Version**: 1.5.1
**Goal**: Implement the 20% of Tera features that handle 80% of template use cases
**Status**: Planning

---

## The 80/20 Analysis

### ✅ The Critical 20% (Must Implement)

These 5 features cover 80% of real-world template needs:

1. **Variable Substitution** ✅ (Already Working!)
   ```toml
   {{ vars.service_name }}
   {{ vars.port }}
   ```

2. **Simple Conditionals** 🎯 (Priority 1)
   ```toml
   {% if vars.env == "prod" %}
   debug = false
   {% else %}
   debug = true
   {% endif %}
   ```

3. **Simple For Loops** 🎯 (Priority 2)
   ```toml
   {% for service in vars.services %}
   [services.{{ service }}]
   type = "container"
   {% endfor %}
   ```

4. **Environment Variable Access** 🎯 (Priority 3)
   ```toml
   endpoint = "{{ env.OTEL_ENDPOINT | default(value='http://localhost:4318') }}"
   ```

5. **Basic String Filters** 🎯 (Priority 4)
   ```toml
   name = "{{ vars.service | upper }}"
   image = "{{ vars.repo | lower }}:{{ vars.tag }}"
   ```

### ❌ Outside the 80/20 (Not Implementing)

Complex features with low ROI:
- `{% macro %}` definitions
- `{% extends %}` template inheritance
- Custom Tera functions
- Deeply nested conditionals
- Complex filter chains
- `{% include %}` statements
- `{% block %}` definitions

---

## Implementation Strategy

### Phase 1: Full Template Preprocessing (Week 1)

**Goal**: Process Tera templates BEFORE TOML parsing

**Current Problem**:
```rust
// config/loader.rs currently does this:
fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;

    // Extract [vars] section
    let vars = extract_vars_section(&content)?;

    // Render simple {{}} substitutions
    let rendered = render_template(&content, vars)?;

    // Parse TOML
    let config: TestConfig = toml::from_str(&rendered)?;
    // ❌ Problem: {% if %}, {% for %} are still in the TOML!
}
```

**New Approach**:
```rust
fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;

    // Extract ALL template sections
    let template_context = extract_template_context(&content)?;
    // Extracts: [vars], [template.matrix], [template.env_defaults]

    // FULL Tera preprocessing
    let fully_rendered = preprocess_template(&content, template_context)?;
    // Processes: {% if %}, {% for %}, {% macro %}, {{ }}

    // Parse clean TOML
    let config: TestConfig = toml::from_str(&fully_rendered)?;
    // ✅ All template syntax processed!
}
```

### Phase 2: Conditional Support (Week 1)

**Implementation**:
```rust
// In clnrm-template crate
pub fn preprocess_template(content: &str, context: TemplateContext) -> Result<String> {
    let mut tera = Tera::default();

    // Add the template
    tera.add_raw_template("config", content)?;

    // Build Tera context with all variables
    let tera_context = build_tera_context(context)?;

    // Render with full Tera processing
    let rendered = tera.render("config", &tera_context)?;

    Ok(rendered)
}
```

**Example**:
```toml
[vars]
env = "prod"

{% if vars.env == "prod" %}
[otel]
exporter = "otlp"
endpoint = "https://prod.example.com"
{% else %}
[otel]
exporter = "stdout"
endpoint = "http://localhost:4318"
{% endif %}
```

**Renders to**:
```toml
[vars]
env = "prod"

[otel]
exporter = "otlp"
endpoint = "https://prod.example.com"
```

### Phase 3: Loop Support (Week 2)

**Implementation**:
```rust
fn extract_template_context(content: &str) -> Result<TemplateContext> {
    let mut context = TemplateContext::new();

    // Extract [vars]
    context.vars = extract_vars_section(content)?;

    // Extract [template.matrix]
    context.matrix = extract_matrix_section(content)?;

    // Extract [template.env_defaults]
    context.env_defaults = extract_env_defaults(content)?;

    Ok(context)
}
```

**Example**:
```toml
[vars]
services = ["api", "web", "worker"]
image_tag = "v1.0.0"

{% for service in vars.services %}
[services.{{ service }}]
type = "generic_container"
image = "myapp/{{ service }}:{{ vars.image_tag }}"
{% endfor %}
```

**Renders to**:
```toml
[services.api]
type = "generic_container"
image = "myapp/api:v1.0.0"

[services.web]
type = "generic_container"
image = "myapp/web:v1.0.0"

[services.worker]
type = "generic_container"
image = "myapp/worker:v1.0.0"
```

### Phase 4: Environment & Filters (Week 2)

**Environment Variables**:
```rust
fn build_tera_context(template_ctx: TemplateContext) -> Result<tera::Context> {
    let mut ctx = tera::Context::new();

    // Add user vars
    ctx.insert("vars", &template_ctx.vars);

    // Add environment variables
    let env_vars: HashMap<String, String> = std::env::vars().collect();
    ctx.insert("env", &env_vars);

    // Add defaults
    ctx.insert("defaults", &template_ctx.env_defaults);

    Ok(ctx)
}
```

**Example**:
```toml
[vars]
service_name = "myapp"

[otel]
# Use env var with fallback
endpoint = "{{ env.OTEL_ENDPOINT | default(value='http://localhost:4318') }}"

# Use env var in service name
[meta]
name = "{{ vars.service_name }}_{{ env.USER | lower }}"
```

**Filters Already Built-in to Tera**:
- `upper`, `lower` - Case conversion
- `trim` - Remove whitespace
- `default(value='x')` - Fallback values
- `replace(from='x', to='y')` - String replacement

---

## File Changes Required

### 1. Update `config/loader.rs`

```rust
// OLD: Simple variable extraction
fn extract_vars_section(content: &str) -> Result<HashMap<String, Value>>

// NEW: Full template context extraction
fn extract_template_context(content: &str) -> Result<TemplateContext>

// NEW: Full Tera preprocessing
fn preprocess_template(content: &str, context: TemplateContext) -> Result<String>

// UPDATED: Use full preprocessing
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;
    let template_context = extract_template_context(&content)?;
    let rendered = preprocess_template(&content, template_context)?;
    let config: TestConfig = toml::from_str(&rendered)?;
    config.validate()?;
    Ok(config)
}
```

### 2. New `config/template_context.rs`

```rust
pub struct TemplateContext {
    /// Variables from [vars] section
    pub vars: HashMap<String, serde_json::Value>,

    /// Matrix data from [template.matrix] section
    pub matrix: Option<Vec<HashMap<String, serde_json::Value>>>,

    /// Environment variable defaults from [template.env_defaults]
    pub env_defaults: HashMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self { ... }

    pub fn to_tera_context(&self) -> tera::Context { ... }
}
```

### 3. Update `clnrm-template/src/renderer.rs`

```rust
impl TemplateRenderer {
    /// Preprocess full Tera template with control flow
    pub fn preprocess_full_template(
        &mut self,
        template_content: &str,
        context: TemplateContext,
    ) -> Result<String> {
        // Add template to Tera
        self.tera.add_raw_template("config", template_content)?;

        // Build Tera context
        let tera_ctx = context.to_tera_context()?;

        // Render with full Tera processing
        self.tera.render("config", &tera_ctx)
            .map_err(|e| TemplateError::RenderError(format!("Tera rendering failed: {}", e)))
    }
}
```

---

## Test Plan

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_conditional() {
        let template = r#"
[vars]
env = "prod"

{% if vars.env == "prod" %}
debug = false
{% else %}
debug = true
{% endif %}
"#;
        let rendered = preprocess_template(template, TemplateContext::default())?;
        assert!(rendered.contains("debug = false"));
        assert!(!rendered.contains("{% if"));
    }

    #[test]
    fn test_simple_loop() {
        let template = r#"
[vars]
services = ["api", "web"]

{% for service in vars.services %}
[services.{{ service }}]
type = "container"
{% endfor %}
"#;
        let rendered = preprocess_template(template, TemplateContext::default())?;
        assert!(rendered.contains("[services.api]"));
        assert!(rendered.contains("[services.web]"));
        assert!(!rendered.contains("{% for"));
    }

    #[test]
    fn test_env_vars() {
        std::env::set_var("TEST_VAR", "test_value");
        let template = r#"
value = "{{ env.TEST_VAR }}"
fallback = "{{ env.MISSING | default(value='fallback') }}"
"#;
        let rendered = preprocess_template(template, TemplateContext::default())?;
        assert!(rendered.contains("value = \"test_value\""));
        assert!(rendered.contains("fallback = \"fallback\""));
    }

    #[test]
    fn test_filters() {
        let template = r#"
[vars]
name = "MyService"

upper = "{{ vars.name | upper }}"
lower = "{{ vars.name | lower }}"
"#;
        let rendered = preprocess_template(template, TemplateContext::default())?;
        assert!(rendered.contains("upper = \"MYSERVICE\""));
        assert!(rendered.contains("lower = \"myservice\""));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_conditional_toml_validation() {
    let path = "tests/templates/conditional.clnrm.toml";
    let config = load_config_from_file(Path::new(path))?;
    assert_eq!(config.get_name()?, "prod_config");
}

#[test]
fn test_loop_toml_validation() {
    let path = "tests/templates/loop.clnrm.toml";
    let config = load_config_from_file(Path::new(path))?;
    assert_eq!(config.services.as_ref().unwrap().len(), 3);
}
```

### End-to-End Tests

```bash
# Test conditional rendering
clnrm validate tests/templates/conditional.clnrm.toml
# Should pass with rendered conditional

# Test loop rendering
clnrm validate tests/templates/loop.clnrm.toml
# Should pass with rendered services

# Test env vars
OTEL_ENDPOINT=http://example.com clnrm validate tests/templates/env.clnrm.toml
# Should use environment variable
```

---

## Example Templates (80/20 Coverage)

### 1. Multi-Environment Configuration

```toml
[vars]
env = "prod"  # Change to "dev", "staging", "prod"

[meta]
name = "myapp_{{ vars.env }}"

{% if vars.env == "prod" %}
[otel]
exporter = "otlp"
endpoint = "https://otel.prod.example.com"
sample_rate = 0.1

[services.database]
type = "generic_container"
image = "postgres:16-alpine"
environment = { "POSTGRES_PASSWORD" = "{{ env.DB_PASSWORD }}" }
{% elif vars.env == "staging" %}
[otel]
exporter = "otlp"
endpoint = "https://otel.staging.example.com"
sample_rate = 1.0

[services.database]
type = "generic_container"
image = "postgres:16-alpine"
environment = { "POSTGRES_PASSWORD" = "staging_pass" }
{% else %}
[otel]
exporter = "stdout"
sample_rate = 1.0

[services.database]
type = "generic_container"
image = "postgres:16-alpine"
environment = { "POSTGRES_PASSWORD" = "dev_pass" }
{% endif %}
```

### 2. Service Matrix Generation

```toml
[vars]
services = ["api", "web", "worker", "scheduler"]
base_image = "mycompany/base"
version = "v1.2.3"
replicas = 3

# Generate service definitions
{% for service in vars.services %}
[services.{{ service }}]
type = "generic_container"
image = "{{ vars.base_image }}/{{ service }}:{{ vars.version }}"

[[scenario]]
name = "test_{{ service }}_health"
service = "{{ service }}"
run = "curl http://localhost:8080/health"
{% endfor %}

# Generate test steps
{% for i in range(end=vars.replicas) %}
[[steps]]
name = "parallel_test_{{ i }}"
command = ["echo", "Running test batch {{ i }}"]
{% endfor %}
```

### 3. Environment-Aware OTEL Configuration

```toml
[vars]
service_name = "my_api"
environment = "{{ env.DEPLOY_ENV | default(value='development') }}"
region = "{{ env.AWS_REGION | default(value='us-west-2') }}"

[meta]
name = "{{ vars.service_name }}_{{ vars.environment }}"

[otel]
exporter = "otlp"
endpoint = "{{ env.OTEL_ENDPOINT | default(value='http://localhost:4318') }}"

[otel.resources]
"service.name" = "{{ vars.service_name }}"
"service.environment" = "{{ vars.environment }}"
"service.region" = "{{ vars.region }}"
"service.version" = "{{ env.GIT_COMMIT | default(value='unknown') | truncate(length=8) }}"
"host.name" = "{{ env.HOSTNAME | lower }}"
```

### 4. Test Matrix with Filters

```toml
[vars]
test_cases = [
    {name = "basic", timeout = 1000},
    {name = "advanced", timeout = 5000},
    {name = "stress", timeout = 30000}
]
service_prefix = "TEST"

{% for test in vars.test_cases %}
[[steps]]
name = "run_{{ test.name }}_test"
command = ["pytest", "tests/{{ test.name }}.py", "--timeout={{ test.timeout }}"]
service = "{{ vars.service_prefix | lower }}_runner"
{% endfor %}

[services.{{ vars.service_prefix | lower }}_runner]
type = "generic_container"
image = "python:3.11-slim"
```

---

## Success Metrics

### Before v1.5.1 (Current State)
- Templates passing: 1/9 (11%)
- Supported features: Variable substitution only
- Advanced templates: 0% working

### After v1.5.1 (Target)
- Templates passing: 7/9 (78%)
- Supported features: Variables, conditionals, loops, env vars, filters
- Advanced templates: 80% of use cases covered

### Coverage Target

| Feature | Before | After | Delta |
|---------|--------|-------|-------|
| Variable substitution | ✅ 100% | ✅ 100% | - |
| Simple conditionals | ❌ 0% | ✅ 100% | +100% |
| Simple loops | ❌ 0% | ✅ 100% | +100% |
| Environment variables | ❌ 0% | ✅ 100% | +100% |
| Basic filters | ❌ 0% | ✅ 100% | +100% |
| Macros | ❌ 0% | ❌ 0% | - (out of scope) |
| Template inheritance | ❌ 0% | ❌ 0% | - (out of scope) |

**Overall Coverage**: 11% → 78% (+67% improvement)

---

## Timeline

### Week 1: Core Template Preprocessing
- **Day 1-2**: Implement `extract_template_context()`
- **Day 3-4**: Implement `preprocess_template()` with conditionals
- **Day 5**: Unit tests for conditionals

### Week 2: Loops & Environment
- **Day 1-2**: Implement loop support
- **Day 3**: Environment variable access
- **Day 4**: Basic filter support
- **Day 5**: Integration tests

### Week 3: Testing & Documentation
- **Day 1-2**: Update all 6 advanced template files
- **Day 3**: End-to-end validation
- **Day 4**: Performance testing
- **Day 5**: Documentation & examples

**Total Effort**: 3 weeks (15 days)

---

## Risk Analysis

### Low Risk
- ✅ Tera library already supports all these features
- ✅ Similar pattern already working for simple substitution
- ✅ No breaking changes to existing configs

### Medium Risk
- ⚠️ Template context extraction from TOML strings
- ⚠️ TOML section detection with template syntax

### Mitigation
- Extensive unit tests for edge cases
- Validation tests on real template files
- Gradual rollout with feature flag

---

## Backward Compatibility

### All Existing Configs Continue Working

```toml
# v1.4.1 style (no templates)
[meta]
name = "my_test"
# ✅ Still works in v1.5.1

# v1.4.1 style (simple vars)
[vars]
name = "test"

[meta]
name = "{{ vars.name }}"
# ✅ Still works in v1.5.1

# v1.5.1 style (conditionals)
{% if vars.env == "prod" %}
debug = false
{% endif %}
# ✅ NEW in v1.5.1
```

---

## Documentation Plan

### User-Facing Docs
- `docs/TEMPLATE_GUIDE.md` - Complete template reference
- `docs/TEMPLATE_EXAMPLES.md` - Real-world examples
- `docs/TEMPLATE_MIGRATION.md` - Upgrade guide

### Developer Docs
- `docs/TEMPLATE_ARCHITECTURE.md` - Implementation details
- `crates/clnrm-template/README.md` - Template crate docs

---

## Conclusion

**v1.5.1 will deliver 80% of template functionality with 20% of the effort** by focusing on the most commonly needed features:

1. ✅ Variable substitution (already working)
2. 🎯 Simple conditionals
3. 🎯 Simple loops
4. 🎯 Environment variables
5. 🎯 Basic filters

This covers the vast majority of real-world template needs while keeping implementation scope reasonable.

**Estimated effort**: 3 weeks
**Impact**: 67% improvement in template coverage (11% → 78%)
**Risk**: Low (using proven Tera library features)

---

**Status**: ✅ **READY FOR IMPLEMENTATION**
