# gVisor Migration - Quick Start Guide

**TL;DR:** This migration replaces testcontainers with gVisor for better security and no Docker dependency. Most tests require **zero changes** thanks to the Backend trait abstraction.

## 📚 Documentation Overview

| Document | Purpose | When to Read |
|----------|---------|--------------|
| **[This README](GVISOR_MIGRATION_README.md)** | Quick start & overview | Start here! |
| **[Executive Summary](GVISOR_MIGRATION_SUMMARY.md)** | Complete overview, deliverables, timeline | Before planning |
| **[Migration Plan](GVISOR_MIGRATION_PLAN.md)** | Detailed architecture, strategy, examples | During implementation |
| **[Test Migration Guide](GVISOR_TEST_MIGRATION_GUIDE.md)** | Test-by-test migration instructions | During test migration |
| **[Backend Skeleton](../crates/clnrm-core/src/backend/gvisor_skeleton.rs)** | Reference implementation | During development |

## 🎯 What's Changing?

### Current: Testcontainers
```rust
CleanroomEnvironment → TestcontainerBackend → Docker Daemon
```

### New: gVisor
```rust
CleanroomEnvironment → GVisorBackend → runsc (no Docker!)
```

### Impact on Tests
- **70% of tests:** ✅ No changes needed (Backend trait abstraction!)
- **25% of tests:** ⚠️ Minor service plugin updates
- **5% of tests:** 🔴 New performance baselines

## 🚀 Quick Start (Developers)

### 1. Install gVisor (5 minutes)

```bash
# Ubuntu/Debian
curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
sudo apt-get update
sudo apt-get install -y runsc skopeo

# Verify
runsc --version
skopeo --version
```

### 2. Start Implementation (30 minutes)

```bash
# Copy skeleton to implementation
cd /home/user/clnrm
cp crates/clnrm-core/src/backend/gvisor_skeleton.rs \
   crates/clnrm-core/src/backend/gvisor.rs

# Update backend module
# Add to crates/clnrm-core/src/backend/mod.rs:
#   pub mod gvisor;
#   pub use gvisor::GVisorBackend;
```

### 3. Implement Core Methods (Follow TODOs)

Open `crates/clnrm-core/src/backend/gvisor.rs` and search for `TODO`:

1. **pull_image()** - Pull OCI images via skopeo
2. **create_bundle()** - Create OCI bundle directory
3. **generate_oci_config()** - Generate config.json
4. **execute_in_container()** - Execute via runsc
5. **cleanup_bundle()** - Clean up containers

### 4. Run Tests

```bash
# Add feature flag to Cargo.toml
# [features]
# gvisor = []

# Run with gVisor
cargo test --features gvisor

# Compare with testcontainers
cargo test  # Old way
cargo test --features gvisor  # New way
```

## 📊 Migration Timeline

| Phase | Duration | Owner | Key Deliverable |
|-------|----------|-------|-----------------|
| **Phase 1** | Week 1-2 | Backend team | GVisorBackend implementation |
| **Phase 2** | Week 3 | Service team | Service plugins updated |
| **Phase 3** | Week 4-5 | QA team | All tests migrated |
| **Phase 4** | Week 6 | DevOps | CI/CD integration |
| **Phase 5** | Week 7-8 | All teams | Production cutover |

**Total:** 8 weeks

## 🔍 Test Migration Examples

### Example 1: No Changes (70% of tests)

```rust
// This test works with BOTH backends - no changes!
#[tokio::test]
async fn test_container_execution() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;  // Auto-detects backend

    let result = env.execute_in_container(
        "test",
        &["echo".to_string(), "hello".to_string()],
        None,
        None,
    ).await?;

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    Ok(())
}
```

### Example 2: Service Plugin Update (25% of tests)

Test code stays the same:
```rust
#[tokio::test]
async fn test_service() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;

    // Plugin uses gVisor internally - test unchanged!
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;
    env.stop_service(&handle.id).await?;
    Ok(())
}
```

