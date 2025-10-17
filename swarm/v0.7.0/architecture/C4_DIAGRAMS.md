# C4 Architecture Diagrams - v0.7.0 DX Layer

This document provides comprehensive C4 model diagrams for the v0.7.0 developer experience enhancements.

## Level 1: System Context Diagram

Shows the big picture of the system and its users.

```
                    ┌─────────────────────────────┐
                    │     Test Developer          │
                    │  • Writes .clnrm.tera       │
                    │  • Expects <3s feedback     │
                    │  • Validates locally        │
                    └──────────────┬──────────────┘
                                   │
                      Edits templates, runs tests
                                   │
                                   ▼
┌────────────────────────────────────────────────────────────┐
│                                                             │
│              clnrm Testing Framework                        │
│                    (v0.7.0)                                │
│                                                             │
│  • Hot reload (<3s dev loop)                               │
│  • Fast validation (dry-run)                               │
│  • Parallel execution                                      │
│  • Span diff visualization                                 │
│                                                             │
└──────────┬────────────────────────────┬────────────────────┘
           │                            │
           │ Manages containers         │ Exports telemetry
           │                            │
           ▼                            ▼
┌────────────────────┐      ┌────────────────────────┐
│ Container Runtime  │      │ Observability Backend  │
│                    │      │                        │
│ • Docker           │      │ • Jaeger               │
│ • Podman           │      │ • DataDog              │
│ • containerd       │      │ • New Relic            │
│                    │      │                        │
└────────────────────┘      └────────────────────────┘
```

---

## Level 2: Container Diagram

Shows the high-level shape of the software architecture and how responsibilities are distributed.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        clnrm Testing Framework                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │                    DX Layer (v0.7.0 NEW)                        │    │
│  ├────────────────────────────────────────────────────────────────┤    │
│  │                                                                 │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │    │
│  │  │    Watch     │  │   Validate   │  │     Diff     │         │    │
│  │  │   Service    │─▶│   Service    │─▶│   Engine     │         │    │
│  │  │              │  │              │  │              │         │    │
│  │  │ • notify-rs  │  │ • Dry-run    │  │ • Tree diff  │         │    │
│  │  │ • Debounce   │  │ • No docker  │  │ • Semantic   │         │    │
│  │  │ • Hash cache │  │ • <700ms     │  │ • <1s        │         │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘         │    │
│  │          │                 │                   │               │    │
│  └──────────┼─────────────────┼───────────────────┼───────────────┘    │
│             │                 │                   │                     │
│             └─────────────────┼───────────────────┘                     │
│                               ▼                                         │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │                  Core Engine (v0.6.0 UNCHANGED)                 │    │
│  ├────────────────────────────────────────────────────────────────┤    │
│  │                                                                 │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │    │
│  │  │  Template    │  │    Config    │  │   Service    │         │    │
│  │  │  Renderer    │─▶│   Parser     │─▶│   Manager    │         │    │
│  │  │              │  │              │  │              │         │    │
│  │  │ • Tera       │  │ • TOML       │  │ • Plugins    │         │    │
│  │  │ • Functions  │  │ • Validate   │  │ • Lifecycle  │         │    │
│  │  └──────────────┘  └──────────────┘  └──────┬───────┘         │    │
│  │                                              │                  │    │
│  │                                              ▼                  │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │    │
│  │  │   Scenario   │  │   Backend    │  │  Validation  │         │    │
│  │  │   Executor   │◀─│   (Docker)   │─▶│  Orchestra-  │         │    │
│  │  │              │  │              │  │    tor       │         │    │
│  │  │ • Steps      │  │ • Containers │  │ • Spans      │         │    │
│  │  │ • Parallel   │  │ • Volumes    │  │ • Graphs     │         │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘         │    │
│  │                                                                 │    │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │                    Reporting Layer                              │    │
│  ├────────────────────────────────────────────────────────────────┤    │
│  │                                                                 │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │    │
│  │  │    JSON      │  │    JUnit     │  │   Digest     │         │    │
│  │  │   Reporter   │  │   Reporter   │  │  (SHA-256)   │         │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘         │    │
│  │                                                                 │    │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Key Containers**:
1. **DX Layer**: New v0.7.0 components for developer productivity
2. **Core Engine**: Existing v0.6.0 functionality (unchanged)
3. **Reporting Layer**: Output generation (JSON, JUnit, Digest)

