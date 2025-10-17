# Behavior Coverage - Quick Start Guide

## What is Behavior Coverage?

**Behavior Coverage** answers the question: **"What percentage of my system's behaviors are actually validated by tests?"**

Unlike code coverage (which measures lines executed), behavior coverage measures **observable system behaviors** across 6 dimensions:

1. **API Surface**: CLI commands tested
2. **State Transitions**: State machine transitions validated
3. **Error Scenarios**: Error conditions tested
4. **Data Flows**: End-to-end workflows validated
5. **Integration Points**: External dependencies tested
6. **Span Coverage**: OTEL spans observed

## Quick Start in 3 Steps

### Step 1: Create a Behavior Manifest

Create `behaviors.clnrm.toml` defining your system's complete behavior inventory:

```toml
[system]
name = "my-api"
version = "1.0.0"

[dimensions.api_surface]
endpoints = [
    "clnrm run",
    "clnrm init",
    "clnrm validate"
]

[[dimensions.state_transitions.entities]]
name = "Test"
states = ["pending", "running", "passed", "failed"]
[[dimensions.state_transitions.entities.transitions]]
from = "pending"
to = "running"

[[dimensions.error_scenarios.scenarios]]
name = "missing_docker"
code = 1

[[dimensions.data_flows.flows]]
name = "simple_test"
steps = ["parse", "execute", "validate"]

[[dimensions.integrations.services]]
name = "docker"
operations = ["create_container", "exec"]

[dimensions.span_coverage]
expected_spans = ["clnrm.run", "clnrm.test"]
```

### Step 2: Track Coverage in Your Tests

```rust
use clnrm_core::{CoverageTracker, BehaviorManifest};

#[tokio::test]
async fn test_simple_execution() -> Result<()> {
    // Create tracker
    let tracker = CoverageTracker::new();

    // Record what behaviors you're testing
    tracker.record_api("clnrm run").await;
    tracker.record_transition("Test", Some("pending"), "running").await;
    tracker.record_flow("simple_test").await;
    tracker.record_integration("docker", "create_container").await;
    tracker.record_span("clnrm.run").await;

    // ... your actual test code ...

    Ok(())
}
```

### Step 3: Generate Coverage Report

```rust
// Load manifest
let manifest = BehaviorManifest::load("behaviors.clnrm.toml")?;

// Get tracked coverage
let coverage = tracker.snapshot().await;

// Calculate and display report
let report = manifest.calculate_coverage(&coverage)?;
println!("{}", report.format_text());

// Or save as HTML
use clnrm_core::{ReportGenerator, ReportFormat};
ReportGenerator::save(&report, "coverage.html", ReportFormat::Html)?;
```

## Sample Output

```
Behavior Coverage Report
========================

Overall Coverage: 60.0% 🟡 (Grade: D)

Dimension Breakdown:
┌─────────────────────┬──────────┬─────────┬──────────┐
│ Dimension           │ Coverage │ Weight  │ Score    │
├─────────────────────┼──────────┼─────────┼──────────┤
│ API Surface         │ 33.3%    │ 20%     │  6.67%   │
│ State Transitions   │ 100.0%   │ 20%     │ 20.00%   │
│ Error Scenarios     │ 0.0%     │ 15%     │  0.00%   │
│ Data Flows          │ 100.0%   │ 20%     │ 20.00%   │
│ Integration Points  │ 50.0%    │ 15%     │  7.50%   │
│ Span Coverage       │ 50.0%    │ 10%     │  5.00%   │
└─────────────────────┴──────────┴─────────┴──────────┘

Top Uncovered Behaviors:
1. clnrm init (API Surface)
2. clnrm validate (API Surface)
3. missing_docker (Error Scenario)
4. docker.exec (Integration)
5. clnrm.test (Span)
```

## Understanding the Scores

### Coverage Grading

- **A (90-100%)**: 🟢 Excellent coverage
- **B (80-89%)**: 🟢 Good coverage
- **C (70-79%)**: 🟡 Acceptable coverage
- **D (60-69%)**: 🟡 Needs improvement
- **F (<60%)**: 🔴 Insufficient coverage

### Dimension Weights (Customizable)

Default weights (sum to 1.0):
- API Surface: 20%
- State Transitions: 20%
- Error Scenarios: 15%
- Data Flows: 20%
- Integration Points: 15%
- Span Coverage: 10%

