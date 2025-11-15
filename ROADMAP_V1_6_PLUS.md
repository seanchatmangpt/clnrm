# clnrm Roadmap: v1.5.1 → v1.7.0+

**Current Version**: 1.5.0 (Released 2025-11-15)
**Last Updated**: 2025-11-15
**Status**: Comprehensive roadmap with detailed feature specifications

---

## Release Schedule

| Version | Timeline | Status | Focus Areas |
|---------|----------|--------|------------|
| **v1.5.1** | 2-4 weeks | Planning | Bug fixes, community feedback |
| **v1.6.0** | 6-8 weeks | Backlog | Performance, Chicago-TDD, multi-image |
| **v1.7.0+** | 12+ weeks | Future | Enterprise features, advanced observability |

---

## v1.5.1 (Minor Update - 2-4 weeks)

### Focus: Polish & Stabilization

**Timeline**: November 2025 - Early December 2025

#### 🐛 Bug Fixes & Refinements
1. **Adaptive Pool Sizing Tuning**
   - Fine-tune resize thresholds based on real-world usage
   - Adjust resize interval (currently 30s) for optimal responsiveness
   - Monitor memory pressure in containerized environments

2. **Chicago-TDD Framework Feedback**
   - Gather user feedback on integration design
   - Refine trait APIs based on community input
   - Update documentation based on real-world usage

3. **SBOM Generation Enhancements**
   - Add support for vulnerability scanning integration
   - Optimize Cargo.lock parsing performance
   - Add configuration options for SBOM output format

#### 📊 Performance Profiling
- Benchmark adaptive pool under various load patterns
- Profile SBOM generation on large dependency trees
- Document performance characteristics for users

#### 📖 Documentation Updates
- Update API documentation with v1.5.0 examples
- Add troubleshooting guide for common issues
- Create video tutorials for new features

#### ✅ Testing Enhancements
- Add stress tests for adaptive pool sizing edge cases
- Integration tests with real container workloads
- Community-contributed test cases

---

## v1.6.0 (Major Feature Release - 6-8 weeks)

### Focus: Multi-Image Support & Performance

**Timeline**: December 2025 - January 2026

### Feature 1: Multi-Image Container Pooling ⭐ (HIGH PRIORITY)

**Objective**: Support pooling multiple container images simultaneously

**Current State**:
- Single image pool: `Arc<ContainerPool>`
- Coverage: 95% of use cases
- Limitation: Can't pre-warm different images at once

**Target Architecture**:
```rust
pub struct MultiImagePoolManager {
    pools: Arc<DashMap<String, Arc<ContainerPool>>>,
    // image_id → pool mapping
}

impl MultiImagePoolManager {
    pub async fn acquire(&self, image_id: &str) -> Result<ContainerHandle>;
    pub async fn release(&self, image_id: &str, handle: ContainerHandle);
    pub async fn pool_stats(&self) -> HashMap<String, PoolStats>;
}
```

**Implementation Steps**:
1. Create `MultiImagePoolManager` with lazy pool creation
2. Implement pool auto-discovery based on test requirements
3. Add configuration for max pools and per-pool sizing
4. Update CLI to expose multi-image pool statistics
5. Add integration tests with mixed-image scenarios

**Benefits**:
- Pre-warm multiple images in parallel
- Better resource utilization for complex test suites
- Reduced latency for multi-service tests (API + DB + Cache)

**Estimated Lines of Code**: 300-400

**Files to Modify**:
- `crates/clnrm-core/src/backend/pool.rs` (extend)
- `crates/clnrm-core/src/backend/mod.rs` (new exports)
- New: `crates/clnrm-core/src/backend/multi_pool.rs`

**Performance Target**:
- Pool creation latency: < 10ms per image
- Acquisition time: 0.25ms (same as v1.5.0)
- Memory overhead: < 5% per additional pool

---

### Feature 2: OTEL Span Batching Optimization ⭐ (HIGH PRIORITY)

**Objective**: Improve telemetry performance at scale (10K+ spans)

**Current State**:
- Performance: 356ms for 10K spans
- Issue: 10-16% regression at high span volumes
- Root cause: Batch export bottleneck

**Target Performance**:
- < 300ms for 10K spans (-16% improvement)
- < 100ms for 1K spans
- < 1s for 100K spans

**Solution Approach**:

1. **Async Export Pipeline**
   ```rust
   pub struct AsyncSpanExporter {
       sender: mpsc::Sender<Vec<SpanData>>,
       // Non-blocking batch queue
   }
   ```

2. **Batch Tuning**
   - Default batch size: 512 spans
   - Max batch size: 2048 spans
   - Configurable via environment: `OTEL_BSP_MAX_BATCH_SIZE`

