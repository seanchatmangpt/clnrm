# Upgrade Guide: v0.3.x to v0.4.0

Complete step-by-step guide for upgrading from Cleanroom v0.3.x to v0.4.0.

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Breaking Changes Summary](#breaking-changes-summary)
4. [Step-by-Step Upgrade](#step-by-step-upgrade)
5. [Configuration Changes](#configuration-changes)
6. [Code Migration](#code-migration)
7. [Testing Recommendations](#testing-recommendations)
8. [Troubleshooting](#troubleshooting)
9. [Rollback Procedure](#rollback-procedure)

---

## Overview

Cleanroom v0.4.0 introduces major new features including real AI integration, plugin marketplace, and autonomous service management. This guide will help you upgrade safely with minimal disruption.

**Upgrade Time Estimate**: 30-60 minutes
**Difficulty Level**: Medium
**Recommended Approach**: Blue-Green deployment or staged rollout

---

## Prerequisites

### System Requirements

**Required**:
- Rust 1.70 or later
- Docker or Podman (latest stable)
- 4GB+ RAM available
- 10GB+ disk space

**Verify Rust version**:
```bash
rustc --version  # Should be 1.70+
```

**Optional (for AI features)**:
- Ollama for real AI integration
- 2GB+ disk space for AI models

### Backup Current Installation

**Backup your configuration**:
```bash
# Backup TOML files
cp -r tests/ tests.backup/
cp cleanroom.toml cleanroom.toml.backup 2>/dev/null || true

# Backup cargo files
cp Cargo.toml Cargo.toml.backup
cp Cargo.lock Cargo.lock.backup
```

**Document current versions**:
```bash
clnrm --version > version_before_upgrade.txt
cargo tree | grep clnrm >> version_before_upgrade.txt
```

---

## Breaking Changes Summary

### 1. Workspace Version Update

**Change**: Version bumped from `0.3.2` to `0.4.0` (already completed)

**Impact**: Internal version change only, no user action required

### 2. AI Service Lifecycle

**Change**: AI services require explicit initialization and cleanup

**Impact**: Direct AI service usage needs code updates

**Migration Required**: Yes, if using AI services directly in code

### 3. Marketplace Configuration

**Change**: New marketplace configuration section required

**Impact**: Marketplace commands require configuration file

**Migration Required**: Yes, run `clnrm init` to generate

### 4. Service Plugin Interface

**Change**: Enhanced `ServicePlugin` trait with new methods

**Impact**: Custom plugins need interface updates

**Migration Required**: Yes, if you have custom plugins

### 5. README Badge Updates

**Change**: Version badges updated to v0.4.0

**Impact**: Documentation only, no functional impact

**Migration Required**: No

---

## Step-by-Step Upgrade

### Step 1: Update Dependencies

#### Update Cargo.toml

**Edit your `Cargo.toml`**:
```toml
[dependencies]
clnrm-core = "0.4.0"
clnrm-shared = "0.4.0"
```

Or if using from git:
```toml
[dependencies]
clnrm-core = { git = "https://github.com/seanchatmangpt/clnrm", tag = "v0.4.0" }
```

#### Update Dependencies

```bash
# Update Cargo.lock
cargo update

# Clean and rebuild
cargo clean
cargo build --release

# Verify compilation
echo $?  # Should be 0
```

**Expected output**:
```
    Updating crates.io index
   Compiling clnrm-shared v0.4.0
   Compiling clnrm-core v0.4.0
   Compiling clnrm v0.4.0
    Finished release [optimized] target(s) in 2m 15s
```

### Step 2: Install Ollama (Optional but Recommended)

Real AI features require Ollama. Skip this step to use simulated AI.

#### macOS Installation

```bash
# Install Ollama
brew install ollama

# Verify installation
ollama --version
```

#### Linux Installation

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Verify installation
ollama --version
```

#### Pull AI Model

```bash
# Pull recommended model (2.0GB download)
ollama pull llama3.2:3b

# Start Ollama service
ollama serve &

# Verify service
curl http://localhost:11434/api/version
```

**Expected output**:
```json
{"version":"0.1.0"}
```

#### Alternative: Smaller Model

For faster setup, use a smaller model:
```bash
# Pull smaller model (1.3GB)
ollama pull phi-3:mini

# Set as default
export OLLAMA_MODEL=phi-3:mini
echo 'export OLLAMA_MODEL=phi-3:mini' >> ~/.bashrc  # or ~/.zshrc
```

### Step 3: Initialize Marketplace Configuration

#### Generate Configuration

```bash
# Initialize marketplace configuration
clnrm init

# This updates cleanroom.toml with marketplace settings
```

#### Verify Configuration

**Check `cleanroom.toml`** contains:
```toml
[marketplace]
registry_url = "https://registry.clnrm.io"
cache_dir = "~/.clnrm/marketplace/cache"
verify_signatures = true

[marketplace.security]
scan_vulnerabilities = true
check_signatures = true
sandbox_validation = true
```

#### Manual Configuration (if needed)

If `clnrm init` doesn't create marketplace config, add manually:

```toml
[marketplace]
registry_url = "https://registry.clnrm.io"
cache_dir = "~/.clnrm/marketplace/cache"
verify_signatures = true
auto_update = false

[marketplace.security]
scan_vulnerabilities = true
check_signatures = true
sandbox_validation = true
max_plugin_size = "100MB"
allowed_sources = ["official", "verified"]
```

### Step 4: Update Service Plugins (If Applicable)

If you have custom service plugins, update them to the new interface.

#### Old Interface (v0.3.x)

```rust
pub trait ServicePlugin {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
}
```

#### New Interface (v0.4.0)

```rust
use clnrm_core::services::{ServicePlugin, ServiceHandle, HealthStatus};

pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;
    fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self) -> Result<HealthStatus>;
    fn scale(&self, instances: usize) -> Result<()>;
}
```

#### Migration Example

**Before (v0.3.x)**:
```rust
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<()> {
        // Start service
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        // Stop service
        Ok(())
    }
}
```

**After (v0.4.0)**:
```rust
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Start service
        let handle = ServiceHandle::new("my-service");
        Ok(handle)
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        // Stop service using handle
        info!("Stopping service: {}", handle.id());
        Ok(())
    }

    fn health_check(&self) -> Result<HealthStatus> {
        // Check service health
        Ok(HealthStatus::Healthy)
    }

    fn scale(&self, instances: usize) -> Result<()> {
        // Scale to N instances
        info!("Scaling to {} instances", instances);
        Ok(())
    }
}
```

### Step 5: Update AI Service Usage (If Applicable)

If your code directly uses AI services, update to new lifecycle management.

#### Old Usage (v0.3.x)

```rust
// AI was auto-initialized
let result = ai_function().await?;
```

#### New Usage (v0.4.0)

```rust
use clnrm_core::services::ai_intelligence::AIIntelligenceService;

// Explicit lifecycle management
let ai_service = AIIntelligenceService::new();

// Start service
let handle = match ai_service.start().await {
    Ok(h) => {
        info!("✅ Real AI service initialized");
        Some(h)
    },
    Err(e) => {
        warn!("⚠️ Ollama unavailable: {}", e);
        None
    }
};

// Use AI service
let result = ai_service.analyze(&data).await?;

// Cleanup
if let Some(h) = handle {
    ai_service.stop(h).await?;
}
```

### Step 6: Configure Service Management

Add service management configuration to `cleanroom.toml`:

```toml
[services]
# Auto-scaling configuration
auto_scaling = true
max_instances = 10
min_instances = 1
scale_up_threshold = 0.8
scale_down_threshold = 0.3

# Health monitoring
health_check_interval = 30
health_check_timeout = 10
health_check_retries = 3

# Resource management
cleanup_timeout = 60
restart_on_failure = true
max_restart_attempts = 3

# Service-specific settings
[services.database]
auto_scaling = false
max_instances = 1

[services.cache]
auto_scaling = true
max_instances = 5
```

### Step 7: Verify Installation

#### Run Basic Commands

```bash
# Check version
clnrm --version  # Should show v0.4.0

# Run self-test
clnrm self-test

# Test AI commands
clnrm ai-orchestrate
clnrm ai-predict --predict-failures
clnrm ai-optimize --execution-order

# Test marketplace
clnrm marketplace list
clnrm marketplace search database

# Test service management
clnrm services status
```

#### Expected Output

**Version check**:
```
clnrm 0.4.0
```

**Self-test**:
```
✅ All framework functionality validated
Framework Self-Test Results:
Total Tests: 5
Passed: 5
Failed: 0
```

**AI command (with Ollama)**:
```
🤖 Starting AI-powered test orchestration
✅ Real AI service initialized with Ollama
🧠 Using Ollama AI for genuine intelligence
```

**AI command (without Ollama)**:
```
🤖 Starting AI-powered test orchestration
⚠️ Ollama unavailable, using simulated AI
💡 To enable real AI, ensure Ollama is running
```

### Step 8: Run Test Suite

```bash
# Run all tests
cargo test --all

# Run integration tests
cargo test --test integration_ai_commands

# Run benchmarks
cargo bench ai_intelligence

# Run framework self-tests
clnrm self-test
```

**Expected results**:
- 159+ tests passing
- 0 test failures
- All benchmarks within expected ranges

---

## Configuration Changes

### New Configuration Sections

#### Marketplace Configuration

```toml
[marketplace]
registry_url = "https://registry.clnrm.io"
cache_dir = "~/.clnrm/marketplace/cache"
verify_signatures = true
auto_update = false

[marketplace.security]
scan_vulnerabilities = true
check_signatures = true
sandbox_validation = true
max_plugin_size = "100MB"
allowed_sources = ["official", "verified"]
```

#### AI Configuration

```toml
[ai]
# Ollama configuration
ollama_url = "http://localhost:11434"
ollama_model = "llama3.2:3b"
ollama_timeout = 120
ollama_temperature = 0.7
ollama_max_tokens = 500

# AI features
enable_real_ai = true
fallback_to_simulated = true
confidence_threshold = 0.8

# Model selection
models = [
  { name = "fast", model = "phi-3:mini", timeout = 10 },
  { name = "balanced", model = "llama3.2:3b", timeout = 30 },
  { name = "accurate", model = "llama3.2:14b", timeout = 60 }
]
default_model = "balanced"
```

#### Service Management Configuration

```toml
[services]
# Auto-scaling
auto_scaling = true
max_instances = 10
min_instances = 1
scale_up_threshold = 0.8
scale_down_threshold = 0.3

# Health monitoring
health_check_interval = 30
health_check_timeout = 10
health_check_retries = 3

# Resource management
cleanup_timeout = 60
restart_on_failure = true
max_restart_attempts = 3
```

### Environment Variables

#### New Variables

```bash
# AI configuration
export OLLAMA_MODEL=llama3.2:3b
export OLLAMA_URL=http://localhost:11434
export OLLAMA_TIMEOUT_SECONDS=120

# Marketplace configuration
export CLNRM_MARKETPLACE_CACHE=~/.clnrm/marketplace/cache
export CLNRM_MARKETPLACE_REGISTRY=https://registry.clnrm.io

# Service management
export CLNRM_AUTO_SCALING=true
export CLNRM_MAX_INSTANCES=10
```

#### Updated Variables

```bash
# Version updated
export CLNRM_VERSION=0.4.0

# Log level (unchanged)
export RUST_LOG=info
```

---

## Code Migration

### Migrating AI Service Usage

#### Pattern 1: Basic AI Usage

**Before**:
```rust
// No explicit initialization
let insights = generate_insights(&data).await?;
```

**After**:
```rust
use clnrm_core::services::ai_intelligence::AIIntelligenceService;

let ai = AIIntelligenceService::new();
let handle = ai.start().await?;
let insights = ai.generate_insights(&data).await?;
ai.stop(handle).await?;
```

#### Pattern 2: AI with Fallback

**Before**:
```rust
// No fallback handling
let result = ai_analyze(&data).await?;
```

**After**:
```rust
let ai = AIIntelligenceService::new();
let (use_real_ai, handle) = match ai.start().await {
    Ok(h) => (true, Some(h)),
    Err(e) => {
        warn!("Falling back to simulated AI: {}", e);
        (false, None)
    }
};

let result = if use_real_ai {
    ai.real_analyze(&data).await?
} else {
    ai.simulated_analyze(&data).await?
};

if let Some(h) = handle {
    ai.stop(h).await?;
}
```

### Migrating Service Plugin Implementation

#### Complete Migration Example

**Before (v0.3.x)**:
```rust
use clnrm_core::services::ServicePlugin;

pub struct MyDatabasePlugin {
    connection_string: String,
}

impl ServicePlugin for MyDatabasePlugin {
    fn start(&self) -> Result<()> {
        println!("Starting database");
        // Start database logic
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        println!("Stopping database");
        // Stop database logic
        Ok(())
    }
}
```

**After (v0.4.0)**:
```rust
use clnrm_core::services::{ServicePlugin, ServiceHandle, HealthStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct MyDatabasePlugin {
    connection_string: String,
    running: Arc<AtomicBool>,
}

impl ServicePlugin for MyDatabasePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        info!("Starting database service");

        // Start database logic
        self.running.store(true, Ordering::SeqCst);

        // Create and return handle
        let handle = ServiceHandle::new("my-database");
        Ok(handle)
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        info!("Stopping database service: {}", handle.id());

        // Stop database logic
        self.running.store(false, Ordering::SeqCst);

        Ok(())
    }

    fn health_check(&self) -> Result<HealthStatus> {
        if self.running.load(Ordering::SeqCst) {
            // Perform health check
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy("Service not running".to_string()))
        }
    }

    fn scale(&self, instances: usize) -> Result<()> {
        info!("Scaling database to {} instances", instances);

        // For databases, typically we don't scale
        // but we implement the interface
        if instances != 1 {
            warn!("Database plugin does not support multiple instances");
        }

        Ok(())
    }
}
```

### Migrating Test Files

Test files typically don't require changes, but verify:

```bash
# Run tests after upgrade
cargo test --all

