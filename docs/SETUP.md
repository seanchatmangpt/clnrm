# gVisor Setup Guide

Complete installation and configuration guide for clnrm with gVisor backend.

**Target Audience**: System administrators, DevOps engineers, developers
**Time Required**: 15-30 minutes
**Prerequisites**: Linux (x86_64), kernel 4.14+, sudo access

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation Steps](#installation-steps)
3. [Configuration](#configuration)
4. [Verification](#verification)
5. [Troubleshooting](#troubleshooting)
6. [Uninstallation](#uninstallation)

---

## System Requirements

### Linux Kernel

gVisor requires Linux kernel 4.14 or later. Check your version:

```bash
uname -r
# Expected output: 4.14.0 or higher
```

If you have an older kernel, update it:
```bash
sudo apt-get update
sudo apt-get install -y linux-image-generic
sudo reboot
```

### CPU Architecture

Supported architectures:
- x86_64 (primary support)
- ARM64 (experimental support)

Check your architecture:
```bash
uname -m
# Expected: x86_64 or aarch64
```

### Disk Space

Minimum requirements:
- 2 GB for gVisor installation
- 5 GB for OCI image cache (scalable)
- 1 GB for temporary container bundles

### Network

- Internet access for downloading gVisor and OCI images
- No Docker daemon or other container runtimes required

---

## Installation Steps

### Step 1: Install gVisor Runtime (runsc)

**Ubuntu/Debian (Recommended)**

Add gVisor repository:
```bash
curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
```

Install runsc:
```bash
sudo apt-get update
sudo apt-get install -y runsc
```

Verify installation:
```bash
runsc --version
# Expected output: release-0.X.X
```

**Alternative: Manual Installation**

If you prefer manual installation:
```bash
# Download runsc
wget https://storage.googleapis.com/gvisor/releases/release/latest/x86_64/runsc

# Make executable
chmod +x runsc

# Install to /usr/local/bin
sudo mv runsc /usr/local/bin/

# Verify
runsc --version
```

### Step 2: Install OCI Image Tools

Install skopeo for pulling OCI images:

```bash
sudo apt-get install -y skopeo
```

Verify installation:
```bash
skopeo --version
# Expected output: skopeo version X.X.X
```

### Step 3: Install Rust (if not already installed)

clnrm requires Rust 1.70 or later:

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source Rust environment
source $HOME/.cargo/env

# Verify Rust installation
rustc --version
# Expected: rustc 1.70.0 or later

cargo --version
# Expected: cargo 1.70.0 or later
```

### Step 4: Install clnrm

Install from crates.io:
```bash
cargo install clnrm
```

Or build from source:
```bash
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm
cargo install --path crates/clnrm-cli
```

Verify installation:
```bash
clnrm --version
# Expected: clnrm X.X.X
```

### Step 5: Create Cache Directory

Create directory for OCI image cache:

```bash
# Create cache directory
sudo mkdir -p /var/cache/clnrm

# Set permissions (replace 'username' with your username)
sudo chown username:username /var/cache/clnrm
sudo chmod 755 /var/cache/clnrm

# Verify
ls -ld /var/cache/clnrm
# Expected: drwxr-xr-x ... /var/cache/clnrm
```

---

## Configuration

### Create Configuration File

Create `.clnrm.toml` in your project root:

```toml
# clnrm v2.0.0 Configuration File
# Place this file in your project root directory

[backend]
# Backend type: gvisor (required)
type = "gvisor"

# gVisor backend configuration
[backend.gvisor]
# Directory for caching OCI images (optional, default: ~/.cache/clnrm)
cache_dir = "/var/cache/clnrm"

# Container startup timeout in seconds (optional, default: 30)
startup_timeout = 30

# Command execution timeout in seconds (optional, default: 300)
execution_timeout = 300

# Enable debug logging (optional, default: false)
debug = false

# gVisor platform: "systrap" (default) or "kvm"
# - systrap: Full emulation of all syscalls (slower, more compatible)
# - kvm: Virtual machine-based isolation (faster, requires /dev/kvm)
platform = "systrap"

# Network mode: "sandbox" (default, isolated) or "host" (host network)
# - sandbox: Network namespace isolation (more secure)
# - host: Use host network (less isolated, useful for debugging)
network_mode = "sandbox"

# Filesystem mode: "shared" (default) or "exclusive"
# - shared: Container and host share filesystem (faster)
# - exclusive: Filesystem isolation (more secure)
file_access = "shared"

# Resource limits (all optional)
[backend.gvisor.limits]
# Memory limit in MB (optional, default: unlimited)
# Example: 512 MB per container
memory_mb = 512

# CPU limit in number of CPUs (optional, default: unlimited)
# Example: 2 CPUs per container
cpus = 2.0

# Disk I/O limit in MB/s (optional, default: unlimited)
disk_io_mbps = 100

# OCI Registry configuration (optional)
[backend.gvisor.registry]
# Default registry for images without registry prefix
default = "docker.io"

# Registry authentication (optional, for private registries)
# [backend.gvisor.registry.auth]
# "docker.io" = { username = "user", password_env = "DOCKER_PASSWORD" }
# "ghcr.io" = { token_env = "GITHUB_TOKEN" }
# "registry.example.com" = { ca_cert = "/etc/ssl/certs/ca.crt" }

# Predefined services (optional)
# Use these if you need SurrealDB, OpenTelemetry Collector, etc.
# [[backend.gvisor.services]]
# name = "surrealdb"
# image = "surrealdb/surrealdb:latest"
# ports = [8000]
# env = { SURREAL_USER = "root", SURREAL_PASS = "root" }
```

### Environment Variables

Override configuration using environment variables:

```bash
# Backend type (default: gvisor)
export CLNRM_BACKEND=gvisor

# Image cache directory (overrides config file)
export CLNRM_CACHE_DIR=/var/cache/clnrm

# Debug mode (default: false)
export CLNRM_DEBUG=true

# Startup timeout in seconds (default: 30)
export CLNRM_STARTUP_TIMEOUT=60

# Execution timeout in seconds (default: 300)
export CLNRM_EXECUTION_TIMEOUT=600

# Resource limits
export CLNRM_MEMORY_LIMIT_MB=1024
export CLNRM_CPU_LIMIT=4.0

# Platform: systrap or kvm
export CLNRM_PLATFORM=systrap

# Network mode: sandbox or host
export CLNRM_NETWORK_MODE=sandbox

# File access: shared or exclusive
export CLNRM_FILE_ACCESS=shared

# Registry credentials (for private registries)
export DOCKER_PASSWORD=<password>
export GITHUB_TOKEN=<token>
```

### Configuration Precedence

Configuration is applied in this order (later overrides earlier):

1. Default values in clnrm
2. `.clnrm.toml` configuration file
3. Environment variables (`CLNRM_*`)
4. Programmatic configuration (in code)
5. Runtime overrides (command-line arguments)

---

## Verification

### 1. Check gVisor Installation

```bash
# Verify runsc is installed
which runsc
# Expected: /usr/bin/runsc or /usr/local/bin/runsc

# Check version
runsc --version
# Expected: release-0.X.X

# Check root access (required for gVisor)
sudo runsc --version
# Expected: release-0.X.X (no errors)
```

### 2. Verify OCI Tools

```bash
# Check skopeo
skopeo --version
# Expected: skopeo version X.X.X
```

### 3. Verify clnrm Installation

```bash
# Check clnrm
clnrm --version
# Expected: clnrm X.X.X

# Show help
clnrm --help
```

### 4. Test gVisor Runtime

Run a simple test to verify everything works:

```bash
# Create a simple test file
cat > test.rs <<'EOF'
#[test]
fn test_gvisor_hello() {
    let output = std::process::Command::new("echo")
        .arg("Hello from gVisor!")
        .output()
        .expect("Failed to execute echo");

    assert!(output.status.success());
}
EOF

# Run the test with gVisor
CLNRM_BACKEND=gvisor cargo test --lib test_gvisor_hello -- --nocapture
```

### 5. Verify Configuration File

```bash
# Validate your .clnrm.toml
clnrm config validate

# Show resolved configuration
clnrm config show

# Test with specific file
clnrm config validate --config ./my-custom.toml
```

---

## Troubleshooting

### Issue: "runsc: command not found"

**Symptoms**: Installing gVisor completed, but `runsc` is not in PATH

**Solutions**:
```bash
# Check if runsc is installed
which runsc

# If not found, try:
sudo apt-get install -y runsc

# Or manually download and install
wget https://storage.googleapis.com/gvisor/releases/release/latest/x86_64/runsc
chmod +x runsc
sudo mv runsc /usr/local/bin/
sudo chmod +x /usr/local/bin/runsc

# Verify
runsc --version
```

### Issue: "Permission denied" when running tests

**Symptoms**: Tests fail with permission errors, even with `sudo`

**Solutions**:
```bash
# gVisor requires certain capabilities
# Run with sudo:
sudo cargo test --all

# Or configure user namespaces:
# This is complex - see gVisor docs for rootless setup

# For development, use sudo in your test command:
sudo -E cargo test
```

### Issue: "Kernel too old" error

**Symptoms**: gVisor complains about kernel version

**Solutions**:
```bash
# Check your kernel version
uname -r

# Update to a newer kernel
sudo apt-get update
sudo apt-get install -y linux-image-generic
sudo reboot

# After reboot, verify new kernel
uname -r  # Should be 4.14 or higher
```

### Issue: "Image pull timeout"

**Symptoms**: Tests timeout when pulling OCI images

**Solutions**:
```bash
# Increase timeout in config
[backend.gvisor]
startup_timeout = 60

# Or via environment variable
export CLNRM_STARTUP_TIMEOUT=60

# Pre-pull images before running tests
skopeo copy docker://alpine:latest oci:/var/cache/clnrm/alpine:latest
skopeo copy docker://python:3.11-slim oci:/var/cache/clnrm/python:3.11-slim

# Check network connectivity
ping docker.io

# If behind proxy, configure it:
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=https://proxy.example.com:8080
```

### Issue: "Not enough disk space"

**Symptoms**: Tests fail with "No space left on device"

**Solutions**:
```bash
# Check disk usage
df -h /var/cache/clnrm

# Clear image cache
rm -rf /var/cache/clnrm/*

# Or configure alternative cache directory
export CLNRM_CACHE_DIR=/mnt/large-disk/clnrm
mkdir -p $CLNRM_CACHE_DIR

# Check what's using space
du -sh /var/cache/clnrm/*
```

### Issue: "Too many open files"

**Symptoms**: Tests fail with "Too many open files" error

**Solutions**:
```bash
# Increase file descriptor limit
ulimit -n 4096

# For permanent increase, edit /etc/security/limits.conf:
echo "* soft nofile 4096" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 8192" | sudo tee -a /etc/security/limits.conf

# Logout and login for changes to take effect

# Verify
ulimit -n  # Should show 4096 or higher
```

---

## Uninstallation

### Remove gVisor

```bash
# Remove runsc package
sudo apt-get remove -y runsc

# Remove gVisor repository
sudo rm /etc/apt/sources.list.d/gvisor.list

# If manually installed
sudo rm /usr/local/bin/runsc
```

### Remove clnrm

```bash
# Remove clnrm installation
cargo uninstall clnrm

# Remove cache directory (if desired)
sudo rm -rf /var/cache/clnrm
```

### Clean Up Configuration

```bash
# Remove .clnrm.toml from projects
rm .clnrm.toml

# Remove cache directory from home
rm -rf ~/.cache/clnrm
```

---

## Advanced Configuration

### Using KVM Platform (Faster)

If your system has `/dev/kvm` available:

```toml
[backend.gvisor]
platform = "kvm"  # Enable KVM acceleration
```

Check if KVM is available:
```bash
ls -la /dev/kvm
# Should show: crw-rw----- ... /dev/kvm
```

### Configuring Private Registry Access

For images from private registries:

1. Create registry credentials file:
```bash
mkdir -p ~/.docker
cat > ~/.docker/config.json <<'EOF'
{
  "auths": {
    "registry.example.com": {
      "auth": "base64-encoded-username:password"
    }
  }
}
EOF
```

2. Update `.clnrm.toml`:
```toml
[backend.gvisor.registry.auth]
"registry.example.com" = {
  ca_cert = "/etc/ssl/certs/ca-cert.crt",
  username_env = "REGISTRY_USER",
  password_env = "REGISTRY_PASS"
}
```

3. Set environment variables:
```bash
export REGISTRY_USER=myuser
export REGISTRY_PASS=mypass
```

### Setting Up Image Pre-cache

Pre-pull images for faster test execution:

```bash
#!/bin/bash
# pre-cache-images.sh

IMAGES=(
  "alpine:latest"
  "python:3.11-slim"
  "rust:latest"
  "ubuntu:22.04"
  "surrealdb/surrealdb:latest"
  "otel/opentelemetry-collector:latest"
)

for image in "${IMAGES[@]}"; do
  echo "Pulling $image..."
  skopeo copy "docker://$image" "oci:///var/cache/clnrm/${image// /}"
done

echo "Image cache populated!"
```

Run it:
```bash
chmod +x pre-cache-images.sh
./pre-cache-images.sh
```

---

## Security Considerations

### File Permissions

```bash
# Secure cache directory
sudo chmod 700 /var/cache/clnrm
sudo chown username:username /var/cache/clnrm

# Secure configuration file
chmod 600 .clnrm.toml

# Protect credentials
chmod 600 ~/.docker/config.json
```

### Running Tests Safely

```bash
# Use separate user for CI/CD
sudo useradd -m -s /bin/bash clnrm-runner
sudo usermod -aG sudo clnrm-runner

# Configure sudoers for gVisor without password (optional)
echo "clnrm-runner ALL=(ALL) NOPASSWD: /usr/bin/runsc" | sudo tee /etc/sudoers.d/clnrm-runner
```

### Network Security

```toml
# Use sandbox network mode (default, recommended)
[backend.gvisor]
network_mode = "sandbox"  # Isolated network

# Host network only when necessary for debugging
network_mode = "host"  # Full host network access (less secure)
```

---

## Next Steps

After completing setup:

1. **Read**: [DEVELOPMENT.md](DEVELOPMENT.md) to set up your development environment
2. **Learn**: [TESTING.md](TESTING.md) for running tests with gVisor
3. **Migrate**: [MIGRATION_FROM_DOCKER.md](MIGRATION_FROM_DOCKER.md) if transitioning from Docker
4. **Reference**: [GVISOR_QUICK_START.md](GVISOR_QUICK_START.md) for quick commands

---

## Support and Feedback

- **Issues**: Report problems at https://github.com/seanchatmangpt/clnrm/issues
- **Documentation**: Full docs at https://github.com/seanchatmangpt/clnrm/tree/main/docs
- **gVisor Help**: https://gvisor.dev/docs

---

**Installation Complete!** You're ready to use clnrm with gVisor. See [DEVELOPMENT.md](DEVELOPMENT.md) to start developing.