---

## Level 3: Component Diagram - DX Layer

Detailed view of the DX Layer components.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DX Layer Components                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │                    Watch Service                            │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │                                                             │    │
│  │  ┌──────────────────┐         ┌───────────────────┐        │    │
│  │  │  FileWatcher     │         │  ChangeDetector   │        │    │
│  │  │                  │────────▶│                   │        │    │
│  │  │ • notify::Watcher│  Event  │ • SHA-256 hash    │        │    │
│  │  │ • Glob patterns  │         │ • Hash cache      │        │    │
│  │  │ • Debouncer      │         │ • Comparison      │        │    │
│  │  └──────────────────┘         └─────────┬─────────┘        │    │
│  │          │                               │                  │    │
│  │          │ File event                    │ Changed?         │    │
│  │          │                               │                  │    │
│  │          └───────────────┬───────────────┘                  │    │
│  │                          ▼                                  │    │
│  │  ┌──────────────────────────────────────────┐              │    │
│  │  │         EventQueue                       │              │    │
│  │  │                                          │              │    │
│  │  │ • mpsc::channel                          │              │    │
│  │  │ • Batching (50ms window)                 │              │    │
│  │  │ • Deduplication                          │              │    │
│  │  └──────────────────┬───────────────────────┘              │    │
│  │                     │                                       │    │
│  └─────────────────────┼───────────────────────────────────────┘    │
│                        │ WatchEvent                                 │
│                        ▼                                            │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │                  Validate Service                           │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │                                                             │    │
│  │  ┌──────────────────┐         ┌───────────────────┐        │    │
│  │  │  RenderCache     │         │ DryRunValidator   │        │    │
│  │  │                  │────────▶│                   │        │    │
│  │  │ • LRU cache      │  Miss   │ • TemplateRenderer│        │    │
│  │  │ • Key: hash      │         │ • ConfigParser    │        │    │
│  │  │ • Max: 100       │◀────────│ • ConfigValidator │        │    │
│  │  └──────────────────┘  Store  └─────────┬─────────┘        │    │
│  │          │                               │                  │    │
│  │          │ Cache hit                     │ Result           │    │
│  │          │                               │                  │    │
│  │          └───────────────┬───────────────┘                  │    │
│  │                          ▼                                  │    │
│  │  ┌──────────────────────────────────────────┐              │    │
│  │  │      ValidationResult                    │              │    │
│  │  │                                          │              │    │
│  │  │ • Valid: bool                            │              │    │
│  │  │ • Errors: Vec<ValidationError>           │              │    │
│  │  │ • Warnings: Vec<String>                  │              │    │
│  │  │ • Duration: u64                          │              │    │
│  │  └──────────────────┬───────────────────────┘              │    │
│  │                     │                                       │    │
│  └─────────────────────┼───────────────────────────────────────┘    │
│                        │ ValidationResult                           │
│                        ▼                                            │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │                     Diff Engine                             │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │                                                             │    │
│  │  ┌──────────────────┐         ┌───────────────────┐        │    │
│  │  │ExpectationTree   │         │  DiffAlgorithm    │        │    │
│  │  │     Builder      │────────▶│                   │        │    │
│  │  │                  │  Trees  │ • Set difference  │        │    │
│  │  │ • from_config()  │         │ • HashMap diff    │        │    │
│  │  │ • Graph edges    │         │ • Custom compare  │        │    │
│  │  │ • Counts         │         │                   │        │    │
│  │  │ • Windows        │         │                   │        │    │
│  │  └──────────────────┘         └─────────┬─────────┘        │    │
│  │          │                               │                  │    │
│  │          │ Previous tree                 │ Diff             │    │
│  │          │                               │                  │    │
│  │          └───────────────┬───────────────┘                  │    │
│  │                          ▼                                  │    │
│  │  ┌──────────────────────────────────────────┐              │    │
│  │  │        ExpectationDiff                   │              │    │
│  │  │                                          │              │    │
│  │  │ • Added: Vec<DiffItem>                   │              │    │
│  │  │ • Removed: Vec<DiffItem>                 │              │    │
│  │  │ • Modified: Vec<(DiffItem, DiffItem)>    │              │    │
│  │  └──────────────────┬───────────────────────┘              │    │
│  │                     │                                       │    │
│  └─────────────────────┼───────────────────────────────────────┘    │
│                        │ ExpectationDiff                            │
│                        ▼                                            │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │                  Output Formatter                           │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │                                                             │    │
│  │  • Terminal colors                                          │    │
│  │  • Diff symbols (+, -, ~)                                   │    │
│  │  • Tree visualization                                       │    │
│  │  • Timing breakdown                                         │    │
│  │                                                             │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

