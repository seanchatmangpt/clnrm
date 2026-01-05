# OCI Image Loading and gVisor Execution Architecture

## Overview

This document describes the architecture for direct OCI image loading and execution without Docker daemon dependency, using gVisor's runsc for container runtime.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Scenario Layer                            │
│                  (scenario.rs - unchanged)                       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Backend Abstraction                         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │ TestcontainerBE  │  │  GvisorBackend   │  │  MockBackend  │ │
│  │  (Docker daemon) │  │ (OCI + runsc)    │  │  (testing)    │ │
│  └──────────────────┘  └────────┬─────────┘  └───────────────┘ │
└─────────────────────────────────┼────────────────────────────────┘
                                  │
                                  ▼
              ┌──────────────────────────────────────────┐
              │        OCI Image Manager                 │
              │  ┌──────────────┐  ┌─────────────────┐  │
              │  │ Image Loader │  │  Image Cache    │  │
              │  │  - Registry  │  │  - Layers       │  │
              │  │  - Local     │  │  - Metadata     │  │
              │  │  - Embedded  │  │  - LRU eviction │  │
              │  └──────┬───────┘  └────────┬────────┘  │
              └─────────┼──────────────────┬─┼───────────┘
                        │                  │ │
                        ▼                  │ │
              ┌──────────────────────┐     │ │
              │  OCI Bundle Builder  │     │ │
              │  - rootfs extraction │     │ │
              │  - config.json       │     │ │
              │  - layer merging     │     │ │
              └──────────┬───────────┘     │ │
                         │                 │ │
                         ▼                 │ │
              ┌──────────────────────┐     │ │
              │  gVisor runsc        │     │ │
              │  - create bundle     │     │ │
              │  - run container     │     │ │
              │  - capture output    │     │ │
              │  - handle exit       │     │ │
              └──────────────────────┘     │ │
                                           │ │
                                           ▼ ▼
                        ┌────────────────────────────────┐
                        │   Filesystem Cache Store       │
                        │   ~/.cache/clnrm/oci/          │
                        │   - images/                    │
                        │   - layers/                    │
                        │   - bundles/                   │
                        └────────────────────────────────┘
```

## Module Structure

```
crates/clnrm-core/src/
├── backend/
│   ├── mod.rs                    # Backend trait + exports
│   ├── testcontainer.rs          # Existing Docker backend
│   ├── gvisor.rs                 # NEW: gVisor + OCI backend
│   └── oci/
│       ├── mod.rs                # OCI module exports
│       ├── image_loader.rs       # Image loading from sources
│       ├── registry_client.rs    # Docker registry API v2
│       ├── bundle_builder.rs     # OCI bundle creation
│       ├── layer_manager.rs      # Layer extraction/merging
│       ├── config_parser.rs      # OCI config.json parsing
│       ├── cache.rs              # Image/layer caching
│       └── runsc_executor.rs     # gVisor runsc CLI integration
```

## 1. OCI Image Loading

### 1.1 Image Loader (`image_loader.rs`)

```rust
/// OCI image loading from multiple sources
pub struct OciImageLoader {
    cache: Arc<ImageCache>,
    registry_client: RegistryClient,
    local_store: LocalImageStore,
}

pub enum ImageSource {
    /// Docker registry (registry.hub.docker.com/library/alpine:latest)
    Registry {
        registry: String,
        repository: String,
        tag: String,
    },
    /// Local OCI directory layout
    Local {
        path: PathBuf,
    },
    /// Embedded tarball in binary
    Embedded {
        data: &'static [u8],
    },
}

impl OciImageLoader {
    /// Load image from any source
    pub async fn load_image(&self, source: ImageSource) -> Result<OciImage> {
        match source {
            ImageSource::Registry { registry, repository, tag } => {
                // Check cache first
                let image_ref = format!("{}/{}:{}", registry, repository, tag);
                if let Some(cached) = self.cache.get(&image_ref).await? {
                    return Ok(cached);
                }

                // Pull from registry
                let image = self.registry_client
                    .pull_image(&registry, &repository, &tag)
                    .await?;

                // Cache for future use
                self.cache.store(&image_ref, &image).await?;

                Ok(image)
            }
            ImageSource::Local { path } => {
                self.local_store.load_from_path(path).await
            }
            ImageSource::Embedded { data } => {
                self.local_store.load_from_tarball(data).await
            }
        }
    }
}
```

### 1.2 Registry Client (`registry_client.rs`)

```rust
/// Docker Registry API v2 client
pub struct RegistryClient {
    http_client: reqwest::Client,
    auth_cache: DashMap<String, AuthToken>,
}

