# Volume Support - Architecture Diagrams

**Version**: 1.0
**Date**: 2025-10-16

This document provides visual representations of the volume support architecture.

---

## 1. Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         TOML Configuration                       │
│  [services.my_service]                                          │
│  [[services.my_service.volumes]]                                │
│  host_path = "/tmp/data"                                        │
│  container_path = "/data"                                       │
│  read_only = false                                              │
└────────────────────────┬────────────────────────────────────────┘
                         │ Parse & Validate
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│                    config::VolumeConfig                         │
│  + host_path: String                                            │
│  + container_path: String                                       │
│  + read_only: bool                                              │
│  + validate() -> Result<()>                                     │
└────────────────────────┬────────────────────────────────────────┘
                         │ Convert & Validate
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│               backend::volume::VolumeMount                      │
│  - host_path: PathBuf (canonical)                               │
│  - container_path: PathBuf                                      │
│  - read_only: bool                                              │
│  + new() -> Result<Self>                                        │
│  + from_config() -> Result<Self>                                │
│  + validate() -> Result<()>                                     │
│  + to_mount_string() -> String                                  │
└────────────────────────┬────────────────────────────────────────┘
                         │ Security Check
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│            backend::volume::VolumeValidator                     │
│  - allowed_host_dirs: Vec<PathBuf>                              │
│  - permissive: bool                                             │
│  + validate(&VolumeMount) -> Result<()>                         │
│  + validate_all(&[VolumeMount]) -> Result<()>                  │
└────────────────────────┬────────────────────────────────────────┘
                         │ Apply to Backend/Plugin
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│              backend::TestcontainerBackend                      │
│  - volume_mounts: Vec<VolumeMount>                              │
│  - volume_validator: Arc<VolumeValidator>                       │
│  + with_volume() -> Result<Self>                                │
│  + with_volume_validator() -> Self                              │
│  + volumes() -> &[VolumeMount]                                  │
│  + execute_in_container() -> Result<RunResult>                  │
│                      (mounts volumes)                           │
└────────────────────────┬────────────────────────────────────────┘
                         │ Used by
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│           services::GenericContainerPlugin                      │
│  - volumes: Vec<VolumeMount>                                    │
│  + with_volume() -> Result<Self>                                │
│  + with_volumes_from_config() -> Result<Self>                   │
│  + start() -> Result<ServiceHandle>                             │
│            (creates container with volumes)                     │
└────────────────────────┬────────────────────────────────────────┘
                         │ Registered in
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│              cleanroom::CleanroomEnvironment                    │
│  - services: Arc<RwLock<ServiceRegistry>>                       │
│  + register_service(plugin)                                     │
│  + start_service(name) -> Result<ServiceHandle>                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Data Flow Sequence

