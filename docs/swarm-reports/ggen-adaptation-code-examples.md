# GGEN Component Adaptation - Code Examples & Templates

**Companion Document to Adaptation Strategy**
**Date**: 2025-10-17

This document provides ready-to-use code templates and concrete examples for adapting ggen components to your projects.

---

## Table of Contents

1. [PQC Module - Drop-in Integration](#1-pqc-module---drop-in-integration)
2. [OpenTelemetry - Quick Start](#2-opentelemetry---quick-start)
3. [Registry Client - Custom Marketplace](#3-registry-client---custom-marketplace)
4. [Cache Manager - Plugin System](#4-cache-manager---plugin-system)
5. [Lifecycle System - Build Orchestration](#5-lifecycle-system---build-orchestration)
6. [Three-Way Merge - Code Generation](#6-three-way-merge---code-generation)

---

## 1. PQC Module - Drop-in Integration

### File: `src/crypto/pqc.rs`

```rust
//! Post-Quantum Cryptography Module
//!
//! Adapted from ggen-core v1.2.0
//! Original: https://github.com/seanchatmangpt/ggen/blob/main/ggen-core/src/pqc.rs
//! Changes: None (direct copy)

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use pqcrypto_mldsa::mldsa65;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// PQC signer for creating quantum-resistant signatures
pub struct PqcSigner {
    secret_key: mldsa65::SecretKey,
    public_key: mldsa65::PublicKey,
}

impl PqcSigner {
    /// Generate a new keypair
    pub fn new() -> Self {
        let (public_key, secret_key) = mldsa65::keypair();
        Self {
            secret_key,
            public_key,
        }
    }

    /// Load keypair from files
    pub fn from_files(secret_key_path: &Path, public_key_path: &Path) -> Result<Self> {
        let sk_bytes = fs::read(secret_key_path).context("Failed to read secret key")?;
        let pk_bytes = fs::read(public_key_path).context("Failed to read public key")?;

        let secret_key = mldsa65::SecretKey::from_bytes(&sk_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid secret key format"))?;
        let public_key = mldsa65::PublicKey::from_bytes(&pk_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid public key format"))?;

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signed = mldsa65::sign(message, &self.secret_key);
        signed.as_bytes().to_vec()
    }

    /// Sign an artifact (generic version - was sign_pack in ggen)
    pub fn sign_artifact(&self, id: &str, version: &str, sha256: &str) -> String {
        let message = format!("{}:{}:{}", id, version, sha256);
        let signature = self.sign(message.as_bytes());
        general_purpose::STANDARD.encode(&signature)
    }

    /// Get public key as base64
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }
}

/// PQC verifier for checking quantum-resistant signatures
pub struct PqcVerifier {
    public_key: mldsa65::PublicKey,
}

impl PqcVerifier {
    /// Create verifier from base64-encoded public key
    pub fn from_base64(public_key_b64: &str) -> Result<Self> {
        let public_key_bytes = general_purpose::STANDARD
            .decode(public_key_b64)
            .context("Failed to decode public key")?;
        let public_key = mldsa65::PublicKey::from_bytes(&public_key_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid public key format"))?;
        Ok(Self { public_key })
    }

    /// Verify an artifact signature
    pub fn verify_artifact(
        &self,
        id: &str,
        version: &str,
        sha256: &str,
        signature_b64: &str,
    ) -> Result<bool> {
        let message = format!("{}:{}:{}", id, version, sha256);
        let signature = general_purpose::STANDARD
            .decode(signature_b64)
            .context("Failed to decode signature")?;

        match mldsa65::open(
            &SignedMessage::from_bytes(&signature).unwrap(),
            &self.public_key,
        ) {
            Ok(verified_msg) => Ok(verified_msg == message.as_bytes()),
            Err(_) => Ok(false),
        }
    }
}

/// Calculate SHA256 hash of file content
pub fn calculate_sha256_file(path: &Path) -> Result<String> {
    let content = fs::read(path).context("Failed to read file")?;
    Ok(calculate_sha256(&content))
}

/// Calculate SHA256 hash of bytes
pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_sign_and_verify() {
        let signer = PqcSigner::new();
        let signature = signer.sign_artifact("my-plugin", "1.0.0", "abc123");

        let verifier = PqcVerifier::from_base64(&signer.public_key_base64()).unwrap();
        let verified = verifier
            .verify_artifact("my-plugin", "1.0.0", "abc123", &signature)
            .unwrap();

        assert!(verified);
    }
}
```

### File: `Cargo.toml` (dependencies)

```toml
[dependencies]
pqcrypto-mldsa = "0.1"
pqcrypto-traits = "0.3"
sha2 = "0.10"
base64 = "0.22"
anyhow = "1.0"
```

### Usage Example

```rust
use crate::crypto::pqc::{PqcSigner, PqcVerifier, calculate_sha256_file};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Generate keypair (do this once, save to secure storage)
    let signer = PqcSigner::new();
    signer.save_to_files(
        Path::new("keys/secret.key"),
        Path::new("keys/public.key"),
    )?;

    // Sign a release artifact
    let artifact_path = Path::new("dist/my-plugin-1.0.0.tar.gz");
    let sha256 = calculate_sha256_file(artifact_path)?;
    let signature = signer.sign_artifact("my-plugin", "1.0.0", &sha256);

    println!("Signature: {}", signature);
    println!("Public Key: {}", signer.public_key_base64());

    // Later: Verify the signature
    let verifier = PqcVerifier::from_base64(&signer.public_key_base64())?;
    let valid = verifier.verify_artifact("my-plugin", "1.0.0", &sha256, &signature)?;

    assert!(valid, "Signature verification failed!");
    Ok(())
}
```

---

## 2. OpenTelemetry - Quick Start

### File: `src/observability/telemetry.rs`

```rust
//! OpenTelemetry Integration
//!
//! Adapted from ggen-core v1.2.0
//! Changes: Added structured logging helpers

use anyhow::{Context, Result};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{RandomIdGenerator, Sampler},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub endpoint: String,
    pub service_name: String,
    pub sample_ratio: f64,
    pub console_output: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4318".to_string()),
            service_name: env!("CARGO_PKG_NAME").to_string(),
            sample_ratio: 1.0,
            console_output: true,
        }
    }
}

pub fn init_telemetry(config: TelemetryConfig) -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(config.sample_ratio))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", config.service_name.clone()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    KeyValue::new("deployment.environment",
                        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
                    ),
                ])),
        )
        .install_batch(runtime::Tokio)
        .context("Failed to install OTLP tracer")?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(telemetry_layer);

    if config.console_output {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .json(); // Structured JSON logging
        subscriber.with(fmt_layer).init();
    } else {
        subscriber.init();
    }

    tracing::info!(
        endpoint = %config.endpoint,
        service = %config.service_name,
        "OpenTelemetry initialized"
    );

    Ok(())
}

pub fn shutdown_telemetry() {
    tracing::info!("Shutting down OpenTelemetry");
    global::shutdown_tracer_provider();
}

/// Helper macro for creating spans with common attributes
#[macro_export]
macro_rules! traced_operation {
    ($name:expr, $($key:expr => $value:expr),* $(,)?) => {
        {
            let span = tracing::info_span!($name, $($key = tracing::field::Empty),*);
            let _guard = span.enter();
            $(
                span.record($key, &tracing::field::display($value));
            )*
        }
    };
}
```

### File: `src/main.rs`

```rust
use anyhow::Result;
use tracing::{info, instrument};

mod observability;
use observability::telemetry::{init_telemetry, shutdown_telemetry, TelemetryConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry
    let config = TelemetryConfig::default();
    init_telemetry(config)?;

    // Application code with automatic tracing
    process_request("user-123", "create-order").await?;

    // Cleanup
    shutdown_telemetry();
    Ok(())
}

#[instrument(
    name = "app.process_request",
    skip(user_id, action),
    fields(user.id = %user_id, request.action = %action, request.result)
)]
async fn process_request(user_id: &str, action: &str) -> Result<()> {
    info!("Processing request");

    // Your business logic here
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Record result
    tracing::Span::current().record("request.result", "success");
    info!("Request processed successfully");

    Ok(())
}
```

### Docker Compose (for local testing)

```yaml
# docker-compose.yml
version: '3'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "4318:4318"    # OTLP HTTP
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

---

## 3. Registry Client - Custom Marketplace

### File: `src/marketplace/registry.rs`

```rust
//! Plugin Registry Client
//!
//! Adapted from ggen-core v1.2.0 registry.rs
//! Changes: Renamed "pack" to "plugin", added capability filtering

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone)]
pub struct RegistryClient {
    base_url: Url,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub updated: DateTime<Utc>,
    pub plugins: HashMap<String, PluginMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub latest_version: String,
    pub versions: HashMap<String, VersionMetadata>,
    pub capabilities: Vec<String>,  // NEW: Plugin capabilities
    pub tags: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub version: String,
    pub git_url: String,
    pub git_rev: String,
    pub sha256: String,
    pub signature: String,  // NEW: PQC signature
}

#[derive(Debug, Clone)]
pub struct ResolvedPlugin {
    pub id: String,
    pub version: String,
    pub git_url: String,
    pub git_rev: String,
    pub sha256: String,
    pub signature: String,
}

impl RegistryClient {
    pub fn new() -> Result<Self> {
        let registry_url = std::env::var("PLUGIN_REGISTRY_URL")
            .unwrap_or_else(|_| "https://plugins.example.com/registry/".to_string());

        let base_url = Url::parse(&registry_url)?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { base_url, client })
    }

    #[tracing::instrument(name = "registry.fetch_index", skip(self))]
    pub async fn fetch_index(&self) -> Result<RegistryIndex> {
        let url = self.base_url.join("index.json")?;

        let response = self.client.get(url.clone()).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Registry returned status: {}", response.status());
        }

        let index: RegistryIndex = response.json().await?;
        Ok(index)
    }

    #[tracing::instrument(name = "registry.search", skip(self), fields(query, result_count))]
    pub async fn search(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        let index = self.fetch_index().await?;
        let query_lower = query.to_lowercase();

        let results: Vec<_> = index
            .plugins
            .into_values()
            .filter(|plugin| {
                plugin.name.to_lowercase().contains(&query_lower)
                    || plugin.description.to_lowercase().contains(&query_lower)
                    || plugin.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                    || plugin.capabilities.iter().any(|c| c.to_lowercase().contains(&query_lower))
            })
            .collect();

        tracing::Span::current().record("result_count", results.len());
        Ok(results)
    }

    /// NEW: Search by capability
    pub async fn search_by_capability(&self, capability: &str) -> Result<Vec<PluginMetadata>> {
        let index = self.fetch_index().await?;

        let results: Vec<_> = index
            .plugins
            .into_values()
            .filter(|plugin| {
                plugin.capabilities.iter().any(|c| c == capability)
            })
            .collect();

        Ok(results)
    }

    #[tracing::instrument(name = "registry.resolve", skip(self))]
    pub async fn resolve(&self, plugin_id: &str, version: Option<&str>) -> Result<ResolvedPlugin> {
        let index = self.fetch_index().await?;

        let plugin = index
            .plugins
            .get(plugin_id)
            .with_context(|| format!("Plugin '{}' not found", plugin_id))?;

        let target_version = version.unwrap_or(&plugin.latest_version);

        let version_meta = plugin
            .versions
            .get(target_version)
            .with_context(|| format!("Version '{}' not found", target_version))?;

        Ok(ResolvedPlugin {
            id: plugin_id.to_string(),
            version: target_version.to_string(),
            git_url: version_meta.git_url.clone(),
            git_rev: version_meta.git_rev.clone(),
            sha256: version_meta.sha256.clone(),
            signature: version_meta.signature.clone(),
        })
    }
}
```

### File: `registry/index.json` (example registry)

```json
{
  "updated": "2025-10-17T00:00:00Z",
  "plugins": {
    "auth-plugin": {
      "id": "auth-plugin",
      "name": "Authentication Plugin",
      "description": "OAuth2 and JWT authentication",
      "author": "YourOrg",
      "latest_version": "1.0.0",
      "capabilities": ["oauth2", "jwt", "session-management"],
      "tags": ["auth", "security"],
      "license": "MIT",
      "repository": "https://github.com/yourorg/auth-plugin",
      "versions": {
        "1.0.0": {
          "version": "1.0.0",
          "git_url": "https://github.com/yourorg/auth-plugin.git",
          "git_rev": "v1.0.0",
          "sha256": "abc123...",
          "signature": "base64_pqc_signature..."
        }
      }
    }
  }
}
```

---

## 4. Cache Manager - Plugin System

### File: `src/marketplace/cache.rs`

```rust
//! Plugin Cache Manager
//!
//! Adapted from ggen-core v1.2.0 cache.rs

use anyhow::{Context, Result};
use git2::{FetchOptions, RemoteCallbacks, Repository};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::marketplace::registry::ResolvedPlugin;
use crate::crypto::pqc::{PqcVerifier, calculate_sha256};

#[derive(Debug, Clone)]
pub struct CacheManager {
    cache_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CachedPlugin {
    pub id: String,
    pub version: String,
    pub path: PathBuf,
    pub sha256: String,
    pub verified: bool,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to find cache directory")?
            .join("your-app")
            .join("plugins");

        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    #[tracing::instrument(name = "cache.ensure", skip(self, resolved), fields(plugin_id = %resolved.id, version = %resolved.version))]
    pub async fn ensure(&self, resolved: &ResolvedPlugin, verifier: &PqcVerifier) -> Result<CachedPlugin> {
        let plugin_dir = self.cache_dir.join(&resolved.id).join(&resolved.version);

        // Check cache
        if plugin_dir.exists() {
            if let Ok(cached) = self.load_cached(&resolved.id, &resolved.version) {
                if cached.sha256 == resolved.sha256 {
                    tracing::info!("Cache hit");
                    return Ok(cached);
                } else {
                    tracing::warn!("Cache corrupted, re-downloading");
                    fs::remove_dir_all(&plugin_dir)?;
                }
            }
        }

        // Download
        tracing::info!("Downloading plugin");
        self.download_plugin(resolved, &plugin_dir).await?;

        // Verify
        let actual_sha256 = self.calculate_sha256(&plugin_dir)?;
        if actual_sha256 != resolved.sha256 {
            anyhow::bail!("SHA256 mismatch: expected {}, got {}", resolved.sha256, actual_sha256);
        }

        let verified = verifier
            .verify_artifact(&resolved.id, &resolved.version, &resolved.sha256, &resolved.signature)
            .unwrap_or(false);

        if !verified {
            anyhow::bail!("Signature verification failed for {}@{}", resolved.id, resolved.version);
        }

        tracing::info!(verified = true, "Plugin verified");

        Ok(CachedPlugin {
            id: resolved.id.clone(),
            version: resolved.version.clone(),
            path: plugin_dir,
            sha256: actual_sha256,
            verified: true,
        })
    }

    async fn download_plugin(&self, resolved: &ResolvedPlugin, plugin_dir: &Path) -> Result<()> {
        let parent_dir = plugin_dir.parent().unwrap();
        fs::create_dir_all(parent_dir)?;

        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();

        callbacks.transfer_progress(|stats| {
            if stats.received_objects() % 100 == 0 {
                tracing::debug!(objects = stats.received_objects(), "Download progress");
            }
            true
        });

        fetch_options.remote_callbacks(callbacks);

        let temp_dir = TempDir::new()?;
        let repo = Repository::clone(&resolved.git_url, temp_dir.path())?;

        let object = repo.revparse_single(&resolved.git_rev)?;
        repo.checkout_tree(&object, None)?;

        fs::rename(temp_dir.path(), plugin_dir)?;
        Ok(())
    }

    pub fn load_cached(&self, plugin_id: &str, version: &str) -> Result<CachedPlugin> {
        let plugin_dir = self.cache_dir.join(plugin_id).join(version);

        if !plugin_dir.exists() {
            anyhow::bail!("Plugin not cached: {}@{}", plugin_id, version);
        }

        let sha256 = self.calculate_sha256(&plugin_dir)?;

        Ok(CachedPlugin {
            id: plugin_id.to_string(),
            version: version.to_string(),
            path: plugin_dir,
            sha256,
            verified: false, // Need to re-verify
        })
    }

    fn calculate_sha256(&self, dir: &Path) -> Result<String> {
        // Calculate SHA256 of directory contents
        let mut hasher = Sha256::new();
        for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
            let entry = entry?;
            if entry.file_type().is_file() {
                let content = fs::read(entry.path())?;
                hasher.update(&content);
            }
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}
```

---

## 5. Lifecycle System - Build Orchestration

### File: `make.toml` (configuration)

```toml
[project]
name = "my-fullstack-app"
type = "monorepo"
version = "1.0.0"

[workspace.backend]
path = "services/api"
framework = "rust"
runtime = "tokio"
package_manager = "cargo"

[workspace.frontend]
path = "apps/web"
framework = "react"
runtime = "node"
package_manager = "npm"

[workspace.database]
path = "database"
framework = "postgres"

[lifecycle.setup]
description = "Install dependencies for all workspaces"
parallel = true
workspaces = ["backend", "frontend"]
commands = [
    "cargo build",  # For backend
    "npm install"   # For frontend
]

[lifecycle.build]
description = "Build all services"
parallel = true
workspaces = ["backend", "frontend"]
outputs = [
    "services/api/target/release/api",
    "apps/web/dist/**"
]
cache = true

[lifecycle.test]
description = "Run all tests"
commands = [
    "cargo test --workspace",
    "npm test"
]

[lifecycle.deploy]
description = "Deploy to production"
workspaces = ["backend", "frontend", "database"]
parallel = false  # Sequential deployment

[hooks]
before_build = [
    "./scripts/check-dependencies.sh",
    "./scripts/generate-env.sh"
]

after_build = [
    "./scripts/run-linters.sh"
]

before_deploy = [
    "./scripts/run-migrations.sh",
    "./scripts/health-check.sh"
]

after_deploy = [
    "./scripts/smoke-tests.sh",
    "./scripts/notify-slack.sh"
]
```

### File: `src/lifecycle/mod.rs` (simplified version)

```rust
//! Lifecycle Orchestration
//!
//! Adapted from ggen-core v1.2.0 lifecycle module
//! Simplified for essential features only

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Make {
    pub project: Project,
    pub workspace: Option<BTreeMap<String, Workspace>>,
    pub lifecycle: BTreeMap<String, Phase>,
    pub hooks: Option<Hooks>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Workspace {
    pub path: String,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Phase {
    pub description: Option<String>,
    pub commands: Option<Vec<String>>,
    pub parallel: Option<bool>,
    pub workspaces: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Hooks {
    pub before_build: Option<Vec<String>>,
    pub after_build: Option<Vec<String>>,
    pub before_deploy: Option<Vec<String>>,
    pub after_deploy: Option<Vec<String>>,
}

pub fn load_make<P: AsRef<Path>>(path: P) -> Result<Make> {
    let content = std::fs::read_to_string(path)?;
    let make: Make = toml::from_str(&content)?;
    Ok(make)
}

#[tracing::instrument(name = "lifecycle.run_phase", skip(make), fields(phase = %phase_name))]
pub async fn run_phase(phase_name: &str, make: &Make) -> Result<()> {
    tracing::info!("Running phase: {}", phase_name);

    // Execute hooks before
    if let Some(hooks) = &make.hooks {
        execute_hooks(&get_before_hooks(hooks, phase_name)).await?;
    }

    // Execute phase
    if let Some(phase) = make.lifecycle.get(phase_name) {
        execute_phase(phase).await?;
    }

    // Execute hooks after
    if let Some(hooks) = &make.hooks {
        execute_hooks(&get_after_hooks(hooks, phase_name)).await?;
    }

    tracing::info!("Phase completed: {}", phase_name);
    Ok(())
}

async fn execute_phase(phase: &Phase) -> Result<()> {
    if let Some(commands) = &phase.commands {
        for cmd_str in commands {
            execute_command(cmd_str).await?;
        }
    }
    Ok(())
}

async fn execute_command(cmd_str: &str) -> Result<()> {
    tracing::info!("Executing: {}", cmd_str);

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let (program, args) = parts.split_first().unwrap();

    let output = Command::new(program).args(args).output()?;

    if !output.status.success() {
        anyhow::bail!("Command failed: {}", cmd_str);
    }

    Ok(())
}

async fn execute_hooks(hooks: &[String]) -> Result<()> {
    for hook in hooks {
        execute_command(hook).await?;
    }
    Ok(())
}

fn get_before_hooks(hooks: &Hooks, phase: &str) -> Vec<String> {
    match phase {
        "build" => hooks.before_build.clone().unwrap_or_default(),
        "deploy" => hooks.before_deploy.clone().unwrap_or_default(),
        _ => vec![],
    }
}

fn get_after_hooks(hooks: &Hooks, phase: &str) -> Vec<String> {
    match phase {
        "build" => hooks.after_build.clone().unwrap_or_default(),
        "deploy" => hooks.after_deploy.clone().unwrap_or_default(),
        _ => vec![],
    }
}
```

### Usage Example

```rust
use crate::lifecycle::{load_make, run_phase};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let make = load_make("make.toml")?;

    // Run phases
    run_phase("setup", &make).await?;
    run_phase("build", &make).await?;
    run_phase("test", &make).await?;
    run_phase("deploy", &make).await?;

    Ok(())
}
```

---

## 6. Three-Way Merge - Code Generation

### File: `src/codegen/merge.rs`

```rust
//! Three-Way Merge for Generated Code
//!
//! Adapted from ggen-core v1.2.0 merge.rs

use anyhow::Result;
use std::path::Path;

pub enum MergeStrategy {
    GeneratedWins,
    ManualWins,
    Interactive,
}

pub struct ThreeWayMerger {
    strategy: MergeStrategy,
}

impl ThreeWayMerger {
    pub fn new(strategy: MergeStrategy) -> Self {
        Self { strategy }
    }

    pub fn merge(
        &self,
        baseline: &str,
        generated: &str,
        current: &str,
        _file_path: &Path,
    ) -> Result<String> {
        match self.strategy {
            MergeStrategy::GeneratedWins => Ok(generated.to_string()),
            MergeStrategy::ManualWins => Ok(current.to_string()),
            MergeStrategy::Interactive => {
                // Simple region-based merge
                self.region_aware_merge(baseline, generated, current)
            }
        }
    }

    fn region_aware_merge(
        &self,
        _baseline: &str,
        generated: &str,
        current: &str,
    ) -> Result<String> {
        let mut result = String::new();
        let mut in_manual_region = false;

        for line in current.lines() {
            if line.contains("// BEGIN:MANUAL") {
                in_manual_region = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if line.contains("// END:MANUAL") {
                in_manual_region = false;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_manual_region {
                // Preserve manual content
                result.push_str(line);
                result.push('\n');
            }
        }

        // Merge generated content
        let mut in_generated_region = false;
        for line in generated.lines() {
            if line.contains("// BEGIN:GENERATED") {
                in_generated_region = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if line.contains("// END:GENERATED") {
                in_generated_region = false;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_generated_region {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(result)
    }
}
```

### File: `templates/api_endpoint.rs.template`

```rust
// BEGIN:GENERATED
// This section is auto-generated. Do not edit directly.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct {{struct_name}} {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_{{resource}}(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<{{struct_name}}>, StatusCode> {
    // END:GENERATED

    // BEGIN:MANUAL
    // Add your custom logic here
    let item = state.db.get(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(item))
    // END:MANUAL

    // BEGIN:GENERATED
}
// END:GENERATED
```

### Usage Example

```rust
use crate::codegen::merge::{ThreeWayMerger, MergeStrategy};
use std::path::Path;

fn regenerate_code() -> anyhow::Result<()> {
    // Previous generation
    let baseline = std::fs::read_to_string("src/api/users.rs")?;

    // New generation
    let generated = generate_from_template("templates/api_endpoint.rs.template", &context)?;

    // Current file (with manual edits)
    let current = std::fs::read_to_string("src/api/users.rs")?;

    // Merge
    let merger = ThreeWayMerger::new(MergeStrategy::Interactive);
    let merged = merger.merge(&baseline, &generated, &current, Path::new("src/api/users.rs"))?;

    // Write back
    std::fs::write("src/api/users.rs", merged)?;

    Ok(())
}
```

---

## Complete Integration Example

### Project Structure

```
my-project/
├── Cargo.toml
├── make.toml
├── src/
│   ├── main.rs
│   ├── crypto/
│   │   └── pqc.rs
│   ├── observability/
│   │   └── telemetry.rs
│   ├── marketplace/
│   │   ├── registry.rs
│   │   └── cache.rs
│   ├── lifecycle/
│   │   └── mod.rs
│   └── codegen/
│       └── merge.rs
├── keys/
│   ├── secret.key
│   └── public.key
└── registry/
    └── index.json
```

### File: `Cargo.toml`

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# PQC
pqcrypto-mldsa = "0.1"
pqcrypto-traits = "0.3"
sha2 = "0.10"
base64 = "0.22"

# OpenTelemetry
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
opentelemetry_sdk = { version = "0.21", features = ["rt-tokio"] }
tracing = "0.1"
tracing-opentelemetry = "0.22"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Registry
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
url = "2.5"
semver = "1.0"

# Cache
git2 = { version = "0.20", features = ["vendored-openssl"] }
dirs = "6.0"
tempfile = "3"
walkdir = "2.5"

# Lifecycle
toml = "0.9"

# Common
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "2.0"
tokio = { version = "1", features = ["full"] }
```

### File: `src/main.rs`

```rust
use anyhow::Result;
use tracing::info;

mod crypto;
mod observability;
mod marketplace;
mod lifecycle;

use crypto::pqc::{PqcSigner, PqcVerifier};
use observability::telemetry::{init_telemetry, shutdown_telemetry, TelemetryConfig};
use marketplace::{registry::RegistryClient, cache::CacheManager};
use lifecycle::{load_make, run_phase};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry
    let telemetry_config = TelemetryConfig::default();
    init_telemetry(telemetry_config)?;

    info!("Application starting");

    // Load lifecycle configuration
    let make = load_make("make.toml")?;

    // Run build phases
    run_phase("setup", &make).await?;
    run_phase("build", &make).await?;

    // Marketplace operations
    let registry = RegistryClient::new()?;
    let cache = CacheManager::new()?;

    // Search for plugins
    let plugins = registry.search("auth").await?;
    info!(count = plugins.len(), "Found plugins");

    // Download and verify first plugin
    if let Some(plugin_meta) = plugins.first() {
        let resolved = registry.resolve(&plugin_meta.id, None).await?;

        // Verify signature
        let verifier = PqcVerifier::from_base64(&plugin_meta.signature)?;
        let cached = cache.ensure(&resolved, &verifier).await?;

        info!(
            plugin_id = %cached.id,
            version = %cached.version,
            verified = cached.verified,
            "Plugin cached"
        );
    }

    info!("Application finished");
    shutdown_telemetry();
    Ok(())
}
```

---

## Testing Adapted Components

### Integration Test Template

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_full_workflow() -> Result<()> {
        // Setup
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("cache");

        // Components
        let registry = RegistryClient::new()?;
        let cache = CacheManager::with_dir(cache_dir)?;
        let signer = PqcSigner::new();
        let verifier = PqcVerifier::from_base64(&signer.public_key_base64())?;

        // Test workflow
        let plugins = registry.search("test").await?;
        assert!(!plugins.is_empty());

        let resolved = registry.resolve(&plugins[0].id, None).await?;
        let cached = cache.ensure(&resolved, &verifier).await?;

        assert!(cached.path.exists());
        assert!(cached.verified);

        Ok(())
    }
}
```

---

## Next Steps Checklist

After integrating components:

- [ ] Update module paths in imports
- [ ] Customize error types for your domain
- [ ] Add feature flags in `Cargo.toml`
- [ ] Write integration tests
- [ ] Add documentation examples
- [ ] Set up CI/CD for verification
- [ ] Configure observability backends
- [ ] Create registry infrastructure
- [ ] Implement key management strategy
- [ ] Define lifecycle phases for your project

---

**Document End**
**SystemArchitect**: Claude (Sonnet 4.5)
**Date**: 2025-10-17