impl RegistryClient {
    /// Pull image manifest and layers from registry
    pub async fn pull_image(
        &self,
        registry: &str,
        repository: &str,
        tag: &str,
    ) -> Result<OciImage> {
        // 1. Get authentication token
        let token = self.authenticate(registry, repository).await?;

        // 2. Fetch manifest
        let manifest = self.fetch_manifest(registry, repository, tag, &token).await?;

        // 3. Download config blob
        let config = self.fetch_blob(registry, repository, &manifest.config.digest, &token).await?;

        // 4. Download layer blobs
        let mut layers = Vec::new();
        for layer_desc in &manifest.layers {
            let layer_data = self.fetch_blob(
                registry,
                repository,
                &layer_desc.digest,
                &token
            ).await?;
            layers.push(OciLayer {
                digest: layer_desc.digest.clone(),
                media_type: layer_desc.media_type.clone(),
                size: layer_desc.size,
                data: layer_data,
            });
        }

        Ok(OciImage {
            manifest,
            config: serde_json::from_slice(&config)?,
            layers,
        })
    }

    /// Authenticate with registry (supports bearer tokens)
    async fn authenticate(&self, registry: &str, repository: &str) -> Result<AuthToken> {
        // Check cache
        let cache_key = format!("{}:{}", registry, repository);
        if let Some(token) = self.auth_cache.get(&cache_key) {
            if !token.is_expired() {
                return Ok(token.clone());
            }
        }

        // Request new token
        let auth_url = format!("https://{}/token?service={}&scope=repository:{}:pull",
            registry, registry, repository);

        let response: AuthResponse = self.http_client
            .get(&auth_url)
            .send()
            .await?
            .json()
            .await?;

        let token = AuthToken {
            token: response.token,
            expires_at: Utc::now() + Duration::seconds(response.expires_in),
        };

        self.auth_cache.insert(cache_key, token.clone());
        Ok(token)
    }

    /// Fetch image manifest
    async fn fetch_manifest(
        &self,
        registry: &str,
        repository: &str,
        tag: &str,
        token: &AuthToken,
    ) -> Result<OciManifest> {
        let url = format!("https://{}/v2/{}/manifests/{}",
            registry, repository, tag);

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.token))
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CleanroomError::oci_error(
                format!("Failed to fetch manifest: {}", response.status())
            ));
        }

        Ok(response.json().await?)
    }

    /// Fetch blob (config or layer)
    async fn fetch_blob(
        &self,
        registry: &str,
        repository: &str,
        digest: &str,
        token: &AuthToken,
    ) -> Result<Vec<u8>> {
        let url = format!("https://{}/v2/{}/blobs/{}",
            registry, repository, digest);

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CleanroomError::oci_error(
                format!("Failed to fetch blob {}: {}", digest, response.status())
            ));
        }

        Ok(response.bytes().await?.to_vec())
    }
}
```

## 2. OCI Image Unpacking

### 2.1 Layer Manager (`layer_manager.rs`)

```rust
/// Manages OCI layer extraction and merging
pub struct LayerManager {
    cache_dir: PathBuf,
    temp_dir: PathBuf,
}

impl LayerManager {
    /// Extract all layers to create merged rootfs
    pub async fn extract_rootfs(
        &self,
        layers: &[OciLayer],
        target_dir: &Path,
    ) -> Result<PathBuf> {
        let rootfs_path = target_dir.join("rootfs");
        fs::create_dir_all(&rootfs_path).await?;

        // Extract layers in order (base to top)
        for (idx, layer) in layers.iter().enumerate() {
            tracing::info!("Extracting layer {}/{}: {}",
                idx + 1, layers.len(), layer.digest);

            match layer.media_type.as_str() {
                "application/vnd.docker.image.rootfs.diff.tar.gzip" |
                "application/vnd.oci.image.layer.v1.tar+gzip" => {
                    self.extract_gzip_layer(layer, &rootfs_path).await?;
                }
                "application/vnd.docker.image.rootfs.diff.tar" |
                "application/vnd.oci.image.layer.v1.tar" => {
                    self.extract_tar_layer(layer, &rootfs_path).await?;
                }
                _ => {
                    return Err(CleanroomError::oci_error(
                        format!("Unsupported layer media type: {}", layer.media_type)
                    ));
                }
            }
        }

        Ok(rootfs_path)
    }

