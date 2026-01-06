# gVisor Isolation Design: Complete Docker Elimination

**Status**: Design Document
**Branch**: claude/gvisor-testcontainers-replacement-7o2EO
**Goal**: Replace testcontainers/Docker with direct gvisor (runsc) for network and filesystem isolation

## Executive Summary

This document details the complete architecture for eliminating Docker dependency by using gvisor's `runsc` directly for container execution. The design provides comprehensive network and filesystem isolation without requiring Docker daemon or CLI.

**Key Benefits**:
- ✅ No Docker daemon required
- ✅ No docker CLI dependency
- ✅ Reduced attack surface (gvisor sandbox)
- ✅ Better resource control
- ✅ Faster startup (no daemon overhead)
- ✅ Hermetic isolation guarantees

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Cleanroom Test Execution                     │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                   GvisorBackend (New)                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Network    │  │  Filesystem  │  │   Resource   │          │
│  │   Manager    │  │   Manager    │  │   Manager    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Namespace & OCI Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Network NS  │  │  Mount NS    │  │   User NS    │          │
│  │  Management  │  │  Management  │  │  Management  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    gVisor (runsc)                                │
│  ┌──────────────────────────────────────────────────────┐       │
│  │  Sandboxed Container (ptrace/kvm platform)           │       │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐     │       │
│  │  │ User Space │  │  Network   │  │ Filesystem │     │       │
│  │  │   Kernel   │  │   Stack    │  │   Stack    │     │       │
│  │  └────────────┘  └────────────┘  └────────────┘     │       │
│  └──────────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Host Kernel                                 │
│  (Minimal syscall exposure via seccomp/ptrace)                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. Network Isolation

### 1.1 Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                       Network Manager                             │
│  ┌────────────────────────────────────────────────────────┐      │
│  │  NetworkManager::new()                                  │      │
│  │    ├─ Create network namespace (unshare -n)            │      │
│  │    ├─ Setup veth pair (ip link add)                    │      │
│  │    ├─ Configure IP allocation                          │      │
│  │    ├─ Setup iptables/nftables rules                    │      │
│  │    └─ Configure DNS resolution                         │      │
│  └────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌──────────┐        ┌──────────┐        ┌──────────┐
    │  veth0   │◄──────►│  veth1   │        │   DNS    │
    │ (host)   │        │(container)│        │ Resolver │
    └──────────┘        └──────────┘        └──────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    Host Network       Container Network     /etc/resolv.conf
    10.0.0.1/24        10.0.0.2/24          nameserver 8.8.8.8
```

### 1.2 Implementation Strategy

#### 1.2.1 Network Namespace Creation

```rust
// File: crates/clnrm-core/src/backend/gvisor/network.rs

use std::process::Command;
use std::path::PathBuf;

pub struct NetworkManager {
    namespace_path: PathBuf,
    container_ip: String,
    host_ip: String,
    subnet: String,
    veth_host: String,
    veth_container: String,
}

impl NetworkManager {
    /// Create isolated network namespace for container
    pub fn create_namespace(&self, container_id: &str) -> Result<()> {
        // 1. Create network namespace
        self.exec_netns(&[
            "add",
            container_id,
        ])?;

        // 2. Create veth pair
        self.exec_ip(&[
            "link", "add",
            &self.veth_host, "type", "veth",
            "peer", "name", &self.veth_container,
        ])?;

        // 3. Move container veth to namespace
        self.exec_ip(&[
            "link", "set",
            &self.veth_container,
            "netns", container_id,
        ])?;

        // 4. Configure host veth
        self.exec_ip(&[
            "addr", "add",
            &format!("{}/24", self.host_ip),
            "dev", &self.veth_host,
        ])?;

        self.exec_ip(&[
            "link", "set",
            &self.veth_host, "up",
        ])?;

        // 5. Configure container veth (in namespace)
        self.exec_ip_netns(container_id, &[
            "addr", "add",
            &format!("{}/24", self.container_ip),
            "dev", &self.veth_container,
        ])?;

        self.exec_ip_netns(container_id, &[
            "link", "set",
            &self.veth_container, "up",
        ])?;

        // 6. Add default route in container
        self.exec_ip_netns(container_id, &[
            "route", "add", "default",
            "via", &self.host_ip,
        ])?;

        Ok(())
    }

    fn exec_netns(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("ip")
            .arg("netns")
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to create network namespace: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
        Ok(())
    }

    fn exec_ip(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("ip")
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to configure network: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
        Ok(())
    }

    fn exec_ip_netns(&self, netns: &str, args: &[&str]) -> Result<()> {
        let output = Command::new("ip")
            .arg("netns")
            .arg("exec")
            .arg(netns)
            .arg("ip")
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to configure network in namespace: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
        Ok(())
    }
}
```

#### 1.2.2 IP Allocation Strategy

```rust
// File: crates/clnrm-core/src/backend/gvisor/ip_allocator.rs

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::net::Ipv4Addr;

/// Thread-safe IP address allocator for container networks
pub struct IpAllocator {
    allocated: Arc<Mutex<HashSet<Ipv4Addr>>>,
    subnet_base: Ipv4Addr,
    subnet_mask: u8,
    next_offset: Arc<Mutex<u32>>,
}

impl IpAllocator {
    /// Create allocator for subnet (e.g., 10.88.0.0/16)
    pub fn new(subnet: &str) -> Result<Self> {
        let (base, mask) = parse_cidr(subnet)?;

        Ok(Self {
            allocated: Arc::new(Mutex::new(HashSet::new())),
            subnet_base: base,
            subnet_mask: mask,
            next_offset: Arc::new(Mutex::new(2)), // Reserve .0 and .1
        })
    }

    /// Allocate next available IP in subnet
    pub fn allocate(&self) -> Result<Ipv4Addr> {
        let mut allocated = self.allocated.lock().unwrap();
        let mut offset = self.next_offset.lock().unwrap();

        let max_hosts = (1u32 << (32 - self.subnet_mask)) - 2;

        loop {
            if *offset >= max_hosts {
                return Err("Subnet exhausted".into());
            }

            let ip = self.offset_to_ip(*offset);
            *offset += 1;

            if !allocated.contains(&ip) {
                allocated.insert(ip);
                return Ok(ip);
            }
        }
    }

    /// Release IP back to pool
    pub fn release(&self, ip: Ipv4Addr) {
        let mut allocated = self.allocated.lock().unwrap();
        allocated.remove(&ip);
    }

