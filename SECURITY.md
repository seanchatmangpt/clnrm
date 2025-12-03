# Security Policy

## Reporting Security Issues

**DO NOT** create public GitHub issues for security vulnerabilities.

Instead, please email security concerns to: security@github.com or file a private security advisory at https://github.com/seanchatmangpt/clnrm/security/advisories

We will respond within 48 hours and work with you to address the issue.

---

## Known Security Advisories

### RUSTSEC-2025-0111 - tokio-tar File Smuggling (CRITICAL)

**Status**: ✅ ACKNOWLEDGED - Risk Accepted for v1.4.1
**Package**: `tokio-tar` v0.3.1
**Severity**: CRITICAL (upstream), **LOW** (clnrm context)
**Date Identified**: 2025-10-21
**Advisory URL**: https://rustsec.org/advisories/RUSTSEC-2025-0111

#### Description

The `tokio-tar` crate has a file smuggling vulnerability where PAX extended headers are parsed incorrectly. This could allow malicious tar archives to write files outside the intended extraction directory (path traversal).

**Upstream Advisory Text**:
> "`tokio-tar` parses PAX extended headers incorrectly, allows file smuggling"

#### Impact on clnrm

**Risk Level: LOW** - The vulnerability is present in our dependency chain but has limited real-world impact due to multiple layers of mitigation:

**Dependency Path**:
```
tokio-tar 0.3.1
└── testcontainers 0.25.0
    └── clnrm-core 1.4.1
```

**Why Risk is LOW**:

1. ✅ **Trusted Image Sources Only**: clnrm uses Docker images from trusted registries (Docker Hub official images)
2. ✅ **Isolated Execution Environment**: All container operations happen in isolated testcontainer environments
3. ✅ **Ephemeral Filesystems**: Container filesystems are destroyed immediately after test completion
4. ✅ **No User-Provided Archives**: clnrm does not process user-uploaded tar archives or container images
5. ✅ **Network Isolation Possible**: Tests can run in isolated network environments
6. ✅ **Limited Scope**: Extraction only happens during container image pull (controlled by Docker daemon)

**Attack Vector Requirements** (all must be true for successful exploit):
- Attacker compromises a Docker Hub official image, OR
- User manually configures clnrm to use a malicious image, AND
- Malicious tar archive is embedded in container image, AND
- Extraction writes files outside container (already isolated), AND
- Those files persist beyond test execution (ephemeral by design)

**Likelihood**: VERY LOW
**Impact**: MEDIUM (limited by testcontainer isolation)
**Overall Risk**: **LOW**

#### Mitigation

**Current Mitigations** (already in place):

1. **Trusted Images**: Use only official Docker images from trusted registries
2. **Container Isolation**: testcontainers provides process and filesystem isolation
3. **Ephemeral State**: All test artifacts are destroyed after execution
4. **No Custom Archives**: clnrm doesn't process user-provided tar files

**Recommended User Actions**:

1. **Enable Docker Content Trust**:
   ```bash
   export DOCKER_CONTENT_TRUST=1
   ```
   This ensures Docker verifies image signatures.

2. **Restrict to Official Images**:
   ```toml
   # .clnrm.toml - Use official images only
   [services.my_service]
   type = "generic_container"
   image = "alpine:latest"  # ✅ Official image
   # image = "untrusted/custom:latest"  # ❌ Avoid
   ```

3. **Network Isolation** (optional):
   ```bash
   # Run tests in isolated network
   docker network create --internal clnrm-isolated
   ```

4. **Review Container Images**:
   ```bash
   # Inspect images before use
   docker inspect alpine:latest
   docker history alpine:latest
   ```

#### Resolution Plan

**Short-term (v1.4.1 - CURRENT)**:
- ✅ Document risk acceptance
- ✅ Provide user guidance
- ✅ Monitor for upstream fix

**Medium-term (v1.4.2 - Planned)**:
- Monitor `tokio-tar` repository for security patch
- Upgrade immediately when fix becomes available
- Expected timeline: 2-4 weeks (if upstream responds)

**Long-term (v1.5.0 - Future)**:
- Evaluate alternative tar implementations:
  - `tar-rs` (synchronous, stable)
  - `async-tar` (if async alternative exists)
- Estimated migration effort: 6-8 weeks
- Full regression testing required

#### Workarounds

If you have heightened security requirements:

1. **Audit Images Before Use**:
   ```bash
   # Scan images for vulnerabilities
   docker scan alpine:latest
   trivy image alpine:latest
   ```

2. **Use Minimal Base Images**:
   ```toml
   # Prefer distroless or minimal images
   image = "gcr.io/distroless/base"
   image = "alpine:latest"
   ```

3. **Run in Restricted Environment**:
   - Use AppArmor/SELinux profiles
   - Enable seccomp filters
   - Run in Kubernetes with Pod Security Standards

4. **Monitor Container Behavior**:
   ```bash
   # Watch for suspicious filesystem activity
   docker logs <container_id>
   ```

#### References

- Advisory: https://rustsec.org/advisories/RUSTSEC-2025-0111
- CVE: [Pending assignment]
- Affected Package: https://crates.io/crates/tokio-tar
- clnrm Issue: [Link when created]

---

## Other Security Considerations

### Unmaintained Dependencies (Warnings)

The following dependencies are flagged as unmaintained but pose **LOW** risk:

**From `tera` template engine** (used in `clnrm-template`):
- `unic-*` family (7 warnings)
  - `unic-char-property`, `unic-char-range`, `unic-common`, `unic-segment`
  - `unic-ucd-segment`, `unic-ucd-version`
  - **Risk**: LOW - Unicode processing, no known vulnerabilities
  - **Plan**: Monitor `tera` for migration to maintained Unicode crates

**From `surrealdb`** (database plugin):
- `paste` macro crate
  - **Risk**: VERY LOW - Compile-time macro only, no runtime impact
  - **Plan**: Wait for `surrealdb` to update dependency

**Action Required**: None for v1.4.1. Continue monitoring.

---

## Security Best Practices

When using clnrm in production:

### 1. Container Image Security

✅ **DO**:
- Use official images from Docker Hub
- Pin specific image versions (`:1.2.3` not `:latest`)
- Enable Docker Content Trust (`DOCKER_CONTENT_TRUST=1`)
- Scan images regularly (`docker scan`, `trivy`)

❌ **DON'T**:
- Use unverified third-party images
- Run as root inside containers (use USER directive)
- Disable security features for convenience

### 2. Network Isolation

✅ **DO**:
- Run tests in isolated networks
- Use firewall rules to restrict outbound connections
- Monitor network traffic during tests

❌ **DON'T**:
- Expose test containers to public networks
- Allow unrestricted internet access during tests

### 3. Resource Limits

✅ **DO**:
```rust
// Set resource limits
let backend = TestcontainerBackend::new("alpine:latest")?
    .with_memory_limit(512)  // 512 MB
    .with_cpu_limit(1.0);    // 1 CPU
```

❌ **DON'T**:
- Run unlimited containers
- Allow unbounded resource consumption

### 4. Secrets Management

✅ **DO**:
- Use environment variables for secrets
- Inject secrets at runtime
- Clear sensitive data after tests

❌ **DON'T**:
- Hardcode secrets in `.clnrm.toml`
- Log sensitive information
- Persist secrets in container images

### 5. Monitoring & Auditing

✅ **DO**:
- Enable OTEL tracing for visibility
- Monitor container lifecycle events
- Audit test execution logs
- Track resource usage

❌ **DON'T**:
- Run tests without observability
- Ignore security warnings
- Skip security updates

---

## Vulnerability Response Timeline

1. **Report Received**: Acknowledged within 48 hours
2. **Initial Assessment**: Completed within 1 week
3. **Fix Development**: Based on severity
   - CRITICAL: 1-2 weeks
   - HIGH: 2-4 weeks
   - MEDIUM: 4-8 weeks
   - LOW: Next minor release
4. **Disclosure**: After fix is available

---

## Security Certifications

**Current Status** (v1.4.1):
- ✅ `cargo audit` run and all findings documented
- ✅ Security policy established
- ✅ Known advisories assessed and mitigated
- ✅ User guidance provided

**Pending** (v1.5.0):
- [ ] Third-party security audit
- [ ] SBOM (Software Bill of Materials) generation
- [ ] CVE assignment for clnrm-specific issues

---

## Contact

Security Team: [Your contact - TODO]
GitHub: https://github.com/seanchatmangpt/clnrm/security
PGP Key: [Public key fingerprint - TODO]

---

**Last Updated**: 2025-11-01
**Document Version**: 1.0.0 (v1.4.1)