    /// Extract gzipped tar layer
    async fn extract_gzip_layer(&self, layer: &OciLayer, target: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let decoder = GzDecoder::new(&layer.data[..]);
        let mut archive = Archive::new(decoder);

        // Extract with whiteout handling (Docker layer spec)
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            // Handle whiteout files (.wh.* files delete files)
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with(".wh.") {
                    let whiteout_target = path.with_file_name(
                        name.to_string_lossy().strip_prefix(".wh.").unwrap()
                    );
                    let full_path = target.join(&whiteout_target);
                    if full_path.exists() {
                        fs::remove_file(&full_path).await?;
                    }
                    continue;
                }
            }

            // Extract normally
            let full_path = target.join(&path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            entry.unpack(&full_path)?;
        }

        Ok(())
    }

    /// Extract plain tar layer
    async fn extract_tar_layer(&self, layer: &OciLayer, target: &Path) -> Result<()> {
        use tar::Archive;

        let mut archive = Archive::new(&layer.data[..]);
        archive.unpack(target)?;
        Ok(())
    }
}
```

### 2.2 Config Parser (`config_parser.rs`)

```rust
/// OCI image config.json parser
pub struct ConfigParser;

impl ConfigParser {
    /// Parse OCI image config
    pub fn parse(config_data: &[u8]) -> Result<OciImageConfig> {
        let config: OciImageConfig = serde_json::from_slice(config_data)?;
        Ok(config)
    }

    /// Convert OCI image config to runtime config.json for runsc
    pub fn to_runtime_config(
        &self,
        image_config: &OciImageConfig,
        cmd_override: Option<&Cmd>,
    ) -> Result<RuntimeConfig> {
        let config_data = &image_config.config;

        // Build process config
        let mut process = ProcessConfig {
            terminal: false,
            user: config_data.user.clone().unwrap_or_else(|| "0:0".to_string()),
            args: Vec::new(),
            env: config_data.env.clone().unwrap_or_default(),
            cwd: config_data.working_dir.clone()
                .unwrap_or_else(|| "/".to_string()),
            capabilities: None,
            rlimits: vec![],
            no_new_privileges: true,
        };

        // Handle command override or use image defaults
        if let Some(cmd) = cmd_override {
            // Use provided command
            process.args = vec![cmd.bin.clone()];
            process.args.extend_from_slice(&cmd.args);

            // Merge environment variables
            for (key, value) in &cmd.env {
                process.env.push(format!("{}={}", key, value));
            }

            // Override working directory if specified
            if let Some(workdir) = &cmd.workdir {
                process.cwd = workdir.to_string_lossy().to_string();
            }
        } else {
            // Use image's CMD and ENTRYPOINT
            if let Some(entrypoint) = &config_data.entrypoint {
                process.args.extend_from_slice(entrypoint);
            }
            if let Some(cmd) = &config_data.cmd {
                process.args.extend_from_slice(cmd);
            }
        }

        // Build runtime config
        Ok(RuntimeConfig {
            oci_version: "1.0.2".to_string(),
            process,
            root: RootConfig {
                path: "rootfs".to_string(),
                readonly: false,
            },
            hostname: "clnrm-container".to_string(),
            mounts: self.default_mounts(),
            linux: Some(LinuxConfig {
                namespaces: vec![
                    NamespaceConfig { typ: "pid".to_string() },
                    NamespaceConfig { typ: "network".to_string() },
                    NamespaceConfig { typ: "ipc".to_string() },
                    NamespaceConfig { typ: "uts".to_string() },
                    NamespaceConfig { typ: "mount".to_string() },
                ],
                resources: None,
                masked_paths: vec![
                    "/proc/kcore".to_string(),
                    "/proc/latency_stats".to_string(),
                    "/proc/timer_list".to_string(),
                    "/proc/sched_debug".to_string(),
                ],
                readonly_paths: vec![
                    "/proc/asound".to_string(),
                    "/proc/bus".to_string(),
                    "/proc/fs".to_string(),
                    "/proc/irq".to_string(),
                    "/proc/sys".to_string(),
                    "/proc/sysrq-trigger".to_string(),
                ],
            }),
        })
    }