    fn offset_to_ip(&self, offset: u32) -> Ipv4Addr {
        let base_int = u32::from(self.subnet_base);
        Ipv4Addr::from(base_int + offset)
    }
}

fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, u8)> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid CIDR format".into());
    }

    let addr: Ipv4Addr = parts[0].parse()?;
    let mask: u8 = parts[1].parse()?;

    if mask > 32 {
        return Err("Invalid subnet mask".into());
    }

    Ok((addr, mask))
}
```

#### 1.2.3 Port Mapping (No Daemon Required)

```rust
// File: crates/clnrm-core/src/backend/gvisor/port_mapper.rs

use std::process::Command;

pub struct PortMapper {
    container_id: String,
    host_ip: String,
    container_ip: String,
}

impl PortMapper {
    /// Map host port to container port using iptables
    pub fn map_port(&self, host_port: u16, container_port: u16) -> Result<()> {
        // DNAT rule: redirect host traffic to container
        self.exec_iptables(&[
            "-t", "nat",
            "-A", "PREROUTING",
            "-p", "tcp",
            "-d", &self.host_ip,
            "--dport", &host_port.to_string(),
            "-j", "DNAT",
            "--to-destination", &format!("{}:{}", self.container_ip, container_port),
        ])?;

        // Masquerade for return traffic
        self.exec_iptables(&[
            "-t", "nat",
            "-A", "POSTROUTING",
            "-p", "tcp",
            "-s", &self.container_ip,
            "--sport", &container_port.to_string(),
            "-j", "MASQUERADE",
        ])?;

        Ok(())
    }

    /// Remove port mapping
    pub fn unmap_port(&self, host_port: u16, container_port: u16) -> Result<()> {
        // Delete DNAT rule
        self.exec_iptables(&[
            "-t", "nat",
            "-D", "PREROUTING",
            "-p", "tcp",
            "-d", &self.host_ip,
            "--dport", &host_port.to_string(),
            "-j", "DNAT",
            "--to-destination", &format!("{}:{}", self.container_ip, container_port),
        ])?;

        // Delete masquerade rule
        self.exec_iptables(&[
            "-t", "nat",
            "-D", "POSTROUTING",
            "-p", "tcp",
            "-s", &self.container_ip,
            "--sport", &container_port.to_string(),
            "-j", "MASQUERADE",
        ])?;

        Ok(())
    }

    fn exec_iptables(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("iptables")
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to configure iptables: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
        Ok(())
    }
}
```

#### 1.2.4 DNS Resolution

```rust
// File: crates/clnrm-core/src/backend/gvisor/dns.rs

use std::fs;
use std::path::Path;

pub struct DnsResolver {
    rootfs_path: String,
}

impl DnsResolver {
    /// Configure DNS in container rootfs
    pub fn configure(&self, nameservers: &[&str]) -> Result<()> {
        let resolv_conf_path = Path::new(&self.rootfs_path)
            .join("etc")
            .join("resolv.conf");

        // Ensure /etc directory exists
        fs::create_dir_all(
            Path::new(&self.rootfs_path).join("etc")
        )?;

        let mut content = String::new();
        for ns in nameservers {
            content.push_str(&format!("nameserver {}\n", ns));
        }

        // Add search domain and options
        content.push_str("search localdomain\n");
        content.push_str("options ndots:0\n");

        fs::write(&resolv_conf_path, content)?;

        Ok(())
    }
}
```

### 1.3 Network Namespace Management

```rust
// File: crates/clnrm-core/src/backend/gvisor/netns.rs

use std::fs;
use std::path::PathBuf;

pub struct NetnsManager {
    netns_dir: PathBuf,
}

impl NetnsManager {
    pub fn new() -> Self {
        Self {
            netns_dir: PathBuf::from("/var/run/netns"),
        }
    }

    /// Check if network namespace exists
    pub fn exists(&self, name: &str) -> bool {
        self.netns_dir.join(name).exists()
    }

    /// List all network namespaces
    pub fn list(&self) -> Result<Vec<String>> {
        let mut namespaces = Vec::new();

        if !self.netns_dir.exists() {
            return Ok(namespaces);
        }

        for entry in fs::read_dir(&self.netns_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                namespaces.push(name.to_string());
            }
        }

        Ok(namespaces)
    }

    /// Cleanup orphaned namespaces
    pub fn cleanup_orphaned(&self, active_containers: &[&str]) -> Result<usize> {
        let all_netns = self.list()?;
        let mut cleaned = 0;

        for netns in all_netns {
            // Skip active containers
            if active_containers.contains(&netns.as_str()) {
                continue;
            }

            // Check if namespace is still in use
            if !self.is_in_use(&netns)? {
                self.delete(&netns)?;
                cleaned += 1;
            }
        }

        Ok(cleaned)
    }

    /// Check if namespace is in use
    fn is_in_use(&self, name: &str) -> Result<bool> {
        // Try to enter namespace - if it fails, it's orphaned
        let output = std::process::Command::new("ip")
            .args(&["netns", "exec", name, "true"])
            .output()?;

        Ok(output.status.success())
    }

    /// Delete namespace
    fn delete(&self, name: &str) -> Result<()> {
        let output = std::process::Command::new("ip")
            .args(&["netns", "delete", name])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to delete namespace {}: {}",
                name,
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        Ok(())
    }
}
```

---

## 2. Filesystem Isolation

### 2.1 Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                    Filesystem Manager                             │
│  ┌────────────────────────────────────────────────────────┐      │
│  │  1. Pull OCI Image (skopeo/umoci)                      │      │
│  │     ├─ skopeo copy docker://alpine:latest oci:...     │      │
│  │     └─ umoci unpack --image oci:alpine bundle          │      │
│  │                                                         │      │
│  │  2. Setup Rootfs                                       │      │
│  │     ├─ Extract layers to rootfs/                      │      │
│  │     ├─ Apply whiteouts (.wh. files)                   │      │
│  │     └─ Set permissions and ownership                  │      │
│  │                                                         │      │
│  │  3. Configure Mounts                                   │      │
│  │     ├─ Setup /proc (proc type)                        │      │
│  │     ├─ Setup /sys (sysfs type)                        │      │
│  │     ├─ Setup /dev (tmpfs + bind mounts)              │      │
│  │     ├─ Setup /tmp (tmpfs)                             │      │
│  │     └─ Bind mount volumes                             │      │
│  │                                                         │      │
│  │  4. Mount Propagation                                  │      │
│  │     ├─ Set MS_PRIVATE (default isolation)            │      │
│  │     ├─ Set MS_SLAVE for volumes (one-way)            │      │
│  │     └─ Set MS_SHARED for special cases               │      │
│  └────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌──────────┐        ┌──────────┐        ┌──────────┐
    │  Rootfs  │        │  Tmpfs   │        │  Volumes │
    │  Layer   │        │  Mounts  │        │   Bind   │
    └──────────┘        └──────────┘        └──────────┘
```

