# KCura Directory Structure Analysis

**Analysis Date:** 2025-10-17
**Target Directory:** /Users/sac/dev/kcura
**Analyst:** Directory Structure Analyst Agent

---

## Executive Summary

KCura is a **production-ready enterprise knowledge processing engine** implemented as a large-scale **Rust monorepo** with polyglot FFI support (Go, Python, Node.js). The project converts semantic web standards (OWL/SHACL/SPARQL) to optimized DuckDB SQL with cryptographic receipts and deterministic governance hooks.

### Key Metrics
- **Total Files:** 44,086
- **Total Directories:** 4,223
- **Total Size:** 19GB
- **Primary Language:** Rust (362 .rs files)
- **Documentation:** 128 Markdown files
- **Rust Crates:** 19 specialized crates
- **Secondary Languages:** Go (3 files), Python (7 files), JavaScript/TypeScript (16 files)

---

## 1. Repository Type & Structure

### Classification
**Type:** Cargo Workspace Monorepo with Polyglot FFI Bindings

### Organizational Pattern
```
kcura/
├── crates/              # 19 Rust crates (core workspace)
├── go/                  # Go bindings and smoke tests
├── python/              # Python FFI bindings
├── node-*/              # Node.js bindings (3 packages)
├── docs/                # Comprehensive documentation (128 files)
├── scripts/             # Build & CI automation scripts
├── examples/            # Usage examples & quick-start
├── bdd/                 # BDD test specifications (Cucumber)
├── benches/             # Performance benchmarks
├── target/              # Build artifacts (19GB)
└── [config files]       # Workspace configuration
```

---

## 2. Core Workspace Structure

### Root-Level Directories (24 total)

