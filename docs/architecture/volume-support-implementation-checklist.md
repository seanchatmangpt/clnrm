# Volume Support - Implementation Checklist

**Version**: 1.0
**Date**: 2025-10-16
**Estimated Total Time**: 6-8 hours

This checklist provides a step-by-step guide for implementing volume support in the clnrm framework.

---

## Phase 1: Core Volume Support (2-3 hours)

### Step 1.1: Create Volume Module

**File**: `crates/clnrm-core/src/backend/volume.rs`

- [ ] Create new module file
- [ ] Add module declaration to `crates/clnrm-core/src/backend/mod.rs`:
  ```rust
  pub mod volume;
  pub use volume::{VolumeMount, VolumeValidator};
  ```

**VolumeMount Structure**:
- [ ] Define `VolumeMount` struct with `PathBuf` fields
- [ ] Implement `VolumeMount::new()` with validation
- [ ] Implement `VolumeMount::from_config()` converter
- [ ] Implement `VolumeMount::host_path()` getter
- [ ] Implement `VolumeMount::container_path()` getter
- [ ] Implement `VolumeMount::is_read_only()` getter
- [ ] Implement `VolumeMount::to_mount_string()` formatter
- [ ] Add `#[derive(Debug, Clone, PartialEq, Eq)]`

**Validation in VolumeMount::new()**:
- [ ] Check host path is absolute
- [ ] Check host path exists
- [ ] Canonicalize host path (resolve symlinks)
- [ ] Check container path is absolute
- [ ] Reject paths with `..` components
- [ ] Return `Result<VolumeMount, CleanroomError>`

**VolumeValidator Structure**:
- [ ] Define `VolumeValidator` struct
- [ ] Implement `VolumeValidator::new()` with whitelist
- [ ] Implement `VolumeValidator::permissive()` constructor
- [ ] Implement `VolumeValidator::validate()` for single mount
- [ ] Implement `VolumeValidator::validate_all()` for multiple mounts
- [ ] Implement `Default` trait with safe defaults (`/tmp`, `/var/tmp`, CWD)

**Tests for volume.rs**:
- [ ] Test `VolumeMount::new()` with valid paths
- [ ] Test rejection of relative host path
- [ ] Test rejection of nonexistent host path
- [ ] Test rejection of relative container path
- [ ] Test `to_mount_string()` format (read-only)
- [ ] Test `to_mount_string()` format (read-write)
- [ ] Test `VolumeValidator::default()` allows `/tmp`
- [ ] Test `VolumeValidator` rejects unauthorized paths
- [ ] Test `VolumeValidator::permissive()` allows all paths

### Step 1.2: Update Config Validation

**File**: `crates/clnrm-core/src/config.rs`

**VolumeConfig Validation**:
- [ ] Add `impl VolumeConfig` block
- [ ] Implement `validate()` method
- [ ] Check `host_path` not empty
- [ ] Check `container_path` not empty
- [ ] Check `host_path` is absolute
- [ ] Check `container_path` is absolute
- [ ] Return `Result<(), CleanroomError>`

**ServiceConfig Enhancement**:
- [ ] Update `ServiceConfig::validate()` to validate volumes
- [ ] Iterate through `self.volumes` if present
- [ ] Call `volume.validate()` for each volume
- [ ] Add context to errors (e.g., "Volume 0: ...")

**Tests for config.rs**:
- [ ] Test `VolumeConfig::validate()` with valid config
- [ ] Test rejection of empty `host_path`
- [ ] Test rejection of empty `container_path`
- [ ] Test rejection of relative paths
- [ ] Test `ServiceConfig::validate()` with volumes

### Step 1.3: Update Backend Trait

**File**: `crates/clnrm-core/src/backend/mod.rs`

**Backend Trait Extensions**:
- [ ] Add `supports_volumes()` method with default `false`
- [ ] Add `max_volumes()` method with default `Some(10)`
- [ ] Add doc comments explaining methods
- [ ] Ensure backward compatibility (default implementations)

### Step 1.4: Modify TestcontainerBackend

**File**: `crates/clnrm-core/src/backend/testcontainer.rs`

