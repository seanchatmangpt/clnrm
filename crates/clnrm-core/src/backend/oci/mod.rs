//! OCI image loading and management without Docker daemon
//!
//! This module provides direct OCI image operations:
//! - Loading images from Docker registries
//! - Loading images from local OCI directories
//! - Extracting and merging image layers
//! - Creating OCI bundles for runsc execution

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod bundle_builder;
pub mod cache;
pub mod config_parser;
pub mod image_loader;
pub mod layer_manager;
pub mod registry_client;
pub mod runsc_executor;

pub use bundle_builder::{OciBundle, OciBundleBuilder};
pub use cache::ImageCache;
pub use config_parser::{ConfigParser, RuntimeConfig};
pub use image_loader::{ImageSource, LocalImageStore, OciImageLoader};
pub use layer_manager::LayerManager;
pub use registry_client::RegistryClient;
pub use runsc_executor::{RunscExecutor, RunscOutput};

/// OCI image with manifest, config, and layers
#[derive(Debug, Clone)]
pub struct OciImage {
    pub manifest: OciManifest,
    pub config: OciImageConfig,
    pub layers: Vec<OciLayer>,
    pub config_bytes: Vec<u8>,
}

/// OCI manifest (Docker Image Manifest V2, Schema 2)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
}

/// OCI descriptor for config or layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciDescriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: u64,
    pub digest: String,
}

/// OCI image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciImageConfig {
    pub architecture: String,
    pub os: String,
    pub config: OciContainerConfig,
    pub rootfs: OciRootfs,
    pub history: Option<Vec<OciHistory>>,
}

/// OCI container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciContainerConfig {
    #[serde(rename = "User")]
    pub user: Option<String>,
    #[serde(rename = "ExposedPorts")]
    pub exposed_ports: Option<std::collections::HashMap<String, serde_json::Value>>,
    #[serde(rename = "Env")]
    pub env: Option<Vec<String>>,
    #[serde(rename = "Cmd")]
    pub cmd: Option<Vec<String>>,
    #[serde(rename = "Volumes")]
    pub volumes: Option<std::collections::HashMap<String, serde_json::Value>>,
    #[serde(rename = "WorkingDir")]
    pub working_dir: Option<String>,
    #[serde(rename = "Entrypoint")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(rename = "Labels")]
    pub labels: Option<std::collections::HashMap<String, String>>,
}

/// OCI rootfs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciRootfs {
    #[serde(rename = "type")]
    pub typ: String,
    pub diff_ids: Vec<String>,
}

/// OCI history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciHistory {
    pub created: Option<String>,
    pub created_by: Option<String>,
    pub empty_layer: Option<bool>,
}

/// OCI layer
#[derive(Debug, Clone)]
pub struct OciLayer {
    pub digest: String,
    pub media_type: String,
    pub size: u64,
    pub data: Vec<u8>,
}

/// Runtime config for OCI bundle (config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub terminal: bool,
    pub user: String,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<CapabilitiesConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rlimits: Vec<RlimitConfig>,
    #[serde(rename = "noNewPrivileges")]
    pub no_new_privileges: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesConfig {
    pub bounding: Vec<String>,
    pub effective: Vec<String>,
    pub inheritable: Vec<String>,
    pub permitted: Vec<String>,
    pub ambient: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlimitConfig {
    #[serde(rename = "type")]
    pub typ: String,
    pub hard: u64,
    pub soft: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    pub path: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub destination: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub source: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxConfig {
    pub namespaces: Vec<NamespaceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<serde_json::Value>,
    #[serde(rename = "maskedPaths", skip_serializing_if = "Vec::is_empty", default)]
    pub masked_paths: Vec<String>,
    #[serde(rename = "readonlyPaths", skip_serializing_if = "Vec::is_empty", default)]
    pub readonly_paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oci_manifest_deserialization() {
        let json = r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
            "config": {
                "mediaType": "application/vnd.docker.container.image.v1+json",
                "size": 1234,
                "digest": "sha256:abc123"
            },
            "layers": []
        }"#;

        let manifest: OciManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.schema_version, 2);
        assert_eq!(manifest.config.digest, "sha256:abc123");
    }
}