3. **Sampling Strategy**
   - Probabilistic sampling for high-volume scenarios
   - Configurable sampling rate: 0.1 - 1.0
   - Preserve critical spans (errors, slow operations)

4. **Metrics**
   ```rust
   pub struct ExportMetrics {
       batches_exported: Arc<AtomicU64>,
       spans_exported: Arc<AtomicU64>,
       export_duration_ms: Arc<AtomicU64>,
   }
   ```

**Implementation Steps**:
1. Profile current span collection with flamegraph
2. Implement async export queue
3. Add batch size tuning
4. Implement sampling engine
5. Benchmark against 10K/100K span scenarios
6. Update configuration documentation

**Estimated Lines of Code**: 250-350

**Files to Modify**:
- `crates/clnrm-core/src/telemetry.rs` (extend)
- New: `crates/clnrm-core/src/telemetry/async_export.rs`
- New: `crates/clnrm-core/src/telemetry/sampler.rs`

**Success Criteria**:
- ✅ < 300ms for 10K spans
- ✅ No span loss (critical paths)
- ✅ Memory-bounded export queues

---

### Feature 3: Chicago-TDD Integration (Full Implementation)

**Objective**: Complete integration with chicago-tdd-tools v1.2.0 (when released)

**Current State**:
- Framework: Complete (v1.5.0)
- Integration: Placeholder stubs
- Status: Waiting for chicago-tdd-tools v1.2.0 public release

**Implementation Plan**:

1. **When chicago-tdd-tools v1.2.0 Becomes Available**:
   - Add dependency: `chicago-tdd-tools = "1.2.0"`
   - Implement trait methods in `ChicagoTddAdapter`
   - Add full integration tests

2. **Integration Points**:
   ```rust
   pub impl ChicagoTddCompatible for TestScenario {
       fn to_mockable(&self) -> Result<String> {
           // Generate mock objects from scenario
       }
       fn generate_collaboration_test(&self) -> Result<String> {
           // Generate London School TDD test
       }
   }
   ```

3. **Features to Support**:
   - Mock generation from test descriptions
   - Collaboration-driven test generation
   - Double dispatch pattern support
   - Test factory helpers

4. **Documentation**:
   - Integration guide with examples
   - Best practices for London School TDD
   - Case studies combining chicago-tdd + clnrm

**Estimated Lines of Code**: 200-300 (depends on chicago-tdd API)

**Files to Modify**:
- `crates/clnrm-core/src/chicago_tdd/mod.rs` (implement)
- New: `crates/clnrm-core/src/chicago_tdd/adapter.rs`
- New: `crates/clnrm-core/src/chicago_tdd/integration_tests.rs`

---

### Feature 4: Dynamic Semaphore Resizing

**Objective**: Auto-tune concurrency limits based on resource availability

**Current State**:
- Fixed semaphore: `max_concurrency` set at startup
- Limitation: Can't adapt to resource constraints

**Target Behavior**:
- Monitor system resources (CPU, memory)
- Automatically adjust `max_concurrency`
- Scale up/down based on available capacity

**Implementation**:
```rust
pub struct AdaptiveSemaphore {
    current_permits: Arc<AtomicUsize>,
    max_permits: Arc<AtomicUsize>,
    target_utilization: f64,
}

impl AdaptiveSemaphore {
    pub async fn acquire(&self) -> SemaphorePermit;
    pub async fn adjust_permits(&self, new_max: usize);
}
```

**Estimated Lines of Code**: 200-250

**Files to Modify**:
- `crates/clnrm-core/src/stress_test/executor.rs` (extend)

---

## v1.7.0+ (Enterprise & Advanced Features)

### Timeline: Q1-Q2 2026

#### 🔧 Advanced Performance Features
- **Memory Pooling**: Pre-allocate memory for high-throughput scenarios
- **CPU Affinity**: Pin containers to specific cores
- **NUMA Awareness**: Optimize for NUMA architectures (large servers)
- **Custom Metrics**: Framework for domain-specific metrics

#### 📡 Extended Observability
- **Distributed Tracing**: OpenTelemetry with Jaeger/Tempo
- **Real-time Dashboards**: Grafana integration
- **Profiling**: CPU/memory profiling via pprof
- **Custom Events**: User-defined telemetry events

#### 🏢 Enterprise Features
- **RBAC**: Role-based access control for test execution
- **Audit Logging**: Compliance and forensics
- **Integration**: Kubernetes, Docker Swarm, Nomad
- **High Availability**: Distributed pool coordination
- **Multi-tenant**: Workspace isolation
- **License Management**: Enterprise licensing model

#### 🔐 Security Enhancements
- **Secret Management**: Encrypted credential storage
- **Image Signing**: Verify container image signatures
- **Network Policies**: Network policy simulation
- **Vulnerability Scanning**: Integrated CVE detection