    /// Default mounts for container
    fn default_mounts(&self) -> Vec<MountConfig> {
        vec![
            MountConfig {
                destination: "/proc".to_string(),
                typ: "proc".to_string(),
                source: "proc".to_string(),
                options: vec![],
            },
            MountConfig {
                destination: "/dev".to_string(),
                typ: "tmpfs".to_string(),
                source: "tmpfs".to_string(),
                options: vec!["nosuid".to_string(), "strictatime".to_string(), "mode=755".to_string()],
            },
            MountConfig {
                destination: "/dev/pts".to_string(),
                typ: "devpts".to_string(),
                source: "devpts".to_string(),
                options: vec!["nosuid".to_string(), "noexec".to_string(), "newinstance".to_string()],
            },
            MountConfig {
                destination: "/dev/shm".to_string(),
                typ: "tmpfs".to_string(),
                source: "shm".to_string(),
                options: vec!["nosuid".to_string(), "noexec".to_string(), "nodev".to_string(), "mode=1777".to_string()],
            },
            MountConfig {
                destination: "/sys".to_string(),
                typ: "sysfs".to_string(),
                source: "sysfs".to_string(),
                options: vec!["nosuid".to_string(), "noexec".to_string(), "nodev".to_string(), "ro".to_string()],
            },
        ]
    }
}
```

## 3. OCI Bundle Creation

### 3.1 Bundle Builder (`bundle_builder.rs`)

```rust
/// OCI bundle builder for runsc
pub struct OciBundleBuilder {
    layer_manager: LayerManager,
    config_parser: ConfigParser,
    bundle_dir: PathBuf,
}

impl OciBundleBuilder {
    /// Create OCI bundle from image
    pub async fn create_bundle(
        &self,
        image: &OciImage,
        cmd: Option<&Cmd>,
    ) -> Result<OciBundle> {
        // Create unique bundle directory
        let bundle_id = uuid::Uuid::new_v4().to_string();
        let bundle_path = self.bundle_dir.join(&bundle_id);
        fs::create_dir_all(&bundle_path).await?;

        tracing::info!("Creating OCI bundle at: {}", bundle_path.display());

        // 1. Extract rootfs
        let rootfs_path = self.layer_manager
            .extract_rootfs(&image.layers, &bundle_path)
            .await?;

        tracing::info!("Rootfs extracted to: {}", rootfs_path.display());

        // 2. Generate runtime config.json
        let runtime_config = self.config_parser
            .to_runtime_config(&image.config, cmd)?;

        let config_path = bundle_path.join("config.json");
        let config_json = serde_json::to_string_pretty(&runtime_config)?;
        fs::write(&config_path, config_json).await?;

        tracing::info!("Runtime config written to: {}", config_path.display());

        Ok(OciBundle {
            id: bundle_id,
            path: bundle_path,
            rootfs: rootfs_path,
            config: runtime_config,
        })
    }
}

/// OCI bundle ready for runsc
pub struct OciBundle {
    pub id: String,
    pub path: PathBuf,
    pub rootfs: PathBuf,
    pub config: RuntimeConfig,
}
```

## 4. gVisor runsc Integration

### 4.1 runsc Executor (`runsc_executor.rs`)

```rust
/// gVisor runsc executor
pub struct RunscExecutor {
    runsc_path: PathBuf,
    root_dir: PathBuf,
}

impl RunscExecutor {
    /// Create new runsc executor
    pub fn new() -> Result<Self> {
        // Find runsc binary
        let runsc_path = which::which("runsc")
            .map_err(|_| CleanroomError::runtime_error(
                "runsc not found in PATH. Install gVisor: https://gvisor.dev/docs/user_guide/install/"
            ))?;

        // Create root directory for runsc state
        let root_dir = dirs::cache_dir()
            .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache directory"))?
            .join("clnrm")
            .join("runsc");

        fs::create_dir_all(&root_dir)?;

        Ok(Self {
            runsc_path,
            root_dir,
        })
    }

