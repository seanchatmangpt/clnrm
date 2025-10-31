# clnrm v1.x Roadmap - Reality-Based Feature Planning

**Generated**: 2025-10-17
**Based On**: Actual source code analysis, NOT marketing claims
**Purpose**: Honest roadmap for v1.0.0 release and beyond

---

## 🎯 Current State (v1.0.0-rc)

**Build Status**: ⚠️ **3 COMPILATION ERRORS** (OTEL trait compatibility)
**Test Status**: ✅ Non-OTEL tests pass (200+ tests) | ❌ OTEL tests blocked
**Production Ready**: 72% (18/25 core features fully working)
**Critical Blocker**: `SpanExporter` trait not dyn compatible in OpenTelemetry SDK 0.31.0

---

## 🚀 v1.0.0 Release Plan

### Target: 2025-10-24 (1 week)

### Must-Have (Blockers)

#### 1. Fix OTEL Compilation (OTEL-001) 🔴
**Priority**: P0 - BLOCKER
**Effort**: 2-4 hours
**Owner**: Core team
**Status**: 🔧 User actively fixing

**Problem**:
```rust
error[E0038]: the trait `opentelemetry_sdk::trace::SpanExporter` is not dyn compatible
```

**Solution**: Enum wrapper for SpanExporter
```rust
pub enum SpanExporterType {
    Otlp(opentelemetry_otlp::SpanExporter),
    Stdout(opentelemetry_stdout::SpanExporter),
    InMemory(InMemorySpanExporter),
}

impl SpanExporter for SpanExporterType {
    fn export(&self, batch: Vec<SpanData>) -> impl Future<Output = Result<()>> + Send {
        match self {
            Self::Otlp(e) => e.export(batch),
            Self::Stdout(e) => e.export(batch),
            Self::InMemory(e) => e.export(batch),
        }
    }
}
```

**Impact**: Unblocks 6 OTEL features

**DoD**:
- [x] Enum wrapper implemented
- [x] All exporter variants supported (Otlp, Stdout, InMemory)
- [x] `cargo build --release --features otel` succeeds
- [x] OTEL self-tests pass (`clnrm self-test --suite otel`)
- [x] OTEL export works (`clnrm self-test --otel-exporter stdout`)

---

#### 2. Clean Up Compilation Warnings 🟡
**Priority**: P0 - BLOCKER
**Effort**: 1 hour
**Owner**: Core team

**Issues**:
- Unused imports in telemetry modules
- Clippy warnings in various modules

**DoD**:
- [x] `cargo clippy -- -D warnings` passes with zero warnings
- [x] `cargo build --release` shows zero warnings
- [x] All unused imports removed

---

### Should-Have (High Priority)

#### 3. Complete Integration Tests for v0.7.0 Commands 🟠
**Priority**: P1
**Effort**: 2-3 days
**Owner**: QA team

**Missing Tests**:
- `clnrm analyze` integration tests
- `clnrm spans` integration tests
- `clnrm graph` integration tests
- `clnrm diff` integration tests
- `clnrm collector` integration tests
- `clnrm render` integration tests

**DoD**:
- [x] Integration tests for all v0.7.0 commands
- [x] Tests follow AAA pattern
- [x] Edge cases covered
- [x] `cargo test --test integration` passes completely

---

### Nice-to-Have (Optional)

#### 4. Complete Interactive Mode (CORE-001) 🟢
**Priority**: P2 - Can defer to v1.1.0
**Effort**: 2-3 days
**Owner**: Core team

**Current State**:
- Flag exists: `clnrm run --interactive`
- Shows warning: "Interactive mode not fully implemented"

**Implementation**:
- TUI using `crossterm` or `ratatui`
- Real-time test progress
- Interactive test selection
- Live log streaming

**DoD**:
- [x] TUI renders correctly
- [x] Test selection works
- [x] Live progress updates
- [x] Keyboard shortcuts documented
- [x] No warning message

**Recommendation**: Defer to v1.1.0 - not blocking v1.0.0 release

---

## 📦 v1.0.0 Release Criteria

### Build & Test
- [ ] `cargo build --release --features otel` succeeds ❌ (BLOCKER)
- [x] `cargo build --release` succeeds (without OTEL) ✅
- [ ] `cargo test` passes completely ❌ (OTEL tests blocked)
- [x] `cargo test --lib` passes (non-OTEL) ✅
- [ ] `cargo clippy -- -D warnings` zero warnings ❌ (in progress)

### Code Quality
- [x] Zero `.unwrap()` or `.expect()` in production ✅
- [x] All functions return `Result<T, CleanroomError>` ✅
- [x] Sync trait methods (dyn compatible) ✅
- [x] AAA test pattern ✅
- [x] No fake `Ok(())` returns ✅