| Directory | Purpose | File Count | Key Contents |
|-----------|---------|------------|--------------|
| **crates/** | Rust workspace crates | ~360 .rs files | 19 specialized crates |
| **docs/** | Documentation | 128 files | Architecture, API refs, guides |
| **scripts/** | Automation & tooling | 43 files | Build, CI, optimization scripts |
| **examples/** | Usage examples | 14 files | Quick-start guides, sample data |
| **bdd/** | BDD test specifications | 32 .feature files | Cucumber/Gherkin tests |
| **benches/** | Performance benchmarks | 19 files | Criterion benchmarks |
| **go/** | Go FFI bindings | 3 .go files | Go package & smoke tests |
| **python/** | Python FFI bindings | 7 .py files | Python wrapper library |
| **node-kcura/** | Node.js native addon | Mixed | N-API bindings |
| **node-smoke/** | Node.js smoke tests | Test files | Integration validation |
| **node-e2e/** | Node.js E2E tests | Test files | End-to-end validation |
| **target/** | Build artifacts | ~43k files | Cargo build output (19GB) |
| **patches/** | Dependency patches | 1 patch | Arrow-RS quarter fix |
| **deploy/** | Deployment configs | Mixed | Production deployment |
| **fuzz/** | Fuzz testing | Mixed | cargo-fuzz targets |
| **templates/** | Code generation | Mixed | Template files |
| **prebuilt/** | Precompiled binaries | Mixed | DuckDB prebuilts |
| **.github/** | GitHub Actions | 2 workflows | CI/CD automation |
| **.cargo/** | Cargo config | Config files | Build configuration |
| **.git/** | Git repository | Git data | Version control |
| **.lh/** | LocalHistory backup | Mirror of source | IDE plugin backup |
| **src/** | Workspace root src | 1 file | Workspace-level code |
| **include/** | C header files | Header files | FFI C headers |
| **test-data-generator/** | Test data generation | Rust crate | Data generation tools |

---

## 3. Rust Crates Architecture (19 Crates)

### Crate Organization

The workspace is organized into **19 specialized crates** following domain-driven design:

```
crates/
├── kcura-core              # Main API coordination & public interface
├── kcura-catalog           # Schema mapping & DDL generation
├── kcura-compiler          # Core semantic compilation orchestration
├── kcura-compiler-sparql   # SPARQL → SQL compilation
├── kcura-compiler-shacl    # SHACL → SQL compilation
├── kcura-compiler-owl-rl   # OWL-RL → SQL compilation
├── kcura-owl-rl-incremental # Incremental OWL-RL materialization
├── kcura-engine            # Query execution engine
├── kcura-kernels           # SIMD/branchless hot-path kernels
├── kcura-hooks             # Deterministic governance hooks
├── kcura-receipts          # Cryptographic receipt generation
├── kcura-duck              # DuckDB integration & session mgmt
├── kcura-inmem             # In-memory store for agents/IoT
├── kcura-ffi               # C ABI for cross-language bindings
├── kcura-cli               # Command-line interface
├── kcura-tests             # Integration & property tests
├── kcura-swarm             # Swarm coordination (experimental)
├── kcura_temporal          # Temporal data handling
└── kcura_macros            # Procedural macros
```

### Crate Purposes

| Category | Crates | Purpose |
|----------|--------|---------|
| **Core API** | `kcura-core`, `kcura-catalog` | Public interface, schema management |
| **Compilation** | `kcura-compiler`, `-sparql`, `-shacl`, `-owl-rl`, `-owl-rl-incremental` | Semantic → SQL transformation |
| **Execution** | `kcura-engine`, `kcura-kernels`, `kcura-duck` | Query execution & optimization |
| **Governance** | `kcura-hooks`, `kcura-receipts` | Compliance & cryptographic proof |
| **Storage** | `kcura-inmem` | In-memory data structures |
| **Interop** | `kcura-ffi` | Foreign function interface |
| **Tooling** | `kcura-cli`, `kcura-tests`, `kcura-swarm` | CLI, testing, orchestration |
| **Utilities** | `kcura_temporal`, `kcura_macros` | Temporal logic, code generation |

---

## 4. File Type Distribution

### Source Code Files

| Extension | Count | Purpose | Location |
|-----------|-------|---------|----------|
| **.rs** | 362 | Rust source code | `crates/*/src/`, root |
| **.toml** | 40 | Configuration (Cargo, etc.) | Workspace root, crates |
| **.md** | 128 | Documentation | `docs/`, root |
| **.sh** | 45 | Shell scripts | `scripts/`, root |
| **.go** | 3 | Go FFI bindings | `go/pkg/`, `go/cmd/` |
| **.py** | 7 | Python FFI bindings | `python/kcura/` |
| **.js/.ts** | 16 | Node.js bindings | `node-*/` |
| **.yml/.yaml** | 13 | YAML configs | `.github/workflows/`, root |
| **.json** | 49 | JSON configs | Root, various |
| **.feature** | 32 | BDD test specs | `bdd/features/`, `docs/v1-requirements-bdd/` |
| **.c/.h** | Unknown | C FFI headers | `include/`, `examples/` |

### Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` (root) | Workspace configuration, 22 member crates |
| `Cargo.lock` | Dependency lock file (143KB) |
| `rust-toolchain.toml` | Rust version specification (1.76.0+) |
| `cbindgen.toml` | C binding generation config |
| `deny.toml` | cargo-deny license/security checks |
| `rustfmt.toml` | Code formatting rules |
| `.editorconfig` | Editor configuration |
| `.commitlintrc.json` | Commit message linting |
| `.pre-commit-config.yaml` | Pre-commit hook config |
| `mkdocs.yml` | Documentation site config |
| `requirements.txt` | Python dependencies |
| `go/go.mod` | Go module definition |
| `node-*/package.json` | Node.js package configs |

---

## 5. Package Manager Detection

### Primary: Cargo (Rust)

**Workspace Definition:** `/Users/sac/dev/kcura/Cargo.toml`

```toml
[workspace]
members = [
  "crates/kcura-core",
  "crates/kcura-catalog",
  # ... 19 total members ...
  "bdd",
  "benches",
]
resolver = "2"
```