    /// Execute container using runsc
    pub async fn run_container(
        &self,
        bundle: &OciBundle,
        timeout: Duration,
    ) -> Result<RunscOutput> {
        let container_id = format!("clnrm-{}", bundle.id);

        tracing::info!("Starting container {} with runsc", container_id);

        // Create container
        let create_result = self.create_container(&container_id, bundle).await?;
        if !create_result.success {
            return Err(CleanroomError::runtime_error(format!(
                "Failed to create container: {}", create_result.stderr
            )));
        }

        // Start container
        let start_result = self.start_container(&container_id).await?;
        if !start_result.success {
            // Cleanup on failure
            let _ = self.delete_container(&container_id).await;
            return Err(CleanroomError::runtime_error(format!(
                "Failed to start container: {}", start_result.stderr
            )));
        }

        // Wait for container to complete (with timeout)
        let wait_result = tokio::time::timeout(
            timeout,
            self.wait_container(&container_id)
        ).await;

        let output = match wait_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                // Cleanup on error
                let _ = self.kill_container(&container_id).await;
                let _ = self.delete_container(&container_id).await;
                return Err(e);
            }
            Err(_) => {
                // Timeout - kill container
                tracing::warn!("Container {} timed out, killing", container_id);
                let _ = self.kill_container(&container_id).await;
                let _ = self.delete_container(&container_id).await;
                return Err(CleanroomError::timeout_error(format!(
                    "Container execution timed out after {}s", timeout.as_secs()
                )));
            }
        };

        // Get container logs
        let logs = self.get_container_logs(&container_id).await?;

        // Cleanup container
        self.delete_container(&container_id).await?;

        Ok(RunscOutput {
            exit_code: output.exit_code,
            stdout: logs.stdout,
            stderr: logs.stderr,
            duration_ms: output.duration_ms,
        })
    }

    /// Create container (runsc create)
    async fn create_container(
        &self,
        container_id: &str,
        bundle: &OciBundle,
    ) -> Result<CommandResult> {
        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("create")
            .arg("--bundle")
            .arg(&bundle.path)
            .arg(container_id)
            .output()
            .await?;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Start container (runsc start)
    async fn start_container(&self, container_id: &str) -> Result<CommandResult> {
        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("start")
            .arg(container_id)
            .output()
            .await?;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Wait for container to complete (runsc wait)
    async fn wait_container(&self, container_id: &str) -> Result<WaitResult> {
        let start_time = std::time::Instant::now();

        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("wait")
            .arg(container_id)
            .output()
            .await?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Parse exit code from stdout (runsc wait outputs exit code)
        let exit_code = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<i32>()
                .unwrap_or(0)
        } else {
            -1
        };

        Ok(WaitResult {
            exit_code,
            duration_ms,
        })
    }

    /// Get container logs (runsc events for stdout/stderr)
    async fn get_container_logs(&self, container_id: &str) -> Result<LogOutput> {
        // Note: runsc doesn't have built-in log capture like Docker
        // We need to redirect stdout/stderr when creating the container
        // or use runsc events to get output

        // For now, return empty logs (will be enhanced with proper log capture)
        Ok(LogOutput {
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    /// Kill container (runsc kill)
    async fn kill_container(&self, container_id: &str) -> Result<()> {
        let _ = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("kill")
            .arg(container_id)
            .arg("SIGKILL")
            .output()
            .await?;

        Ok(())
    }

    /// Delete container (runsc delete)
    async fn delete_container(&self, container_id: &str) -> Result<()> {
        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("delete")
            .arg(container_id)
            .output()
            .await?;

        if !output.status.success() {
            tracing::warn!(
                "Failed to delete container {}: {}",
                container_id,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Check if runsc is available
    pub fn is_available() -> bool {
        which::which("runsc").is_ok()
    }
}

struct CommandResult {
    success: bool,
    stdout: String,
    stderr: String,
}

struct WaitResult {
    exit_code: i32,
    duration_ms: u64,
}

struct LogOutput {
    stdout: String,
    stderr: String,
}

pub struct RunscOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}
```

## 5. Image Cache Strategy

### 5.1 Cache Manager (`cache.rs`)

```rust
/// Image cache with LRU eviction
pub struct ImageCache {
    cache_dir: PathBuf,
    max_size_gb: u64,
    index: Arc<RwLock<CacheIndex>>,
}

struct CacheIndex {
    entries: BTreeMap<String, CacheEntry>,
    total_size: u64,
}

struct CacheEntry {
    image_ref: String,
    layers: Vec<LayerEntry>,
    config_digest: String,
    last_accessed: SystemTime,
    total_size: u64,
}

struct LayerEntry {
    digest: String,
    size: u64,
    path: PathBuf,
}

impl ImageCache {
    /// Create new cache
    pub fn new(max_size_gb: u64) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache directory"))?
            .join("clnrm")
            .join("oci");

        fs::create_dir_all(&cache_dir)?;

        // Load existing index
        let index = Self::load_index(&cache_dir)?;

        Ok(Self {
            cache_dir,
            max_size_gb,
            index: Arc::new(RwLock::new(index)),
        })
    }

    /// Get cached image
    pub async fn get(&self, image_ref: &str) -> Result<Option<OciImage>> {
        let mut index = self.index.write().await;

        if let Some(entry) = index.entries.get_mut(image_ref) {
            // Update last accessed time
            entry.last_accessed = SystemTime::now();

            // Load image from cache
            let image = self.load_from_cache(entry).await?;
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    /// Store image in cache
    pub async fn store(&self, image_ref: &str, image: &OciImage) -> Result<()> {
        let mut index = self.index.write().await;

        // Calculate total size
        let total_size: u64 = image.layers.iter().map(|l| l.size).sum();

        // Check if we need to evict
        while index.total_size + total_size > self.max_size_gb * 1024 * 1024 * 1024 {
            self.evict_lru(&mut index).await?;
        }

        // Store layers
        let mut layer_entries = Vec::new();
        for layer in &image.layers {
            let layer_path = self.cache_dir
                .join("layers")
                .join(&layer.digest.replace(':', "_"));

            fs::create_dir_all(layer_path.parent().unwrap()).await?;
            fs::write(&layer_path, &layer.data).await?;

            layer_entries.push(LayerEntry {
                digest: layer.digest.clone(),
                size: layer.size,
                path: layer_path,
            });
        }

        // Store config
        let config_digest = format!("sha256:{}",
            hex::encode(sha2::Sha256::digest(&image.config_bytes)));
        let config_path = self.cache_dir
            .join("configs")
            .join(&config_digest.replace(':', "_"));

        fs::create_dir_all(config_path.parent().unwrap()).await?;
        fs::write(&config_path, &image.config_bytes).await?;

        // Add to index
        let entry = CacheEntry {
            image_ref: image_ref.to_string(),
            layers: layer_entries,
            config_digest,
            last_accessed: SystemTime::now(),
            total_size,
        };

        index.entries.insert(image_ref.to_string(), entry);
        index.total_size += total_size;

        // Save index
        self.save_index(&index).await?;

        Ok(())
    }

    /// Evict least recently used image
    async fn evict_lru(&self, index: &mut CacheIndex) -> Result<()> {
        // Find LRU entry
        let lru_ref = index.entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, _)| k.clone());

        if let Some(ref_to_remove) = lru_ref {
            if let Some(entry) = index.entries.remove(&ref_to_remove) {
                tracing::info!("Evicting cached image: {}", ref_to_remove);

                // Remove layer files
                for layer in &entry.layers {
                    let _ = fs::remove_file(&layer.path).await;
                }

                // Update total size
                index.total_size -= entry.total_size;
            }
        }

        Ok(())
    }

    /// Load image from cache
    async fn load_from_cache(&self, entry: &CacheEntry) -> Result<OciImage> {
        // Load layers
        let mut layers = Vec::new();
        for layer_entry in &entry.layers {
            let data = fs::read(&layer_entry.path).await?;
            layers.push(OciLayer {
                digest: layer_entry.digest.clone(),
                media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_string(),
                size: layer_entry.size,
                data,
            });
        }

        // Load config
        let config_path = self.cache_dir
            .join("configs")
            .join(&entry.config_digest.replace(':', "_"));
        let config_bytes = fs::read(&config_path).await?;
        let config = serde_json::from_slice(&config_bytes)?;

        Ok(OciImage {
            manifest: OciManifest::default(), // Reconstruct if needed
            config,
            layers,
        })
    }

    /// Load cache index from disk
    fn load_index(cache_dir: &Path) -> Result<CacheIndex> {
        let index_path = cache_dir.join("index.json");
        if index_path.exists() {
            let data = std::fs::read(&index_path)?;
            Ok(serde_json::from_slice(&data)?)
        } else {
            Ok(CacheIndex {
                entries: BTreeMap::new(),
                total_size: 0,
            })
        }
    }

    /// Save cache index to disk
    async fn save_index(&self, index: &CacheIndex) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");
        let data = serde_json::to_vec_pretty(index)?;
        fs::write(&index_path, data).await?;
        Ok(())
    }
}
```

## 6. gVisor Backend Implementation

### 6.1 GvisorBackend (`gvisor.rs`)

```rust
/// gVisor backend using OCI images and runsc
#[derive(Debug, Clone)]
pub struct GvisorBackend {
    image_source: ImageSource,
    image_loader: Arc<OciImageLoader>,
    bundle_builder: Arc<OciBundleBuilder>,
    runsc_executor: Arc<RunscExecutor>,
    policy: Policy,
    timeout: Duration,
}

impl GvisorBackend {
    /// Create new gVisor backend
    pub async fn new(image: impl Into<String>) -> Result<Self> {
        let image_str = image.into();

        // Parse image reference
        let image_source = Self::parse_image_ref(&image_str)?;

        // Initialize components
        let cache = Arc::new(ImageCache::new(10)?); // 10GB cache
        let registry_client = RegistryClient::new()?;
        let local_store = LocalImageStore::new()?;

        let image_loader = Arc::new(OciImageLoader {
            cache,
            registry_client,
            local_store,
        });

        let layer_manager = LayerManager::new()?;
        let config_parser = ConfigParser;
        let bundle_dir = dirs::cache_dir()
            .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache dir"))?
            .join("clnrm")
            .join("bundles");

        let bundle_builder = Arc::new(OciBundleBuilder {
            layer_manager,
            config_parser,
            bundle_dir,
        });

        let runsc_executor = Arc::new(RunscExecutor::new()?);

        Ok(Self {
            image_source,
            image_loader,
            bundle_builder,
            runsc_executor,
            policy: Policy::default(),
            timeout: Duration::from_secs(30),
        })
    }

    /// Parse image reference string
    fn parse_image_ref(image_ref: &str) -> Result<ImageSource> {
        // Check if it's a local path
        if Path::new(image_ref).exists() {
            return Ok(ImageSource::Local {
                path: PathBuf::from(image_ref),
            });
        }

        // Parse as registry reference
        // Format: [registry/]repository[:tag]
        let (registry, repo_tag) = if image_ref.contains('/') {
            let parts: Vec<&str> = image_ref.splitn(2, '/').collect();
            if parts[0].contains('.') || parts[0].contains(':') {
                // Has registry
                (parts[0].to_string(), parts[1].to_string())
            } else {
                // No registry, assume Docker Hub
                ("registry-1.docker.io".to_string(), image_ref.to_string())
            }
        } else {
            // No registry, assume Docker Hub library
            ("registry-1.docker.io".to_string(), format!("library/{}", image_ref))
        };

        let (repository, tag) = if let Some((repo, tag)) = repo_tag.split_once(':') {
            (repo.to_string(), tag.to_string())
        } else {
            (repo_tag, "latest".to_string())
        };

        Ok(ImageSource::Registry {
            registry,
            repository,
            tag,
        })
    }

    /// Set policy
    pub fn with_policy(mut self, policy: Policy) -> Self {
        self.policy = policy;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if gVisor is available
    pub fn is_available() -> bool {
        RunscExecutor::is_available()
    }
}

impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        // Run async operations in blocking context
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                let start_time = Instant::now();

                // 1. Load image
                tracing::info!("Loading OCI image");
                let image = self.image_loader
                    .load_image(self.image_source.clone())
                    .await?;

                // 2. Create OCI bundle
                tracing::info!("Creating OCI bundle");
                let bundle = self.bundle_builder
                    .create_bundle(&image, Some(&cmd))
                    .await?;

                // 3. Execute with runsc
                tracing::info!("Executing with runsc");
                let output = self.runsc_executor
                    .run_container(&bundle, self.timeout)
                    .await?;

                // 4. Cleanup bundle
                tracing::info!("Cleaning up bundle");
                let _ = tokio::fs::remove_dir_all(&bundle.path).await;

                let duration_ms = start_time.elapsed().as_millis() as u64;

                Ok(RunResult {
                    exit_code: output.exit_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    duration_ms,
                    steps: Vec::new(),
                    redacted_env: Vec::new(),
                    backend: "gvisor".to_string(),
                    concurrent: false,
                    step_order: Vec::new(),
                })
            })
        })
    }

    fn name(&self) -> &str {
        "gvisor"
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }

    fn supports_hermetic(&self) -> bool {
        true
    }

    fn supports_deterministic(&self) -> bool {
        true
    }
}
```

## 7. Scenario Integration

Update `/home/user/clnrm/crates/clnrm-core/src/scenario.rs`:

```rust
impl Scenario {
    /// Run with gVisor backend (OCI + runsc)
    pub async fn run_gvisor(self, image: &str) -> Result<RunResult> {
        let backend = crate::backend::GvisorBackend::new(image).await?;
        self.run_with_backend_async(backend).await
    }