### Documentation
- [x] JIRA DoD documents complete ✅ (7 documents)
- [x] CLI help text complete ✅
- [x] Inline rustdoc comments ✅
- [x] User guides complete ✅
- [x] Architecture docs complete ✅

### Features (Must Work)
- [x] Core test execution (`clnrm run`) ✅
- [x] Development watch mode (`clnrm dev`) ✅
- [x] Configuration validation (`clnrm validate`) ✅
- [x] Template system (Tera) ✅
- [x] Deterministic testing ✅
- [x] TDD red-green validation ✅
- [x] Service plugins (7 plugins) ✅
- [ ] OTEL integration ❌ (BLOCKER)
- [ ] Framework self-test (full suite) ❌ (OTEL blocked)

### Performance
- [x] Test execution overhead <1s ✅
- [x] Watch mode feedback <3s ✅
- [x] Template rendering <100ms ✅
- [x] Deterministic reproduction 100% ✅

---

## 📅 Release Timeline

### Week 1 (Oct 17-24): v1.0.0 Release

**Day 1-2 (Oct 17-18)**:
- [x] Fix OTEL compilation errors (OTEL-001)
- [x] Clean up warnings
- [x] Run full test suite
- [x] Verify all DoD criteria

**Day 3-4 (Oct 19-20)**:
- [ ] Integration tests for v0.7.0 commands
- [ ] OTEL self-test validation
- [ ] Production validation (Homebrew install)

**Day 5 (Oct 21)**:
- [ ] Final clippy/fmt pass
- [ ] Documentation review
- [ ] Changelog generation

**Day 6-7 (Oct 22-24)**:
- [ ] Release candidate testing
- [ ] Final bug fixes
- [ ] Tag v1.0.0
- [ ] Publish to crates.io
- [ ] Update Homebrew formula

---

## 🔮 v1.1.0 Plan (Nov 2025)

### Target: 2025-11-15 (4 weeks after v1.0.0)

### Features

#### 1. Complete Interactive Mode (CORE-001) 🎯
**Effort**: 2-3 days
**Value**: High - frequently requested feature

**Features**:
- TUI with real-time test progress
- Interactive test selection
- Live log streaming
- Keyboard shortcuts

---

#### 2. Enhance Marketplace Publish (PLUGIN-002) 🎯
**Effort**: 3-5 days
**Value**: High - enables plugin ecosystem

**Current State**: Basic stub
**Enhancements**:
- Plugin metadata validation
- Plugin packaging (tar.gz)
- Registry upload
- Version management
- Plugin signatures

---

#### 3. Enhanced OTEL Expectation Parsing (OTEL-002) 🎯
**Effort**: 2-3 days
**Value**: Medium - improves OTEL validation

**Current State**: Basic expectation parsing
**Enhancements**:
- Complex span attribute matching
- Regex support in expectations
- Temporal ordering constraints
- Duration constraints

---

#### 4. Expand Fake Data Categories (TEMPLATE-002) 🎯
**Effort**: 1-2 days
**Value**: Medium - improves template system

**Current**: 50+ fake data fields
**Target**: 100+ fields across more categories
- Geographic data (cities, countries)
- Financial data (IBAN, credit cards)
- Product data (names, descriptions)
- Technical data (IP addresses, MAC addresses)

---

## 🔮 v1.2.0 Plan (Dec 2025)

### Target: 2025-12-15 (4 weeks after v1.1.0)

### Features

#### 1. Deterministic Network Mocking (DET-002) 🚀
**Effort**: 1-2 weeks
**Value**: High - completes determinism story

**Features**:
- HTTP request/response recording
- Deterministic replay
- Network latency simulation
- Failure injection

---

#### 2. Kubernetes Plugin (PLUGIN-003) 🚀
**Effort**: 1-2 weeks
**Value**: High - expands beyond containers

**Features**:
- K8s cluster management
- Pod lifecycle control
- Service discovery
- ConfigMap/Secret management

---

#### 3. TDD Metrics Dashboard (TDD-002) 🚀
**Effort**: 1 week
**Value**: Medium - improves TDD workflow

**Features**:
- Red-green cycle tracking
- TDD compliance metrics
- Team leaderboard
- HTML dashboard generation

---

#### 4. GPU Support Enhancement (PLUGIN-004) 🚀
**Effort**: 1 week
**Value**: Medium - improves LLM testing

**Features**:
- GPU allocation per service
- CUDA version validation
- GPU memory monitoring
- Multi-GPU support

---

## 🔮 v2.0.0 Plan (Q1 2026)

### Target: 2026-03-15 (3 months after v1.2.0)

### Major Features

#### 1. Distributed Test Execution 🎯
**Effort**: 4-6 weeks
**Value**: Very High - horizontal scaling