**Key Features:**
- Workspace-level dependency management (38 shared dependencies)
- Strict linting rules (forbid `unwrap`, `expect`, `panic`)
- Production-ready standards enforcement
- Dependency patching for arrow-rs

### Secondary Package Managers

| Manager | File | Location | Purpose |
|---------|------|----------|---------|
| **Go Modules** | `go.mod` | `/Users/sac/dev/kcura/go/` | Go FFI bindings |
| **pip** | `requirements.txt` | Root | Python dependencies |
| **npm** | `package.json` | `node-kcura/`, `node-smoke/`, `node-e2e/` | Node.js packages |
| **Cargo** | Multiple `Cargo.toml` | `scripts/`, `benches/`, `bdd/`, `test-data-generator/` | Auxiliary crates |

---

## 6. Documentation Structure

### Documentation Organization (128 Files)

The `docs/` directory contains comprehensive technical documentation:

```
docs/
├── USER_GUIDE.md                    # End-user documentation
├── DEVELOPER_GUIDE.md               # Developer onboarding
├── API_REFERENCE.md                 # API documentation
├── COMPILER_V1.md                   # Compiler specification
├── HOOKS_RUNTIME_V1.md              # Hooks system spec
├── KERNEL_ROUTER_V1.md              # Performance optimization
├── ARCHITECTURE.md                  # System architecture
├── DEPLOYMENT.md                    # Deployment guide
├── CROSS_LANGUAGE_CLIENTS.md        # FFI integration guide
├── DUCK_INTEGRATION_GUIDE.md        # DuckDB integration
├── CORE_TEAM_*.md                   # Team standards (3 files)
├── D0_EXECUTION_SUMMARY.md          # Definition of Done
├── CURRENT_STATE_AUDIT.md           # State audit
├── architecture/                    # Architecture diagrams & specs
├── api-reference/                   # API reference docs
├── developer-guide/                 # Developer guides
├── data-contracts/                  # Error codes, schemas
├── v1-requirements-bdd/             # BDD requirements (7 .feature files)
├── diagrams/                        # Architecture diagrams
└── arrow/                           # Arrow-RS integration docs
```

### Documentation Categories

| Category | Files | Focus |
|----------|-------|-------|
| **User Documentation** | ~15 | Usage guides, examples, CLI help |
| **Developer Guides** | ~20 | API references, integration guides |
| **Architecture** | ~25 | System design, component specs |
| **Standards & Process** | ~10 | Core team standards, DoD, audits |
| **BDD Specifications** | 7 | Gherkin feature specs |
| **Technical Specs** | ~51 | Compiler, hooks, kernels, DuckDB |

---

## 7. Testing Infrastructure

### Test Directory Structure

| Location | Type | Count | Framework |
|----------|------|-------|-----------|
| `crates/*/src/*_test.rs` | Unit tests | Inline | Rust `#[test]` |
| `crates/*/tests/` | Integration tests | 21 files | Rust `#[test]` |
| `bdd/features/*.feature` | BDD specs | 25 files | Cucumber-rs |
| `docs/v1-requirements-bdd/*.feature` | Requirements BDD | 7 files | Documentation |
| `node-smoke/tests/` | Node.js smoke | Multiple | Jest/Mocha |
| `node-e2e/tests/` | Node.js E2E | Multiple | Jest/Mocha |
| `benches/*.rs` | Benchmarks | 19 files | Criterion |
| `fuzz/` | Fuzz tests | Mixed | cargo-fuzz |
| `crates/kcura-compiler-sparql/proptest-regressions/` | Property tests | Regressions | Proptest |

### BDD Feature Files (32 Total)