### 2.2 Implementation Strategy

#### 2.2.1 OCI Image Handling

```rust
// File: crates/clnrm-core/src/backend/gvisor/oci.rs

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct OciImageManager {
    cache_dir: PathBuf,
}

impl OciImageManager {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Pull OCI image using skopeo
    pub fn pull_image(&self, image_ref: &str) -> Result<PathBuf> {
        let image_hash = self.image_hash(image_ref);
        let oci_dir = self.cache_dir.join("oci").join(&image_hash);

        // Skip if already cached
        if oci_dir.exists() {
            return Ok(oci_dir);
        }

        fs::create_dir_all(&oci_dir)?;

        // Pull image with skopeo
        let output = Command::new("skopeo")
            .args(&[
                "copy",
                &format!("docker://{}", image_ref),
                &format!("oci:{}:latest", oci_dir.display()),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to pull image {}: {}",
                image_ref,
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        Ok(oci_dir)
    }

    /// Unpack OCI image to rootfs using umoci
    pub fn unpack_image(&self, oci_dir: &Path, bundle_dir: &Path) -> Result<()> {
        fs::create_dir_all(bundle_dir)?;

        // Unpack with umoci
        let output = Command::new("umoci")
            .args(&[
                "unpack",
                "--image",
                &format!("{}:latest", oci_dir.display()),
                bundle_dir.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to unpack image: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        Ok(())
    }

    /// Alternative: Manual layer extraction (if umoci not available)
    pub fn extract_layers_manual(&self, oci_dir: &Path, rootfs_dir: &Path) -> Result<()> {
        // Read manifest
        let manifest = self.read_manifest(oci_dir)?;

        // Extract each layer in order
        for layer in manifest.layers {
            let layer_path = oci_dir.join("blobs").join(&layer.digest);
            self.extract_layer(&layer_path, rootfs_dir)?;
        }

        Ok(())
    }

    fn extract_layer(&self, layer_path: &Path, rootfs_dir: &Path) -> Result<()> {
        // Extract tar.gz layer
        let output = Command::new("tar")
            .args(&[
                "-xzf",
                layer_path.to_str().unwrap(),
                "-C",
                rootfs_dir.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to extract layer: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        // Handle whiteouts (.wh. files)
        self.apply_whiteouts(rootfs_dir)?;

        Ok(())
    }

    fn apply_whiteouts(&self, rootfs_dir: &Path) -> Result<()> {
        // Walk directory tree and remove files marked with .wh. prefix
        for entry in walkdir::WalkDir::new(rootfs_dir) {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy();

            if filename.starts_with(".wh.") {
                let target = entry.path().parent().unwrap()
                    .join(&filename[4..]); // Remove .wh. prefix

                // Remove whiteout marker and target file
                fs::remove_file(entry.path())?;
                if target.exists() {
                    fs::remove_file(target)?;
                }
            }
        }

        Ok(())
    }

    fn image_hash(&self, image_ref: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(image_ref.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn read_manifest(&self, oci_dir: &Path) -> Result<OciManifest> {
        let manifest_path = oci_dir.join("index.json");
        let content = fs::read_to_string(manifest_path)?;
        let manifest: OciManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }
}

#[derive(serde::Deserialize)]
struct OciManifest {
    layers: Vec<OciLayer>,
}

#[derive(serde::Deserialize)]
struct OciLayer {
    digest: String,
    #[serde(rename = "mediaType")]
    media_type: String,
}
```

#### 2.2.2 Mount Configuration

```rust
// File: crates/clnrm-core/src/backend/gvisor/mount.rs

use std::path::Path;

pub struct MountManager {
    rootfs_path: String,
}

#[derive(Debug, Clone)]
pub enum MountPropagation {
    Private,   // MS_PRIVATE - completely isolated
    Slave,     // MS_SLAVE - receive from parent only
    Shared,    // MS_SHARED - bidirectional propagation
}

#[derive(Debug, Clone)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub fs_type: String,
    pub options: Vec<String>,
    pub propagation: MountPropagation,
}

impl MountManager {
    /// Generate OCI runtime spec mounts for gvisor
    pub fn generate_mounts(&self) -> Vec<Mount> {
        vec![
            // /proc - process information
            Mount {
                source: "proc".to_string(),
                destination: "/proc".to_string(),
                fs_type: "proc".to_string(),
                options: vec!["nosuid".to_string(), "noexec".to_string(), "nodev".to_string()],
                propagation: MountPropagation::Private,
            },

            // /dev - device files
            Mount {
                source: "tmpfs".to_string(),
                destination: "/dev".to_string(),
                fs_type: "tmpfs".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "strictatime".to_string(),
                    "mode=755".to_string(),
                    "size=65536k".to_string(),
                ],
                propagation: MountPropagation::Private,
            },

            // /dev/pts - pseudo-terminals
            Mount {
                source: "devpts".to_string(),
                destination: "/dev/pts".to_string(),
                fs_type: "devpts".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "newinstance".to_string(),
                    "ptmxmode=0666".to_string(),
                    "mode=0620".to_string(),
                ],
                propagation: MountPropagation::Private,
            },

            // /dev/shm - shared memory
            Mount {
                source: "shm".to_string(),
                destination: "/dev/shm".to_string(),
                fs_type: "tmpfs".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "mode=1777".to_string(),
                    "size=65536k".to_string(),
                ],
                propagation: MountPropagation::Private,
            },

            // /sys - system information
            Mount {
                source: "sysfs".to_string(),
                destination: "/sys".to_string(),
                fs_type: "sysfs".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "ro".to_string(),
                ],
                propagation: MountPropagation::Private,
            },

            // /tmp - temporary files
            Mount {
                source: "tmpfs".to_string(),
                destination: "/tmp".to_string(),
                fs_type: "tmpfs".to_string(),
                options: vec![
                    "nosuid".to_string(),
                    "nodev".to_string(),
                    "mode=1777".to_string(),
                ],
                propagation: MountPropagation::Private,
            },
        ]
    }

    /// Add custom volume mount
    pub fn add_volume_mount(
        &self,
        source: &Path,
        destination: &Path,
        readonly: bool,
    ) -> Mount {
        let mut options = vec!["rbind".to_string()];
        if readonly {
            options.push("ro".to_string());
        }

        Mount {
            source: source.to_string_lossy().to_string(),
            destination: destination.to_string_lossy().to_string(),
            fs_type: "none".to_string(),
            options,
            propagation: MountPropagation::Slave, // One-way from host
        }
    }
}
```