```
User Test Definition (TOML)
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Configuration Loading                                   │
└─────────────────────────────────────────────────────────────────┘
    ↓
load_config_from_file("test.clnrm.toml")
    ↓
TestConfig {
    services: {
        "my_service": ServiceConfig {
            volumes: Some([VolumeConfig {...}])
        }
    }
}
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: Configuration Validation                                │
└─────────────────────────────────────────────────────────────────┘
    ↓
TestConfig::validate()
    ↓
ServiceConfig::validate()
    ↓
VolumeConfig::validate()
    ├─ Check host_path not empty
    ├─ Check container_path not empty
    ├─ Check host_path is absolute
    └─ Check container_path is absolute
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: Volume Mount Creation                                   │
└─────────────────────────────────────────────────────────────────┘
    ↓
VolumeMount::from_config(&volume_config)
    ↓
VolumeMount::new(host_path, container_path, read_only)
    ├─ Validate host_path is absolute
    ├─ Validate host_path exists
    ├─ Canonicalize host_path (resolve symlinks)
    ├─ Check for ".." traversal
    └─ Validate container_path is absolute
    ↓
VolumeMount { host_path, container_path, read_only }
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 4: Security Validation                                     │
└─────────────────────────────────────────────────────────────────┘
    ↓
VolumeValidator::validate(&volume_mount)
    ↓
if permissive_mode {
    ✓ Allow
} else {
    Check if host_path in allowed_host_dirs
    ├─ In whitelist → ✓ Allow
    └─ Not in whitelist → ✗ Reject
}
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 5: Service Plugin Creation                                 │
└─────────────────────────────────────────────────────────────────┘
    ↓
GenericContainerPlugin::new(name, image)
    .with_volumes_from_config(&volume_configs)?
    ↓
For each VolumeConfig:
    VolumeMount::from_config(config)?
    Add to plugin.volumes
    ↓
Plugin { volumes: Vec<VolumeMount> }
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 6: Service Registration                                    │
└─────────────────────────────────────────────────────────────────┘
    ↓
CleanroomEnvironment::register_service(plugin)
    ↓
ServiceRegistry::register_plugin(plugin)
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 7: Service Start (Container Creation)                      │
└─────────────────────────────────────────────────────────────────┘
    ↓
CleanroomEnvironment::start_service("my_service")
    ↓
ServiceRegistry::start_service("my_service")
    ↓
Plugin::start()
    ↓
GenericContainerPlugin::start()
    ↓
let image = GenericImage::new(...)
let mut container_request = image.into()
    ↓
For each volume in plugin.volumes:
    let access_mode = if volume.is_read_only() {
        AccessMode::ReadOnly
    } else {
        AccessMode::ReadWrite
    }
    container_request = container_request.with_mount(
        Mount::bind_mount(
            volume.host_path(),
            volume.container_path()
        ).with_access_mode(access_mode)
    )
    ↓
container_request.start().await
    ↓
Docker Container Created with Volumes Mounted
    ↓
ServiceHandle { id, metadata }
    ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 8: Test Execution                                          │
└─────────────────────────────────────────────────────────────────┘
    ↓
Container executes test steps with volumes accessible
    ↓
Results collected and returned
```

---

## 3. Module Dependency Graph

```
┌───────────────────────────────────────────────────────────────┐
│                      Application Layer                         │
│                                                                │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ crates/clnrm/src/main.rs                              │    │
│  │ CLI commands: init, run, report                       │    │
│  └────────────────────────┬──────────────────────────────┘    │
│                           │ uses                               │
└───────────────────────────┼────────────────────────────────────┘
                            ↓
┌───────────────────────────────────────────────────────────────┐
│                    Core Library Layer                          │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ crates/clnrm-core/src/cleanroom.rs                     │   │
│  │ CleanroomEnvironment, ServiceRegistry                  │   │
│  └───────────────┬────────────────────────────────────────┘   │
│                  │ uses                                        │
│                  ↓                                             │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ crates/clnrm-core/src/services/                        │   │
│  │ GenericContainerPlugin, SurrealDbPlugin, etc.          │   │
│  └───────────────┬────────────────────────────────────────┘   │
│                  │ uses                                        │
│                  ↓                                             │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ crates/clnrm-core/src/backend/                         │   │
│  │ Backend trait, TestcontainerBackend                    │   │
│  └───────────────┬────────────────────────────────────────┘   │
│                  │ uses                                        │
│                  ↓                                             │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ crates/clnrm-core/src/backend/volume.rs  [NEW]        │   │
│  │ VolumeMount, VolumeValidator                           │   │
│  └───────────────┬────────────────────────────────────────┘   │
│                  │ uses                                        │
│                  ↓                                             │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ crates/clnrm-core/src/config.rs                        │   │
│  │ VolumeConfig, ServiceConfig, TestConfig                │   │
│  └───────────────┬────────────────────────────────────────┘   │
│                  │ uses                                        │
│                  ↓                                             │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ crates/clnrm-core/src/error.rs                         │   │
│  │ CleanroomError, Result                                 │   │
│  └────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────┘
                            ↓
┌───────────────────────────────────────────────────────────────┐
│                  External Dependencies                         │
│                                                                │
│  testcontainers, testcontainers-modules                        │
│  tokio, serde, toml                                            │
└───────────────────────────────────────────────────────────────┘
```