**Add Volume Support Fields**:
- [ ] Import `VolumeMount` and `VolumeValidator`
- [ ] Change `volume_mounts` from `Vec<(String, String)>` to `Vec<VolumeMount>`
- [ ] Add `volume_validator: Arc<VolumeValidator>` field

**Update Constructor**:
- [ ] Initialize `volume_mounts: Vec::new()`
- [ ] Initialize `volume_validator: Arc::new(VolumeValidator::default())`

**Update Builder Methods**:
- [ ] Modify `with_volume()` signature to return `Result<Self>`
- [ ] In `with_volume()`, create `VolumeMount::new()`
- [ ] In `with_volume()`, call `validator.validate()`
- [ ] In `with_volume()`, push to `volume_mounts`
- [ ] Add `with_volume_validator()` method
- [ ] Add `volumes()` getter method

**Implement Volume Mounting**:
- [ ] Find `execute_in_container()` method
- [ ] Locate TODO comment at lines 165-169
- [ ] Replace TODO with actual volume mounting code:
  ```rust
  for mount in &self.volume_mounts {
      use testcontainers::core::{Mount, AccessMode};
      let access_mode = if mount.is_read_only() {
          AccessMode::ReadOnly
      } else {
          AccessMode::ReadWrite
      };
      container_request = container_request.with_mount(
          Mount::bind_mount(
              mount.host_path().to_string_lossy().to_string(),
              mount.container_path().to_string_lossy().to_string(),
          ).with_access_mode(access_mode)
      );
  }
  ```

**Implement Backend Trait Methods**:
- [ ] Override `supports_volumes()` to return `true`
- [ ] Override `max_volumes()` to return `None` (unlimited)

**Tests for testcontainer.rs**:
- [ ] Test `with_volume()` with valid paths
- [ ] Test `with_volume()` rejects invalid paths
- [ ] Test `volumes()` getter
- [ ] Test `supports_volumes()` returns true
- [ ] Test `max_volumes()` returns None

---

## Phase 2: Plugin Integration (1-2 hours)

### Step 2.1: Update GenericContainerPlugin

**File**: `crates/clnrm-core/src/services/generic.rs`

**Add Volume Support**:
- [ ] Import `VolumeMount` and `VolumeConfig`
- [ ] Add `volumes: Vec<VolumeMount>` field to struct
- [ ] Initialize `volumes: Vec::new()` in `new()`

**Add Volume Builder Methods**:
- [ ] Implement `with_volume()` method returning `Result<Self>`
- [ ] Implement `with_volumes_from_config()` method
- [ ] Convert each `VolumeConfig` to `VolumeMount`
- [ ] Handle errors with proper context

**Update start() Method**:
- [ ] Locate container creation code
- [ ] After environment variables, add volume mounting:
  ```rust
  for mount in &self.volumes {
      use testcontainers::core::{Mount, AccessMode};
      let access_mode = if mount.is_read_only() {
          AccessMode::ReadOnly
      } else {
          AccessMode::ReadWrite
      };
      container_request = container_request.with_mount(
          Mount::bind_mount(
              mount.host_path().to_string_lossy().to_string(),
              mount.container_path().to_string_lossy().to_string(),
          ).with_access_mode(access_mode)
      );
  }
  ```
- [ ] Add volume info to metadata:
  ```rust
  for (i, mount) in self.volumes.iter().enumerate() {
      metadata.insert(format!("volume_{}_host", i), mount.host_path().display().to_string());
      metadata.insert(format!("volume_{}_container", i), mount.container_path().display().to_string());
      metadata.insert(format!("volume_{}_readonly", i), mount.is_read_only().to_string());
  }
  ```

**Tests for generic.rs**:
- [ ] Test `with_volume()` method
- [ ] Test `with_volumes_from_config()` method
- [ ] Test metadata includes volume information
- [ ] Test volume mounting in integration test

### Step 2.2: Create Service Helper Function

**File**: `crates/clnrm-core/src/services/mod.rs` (or new helper module)

**Service Creation Helper**:
- [ ] Create `create_service_from_config()` function
- [ ] Accept `name: &str` and `config: &ServiceConfig`
- [ ] Match on `config.plugin`
- [ ] For "alpine" or "generic_container":
  - [ ] Create `GenericContainerPlugin::new()`
  - [ ] Add environment variables if present
  - [ ] Add ports if present
  - [ ] Add volumes if present using `with_volumes_from_config()`
