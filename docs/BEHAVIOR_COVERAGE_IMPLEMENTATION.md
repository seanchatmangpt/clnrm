# Behavior Coverage Implementation Summary

## Overview

**Behavior Coverage** is a comprehensive heuristic system for measuring what percentage of a system's behaviors are actually validated by clnrm tests. Unlike code coverage which measures which lines of code were executed, behavior coverage measures which **observable system behaviors** have been validated.

## Implementation Status

✅ **COMPLETE** - Core behavior coverage system implemented and ready for use

## Architecture

### 1. Core Types (`src/coverage/mod.rs`)

**`BehaviorCoverage`**: Tracks what has been tested
```rust
pub struct BehaviorCoverage {
    pub api_endpoints_covered: HashSet<String>,
    pub state_transitions_covered: HashSet<StateTransition>,
    pub error_scenarios_covered: HashSet<String>,
    pub data_flows_covered: HashSet<String>,
    pub integrations_covered: HashMap<String, HashSet<String>>,
    pub spans_observed: HashSet<String>,
}
```

**`BehaviorCoverageReport`**: Coverage analysis results
```rust
pub struct BehaviorCoverageReport {
    pub total_coverage: f64,  // 0.0 to 100.0
    pub dimensions: Vec<DimensionCoverage>,
    pub uncovered_behaviors: UncoveredBehaviors,
    pub total_behaviors: usize,
    pub covered_behaviors: usize,
}
```

**`StateTransition`**: State machine transitions
```rust
pub struct StateTransition {
    pub entity: String,              // e.g., "Container", "Test"
    pub from_state: Option<String>,  // None for creation
    pub to_state: String,
}
```

### 2. Behavior Manifest (`src/coverage/manifest.rs`)

**`BehaviorManifest`**: Complete inventory of system behaviors
- Loaded from `behaviors.clnrm.toml`
- Defines all expected behaviors across 6 dimensions
- Configurable weights per dimension
- Calculates coverage by comparing manifest to tracked coverage

**Manifest Structure**:
```toml
[system]
name = "clnrm"
version = "1.0.1"

[weights]  # Optional custom weights (must sum to 1.0)
api_surface = 0.20
state_transitions = 0.20
error_scenarios = 0.15
data_flows = 0.20
integrations = 0.15
span_coverage = 0.10

[dimensions.api_surface]
endpoints = ["clnrm run", "clnrm init", ...]

[[dimensions.state_transitions.entities]]
name = "Container"
states = ["created", "running", "stopped"]
[[dimensions.state_transitions.entities.transitions]]
from = "created"
to = "running"
trigger = "start_command"

[[dimensions.error_scenarios.scenarios]]
name = "missing_docker"
code = 1
description = "Docker is not available"

[[dimensions.data_flows.flows]]
name = "simple_test_execution"
steps = ["parse_toml", "create_container", "execute_command"]

[[dimensions.integrations.services]]
name = "docker"
operations = ["pull_image", "create_container", "start_container"]

[dimensions.span_coverage]
expected_spans = ["clnrm.run", "clnrm.test", ...]
```

### 3. Coverage Tracker (`src/coverage/tracker.rs`)

**`CoverageTracker`**: Thread-safe coverage tracking for CleanroomEnvironment
```rust
let tracker = CoverageTracker::new();

// Record behaviors as they're tested
tracker.record_api("clnrm run").await;
tracker.record_transition("Container", None, "running").await;
tracker.record_error("missing_docker").await;
tracker.record_flow("simple_test_execution").await;
tracker.record_integration("docker", "create_container").await;
tracker.record_span("clnrm.run").await;

// Get coverage snapshot
let coverage = tracker.snapshot().await;
```

### 4. Report Generator (`src/coverage/report.rs`)

**`ReportGenerator`**: Multi-format report generation
- **Text**: Console-friendly output with ASCII tables
- **JSON**: Machine-readable format
- **HTML**: Interactive web report with CSS styling
- **Markdown**: Documentation-friendly format

```rust
let report = manifest.calculate_coverage(&coverage)?;

// Generate in various formats
let text = ReportGenerator::generate(&report, ReportFormat::Text)?;
let html = ReportGenerator::generate(&report, ReportFormat::Html)?;
let json = ReportGenerator::generate(&report, ReportFormat::Json)?;

// Save to file
ReportGenerator::save(&report, "coverage.html", ReportFormat::Html)?;
```

## Coverage Dimensions

### 1. API Surface (20% default weight)

**What it measures**: What percentage of CLI commands have been tested?

