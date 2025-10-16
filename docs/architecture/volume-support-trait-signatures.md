# Volume Support - Trait Signatures and Integration Points

**Version**: 1.0
**Date**: 2025-10-16

This document provides the complete trait method signatures and integration points for volume support.

---

## 1. New Module: `backend::volume`

**File**: `crates/clnrm-core/src/backend/volume.rs`

### 1.1 VolumeMount Structure

```rust
/// Validated volume mount specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMount {
    host_path: PathBuf,
    container_path: PathBuf,
    read_only: bool,
}

impl VolumeMount {
    /// Create and validate a new volume mount
    ///
    /// # Arguments
    /// * `host_path` - Absolute path on host filesystem (must exist)
    /// * `container_path` - Absolute path in container
    /// * `read_only` - Whether volume is read-only
    ///
    /// # Errors
    /// - Host path not absolute
    /// - Host path does not exist
    /// - Container path not absolute
    /// - Path contains security violations
    ///
    /// # Examples
    /// ```rust
    /// let mount = VolumeMount::new("/tmp/data", "/data", false)?;
    /// ```
    pub fn new(
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self>;

    /// Create from VolumeConfig with validation
    pub fn from_config(config: &crate::config::VolumeConfig) -> Result<Self>;

    /// Get host path (always absolute, canonical)
    pub fn host_path(&self) -> &Path;

    /// Get container path (always absolute)
    pub fn container_path(&self) -> &Path;

    /// Check if read-only
    pub fn is_read_only(&self) -> bool;

    /// Format for Docker/testcontainers
    /// Returns: "/host/path:/container/path:ro" or "/host/path:/container/path:rw"
    pub fn to_mount_string(&self) -> String;
}
```

### 1.2 VolumeValidator Structure

```rust
/// Volume security validator with whitelist support
pub struct VolumeValidator {
    allowed_host_dirs: Vec<PathBuf>,
    permissive: bool,
}

impl VolumeValidator {
    /// Create strict validator with whitelist
    ///
    /// # Arguments
    /// * `allowed_dirs` - List of allowed host directories for mounting
    ///
    /// # Examples
    /// ```rust
    /// let validator = VolumeValidator::new(vec![
    ///     PathBuf::from("/tmp"),
    ///     PathBuf::from("/var/tmp"),
    /// ]);
    /// ```
    pub fn new(allowed_dirs: Vec<PathBuf>) -> Self;

    /// Create permissive validator (allows all paths)
    ///
    /// # Warning
    /// Only use in trusted environments. Bypasses security checks.
    pub fn permissive() -> Self;

    /// Validate a volume mount against security policy
    ///
    /// # Errors
    /// - Host path not in allowed directories (strict mode)
    pub fn validate(&self, mount: &VolumeMount) -> Result<()>;

    /// Validate multiple volume mounts
    pub fn validate_all(&self, mounts: &[VolumeMount]) -> Result<()>;
}

impl Default for VolumeValidator {
    /// Default validator allows: /tmp, /var/tmp, and current working directory
    fn default() -> Self;
}
```

---

## 2. Backend Trait Extensions

**File**: `crates/clnrm-core/src/backend/mod.rs`

### 2.1 Backend Trait - New Methods

```rust
pub trait Backend: Send + Sync + std::fmt::Debug {
    // ... existing methods ...

    /// Check if the backend supports volume mounting
    ///
    /// # Returns
    /// `true` if backend can mount volumes, `false` otherwise
    ///
    /// # Default Implementation
    /// Returns `false` for backward compatibility
    fn supports_volumes(&self) -> bool {
        false
    }

    /// Get maximum number of volumes supported
    ///
    /// # Returns
    /// - `Some(n)` - Maximum number of volumes
    /// - `None` - Unlimited volumes
    ///
    /// # Default Implementation
    /// Returns `Some(10)` as reasonable default
    fn max_volumes(&self) -> Option<usize> {
        Some(10)
    }
}
```

---

## 3. TestcontainerBackend Extensions

**File**: `crates/clnrm-core/src/backend/testcontainer.rs`

### 3.1 TestcontainerBackend Structure Changes

```rust
#[derive(Debug, Clone)]
pub struct TestcontainerBackend {
    // ... existing fields ...