    /// Run with auto-detected backend (gVisor if available, otherwise testcontainers)
    pub async fn run_auto(self, image: &str) -> Result<RunResult> {
        if crate::backend::GvisorBackend::is_available() {
            self.run_gvisor(image).await
        } else {
            let backend = crate::backend::TestcontainerBackend::new(image)?;
            self.run_with_backend_async(backend).await
        }
    }
}
```

## 8. Error Handling

```rust
// In error.rs
impl CleanroomError {
    pub fn oci_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::BackendError, msg.into())
    }

    pub fn registry_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::BackendError, format!("Registry error: {}", msg.into()))
    }

    pub fn runsc_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::BackendError, format!("runsc error: {}", msg.into()))
    }
}
```

## 9. Usage Examples

### 9.1 Basic Usage with Alpine

```rust
use clnrm::{scenario, GvisorBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let scenario = scenario("alpine_test")
        .step("test".to_string(), ["echo", "Hello from gVisor!"])
        .step("check".to_string(), ["cat", "/etc/os-release"]);

    let result = scenario.run_gvisor("alpine:latest").await?;
    println!("Output: {}", result.stdout);
    Ok(())
}
```

### 9.2 SurrealDB Integration

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let scenario = scenario("surrealdb_test")
        .step("start_db".to_string(), [
            "surreal", "start",
            "--bind", "0.0.0.0:8000",
            "memory"
        ])
        .timeout_ms(5000);

    let result = scenario.run_gvisor("surrealdb/surrealdb:latest").await?;
    Ok(())
}
```