---

## 4. Class Diagram (Rust Structures)

```
┌─────────────────────────────────────────────────────────────┐
│                       VolumeConfig                           │
├─────────────────────────────────────────────────────────────┤
│ + host_path: String                                         │
│ + container_path: String                                    │
│ + read_only: bool                                           │
├─────────────────────────────────────────────────────────────┤
│ + validate() -> Result<()>                                  │
└─────────────────────────┬───────────────────────────────────┘
                          │ converts to
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                       VolumeMount                            │
├─────────────────────────────────────────────────────────────┤
│ - host_path: PathBuf                                        │
│ - container_path: PathBuf                                   │
│ - read_only: bool                                           │
├─────────────────────────────────────────────────────────────┤
│ + new(host, container, ro) -> Result<Self>                  │
│ + from_config(config) -> Result<Self>                       │
│ + host_path() -> &Path                                      │
│ + container_path() -> &Path                                 │
│ + is_read_only() -> bool                                    │
│ + to_mount_string() -> String                               │
└─────────────────────────┬───────────────────────────────────┘
                          │ validated by
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                    VolumeValidator                           │
├─────────────────────────────────────────────────────────────┤
│ - allowed_host_dirs: Vec<PathBuf>                           │
│ - permissive: bool                                          │
├─────────────────────────────────────────────────────────────┤
│ + new(allowed_dirs) -> Self                                 │
│ + permissive() -> Self                                      │
│ + validate(&VolumeMount) -> Result<()>                      │
│ + validate_all(&[VolumeMount]) -> Result<()>               │
└─────────────────────────┬───────────────────────────────────┘
                          │ used by
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                  TestcontainerBackend                        │
├─────────────────────────────────────────────────────────────┤
│ - image_name: String                                        │
│ - image_tag: String                                         │
│ - volume_mounts: Vec<VolumeMount>                           │
│ - volume_validator: Arc<VolumeValidator>                    │
│ - env_vars: HashMap<String, String>                         │
├─────────────────────────────────────────────────────────────┤
│ + new(image) -> Result<Self>                                │
│ + with_volume(host, container, ro) -> Result<Self>          │
│ + with_volume_validator(validator) -> Self                  │
│ + volumes() -> &[VolumeMount]                               │
│ + execute_in_container(cmd) -> Result<RunResult>            │
├─────────────────────────────────────────────────────────────┤
│ implements Backend                                          │
│   + supports_volumes() -> bool                              │
│   + max_volumes() -> Option<usize>                          │
└─────────────────────────┬───────────────────────────────────┘
                          │ used by
                          ↓
┌─────────────────────────────────────────────────────────────┐
│               GenericContainerPlugin                         │
├─────────────────────────────────────────────────────────────┤
│ - name: String                                              │
│ - image: String                                             │
│ - tag: String                                               │
│ - volumes: Vec<VolumeMount>                                 │
│ - env_vars: HashMap<String, String>                         │
│ - ports: Vec<u16>                                           │
├─────────────────────────────────────────────────────────────┤
│ + new(name, image) -> Self                                  │
│ + with_volume(host, container, ro) -> Result<Self>          │
│ + with_volumes_from_config(&[VolumeConfig]) -> Result<Self> │
│ + with_env(key, val) -> Self                                │
│ + with_port(port) -> Self                                   │
├─────────────────────────────────────────────────────────────┤
│ implements ServicePlugin                                    │
│   + name() -> &str                                          │
│   + start() -> Result<ServiceHandle>                        │
│   + stop(handle) -> Result<()>                              │
│   + health_check(handle) -> HealthStatus                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Security Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Input (TOML)                         │
│  host_path = "/some/path"                                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ↓
        ┌────────────────────────────────────┐
        │   Configuration Validation Layer   │
        │                                    │
        │  1. Path not empty                 │
        │  2. Host path is absolute          │
        │  3. Container path is absolute     │
        └────────────┬───────────────────────┘
                     │
                     ↓
        ┌────────────────────────────────────┐
        │   Volume Mount Creation Layer      │
        │                                    │
        │  1. Host path exists               │
        │  2. Canonicalize (resolve symlinks)│
        │  3. Check for ".." traversal       │
        │  4. Validate absolute paths        │
        └────────────┬───────────────────────┘
                     │
                     ↓
        ┌────────────────────────────────────┐
        │   Security Validation Layer        │
        │                                    │
        │  VolumeValidator checks:           │
        │  - Permissive mode? → Allow all    │
        │  - Strict mode?                    │
        │    → Check whitelist               │
        │    → /tmp ✓                        │
        │    → /var/tmp ✓                    │
        │    → CWD ✓                         │
        │    → /etc ✗ (not in whitelist)    │
        └────────────┬───────────────────────┘
                     │
                     ↓
        ┌────────────────────────────────────┐
        │   Container Runtime Layer          │
        │                                    │
        │  Docker/Podman enforces:           │
        │  - Read-only mounts (kernel level) │
        │  - Container user isolation        │
        │  - No privilege escalation         │
        └────────────┬───────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────────┐
│              Safe Container Execution                        │
│  - Volumes mounted with correct permissions                 │
│  - Host filesystem protected                                │
│  - Test executes in isolated environment                    │
└─────────────────────────────────────────────────────────────┘

Security Layers Summary:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Layer 1: Input Validation       (catches typos, format errors)
Layer 2: Path Validation        (catches malformed paths)
Layer 3: Security Validation    (catches unauthorized access)
Layer 4: Container Runtime      (kernel-level enforcement)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 6. Error Flow Diagram

```
User Action: Define volume in TOML
    ↓
