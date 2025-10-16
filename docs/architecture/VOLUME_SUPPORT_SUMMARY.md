# Volume Support - Executive Summary

**Version**: 1.0
**Date**: 2025-10-16
**Estimated Implementation**: 6-8 hours

---

## Quick Overview

This design adds **minimal but complete volume support** to clnrm, covering 80% of use cases with bind mounts and read-only/read-write modes. Zero new dependencies required.

---

## What Gets Added

### New Module
- `crates/clnrm-core/src/backend/volume.rs` (NEW)
  - `VolumeMount` - Validated volume mount specification
  - `VolumeValidator` - Security validation with whitelist

### Modified Files
- `crates/clnrm-core/src/backend/mod.rs` - Add volume support to Backend trait
- `crates/clnrm-core/src/backend/testcontainer.rs` - Implement actual volume mounting
- `crates/clnrm-core/src/config.rs` - Add VolumeConfig validation
- `crates/clnrm-core/src/services/generic.rs` - Add volume support to plugin

---

## Key Design Decisions

### 1. 80/20 Feature Scope

**INCLUDE** (80% of use cases):
- ✅ Bind mounts (host directory → container directory)
- ✅ Read-only and read-write modes
- ✅ Service-level volume configuration
- ✅ Path validation and security checks
- ✅ TOML-based declarative configuration

**EXCLUDE** (defer to future):
- ❌ Named Docker volumes
- ❌ Volume drivers (NFS, cloud storage)
- ❌ Tmpfs mounts
- ❌ Per-step volume mounting
- ❌ Dynamic volume provisioning

### 2. Core Team Standards Compliance

✅ **No `.unwrap()` or `.expect()`** - All production code uses proper error handling
✅ **Result<T, CleanroomError>** - All functions return Results
✅ **Sync trait methods** - ServicePlugin trait remains sync (uses `block_in_place` internally)
✅ **Clear error messages** - Descriptive errors for debugging
✅ **Zero new dependencies** - Uses existing testcontainers functionality

### 3. Security Model

**Multi-layer validation**:
1. Configuration validation (TOML parsing)
2. Path validation (absolute, exists, no traversal)
3. Security validation (whitelist enforcement)
4. Container runtime enforcement (kernel-level)

**Default whitelist**:
- `/tmp`
- `/var/tmp`
- Current working directory

---

## TOML Configuration Example

```toml
[test.metadata]
name = "volume_integration_test"
description = "Test volume mounting with data persistence"

[services.data_processor]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

# Volume configuration
[[services.data_processor.volumes]]
host_path = "/tmp/test-data"
container_path = "/data"
read_only = false

[[services.data_processor.volumes]]
host_path = "/tmp/test-config"
container_path = "/config"
read_only = true

[[steps]]
name = "write_data"
command = ["sh", "-c", "echo 'test-data' > /data/output.txt"]
service = "data_processor"

[[steps]]
name = "verify_readonly"
command = ["sh", "-c", "echo 'should-fail' > /config/readonly.txt"]
service = "data_processor"
expected_exit_code = 1  # Should fail due to read-only
```

---

## Rust API Example

```rust
use clnrm_core::backend::TestcontainerBackend;
use clnrm_core::services::GenericContainerPlugin;

// Backend with volumes
let backend = TestcontainerBackend::new("alpine:latest")?
    .with_volume("/tmp/data", "/data", false)?
    .with_volume("/tmp/config", "/config", true)?;

// Service plugin with volumes
let plugin = GenericContainerPlugin::new("my_service", "alpine:latest")
    .with_volume("/tmp/data", "/data", false)?;

// From configuration
let plugin = GenericContainerPlugin::new("service", "alpine:latest")
    .with_volumes_from_config(&service_config.volumes.unwrap())?;
```

---

## Implementation Phases

### Phase 1: Core Volume Support (2-3 hours)
1. Create `backend/volume.rs` with VolumeMount and VolumeValidator
2. Add VolumeConfig validation in `config.rs`
3. Modify TestcontainerBackend to use VolumeMount
4. Implement actual volume mounting in `execute_in_container()`

### Phase 2: Plugin Integration (1-2 hours)
1. Update GenericContainerPlugin with volume support
2. Create service creation helper function
3. Wire up volume configuration in service registry

### Phase 3: Testing (2-3 hours)
1. Unit tests for VolumeMount and VolumeValidator
2. Integration tests for volume mounting
3. Security tests for path validation
4. TOML configuration tests

### Phase 4: Documentation (1 hour)
1. Update TOML_REFERENCE.md
2. Create VOLUME_GUIDE.md
3. Add example test files

---

## Key Data Structures

### VolumeMount
```rust
pub struct VolumeMount {
    host_path: PathBuf,        // Absolute, canonical
    container_path: PathBuf,   // Absolute
    read_only: bool,
}

impl VolumeMount {
    pub fn new(host, container, ro) -> Result<Self>;
    pub fn from_config(config: &VolumeConfig) -> Result<Self>;
    pub fn to_mount_string(&self) -> String;  // "/host:/container:ro"
}
```

### VolumeValidator
```rust
pub struct VolumeValidator {
    allowed_host_dirs: Vec<PathBuf>,
    permissive: bool,
}

impl VolumeValidator {
    pub fn new(allowed_dirs: Vec<PathBuf>) -> Self;
    pub fn permissive() -> Self;
    pub fn validate(&self, mount: &VolumeMount) -> Result<()>;
}
```

---

## Security Features