**Component Responsibilities**:

| Component | Responsibility | Performance Target |
|-----------|---------------|-------------------|
| **FileWatcher** | Detect file system changes | <50ms |
| **ChangeDetector** | Hash-based change detection | <10ms |
| **EventQueue** | Batch and deduplicate events | <50ms (window) |
| **RenderCache** | Cache rendered TOML | <1ms (hit) |
| **DryRunValidator** | Validate without containers | <700ms |
| **ExpectationTree** | Build diffable tree | <100ms |
| **DiffAlgorithm** | Compute tree differences | <1s |
| **OutputFormatter** | Human-readable output | <100ms |

---

## Level 4: Code Diagram - FileWatcher

Detailed implementation view of the FileWatcher component.

```
┌─────────────────────────────────────────────────────────────┐
│                      FileWatcher                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Fields:                                                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │ watcher: RecommendedWatcher                        │     │
│  │ debounce_ms: u64                                   │     │
│  │ hash_cache: HashMap<PathBuf, String>               │     │
│  │ tx: mpsc::Sender<WatchEvent>                       │     │
│  │ patterns: Vec<glob::Pattern>                       │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  Methods:                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │ pub fn new(patterns: Vec<String>)                  │     │
│  │     -> Result<Self>                                │     │
│  │                                                     │     │
│  │ • Create notify::Watcher                           │     │
│  │ • Compile glob patterns                            │     │
│  │ • Initialize hash cache                            │     │
│  │ • Create mpsc channel                              │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │ pub fn start(&mut self)                            │     │
│  │     -> Result<mpsc::Receiver<WatchEvent>>          │     │
│  │                                                     │     │
│  │ • Start watcher.watch()                            │     │
│  │ • Spawn event processing task                      │     │
│  │ • Return receiver channel                          │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │ fn is_changed(&mut self, path: &Path)              │     │
│  │     -> Result<bool>                                │     │
│  │                                                     │     │
│  │ • Compute current hash: SHA-256                    │     │
│  │ • Lookup cached hash                               │     │
│  │ • Compare hashes                                   │     │
│  │ • Update cache if changed                          │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │ fn compute_hash(path: &Path)                       │     │
│  │     -> Result<String>                              │     │
│  │                                                     │     │
│  │ • Read file content                                │     │
│  │ • Compute SHA-256 digest                           │     │
│  │ • Return hex string                                │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │ fn matches_pattern(&self, path: &Path)             │     │
│  │     -> bool                                        │     │
│  │                                                     │     │
│  │ • Check if path matches any pattern                │     │
│  │ • Support globs: *.clnrm.tera, tests/**/*.tera     │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │ async fn process_events(                           │     │
│  │     rx: notify::Receiver,                          │     │
│  │     tx: mpsc::Sender<WatchEvent>                   │     │
│  │ )                                                  │     │
│  │                                                     │     │
│  │ • Receive notify events                            │     │
│  │ • Debounce (batch 50ms window)                     │     │
│  │ • Filter by pattern                                │     │
│  │ • Check hash change                                │     │
│  │ • Send WatchEvent                                  │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Data Structures**:

```rust
pub enum WatchEvent {
    Changed { path: PathBuf, hash: String },
    Deleted { path: PathBuf },
    Error { message: String },
}

struct DebouncedEvent {
    path: PathBuf,
    kind: EventKind,
    timestamp: Instant,
}