**Core Features (bdd/features/):**
- `duckdb_core.feature`, `duckdb_smoke.feature`, `duckdb_performance.feature`
- `sparql.feature`, `shacl.feature`, `owl_rl_materialize.feature`
- `hooks_guard.feature`, `hooks_on_commit.feature`, `hooks_threshold_timer.feature`
- `receipts.feature`, `telemetry.feature`, `tx_manager.feature`
- `catalog_crud.feature`, `convert_owl_catalog.feature`, `convert_hints.feature`
- `ffi_surface.feature`, `exec_sql.feature`, `attach_external.feature`
- `multitenancy.feature`, `time_travel.feature`, `router_route_selection.feature`

**Requirements BDD (docs/v1-requirements-bdd/):**
- `core_architecture.feature`, `duckdb_integration.feature`
- `hooks_runtime.feature`, `security_governance.feature`
- `ffi_clients_deployment.feature`, `acid_transactions.feature`
- `performance_slo.feature`

---

## 8. Cross-Language Integration

### Polyglot Architecture

KCura provides FFI bindings for multiple languages:

```
Language Bindings:
├── Rust (Native)       → crates/kcura-core (primary API)
├── C ABI               → crates/kcura-ffi (stable interface)
├── Go                  → go/pkg/kcura/ (production)
├── Python              → python/kcura/ (production)
└── Node.js             → node-kcura/ (production, N-API)
```

### Go Integration

**Location:** `go/`
```
go/
├── go.mod              # Go module definition
├── cmd/smoke/          # Go smoke tests
└── pkg/kcura/          # Go package wrapping C FFI
```

**Files:** 3 .go files

### Python Integration

**Location:** `python/`
```
python/
├── kcura/              # Python package
│   ├── __init__.py
│   ├── __pycache__/
│   └── [7 .py files total]
```

**Dependencies:** `requirements.txt` (Python pip)

### Node.js Integration

**Locations:** `node-kcura/`, `node-smoke/`, `node-e2e/`

| Package | Purpose | Files |
|---------|---------|-------|
| **node-kcura** | Native N-API addon | Cargo.toml + package.json (dual build) |
| **node-smoke** | Smoke tests | package.json + test files |
| **node-e2e** | End-to-end tests | package.json + test files |

**Technology:** Node.js N-API (native addons using Rust neon or napi-rs)

---

## 9. Build & CI/CD Infrastructure

### Build Scripts (scripts/ directory)

**Total Scripts:** 43 files

**Key Scripts:**
- `ci_gate.sh`, `ci_gates.sh` - CI/CD quality gates
- `build_duckdb_optimized.sh` - Optimized DuckDB build
- `build_frozen_duckdb.sh` - Frozen DuckDB build
- `build_static_duckdb.sh` - Static linking build
- `create_arrow_cache.sh` - Arrow-RS caching
- `create_precompiled_duckdb.sh` - Precompiled binaries
- `download_duckdb_binaries.sh` - Binary downloads
- `precompile_deps.sh` - Dependency precompilation
- `bench_gate.sh` - Performance benchmarking gate
- `docs_check.sh` - Documentation validation
- `check_workspace_deps.sh` - Dependency checking
- `generate-changelog.sh` - Changelog generation
- `open_gaps.sh` - Gap analysis
- `fake_guard.sh` - Test guard
- `ffi_constant_return_check.py` - FFI validation

### GitHub Actions Workflows

**Location:** `.github/workflows/`

| Workflow | Purpose |
|----------|---------|
| `ci.yml` | Continuous integration (build, test, lint) |
| `docs.yml` | Documentation building and validation |

### Pre-commit Hooks

**Configuration:** `.pre-commit-config.yaml`
- Code formatting checks
- Linting validation
- Commit message linting (`.commitlintrc.json`)

---

## 10. Development Tools & Configuration

### Code Quality Tools

| Tool | Configuration | Purpose |
|------|--------------|---------|
| **rustfmt** | `rustfmt.toml` | Code formatting |
| **clippy** | `Cargo.toml` lints section | 40+ strict lint rules |
| **cargo-deny** | `deny.toml` | License/security checks |
| **cbindgen** | `cbindgen.toml` | C header generation |
| **commitlint** | `.commitlintrc.json` | Commit message validation |
| **EditorConfig** | `.editorconfig` | Editor consistency |
| **pre-commit** | `.pre-commit-config.yaml` | Pre-commit validation |

