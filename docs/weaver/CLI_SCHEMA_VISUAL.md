# CLI Schema Visual Reference

## Schema Organization Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    CLNRM CLI SCHEMA REGISTRY                     │
│                         (v1.2.0)                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
    ┌───────┐           ┌──────────┐          ┌─────────┐
    │ SPANS │           │ METRICS  │          │ EVENTS  │
    └───┬───┘           └────┬─────┘          └────┬────┘
        │                    │                     │
        │                    │                     │
   ┌────┴────┐          ┌────┴────┐          ┌────┴────┐
   │         │          │         │          │         │
   ▼         ▼          ▼         ▼          ▼         ▼
┌──────┐ ┌─────┐   ┌──────┐ ┌──────┐   ┌──────┐ ┌──────┐
│ Core │ │ CLI │   │ Core │ │ CLI  │   │ Core │ │ CLI  │
│      │ │     │   │      │ │      │   │      │ │      │
└──────┘ └─────┘   └──────┘ └──────┘   └──────┘ └──────┘
```

## CLI Commands Schema Hierarchy

```
registry/cli/
│
├── initialization.yaml          ┌─────────────────────┐
│   └── span.clnrm.cli.init ────▶│ init                │
│                                 │ - project.path      │
│                                 │ - config.generated  │
│                                 └─────────────────────┘
│
├── health_check.yaml            ┌─────────────────────┐
│   └── span.clnrm.cli.health ──▶│ health              │
│                                 │ - health.overall    │
│                                 │ - checks_total      │
│                                 └─────────────────────┘
│
├── plugin_operations.yaml       ┌─────────────────────┐
│   └── span.clnrm.cli.plugins ─▶│ plugins             │
│                                 │ - plugins.discovered│
│                                 │ - plugins.by_type   │
│                                 └─────────────────────┘
│
├── service_management.yaml      ┌─────────────────────┐
│   ├── span.clnrm.cli.services ▶│ services            │
│   │                             │ - service.operation │
│   │                             │ - services.total    │
│   │                             └─────────────────────┘
│   │
│   └── span.clnrm.cli.collector ┌─────────────────────┐
│                                ▶│ collector           │
│                                 │ - collector.operation│
│                                 │ - collector.running │
│                                 └─────────────────────┘
│
├── project_operations.yaml      ┌─────────────────────┐
│   ├── span.clnrm.cli.fmt ─────▶│ fmt                 │
│   │                             │ - files.formatted   │
│   │                             └─────────────────────┘
│   │
│   ├── span.clnrm.cli.render ──▶┌─────────────────────┐
│   │                             │ render              │
│   │                             │ - template.path     │
│   │                             └─────────────────────┘
│   │
│   └── span.clnrm.cli.record ──▶┌─────────────────────┐
│                                 │ record              │
│                                 │ - baseline.digest   │
│                                 └─────────────────────┘
│
├── image_operations.yaml        ┌─────────────────────┐
│   ├── span.clnrm.cli.pull ────▶│ pull                │
│   │                             │ - images.discovered │
│   │                             │ - parallel.enabled  │
│   │                             └─────────────────────┘
│   │                                      │
│   └── span.clnrm.cli.pull.image ────────┘ (child span)
│                                 ┌─────────────────────┐
│                                ▶│ pull.image          │
│                                 │ - image.digest      │
│                                 │ - pull.duration_ms  │
│                                 └─────────────────────┘
│
└── tdd_workflow.yaml            ┌─────────────────────┐
    ├── span.clnrm.cli.red_green ▶│ red-green           │
    │                             │ - tdd.expected_state│
    │                             │ - tdd.validation_passed│
    │                             └─────────────────────┘
    │
    └── span.clnrm.cli.repro ────▶┌─────────────────────┐
                                   │ repro               │
                                   │ - baseline.digest   │
                                   │ - digest.verified   │
                                   └─────────────────────┘