#### 2.2.3 Tmpfs for Ephemeral Storage

```rust
// File: crates/clnrm-core/src/backend/gvisor/tmpfs.rs

pub struct TmpfsManager {
    max_size: u64, // bytes
}

impl TmpfsManager {
    /// Create tmpfs mount specification
    pub fn create_tmpfs(&self, destination: &str, size_mb: Option<u64>) -> Mount {
        let size = size_mb.unwrap_or(64); // Default 64MB

        Mount {
            source: "tmpfs".to_string(),
            destination: destination.to_string(),
            fs_type: "tmpfs".to_string(),
            options: vec![
                "nosuid".to_string(),
                "nodev".to_string(),
                format!("size={}m", size),
                "mode=1777".to_string(),
            ],
            propagation: MountPropagation::Private,
        }
    }

    /// Create tmpfs with custom options
    pub fn create_tmpfs_custom(
        &self,
        destination: &str,
        size_mb: u64,
        mode: &str,
        options: Vec<String>,
    ) -> Mount {
        let mut all_options = vec![
            "nosuid".to_string(),
            "nodev".to_string(),
            format!("size={}m", size_mb),
            format!("mode={}", mode),
        ];
        all_options.extend(options);

        Mount {
            source: "tmpfs".to_string(),
            destination: destination.to_string(),
            fs_type: "tmpfs".to_string(),
            options: all_options,
            propagation: MountPropagation::Private,
        }
    }
}
```

#### 2.2.4 File Permissions and Ownership

```rust
// File: crates/clnrm-core/src/backend/gvisor/permissions.rs

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct PermissionManager {
    rootfs_path: PathBuf,
}

impl PermissionManager {
    /// Set up proper permissions for rootfs
    pub fn setup_rootfs_permissions(&self) -> Result<()> {
        // Root directory
        self.set_perms("/", 0o755)?;

        // Standard directories
        self.set_perms("/bin", 0o755)?;
        self.set_perms("/sbin", 0o755)?;
        self.set_perms("/usr", 0o755)?;
        self.set_perms("/lib", 0o755)?;
        self.set_perms("/etc", 0o755)?;

        // Restricted directories
        self.set_perms("/root", 0o700)?;

        // World-writable with sticky bit
        self.set_perms("/tmp", 0o1777)?;
        self.set_perms("/var/tmp", 0o1777)?;

        // Device directories
        self.set_perms("/dev", 0o755)?;

        Ok(())
    }

    /// Set permissions for a path within rootfs
    fn set_perms(&self, path: &str, mode: u32) -> Result<()> {
        let full_path = self.rootfs_path.join(path.trim_start_matches('/'));

        if !full_path.exists() {
            return Ok(()); // Skip non-existent paths
        }

        let perms = fs::Permissions::from_mode(mode);
        fs::set_permissions(&full_path, perms)?;

        Ok(())
    }

    /// Setup ownership (requires root or user namespaces)
    pub fn setup_ownership(&self, uid: u32, gid: u32) -> Result<()> {
        use std::os::unix::fs::chown;

        // Change ownership of rootfs
        self.chown_recursive(&self.rootfs_path, uid, gid)?;

        Ok(())
    }

    fn chown_recursive(&self, path: &Path, uid: u32, gid: u32) -> Result<()> {
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            let path = entry.path();

            // Use libc for chown (Rust std doesn't expose this)
            unsafe {
                let path_cstr = std::ffi::CString::new(
                    path.to_str().unwrap()
                )?;
                libc::chown(path_cstr.as_ptr(), uid, gid);
            }
        }

        Ok(())
    }
}
```

---

## 3. Integration with gVisor

### 3.1 gVisor Backend Implementation

