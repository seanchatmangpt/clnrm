# Volume Support Architecture Design (80/20)

**Version**: 1.0
**Date**: 2025-10-16
**Status**: Proposed

## Executive Summary

This document presents a minimal but complete volume support architecture for the clnrm framework, designed using the 80/20 principle to cover 80% of use cases with 20% of the implementation effort. The design maintains strict adherence to core team standards (no unwrap/expect, proper Result types, sync traits) and integrates seamlessly with existing components.

---

## Table of Contents

1. [Current Architecture Analysis](#current-architecture-analysis)
2. [80/20 Feature Scope](#8020-feature-scope)
3. [Core Data Structures](#core-data-structures)
4. [Backend Integration](#backend-integration)
5. [TOML Configuration Schema](#toml-configuration-schema)
6. [Service Plugin Integration](#service-plugin-integration)
7. [Implementation Strategy](#implementation-strategy)
8. [Security & Validation](#security--validation)
9. [Testing Strategy](#testing-strategy)

---

## 1. Current Architecture Analysis

### 1.1 Existing Components

**Backend Trait** (`src/backend/mod.rs`)
- `Backend` trait defines container operations
- `TestcontainerBackend` is primary implementation
- `Cmd` struct holds execution parameters
- Currently NO volume support in trait methods

**TestcontainerBackend** (`src/backend/testcontainer.rs`)
- Has `volume_mounts: Vec<(String, String)>` field (UNUSED)
- Builder method `with_volume()` exists but NOT CONNECTED
- Line 166-169: TODO comment shows volumes not implemented
- Testcontainers-rs library DOES support volume mounting

**ServicePlugin Trait** (`src/cleanroom.rs`)
- Sync trait methods (no async)
- `start()` returns `Result<ServiceHandle>`
- Plugins like `GenericContainerPlugin` and `SurrealDbPlugin` use testcontainers

**Configuration System** (`src/config.rs`)
- `VolumeConfig` struct ALREADY EXISTS (lines 117-125)
- `ServiceConfig` has `volumes: Option<Vec<VolumeConfig>>`
- Structure present but NOT USED in execution

### 1.2 Key Findings

✅ **Already Present**:
- Volume data structures defined
- TOML schema partially specified
- TestcontainerBackend has volume storage

❌ **Missing**:
- Actual volume mounting in container execution
- Volume validation and security checks
- Backend trait method for volume support
- Plugin integration with volumes
- Documentation and examples

---

## 2. 80/20 Feature Scope

### 2.1 INCLUDE (80% Use Cases)

**Essential Volume Types**:
1. **Bind Mounts** - Host directory → Container directory
2. **Read-Only vs Read-Write** - Simple boolean flag
3. **Service-Level Volumes** - Volumes specified in service config
4. **Path Validation** - Ensure paths exist and are safe

**Configuration**:
1. **TOML-based** - Declarative volume specifications
2. **Service-scoped** - Volumes tied to specific services
3. **Absolute paths** - Simple, explicit path handling

**Security**:
1. **Path sanitization** - Prevent directory traversal
2. **Read-only enforcement** - Protect host filesystem
3. **Whitelist directories** - Configurable allowed mount points

### 2.2 EXCLUDE (Defer to Future)

**Advanced Features** (20% of use cases, 80% of complexity):
- Named Docker volumes (anonymous volumes)
- Volume drivers (NFS, cloud storage)
- Volume labels and metadata
- Tmpfs mounts and in-memory volumes
- Volume sharing between containers
- Dynamic volume provisioning
- Volume lifecycle management (create/destroy)
- SELinux/AppArmor labeling
- Per-step volume mounting (only service-level for now)

---

## 3. Core Data Structures

### 3.1 Volume Configuration (ALREADY EXISTS - ENHANCE)

```rust
// src/config.rs - EXISTING, needs validation enhancement

/// Volume mount configuration
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct VolumeConfig {
    /// Host path (absolute path required)
    pub host_path: String,
    /// Container path (absolute path required)
    pub container_path: String,
    /// Whether volume is read-only (default: false)
    #[serde(default)]
    pub read_only: bool,
}
```

### 3.2 Volume Mount (NEW - Internal Representation)

```rust
// src/backend/volume.rs - NEW MODULE

use std::path::{Path, PathBuf};
use crate::error::{CleanroomError, Result};

/// Validated volume mount specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMount {
    /// Validated host path (absolute, exists)
    host_path: PathBuf,
    /// Container path (absolute)
    container_path: PathBuf,
    /// Read-only flag
    read_only: bool,
}

impl VolumeMount {
    /// Create and validate a new volume mount
    ///
    /// # Errors
    /// - Host path not absolute
    /// - Host path does not exist
    /// - Container path not absolute
    /// - Path contains security violations (e.g., ..)
    pub fn new(
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self> {
        let host_path = host_path.as_ref();
        let container_path = container_path.as_ref();

        // Validate host path is absolute
        if !host_path.is_absolute() {
            return Err(CleanroomError::validation_error(
                format!("Host path must be absolute: {:?}", host_path)
            ));
        }

        // Validate host path exists
        if !host_path.exists() {
            return Err(CleanroomError::validation_error(
                format!("Host path does not exist: {:?}", host_path)
            ));
        }

        // Validate container path is absolute
        if !container_path.is_absolute() {
            return Err(CleanroomError::validation_error(
                format!("Container path must be absolute: {:?}", container_path)
            ));
        }

        // Security: Check for path traversal attempts
        let host_canonical = host_path.canonicalize().map_err(|e| {
            CleanroomError::validation_error(
                format!("Failed to canonicalize host path: {}", e)
            )
        })?;

        if host_canonical.components().any(|c| {
            matches!(c, std::path::Component::ParentDir)
        }) {
            return Err(CleanroomError::security_error(
                "Host path contains parent directory traversal"
            ));
        }

        Ok(Self {
            host_path: host_canonical,
            container_path: container_path.to_path_buf(),
            read_only,
        })
    }

    /// Create from VolumeConfig with validation
    pub fn from_config(config: &crate::config::VolumeConfig) -> Result<Self> {
        Self::new(
            &config.host_path,
            &config.container_path,
            config.read_only,
        )
    }

    /// Get host path
    pub fn host_path(&self) -> &Path {
        &self.host_path
    }

    /// Get container path
    pub fn container_path(&self) -> &Path {
        &self.container_path
    }

    /// Check if read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Format for Docker/testcontainers
    /// Returns: "/host/path:/container/path:ro" or "/host/path:/container/path:rw"
    pub fn to_mount_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.host_path.display(),
            self.container_path.display(),
            if self.read_only { "ro" } else { "rw" }
        )
    }
}
```

### 3.3 Volume Validator (NEW - Security Layer)

```rust
// src/backend/volume.rs - continued

/// Volume security validator
pub struct VolumeValidator {
    /// Allowed host directories for mounting (whitelist)
    allowed_host_dirs: Vec<PathBuf>,
    /// Whether to allow all directories (permissive mode)
    permissive: bool,
}

impl VolumeValidator {
    /// Create strict validator with whitelist
    pub fn new(allowed_dirs: Vec<PathBuf>) -> Self {
        Self {
            allowed_host_dirs: allowed_dirs,
            permissive: false,
        }
    }

    /// Create permissive validator (allows all paths)
    pub fn permissive() -> Self {
        Self {
            allowed_host_dirs: Vec::new(),
            permissive: true,
        }
    }

    /// Validate a volume mount against security policy
    pub fn validate(&self, mount: &VolumeMount) -> Result<()> {
        if self.permissive {
            return Ok(());
        }

        // Check if host path is within allowed directories
        let host_path = mount.host_path();

        for allowed_dir in &self.allowed_host_dirs {
            if host_path.starts_with(allowed_dir) {
                return Ok(());
            }
        }

        Err(CleanroomError::security_error(
            format!(
                "Host path {:?} not in allowed directories. Allowed: {:?}",
                host_path,
                self.allowed_host_dirs
            )
        ))
    }

    /// Validate multiple volume mounts
    pub fn validate_all(&self, mounts: &[VolumeMount]) -> Result<()> {
        for mount in mounts {
            self.validate(mount)?;
        }
        Ok(())
    }
}

impl Default for VolumeValidator {
    fn default() -> Self {
        // Default: Allow common safe directories
        Self::new(vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/var/tmp"),
            // Add current working directory
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ])
    }
}
```

---

## 4. Backend Integration

### 4.1 Backend Trait Enhancement

```rust
// src/backend/mod.rs - MODIFY EXISTING

pub trait Backend: Send + Sync + std::fmt::Debug {
    /// Run a command in the backend
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult>;

    /// Get the name of the backend
    fn name(&self) -> &str;

    /// Check if the backend is available
    fn is_available(&self) -> bool;

    /// Check if the backend supports hermetic execution
    fn supports_hermetic(&self) -> bool;

    /// Check if the backend supports deterministic execution
    fn supports_deterministic(&self) -> bool;

    // NEW METHODS (with default implementations for backward compatibility)

    /// Check if the backend supports volume mounting
    fn supports_volumes(&self) -> bool {
        false  // Default: no support
    }

    /// Get maximum number of volumes supported (None = unlimited)
    fn max_volumes(&self) -> Option<usize> {
        Some(10)  // Reasonable default
    }
}
```

### 4.2 TestcontainerBackend Enhancement

```rust
// src/backend/testcontainer.rs - MODIFY EXISTING

use crate::backend::volume::{VolumeMount, VolumeValidator};

#[derive(Debug, Clone)]
pub struct TestcontainerBackend {
    // ... existing fields ...

    /// Volume mounts for the container (ALREADY EXISTS - change type)
    volume_mounts: Vec<VolumeMount>,  // Changed from Vec<(String, String)>

    /// Volume validator for security checks (NEW)
    volume_validator: Arc<VolumeValidator>,
}

impl TestcontainerBackend {
    pub fn new(image: impl Into<String>) -> Result<Self> {
        // ... existing code ...
        Ok(Self {
            // ... existing fields ...
            volume_mounts: Vec::new(),
            volume_validator: Arc::new(VolumeValidator::default()),
        })
    }

    /// Add volume mount (MODIFY EXISTING)
    pub fn with_volume(
        mut self,
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self> {
        let mount = VolumeMount::new(host_path, container_path, read_only)?;
        self.volume_validator.validate(&mount)?;
        self.volume_mounts.push(mount);
        Ok(self)
    }

    /// Set custom volume validator
    pub fn with_volume_validator(mut self, validator: VolumeValidator) -> Self {
        self.volume_validator = Arc::new(validator);
        self
    }

    /// Get configured volume mounts
    pub fn volumes(&self) -> &[VolumeMount] {
        &self.volume_mounts
    }
}

impl Backend for TestcontainerBackend {
    // ... existing methods ...

    fn supports_volumes(&self) -> bool {
        true  // Testcontainers supports volumes
    }

    fn max_volumes(&self) -> Option<usize> {
        None  // Unlimited
    }
}
```

### 4.3 Volume Mounting in Container Execution

```rust
// src/backend/testcontainer.rs - MODIFY execute_in_container()

impl TestcontainerBackend {
    fn execute_in_container(&self, cmd: &Cmd) -> Result<RunResult> {
        // ... existing setup code ...

        // Build container request with all configurations
        let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
            image.into();

        // ... existing env vars code ...

        // REPLACE LINES 165-169 (TODO comment) with actual implementation:

        // Add volume mounts using testcontainers API
        for mount in &self.volume_mounts {
            use testcontainers::core::{Mount, AccessMode};

            let access_mode = if mount.is_read_only() {
                AccessMode::ReadOnly
            } else {
                AccessMode::ReadWrite
            };

            container_request = container_request.with_mount(Mount::bind_mount(
                mount.host_path().to_string_lossy().to_string(),
                mount.container_path().to_string_lossy().to_string(),
            ).with_access_mode(access_mode));
        }

        // ... rest of existing code ...
    }
}
```

---

## 5. TOML Configuration Schema

### 5.1 Service-Level Volume Configuration

```toml
# Example: tests/volume-example.clnrm.toml

[test.metadata]
name = "volume_integration_test"
description = "Test volume mounting with data persistence"
timeout = "120s"

[services.data_processor]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

# Volume configuration (ALREADY SUPPORTED IN SCHEMA)
[[services.data_processor.volumes]]
host_path = "/tmp/test-data"
container_path = "/data"
read_only = false

[[services.data_processor.volumes]]
host_path = "/tmp/test-config"
container_path = "/config"
read_only = true  # Config files should be read-only

[[steps]]
name = "write_data"
command = ["sh", "-c", "echo 'test-data' > /data/output.txt"]
service = "data_processor"

[[steps]]
name = "read_data"
command = ["cat", "/data/output.txt"]
service = "data_processor"
expected_output_regex = "test-data"

[[steps]]
name = "verify_readonly"
command = ["sh", "-c", "echo 'should-fail' > /config/readonly.txt"]
service = "data_processor"
expected_exit_code = 1  # Should fail due to read-only mount
```

### 5.2 Configuration Validation Enhancement

```rust
// src/config.rs - ADD VALIDATION

impl VolumeConfig {
    /// Validate volume configuration
    pub fn validate(&self) -> Result<()> {
        // Validate paths are not empty
        if self.host_path.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Volume host_path cannot be empty"
            ));
        }

        if self.container_path.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Volume container_path cannot be empty"
            ));
        }

        // Validate paths are absolute
        let host_path = Path::new(&self.host_path);
        if !host_path.is_absolute() {
            return Err(CleanroomError::validation_error(
                format!("Volume host_path must be absolute: {}", self.host_path)
            ));
        }

        let container_path = Path::new(&self.container_path);
        if !container_path.is_absolute() {
            return Err(CleanroomError::validation_error(
                format!("Volume container_path must be absolute: {}", self.container_path)
            ));
        }

        Ok(())
    }
}

impl ServiceConfig {
    pub fn validate(&self) -> Result<()> {
        // ... existing validation ...

        // ADD: Validate volumes if present
        if let Some(ref volumes) = self.volumes {
            for (i, volume) in volumes.iter().enumerate() {
                volume.validate().map_err(|e| {
                    CleanroomError::validation_error(
                        format!("Volume {}: {}", i, e)
                    )
                })?;
            }
        }

        Ok(())
    }
}
```

---

## 6. Service Plugin Integration

### 6.1 GenericContainerPlugin Enhancement

```rust
// src/services/generic.rs - MODIFY EXISTING

pub struct GenericContainerPlugin {
    name: String,
    image: String,
    tag: String,
    container_id: Arc<RwLock<Option<String>>>,
    env_vars: HashMap<String, String>,
    ports: Vec<u16>,
    volumes: Vec<VolumeMount>,  // NEW FIELD
}

impl GenericContainerPlugin {
    pub fn new(name: &str, image: &str) -> Self {
        // ... existing code ...
        Self {
            // ... existing fields ...
            volumes: Vec::new(),  // NEW
        }
    }

    /// Add volume mount to plugin (NEW METHOD)
    pub fn with_volume(
        mut self,
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self> {
        let mount = VolumeMount::new(host_path, container_path, read_only)?;
        self.volumes.push(mount);
        Ok(self)
    }

    /// Set volumes from configuration (NEW METHOD)
    pub fn with_volumes_from_config(
        mut self,
        volume_configs: &[crate::config::VolumeConfig],
    ) -> Result<Self> {
        for config in volume_configs {
            let mount = VolumeMount::from_config(config)?;
            self.volumes.push(mount);
        }
        Ok(self)
    }
}

impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // ... existing setup code ...

                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // ... existing env vars and ports ...

                // ADD: Mount volumes
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

                // ... existing start and metadata code ...

                // ADD: Volume information to metadata
                for (i, mount) in self.volumes.iter().enumerate() {
                    metadata.insert(
                        format!("volume_{}_host", i),
                        mount.host_path().display().to_string()
                    );
                    metadata.insert(
                        format!("volume_{}_container", i),
                        mount.container_path().display().to_string()
                    );
                    metadata.insert(
                        format!("volume_{}_readonly", i),
                        mount.is_read_only().to_string()
                    );
                }

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    // ... existing stop() and health_check() methods ...
}
```

### 6.2 Service Creation from Configuration

```rust
// src/services/mod.rs or similar - NEW HELPER FUNCTION

use crate::config::{ServiceConfig, VolumeConfig};
use crate::backend::volume::VolumeMount;

/// Create a service plugin from configuration with volume support
pub fn create_service_from_config(
    name: &str,
    config: &ServiceConfig,
) -> Result<Box<dyn ServicePlugin>> {
    match config.plugin.as_str() {
        "alpine" | "generic_container" => {
            let image = config.image.as_ref().ok_or_else(|| {
                CleanroomError::config_error("Generic container requires image")
            })?;

            let mut plugin = GenericContainerPlugin::new(name, image);

            // Add environment variables
            if let Some(ref env) = config.env {
                for (key, value) in env {
                    plugin = plugin.with_env(key, value);
                }
            }

            // Add ports
            if let Some(ref ports) = config.ports {
                for port in ports {
                    plugin = plugin.with_port(*port);
                }
            }

            // ADD: Mount volumes
            if let Some(ref volumes) = config.volumes {
                plugin = plugin.with_volumes_from_config(volumes)?;
            }

            Ok(Box::new(plugin))
        }
        // ... other plugin types ...
        _ => Err(CleanroomError::config_error(
            format!("Unknown service plugin: {}", config.plugin)
        ))
    }
}
```

---

## 7. Implementation Strategy

### 7.1 Implementation Phases

**Phase 1: Core Volume Support** (2-3 hours)
1. Create `src/backend/volume.rs` with `VolumeMount` and `VolumeValidator`
2. Update `VolumeConfig` validation in `src/config.rs`
3. Modify `TestcontainerBackend` to use `VolumeMount`
4. Add actual volume mounting in `execute_in_container()`

**Phase 2: Plugin Integration** (1-2 hours)
1. Update `GenericContainerPlugin` with volume support
2. Create helper function for service creation from config
3. Wire up volume configuration in service registry

**Phase 3: Testing** (2-3 hours)
1. Unit tests for `VolumeMount` and `VolumeValidator`
2. Integration tests for volume mounting
3. Security tests for path validation
4. TOML configuration tests

**Phase 4: Documentation** (1 hour)
1. Update TOML_REFERENCE.md with volume examples
2. Add volume guide to documentation
3. Create example test files

### 7.2 File Changes Summary

**New Files**:
- `crates/clnrm-core/src/backend/volume.rs` - Volume types and validation
- `tests/volumes/basic-volume.clnrm.toml` - Example test
- `docs/VOLUME_GUIDE.md` - User documentation

**Modified Files**:
- `crates/clnrm-core/src/backend/mod.rs` - Add volume support to Backend trait
- `crates/clnrm-core/src/backend/testcontainer.rs` - Implement volume mounting
- `crates/clnrm-core/src/config.rs` - Add VolumeConfig validation
- `crates/clnrm-core/src/services/generic.rs` - Add volume support to plugin
- `crates/clnrm-core/src/services/mod.rs` - Service creation with volumes

### 7.3 Dependencies

**No new dependencies required!**
- `testcontainers` already supports volume mounting
- Standard library `Path` and `PathBuf` for path handling
- Existing error types for validation

---

## 8. Security & Validation

### 8.1 Security Considerations

**Path Safety**:
1. ✅ Require absolute paths (no relative paths)
2. ✅ Canonicalize host paths to resolve symlinks
3. ✅ Reject paths with `..` components (traversal)
4. ✅ Validate host paths exist before mounting
5. ✅ Whitelist allowed host directories

**Permission Safety**:
1. ✅ Read-only flag enforcement at container level
2. ✅ No privilege escalation via volume mounts
3. ✅ Container runs as non-root user (testcontainers default)

**Resource Safety**:
1. ✅ Limit number of volumes per service (configurable)
2. ✅ No anonymous volumes (explicit paths only)
3. ✅ No volume drivers (bind mounts only)

### 8.2 Validation Rules

**Host Path Validation**:
```rust
- Must be absolute path
- Must exist on host filesystem
- Must be readable by current user
- Must not contain ".." components
- Must be in allowed directory whitelist (if strict mode)
```

**Container Path Validation**:
```rust
- Must be absolute path
- Must not conflict with system directories (/sys, /proc, /dev)
- Should be unique (no duplicate mounts to same container path)
```

**Configuration Validation**:
```rust
- VolumeConfig fields must not be empty
- Paths must pass path validation
- Multiple volumes must not conflict
```

### 8.3 Error Messages

```rust
// Good error messages for debugging

CleanroomError::validation_error(
    "Host path must be absolute: 'relative/path'"
)

CleanroomError::validation_error(
    "Host path does not exist: '/nonexistent/path'"
)

CleanroomError::security_error(
    "Host path '/etc/shadow' not in allowed directories. Allowed: ['/tmp', '/var/tmp']"
)

CleanroomError::validation_error(
    "Volume 0: Host path must be absolute: 'data'"
)
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
// tests/backend/volume_tests.rs

#[test]
fn test_volume_mount_creation_with_valid_paths() {
    let temp_dir = std::env::temp_dir();
    let result = VolumeMount::new(&temp_dir, "/container/path", false);
    assert!(result.is_ok());
}

#[test]
fn test_volume_mount_rejects_relative_host_path() {
    let result = VolumeMount::new("relative/path", "/container/path", false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute"));
}

#[test]
fn test_volume_mount_rejects_nonexistent_host_path() {
    let result = VolumeMount::new("/nonexistent/path", "/container/path", false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn test_volume_mount_rejects_relative_container_path() {
    let temp_dir = std::env::temp_dir();
    let result = VolumeMount::new(&temp_dir, "relative/path", false);
    assert!(result.is_err());
}

#[test]
fn test_volume_mount_to_mount_string_format() {
    let temp_dir = std::env::temp_dir();
    let mount = VolumeMount::new(&temp_dir, "/data", true).unwrap();
    let mount_str = mount.to_mount_string();
    assert!(mount_str.ends_with(":/data:ro"));
}

#[test]
fn test_volume_validator_default_allows_tmp() {
    let validator = VolumeValidator::default();
    let temp_dir = std::env::temp_dir();
    let mount = VolumeMount::new(&temp_dir, "/data", false).unwrap();
    assert!(validator.validate(&mount).is_ok());
}

#[test]
fn test_volume_validator_strict_rejects_unauthorized_path() {
    let validator = VolumeValidator::new(vec![PathBuf::from("/allowed")]);
    let temp_dir = std::env::temp_dir();
    let mount = VolumeMount::new(&temp_dir, "/data", false).unwrap();
    assert!(validator.validate(&mount).is_err());
}

#[test]
fn test_volume_config_validation_empty_host_path() {
    let config = VolumeConfig {
        host_path: "".to_string(),
        container_path: "/data".to_string(),
        read_only: false,
    };
    assert!(config.validate().is_err());
}
```

### 9.2 Integration Tests

```rust
// tests/integration/volume_integration_tests.rs

#[tokio::test]
async fn test_volume_mount_allows_file_creation() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let host_path = temp_dir.path();

    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume(host_path, "/data", false)?;

    let cmd = Cmd::new("sh")
        .arg("-c")
        .arg("echo 'test-content' > /data/test.txt");

    let result = backend.run_cmd(cmd)?;
    assert!(result.success());

    // Verify file was created on host
    let file_path = host_path.join("test.txt");
    assert!(file_path.exists());

    let content = std::fs::read_to_string(file_path)?;
    assert_eq!(content.trim(), "test-content");

    Ok(())
}

#[tokio::test]
async fn test_readonly_volume_prevents_writes() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let host_path = temp_dir.path();

    // Create a test file
    std::fs::write(host_path.join("readonly.txt"), "readonly-content")?;

    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume(host_path, "/data", true)?;  // READ-ONLY

    let cmd = Cmd::new("sh")
        .arg("-c")
        .arg("echo 'should-fail' > /data/readonly.txt");

    let result = backend.run_cmd(cmd)?;
    assert!(result.failed());  // Should fail due to read-only

    Ok(())
}

#[tokio::test]
async fn test_multiple_volumes_on_same_container() -> Result<()> {
    let temp_dir1 = tempfile::tempdir()?;
    let temp_dir2 = tempfile::tempdir()?;

    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume(temp_dir1.path(), "/data1", false)?
        .with_volume(temp_dir2.path(), "/data2", false)?;

    let cmd = Cmd::new("sh")
        .arg("-c")
        .arg("echo 'data1' > /data1/file.txt && echo 'data2' > /data2/file.txt");

    let result = backend.run_cmd(cmd)?;
    assert!(result.success());

    // Verify both files created
    assert!(temp_dir1.path().join("file.txt").exists());
    assert!(temp_dir2.path().join("file.txt").exists());

    Ok(())
}
```

### 9.3 TOML Configuration Tests

```toml
# tests/volumes/readonly-enforcement.clnrm.toml

[test.metadata]
name = "readonly_volume_enforcement"
description = "Verify read-only volumes prevent writes"

[services.readonly_test]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[services.readonly_test.volumes]]
host_path = "/tmp/readonly-test"
container_path = "/config"
read_only = true

[[steps]]
name = "read_should_succeed"
command = ["cat", "/config/test.txt"]
service = "readonly_test"

[[steps]]
name = "write_should_fail"
command = ["sh", "-c", "echo 'fail' > /config/test.txt"]
service = "readonly_test"
expected_exit_code = 1
```

---

## 10. Documentation Requirements

### 10.1 TOML Reference Update

Add to `docs/TOML_REFERENCE.md`:

```markdown
### Volume Configuration

Volumes allow mounting host directories into containers for data persistence and sharing.

#### Basic Volume Mount

\`\`\`toml
[services.my_service]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[services.my_service.volumes]]
host_path = "/tmp/data"
container_path = "/data"
read_only = false
\`\`\`

#### Read-Only Volume Mount

\`\`\`toml
[[services.my_service.volumes]]
host_path = "/etc/config"
container_path = "/config"
read_only = true
\`\`\`

#### Multiple Volumes

\`\`\`toml
[[services.my_service.volumes]]
host_path = "/tmp/input"
container_path = "/input"
read_only = true

[[services.my_service.volumes]]
host_path = "/tmp/output"
container_path = "/output"
read_only = false
\`\`\`

#### Volume Requirements

- **Host paths** must be absolute paths
- **Host paths** must exist before test execution
- **Container paths** must be absolute paths
- **Read-only flag** defaults to `false` (read-write)
```

### 10.2 User Guide

Create `docs/VOLUME_GUIDE.md` with:
- Common volume use cases
- Security best practices
- Troubleshooting guide
- Performance considerations

---

## 11. Success Criteria

### 11.1 Functional Requirements

- ✅ Support bind mount volumes in TOML configuration
- ✅ Support read-only and read-write volumes
- ✅ Validate host and container paths
- ✅ Integrate with TestcontainerBackend
- ✅ Support multiple volumes per service
- ✅ Security validation with whitelisting

### 11.2 Non-Functional Requirements

- ✅ No `.unwrap()` or `.expect()` in production code
- ✅ All functions return `Result<T, CleanroomError>`
- ✅ Trait methods remain sync (use `block_in_place` internally)
- ✅ Clear error messages for debugging
- ✅ Comprehensive test coverage (>80%)
- ✅ Zero new external dependencies

### 11.3 Definition of Done

- [ ] All code passes `cargo clippy -- -D warnings`
- [ ] All tests pass: `cargo test`
- [ ] Integration tests demonstrate volume functionality
- [ ] TOML reference documentation updated
- [ ] Example test files created
- [ ] Security validation tested
- [ ] Framework self-test includes volume tests

---

## 12. Risk Analysis

### 12.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Testcontainers API changes | Medium | Low | Use stable API methods, add version constraints |
| Path validation edge cases | High | Medium | Comprehensive unit tests, fuzzing |
| Permission issues on host | Medium | Medium | Clear error messages, documentation |
| Volume conflicts between services | Low | Low | Validation prevents duplicate mounts |

### 12.2 Security Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Host filesystem exposure | High | Low | Whitelist validation, read-only enforcement |
| Path traversal attacks | High | Low | Canonicalization, parent dir rejection |
| Symlink exploitation | Medium | Low | Canonicalize paths before validation |
| Resource exhaustion | Low | Low | Limit max volumes per service |

---

## 13. Future Extensions (Post-80/20)

### 13.1 Named Volumes
- Docker named volumes (not bind mounts)
- Volume lifecycle management (create/destroy)
- Volume reuse across tests

### 13.2 Advanced Features
- Tmpfs mounts for in-memory volumes
- Volume drivers (cloud storage, NFS)
- SELinux/AppArmor labels
- Volume labels and metadata

### 13.3 Per-Step Volumes
- Step-specific volume mounts
- Dynamic volume mounting during test execution
- Volume cleanup between steps

---

## Appendix A: Code Examples

### A.1 Complete Example Test

```toml
# tests/volumes/data-processing.clnrm.toml

[test.metadata]
name = "data_processing_pipeline"
description = "Test data processing with input/output volumes"
timeout = "120s"

[services.processor]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[services.processor.volumes]]
host_path = "/tmp/test-input"
container_path = "/input"
read_only = true

[[services.processor.volumes]]
host_path = "/tmp/test-output"
container_path = "/output"
read_only = false

[[steps]]
name = "process_data"
command = ["sh", "-c", "cat /input/data.txt | tr '[:lower:]' '[:upper:]' > /output/result.txt"]
service = "processor"

[[steps]]
name = "verify_output"
command = ["cat", "/output/result.txt"]
service = "processor"
expected_output_regex = "[A-Z]+"
```

### A.2 Rust Usage Example

```rust
use clnrm_core::backend::TestcontainerBackend;
use clnrm_core::backend::volume::{VolumeMount, VolumeValidator};
use clnrm_core::services::GenericContainerPlugin;

// Create backend with volumes
let backend = TestcontainerBackend::new("alpine:latest")?
    .with_volume("/tmp/data", "/data", false)?
    .with_volume("/tmp/config", "/config", true)?;

// Create service plugin with volumes
let plugin = GenericContainerPlugin::new("my_service", "alpine:latest")
    .with_volume("/tmp/data", "/data", false)?;

// Use custom validator
let validator = VolumeValidator::new(vec![
    PathBuf::from("/tmp"),
    PathBuf::from("/var/tmp"),
]);

let backend = TestcontainerBackend::new("alpine:latest")?
    .with_volume_validator(validator);
```

---

## Appendix B: API Reference

### B.1 VolumeMount API

```rust
impl VolumeMount {
    pub fn new(
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self>;

    pub fn from_config(config: &VolumeConfig) -> Result<Self>;
    pub fn host_path(&self) -> &Path;
    pub fn container_path(&self) -> &Path;
    pub fn is_read_only(&self) -> bool;
    pub fn to_mount_string(&self) -> String;
}
```

### B.2 VolumeValidator API

```rust
impl VolumeValidator {
    pub fn new(allowed_dirs: Vec<PathBuf>) -> Self;
    pub fn permissive() -> Self;
    pub fn validate(&self, mount: &VolumeMount) -> Result<()>;
    pub fn validate_all(&self, mounts: &[VolumeMount]) -> Result<()>;
}
```

### B.3 TestcontainerBackend Extensions

```rust
impl TestcontainerBackend {
    pub fn with_volume(
        self,
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self>;

    pub fn with_volume_validator(self, validator: VolumeValidator) -> Self;
    pub fn volumes(&self) -> &[VolumeMount];
}
```

---

## Conclusion

This architecture design provides a minimal, complete, and secure volume support system for clnrm that:

1. **Covers 80% of use cases** with bind mounts and read-only/read-write modes
2. **Maintains core team standards** with proper error handling and sync traits
3. **Integrates seamlessly** with existing Backend trait and ServicePlugin system
4. **Requires zero new dependencies** using existing testcontainers functionality
5. **Provides strong security** with path validation and whitelisting
6. **Is fully testable** with comprehensive unit and integration tests

**Implementation effort**: 6-8 hours total for a production-ready feature.

**Next steps**: Review design → Approve → Implement Phase 1 → Test → Document → Ship