### Path Validation
- ✅ Require absolute paths (no relative paths)
- ✅ Verify host paths exist
- ✅ Canonicalize to resolve symlinks
- ✅ Reject paths with `..` components (traversal attack prevention)

### Whitelist Enforcement
- ✅ Default whitelist: `/tmp`, `/var/tmp`, CWD
- ✅ Custom whitelist support
- ✅ Permissive mode for trusted environments

### Runtime Protection
- ✅ Read-only enforcement at container level
- ✅ Container user isolation (non-root)
- ✅ No privilege escalation

---

## Testing Strategy

### Unit Tests (~20 tests, <1 second)
- VolumeMount creation and validation
- VolumeValidator whitelist logic
- VolumeConfig validation
- Path validation edge cases

### Integration Tests (~10 tests, 30-60 seconds)
- Mount volume and write file
- Read-only enforcement
- Multiple volumes on same container
- Host filesystem persistence

### TOML Tests (~5 files, 30-60 seconds)
- Basic volume mounting
- Read-only enforcement
- Multiple volumes
- Data processing pipeline

### Security Tests (~8 tests, 10-20 seconds)
- Reject relative paths
- Reject nonexistent paths
- Reject path traversal
- Enforce whitelist

**Total Coverage**: >80%

---

## Performance Impact

| Operation | Impact |
|-----------|--------|
| Volume validation | 2-10ms per volume |
| Container startup | +10-50ms per volume |
| File I/O through volume | Zero overhead (bind mounts are native) |
| Memory overhead | ~200 bytes per volume |

**Expected total overhead**: <100ms for typical test with 2-3 volumes

---

## Error Messages

All errors are clear and actionable:

```rust
// Validation errors
"Host path must be absolute: 'relative/path'"
"Host path does not exist: '/nonexistent/path'"
"Container path must be absolute: 'relative/path'"

// Security errors
"Host path '/etc/shadow' not in allowed directories. Allowed: ['/tmp', '/var/tmp']"
"Host path contains parent directory traversal"

// Container errors
"Failed to mount volume: permission denied"
```

---

## Definition of Done

- [ ] All code passes `cargo clippy -- -D warnings` with zero warnings
- [ ] All tests pass: `cargo test`
- [ ] Integration tests demonstrate volume functionality
- [ ] TOML reference documentation updated
- [ ] Example test files created
- [ ] Security validation tested
- [ ] Framework self-test includes volume tests
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] Coverage >80%

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Testcontainers API changes | Use stable API methods, version constraints |
| Path validation edge cases | Comprehensive unit tests, fuzzing |
| Permission issues | Clear error messages, documentation |
| Host filesystem exposure | Multi-layer validation, whitelist enforcement |
| Path traversal attacks | Canonicalization, parent dir rejection |

---

## Future Extensions (Post-MVP)

After initial 80/20 implementation, consider:

1. **Named volumes** - Docker named volumes (not bind mounts)
2. **Tmpfs mounts** - In-memory volumes for temporary data
3. **Volume drivers** - Cloud storage, NFS integration
4. **Per-step volumes** - Dynamic mounting during test execution
5. **Volume lifecycle** - Automatic creation/cleanup

---

## Success Criteria

### Functional
- ✅ Support bind mount volumes in TOML
- ✅ Support read-only and read-write modes
- ✅ Validate host and container paths
- ✅ Integrate with TestcontainerBackend
- ✅ Support multiple volumes per service
- ✅ Security validation with whitelisting

### Non-Functional
- ✅ Zero new dependencies
- ✅ <100ms overhead per test
- ✅ Core team standards compliant
- ✅ >80% test coverage
- ✅ Clear error messages

---

## Related Documents

1. **[volume-support-design.md](./volume-support-design.md)** - Complete architecture design (13 sections, 60+ pages)
2. **[volume-support-trait-signatures.md](./volume-support-trait-signatures.md)** - All trait method signatures and APIs
3. **[volume-support-architecture-diagram.md](./volume-support-architecture-diagram.md)** - Visual diagrams and flow charts

---

## Quick Start for Implementers

1. Read this summary (5 minutes)
2. Review trait signatures document (10 minutes)
3. Look at architecture diagrams (10 minutes)
4. Start with Phase 1: Create `backend/volume.rs` (2-3 hours)
5. Implement Phase 2: Plugin integration (1-2 hours)
6. Write tests Phase 3 (2-3 hours)
7. Document Phase 4 (1 hour)

**Total time**: 6-8 hours for production-ready volume support.

---

## Questions?

**Q: Why not named volumes?**
A: Bind mounts cover 80% of use cases. Named volumes add complexity (lifecycle management, cleanup) for minimal benefit in testing context.

**Q: Why not per-step volumes?**
A: Service-level volumes are simpler and sufficient for most test scenarios. Per-step mounting adds complexity for edge cases.

**Q: Why whitelist by default?**
A: Security-first approach. Users can opt into permissive mode if needed.

**Q: Performance impact?**
A: Minimal. Bind mounts are native filesystem operations with zero data copying.

**Q: Backward compatible?**
A: Yes. All new Backend trait methods have default implementations. VolumeConfig is optional.

---

## Approval Checklist

- [ ] Architecture design reviewed
- [ ] 80/20 scope approved
- [ ] Security model approved
- [ ] Core team standards compliance verified
- [ ] Performance impact acceptable
- [ ] Testing strategy approved
- [ ] Documentation plan approved
- [ ] Implementation timeline approved

**Ready to implement**: Yes ✅

---

*Generated as part of the clnrm volume support architecture design.*