But plugin implementation changes:
```diff
// In src/services/generic.rs
- use testcontainers::runners::AsyncRunner;
- use testcontainers::{GenericImage, ImageExt};
+ use crate::backend::{GVisorBackend, Backend};

impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
-       let image = GenericImage::new(self.image.clone(), self.tag.clone());
-       let node = container_request.start().await?;
+       let backend = GVisorBackend::new(&image_ref)?;
+       let container_id = backend.start_service(&["sleep", "3600"]).await?;
    }
}
```

### Example 3: Performance Test Update (5% of tests)

```diff
#[tokio::test]
async fn bench_startup() -> Result<()> {
    let start = Instant::now();
    let env = CleanroomEnvironment::new().await?;
    let result = env.execute_in_container("test", &["echo", "test"], None, None).await?;
    let duration = start.elapsed();

-   assert!(duration < Duration::from_secs(2));  // Old baseline
+   assert!(duration < Duration::from_secs(3));  // gVisor slightly slower
    Ok(())
}
```

## 📋 Migration Checklist

### Pre-Migration
- [ ] Read [Executive Summary](GVISOR_MIGRATION_SUMMARY.md)
- [ ] Install gVisor (runsc + skopeo)
- [ ] Review [Migration Plan](GVISOR_MIGRATION_PLAN.md)

### Phase 1: Backend (Weeks 1-2)
- [ ] Copy skeleton to gvisor.rs
- [ ] Implement pull_image()
- [ ] Implement create_bundle()
- [ ] Implement generate_oci_config()
- [ ] Implement execute_in_container()
- [ ] Implement cleanup_bundle()
- [ ] Write unit tests
- [ ] Write integration tests

### Phase 2: Services (Week 3)
- [ ] Update GenericContainerPlugin
- [ ] Update SurrealDbPlugin
- [ ] Update OtelCollectorPlugin
- [ ] Test service lifecycle

### Phase 3: Tests (Weeks 4-5)
- [ ] Migrate docker_integration.rs (12 tests)
- [ ] Migrate e2e_basic_workflow.rs (8 tests)
- [ ] Migrate service tests (10+ tests)
- [ ] Migrate advanced feature tests

### Phase 4: CI/CD (Week 6)
- [ ] Install runsc in CI
- [ ] Update GitHub Actions
- [ ] Run parallel tests (both backends)
- [ ] Create performance dashboard

### Phase 5: Cutover (Weeks 7-8)
- [ ] Deprecate TestcontainerBackend
- [ ] Update documentation
- [ ] Final validation
- [ ] Announce migration

## 🎓 Key Concepts

### OCI Bundle Structure
```
/tmp/clnrm-gvisor-bundles/container-id/
├── config.json          # OCI runtime spec
└── rootfs/              # Container filesystem
    ├── bin/
    ├── etc/
    └── ...
```

### runsc Workflow
```bash
# 1. Create container
runsc create --bundle /path/to/bundle container-id

# 2. Start container
runsc start container-id

# 3. Wait for completion
runsc wait container-id

# 4. Get state
runsc state container-id

# 5. Clean up
runsc delete container-id
```

### Port Allocation
```rust
// gVisor doesn't auto-allocate ports like Docker
// We use PortAllocator for ephemeral ports (49152-65535)
let allocator = PortAllocator::new();
let port = allocator.allocate().await?;  // e.g., 50123

// Release when done
allocator.release(port).await;
```

## ⚠️ Common Issues

### Issue: "runsc not found"
**Solution:**
```bash
# Check installation
which runsc
runsc --version

# Reinstall if needed
sudo apt-get install -y runsc
```

### Issue: "Image pull timeout"
**Solution:**
```bash
# Pre-pull common images
skopeo copy docker://alpine:latest oci:/tmp/test-images/alpine:latest

# Or increase timeout
let backend = GVisorBackend::new("docker.io/library/alpine:latest")?
    .with_startup_timeout(Duration::from_secs(120));
```