### 9.3 Custom Application Image

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let scenario = scenario("custom_app_test")
        .step("build".to_string(), ["cargo", "build", "--release"])
        .step("test".to_string(), ["cargo", "test"])
        .step("run".to_string(), ["./target/release/myapp"]);

    let result = scenario.run_gvisor("myregistry.io/myapp:v1.0").await?;
    Ok(())
}
```

## 10. Dependencies to Add

Add to `crates/clnrm-core/Cargo.toml`:

```toml
[dependencies]
# OCI and compression
flate2 = "1.0"
tar = "0.4"
sha2 = "0.10"  # Already present
hex = "0.4"    # Already present

# HTTP client for registry
reqwest = { workspace = true }

# Filesystem utilities
dirs = "5.0"

# Binary lookup
which = "6.0"  # Already present

# Async filesystem
tokio = { workspace = true, features = ["fs"] }
```

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_image_ref_parsing() {
        let source = GvisorBackend::parse_image_ref("alpine:latest").unwrap();
        match source {
            ImageSource::Registry { registry, repository, tag } => {
                assert_eq!(registry, "registry-1.docker.io");
                assert_eq!(repository, "library/alpine");
                assert_eq!(tag, "latest");
            }
            _ => panic!("Expected registry source"),
        }
    }

    #[tokio::test]
    async fn test_layer_extraction() {
        // Test layer extraction with mock data
    }

    #[test]
    fn test_config_parsing() {
        // Test OCI config parsing
    }
}
```