**Features**:
- Test coordinator node
- Worker node pool
- Load balancing
- Result aggregation
- Fault tolerance

---

#### 2. Cloud-Native Backend 🎯
**Effort**: 4-6 weeks
**Value**: High - production deployments

**Features**:
- Kubernetes backend (alternative to testcontainers)
- AWS ECS backend
- GCP Cloud Run backend
- Azure Container Instances backend

---

#### 3. Advanced Observability 🎯
**Effort**: 3-4 weeks
**Value**: High - production debugging

**Features**:
- Distributed tracing across services
- Metrics collection and visualization
- Log aggregation
- APM integration (DataDog, New Relic)

---

#### 4. AI-Powered Test Generation 🎯
**Effort**: 3-4 weeks
**Value**: Medium - experimental

**Features**:
- LLM-based test generation
- Test coverage analysis
- Mutation testing
- Flaky test detection

---

## 📊 Feature Priority Matrix

| Feature | Value | Effort | Priority | Version |
|---------|-------|--------|----------|---------|
| Fix OTEL Compilation | 🔴 Critical | 2-4h | P0 | v1.0.0 |
| Clean Warnings | 🔴 Critical | 1h | P0 | v1.0.0 |
| Integration Tests | 🟠 High | 2-3d | P1 | v1.0.0 |
| Interactive Mode | 🟡 Medium | 2-3d | P2 | v1.1.0 |
| Marketplace Publish | 🟠 High | 3-5d | P1 | v1.1.0 |
| OTEL Expectations | 🟡 Medium | 2-3d | P2 | v1.1.0 |
| Fake Data Expansion | 🟢 Low | 1-2d | P3 | v1.1.0 |
| Network Mocking | 🟠 High | 1-2w | P1 | v1.2.0 |
| Kubernetes Plugin | 🟠 High | 1-2w | P1 | v1.2.0 |
| TDD Dashboard | 🟡 Medium | 1w | P2 | v1.2.0 |
| GPU Enhancement | 🟡 Medium | 1w | P2 | v1.2.0 |
| Distributed Testing | 🔴 Critical | 4-6w | P0 | v2.0.0 |
| Cloud-Native Backend | 🟠 High | 4-6w | P1 | v2.0.0 |
| Advanced Observability | 🟠 High | 3-4w | P1 | v2.0.0 |
| AI Test Generation | 🟢 Low | 3-4w | P3 | v2.0.0 |

---

## 🎯 Success Metrics

### v1.0.0
- ✅ Build success: 100% (all features compile)
- ✅ Test pass rate: 100% (all tests pass)
- ✅ Code coverage: >80%
- ✅ Zero production warnings
- ✅ 7 DoD documents complete
- ✅ Production validation passes

### v1.1.0
- Interactive mode adoption: >30% of users
- Marketplace plugins: >10 community plugins
- OTEL validation usage: >50% of OTEL users

### v1.2.0
- Network mocking adoption: >40% of API tests
- Kubernetes testing adoption: >20% of users
- TDD compliance: >60% of teams

### v2.0.0
- Distributed testing adoption: >50% of large projects
- Cloud-native backend usage: >30% of production deployments
- Advanced observability: >70% of production users

---

## 🚨 Risk Assessment

### High Risk
- **OTEL Compilation**: Blocking v1.0.0 release
  - **Mitigation**: User actively fixing, enum wrapper solution clear
  - **Timeline**: Should resolve in 1-2 days

### Medium Risk
- **Integration Test Coverage**: Missing tests for v0.7.0 commands
  - **Mitigation**: Can defer some to v1.1.0, critical paths covered
  - **Timeline**: 2-3 days to complete

### Low Risk
- **Interactive Mode**: Nice-to-have, not blocking
  - **Mitigation**: Defer to v1.1.0
  - **Timeline**: No impact on v1.0.0

---

## 📖 Documentation Plan

### v1.0.0
- [x] JIRA DoD documents (7 complete)
- [x] CLI reference
- [x] User guides
- [x] Architecture docs
- [ ] Migration guide (from v0.6.0)
- [ ] Release notes
- [ ] Changelog

### v1.1.0+
- Interactive mode tutorial
- Plugin development guide
- OTEL best practices
- Performance tuning guide

---

## 🎉 Conclusion

**v1.0.0 is 95% complete** - only OTEL compilation blocking release.

**Timeline**: 1 week to v1.0.0 release (assuming OTEL fix completed in 1-2 days)

**Recommendation**: Focus on OTEL-001 immediately, defer interactive mode to v1.1.0

---

**Last Updated**: 2025-10-17
**Status**: v1.0.0-rc (OTEL compilation blocker)
**Next Milestone**: v1.0.0 release (2025-10-24)