**Example**:
- Total commands: 35 (clnrm run, clnrm init, clnrm validate, etc.)
- Tested: 20
- Coverage: 20/35 = 57.1%

### 2. State Transitions (20% default weight)

**What it measures**: What percentage of state machine transitions have been validated?

**Example**:
- Container states: created → starting → running → stopping → stopped
- Tested transitions: created → starting, starting → running
- Coverage: 2/5 = 40%

### 3. Error Scenarios (15% default weight)

**What it measures**: What percentage of error conditions are tested?

**Example**:
- Total error scenarios: 10 (missing_docker, invalid_toml, etc.)
- Tested: 3
- Coverage: 3/10 = 30%

### 4. Data Flows (20% default weight)

**What it measures**: What percentage of end-to-end workflows are validated?

**Example**:
- Total flows: 7 (simple_test_execution, service_lifecycle, etc.)
- Tested: 4
- Coverage: 4/7 = 57.1%

### 5. Integration Points (15% default weight)

**What it measures**: What percentage of external integrations are tested?

**Example**:
- Docker operations: 6 total, 4 tested = 66.7%
- Redis operations: 4 total, 0 tested = 0%
- Overall: 4/10 = 40%

### 6. Span Coverage (10% default weight)

**What it measures**: What percentage of expected OTEL spans are observed?

**Example**:
- Expected spans: 18
- Observed: 12
- Coverage: 12/18 = 66.7%

## Composite Coverage Calculation

```
Total Coverage = Σ (dimension_coverage × dimension_weight)

Example:
API Surface:       57.1% × 0.20 = 11.42%
State Transitions: 40.0% × 0.20 =  8.00%
Error Scenarios:   30.0% × 0.15 =  4.50%
Data Flows:        57.1% × 0.20 = 11.42%
Integrations:      40.0% × 0.15 =  6.00%
Span Coverage:     66.7% × 0.10 =  6.67%
                              ──────────
Total Coverage:              48.01%
```

## Usage Example

```rust
use clnrm_core::{BehaviorManifest, CoverageTracker, ReportGenerator, ReportFormat};

// 1. Load behavior manifest
let manifest = BehaviorManifest::load("behaviors.clnrm.toml")?;

// 2. Create coverage tracker
let tracker = CoverageTracker::new();

// 3. Run tests and record coverage
tracker.record_api("clnrm run").await;
tracker.record_transition("Test", Some("pending"), "running").await;
tracker.record_flow("simple_test_execution").await;

// 4. Generate coverage report
let coverage = tracker.snapshot().await;
let report = manifest.calculate_coverage(&coverage)?;

// 5. Display results
println!("{}", report.format_text());

// 6. Save HTML report
ReportGenerator::save(&report, "coverage.html", ReportFormat::Html)?;
```

## Integration with CleanroomEnvironment

Future enhancement to automatically track coverage:

```rust
// In CleanroomEnvironment
pub struct CleanroomEnvironment {
    // ... existing fields ...
    coverage_tracker: Option<Arc<CoverageTracker>>,
}

impl CleanroomEnvironment {
    pub async fn enable_coverage_tracking(&mut self) {
        self.coverage_tracker = Some(Arc::new(CoverageTracker::new()));
    }

    pub async fn execute_in_container(&self, ...) -> Result<ExecutionResult> {
        // Automatically record span coverage
        if let Some(tracker) = &self.coverage_tracker {
            tracker.record_span("clnrm.container.exec").await;
        }

        // ... existing implementation ...
    }
}
```

## CLI Commands (Future Work)

```bash
# Initialize behavior manifest
clnrm coverage init

# Analyze current coverage
clnrm coverage analyze --manifest behaviors.clnrm.toml

# Generate coverage report
clnrm coverage report --format html --output coverage.html

# Show uncovered behaviors
clnrm coverage gaps

# Validate manifest completeness
clnrm coverage validate-manifest

# Compare coverage between versions
clnrm coverage diff --baseline v1.0.0 --current v1.1.0
```

## Report Output Examples

### Text Format

```
Behavior Coverage Report
========================

Overall Coverage: 48.0% 🟠 (Grade: F)

Dimension Breakdown:
┌─────────────────────┬──────────┬─────────┬──────────┐
│ Dimension           │ Coverage │ Weight  │ Score    │
├─────────────────────┼──────────┼─────────┼──────────┤
│ API Surface         │ 57.1%    │ 20%     │ 11.42%   │
│ State Transitions   │ 40.0%    │ 20%     │  8.00%   │
│ Error Scenarios     │ 30.0%    │ 15%     │  4.50%   │
│ Data Flows          │ 57.1%    │ 20%     │ 11.42%   │
│ Integration Points  │ 40.0%    │ 15%     │  6.00%   │
│ Span Coverage       │ 66.7%    │ 10%     │  6.67%   │
└─────────────────────┴──────────┴─────────┴──────────┘

Top Uncovered Behaviors:
1. otel_tracing (Data Flow)
2. Container: starting → failed (State Transition)
3. clnrm self-test --suite otel (API Surface)
4. otel_export_failure (Error Scenario)
5. redis.get (Integration)
```