```rust
// File: crates/clnrm-core/src/backend/gvisor/mod.rs

use std::path::PathBuf;
use std::process::Command;

pub mod network;
pub mod filesystem;
pub mod oci;
pub mod mount;
pub mod dns;
pub mod permissions;

use crate::backend::{Backend, Cmd, RunResult};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct GvisorBackend {
    /// Image reference (e.g., "alpine:latest")
    image: String,

    /// Root directory for container state
    root_dir: PathBuf,

    /// Bundle directory for this container
    bundle_dir: PathBuf,

    /// Container ID
    container_id: String,

    /// Network configuration
    network_config: NetworkConfig,

    /// Platform (ptrace or kvm)
    platform: GvisorPlatform,

    /// Resource limits
    resource_limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub enum GvisorPlatform {
    Ptrace,  // User-space syscall interception
    Kvm,     // KVM-based virtualization
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub subnet: String,
    pub enable_network: bool,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub memory_mb: Option<u64>,
    pub cpu_shares: Option<u64>,
    pub pids_limit: Option<u64>,
}

impl GvisorBackend {
    pub fn new(image: &str) -> Result<Self> {
        let container_id = uuid::Uuid::new_v4().to_string();
        let root_dir = PathBuf::from("/var/lib/cleanroom/gvisor");
        let bundle_dir = root_dir.join("bundles").join(&container_id);

        Ok(Self {
            image: image.to_string(),
            root_dir,
            bundle_dir,
            container_id,
            network_config: NetworkConfig {
                subnet: "10.88.0.0/16".to_string(),
                enable_network: true,
                dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            },
            platform: GvisorPlatform::Ptrace,
            resource_limits: ResourceLimits {
                memory_mb: Some(512),
                cpu_shares: Some(1024),
                pids_limit: Some(100),
            },
        })
    }

    /// Setup container bundle
    fn setup_bundle(&self) -> Result<()> {
        // 1. Pull and unpack OCI image
        let oci_mgr = oci::OciImageManager::new(
            self.root_dir.join("cache")
        );
        let oci_dir = oci_mgr.pull_image(&self.image)?;
        oci_mgr.unpack_image(&oci_dir, &self.bundle_dir)?;

        // 2. Setup rootfs permissions
        let perm_mgr = permissions::PermissionManager {
            rootfs_path: self.bundle_dir.join("rootfs"),
        };
        perm_mgr.setup_rootfs_permissions()?;

        // 3. Configure DNS
        let dns = dns::DnsResolver {
            rootfs_path: self.bundle_dir.join("rootfs").to_string_lossy().to_string(),
        };
        dns.configure(&self.network_config.dns_servers.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;

        // 4. Generate OCI runtime config
        self.generate_oci_config()?;

        Ok(())
    }

    /// Generate OCI runtime configuration (config.json)
    fn generate_oci_config(&self) -> Result<()> {
        let config = OciRuntimeConfig {
            oci_version: "1.0.0".to_string(),
            root: OciRoot {
                path: "rootfs".to_string(),
                readonly: false,
            },
            mounts: self.generate_mounts(),
            process: OciProcess {
                terminal: false,
                user: OciUser { uid: 0, gid: 0 },
                args: vec!["sh".to_string()],
                env: vec![
                    "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
                ],
                cwd: "/".to_string(),
            },
            linux: OciLinux {
                namespaces: vec![
                    OciNamespace { ns_type: "pid".to_string() },
                    OciNamespace { ns_type: "network".to_string() },
                    OciNamespace { ns_type: "ipc".to_string() },
                    OciNamespace { ns_type: "uts".to_string() },
                    OciNamespace { ns_type: "mount".to_string() },
                ],
                resources: Some(OciResources {
                    memory: self.resource_limits.memory_mb.map(|mb| OciMemory {
                        limit: mb * 1024 * 1024,
                    }),
                    cpu: self.resource_limits.cpu_shares.map(|shares| OciCpu {
                        shares,
                    }),
                    pids: self.resource_limits.pids_limit.map(|limit| OciPids {
                        limit,
                    }),
                }),
            },
        };

        let config_json = serde_json::to_string_pretty(&config)?;
        std::fs::write(self.bundle_dir.join("config.json"), config_json)?;

        Ok(())
    }

    fn generate_mounts(&self) -> Vec<OciMount> {
        let mount_mgr = mount::MountManager {
            rootfs_path: self.bundle_dir.join("rootfs").to_string_lossy().to_string(),
        };

        mount_mgr.generate_mounts()
            .into_iter()
            .map(|m| OciMount {
                destination: m.destination,
                source: Some(m.source),
                options: Some(m.options),
                mount_type: Some(m.fs_type),
            })
            .collect()
    }

    /// Run container with runsc
    fn run_container(&self, cmd: &Cmd) -> Result<RunResult> {
        let start = std::time::Instant::now();

        // Build runsc command
        let mut runsc_cmd = Command::new("runsc");

        // Basic flags
        runsc_cmd
            .arg("--root").arg(&self.root_dir)
            .arg("--network").arg(if self.network_config.enable_network { "sandbox" } else { "none" })
            .arg("--platform").arg(match self.platform {
                GvisorPlatform::Ptrace => "ptrace",
                GvisorPlatform::Kvm => "kvm",
            });

        // Run command
        runsc_cmd
            .arg("run")
            .arg("--bundle").arg(&self.bundle_dir)
            .arg(&self.container_id);

        let output = runsc_cmd.output()?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(RunResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            steps: Vec::new(),
            redacted_env: Vec::new(),
            backend: "gvisor".to_string(),
            concurrent: false,
            step_order: Vec::new(),
        })
    }
}

impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        self.setup_bundle()?;
        self.run_container(&cmd)
    }

    fn name(&self) -> &str {
        "gvisor"
    }

    fn is_available(&self) -> bool {
        // Check if runsc is installed
        Command::new("runsc").arg("--version").output().is_ok()
    }

    fn supports_hermetic(&self) -> bool {
        true
    }

    fn supports_deterministic(&self) -> bool {
        true
    }
}

// OCI Runtime Spec structures
#[derive(serde::Serialize)]
struct OciRuntimeConfig {
    #[serde(rename = "ociVersion")]
    oci_version: String,
    root: OciRoot,
    mounts: Vec<OciMount>,
    process: OciProcess,
    linux: OciLinux,
}

#[derive(serde::Serialize)]
struct OciRoot {
    path: String,
    readonly: bool,
}

#[derive(serde::Serialize)]
struct OciMount {
    destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<String>>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    mount_type: Option<String>,
}

#[derive(serde::Serialize)]
struct OciProcess {
    terminal: bool,
    user: OciUser,
    args: Vec<String>,
    env: Vec<String>,
    cwd: String,
}

#[derive(serde::Serialize)]
struct OciUser {
    uid: u32,
    gid: u32,
}

#[derive(serde::Serialize)]
struct OciLinux {
    namespaces: Vec<OciNamespace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<OciResources>,
}

#[derive(serde::Serialize)]
struct OciNamespace {
    #[serde(rename = "type")]
    ns_type: String,
}

#[derive(serde::Serialize)]
struct OciResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<OciMemory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<OciCpu>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pids: Option<OciPids>,
}

#[derive(serde::Serialize)]
struct OciMemory {
    limit: u64,
}

#[derive(serde::Serialize)]
struct OciCpu {
    shares: u64,
}

#[derive(serde::Serialize)]
struct OciPids {
    limit: u64,
}
```

### 3.2 gVisor CLI Flags Reference

```bash
# Essential runsc flags for cleanroom

# Core execution flags
runsc \
  --root /var/lib/cleanroom/gvisor \      # State directory
  --platform ptrace \                      # Platform (ptrace or kvm)
  --network sandbox \                      # Network mode
  --file-access exclusive \                # File access mode
  run --bundle /path/to/bundle container-id

# Network isolation flags
--network=none                # No network
--network=sandbox             # Sandboxed network with netstack
--network=host                # Host network (not recommended)

# Filesystem flags
--file-access=exclusive       # Exclusive gofer access (secure)
--file-access=shared          # Shared gofer access (faster)
--overlay                     # Enable overlay filesystem

# Security flags
--platform=ptrace             # Ptrace-based syscall interception
--platform=kvm                # KVM-based virtualization (requires /dev/kvm)

# Resource limits (via OCI config.json)
{
  "linux": {
    "resources": {
      "memory": { "limit": 536870912 },    # 512MB
      "cpu": { "shares": 1024 },           # CPU shares
      "pids": { "limit": 100 }             # Max processes
    }
  }
}

# Debug flags
--debug                       # Enable debug logging
--debug-log=/tmp/runsc.log    # Debug log file
--strace                      # Enable syscall tracing

# Performance flags
--num-network-channels=1      # Network goroutines
--watchdog-action=panic       # Watchdog behavior
```