    /// Volume mounts for the container
    /// CHANGED FROM: Vec<(String, String)>
    /// CHANGED TO:   Vec<VolumeMount>
    volume_mounts: Vec<VolumeMount>,

    /// Volume validator for security checks (NEW)
    volume_validator: Arc<VolumeValidator>,
}
```

### 3.2 TestcontainerBackend - New Methods

```rust
impl TestcontainerBackend {
    /// Add volume mount (MODIFIED)
    ///
    /// # Arguments
    /// * `host_path` - Path on host filesystem (must be absolute and exist)
    /// * `container_path` - Path in container (must be absolute)
    /// * `read_only` - Whether volume is read-only
    ///
    /// # Errors
    /// - Path validation errors
    /// - Security policy violations
    ///
    /// # Examples
    /// ```rust
    /// let backend = TestcontainerBackend::new("alpine:latest")?
    ///     .with_volume("/tmp/data", "/data", false)?
    ///     .with_volume("/tmp/config", "/config", true)?;
    /// ```
    pub fn with_volume(
        mut self,
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self>;

    /// Set custom volume validator (NEW)
    ///
    /// # Arguments
    /// * `validator` - Custom volume validator with security policy
    ///
    /// # Examples
    /// ```rust
    /// let validator = VolumeValidator::new(vec![PathBuf::from("/allowed")]);
    /// let backend = TestcontainerBackend::new("alpine:latest")?
    ///     .with_volume_validator(validator);
    /// ```
    pub fn with_volume_validator(mut self, validator: VolumeValidator) -> Self;

    /// Get configured volume mounts (NEW)
    ///
    /// # Returns
    /// Slice of configured volume mounts
    pub fn volumes(&self) -> &[VolumeMount];
}
```

### 3.3 Backend Trait Implementation

```rust
impl Backend for TestcontainerBackend {
    // ... existing implementations ...

    fn supports_volumes(&self) -> bool {
        true  // Testcontainers supports volumes
    }

    fn max_volumes(&self) -> Option<usize> {
        None  // Unlimited
    }
}
```

---

## 4. Configuration Extensions

**File**: `crates/clnrm-core/src/config.rs`

### 4.1 VolumeConfig - New Method

```rust
impl VolumeConfig {
    /// Validate volume configuration
    ///
    /// # Errors
    /// - Empty paths
    /// - Non-absolute paths
    ///
    /// # Examples
    /// ```rust
    /// let config = VolumeConfig {
    ///     host_path: "/tmp/data".to_string(),
    ///     container_path: "/data".to_string(),
    ///     read_only: false,
    /// };
    /// config.validate()?;
    /// ```
    pub fn validate(&self) -> Result<()>;
}
```

### 4.2 ServiceConfig - Enhanced Validation

```rust
impl ServiceConfig {
    pub fn validate(&self) -> Result<()> {
        // ... existing validation ...

        // NEW: Validate volumes if present
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

## 5. Service Plugin Extensions

**File**: `crates/clnrm-core/src/services/generic.rs`

### 5.1 GenericContainerPlugin Structure Changes

```rust
pub struct GenericContainerPlugin {
    // ... existing fields ...

    /// Volume mounts for the container (NEW)
    volumes: Vec<VolumeMount>,
}
```

### 5.2 GenericContainerPlugin - New Methods

```rust
impl GenericContainerPlugin {
    /// Add volume mount to plugin (NEW)
    ///
    /// # Arguments
    /// * `host_path` - Path on host filesystem
    /// * `container_path` - Path in container
    /// * `read_only` - Whether volume is read-only
    ///
    /// # Errors
    /// - Path validation errors
    ///
    /// # Examples
    /// ```rust
    /// let plugin = GenericContainerPlugin::new("my_service", "alpine:latest")
    ///     .with_volume("/tmp/data", "/data", false)?;
    /// ```
    pub fn with_volume(
        mut self,
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self>;

    /// Set volumes from configuration (NEW)
    ///
    /// # Arguments
    /// * `volume_configs` - Volume configurations from TOML
    ///
    /// # Errors
    /// - Invalid volume configurations
    ///
    /// # Examples
    /// ```rust
    /// let plugin = GenericContainerPlugin::new("my_service", "alpine:latest")
    ///     .with_volumes_from_config(&service_config.volumes)?;
    /// ```
    pub fn with_volumes_from_config(
        mut self,
        volume_configs: &[crate::config::VolumeConfig],
    ) -> Result<Self>;
}
```

### 5.3 ServicePlugin Trait Implementation - Enhanced

```rust
impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // ... existing setup code ...

                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // ... existing env vars and ports ...

                // NEW: Mount volumes
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

                // ... rest of existing code ...

                // NEW: Add volume info to metadata
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

                Ok(ServiceHandle { /* ... */ })
            })
        })
    }

    // ... existing stop() and health_check() ...
}
```

---

## 6. Helper Functions

**File**: `crates/clnrm-core/src/services/mod.rs` (or similar)

### 6.1 Service Creation with Volumes

```rust
/// Create a service plugin from configuration with volume support
///
/// # Arguments
/// * `name` - Service name
/// * `config` - Service configuration from TOML
///
/// # Errors
/// - Unknown plugin type
/// - Invalid configuration
/// - Volume validation errors
///
/// # Examples
/// ```rust
/// let plugin = create_service_from_config("my_service", &service_config)?;
/// environment.register_service(plugin).await?;
/// ```
pub fn create_service_from_config(
    name: &str,
    config: &ServiceConfig,
) -> Result<Box<dyn ServicePlugin>>;
```

---

## 7. Integration Points Summary

### 7.1 Data Flow

```
TOML Config (VolumeConfig)
    ↓