### Strict Linting Rules (Cargo.toml)

```toml
[workspace.lints.clippy]
unwrap_used = "deny"          # No .unwrap()
expect_used = "deny"          # No .expect()
panic = "deny"                # No panic!()
unimplemented = "deny"        # No unimplemented!()
exit = "deny"                 # No std::process::exit()
```

**Total Lints Enforced:** 40+ rules (production-grade safety)

---

## 11. Experimental & Auxiliary Components

### Experimental Features

| Component | Location | Status |
|-----------|----------|--------|
| **kcura-swarm** | `crates/kcura-swarm/` | Experimental crate |
| **AI Features** | Various | Mentioned in docs |

### Auxiliary Crates

| Crate | Purpose |
|-------|---------|
| `benches/` | Performance benchmarking suite |
| `bdd/` | BDD test execution crate |
| `scripts/` | Build script utilities (has Cargo.toml) |
| `test-data-generator/` | Test data generation tool |

### Patches

**Location:** `patches/arrow-rs/arrow-arith/`
- **Purpose:** Fix for Arrow-RS quarter compilation issue
- **Applied via:** `[patch.crates-io]` in workspace Cargo.toml

---

## 12. Hidden Directories & Tooling

### Version Control

- **`.git/`** - Git repository data
- **`.github/`** - GitHub Actions workflows (2 files)

### Build Artifacts

- **`target/`** - Cargo build output (19GB, 43k+ files)
  - `target/release/` - Release builds
  - `target/debug/` - Debug builds
  - `target/duckdb-frozen/` - Frozen DuckDB build
  - `target/duckdb-prebuilt/` - Prebuilt DuckDB binaries

### IDE & Tools

- **`.cargo/`** - Cargo local configuration
- **`.lh/`** - LocalHistory backup (IDE plugin, mirrors source tree)
- **`.cursorrules`** - Cursor IDE rules (22KB file)
- **`.tool-versions`** - asdf version manager config

---

## 13. Key Architectural Patterns

### Monorepo Structure

**Pattern:** Cargo workspace with domain-driven crate boundaries

**Benefits:**
- Shared dependency management (38 workspace dependencies)
- Consistent versioning across crates
- Unified build and test infrastructure
- Strong compile-time guarantees across module boundaries

### Layered Architecture

```
┌─────────────────────────────────────────────┐
│  CLI / FFI (Go, Python, Node.js)            │  User Interface
├─────────────────────────────────────────────┤
│  kcura-core (Public API)                    │  API Layer
├─────────────────────────────────────────────┤
│  Compilers (SPARQL, SHACL, OWL-RL)          │  Compilation Layer
├─────────────────────────────────────────────┤
│  Engine + Kernels (Execution)               │  Execution Layer
├─────────────────────────────────────────────┤
│  DuckDB + Hooks + Receipts                  │  Runtime Layer
├─────────────────────────────────────────────┤
│  Catalog + Inmem (Storage)                  │  Storage Layer
└─────────────────────────────────────────────┘
```

### Test Strategy

**Multi-level Testing:**
1. **Unit Tests** - Inline with source code
2. **Integration Tests** - `crates/*/tests/`
3. **BDD Tests** - Cucumber feature files (32 specs)
4. **Property Tests** - Proptest for semantic correctness
5. **Benchmarks** - Criterion for performance validation
6. **Smoke Tests** - Cross-language FFI validation
7. **E2E Tests** - Full workflow validation

---

## 14. Performance & Optimization

### Performance-Critical Components