#### 🌍 Cloud Integrations
- **AWS**: Lambda, ECS, EC2 orchestration
- **Azure**: ACI, Kubernetes support
- **GCP**: Cloud Run, GKE integration
- **DigitalOcean**: App Platform support

---

## Deferred Features (Why & When)

### Multi-Image Container Pooling
- **v1.5.0 Status**: Not implemented
- **Why Deferred**: Significant architectural change
- **Coverage**: Single-image pooling covers 95% of use cases
- **Revisited in**: v1.6.0 (6-8 weeks)

### OTEL Span Emission Optimization
- **v1.5.0 Status**: Not optimized
- **Why Deferred**: Requires extensive benchmarking
- **Current Performance**: 356ms for 10K spans (acceptable)
- **Target**: < 300ms (16% improvement)
- **Revisited in**: v1.6.0 with dedicated performance team

### Tar Implementation Migration
- **v1.5.0 Status**: Documented, not implemented
- **Why Deferred**: Not applicable (no direct tokio-tar usage)
- **Dependency**: Transitive via testcontainers
- **Risk Level**: LOW (5 mitigation layers)
- **Revisited**: If security patch not available by v1.6.0

---

## Community & Contribution Roadmap

### Call for Contributions
We welcome community contributions in these areas:

1. **Chicago-TDD Integration**
   - Help implement once v1.2.0 released
   - Write integration examples
   - Create video tutorials

2. **Performance Optimization**
   - Benchmark reports from real-world usage
   - Flame graphs and profiling data
   - Container orchestration comparisons

3. **Documentation**
   - Translation to other languages
   - Case studies and tutorials
   - Architecture deep-dives

4. **Testing**
   - Fuzzing tests
   - Property-based testing
   - Chaos engineering scenarios

5. **Integrations**
   - CI/CD pipeline plugins
   - IDE extensions
   - Cloud platform adapters

### Contribution Process
1. Open discussion issue for major features
2. Create RFC (Request for Comments) for architecture changes
3. Submit PR with tests and documentation
4. Code review with maintainers
5. Merge and inclusion in next release

---

## Success Metrics & KPIs

### Performance Targets (v1.6.0)
| Metric | v1.5.0 | v1.6.0 Target | Success Criteria |
|--------|---------|---------------|-----------------|
| Container Acquisition | 0.25ms | 0.2ms | -20% latency |
| OTEL Spans (10K) | 356ms | <300ms | -16% regression |
| Memory (multi-image) | N/A | Baseline +5% | Efficient pooling |
| Multi-image Throughput | N/A | 1000+ tests/s | Linear scaling |

### Quality Targets (All Releases)
- ✅ 100% test coverage for new features
- ✅ Zero clippy warnings
- ✅ Zero production panics
- ✅ Backward compatibility
- ✅ CLAUDE.md compliance

### Adoption Targets
- v1.6.0: 500+ GitHub stars
- v1.7.0: 1000+ GitHub stars
- Enterprise: 10+ enterprise customers

---

## Release Coordination

### Release Process
1. **Planning** (Week 1): Feature freeze, prioritization
2. **Development** (Weeks 2-6): Feature implementation
3. **Testing** (Week 7): QA, integration testing
4. **Documentation** (Week 8): Release notes, guides
5. **Release** (Week 8): Tag, publish, announce

### Quality Gates (Before Release)
- [ ] All tests passing
- [ ] Zero clippy warnings
- [ ] Weaver validation passing
- [ ] Performance benchmarks within targets
- [ ] Documentation complete
- [ ] Migration guide (if breaking changes)

### Communication
- Blog post announcing release
- GitHub releases page with full changelog
- Community discussion thread
- Twitter/LinkedIn announcements

---

## Feedback & Discussion

Have opinions on the roadmap? Help us prioritize!

- **GitHub Issues**: Feature requests and discussions
- **GitHub Discussions**: General feedback and ideas
- **Email**: seanchatmangpt@gmail.com
- **Twitter**: @seanchatmangpt

---

## Version History

| Version | Release Date | Major Features | Status |
|---------|---|---|---|
| **1.5.0** | 2025-11-15 | Zero-copy RAII, Adaptive pooling, SBOM, Chicago-TDD framework | ✅ Released |
| **1.4.1** | 2025-10-30 | Container pooling, performance optimization | ✅ Released |
| **1.4.0** | 2025-10-15 | Lock-free concurrency, semaphore limiting | ✅ Released |
| **1.3.0** | 2025-09-01 | Weaver schema validation | ✅ Released |
| **1.2.0** | 2025-08-01 | OpenTelemetry integration | ✅ Released |

---

**Last Updated**: 2025-11-15
**Next Review**: 2025-11-22 (after v1.5.1 planning)
**Questions?** Open an issue on GitHub!