Config Validation (validate())
    ↓
VolumeMount Creation (from_config())
    ↓
Security Validation (VolumeValidator::validate())
    ↓
Backend/Plugin Configuration (with_volume())
    ↓
Container Creation (with_mount() via testcontainers)
    ↓
Container Execution (volumes mounted)
```

### 7.2 Module Dependencies

```
config.rs (VolumeConfig)
    ↓
backend/volume.rs (VolumeMount, VolumeValidator)
    ↓
backend/testcontainer.rs (TestcontainerBackend)
    ↓
services/generic.rs (GenericContainerPlugin)
    ↓
cleanroom.rs (CleanroomEnvironment)
```

### 7.3 Trait Compatibility Matrix

| Trait/Structure | Sync/Async | Result Type | Core Team Compliant |
|----------------|------------|-------------|---------------------|
| `VolumeMount::new()` | Sync | `Result<VolumeMount>` | ✅ Yes |
| `VolumeValidator::validate()` | Sync | `Result<()>` | ✅ Yes |
| `TestcontainerBackend::with_volume()` | Sync | `Result<Self>` | ✅ Yes |
| `GenericContainerPlugin::with_volume()` | Sync | `Result<Self>` | ✅ Yes |
| `ServicePlugin::start()` | Sync (async internal) | `Result<ServiceHandle>` | ✅ Yes |
| `VolumeConfig::validate()` | Sync | `Result<()>` | ✅ Yes |

---

## 8. Error Types

### 8.1 Volume-Specific Errors

```rust
// All use existing CleanroomError types - no new error types needed

// Validation errors
CleanroomError::validation_error("Host path must be absolute")
CleanroomError::validation_error("Host path does not exist")
CleanroomError::validation_error("Container path must be absolute")
CleanroomError::validation_error("Volume host_path cannot be empty")

// Security errors
CleanroomError::security_error("Host path contains parent directory traversal")
CleanroomError::security_error("Host path not in allowed directories")

// Container errors (during mounting)
CleanroomError::container_error("Failed to mount volume")
    .with_context("Volume mounting failed during container creation")
    .with_source(underlying_error)
```

---

## 9. Type Aliases and Constants

```rust
// No new type aliases needed