- [ ] Return `Result<Box<dyn ServicePlugin>>`
- [ ] Add error handling for unknown plugin types

**Tests for service helper**:
- [ ] Test service creation without volumes
- [ ] Test service creation with volumes
- [ ] Test error handling for missing image
- [ ] Test error handling for unknown plugin

---

## Phase 3: Testing (2-3 hours)

### Step 3.1: Unit Tests

**Create**: `crates/clnrm-core/tests/backend/volume_tests.rs`

**VolumeMount Tests**:
- [ ] Test successful creation with valid paths
- [ ] Test rejection of relative host path
- [ ] Test rejection of nonexistent host path
- [ ] Test rejection of relative container path
- [ ] Test rejection of paths with ".." components
- [ ] Test `to_mount_string()` with read-only
- [ ] Test `to_mount_string()` with read-write
- [ ] Test `from_config()` conversion
- [ ] Test path canonicalization

**VolumeValidator Tests**:
- [ ] Test default validator allows `/tmp`
- [ ] Test default validator allows `/var/tmp`
- [ ] Test default validator allows CWD
- [ ] Test custom validator with whitelist
- [ ] Test validator rejects unauthorized paths
- [ ] Test permissive validator allows all paths
- [ ] Test `validate_all()` with multiple mounts

**VolumeConfig Tests**:
- [ ] Test validation with valid config
- [ ] Test rejection of empty host_path
- [ ] Test rejection of empty container_path
- [ ] Test rejection of relative paths

**Run tests**:
- [ ] `cargo test --lib volume`
- [ ] Verify all tests pass
- [ ] Check coverage with `cargo tarpaulin` (if available)

### Step 3.2: Integration Tests

**Create**: `crates/clnrm-core/tests/integration/volume_integration_tests.rs`

**Container Volume Tests**:
- [ ] Test volume mount allows file creation
  - Create temp directory
  - Mount as read-write
  - Write file in container
  - Verify file exists on host
- [ ] Test read-only volume prevents writes
  - Create temp directory with file
  - Mount as read-only
  - Attempt write in container
  - Verify command fails
- [ ] Test multiple volumes on same container
  - Create two temp directories
  - Mount both
  - Write to both in container
  - Verify both files on host
- [ ] Test volume data persistence
  - Create container, write file
  - Stop container
  - Create new container with same volume
  - Verify file still exists

**Service Plugin Tests**:
- [ ] Test `GenericContainerPlugin` with volumes
- [ ] Test service creation from config with volumes
- [ ] Test volume metadata in ServiceHandle
- [ ] Test multiple services with different volumes

**Run tests**:
- [ ] `cargo test --test volume_integration_tests`
- [ ] Verify all tests pass
- [ ] Check test execution time (<2 minutes)

### Step 3.3: TOML Configuration Tests

**Create test files**:
- [ ] `tests/volumes/basic-volume.clnrm.toml`
- [ ] `tests/volumes/readonly-enforcement.clnrm.toml`
- [ ] `tests/volumes/multiple-volumes.clnrm.toml`
- [ ] `tests/volumes/data-processing.clnrm.toml`

**Test basic-volume.clnrm.toml**:
```toml
[test.metadata]
name = "basic_volume_test"

[services.test_service]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[services.test_service.volumes]]
host_path = "/tmp/test-data"
container_path = "/data"
read_only = false

[[steps]]
name = "write_file"
command = ["sh", "-c", "echo 'test' > /data/test.txt"]
service = "test_service"
```

**Test readonly-enforcement.clnrm.toml**:
```toml
[test.metadata]
name = "readonly_volume_test"

[services.test_service]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[services.test_service.volumes]]
host_path = "/tmp/readonly-test"
container_path = "/config"
read_only = true

[[steps]]
name = "write_should_fail"
command = ["sh", "-c", "echo 'fail' > /config/test.txt"]
service = "test_service"
expected_exit_code = 1
```

**Run TOML tests**:
- [ ] Setup test directories: `mkdir -p /tmp/test-data /tmp/readonly-test`
- [ ] Create test file: `echo "existing" > /tmp/readonly-test/test.txt`
- [ ] `cargo run -- run tests/volumes/`
- [ ] Verify all tests pass