### HTML Format

Interactive report with:
- Grade badge (A-F) with color coding
- Progress bars for each dimension
- Expandable uncovered behaviors list
- Responsive design for mobile viewing

### JSON Format

```json
{
  "total_coverage": 48.01,
  "dimensions": [
    {
      "name": "API Surface",
      "coverage": 0.571,
      "weight": 0.20,
      "weighted_score": 0.1142,
      "total": 35,
      "covered": 20
    }
  ],
  "uncovered_behaviors": {
    "api_endpoints": ["clnrm self-test --suite otel"],
    "state_transitions": [...],
    "error_scenarios": [...],
    "data_flows": [...],
    "integrations": {...},
    "missing_spans": [...]
  },
  "total_behaviors": 150,
  "covered_behaviors": 72
}
```

## Advanced Features

### 1. Production Span Discovery

Auto-discover behaviors from production OTEL data:

```bash
# Import production spans to generate manifest
clnrm coverage discover --source jaeger --endpoint http://jaeger:16686
```

### 2. Criticality Marking

Mark critical behaviors that MUST be tested:

```toml
[behavior.critical]
must_cover = [
  { type = "flow", name = "otel_tracing", reason = "core feature" },
  { type = "api", name = "clnrm run", reason = "primary command" }
]
```

### 3. Complexity Weighting

Weight complex behaviors higher:

```toml
[[dimensions.data_flows.flows]]
name = "parallel_execution"
weight = 3.0  # 3x more important than simple flows
```

### 4. Trend Tracking

Monitor coverage over time:

```bash
# Track coverage history
clnrm coverage track --version 1.0.1

# Show coverage trend
clnrm coverage trend --from 1.0.0 --to 1.0.1
```

## Files Created

1. **`crates/clnrm-core/src/coverage/mod.rs`** (554 lines)
   - Core types: BehaviorCoverage, BehaviorCoverageReport, StateTransition
   - Dimension weights and uncovered behaviors tracking

2. **`crates/clnrm-core/src/coverage/manifest.rs`** (370 lines)
   - BehaviorManifest for TOML parsing
   - Coverage calculation logic
   - Dimensions and entity definitions

3. **`crates/clnrm-core/src/coverage/tracker.rs`** (113 lines)
   - Thread-safe CoverageTracker
   - Async recording methods

4. **`crates/clnrm-core/src/coverage/report.rs`** (240 lines)
   - Multi-format report generation (Text, JSON, HTML, Markdown)
   - File output support

5. **`examples/behaviors.clnrm.toml`** (247 lines)
   - Complete behavior manifest for clnrm framework
   - Demonstrates all 6 dimensions

6. **`docs/BEHAVIOR_COVERAGE_DESIGN.md`** (480 lines)
   - Comprehensive design documentation
   - Heuristics and formulas

7. **Updated `crates/clnrm-core/src/lib.rs`**
   - Added coverage module exports

## Benefits

1. **Objective Quality Metric**: Go beyond "lines covered" to "behaviors validated"
2. **Gap Identification**: Precisely identify untested behaviors
3. **Prioritization**: Weight critical behaviors higher
4. **Compliance**: Prove regulatory requirements are tested
5. **OTEL-Native**: Leverages existing observability infrastructure
6. **Trend Tracking**: Monitor coverage improvements over time

## Next Steps

To fully integrate behavior coverage into clnrm:

1. **CLI Commands**: Add `clnrm coverage` subcommands
2. **Auto-tracking**: Integrate CoverageTracker into CleanroomEnvironment
3. **CI Integration**: Fail CI if coverage drops below threshold
4. **Test Metadata**: Add `[test.covers]` section to .clnrm.toml files
5. **Production Discovery**: Implement Jaeger/DataDog span import
6. **Dashboard**: Create real-time coverage dashboard

## Conclusion

The behavior coverage system provides a comprehensive, heuristic-based approach to measuring test quality. By tracking 6 orthogonal dimensions of system behavior, clnrm can now answer the question: **"What percentage of our system's behaviors are actually validated?"**

This goes far beyond code coverage to measure the true quality and completeness of the test suite.