### 3.3 Seccomp/AppArmor Profiles

```rust
// File: crates/clnrm-core/src/backend/gvisor/seccomp.rs

pub struct SeccompProfile;

impl SeccompProfile {
    /// Generate seccomp profile for OCI config
    pub fn generate() -> OciSeccomp {
        OciSeccomp {
            default_action: "SCMP_ACT_ERRNO".to_string(),
            architectures: vec!["SCMP_ARCH_X86_64".to_string()],
            syscalls: vec![
                // Allow basic syscalls
                OciSyscall {
                    names: vec!["read", "write", "close", "stat", "fstat", "lstat"].iter().map(|s| s.to_string()).collect(),
                    action: "SCMP_ACT_ALLOW".to_string(),
                },
                // Allow process management
                OciSyscall {
                    names: vec!["fork", "vfork", "clone", "execve", "exit", "wait4"].iter().map(|s| s.to_string()).collect(),
                    action: "SCMP_ACT_ALLOW".to_string(),
                },
                // Allow memory management
                OciSyscall {
                    names: vec!["mmap", "munmap", "mprotect", "brk"].iter().map(|s| s.to_string()).collect(),
                    action: "SCMP_ACT_ALLOW".to_string(),
                },
                // Block dangerous syscalls
                OciSyscall {
                    names: vec!["ptrace", "reboot", "kexec_load", "init_module", "delete_module"].iter().map(|s| s.to_string()).collect(),
                    action: "SCMP_ACT_ERRNO".to_string(),
                },
            ],
        }
    }
}

#[derive(serde::Serialize)]
pub struct OciSeccomp {
    #[serde(rename = "defaultAction")]
    default_action: String,
    architectures: Vec<String>,
    syscalls: Vec<OciSyscall>,
}

#[derive(serde::Serialize)]
pub struct OciSyscall {
    names: Vec<String>,
    action: String,
}
```

---

## 4. Cleanup and Resource Management

### 4.1 Cleanup Manager

```rust
// File: crates/clnrm-core/src/backend/gvisor/cleanup.rs

use std::fs;
use std::path::PathBuf;

pub struct CleanupManager {
    root_dir: PathBuf,
}

impl CleanupManager {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Cleanup container on stop
    pub fn cleanup_container(&self, container_id: &str) -> Result<()> {
        // 1. Stop runsc container
        self.stop_runsc_container(container_id)?;

        // 2. Cleanup network namespace
        self.cleanup_network_namespace(container_id)?;

        // 3. Cleanup bundle directory
        self.cleanup_bundle(container_id)?;

        // 4. Cleanup iptables rules
        self.cleanup_iptables_rules(container_id)?;

        // 5. Release IP address
        self.release_ip_address(container_id)?;

        Ok(())
    }

    fn stop_runsc_container(&self, container_id: &str) -> Result<()> {
        let output = std::process::Command::new("runsc")
            .arg("--root").arg(&self.root_dir)
            .arg("delete")
            .arg("--force")
            .arg(container_id)
            .output()?;

        if !output.status.success() {
            // Log warning but don't fail - container may already be stopped
            eprintln!("Warning: Failed to stop container {}: {}",
                container_id,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn cleanup_network_namespace(&self, container_id: &str) -> Result<()> {
        let netns_mgr = super::netns::NetnsManager::new();

        if netns_mgr.exists(container_id) {
            let _ = std::process::Command::new("ip")
                .args(&["netns", "delete", container_id])
                .output();
        }

        Ok(())
    }

    fn cleanup_bundle(&self, container_id: &str) -> Result<()> {
        let bundle_dir = self.root_dir.join("bundles").join(container_id);

        if bundle_dir.exists() {
            fs::remove_dir_all(bundle_dir)?;
        }

        Ok(())
    }

    fn cleanup_iptables_rules(&self, container_id: &str) -> Result<()> {
        // Remove all iptables rules for this container
        // This is a simplified version - in practice, track rules per container
        let _ = std::process::Command::new("iptables")
            .args(&["-t", "nat", "-F"])
            .output();

        Ok(())
    }

    fn release_ip_address(&self, container_id: &str) -> Result<()> {
        // Release IP from allocator
        // Implementation depends on IP allocator design
        Ok(())
    }

    /// Cleanup orphaned namespaces
    pub fn cleanup_orphaned_namespaces(&self) -> Result<usize> {
        let netns_mgr = super::netns::NetnsManager::new();

        // List active containers
        let active = self.list_active_containers()?;

        // Cleanup orphaned namespaces
        netns_mgr.cleanup_orphaned(&active.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    fn list_active_containers(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new("runsc")
            .arg("--root").arg(&self.root_dir)
            .arg("list")
            .arg("--format=json")
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let containers: Vec<RunscContainer> = serde_json::from_slice(&output.stdout)?;
        Ok(containers.into_iter().map(|c| c.id).collect())
    }

    /// Cleanup temporary files
    pub fn cleanup_temp_files(&self) -> Result<usize> {
        let mut cleaned = 0;

        let temp_dirs = vec![
            self.root_dir.join("tmp"),
            self.root_dir.join("cache/tmp"),
        ];

        for dir in temp_dirs {
            if dir.exists() {
                cleaned += self.cleanup_old_files(&dir, 86400)?; // 24 hours
            }
        }

        Ok(cleaned)
    }

    fn cleanup_old_files(&self, dir: &PathBuf, max_age_seconds: u64) -> Result<usize> {
        let mut cleaned = 0;
        let now = std::time::SystemTime::now();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age.as_secs() > max_age_seconds {
                        if metadata.is_dir() {
                            fs::remove_dir_all(entry.path())?;
                        } else {
                            fs::remove_file(entry.path())?;
                        }
                        cleaned += 1;
                    }
                }
            }
        }

        Ok(cleaned)
    }

    /// Enforce resource limits
    pub fn enforce_resource_limits(&self) -> Result<()> {
        // Check disk usage
        let disk_usage = self.calculate_disk_usage()?;
        let max_disk_usage = 10 * 1024 * 1024 * 1024; // 10GB

        if disk_usage > max_disk_usage {
            // Cleanup old caches
            self.cleanup_image_cache()?;
        }

        Ok(())
    }

    fn calculate_disk_usage(&self) -> Result<u64> {
        let output = std::process::Command::new("du")
            .args(&["-sb", self.root_dir.to_str().unwrap()])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let size_str = stdout.split_whitespace().next().unwrap_or("0");

        Ok(size_str.parse().unwrap_or(0))
    }

    fn cleanup_image_cache(&self) -> Result<()> {
        let cache_dir = self.root_dir.join("cache/oci");

        if cache_dir.exists() {
            // Keep only recent images (last 7 days)
            self.cleanup_old_files(&cache_dir, 7 * 86400)?;
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct RunscContainer {
    id: String,
}
```

