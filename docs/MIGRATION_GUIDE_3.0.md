# Migration Guide: v2.x to v3.0.0

This guide helps you migrate from clnrm v2.x to v3.0.0, which introduces gVisor as the default and only backend.

## Overview

Version 3.0.0 makes gVisor the default backend, eliminating the Docker daemon dependency. This provides:
- **Better isolation**: gVisor's user-space kernel provides stronger security boundaries
- **No Docker required**: Direct OCI image loading and execution
- **Faster startup**: Reduced overhead compared to Docker
- **Deterministic execution**: Better reproducibility for testing

## Prerequisites

### Install gVisor

Before migrating, install gVisor on your system:

**Linux:**
```bash
# Download and install runsc
curl -fsSL https://gvisor.dev/install | bash

# Verify installation
runsc --version
```

**macOS:**
```bash
# Install via Homebrew
brew install gvisor

# Verify installation
runsc --version
```

**Windows:**
gVisor is not currently supported on Windows. Use WSL2 with Linux installation.

## Migration Steps

### 1. Update Dependencies

Update your `Cargo.toml`:

```toml
[dependencies]
clnrm-core = "3.0.0"
clnrm-cli = "3.0.0"
```

**If you need legacy testcontainers support** (not recommended):

```toml
[dependencies]
clnrm-core = { version = "3.0.0", features = ["backend-testcontainers"] }
```

### 2. Update Backend Configuration

**Before (v2.x):**
```toml
[test]
backend = "testcontainers"  # or "auto"
```

**After (v3.0.0):**
```toml
[test]
backend = "gvisor"  # or "auto" (both use gVisor)
```

**Note**: If you don't specify a backend, `"auto"` is used, which now defaults to gVisor.

### 3. Service Plugin Changes

Service plugins automatically use gVisor in v3.0.0. No changes needed to your service configurations:

```toml
[containers.surrealdb]
type = "surrealdb"
image = "surrealdb/surrealdb:latest"

[containers.otel]
type = "otel_collector"
image = "otel/opentelemetry-collector:latest"
```

### 4. Port Allocation

gVisor uses a port allocator system instead of Docker's automatic port mapping. Ports are allocated deterministically:

- Ports are allocated from a configurable range (default: 10000-20000)
- Allocation is sequential by default (deterministic for testing)
- Ports are automatically released when services stop

**No changes needed** - this is handled automatically by the service plugins.

### 5. Error Handling

Error messages now reference gVisor instead of Docker/testcontainers:

**Before:**
```
Error: Docker daemon not running
```

**After:**
```
Error: runsc not found in PATH. Install gVisor: https://gvisor.dev/docs/user_guide/install/
```

### 6. Testing Your Migration

1. **Verify gVisor is installed:**
   ```bash
   runsc --version
   ```

2. **Run your tests:**
   ```bash
   cargo test
   ```

3. **Check for deprecation warnings:**
   If you see warnings about testcontainers, update your configuration to use gVisor.

## Breaking Changes

### Backend Selection

- `"testcontainers"` backend is no longer available by default
- Use `"gvisor"` or `"auto"` instead
- `"testcontainers"` available only with `backend-testcontainers` feature (deprecated)

### Service Plugin Behavior

- Service plugins now use gVisor backend internally
- Port allocation is deterministic (sequential by default)
- Container lifecycle management uses gVisor's runsc

### Error Types

- `testcontainers::TestcontainersError` conversion is feature-gated
- gVisor errors are handled through `CleanroomError` directly
- Error messages reference gVisor instead of Docker

## Troubleshooting

### "runsc not found"

**Solution**: Install gVisor following the prerequisites above.

### Port allocation failures

**Solution**: Adjust the port range in your configuration or use a different allocation strategy.

### Container startup failures

**Solution**: 
1. Verify the OCI image is accessible
2. Check gVisor logs: `runsc --root /path/to/root list`
3. Ensure sufficient system resources

### Legacy testcontainers code

**Solution**: 
1. Remove `backend-testcontainers` feature usage
2. Update to use gVisor backend
3. Migrate service plugins to gVisor-native implementations

## Rollback

If you need to rollback to v2.x:

1. Revert `Cargo.toml` dependencies to v2.0.0
2. Restore any backend configuration changes
3. Reinstall Docker if needed

**Note**: v2.x will continue to work, but v3.0.0+ features will not be available.

## Support

For issues or questions:
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: https://github.com/seanchatmangpt/clnrm/docs

## Next Steps

After migration:
1. Test all your scenarios with gVisor
2. Update CI/CD pipelines to install gVisor
3. Remove Docker dependencies from your development environment (optional)
4. Enjoy faster, more isolated test execution!