struct HashCache {
    entries: HashMap<PathBuf, CacheEntry>,
    max_size: usize,
}

struct CacheEntry {
    hash: String,
    modified_at: SystemTime,
}
```

---

## Sequence Diagrams

### Hot Reload Sequence

```
Developer    FileWatcher    ChangeDetector    RenderCache    DryRunValidator    Output
    │             │                │                │                │             │
    │  Save       │                │                │                │             │
    │────────────▶│                │                │                │             │
    │             │                │                │                │             │
    │             │ File event     │                │                │             │
    │             │───────────────▶│                │                │             │
    │             │                │                │                │             │
    │             │                │ Hash check     │                │             │
    │             │                │───────────────▶│                │             │
    │             │                │                │                │             │
    │             │                │                │ Cache lookup   │             │
    │             │                │                │───────────────▶│             │
    │             │                │                │                │             │
    │             │                │                │    Cache miss  │             │
    │             │                │                │◀───────────────│             │
    │             │                │                │                │             │
    │             │                │                │   Render+Parse │             │
    │             │                │                │◀───────────────│             │
    │             │                │                │                │             │
    │             │                │                │   Validate     │             │
    │             │                │                │◀───────────────│             │
    │             │                │                │                │             │
    │             │                │                │  ValidationResult            │
    │             │                │                │─────────────────────────────▶│
    │             │                │                │                │             │
    │◀────────────────────────────────────────────────────────────────────────────│
    │  Feedback   │                │                │                │             │
    │  (<3s)      │                │                │                │             │
```

### Parallel Execution Sequence

```
CLI        Executor      Scheduler    Resource     Container    Validator
 │            │             │         Manager         │            │
 │  run       │             │            │            │            │
 │───────────▶│             │            │            │            │
 │            │             │            │            │            │
 │            │ Plan        │            │            │            │
 │            │────────────▶│            │            │            │
 │            │             │            │            │            │
 │            │             │ Check      │            │            │
 │            │             │───────────▶│            │            │
 │            │             │            │            │            │
 │            │             │ Available  │            │            │
 │            │             │◀───────────│            │            │
 │            │             │            │            │            │
 │            │  ExecutionPlan           │            │            │
 │            │◀────────────│            │            │            │
 │            │             │            │            │            │
 │            │ Execute     │            │            │            │
 │            │────────────▶│            │            │            │
 │            │             │            │            │            │
 │            │             ├─ Task 1 ──────────────▶│            │
 │            │             ├─ Task 2 ──────────────▶│            │
 │            │             ├─ Task 3 ──────────────▶│            │
 │            │             │            │            │            │
 │            │             │            │   Results  │            │
 │            │             │◀──────────────────────────          │
 │            │             │            │            │            │
 │            │             │            │  Validate (parallel)    │
 │            │             │────────────────────────────────────▶│
 │            │             │            │            │            │
 │            │  Results    │            │            │            │
 │            │◀────────────│            │            │            │
 │            │             │            │            │            │
 │  Results   │             │            │            │            │
 │◀───────────│             │            │            │            │
```

---

## Deployment Diagram

Shows how the software maps to infrastructure.

```
┌─────────────────────────────────────────────────────────────┐
│                  Developer Machine                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │           clnrm CLI (binary)                       │     │
│  │  • Installed via cargo install                     │     │
│  │  • Config: ~/.config/cleanroom/                    │     │
│  │  • Cache: ~/.cache/cleanroom/                      │     │
│  └────────────────┬───────────────────────────────────┘     │
│                   │                                          │
│                   │ Uses                                     │
│                   ▼                                          │
│  ┌────────────────────────────────────────────────────┐     │
│  │           Docker/Podman                            │     │
│  │  • Local daemon                                    │     │
│  │  • Container runtime                               │     │
│  │  • Network isolation                               │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
└──────────────────────┬───────────────────────────────────────┘
                       │
                       │ Exports telemetry (optional)
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Observability Stack                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐  │
│  │    Jaeger      │  │    DataDog     │  │  New Relic   │  │
│  │  (localhost)   │  │    (cloud)     │  │   (cloud)    │  │
│  │  Port: 14268   │  │                │  │              │  │
│  └────────────────┘  └────────────────┘  └──────────────┘  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**CI/CD Deployment**:

```
┌─────────────────────────────────────────────────────────────┐
│                    CI/CD Pipeline                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │        GitHub Actions / GitLab CI                  │     │
│  │                                                     │     │
│  │  jobs:                                              │     │
│  │    test:                                            │     │
│  │      - cargo install clnrm                          │     │
│  │      - clnrm run tests/ --parallel --jobs 4         │     │
│  │      - clnrm report --format junit                  │     │
│  │                                                     │     │
│  └────────────────┬───────────────────────────────────┘     │
│                   │                                          │
│                   │ Runs in                                  │
│                   ▼                                          │
│  ┌────────────────────────────────────────────────────┐     │
│  │         Docker-in-Docker Container                 │     │
│  │  • Pre-installed Docker                            │     │
│  │  • Isolated network                                │     │
│  │  • Cleanup on exit                                 │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| **Language** | Rust | 1.70+ | Core implementation |
| **Async Runtime** | Tokio | 1.x | Parallel execution |
| **File Watching** | notify | 6.x | File system events |
| **Templating** | Tera | 1.x | Template rendering |
| **Config Parsing** | toml | 0.8.x | TOML parsing |
| **Hashing** | sha2 | 0.10.x | Content hashing |
| **Caching** | lru | 0.12.x | LRU cache |
| **Container Runtime** | testcontainers-rs | 0.15.x | Container management |
| **CLI** | clap | 4.x | Command-line parsing |
| **Serialization** | serde | 1.x | JSON/TOML serialization |
| **Observability** | opentelemetry | 0.21.x | Tracing/metrics |

---

## Performance Budget

### Latency Breakdown (Hot Reload Path)

```
Total Budget: <3s (p95)

├─ File change detection      <100ms (3.3%)
│  ├─ notify event             <50ms
│  └─ Hash computation          <50ms
│
├─ Cache lookup               <1ms (0.03%)
│
├─ Template rendering         <500ms (16.7%)
│  ├─ Tera parsing             <200ms
│  ├─ Function evaluation      <200ms
│  └─ String interpolation     <100ms
│
├─ TOML parsing               <200ms (6.7%)
│  ├─ Lexing/parsing           <100ms
│  └─ AST construction         <100ms
│
├─ Config validation          <200ms (6.7%)
│  ├─ Field validation         <100ms
│  └─ Semantic checks          <100ms
│
├─ Expectation diff (opt)     <1s (33.3%)
│  ├─ Tree construction        <200ms
│  ├─ Set operations           <300ms
│  └─ Formatting               <500ms
│
└─ Output formatting          <100ms (3.3%)
   ├─ Color codes              <20ms
   ├─ Tree drawing             <50ms
   └─ Terminal I/O             <30ms

Remaining slack: ~900ms (30%)
```

### Memory Budget

```
Total Budget: <100MB

├─ File watcher                <5MB
│  ├─ Hash cache (100 files)   <1MB
│  └─ Event queue              <4MB
│
├─ Render cache                <50MB
│  ├─ LRU entries (100)        <40MB
│  └─ Metadata                 <10MB
│
├─ Validator state             <20MB
│  ├─ AST cache                <15MB
│  └─ Working memory           <5MB
│
├─ Diff engine                 <10MB
│  ├─ Expectation trees        <8MB
│  └─ Diff results             <2MB
│
└─ Output buffers              <5MB

Remaining slack: ~10MB (10%)
```

---

## Scalability Targets

| Metric | Target | Methodology |
|--------|--------|-------------|
| **File count** | 1000+ templates | Glob filtering, selective watching |
| **Concurrent scenarios** | 100+ | Resource limits, semaphores |
| **Cache size** | 100 entries | LRU eviction |
| **Memory per scenario** | <10MB | Efficient data structures |
| **CPU per scenario** | <1 core | Tokio work-stealing scheduler |

---

## References

- [C4 Model Documentation](https://c4model.com/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [notify crate](https://docs.rs/notify/)
- [Tokio documentation](https://tokio.rs/)
- [LRU cache implementation](https://docs.rs/lru/)