// Recommended constants (optional)
pub const DEFAULT_MAX_VOLUMES: usize = 10;
pub const VOLUME_READ_ONLY: bool = true;
pub const VOLUME_READ_WRITE: bool = false;
```

---

## 10. Builder Pattern Examples

### 10.1 TestcontainerBackend with Volumes

```rust
let backend = TestcontainerBackend::new("alpine:latest")?
    .with_timeout(Duration::from_secs(60))
    .with_volume("/tmp/data", "/data", false)?
    .with_volume("/tmp/config", "/config", true)?
    .with_volume_validator(VolumeValidator::permissive());
```

### 10.2 GenericContainerPlugin with Volumes

```rust
let plugin = GenericContainerPlugin::new("processor", "alpine:latest")
    .with_env("ENV_VAR", "value")
    .with_port(8080)
    .with_volume("/tmp/input", "/input", true)?
    .with_volume("/tmp/output", "/output", false)?;
```

### 10.3 From Configuration

```rust
// Service config loaded from TOML
let service_config = load_config_from_file("test.clnrm.toml")?;

// Create plugin with volumes from config
let plugin = GenericContainerPlugin::new("service", "alpine:latest")
    .with_volumes_from_config(
        &service_config.services
            .get("service")
            .unwrap()
            .volumes
            .as_ref()
            .unwrap()
    )?;
```

---

## 11. Testing Trait Methods

### 11.1 Mock Volume Support

```rust
// For testing backends that don't support volumes
struct MockBackend;

impl Backend for MockBackend {
    // ... other methods ...

    fn supports_volumes(&self) -> bool {
        false  // Mock doesn't support volumes
    }

    fn max_volumes(&self) -> Option<usize> {
        Some(0)  // No volumes allowed
    }
}
```

### 11.2 Test Volume Validator

```rust
#[test]
fn test_volume_validator_interface() {
    let validator = VolumeValidator::new(vec![PathBuf::from("/tmp")]);
    let mount = VolumeMount::new("/tmp/test", "/data", false).unwrap();

    // Trait methods are sync - no async needed
    assert!(validator.validate(&mount).is_ok());
}
```

---

## 12. Backward Compatibility

### 12.1 Trait Default Methods

All new `Backend` trait methods have default implementations, ensuring backward compatibility:

```rust
// Existing backends automatically get these defaults
fn supports_volumes(&self) -> bool {
    false  // Safe default - no breaking changes
}

fn max_volumes(&self) -> Option<usize> {
    Some(10)  // Safe default
}
```

### 12.2 Optional Fields

All volume-related fields in configs are `Option<T>`:

```rust
pub struct ServiceConfig {
    // ... existing fields ...

    /// Service volumes (OPTIONAL - backward compatible)
    pub volumes: Option<Vec<VolumeConfig>>,
}
```

---

## 13. Performance Considerations

### 13.1 Volume Mount Validation Cost

```rust
// VolumeMount::new() performs:
// 1. Path validation (cheap)
// 2. File existence check (syscall - moderate)
// 3. Canonicalization (syscall - moderate)
// Total: ~1-5ms per volume on typical systems

// Recommendation: Validate once, reuse VolumeMount instances
```

### 13.2 Container Startup Impact

```rust
// Volume mounting adds minimal overhead:
// - Docker bind mount creation: ~10-50ms per volume
// - No data copying (bind mounts are instant)
// - Read-only enforcement: no overhead

// Expected impact: <100ms for typical test (2-3 volumes)
```

---

## 14. Thread Safety

All volume types are thread-safe:

```rust
// VolumeMount: Send + Sync (no interior mutability)
impl Send for VolumeMount {}
impl Sync for VolumeMount {}

// VolumeValidator: Send + Sync (immutable after creation)
impl Send for VolumeValidator {}
impl Sync for VolumeValidator {}

// TestcontainerBackend: Already Clone
// - Uses Arc<VolumeValidator> for shared ownership
```

---

## Conclusion

This design provides clear trait signatures and integration points that:

1. **Maintain backward compatibility** with default trait implementations
2. **Follow core team standards** with sync methods and Result types
3. **Are thread-safe** with proper Send + Sync bounds
4. **Have minimal performance impact** (<100ms overhead)
5. **Provide clear error messages** using existing error types
6. **Are fully testable** with mock implementations

All public APIs follow Rust naming conventions and include comprehensive documentation comments.