┌─────────────────────────────────────────────────────────────┐
│ Potential Error Points                                       │
└─────────────────────────────────────────────────────────────┘
    ↓
┌────────────────┐
│ Parse TOML     │
└────────┬───────┘
         │ Error: Invalid TOML syntax
         ├─→ CleanroomError::config_error("TOML parse error")
         │   Return to user with line number
         │
         ↓
┌────────────────┐
│ Validate Config│
└────────┬───────┘
         │ Error: Empty host_path
         ├─→ CleanroomError::validation_error("host_path cannot be empty")
         │
         │ Error: Relative path
         ├─→ CleanroomError::validation_error("Host path must be absolute")
         │
         ↓
┌────────────────┐
│ Create Volume  │
│ Mount          │
└────────┬───────┘
         │ Error: Host path doesn't exist
         ├─→ CleanroomError::validation_error("Host path does not exist: /path")
         │
         │ Error: Cannot canonicalize
         ├─→ CleanroomError::validation_error("Failed to canonicalize: permission denied")
         │
         │ Error: Contains ".."
         ├─→ CleanroomError::security_error("Path contains parent directory traversal")
         │
         ↓
┌────────────────┐
│ Security Check │
└────────┬───────┘
         │ Error: Not in whitelist
         ├─→ CleanroomError::security_error("Path not in allowed directories")
         │
         ↓
┌────────────────┐
│ Start Container│
└────────┬───────┘
         │ Error: Mount failed
         ├─→ CleanroomError::container_error("Failed to mount volume")
         │
         ↓
┌────────────────┐
│ Success        │
└────────────────┘

Error Handling Strategy:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
- All functions return Result<T, CleanroomError>
- Errors propagate up with context (.with_context())
- Clear error messages for debugging
- No panics (no unwrap/expect)
- Fail fast: Stop on first error
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 7. Implementation Phases Diagram