### Step 3.4: Security Tests

**Create**: `crates/clnrm-core/tests/security/volume_security_tests.rs`

**Path Validation Security**:
- [ ] Test rejection of path with `..`
- [ ] Test rejection of symlink pointing outside allowed dirs
- [ ] Test rejection of relative paths
- [ ] Test rejection of nonexistent paths

**Whitelist Enforcement**:
- [ ] Test strict validator rejects `/etc`
- [ ] Test strict validator rejects `/root`
- [ ] Test strict validator allows whitelisted paths
- [ ] Test custom validator with specific whitelist

**Permission Tests**:
- [ ] Test read-only mount cannot be written to
- [ ] Test read-write mount allows writes
- [ ] Test file permissions preserved

**Run tests**:
- [ ] `cargo test --test volume_security_tests`
- [ ] Verify all tests pass

---

## Phase 4: Documentation (1 hour)

### Step 4.1: Update TOML Reference

**File**: `docs/TOML_REFERENCE.md`

- [ ] Add "Volume Configuration" section
- [ ] Document `[[services.SERVICE_NAME.volumes]]` syntax
- [ ] Explain `host_path`, `container_path`, `read_only` fields
- [ ] Provide examples:
  - Basic volume mount
  - Read-only volume
  - Multiple volumes
- [ ] Document requirements:
  - Host paths must be absolute
  - Host paths must exist
  - Container paths must be absolute
- [ ] Add troubleshooting section

### Step 4.2: Create Volume Guide

**File**: `docs/VOLUME_GUIDE.md`

**Content**:
- [ ] Introduction to volumes in clnrm
- [ ] Use cases:
  - Data persistence
  - Config file mounting
  - Test fixtures
  - Log collection
- [ ] Configuration examples
- [ ] Security best practices:
  - Use read-only for config files
  - Whitelist allowed directories
  - Avoid mounting sensitive directories
- [ ] Troubleshooting:
  - Path not found errors
  - Permission denied errors
  - Read-only violations
- [ ] Performance considerations
- [ ] Limitations (deferred features)

### Step 4.3: Update CLI Guide

**File**: `docs/CLI_GUIDE.md`

- [ ] Add note about volume support in test configurations
- [ ] Reference VOLUME_GUIDE.md for details
- [ ] Update example test configuration to show volumes

### Step 4.4: Create Example Tests

**Create examples**:
- [ ] `examples/volumes/data-persistence.clnrm.toml`
- [ ] `examples/volumes/config-mounting.clnrm.toml`
- [ ] `examples/volumes/test-fixtures.clnrm.toml`

**Documentation in examples**:
- [ ] Add README.md in `examples/volumes/`
- [ ] Explain each example
- [ ] Provide setup instructions

---

## Verification & Quality Assurance

### Code Quality

- [ ] Run `cargo fmt` - all code formatted
- [ ] Run `cargo clippy -- -D warnings` - zero warnings
- [ ] Run `cargo check` - no errors
- [ ] Run `cargo check --features otel` - no errors with features

### Testing

- [ ] Run `cargo test` - all tests pass
- [ ] Run `cargo test --features otel` - tests pass with features
- [ ] Run integration tests - all pass
- [ ] Run security tests - all pass
- [ ] Test coverage >80% (use `cargo tarpaulin` if available)

### Core Team Standards Compliance

- [ ] No `.unwrap()` in production code (check with `rg "\.unwrap\(\)"`)
- [ ] No `.expect()` in production code (check with `rg "\.expect\(\)"`)
- [ ] All functions return `Result<T, CleanroomError>`
- [ ] All trait methods are sync (ServicePlugin)
- [ ] Clear error messages with context
- [ ] No `println!` in production code (use `tracing` macros)
- [ ] Proper async handling (block_in_place for sync traits)

### Documentation

- [ ] All public APIs have doc comments
- [ ] TOML_REFERENCE.md updated
- [ ] VOLUME_GUIDE.md created
- [ ] Example tests created
- [ ] README.md mentions volume support

### Backward Compatibility

