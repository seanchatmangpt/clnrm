# Volume Support Architecture Documentation

This directory contains the complete architecture design for adding volume support to the clnrm framework using the 80/20 principle.

---

## Document Index

### 1. **[VOLUME_SUPPORT_SUMMARY.md](./VOLUME_SUPPORT_SUMMARY.md)** ⭐ START HERE
**Read Time**: 5-10 minutes
**Audience**: Everyone

Quick executive summary covering:
- What gets added
- Key design decisions
- TOML and Rust examples
- Success criteria
- 6-8 hour implementation estimate

**Use this to**: Get a quick overview before diving into details.

---

### 2. **[volume-support-design.md](./volume-support-design.md)**
**Read Time**: 30-45 minutes
**Audience**: Architects, senior developers

Complete architecture design with 13 sections:
1. Current architecture analysis
2. 80/20 feature scope (what's included vs deferred)
3. Core data structures (VolumeMount, VolumeValidator)
4. Backend integration (trait enhancements)
5. TOML configuration schema
6. Service plugin integration
7. Implementation strategy (4 phases)
8. Security & validation (multi-layer)
9. Testing strategy (>80% coverage)
10. Documentation requirements
11. Success criteria
12. Risk analysis
13. Future extensions

**Use this to**: Understand the complete architecture and rationale.

---

### 3. **[volume-support-trait-signatures.md](./volume-support-trait-signatures.md)**
**Read Time**: 15-20 minutes
**Audience**: Implementers, reviewers

Detailed trait method signatures and integration points:
- Complete API documentation for VolumeMount and VolumeValidator
- Backend trait extensions with examples
- TestcontainerBackend modifications
- ServicePlugin enhancements
- Configuration validation methods
- Error types and messages
- Builder pattern examples
- Thread safety guarantees

**Use this to**: Implement the code or review API design.

---

### 4. **[volume-support-architecture-diagram.md](./volume-support-architecture-diagram.md)**
**Read Time**: 10-15 minutes
**Audience**: Visual learners, architects

Visual representations including:
- Component architecture diagram
- Data flow sequence diagram
- Module dependency graph
- Class diagram (Rust structures)
- Security architecture layers
- Error flow diagram
- Implementation phases timeline
- Testing strategy visualization
- Use case scenarios
- Performance characteristics

**Use this to**: Visualize the architecture and data flows.

---

### 5. **[volume-support-implementation-checklist.md](./volume-support-implementation-checklist.md)**
**Read Time**: 20-30 minutes (reference)
**Audience**: Implementers, QA testers

Step-by-step implementation guide with checkboxes:
- Phase 1: Core volume support (2-3 hours)
- Phase 2: Plugin integration (1-2 hours)
- Phase 3: Testing (2-3 hours)
- Phase 4: Documentation (1 hour)
- Verification & quality assurance checklist
- Pre-merge checklist
- Post-merge tasks
- Troubleshooting guide

**Use this to**: Track progress during implementation.

---

## Quick Navigation

### By Role

**Project Manager / Stakeholder**:
1. Read: [VOLUME_SUPPORT_SUMMARY.md](./VOLUME_SUPPORT_SUMMARY.md)
2. Skim: [volume-support-design.md](./volume-support-design.md) (sections 1, 2, 11)

**Architect**:
1. Read: [VOLUME_SUPPORT_SUMMARY.md](./VOLUME_SUPPORT_SUMMARY.md)
2. Read: [volume-support-design.md](./volume-support-design.md) (all sections)
3. Review: [volume-support-architecture-diagram.md](./volume-support-architecture-diagram.md)

**Developer / Implementer**:
1. Read: [VOLUME_SUPPORT_SUMMARY.md](./VOLUME_SUPPORT_SUMMARY.md)
2. Read: [volume-support-trait-signatures.md](./volume-support-trait-signatures.md)
3. Use: [volume-support-implementation-checklist.md](./volume-support-implementation-checklist.md)
4. Reference: [volume-support-design.md](./volume-support-design.md) as needed

**Reviewer / QA**:
1. Read: [VOLUME_SUPPORT_SUMMARY.md](./VOLUME_SUPPORT_SUMMARY.md)
2. Review: [volume-support-design.md](./volume-support-design.md) (sections 8, 9)
3. Use: [volume-support-implementation-checklist.md](./volume-support-implementation-checklist.md) (verification sections)

**Security Reviewer**:
1. Read: [VOLUME_SUPPORT_SUMMARY.md](./VOLUME_SUPPORT_SUMMARY.md)
2. Read: [volume-support-design.md](./volume-support-design.md) (section 8: Security)
3. Review: [volume-support-architecture-diagram.md](./volume-support-architecture-diagram.md) (section 5: Security Architecture)

---

## Key Design Principles

### 1. 80/20 Rule
Cover 80% of use cases with 20% of the implementation effort:
- ✅ Bind mounts (included)
- ✅ Read-only/read-write (included)
- ❌ Named volumes (deferred)
- ❌ Volume drivers (deferred)

### 2. Core Team Standards
- ✅ No `.unwrap()` or `.expect()` in production code
- ✅ All functions return `Result<T, CleanroomError>`
- ✅ Sync trait methods (dyn compatible)
- ✅ Clear error messages
- ✅ Comprehensive testing (>80% coverage)

### 3. Security First
- Multi-layer validation
- Path canonicalization
- Whitelist enforcement
- Read-only enforcement at kernel level

### 4. Zero New Dependencies
Uses existing testcontainers functionality - no new crates required.

---

## Implementation Summary

### What Gets Added

**New Module**:
- `crates/clnrm-core/src/backend/volume.rs` (~300 lines)
  - `VolumeMount` - Validated volume specification
  - `VolumeValidator` - Security validation with whitelist

**Modified Files** (~500 lines total):
- `crates/clnrm-core/src/backend/mod.rs` - Backend trait extensions
- `crates/clnrm-core/src/backend/testcontainer.rs` - Volume mounting implementation
- `crates/clnrm-core/src/config.rs` - VolumeConfig validation
- `crates/clnrm-core/src/services/generic.rs` - Plugin volume support

### Implementation Estimate

| Phase | Time | Tasks |
|-------|------|-------|
| Phase 1: Core Support | 2-3 hours | Volume module, config validation, backend integration |
| Phase 2: Plugin Integration | 1-2 hours | GenericContainerPlugin, service helpers |
| Phase 3: Testing | 2-3 hours | Unit, integration, security, TOML tests |
| Phase 4: Documentation | 1 hour | TOML reference, volume guide, examples |
| **Total** | **6-8 hours** | Production-ready volume support |

---

## TOML Configuration Example

```toml
[test.metadata]
name = "volume_integration_test"

[services.data_processor]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

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
expected_exit_code = 1
```

---

## Rust API Example

```rust
use clnrm_core::backend::TestcontainerBackend;
use clnrm_core::backend::volume::{VolumeMount, VolumeValidator};
use clnrm_core::services::GenericContainerPlugin;

// Backend with volumes
let backend = TestcontainerBackend::new("alpine:latest")?
    .with_volume("/tmp/data", "/data", false)?
    .with_volume("/tmp/config", "/config", true)?;

// Service plugin with volumes
let plugin = GenericContainerPlugin::new("my_service", "alpine:latest")
    .with_volume("/tmp/data", "/data", false)?;

// Custom validator
let validator = VolumeValidator::new(vec![
    PathBuf::from("/tmp"),
    PathBuf::from("/var/tmp"),
]);
let backend = TestcontainerBackend::new("alpine:latest")?
    .with_volume_validator(validator);
```

---

## Testing Strategy

### Test Coverage

| Test Type | Count | Time | Coverage |
|-----------|-------|------|----------|
| Unit Tests | ~20 | <1s | Path validation, config validation |
| Integration Tests | ~10 | 30-60s | Container mounting, file I/O |
| TOML Tests | ~5 files | 30-60s | End-to-end with real configs |
| Security Tests | ~8 | 10-20s | Path traversal, whitelist |
| **Total** | **~43 tests** | **<2 min** | **>80%** |

---

## Security Features

### Multi-Layer Validation

1. **Configuration Layer**: TOML parsing and validation
2. **Path Layer**: Absolute paths, existence checks, canonicalization
3. **Security Layer**: Whitelist enforcement, traversal prevention
4. **Runtime Layer**: Kernel-level read-only enforcement

### Default Security

- Whitelist: `/tmp`, `/var/tmp`, current working directory
- Reject relative paths
- Reject nonexistent paths
- Reject paths with `..` components
- Read-only enforcement at container level

---

## Performance Impact

| Operation | Impact |
|-----------|--------|
| Volume validation | 2-10ms per volume |
| Container startup | +10-50ms per volume |
| File I/O through volume | Zero overhead (native bind mounts) |
| Memory overhead | ~200 bytes per volume |

**Expected total overhead**: <100ms for typical test with 2-3 volumes

---

## Success Criteria

### Functional Requirements
- ✅ Support bind mount volumes in TOML
- ✅ Support read-only and read-write modes
- ✅ Validate host and container paths
- ✅ Integrate with TestcontainerBackend
- ✅ Support multiple volumes per service
- ✅ Security validation with whitelisting

### Non-Functional Requirements
- ✅ Zero new dependencies
- ✅ <100ms overhead per test
- ✅ Core team standards compliant (no unwrap/expect)
- ✅ >80% test coverage
- ✅ Clear error messages
- ✅ Backward compatible

---

## Future Extensions (Post-MVP)

After initial 80/20 implementation:

1. **Named volumes** - Docker named volumes (not bind mounts)
2. **Tmpfs mounts** - In-memory volumes for temporary data
3. **Volume drivers** - Cloud storage, NFS integration
4. **Per-step volumes** - Dynamic mounting during test execution
5. **Volume lifecycle** - Automatic creation/cleanup

---

## FAQ

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

**Q: New dependencies?**
A: None. Uses existing testcontainers functionality.

---

## Related Documentation

### User Documentation
- `docs/TOML_REFERENCE.md` - TOML configuration reference (to be updated)
- `docs/VOLUME_GUIDE.md` - Volume usage guide (to be created)
- `docs/CLI_GUIDE.md` - CLI usage guide (to be updated)

### Developer Documentation
- `docs/TESTING.md` - Testing guide
- `.cursorrules` - Core team standards

### Examples
- `examples/volumes/` - Volume usage examples (to be created)
- `tests/volumes/` - Volume test cases (to be created)

---

## Contact & Review

### Review Process

1. **Design Review**: Architecture team reviews all documents
2. **Security Review**: Security team reviews section 8 of main design
3. **API Review**: Senior developers review trait signatures
4. **Implementation Review**: Code review during development

### Questions?

For questions about this architecture:
- File an issue: https://github.com/seanchatmangpt/clnrm/issues
- Refer to: Core team standards in `.cursorrules`

---

## Document Status

| Document | Status | Last Updated | Version |
|----------|--------|--------------|---------|
| VOLUME_SUPPORT_SUMMARY.md | ✅ Complete | 2025-10-16 | 1.0 |
| volume-support-design.md | ✅ Complete | 2025-10-16 | 1.0 |
| volume-support-trait-signatures.md | ✅ Complete | 2025-10-16 | 1.0 |
| volume-support-architecture-diagram.md | ✅ Complete | 2025-10-16 | 1.0 |
| volume-support-implementation-checklist.md | ✅ Complete | 2025-10-16 | 1.0 |

**Design Phase**: Complete ✅
**Ready for Implementation**: Yes ✅
**Estimated Effort**: 6-8 hours

---

*This architecture documentation is part of the clnrm hermetic testing framework.*