```
Phase 1: Core Volume Support (2-3 hours)
┌─────────────────────────────────────────────────────────────┐
│ ✓ Create backend/volume.rs                                  │
│   - VolumeMount struct                                      │
│   - VolumeValidator struct                                  │
│                                                             │
│ ✓ Update config.rs                                          │
│   - VolumeConfig::validate()                                │
│                                                             │
│ ✓ Update TestcontainerBackend                               │
│   - Change volume_mounts type                               │
│   - Implement with_volume()                                 │
│   - Implement actual mounting                               │
└─────────────────────────────────────────────────────────────┘
                         ↓
Phase 2: Plugin Integration (1-2 hours)
┌─────────────────────────────────────────────────────────────┐
│ ✓ Update GenericContainerPlugin                             │
│   - Add volumes field                                       │
│   - Implement with_volume()                                 │
│   - Implement with_volumes_from_config()                    │
│   - Update start() to mount volumes                         │
│                                                             │
│ ✓ Create service creation helper                            │
│   - create_service_from_config()                            │
└─────────────────────────────────────────────────────────────┘
                         ↓
Phase 3: Testing (2-3 hours)
┌─────────────────────────────────────────────────────────────┐
│ ✓ Unit tests                                                │
│   - VolumeMount creation                                    │
│   - VolumeValidator logic                                   │
│   - Path validation edge cases                              │
│                                                             │
│ ✓ Integration tests                                         │
│   - Volume mounting end-to-end                              │
│   - Read-only enforcement                                   │
│   - Multiple volumes                                        │
│                                                             │
│ ✓ Security tests                                            │
│   - Path traversal rejection                                │
│   - Whitelist enforcement                                   │
└─────────────────────────────────────────────────────────────┘
                         ↓
Phase 4: Documentation (1 hour)
┌─────────────────────────────────────────────────────────────┐
│ ✓ Update TOML_REFERENCE.md                                  │
│ ✓ Create VOLUME_GUIDE.md                                    │
│ ✓ Add example test files                                    │
└─────────────────────────────────────────────────────────────┘
                         ↓
                    COMPLETE
        Total: 6-8 hours for production-ready feature
```

---

## 8. Testing Strategy Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Unit Tests                              │
│  (Fast, Isolated, No Container Creation)                    │
├─────────────────────────────────────────────────────────────┤
│ ✓ VolumeMount::new() validation                             │
│ ✓ VolumeMount::from_config() conversion                     │
│ ✓ VolumeMount::to_mount_string() formatting                 │
│ ✓ VolumeValidator::validate() whitelist logic               │
│ ✓ VolumeConfig::validate() config validation                │
│                                                             │
│ Test Count: ~20 tests                                       │
│ Execution Time: <1 second                                   │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                  Integration Tests                           │
│  (Container Creation, Volume Mounting, File I/O)            │
├─────────────────────────────────────────────────────────────┤
│ ✓ Mount volume and write file                               │
│ ✓ Mount read-only volume, verify write fails                │
│ ✓ Mount multiple volumes                                    │
│ ✓ Verify host filesystem reflects changes                   │
│ ✓ Service plugin with volumes from config                   │
│                                                             │
│ Test Count: ~10 tests                                       │
│ Execution Time: 30-60 seconds                               │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                   TOML Config Tests                          │
│  (End-to-End with Real TOML Files)                          │
├─────────────────────────────────────────────────────────────┤
│ ✓ tests/volumes/basic-volume.clnrm.toml                     │
│ ✓ tests/volumes/readonly-enforcement.clnrm.toml             │
│ ✓ tests/volumes/multiple-volumes.clnrm.toml                 │
│ ✓ tests/volumes/data-processing.clnrm.toml                  │
│                                                             │
│ Test Count: ~5 TOML test files                              │
│ Execution Time: 30-60 seconds                               │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                    Security Tests                            │
│  (Path Traversal, Whitelist, Permissions)                   │
├─────────────────────────────────────────────────────────────┤
│ ✓ Reject relative paths                                     │
│ ✓ Reject nonexistent paths                                  │
│ ✓ Reject paths with ".."                                    │
│ ✓ Enforce whitelist in strict mode                          │
│ ✓ Verify read-only at kernel level                          │
│                                                             │
│ Test Count: ~8 tests                                        │
│ Execution Time: 10-20 seconds                               │
└─────────────────────────────────────────────────────────────┘
                         ↓
                 Total Coverage: >80%