- [ ] Old tests still pass without volumes
- [ ] Backend trait has default implementations
- [ ] VolumeConfig is optional in ServiceConfig
- [ ] No breaking changes to existing APIs

---

## Pre-Merge Checklist

### Build & Test

- [ ] `cargo build --release` succeeds
- [ ] `cargo test` passes completely
- [ ] `cargo clippy -- -D warnings` shows zero issues
- [ ] `cargo run -- self-test` passes (if applicable)

### Documentation

- [ ] All docs updated and spell-checked
- [ ] Examples tested and working
- [ ] README.md updated with volume support mention

### Git

- [ ] All changes committed with clear messages
- [ ] Commit messages follow project conventions
- [ ] No debug code or commented-out code
- [ ] No temporary files committed

### Review

- [ ] Code reviewed by at least one other developer
- [ ] Design reviewed against architecture documents
- [ ] Security reviewed (path validation, whitelist)
- [ ] Performance impact assessed (<100ms overhead confirmed)

---

## Post-Merge Tasks

### Monitoring

- [ ] Watch for bug reports related to volumes
- [ ] Monitor performance impact in CI
- [ ] Track adoption in user tests

### Iteration

- [ ] Collect user feedback on volume support
- [ ] Identify common use cases for future enhancements
- [ ] Document feature requests for named volumes, tmpfs, etc.

### Communication

- [ ] Announce feature in release notes
- [ ] Update project README with volume support
- [ ] Create blog post or tutorial (optional)

---

## Troubleshooting Guide

### Common Issues During Implementation

**Issue**: `testcontainers::core::Mount` not found
- **Solution**: Check testcontainers version, update if needed
- **Command**: `cargo update -p testcontainers`

**Issue**: Volume mounting doesn't work in tests
- **Solution**: Ensure Docker daemon is running
- **Command**: `docker ps` to verify

**Issue**: Tests fail with permission denied
- **Solution**: Check file permissions on host paths
- **Command**: `chmod 755` on test directories

**Issue**: Clippy warnings about unused Result
- **Solution**: Properly handle all Results, no unused returns

**Issue**: Tests timeout
- **Solution**: Increase timeout or check Docker performance

---

## Time Estimates

| Phase | Task | Estimated Time | Running Total |
|-------|------|----------------|---------------|
| 1.1 | Create volume module | 1.5 hours | 1.5 hours |
| 1.2 | Update config validation | 0.5 hours | 2 hours |
| 1.3 | Update backend trait | 0.25 hours | 2.25 hours |
| 1.4 | Modify TestcontainerBackend | 0.75 hours | 3 hours |
| 2.1 | Update GenericContainerPlugin | 0.75 hours | 3.75 hours |
| 2.2 | Create service helper | 0.5 hours | 4.25 hours |
| 3.1 | Unit tests | 1 hour | 5.25 hours |
| 3.2 | Integration tests | 1 hour | 6.25 hours |
| 3.3 | TOML tests | 0.5 hours | 6.75 hours |
| 3.4 | Security tests | 0.5 hours | 7.25 hours |
| 4.1 | Update TOML reference | 0.25 hours | 7.5 hours |
| 4.2 | Create volume guide | 0.5 hours | 8 hours |
| 4.3 | Update CLI guide | 0.25 hours | 8.25 hours |
| 4.4 | Create examples | 0.25 hours | 8.5 hours |

**Total Estimated Time**: 8-9 hours (including buffer)

---

## Success Metrics

After implementation is complete, verify:

- [ ] Feature works as designed (all tests pass)
- [ ] Performance impact <100ms per test with 2-3 volumes
- [ ] Zero security issues (path validation, whitelist working)
- [ ] Documentation complete and accurate
- [ ] Code coverage >80%
- [ ] Core team standards 100% compliant
- [ ] Backward compatible (old tests still work)
- [ ] User-friendly error messages

---

## Approval Sign-off

**Design Approved**: ___________________ Date: ___________

**Implementation Complete**: ___________________ Date: ___________

**Tests Pass**: ___________________ Date: ___________

**Documentation Complete**: ___________________ Date: ___________

**Code Review Approved**: ___________________ Date: ___________

**Ready for Merge**: ___________________ Date: ___________

---

*This checklist is part of the clnrm volume support architecture design.*
