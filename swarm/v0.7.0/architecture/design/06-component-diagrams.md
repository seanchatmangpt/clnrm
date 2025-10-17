# Component Interaction Diagrams - v0.7.0

## System Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                        v0.7.0 DX System                            │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ File Watcher │───▶│Change Detector│───▶│ Dry-Run     │       │
│  │              │    │              │    │ Validator   │       │
│  └──────────────┘    └──────────────┘    └──────┬───────┘       │
│                                                   │               │
│                                                   ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ Diff Engine  │◀───│   Parallel   │◀───│  Template   │       │
│  │              │    │   Executor   │    │  Renderer   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Component: File Watcher

### Internal Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    WatcherService                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐                                      │
│  │ notify::Watcher  │  Cross-platform file system events  │
│  │ (RecommendedWatcher)                                    │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ EventDebouncer   │  300ms debounce window              │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ Pattern Filter   │  **/*.clnrm.toml.tera               │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ EventProcessor   │  Render → Validate → Execute        │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           └──────────▶ mpsc::channel to Executor           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Event Flow

```
File System Change (editor save)
    │
    ▼
notify::Event { kind: Modify, paths: [...] }
    │
    ▼
EventDebouncer::should_process()
    │
    ├─▶ [YES] Continue processing
    │
    └─▶ [NO]  Skip (within debounce window)
    │
    ▼
Pattern Filter (matches *.clnrm.toml.tera?)
    │
    ├─▶ [YES] Create WatchEvent::Modified
    │
    └─▶ [NO]  Ignore event
    │
    ▼
EventProcessor::process(event)
```

## Component: Change Detector

### Internal Structure

```
┌─────────────────────────────────────────────────────────────┐
│                  ChangeDetector                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐                                      │
│  │   HashCache      │  SHA-256 hashes of rendered TOML    │
│  │ (.clnrm/cache/   │                                      │
│  │  hashes.json)    │                                      │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ VariableTracker  │  Context hash (vars, matrix)        │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │  Comparator      │  Detect changes                     │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           └──────────▶ ChangeResult                        │
│                        { should_execute: bool }            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Decision Flow

```
Template + Context
    │
    ▼
Render Template → TOML String
    │
    ▼
Compute SHA-256(rendered_toml)
    │
    ▼
Lookup in HashCache
    │
    ├─▶ [MISS] No cached hash → EXECUTE
    │
    └─▶ [HIT]  Compare hashes
                │
                ├─▶ [MATCH]    Check variable context
                │               │
                │               ├─▶ [UNCHANGED] SKIP (cache hit)
                │               │
                │               └─▶ [CHANGED]   EXECUTE (vars changed)
                │
                └─▶ [DIFFER]   EXECUTE (template changed)
```

## Component: Dry-Run Validator

### Internal Structure

```
┌─────────────────────────────────────────────────────────────┐
│                  DryRunValidator                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐                                      │
│  │ Tera Engine      │  Template rendering + syntax check  │
│  │ (cached)         │                                      │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ TOML Parser      │  Parse rendered TOML                │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ SchemaValidator  │  Validate v0.6.0 structure          │
│  │  - Required      │   • [meta]                          │
│  │    sections      │   • [otel]                          │
│  │  - Field types   │   • [service.*]                     │
│  │  - Value ranges  │   • [[scenario]]                    │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ RelationshipValidator                                   │
│  │  - Service refs  │  Validate scenario references       │
│  │  - OTEL expects  │  service.db in [[scenario]]         │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           └──────────▶ ValidationResult                    │
│                        { errors: [...], passed: bool }     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Validation Pipeline

```
Template File
    │
    ▼
Tera Rendering
    │
    ├─▶ [ERROR] Syntax error with line/column
    │
    └─▶ [SUCCESS] Rendered TOML
                   │
                   ▼
              TOML Parsing
                   │
                   ├─▶ [ERROR] Parse error with line/column
                   │
                   └─▶ [SUCCESS] Parsed TestConfig
                                  │
                                  ▼
                            Schema Validation
                                  │
                                  ├─▶ Missing [meta]
                                  ├─▶ Missing [otel]
                                  ├─▶ Invalid exporter value
                                  ├─▶ sample_ratio out of range
                                  └─▶ ...
                                  │
                                  ▼
                            Relationship Validation
                                  │
                                  ├─▶ Undefined service reference
                                  ├─▶ Invalid span expectations
                                  └─▶ ...
                                  │
                                  ▼
                            ValidationResult
```

## Component: Parallel Executor

### Internal Structure

```
┌─────────────────────────────────────────────────────────────┐
│                  ParallelExecutor                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐                                      │
│  │ PriorityQueue    │  Scenarios sorted by priority       │
│  │ (BinaryHeap)     │  High → Normal → Low                │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────────────────────────────────┐         │
│  │           WorkerPool                         │         │
│  │  ┌────────┐ ┌────────┐ ┌────────┐          │         │
│  │  │Worker 1│ │Worker 2│ │Worker N│ ...      │         │
│  │  └───┬────┘ └───┬────┘ └───┬────┘          │         │
│  │      │          │          │                │         │
│  │      ▼          ▼          ▼                │         │
│  │  Container  Container  Container            │         │
│  │  (Limited)  (Limited)  (Limited)            │         │
│  └──────────────────────────────────────────────┘         │
│                     │                                      │
│                     ▼                                      │
│  ┌──────────────────────────────────────────────┐         │
│  │        ResourceManager                       │         │
│  │  - Max 10 concurrent containers              │         │
│  │  - Memory: 512MB per container               │         │
│  │  - CPU: 1.0 cores per container              │         │
│  └──────────────────────────────────────────────┘         │
│                     │                                      │
│                     ▼                                      │
│  ┌──────────────────────────────────────────────┐         │
│  │        SpanCollector (OTEL)                  │         │
│  │  Concurrent span collection from all workers │         │
│  └──────────────────────────────────────────────┘         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Execution Flow

```
Scenario Tasks
    │
    ▼
Priority Queue (sorted)
    │
    ▼
Worker Pool Dispatcher
    │
    ├─────┬─────┬─────┐
    │     │     │     │
    ▼     ▼     ▼     ▼
Worker Worker Worker Worker
    │     │     │     │
    │     │     │     └─▶ Resource Limit Check
    │     │     │         (memory, CPU, container count)
    │     │     │              │
    │     │     │              ├─▶ [BLOCKED] Wait for slot
    │     │     │              │
    │     │     │              └─▶ [ALLOWED] Acquire ContainerSlot
    │     │     │                              │
    │     │     │                              ▼
    │     │     │                        Execute Scenario
    │     │     │                              │
    │     │     └──────────────────────────────┼─▶ Collect Spans (OTEL)
    │     │                                    │
    │     └────────────────────────────────────┼─▶ Execution Result
    │                                          │
    └──────────────────────────────────────────┘
                                               │
                                               ▼
                                         Result Aggregation
```

## Component: Diff Engine

### Internal Structure

```
┌─────────────────────────────────────────────────────────────┐
│                     DiffEngine                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐                                      │
│  │ BaselineLoader   │  Load .clnrm/trace.json             │
│  │ (with cache)     │                                      │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ SpanTreeBuilder  │  Build hierarchical tree            │
│  │                  │  Root → Children → Grandchildren    │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ TraceComparator  │  Compare baseline vs current        │
│  │  - Missing spans │  Find differences                   │
│  │  - Extra spans   │                                      │
│  │  - Attribute Δ   │                                      │
│  │  - Structure Δ   │                                      │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                      │
│  │ DiffVisualizer   │  ASCII tree with symbols            │
│  │  ✓ Match         │  ✓ ✏️  ➕ ❌                         │
│  │  ✏️  Modified     │                                      │
│  │  ➕ Added         │                                      │
│  │  ❌ Missing       │                                      │
│  └────────┬─────────┘                                      │
│           │                                                 │
│           └──────────▶ Formatted Diff Output               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Comparison Algorithm

```
Baseline Trace + Current Trace
    │
    ▼
Build Span Maps (name → span)
    │
    ├─▶ Baseline Map: {span1, span2, span3}
    │
    └─▶ Current Map:  {span1, span2, span4}
    │
    ▼
For each span in Baseline:
    │
    ├─▶ In Current?
    │   │
    │   ├─▶ [NO]  Add to missing_spans
    │   │
    │   └─▶ [YES] Compare attributes + structure
    │              │
    │              ├─▶ [DIFFER] Add to modified_spans
    │              │
    │              └─▶ [MATCH]  No action
    │
For each span in Current:
    │
    └─▶ In Baseline?
        │
        ├─▶ [NO]  Add to extra_spans
        │
        └─▶ [YES] Already processed above
    │
    ▼
Generate DiffResult
    │
    ├─▶ missing_spans: [span3]
    ├─▶ extra_spans:  [span4]
    ├─▶ modified_spans: []
    └─▶ match_rate: 66.7%
```

## Integration Flow

### Full Pipeline (Watch Mode)

```
1. File Change Detected
    │
    ▼
2. WatcherService::process_event()
    │
    ├─▶ Debounce (300ms)
    │
    └─▶ Pattern match (*.clnrm.toml.tera)
    │
    ▼
3. TemplateRenderer::render_file()
    │
    └─▶ Rendered TOML string
    │
    ▼
4. ChangeDetector::detect_changes()
    │
    ├─▶ [SKIP] No changes → Exit
    │
    └─▶ [EXECUTE] Changes detected
                   │
                   ▼
5. DryRunValidator::validate()
    │
    ├─▶ [FAIL] Validation errors → Report & Exit
    │
    └─▶ [PASS] Valid configuration
                │
                ▼
6. ParallelExecutor::execute()
    │
    ├─▶ Queue scenarios (priority)
    │
    ├─▶ Distribute to workers
    │
    ├─▶ Apply resource limits
    │
    └─▶ Collect OTEL spans
                │
                ▼
7. DiffEngine::compare()
    │
    ├─▶ Load baseline trace
    │
    ├─▶ Build span trees
    │
    ├─▶ Compare structures
    │
    └─▶ Visualize diffs
                │
                ▼
8. Report Results
    │
    └─▶ Console + JSON + JUnit
```

## Performance Considerations

### Bottleneck Analysis

```
┌─────────────────────┬──────────────┬─────────────────┐
│ Component           │ Operation    │ Optimization    │
├─────────────────────┼──────────────┼─────────────────┤
│ File Watcher        │ Event storm  │ Debouncing      │
│ Template Rendering  │ Tera compile │ Template cache  │
│ Hash Computation    │ Large files  │ Incremental     │
│ TOML Parsing        │ Complex TOML │ Streaming       │
│ Container Start     │ Image pull   │ Pooling/Reuse   │
│ Span Collection     │ Concurrency  │ Lock-free queue │
│ Tree Building       │ Deep nesting │ Depth limit     │
│ Diff Generation     │ Large traces │ Lazy eval       │
└─────────────────────┴──────────────┴─────────────────┘
```

### Concurrency Model

```
Main Thread (CLI)
    │
    ├─▶ Watcher Thread (notify)
    │   └─▶ Event Processing (async tasks)
    │
    ├─▶ Worker Pool Threads (N workers)
    │   ├─▶ Worker 1: Execute Scenario
    │   ├─▶ Worker 2: Execute Scenario
    │   └─▶ Worker N: Execute Scenario
    │
    └─▶ Span Collector Thread (OTEL)
        └─▶ Aggregate spans from all workers
```

## Data Persistence

```
.clnrm/
├── cache/
│   ├── hashes.json          (HashCache)
│   └── templates/           (Tera compiled templates)
├── trace.json               (Baseline trace)
├── trace-current.json       (Latest execution)
└── diff-history/
    ├── 2025-10-16-14-30.json
    └── 2025-10-16-15-00.json
```