```

---

## 9. Use Case Scenarios

### Scenario 1: Data Processing Pipeline

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   Host      │      │  Container  │      │   Host      │
│  /tmp/input │─────>│   /input    │      │ /tmp/output │
│             │ RO   │             │      │             │
│ data.txt    │      │ Read data   │      │             │
└─────────────┘      │ Process     │      │             │
                     │ Transform   │      │             │
                     │   /output   │─────>│ result.txt  │
                     └─────────────┘ RW   └─────────────┘

Volume Configuration:
[[volumes]]
host_path = "/tmp/input"
container_path = "/input"
read_only = true

[[volumes]]
host_path = "/tmp/output"
container_path = "/output"
read_only = false
```

### Scenario 2: Config and Data Separation

```
┌─────────────┐      ┌─────────────┐
│   Host      │      │  Container  │
│ /etc/config │─────>│  /config    │ (Read-Only Config)
└─────────────┘ RO   └─────────────┘
                            │
┌─────────────┐             │
│   Host      │             │
│ /var/data   │<────────────┘ (Read-Write Data)
└─────────────┘ RW

Use Case: Application reads config (cannot modify),
          writes data/logs to data volume
```

### Scenario 3: Test Fixtures

```
┌─────────────┐      ┌─────────────┐
│   Host      │      │  Container  │
│ tests/      │─────>│  /fixtures  │ (Read-Only Fixtures)
│ fixtures/   │ RO   └─────────────┘
└─────────────┘

Use Case: Shared test fixtures across multiple tests,
          immutable to prevent accidental modification
```

---

## 10. Performance Characteristics

```
Operation Performance Profile
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Configuration Loading:
┌─────────────────────────────────────────────┐
│ Parse TOML                       1-5 ms     │
│ Validate VolumeConfig            <1 ms      │
└─────────────────────────────────────────────┘

Volume Mount Creation:
┌─────────────────────────────────────────────┐
│ Path validation                  <1 ms      │
│ File existence check             1-5 ms     │
│ Canonicalization                 1-5 ms     │
│ Security validation              <1 ms      │
│ ─────────────────────────────────────────   │
│ Total per volume:                2-10 ms    │
└─────────────────────────────────────────────┘

Container Startup (with volumes):
┌─────────────────────────────────────────────┐
│ Container creation               100-200 ms │
│ Volume bind mount (x3)           10-50 ms   │
│ Container start                  50-100 ms  │
│ ─────────────────────────────────────────   │
│ Total:                           160-350 ms │
└─────────────────────────────────────────────┘

Impact: +10-50ms per volume (negligible)

File I/O (via volume):
┌─────────────────────────────────────────────┐
│ Write to mounted volume          Same as    │
│ Read from mounted volume         native     │
│                                  filesystem │
└─────────────────────────────────────────────┘

Impact: Zero overhead (bind mounts are native)
```

---

## Conclusion

These diagrams illustrate:

1. **Clear component boundaries** between config, validation, backend, and plugins
2. **Layered security** with multiple validation stages
3. **Straightforward data flow** from TOML to mounted volumes
4. **Comprehensive testing strategy** covering all layers
5. **Predictable performance** with minimal overhead

The architecture is designed for simplicity, security, and maintainability following the 80/20 principle.