### Issue: "Port already in use"
**Solution:**
```rust
// Always release ports in cleanup
if let Some(port) = allocated_port {
    port_allocator.release(port).await?;
}
```

### Issue: "Bundle directory leak"
**Solution:**
```rust
// Implement Drop for cleanup
impl Drop for GVisorBackend {
    fn drop(&mut self) {
        // Clean up bundles
    }
}
```

## 📈 Success Metrics

### Must-Have (Week 8)
- ✅ All tests passing with gVisor
- ✅ Zero orphaned containers
- ✅ Zero bundle leaks
- ✅ CI stable (no flaky tests)

### Nice-to-Have (Week 8)
- ⚠️ Performance within 20% of testcontainers
- ⚠️ Memory usage <2x testcontainers
- ⚠️ Startup time <1s for cached images

## 🆘 Getting Help

### Documentation
- Start with: [Executive Summary](GVISOR_MIGRATION_SUMMARY.md)
- Deep dive: [Migration Plan](GVISOR_MIGRATION_PLAN.md)
- Test migration: [Test Guide](GVISOR_TEST_MIGRATION_GUIDE.md)
- Code reference: [Backend Skeleton](../crates/clnrm-core/src/backend/gvisor_skeleton.rs)

### External Resources
- [gVisor Docs](https://gvisor.dev/docs/)
- [OCI Runtime Spec](https://github.com/opencontainers/runtime-spec)
- [runsc CLI](https://gvisor.dev/docs/user_guide/quick_start/)
- [Skopeo Docs](https://github.com/containers/skopeo)

### Support Channels
- GitHub Issues: Bug reports
- Slack #cleanroom-dev: Questions
- Weekly sync: Thursdays 2pm PT

## 🎉 Benefits

**Security:**
- ✅ gVisor sandbox (better isolation than Docker)
- ✅ Reduced attack surface
- ✅ Application kernel in userspace

**Architecture:**
- ✅ No Docker daemon dependency
- ✅ Direct OCI image usage
- ✅ Cleaner backend abstraction

**Testing:**
- ✅ More deterministic execution
- ✅ Better hermetic isolation
- ✅ Easier debugging (no Docker overhead)

## 🚦 Decision Tree

```
┌─────────────────────────────────────────┐
│ Am I writing a new test?                │
└─────────────┬───────────────────────────┘
              │
              ├─ YES → Use CleanroomEnvironment::new()
              │        (auto-detects gVisor)
              │
              └─ NO → Migrating existing test?
                      │
                      ├─ Basic execution → No changes!
                      ├─ Service plugin → Update plugin
                      └─ Performance → New baseline
```

## 📝 Next Actions

1. **Read** [Executive Summary](GVISOR_MIGRATION_SUMMARY.md) (10 min)
2. **Install** gVisor and skopeo (5 min)
3. **Copy** skeleton to gvisor.rs (1 min)
4. **Implement** core methods (follow TODOs)
5. **Test** with `cargo test --features gvisor`

## 📅 Weekly Checkpoints

- **Week 2:** Phase 1 complete (backend implementation)
- **Week 3:** Phase 2 complete (service plugins)
- **Week 5:** Phase 3 complete (tests migrated)
- **Week 6:** Phase 4 complete (CI/CD)
- **Week 8:** Phase 5 complete (production cutover)

---

**Questions?** Read the [FAQ in Migration Plan](GVISOR_MIGRATION_PLAN.md#common-migration-issues--solutions)

**Ready to start?** Open [Backend Skeleton](../crates/clnrm-core/src/backend/gvisor_skeleton.rs)

**Need details?** Read [Executive Summary](GVISOR_MIGRATION_SUMMARY.md)

---

**Good luck with the migration! 🚀**