```

## Span Attribute Inheritance

```
┌────────────────────────────────────────────────────────────┐
│               UNIVERSAL CLI ATTRIBUTES                      │
│  (All CLI spans MUST have these)                           │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ • cli.command          (string, required)            │ │
│  │ • operation.duration_ms (double, required, must > 0) │ │
│  │ • operation.success     (boolean, required)          │ │
│  │ • error.type           (string, conditionally req'd) │ │
│  │ • error.message        (string, conditionally req'd) │ │
│  └──────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
                         │
                         │ inherited by all
                         │
    ┌────────────────────┼────────────────────┐
    │                    │                    │
    ▼                    ▼                    ▼
┌─────────┐         ┌─────────┐        ┌──────────┐
│  init   │         │ health  │        │ plugins  │
│         │         │         │        │          │
│ + proj  │         │ + health│        │ + plugin │
│   .path │         │   .checks│       │   .count │
└─────────┘         └─────────┘        └──────────┘
    │                    │                    │
    │ + specific         │ + specific         │ + specific
    │   attributes       │   attributes       │   attributes
    ▼                    ▼                    ▼
```

## Validation Flow Diagram

```
┌──────────────┐
│ CLI Command  │
│  Execution   │
└──────┬───────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 1. Emit span.clnrm.cli.{command}            │
│    - Start span at command entry             │
│    - Set cli.command attribute               │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 2. Execute command logic                     │
│    - Perform actual operations               │
│    - Track timestamps                        │
│    - Collect metadata                        │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 3. Populate span attributes                  │
│    - Set all required attributes             │
│    - Set conditional attributes (if needed)  │
│    - Calculate operation.duration_ms         │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 4. Set operation.success                     │
│    - true: Command succeeded                 │
│    - false: Command failed                   │
│       └─> Set error.type & error.message     │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 5. End span                                  │
│    - Export to OTLP collector                │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ 6. Weaver validation                         │
│    - Check required attributes present       │
│    - Validate attribute types                │
│    - Verify conditional requirements         │
│    - Validate count balances                 │
└──────┬───────────────────────────────────────┘
       │
       ▼
  ┌────┴────┐
  │         │
  ▼         ▼
✅ PASS   ❌ FAIL
  │         │
  │         └─> Event: clnrm.cli.validation.failed
  │
  └─> ✓ Command validated by telemetry
```

## Count Balance Validation Pattern

```
services command:
┌────────────────────────────────────────────┐
│ services.total                             │
│    │                                       │
│    ├─▶ services.running                   │
│    ├─▶ services.stopped                   │
│    └─▶ services.error                     │
│                                            │
│ VALIDATION:                                │
│   running + stopped + error = total       │
└────────────────────────────────────────────┘

pull command:
┌────────────────────────────────────────────┐
│ images.discovered                          │
│    │                                       │
│    ├─▶ images.pulled                      │
│    ├─▶ images.failed                      │
│    └─▶ images.skipped                     │
│                                            │
│ VALIDATION:                                │
│   pulled + failed + skipped = discovered  │
└────────────────────────────────────────────┘

fmt command:
┌────────────────────────────────────────────┐
│ files.input (array length)                 │
│    │                                       │
│    ├─▶ files.formatted                    │
│    ├─▶ files.unchanged                    │
│    └─▶ files.errors                       │
│                                            │
│ VALIDATION:                                │
│   formatted + unchanged + errors = input  │
└────────────────────────────────────────────┘
```

## Parent-Child Span Relationships

```
run command (existing):
┌─────────────────────────────────────────────────┐
│ span.clnrm.test_execution                      │
│   - test.name                                  │
│   - container.id                               │
└─────────────┬───────────────────────────────────┘
              │
              ├─▶ span.clnrm.container_lifecycle
              │     - container.state
              │     - cleanup.success
              │
              └─▶ span.clnrm.plugin_execution
                    - plugin.state
                    - health_check.performed

pull command (new):
┌─────────────────────────────────────────────────┐
│ span.clnrm.cli.pull                            │
│   - images.discovered                          │
│   - parallel.enabled                           │
└─────────────┬───────────────────────────────────┘
              │
              ├─▶ span.clnrm.cli.pull.image (image 1)
              │     - image.name: "alpine:latest"
              │     - image.digest: "sha256:abc..."
              │
              ├─▶ span.clnrm.cli.pull.image (image 2)
              │     - image.name: "postgres:15"
              │     - image.digest: "sha256:def..."
              │
              └─▶ span.clnrm.cli.pull.image (image N)
                    - image.name: "redis:7"
                    - image.digest: "sha256:ghi..."
```

## Metric Aggregation Flow

```
┌─────────────────────────────────────────────────────────┐
│           CLI COMMAND EXECUTION                         │
└─────────────────┬───────────────────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
    ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│  Span   │  │ Metrics │  │ Events  │
│ Emitted │  │ Recorded│  │ Emitted │
└────┬────┘  └────┬────┘  └────┬────┘
     │            │             │
     │            │             │
     ▼            ▼             ▼
┌────────────────────────────────────┐
│   OTLP Collector                   │
└────────┬───────────────────────────┘
         │
         ▼
┌────────────────────────────────────┐
│   Aggregation by Attribute         │
│                                    │
│   cli.command:                     │
│     - "init": 42 invocations       │
│     - "health": 128 invocations    │
│     - "run": 356 invocations       │
│                                    │
│   operation.success:               │
│     - true: 520 (98.5%)            │
│     - false: 6 (1.5%)              │
└────────┬───────────────────────────┘
         │
         ▼
┌────────────────────────────────────┐
│   Dashboards / Analytics           │
│   - Command popularity             │
│   - Success rates                  │
│   - P50/P95/P99 latencies          │
│   - Error patterns                 │
└────────────────────────────────────┘
```

## Weaver Live-Check Validation Points

```
For EVERY CLI span:

┌─────────────────────────────────────────┐
│ 1. Required Attributes Present          │
│    ✓ cli.command exists                 │
│    ✓ operation.duration_ms exists       │
│    ✓ operation.success exists           │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ 2. Attribute Type Validation            │
│    ✓ cli.command is string              │
│    ✓ operation.duration_ms is double    │
│    ✓ operation.success is boolean       │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ 3. Value Constraints                    │
│    ✓ operation.duration_ms > 0          │
│    ✓ cli.command in known commands list │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ 4. Conditional Requirements             │
│    IF operation.success = false THEN:   │
│      ✓ error.type MUST exist            │
│      ✓ error.message MUST exist         │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ 5. Command-Specific Validation          │
│    FOR services command:                │
│      ✓ running + stopped + error = total│
│    FOR pull command:                    │
│      ✓ pulled + failed + skipped = disc │
└─────────────────────────────────────────┘
                  ↓
           ┌──────┴──────┐
           │             │
           ▼             ▼
        ✅ PASS       ❌ FAIL
```

## Schema Implementation Timeline

```
Week 1: High Priority
┌──────────────────────────────────────┐
│ Day 1-2: init schema                 │
│   - Create initialization.yaml       │
│   - Validate with Weaver             │
│   - Generate Rust code               │
│   - Implement instrumentation        │
├──────────────────────────────────────┤
│ Day 3-4: health schema               │
│   - Create health_check.yaml         │
│   - Validate with Weaver             │
│   - Generate Rust code               │
│   - Implement instrumentation        │
├──────────────────────────────────────┤
│ Day 5: Integration tests             │
│   - Test init telemetry              │
│   - Test health telemetry            │
│   - Weaver live-check validation     │
└──────────────────────────────────────┘

Week 2: Medium Priority
┌──────────────────────────────────────┐
│ Day 6-7: Service management schemas  │
│   - services, collector              │
│   - Validate and implement           │
├──────────────────────────────────────┤
│ Day 8-9: Project operations schemas  │
│   - fmt, render, record              │
│   - Validate and implement           │
├──────────────────────────────────────┤
│ Day 10: plugins schema               │
│   - plugin_operations.yaml           │
│   - Validate and implement           │
└──────────────────────────────────────┘

Week 3: Low Priority
┌──────────────────────────────────────┐
│ Day 11-12: Image operations          │
│   - pull command with child spans    │
│   - Validate and implement           │
├──────────────────────────────────────┤
│ Day 13-14: TDD workflow schemas      │
│   - red-green, repro                 │
│   - Validate and implement           │
├──────────────────────────────────────┤
│ Day 15: CI/CD integration            │
│   - Automated Weaver validation      │
│   - Documentation updates            │
└──────────────────────────────────────┘

Week 4: Finalization
┌──────────────────────────────────────┐
│ Day 16-17: Metrics & Events          │
│   - cli_metrics.yaml                 │
│   - cli_events.yaml                  │
│   - Implement and validate           │
├──────────────────────────────────────┤
│ Day 18-19: Complete testing          │
│   - All commands validated           │
│   - End-to-end testing               │
│   - Performance benchmarking         │
├──────────────────────────────────────┤
│ Day 20: Documentation & Release      │
│   - Update all docs                  │
│   - Create migration guide           │
│   - Release v1.2.0                   │
└──────────────────────────────────────┘
```

## Coverage Gap Visualization

```
BEFORE (v1.1.0):
CLI Commands: ██████████░░░░░░░░░░░░ 48% (11/23)
                ▲
                └─ Missing telemetry for 11 commands

AFTER (v1.2.0):
CLI Commands: ████████████████████░░ 96% (22/23)
                                   ▲
                                   └─ Only 'analyze' command excluded
                                      (requires OTEL collector setup)

Progress:
  ┌────────────────────────────────────────────┐
  │ Instrumented:   22 commands                │
  │ Uninstrumented:  1 command (analyze)       │
  │ Coverage gain:  +48% → 96%                 │
  │ Schema files:   +7 new files               │
  │ Validation:     100% Weaver-compliant      │
  └────────────────────────────────────────────┘
```

---

**Visual Schema Architecture Complete**

**Next Steps:**
1. Create schema YAML files following this architecture
2. Validate all schemas with `weaver registry check`
3. Begin implementation with init and health commands
4. Iterate through remaining commands per timeline

**References:**
- Full architecture: `docs/weaver/CLI_SCHEMA_ARCHITECTURE.md`
- Quick reference: `docs/weaver/CLI_SCHEMA_QUICK_REFERENCE.md`
- Existing schemas: `registry/core/`, `registry/metrics/`, `registry/events/`