# Check for deprecation warnings
cargo build 2>&1 | grep -i "warning"

# Update any deprecated usages
```

---

## Testing Recommendations

### Pre-Upgrade Testing

**Before upgrading production**:

1. **Test in Staging**:
   ```bash
   # Create staging environment
   git checkout -b staging-v0.4.0

   # Perform upgrade in staging
   # Run full test suite
   # Verify all features work
   ```

2. **Run Comprehensive Tests**:
   ```bash
   # All unit tests
   cargo test --all

   # Integration tests
   cargo test --test '*_integration_test'

   # Benchmarks
   cargo bench

   # Self-tests
   clnrm self-test
   ```

3. **Test AI Features**:
   ```bash
   # With Ollama
   ollama serve &
   clnrm ai-orchestrate --predict-failures

   # Without Ollama (simulated)
   pkill ollama
   clnrm ai-orchestrate
   ```

4. **Test Marketplace**:
   ```bash
   clnrm marketplace search database
   clnrm marketplace install postgres-plugin
   clnrm marketplace list
   ```

### Post-Upgrade Validation

**After upgrading**:

1. **Smoke Tests**:
   ```bash
   clnrm --version
   clnrm --help
   clnrm self-test
   ```

2. **Feature Tests**:
   ```bash
   # Core features
   clnrm run tests/
   clnrm validate tests/

   # AI features
   clnrm ai-orchestrate
   clnrm ai-predict --analyze-history
   clnrm ai-optimize --execution-order

   # Marketplace
   clnrm marketplace list

   # Service management
   clnrm services status
   ```

3. **Performance Tests**:
   ```bash
   # Run benchmarks
   cargo bench ai_intelligence

   # Compare with baseline
   cargo bench -- --baseline before_upgrade
   ```

4. **Integration Tests**:
   ```bash
   # Full integration suite
   cargo test --test system_integration_test
   cargo test --test integration_ai_commands
   ```

### Regression Testing

**Check for regressions**:

```bash
# Compare test results
cargo test --all 2>&1 | tee test_results_v0.4.0.txt
diff test_results_v0.3.x.txt test_results_v0.4.0.txt