| Component | Optimization Strategy |
|-----------|----------------------|
| **kcura-kernels** | SIMD, branchless operations for hot paths |
| **kcura-duck** | DuckDB vectorized execution |
| **kcura-compiler** | Query plan optimization |
| **kcura-inmem** | In-memory data structures for low-latency |

### Build Optimizations

- **Precompiled DuckDB binaries** (`prebuilt/`)
- **Arrow-RS caching** (scripts)
- **Frozen dependency builds** (`target/duckdb-frozen/`)
- **Static linking options** (build scripts)

### Performance SLOs (from README)

- SPARQL 2-hop queries: p95 ≤ 25ms (100M rows)
- SHACL validation: p95 ≤ 60ms (10M constraints)
- Guard hooks: ≤ 20ms (1M affected rows)
- Kernel operations: ≥ 15k predicates/sec/core

---

## 15. Production Readiness Indicators

### Quality Gates

✅ **Comprehensive Testing**
- 100% unit test coverage (claimed)
- 32 BDD feature specifications
- Property-based testing
- Fuzz testing infrastructure
- Cross-language smoke tests

✅ **Documentation Excellence**
- 128 Markdown files
- Complete API reference
- Architecture diagrams
- User guides
- Developer onboarding
- Error code documentation

✅ **Code Quality**
- 40+ strict Clippy lints enforced
- Zero `unwrap()`/`expect()` policy
- Comprehensive error handling
- Production-grade patterns

✅ **CI/CD**
- GitHub Actions workflows
- Pre-commit hooks
- Automated quality gates
- Benchmark gates
- Documentation validation

✅ **Cross-Platform**
- Linux, macOS, Windows support
- Cross-language FFI (Go, Python, Node.js)
- Multiple deployment targets

### Version Status

**Current Version:** v1.0 (Production Ready)
- Minimum Rust Version: 1.76.0 (rust-toolchain.toml)
- Resolver: Cargo resolver v2

---

## 16. Security & Compliance

### Security Features

| Feature | Implementation |
|---------|---------------|
| **Cryptographic Receipts** | `kcura-receipts` crate |
| **Merkle Proofs** | Tamper-evident execution |
| **Deterministic Hooks** | `kcura-hooks` governance |
| **Secure Coding** | No `unsafe_code` (workspace lint) |

### Code Safety

```toml
[workspace.lints.rust]
unsafe_code = "forbid"        # No unsafe Rust allowed
```

**Exception:** Likely required in FFI crate (`kcura-ffi`) for C interop

---

## 17. Dependencies & External Libraries

### Major Dependencies (from Cargo.toml)

**Core:**
- `duckdb = "1.4.0"` - Embedded OLAP database
- `tokio = "1"` - Async runtime
- `serde = "1"` - Serialization
- `anyhow = "1"`, `thiserror = "1"` - Error handling

**Testing:**
- `cucumber = "0.20"` - BDD testing
- `criterion = "0.5"` - Benchmarking
- `proptest = "1"` - Property testing
- `insta = "1"` - Snapshot testing
- `testcontainers = "0.15"` - Integration testing

**Observability:**
- `opentelemetry = "0.25"` - Telemetry
- `tracing = "0.1"` - Structured logging
- `tracing-subscriber = "0.3"` - Log collection

**Cryptography:**
- `sha3 = "0.10"`, `blake3 = "1.0"` - Hashing
- `ed25519-dalek = "2.0"` - Signatures
- `merkle-lite = "0.1"` - Merkle trees

**Utilities:**
- `clap = "4.0"` - CLI parsing
- `regex = "1.0"` - Pattern matching
- `rayon = "1.0"` - Data parallelism
- `reqwest = "0.11"` - HTTP client
- `ollama-rs = "0.3.2"` - LLM integration

**Total Workspace Dependencies:** 38 shared dependencies

---

## 18. Examples & Quick Start

### Examples Directory

**Location:** `/Users/sac/dev/kcura/examples/`