### 11.2 Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires runsc installed
async fn test_gvisor_alpine() {
    let backend = GvisorBackend::new("alpine:latest").await.unwrap();
    let cmd = Cmd::new("echo").arg("test");
    let result = backend.run_cmd(cmd).unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("test"));
}
```

## 12. Migration Path

1. **Phase 1**: Implement OCI loading and caching
2. **Phase 2**: Implement bundle builder and layer manager
3. **Phase 3**: Implement runsc executor
4. **Phase 4**: Create GvisorBackend
5. **Phase 5**: Add auto-detection (fallback to testcontainers)
6. **Phase 6**: Update Scenario API
7. **Phase 7**: Add comprehensive tests
8. **Phase 8**: Deprecate testcontainers (optional)

## 13. Performance Considerations

- **Image caching**: 10GB LRU cache reduces registry pulls
- **Layer reuse**: Shared layers across images save space
- **Bundle pooling**: Reuse bundles for same image
- **Parallel downloads**: Download layers in parallel
- **Compression**: Keep layers compressed until extraction

## 14. Security Considerations

- **Registry authentication**: Bearer token caching
- **Layer verification**: SHA256 digest validation
- **Rootless containers**: gVisor provides strong isolation
- **Read-only rootfs**: Optional for immutability
- **Network policies**: Control container networking

## 15. Future Enhancements

- **OCI artifacts**: Support OCI artifact types
- **Image signing**: cosign/notary integration
- **Multi-platform**: Support arm64, amd64 variants
- **Registry mirrors**: Support local mirrors
- **Offline mode**: Embedded image bundles
- **Container reuse**: Keep containers warm for performance
