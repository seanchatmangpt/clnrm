# clnrm-migrate: Docker/Testcontainers → gVisor Migration Tool

Automated migration tool for converting Docker/testcontainers service configurations to gVisor format.

## Features

- **Auto-detection**: Scans codebase for testcontainers usage
- **Automated Conversion**: Converts `.clnrm.toml` and Rust service definitions
- **Validation**: Ensures converted configurations are valid
- **Reporting**: Generates detailed migration reports (JSON + Markdown)
- **Backwards Compatibility**: Gradual migration with dual-mode support

## Installation

```bash
cd /home/user/clnrm
cargo build --release -p clnrm-migrate
```

## Usage

### Full Migration Pipeline

Run the complete migration in one command:

```bash
clnrm-migrate all --root /home/user/clnrm --output ./migration-output
```

This will:
1. Scan the codebase for testcontainers services
2. Convert them to gVisor format
3. Validate the converted configurations
4. Generate a detailed migration report
5. Write `gvisor-services.toml` with all service definitions

### Step-by-Step Migration

#### 1. Scan Codebase

```bash
clnrm-migrate scan --root /home/user/clnrm --output scan-results.json
```

Discovers all testcontainers service definitions in:
- `.clnrm.toml` files
- Rust source files (`src/services/*.rs`)
- Inline test configurations

#### 2. Convert Configurations

```bash
clnrm-migrate convert --root /home/user/clnrm --output ./migration-output
```

Converts discovered services to gVisor TOML format.

#### 3. Validate Configurations

```bash
clnrm-migrate validate --config ./migration-output/gvisor-services.toml
```

Validates:
- TOML syntax
- Image URLs
- Resource limits
- Network configuration
- Security settings

## Output Files

### gvisor-services.toml

The main configuration file containing all migrated service definitions:

```toml
[registry.metadata]
version = "1.0.0"
schema_version = "gvisor-v1"
created_at = "2026-01-05T00:00:00Z"

[[services]]
name = "surrealdb"
service_type = "database"
# ... full service configuration
```

### migration-report.md

Human-readable migration report:

```markdown
# gVisor Migration Report

## Summary
- Total services found: 15
- Converted services: 15
- Validation errors: 0
- Validation warnings: 3

## Converted Services
| Service | Type | Status | Warnings |
|---------|------|--------|----------|
| surrealdb | database | ✅ Auto | 0 |
| alpine | generic | ✅ Auto | 0 |
...
```

### migration-report.json

Machine-readable report for automation:

```json
{
  "timestamp": "2026-01-05T12:00:00Z",
  "total_services": 15,
  "converted_services": 15,
  "services": [...],
  "errors": [],
  "warnings": [...]
}
```

## Supported Service Types

### Automatically Converted

1. **SurrealDB** (`type = "surrealdb"`)
   - Extracts username, password, strict mode
   - Configures health checks
   - Sets up networking and resources

2. **Generic Containers** (`type = "generic_container"`)
   - Preserves image, environment, ports
   - Converts volume mounts
   - Applies default resource limits

3. **Custom Images** (with explicit image field)
   - Basic conversion with manual review flag
   - Preserves environment and network config

### Manual Migration Required

1. **Testcontainers Modules** (direct Rust API usage)
   - Scanner detects these
   - Requires manual review and configuration

## Configuration Examples

See `/home/user/clnrm/examples/gvisor/` for complete examples:

- `surrealdb.gvisor.toml` - Database service
- `alpine.gvisor.toml` - Minimal generic container
- `custom-app.gvisor.toml` - Full-featured application

## Migration Checklist

- [ ] Run `clnrm-migrate all` to generate configurations
- [ ] Review `migration-report.md` for warnings
- [ ] Address any validation errors
- [ ] Test migrated services in development environment
- [ ] Update `.clnrm.toml` files to reference gVisor backend
- [ ] Enable gVisor as default backend in `cleanroom.toml`
- [ ] Run integration tests
- [ ] Document any manual migrations

## Backwards Compatibility

The migration tool generates configurations that can coexist with existing testcontainers setup:

```toml
# In cleanroom.toml
[cleanroom.backend]
default = "auto"  # Auto-select: gVisor if available, else testcontainers
fallback_enabled = true
```

This allows gradual migration without breaking existing tests.

## Troubleshooting

### "Invalid image URL format"

Ensure image URLs use proper scheme:
- ✅ `docker://alpine:latest`
- ✅ `oci://registry.io/image@sha256:...`
- ❌ `alpine:latest` (missing scheme)

### "Service not detected"

Check that service is defined in:
- `.clnrm.toml` files under `[services.*]`
- Rust files using `ServicePlugin` trait

### "TOML syntax error"

Validate TOML syntax:
```bash
clnrm-migrate validate --config gvisor-services.toml
```

## Advanced Features

### Custom Conversion Rules

Extend `Converter` trait to add custom conversion logic:

```rust
impl Converter {
    fn convert_custom(&self, discovery: &ServiceDiscovery) -> Result<ConversionResult> {
        // Custom logic here
    }
}
```

### Filtering Services

Use JSON report for selective migration:

```bash
# Extract only database services
jq '.services[] | select(.source.service_type == "database")' migration-report.json
```

## Contributing

To add support for new service types:

1. Update `ServiceType` enum in `src/types.rs`
2. Add detection logic in `Scanner::detect_type()`
3. Implement conversion in `Converter::convert_*()`
4. Add validation rules in `Validator`
5. Create example configuration

## Resources

- [gVisor Migration Design](../../docs/GVISOR_MIGRATION_DESIGN.md)
- [gVisor Configuration Reference](../../docs/GVISOR_CONFIG_REFERENCE.md)
- [Example Configurations](../../examples/gvisor/)
- [gVisor Documentation](https://gvisor.dev/docs/)

## License

Same as clnrm project.