You can customize weights in your manifest:

```toml
[weights]
api_surface = 0.25        # More important
state_transitions = 0.25
error_scenarios = 0.20    # Increase error testing
data_flows = 0.15
integrations = 0.10       # Less critical
span_coverage = 0.05
```

## Real-World Example: clnrm Self-Testing

See `examples/behaviors.clnrm.toml` for a complete manifest defining 150+ behaviors across:

- **35 CLI commands** (API surface)
- **12 state transitions** across 3 entities (Container, Test, Service)
- **10 error scenarios** (missing Docker, invalid TOML, etc.)
- **7 data flows** (test execution, service lifecycle, OTEL tracing, etc.)
- **6 integration services** (Docker, Redis, Postgres, Jaeger, etc.)
- **18 expected spans** (clnrm.run, clnrm.test, container.exec, etc.)

## Advanced Usage

### Auto-Tracking in CleanroomEnvironment

Future enhancement to automatically track coverage:

```rust
let mut env = CleanroomEnvironment::new().await?;
env.enable_coverage_tracking().await;

// Coverage is automatically tracked as you use the environment
env.execute_in_container("alpine", &["echo", "hello"]).await?;

// Get coverage report
let report = env.generate_coverage_report("behaviors.clnrm.toml").await?;
```

### Mark Critical Behaviors

Ensure critical behaviors are tested:

```toml
[behavior.critical]
must_cover = [
    { type = "api", name = "clnrm run", reason = "primary command" },
    { type = "flow", name = "otel_tracing", reason = "core feature" }
]
```

### Differential Coverage

Compare coverage between versions:

```bash
# Track coverage for v1.0.0
clnrm coverage track --version 1.0.0

# Compare with v1.0.1
clnrm coverage diff --baseline 1.0.0 --current 1.0.1
# Output: +5.3% behavior coverage (8 new behaviors covered)
```

## FAQ

### Q: How is this different from code coverage?

**A**: Code coverage measures **what code ran**. Behavior coverage measures **what system behaviors were validated**.

Example:
- Code coverage: "We executed 85% of lines"
- Behavior coverage: "We validated 60% of user-facing behaviors"

You can have 100% code coverage but still miss testing critical behaviors like error handling, state transitions, or edge cases.

### Q: Do I need to manually track every behavior?

**A**: For now, yes. Future work will add automatic tracking integrated into CleanroomEnvironment. You can also add coverage tracking to your existing test suite incrementally.

### Q: What's a good target coverage percentage?

**A**:
- **60%+**: Minimum for production
- **70%+**: Good coverage
- **80%+**: Excellent coverage
- **90%+**: Exceptional (likely diminishing returns)

Focus on covering critical behaviors first (authentication, payments, data integrity) rather than achieving 100%.

### Q: Can I use this with existing tests?

**A**: Yes! Add coverage tracking to your existing tests:

```rust
#[tokio::test]
async fn existing_test() {
    let tracker = CoverageTracker::new();

    // Add tracking to existing test
    tracker.record_api("clnrm run").await;

    // ... your existing test code ...
}
```

### Q: How do I know what behaviors to define?

**A**:
1. **API Surface**: List all CLI commands (`clnrm --help`)
2. **State Transitions**: Draw state diagrams for key entities
3. **Error Scenarios**: Review error handling code
4. **Data Flows**: Map user journeys end-to-end
5. **Integrations**: List external dependencies
6. **Spans**: Review OTEL instrumentation

Or use auto-discovery from production traces:
```bash
clnrm coverage discover --source jaeger --endpoint http://jaeger:16686
```

## Next Steps

1. **Create your manifest**: Start with `behaviors.clnrm.toml`
2. **Add tracking**: Instrument your top 5 tests
3. **Generate report**: See your current coverage
4. **Identify gaps**: Focus on uncovered critical behaviors
5. **Iterate**: Add tests to increase coverage

## Resources

- **Design Doc**: `docs/BEHAVIOR_COVERAGE_DESIGN.md`
- **Implementation**: `docs/BEHAVIOR_COVERAGE_IMPLEMENTATION.md`
- **Example Manifest**: `examples/behaviors.clnrm.toml`
- **API Docs**: `cargo doc --open` (look for `coverage` module)

---

**Remember**: The goal isn't 100% coverage. The goal is **confidence that your system's critical behaviors are validated**.