**Contents:**
- `quick-start.sh` - Quick start script
- `person-ontology.ttl` - Sample OWL ontology
- `person-constraints.ttl` - Sample SHACL constraints
- `person-shapes.ttl` - Sample SHACL shapes
- `person-queries.sparql` - Sample SPARQL queries
- `person-data.ttl` - Sample RDF data
- `person-hints.json` - Mapping hints
- `hooks-examples.json` - Hook examples
- `ffi_smoke.c` - C FFI smoke test
- `sample-data.sql` - SQL sample data
- `README.md` - Examples documentation
- `test-suite.md` - Test suite documentation

**Purpose:** Hands-on learning and validation of core functionality

---

## 19. Deployment & Distribution

### Deployment Configuration

**Location:** `deploy/`
- Contains `.kcura/` directory (deployment-specific config)

### Prebuilt Binaries

**Location:** `prebuilt/`
- Precompiled binaries for faster builds
- DuckDB prebuilts

### Build Outputs

**Locations:**
- `target/release/` - Release binaries
- Build scripts generate optimized builds

### Installation Methods (from README)

1. **From Source:** `cargo build --release --package kcura-cli`
2. **Global Install:** `cargo install --path crates/kcura-cli`
3. **Binary Distribution:** Prebuilt binaries (implied by `prebuilt/`)

---

## 20. Organizational Insights

### Project Complexity Indicators

| Metric | Value | Assessment |
|--------|-------|------------|
| **Crate Count** | 19 | High modularity |
| **Total Files** | 44,086 | Large-scale project |
| **Documentation Files** | 128 | Excellent documentation |
| **Test Files** | 50+ | Comprehensive testing |
| **Languages** | 5+ (Rust, Go, Python, JS, Shell) | Polyglot ecosystem |
| **Directory Depth** | 6+ levels | Complex hierarchy |
| **Repository Size** | 19GB | Includes build artifacts |

### Development Maturity

✅ **Production-Ready Indicators:**
- Comprehensive documentation (128 files)
- Multi-language FFI support
- Strict code quality standards (40+ lints)
- BDD specifications (32 feature files)
- Performance benchmarking infrastructure
- CI/CD automation (GitHub Actions)
- Cryptographic security features
- Cross-platform support
- Versioned at v1.0

### Team & Process

**Evidence of Professional Development:**
- Core Team Standards documentation (3 files)
- Definition of Done (D0_EXECUTION_SUMMARY.md)
- Current State Audit (CURRENT_STATE_AUDIT.md)
- Code owners file (CODEOWNERS)
- Changelog tracking (CHANGELOG.md)
- Contribution guidelines (implied by CONTRIBUTION_PACKAGE.md)
- Community engagement (COMMUNITY_POST.md)

---

## 21. Dependency Graph & Crate Relationships

### Inferred Dependency Flow

```
kcura-cli
    └─→ kcura-core (public API)
            ├─→ kcura-compiler (orchestration)
            │       ├─→ kcura-compiler-sparql
            │       ├─→ kcura-compiler-shacl
            │       └─→ kcura-compiler-owl-rl
            │               └─→ kcura-owl-rl-incremental
            ├─→ kcura-catalog (schema)
            ├─→ kcura-engine (execution)
            │       ├─→ kcura-kernels (SIMD)
            │       └─→ kcura-duck (DuckDB)
            ├─→ kcura-hooks (governance)
            ├─→ kcura-receipts (crypto)
            ├─→ kcura-inmem (in-memory)
            └─→ kcura_temporal (temporal)

kcura-ffi (C bindings)
    └─→ kcura-core

kcura-tests
    └─→ All crates (testing)

kcura_macros (proc macros)
    └─→ Used by multiple crates
```

---

## 22. Gaps & Observations

### Notable Observations

