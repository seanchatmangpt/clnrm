# Shape Validation Rules - v0.7.0

This document describes the comprehensive shape validation system implemented for the Cleanroom Testing Framework v0.7.0.

## Overview

The shape validator performs static analysis of TOML configuration files without spinning up containers, providing fast feedback during development.

## Validation Categories

### 1. Container Image Validation

**Rules:**
- Image name cannot be empty
- Image format must be valid (no spaces or invalid characters)
- Image format: `[registry/][namespace/]repository[:tag|@digest]`
- Maximum 3 path segments (registry/namespace/repository)

**Valid Examples:**
```toml
[service.myapp]
image = "alpine:latest"           # ✅ Simple image with tag
image = "ubuntu:20.04"            # ✅ Image with version tag
image = "docker.io/library/postgres:14"  # ✅ Full registry path
image = "ghcr.io/owner/repo:v1.0.0"      # ✅ GitHub container registry
image = "localhost:5000/myimage:tag"      # ✅ Local registry with port
```

**Invalid Examples:**
```toml
[service.bad1]
image = ""                        # ❌ Empty image

[service.bad2]
image = "invalid image format!!!" # ❌ Contains spaces and invalid chars

[service.bad3]
image = "registry/ns/repo/extra"  # ❌ Too many path segments
```

**Error Messages:**
- "Service 'X' has empty image. Suggestion: Use a valid image like 'alpine:latest' or 'ubuntu:20.04'"
- "Service 'X' has invalid image format 'Y'. Images cannot contain spaces. Example: 'alpine:latest'"

### 2. Port Binding Validation

**Rules:**
- Reserved ports (1-1023) are flagged
- Port conflicts across services are detected
- Each service must use unique ports

**Valid Example:**
```toml
[service.api]
ports = [8080, 9000]

[service.database]
ports = [5432]              # ✅ No conflicts
```

**Invalid Examples:**
```toml
[service.bad1]
ports = [22, 80, 443]      # ❌ Reserved system ports

[service.api1]
ports = [8080]

[service.api2]
ports = [8080]              # ❌ Port conflict
```

**Error Messages:**
- "Service 'X' uses reserved port Y. Suggestion: Use ports >= 1024 (e.g., 8080, 9000, 3000)"
- "Port conflict detected: port Y is used by multiple services: X, Z. Each service must use unique ports."

### 3. Volume Mount Validation

**Rules:**
- Host paths must be absolute
- Container paths must be absolute
- Dangerous system paths are flagged (`/etc`, `/var`, `/proc`, `/sys`, `/dev`, `/boot`, etc.)
- Readonly flag recommended for sensitive mounts

**Valid Example:**
```toml
[service.app]
volumes = [
    { host_path = "/tmp/data", container_path = "/app/data", read_only = false }
]
```

**Invalid Examples:**
```toml
[service.bad1]
volumes = [
    { host_path = "relative/path", container_path = "/app/data" }  # ❌ Relative path
]

[service.bad2]
volumes = [
    { host_path = "/tmp/data", container_path = "/etc" }  # ❌ Dangerous system path
]
```

**Error Messages:**
- "Service 'X' volume Y: host path 'Z' must be absolute. Suggestion: Use '/tmp/data' or '/home/user/project'"
- "Service 'X' volume Y: mounting to system path 'Z' is dangerous. Suggestion: Use application paths like '/app/data'"

### 4. Environment Variable Validation

**Rules:**
- Variable names cannot be empty
- Names must start with letter or underscore
- Names contain only alphanumeric characters and underscores
- Pattern: `^[A-Za-z_][A-Za-z0-9_]*$`
- Hardcoded sensitive values are flagged (API_KEY, PASSWORD, SECRET, TOKEN, etc.)

**Valid Example:**
```toml
[service.app]
env = { APP_ENV = "development", DATABASE_URL = "${DB_URL}" }
```

**Invalid Examples:**
```toml
[service.bad1]
env = { "123_INVALID" = "value" }     # ❌ Starts with number

[service.bad2]
env = { "INVALID-NAME" = "value" }    # ❌ Contains hyphen

[service.bad3]
env = { API_KEY = "hardcoded_key" }   # ❌ Hardcoded sensitive value
```

**Error Messages:**
- "Service 'X': environment variable name cannot be empty. Use uppercase names like 'APP_ENV' or 'DATABASE_URL'"
- "Service 'X': invalid environment variable name 'Y'. Names must start with a letter or underscore. Example: 'DATABASE_URL'"
- "Service 'X': potential hardcoded sensitive value in 'API_KEY'. Suggestion: Use environment variable references like '${ENV_VAR}'"

### 5. Service Dependency Validation

**Rules:**
- Circular service dependencies are detected
- Dependencies inferred from health check commands
- Steps referencing non-existent services are flagged

**Valid Example:**
```toml
[service.app]
health_check = { cmd = ["curl", "http://localhost:8080/health"] }

[service.db]
health_check = { cmd = ["pg_isready"] }
```

**Invalid Example:**
```toml
[service.service_a]
health_check = { cmd = ["curl", "http://service_b:8080"] }

[service.service_b]
health_check = { cmd = ["curl", "http://service_a:8080"] }  # ❌ Circular dependency
```

**Error Messages:**
- "Circular service dependency detected involving 'X'. Services cannot depend on each other in a cycle."
- "Step 'X' references undefined service 'Y'"

## London School TDD Methodology

The shape validation system was developed using London School (mockist) TDD:

### 1. Outside-In Development
- Started with acceptance tests for complete validation scenarios
- Mocked invalid configurations to drive implementation
- Tested behavior through interaction verification

### 2. Mock-Driven Development
- Used mocks to isolate validation rules
- Defined contracts through mock expectations
- Focused on HOW validators collaborate

### 3. Behavior Verification
- Tests verify error messages contain suggestions
- Tests verify error categories are correct
- Tests verify all validation paths are covered

## Integration

The shape validator integrates with:

1. **Config Parser**: Validates after TOML parsing
2. **CLI Commands**: `clnrm lint` and `clnrm dry-run`
3. **Template System**: Works with Tera templates
4. **Error Reporting**: Provides detailed, actionable errors

## Usage Examples

### Programmatic Usage
```rust
use clnrm_core::validation::shape::ShapeValidator;
use clnrm_core::config::TestConfig;

let mut validator = ShapeValidator::new();
let result = validator.validate_config(&config)?;

if !result.passed {
    for error in result.errors {
        eprintln!("[{}] {}", error.category, error.message);
    }
}
```

### CLI Usage
```bash
# Validate all test files
clnrm lint tests/

# Validate specific file
clnrm lint tests/api.clnrm.toml

# Dry-run validation (no containers)
clnrm dry-run tests/
```

## Performance

- **Fast**: Static analysis, no container overhead
- **Comprehensive**: Covers all configuration aspects
- **Developer-Friendly**: Clear error messages with suggestions
- **Integration-Ready**: Works with CI/CD pipelines

## Future Enhancements

- [ ] Resource limit validation (CPU, memory)
- [ ] Network configuration validation
- [ ] Security policy validation
- [ ] Multi-file dependency validation
- [ ] Custom validation rules via plugins

## References

- [TOML Reference](../TOML_REFERENCE.md)
- [Testing Guide](../TESTING.md)
- [Core Team Standards](../../.cursorrules)