### 4.2 Resource Limits Enforcement

```rust
// File: crates/clnrm-core/src/backend/gvisor/resources.rs

pub struct ResourceLimitsEnforcer {
    max_containers: usize,
    max_memory_total: u64,
    max_cpu_total: f64,
}

impl ResourceLimitsEnforcer {
    /// Check if resource limits allow new container
    pub fn can_create_container(&self, requested: &ResourceLimits) -> Result<bool> {
        let current = self.get_current_usage()?;

        // Check container count
        if current.container_count >= self.max_containers {
            return Ok(false);
        }

        // Check memory
        if let Some(requested_mem) = requested.memory_mb {
            if current.memory_mb + requested_mem > self.max_memory_total {
                return Ok(false);
            }
        }

        // Check CPU
        if let Some(requested_cpu) = requested.cpu_shares {
            let requested_cpu_fraction = requested_cpu as f64 / 1024.0;
            if current.cpu_fraction + requested_cpu_fraction > self.max_cpu_total {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_current_usage(&self) -> Result<CurrentUsage> {
        // Query runsc for current resource usage
        // This is a simplified version
        Ok(CurrentUsage {
            container_count: 0,
            memory_mb: 0,
            cpu_fraction: 0.0,
        })
    }
}

struct CurrentUsage {
    container_count: usize,
    memory_mb: u64,
    cpu_fraction: f64,
}
```

---

## 5. Error Scenarios and Recovery

### 5.1 Error Handling

```rust
// File: crates/clnrm-core/src/backend/gvisor/errors.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GvisorError {
    #[error("runsc not found: {0}")]
    RunscNotFound(String),

    #[error("Failed to create network namespace: {0}")]
    NetworkNamespaceError(String),

    #[error("Failed to setup network: {0}")]
    NetworkSetupError(String),

    #[error("OCI image pull failed: {0}")]
    ImagePullError(String),

    #[error("OCI image unpack failed: {0}")]
    ImageUnpackError(String),

    #[error("Failed to generate OCI config: {0}")]
    OciConfigError(String),

    #[error("Container start failed: {0}")]
    ContainerStartError(String),

    #[error("Container execution failed: {0}")]
    ContainerExecError(String),

    #[error("Cleanup failed: {0}")]
    CleanupError(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitError(String),

    #[error("IP allocation failed: {0}")]
    IpAllocationError(String),
}

/// Recovery strategies for common errors
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Recover from network namespace error
    pub fn recover_network_namespace(container_id: &str) -> Result<()> {
        // 1. Delete existing namespace
        let _ = std::process::Command::new("ip")
            .args(&["netns", "delete", container_id])
            .output();

        // 2. Cleanup veth interfaces
        let _ = std::process::Command::new("ip")
            .args(&["link", "delete", &format!("veth-{}", container_id)])
            .output();

        Ok(())
    }

    /// Recover from container start failure
    pub fn recover_container_start(container_id: &str, root_dir: &Path) -> Result<()> {
        // 1. Force delete container
        let _ = std::process::Command::new("runsc")
            .arg("--root").arg(root_dir)
            .arg("delete")
            .arg("--force")
            .arg(container_id)
            .output();

        // 2. Cleanup bundle
        let bundle_dir = root_dir.join("bundles").join(container_id);
        if bundle_dir.exists() {
            fs::remove_dir_all(bundle_dir)?;
        }

        Ok(())
    }

    /// Recover from IP allocation failure
    pub fn recover_ip_allocation() -> Result<()> {
        // Reset IP allocator state
        // Implementation depends on allocator design
        Ok(())
    }
}
```

### 5.2 Error Scenarios Table

| Error Scenario | Detection | Recovery Strategy | Prevention |
|---------------|-----------|-------------------|------------|
| runsc not installed | Pre-flight check | Exit with clear error message | Document dependency |
| Network namespace exists | namespace creation fails | Delete existing namespace | Generate unique IDs |
| IP exhaustion | Allocation fails | Cleanup orphaned IPs | Larger subnet |
| OCI image pull timeout | skopeo timeout | Retry with backoff | Cache images |
| Insufficient disk space | df check | Cleanup old images | Monitor disk usage |
| Memory limit exceeded | cgroup limit | Reject container creation | Pre-check limits |
| Orphaned namespaces | Periodic cleanup | Force delete | Track active containers |
| Bundle corruption | config.json parse error | Delete and recreate | Atomic writes |
| iptables rule conflict | Rule add fails | Cleanup existing rules | Unique rule chains |

---

## 6. Performance Considerations

### 6.1 Performance Optimization Strategies