# Check for new failures
grep -i "FAILED" test_results_v0.4.0.txt

# Verify performance
cargo bench -- --baseline v0.3.x
```

---

## Troubleshooting

### Common Issues

#### Issue 1: Ollama Connection Failures

**Symptoms**:
```
⚠️ Ollama unavailable, using simulated AI
Error: Connection refused (os error 61)
```

**Solutions**:

1. **Verify Ollama is running**:
   ```bash
   curl http://localhost:11434/api/version
   ```

2. **Start Ollama**:
   ```bash
   ollama serve &
   ```

3. **Check Ollama logs**:
   ```bash
   tail -f ~/.ollama/logs/server.log
   ```

4. **Test Ollama directly**:
   ```bash
   ollama run llama3.2:3b "test"
   ```

5. **Use fallback mode**:
   ```bash
   # Works without Ollama
   clnrm ai-orchestrate  # Uses simulated AI
   ```

#### Issue 2: Marketplace Configuration Missing

**Symptoms**:
```
Error: Marketplace configuration not found
```

**Solutions**:

1. **Run init**:
   ```bash
   clnrm init
   ```

2. **Manually add configuration** to `cleanroom.toml`:
   ```toml
   [marketplace]
   registry_url = "https://registry.clnrm.io"
   cache_dir = "~/.clnrm/marketplace/cache"
   verify_signatures = true
   ```

3. **Create cache directory**:
   ```bash
   mkdir -p ~/.clnrm/marketplace/cache
   ```

#### Issue 3: Custom Plugin Compilation Errors

**Symptoms**:
```
error: no method named `health_check` found
```

**Solution**: Update plugin to new interface (see [Code Migration](#code-migration))

#### Issue 4: Test Failures After Upgrade

**Symptoms**:
```
test result: FAILED. 150 passed; 9 failed
```

**Solutions**:

1. **Check deprecation warnings**:
   ```bash
   cargo build 2>&1 | grep -i "warning"
   ```

2. **Update deprecated code**:
   ```bash
   # Follow compiler suggestions
   cargo fix --edition
   ```

3. **Run specific failing test**:
   ```bash
   cargo test failing_test_name -- --nocapture
   ```

4. **Check for breaking changes** in this guide

#### Issue 5: Performance Degradation

**Symptoms**: Tests run slower after upgrade

**Solutions**:

1. **Run benchmarks**:
   ```bash
   cargo bench
   ```

2. **Enable AI optimization**:
   ```bash
   clnrm ai-optimize --auto-apply
   ```

3. **Check resource usage**:
   ```bash
   clnrm services status
   ```

4. **Scale services if needed**:
   ```bash
   clnrm services scale my-service 3
   ```

### Getting Help

**If you encounter issues**:

1. **Check documentation**:
   - [Release Notes](./releases/v0.4.0.md)
   - [AI Commands Reference](./AI_COMMANDS_REFERENCE.md)
   - [Troubleshooting Guide](./testing/troubleshooting-guide.md)

2. **Search existing issues**:
   ```bash
   # Search GitHub issues
   open "https://github.com/seanchatmangpt/clnrm/issues?q=is%3Aissue"
   ```

3. **Create new issue** with:
   - Error message
   - Steps to reproduce
   - Environment details:
     ```bash
     clnrm --version
     rustc --version
     uname -a
     ```

4. **Join discussions**:
   - [GitHub Discussions](https://github.com/seanchatmangpt/clnrm/discussions)

---

## Rollback Procedure

If you need to rollback to v0.3.x:

### Step 1: Restore Backups

```bash
# Restore configuration
cp tests.backup/* tests/
cp cleanroom.toml.backup cleanroom.toml 2>/dev/null || true

# Restore Cargo files
cp Cargo.toml.backup Cargo.toml
cp Cargo.lock.backup Cargo.lock
```

### Step 2: Downgrade Dependencies

```bash
# Update Cargo.toml to v0.3.2 (for downgrade only)
sed -i 's/0\.4\.0/0.3.2/g' Cargo.toml

# Update and rebuild
cargo update
cargo build --release
```

### Step 3: Verify Rollback

```bash
# Check version
clnrm --version  # Should show v0.3.2 (downgrade verification)

# Run tests
cargo test --all

# Verify functionality
clnrm self-test
```

### Step 4: Remove v0.4.0 Artifacts

```bash
# Remove marketplace cache
rm -rf ~/.clnrm/marketplace/

# Remove v0.4.0 configuration sections from cleanroom.toml
# (manually remove [marketplace], [ai], [services] sections)
```

---

## Summary

**Upgrade checklist**:

- [ ] Backup current installation
- [ ] Update dependencies to v0.4.0
- [ ] Install Ollama (optional)
- [ ] Initialize marketplace configuration
- [ ] Update custom service plugins (if any)
- [ ] Update AI service usage (if any)
- [ ] Configure service management
- [ ] Run verification tests
- [ ] Test AI commands
- [ ] Test marketplace commands
- [ ] Run full test suite
- [ ] Verify performance
- [ ] Document any issues

**Success criteria**:

- All tests passing
- AI commands work (with or without Ollama)
- Marketplace commands functional
- Service management operational
- No regressions in existing functionality

**Estimated time**: 30-60 minutes

**Need help?** Open an issue on [GitHub](https://github.com/seanchatmangpt/clnrm/issues)

---

**Upgrade Guide Version**: 1.0
**Last Updated**: October 16, 2025
**Target Version**: v0.4.0