1. **Large Build Artifacts:** 19GB in `target/` suggests frequent builds or multiple profiles
2. **LocalHistory Backup:** `.lh/` directory mirrors source (IDE plugin backup)
3. **Multiple Build Strategies:** Prebuilt, frozen, optimized DuckDB builds
4. **Arrow-RS Patch:** Workspace patches arrow-rs for quarter fix (technical debt)
5. **Experimental Swarm:** `kcura-swarm` crate suggests orchestration/distributed features
6. **No Python Tests Visible:** Python bindings lack obvious test infrastructure in tree
7. **Minimal Go Code:** Only 3 .go files suggests thin wrapper over C FFI
8. **Node.js Dual Build:** node-kcura has both Cargo.toml and package.json (N-API pattern)

### Potential Technical Debt

- **Arrow-RS Patch:** Dependency patch indicates upstream issue
- **Build Complexity:** Multiple DuckDB build strategies may indicate optimization challenges
- **Target Size:** 19GB build artifacts (consider cleanup automation)

### Strengths

✅ World-class documentation (128 files)
✅ Comprehensive testing (unit, integration, BDD, property, fuzz)
✅ Production-ready quality standards (40+ lints)
✅ Cross-language support (Rust, Go, Python, Node.js)
✅ Performance-focused (SLOs, benchmarks, SIMD kernels)
✅ Security-first (cryptographic receipts, no unsafe code)

---

## 23. Summary & Recommendations

### Project Classification

**Type:** Enterprise Knowledge Processing Engine
**Architecture:** Rust Monorepo with Polyglot FFI
**Maturity:** Production-Ready (v1.0)
**Scale:** Large (44k files, 19 crates, 19GB)
**Quality:** High (comprehensive docs, tests, standards)

### Key Characteristics

1. **Domain-Driven Design:** 19 specialized crates with clear boundaries
2. **Tables-First Philosophy:** Semantic web → SQL conversion
3. **Performance-Optimized:** SIMD kernels + DuckDB vectorization
4. **Production-Grade:** Strict linting, comprehensive testing, extensive docs
5. **Polyglot Ecosystem:** Native Rust + C FFI → Go/Python/Node.js
6. **Security-Focused:** Cryptographic receipts, deterministic governance
7. **Enterprise-Ready:** Compliance features, audit trails, documentation

### Recommendations for Analysis

1. **Code Analysis:** Focus on `crates/kcura-core/src/` for public API understanding
2. **Compilation Flow:** Study `kcura-compiler-*/` for OWL/SHACL/SPARQL conversion
3. **Performance:** Investigate `kcura-kernels/` for optimization patterns
4. **Integration:** Examine `kcura-ffi/` for C bindings and FFI patterns
5. **Testing:** Review `bdd/features/*.feature` for behavioral specifications
6. **Documentation:** Start with `docs/USER_GUIDE.md` and `docs/ARCHITECTURE.md`

### Next Steps for Swarm Analysis

1. **Crate Dependency Mapping:** Parse Cargo.toml files to build exact dependency graph
2. **API Surface Analysis:** Extract public APIs from each crate
3. **Test Coverage Analysis:** Quantify test coverage per crate
4. **Documentation Gap Analysis:** Identify undocumented components
5. **Performance Profiling:** Analyze benchmark results for bottlenecks
6. **Security Audit:** Review cryptographic implementations and FFI safety

---

## 24. Coordination Protocol Completion

### Memory Storage

**Key:** `swarm/kcura/structure`

**Value:**
```json
{
  "total_files": 44086,
  "total_directories": 4223,
  "total_size_gb": 19,
  "rust_files": 362,
  "crates": 19,
  "languages": ["Rust", "Go", "Python", "JavaScript", "Shell"],
  "documentation_files": 128,
  "test_files": 50,
  "monorepo_type": "Cargo Workspace",
  "version": "v1.0",
  "rust_version": "1.76.0+",
  "status": "Production-Ready"
}
```

**Analysis Complete:** 2025-10-17
**Analyst:** Directory Structure Analyst Agent
**Output Location:** `/Users/sac/clnrm/docs/implementation/kcura-structure-analysis.md`

---

**End of Analysis**