```
┌──────────────────────────────────────────────────────────────────┐
│                   Performance Optimizations                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  1. Image Caching                                                │
│     ├─ Local OCI cache (skopeo)                                 │
│     ├─ Layered filesystem (overlay)                             │
│     └─ Shared base layers                                       │
│                                                                   │
│  2. Network Performance                                          │
│     ├─ Pre-allocate IP addresses                                │
│     ├─ Reuse network namespaces                                 │
│     └─ Batch iptables operations                                │
│                                                                   │
│  3. Filesystem Performance                                       │
│     ├─ tmpfs for /tmp (in-memory)                               │
│     ├─ Overlay filesystem (shared base)                         │
│     └─ Lazy mount (mount on demand)                             │
│                                                                   │
│  4. Container Pooling                                            │
│     ├─ Pre-warmed containers                                    │
│     ├─ Reuse bundles for same image                            │
│     └─ Parallel container creation                              │
│                                                                   │
│  5. Resource Management                                          │
│     ├─ CPU pinning (numactl)                                    │
│     ├─ Memory limits (cgroup v2)                                │
│     └─ I/O scheduling (ionice)                                  │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### 6.2 Benchmark Targets

| Operation | Target | Docker Baseline | gVisor Target | Improvement |
|-----------|--------|-----------------|---------------|-------------|
| Container start (cold) | <2s | 2-3s | 1.5-2s | 25% faster |
| Container start (warm) | <500ms | 500-1000ms | 200-500ms | 50% faster |
| Network setup | <100ms | 50-100ms | 50-100ms | Similar |
| Filesystem mount | <50ms | 20-50ms | 20-50ms | Similar |
| Cleanup | <500ms | 300-500ms | 200-300ms | 40% faster |
| Image pull (cached) | <100ms | 100-200ms | 50-100ms | 50% faster |
| Total overhead | <2.5s | 3-4s | 2-2.5s | 37% faster |

### 6.3 Performance Monitoring

```rust
// File: crates/clnrm-core/src/backend/gvisor/metrics.rs

use std::time::Instant;

pub struct GvisorMetrics {
    container_starts: prometheus::Histogram,
    network_setup_duration: prometheus::Histogram,
    filesystem_setup_duration: prometheus::Histogram,
    cleanup_duration: prometheus::Histogram,
}

impl GvisorMetrics {
    pub fn record_container_start(&self, duration: std::time::Duration) {
        self.container_starts.observe(duration.as_secs_f64());
    }

    pub fn record_network_setup(&self, duration: std::time::Duration) {
        self.network_setup_duration.observe(duration.as_secs_f64());
    }

    pub fn record_filesystem_setup(&self, duration: std::time::Duration) {
        self.filesystem_setup_duration.observe(duration.as_secs_f64());
    }

    pub fn record_cleanup(&self, duration: std::time::Duration) {
        self.cleanup_duration.observe(duration.as_secs_f64());
    }
}
```

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Implement basic GvisorBackend structure
- [ ] OCI image pull and unpack (skopeo + umoci)
- [ ] Basic rootfs setup
- [ ] Simple OCI config.json generation
- [ ] runsc integration (basic execution)

### Phase 2: Network Isolation (Week 2)
- [ ] Network namespace management
- [ ] IP allocator
- [ ] veth pair setup
- [ ] DNS configuration
- [ ] Port mapping (iptables)

### Phase 3: Filesystem Isolation (Week 3)
- [ ] Mount manager
- [ ] Tmpfs configuration
- [ ] Volume mounting
- [ ] Permission management
- [ ] Overlay filesystem support

### Phase 4: Resource Management (Week 4)
- [ ] Cleanup manager
- [ ] Resource limits enforcement
- [ ] Orphaned namespace cleanup
- [ ] Error recovery strategies
- [ ] Performance monitoring

### Phase 5: Testing & Validation (Week 5)
- [ ] Unit tests for all components
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Error scenario tests
- [ ] Documentation

---

## 8. Dependencies

### Required System Tools
```bash
# Essential
runsc              # gVisor runtime
ip                 # Network configuration
iptables          # Port mapping
skopeo            # OCI image operations
umoci             # OCI image unpacking

# Optional (for manual operations)
runc              # Alternative runtime
crun              # Lightweight alternative

# Performance tools
numactl           # CPU pinning
ionice            # I/O scheduling
```

### Rust Dependencies
```toml
[dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
thiserror = "1.0"

# Filesystem
walkdir = "2.3"
sha2 = "0.10"

# Async (for future async support)
tokio = { version = "1.0", features = ["full"] }

# Monitoring
prometheus = "0.13"
tracing = "0.1"
```

---

## 9. Security Considerations

### Attack Surface Reduction
- ✅ No Docker daemon (eliminates daemon attack surface)
- ✅ gVisor sandbox (syscall interception)
- ✅ Network isolation (separate namespaces)
- ✅ Filesystem isolation (private mounts)
- ✅ Resource limits (prevent DoS)
- ✅ Seccomp profiles (syscall filtering)

### Security Checklist
- [ ] Verify runsc integrity (checksum/signature)
- [ ] Use ptrace platform for maximum isolation
- [ ] Enable seccomp filtering
- [ ] Restrict network access (minimal allowed ports)
- [ ] Use read-only rootfs where possible
- [ ] Apply least privilege (non-root user)
- [ ] Monitor resource usage
- [ ] Regular security audits

---

## 10. Migration Path

### From Testcontainers to gVisor

```rust
// Before (testcontainers)
let backend = TestcontainerBackend::new("alpine:latest")?;
let result = backend.run_cmd(cmd)?;

// After (gvisor)
let backend = GvisorBackend::new("alpine:latest")?;
let result = backend.run_cmd(cmd)?;
```

### Compatibility Layer
```rust
// File: crates/clnrm-core/src/backend/compat.rs

/// Compatibility wrapper for smooth migration
pub enum UnifiedBackend {
    Testcontainers(TestcontainerBackend),
    Gvisor(GvisorBackend),
}

impl Backend for UnifiedBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        match self {
            Self::Testcontainers(b) => b.run_cmd(cmd),
            Self::Gvisor(b) => b.run_cmd(cmd),
        }
    }

    // ... other trait methods
}
```

---

## Conclusion

This design provides a complete Docker-free isolation strategy using gVisor directly. Key benefits:

1. **Complete Docker Elimination**: No daemon, no Docker CLI dependency
2. **Enhanced Security**: gVisor sandbox + namespace isolation
3. **Better Performance**: Reduced overhead, optimized caching
4. **Production Ready**: Comprehensive error handling and recovery
5. **Maintainable**: Clean architecture, well-documented

**Next Steps**:
1. Review this design with team
2. Set up development environment (install runsc, skopeo, umoci)
3. Start Phase 1 implementation
4. Create integration tests
5. Performance benchmarking

**Questions/Concerns**:
- Root privileges required for network setup (consider rootless alternatives)
- Platform support (Linux-only initially)
- Learning curve for operations team
- Migration timeline for existing tests
