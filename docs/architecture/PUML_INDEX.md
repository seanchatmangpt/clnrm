# PlantUML Architecture Diagrams - Complete Index

**Purpose:** Complete visual documentation of Weaver Live Check integration with clnrm v1.2.0

**Status:** ✅ Complete suite of 10 comprehensive diagrams

**Last Updated:** 2025-10-30

---

## Core Principle

**clnrm v1.2.0 makes Weaver `registry live-check` the single source of truth for validation.**

All diagrams illustrate how Weaver validation prevents false positives by proving runtime behavior through telemetry schema validation.

---

## Diagram Suite

### 1. **weaver-live-check-complete.puml**
**Weaver Live Check - Complete Architecture**

**What it shows:**
- Complete Weaver Live Check architecture
- Input layer (OTLP gRPC :4317, HTTP :4318, File, stdin)
- Ingester layer (OTLP, File, Stdin with normalizers)
- Registry layer (clnrm registry, jq preprocessor)
- Advisor layer (Built-in, OTel Rego, Custom Rego)
- Output layer (Jinja2 templates, statistics, exit codes)
- Admin interface (:8080 for /stop, /health)

**Key insights:**
- Shows the critical path: Test → OTLP → Weaver → Advisors → Exit code
- Explains streaming vs batch modes
- Documents how false positives are impossible
- Registry: 14 schemas, 200+ entities, zero warnings

**Use when:** Understanding complete Weaver architecture

---

### 2. **weaver-advisor-system.puml**
**Weaver Advisor System - Detailed Architecture**

**What it shows:**
- How advisors process sample entities
- Built-in advisors: missing_attribute, type_mismatch, required_attribute, stability_check
- OTel Rego policies: namespace_check, format_validation, naming_convention, extends_namespace
- Custom user Rego policies (example: reject "test" in attribute names)
- Advice structure and levels (violation, improvement, information)
- How advice prevents false positives

**Key insights:**
- Detailed explanation of each advisor type
- Complete Rego policy examples
- Shows how `required_attribute` advisor proves container ran
- Advice levels and exit code logic
- Examples of advice objects in JSON

**Use when:** Understanding how validation works, writing custom advisors

---

### 3. **weaver-test-execution-flow.puml**
**clnrm Test Execution with Weaver Validation - Complete Flow**

**What it shows:**
- End-to-end sequence: Developer → Tests → OTel SDK → OTLP Exporter → Weaver → CI/CD Gate
- Initialization phase (config CRITICAL: OtlpGrpc not StdoutNdjson)
- Test execution phase (span creation, attribute setting)
- OTLP protobuf message structure
- Advisor validation in real-time
- Statistics generation phase
- CI/CD gate decision (exit 0 vs exit 1)

**Key insights:**
- Shows exact Rust code for test and config
- Demonstrates fake implementation detection
- Compares WITHOUT Weaver (ships false positive) vs WITH Weaver (blocked)
- Explains batch processor behavior
- Complete protobuf message example

**Use when:** Understanding how tests integrate with Weaver

---

### 4. **weaver-cicd-pipeline.puml**
**CI/CD Pipeline with Weaver Validation Gate**

**What it shows:**
- GitHub Actions workflow
- Build stage → Test stage → Weaver validation stage → Deploy stage
- Start Weaver in CI/CD
- Run tests with OTLP export
- Parse validation report
- Gate decision logic
- Merge/deploy flow

**Key insights:**
- Complete GitHub Actions YAML examples
- Shows how to parse live_check.json
- Violation handling (block PR)
- Success path (enable merge)
- Time to discover bugs: Minutes (with Weaver) vs Days (without)

**Use when:** Implementing CI/CD integration, writing GitHub Actions

---

### 5. **weaver-statistics-coverage.puml**
**Weaver Statistics Generation - Coverage Calculation**

**What it shows:**
- Entity counter (by type: attribute, span, metric, resource, event)
- Advice aggregator (level counts, type counts, highest levels)
- Coverage calculator (formula: seen / total)
- Attribute tracker (seen registry attrs, non-registry attrs)
- Metric tracker (similar to attributes)
- No advice counter

**Key insights:**
- Complete statistics JSON structure
- Coverage calculation formula and interpretation
- How to use statistics in CI/CD scripts
- clnrm target metrics (70% min, 85% target)
- Examples of good reports

**Use when:** Understanding coverage metrics, setting thresholds

---

### 6. **weaver-failure-modes.puml**
**Weaver Live Check - Failure Modes and Recovery**

**What it shows:**
- 5 failure modes with recovery strategies:
  1. Weaver not started → Pre-flight check
  2. Tests export to STDOUT → Environment variable
  3. Docker not running → Docker check
  4. Port already in use → Port check and kill
  5. Inactivity timeout → Timeout configuration
- Normal operation flow
- Comprehensive validation script (handles all modes)

**Key insights:**
- Detailed symptoms, causes, detection for each failure
- Recovery code examples (bash scripts)
- Pre-flight checks to prevent failures
- Complete production-ready validation script

**Use when:** Debugging validation issues, writing robust scripts

---

### 7. **weaver-core-architecture.puml** (Existing)
**Weaver-Centric clnrm Architecture**

**What it shows:**
- High-level clnrm architecture with Weaver at center
- Test engine, container backend, plugin system
- How all components export telemetry to Weaver
- Validation hierarchy

**Use when:** Understanding overall clnrm v1.2.0 architecture

---

### 8. **weaver-integration-sequence.puml** (Existing)
**Weaver Integration Detailed Sequence**

**What it shows:**
- Complete end-to-end sequence diagram
- All actors: Developer, CLI, Controller, Engine, Container Manager, Docker, OTLP, Weaver
- Initialization, test execution, cleanup phases
- Validation report generation
- The Weaver decision gate
- Critical decision point explanation

**Use when:** Understanding complete flow from CLI to validation report

---

### 9. **weaver-validation-flow.puml** (Existing)
**Weaver Validation Flow**

**What it shows:**
- Validation workflow: Schema → Code → Tests → Weaver
- Pass/fail paths
- False positive detection

**Use when:** Understanding validation workflow

---

### 10. **london-tdd-workflow.puml** (Existing)
**London TDD Workflow with Weaver**

**What it shows:**
- Red-Green-Refactor cycle
- Mock generation from schemas
- Weaver validation as final proof
- Test-first development with Weaver

**Use when:** Implementing London TDD methodology

---

### 11. **12-agent-swarm-topology.puml** (Existing)
**12-Agent Hive Queen Swarm Topology**

**What it shows:**
- Hive queen coordinator
- 4 layers: Architecture, Implementation, Testing, Validation
- 12 specialized agents
- Communication flows and feedback loops

**Use when:** Understanding agent-based development approach

---

## Diagram Categories

### Architecture & Design
- weaver-live-check-complete.puml
- weaver-core-architecture.puml
- weaver-advisor-system.puml

### Process & Flow
- weaver-test-execution-flow.puml
- weaver-integration-sequence.puml
- weaver-validation-flow.puml
- london-tdd-workflow.puml

### Operations & CI/CD
- weaver-cicd-pipeline.puml
- weaver-statistics-coverage.puml
- weaver-failure-modes.puml

### Development Methodology
- 12-agent-swarm-topology.puml

---

## Quick Reference

### For Developers
1. Start with: **weaver-test-execution-flow.puml**
2. Then read: **weaver-advisor-system.puml**
3. For debugging: **weaver-failure-modes.puml**

### For DevOps/CI
1. Start with: **weaver-cicd-pipeline.puml**
2. Then read: **weaver-statistics-coverage.puml**
3. For troubleshooting: **weaver-failure-modes.puml**

### For Architects
1. Start with: **weaver-live-check-complete.puml**
2. Then read: **weaver-core-architecture.puml**
3. For details: **weaver-integration-sequence.puml**

---

## Key Concepts Illustrated

### The False Positive Paradox
**Shown in:** weaver-test-execution-flow.puml, weaver-advisor-system.puml

Traditional tests can pass while features don't work. Weaver validation proves features work by validating actual runtime telemetry.

**Example:**
- Test returns `Ok(Output)` but no container ran
- Missing `container.id` attribute
- Weaver detects violation → Exit 1 → CI fails
- Cannot ship without proof

### The Validation Hierarchy
**Shown in:** weaver-live-check-complete.puml, weaver-core-architecture.puml

1. **Weaver Schema Validation** (HIGHEST) - Runtime proof
2. **Compilation** (SECOND) - Type safety
3. **Tests** (LOWEST) - Can have false positives

### The Critical Path
**Shown in:** weaver-test-execution-flow.puml, weaver-integration-sequence.puml

Test → Span → OTel SDK → Batch → OTLP gRPC → Weaver :4317 → Advisors → Violations → Exit 1 → Block

### Coverage Calculation
**Shown in:** weaver-statistics-coverage.puml

`coverage = seen_registry_entities / total_registry_entities`

Example: 42 / 50 = 0.84 (84% coverage)

---

## Statistics Targets (clnrm v1.2.0)

**Registry:**
- 14 schema files
- ~50 attributes
- ~200 total entities

**Coverage Targets:**
- Minimum: 70% (0.70)
- Target: 85% (0.85)
- Excellent: 95%+ (0.95)

**Violation Tolerance:** 0 (zero violations blocks merge)

---

## Rendering the Diagrams

### Using PlantUML CLI
```bash
# Install PlantUML
brew install plantuml

# Render all diagrams
for f in docs/architecture/*.puml; do
    plantuml "$f" -tpng
done

# Output: PNG files in same directory
```

### Using VS Code
1. Install "PlantUML" extension
2. Open .puml file
3. Press `Alt+D` to preview

### Using Online
Visit: http://www.plantuml.com/plantuml/uml/
Paste diagram source

---

## Maintenance

### When to Update

**Update weaver-live-check-complete.puml when:**
- Registry schema changes
- New ingester added
- New advisor type added

**Update weaver-advisor-system.puml when:**
- New built-in advisor added
- New OTel Rego policy
- Advice structure changes

**Update weaver-test-execution-flow.puml when:**
- Test initialization changes
- OTLP export config changes
- Validation flow changes

**Update weaver-cicd-pipeline.puml when:**
- GitHub Actions workflow changes
- New CI/CD stage added
- Gate logic changes

**Update weaver-statistics-coverage.puml when:**
- Statistics structure changes
- New coverage metrics added
- Thresholds change

**Update weaver-failure-modes.puml when:**
- New failure mode discovered
- Recovery strategy changes
- Validation script updated

---

## Version History

**v1.0 (2025-10-30)**
- Initial complete suite
- 11 total diagrams
- Covers all aspects of Weaver integration
- Aligned with clnrm v1.2.0 architecture
- Matches Weaver Live Check official documentation

**Planned Updates:**
- Add deployment architecture diagram
- Add performance optimization diagram
- Add multi-registry scenario diagram

---

## Related Documentation

- **Weaver Live Check Docs:** Official Weaver documentation (provided by user)
- **Root Cause Analysis:** `docs/WEAVER_VALIDATION_FAILURE_ROOT_CAUSE_ANALYSIS.md`
- **Validation Summary:** `docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md`
- **Integration Plan:** `docs/WEAVER_INTEGRATION_PLAN.md`
- **User Guide:** `docs/WEAVER_USER_GUIDE.md`

---

## Verification Checklist

### Diagram Completeness
- [x] Input sources documented
- [x] Ingesters explained
- [x] Registry integration shown
- [x] Advisors detailed (built-in, OTel, custom)
- [x] Output layer complete
- [x] Statistics calculation explained
- [x] CI/CD integration shown
- [x] Failure modes documented
- [x] Test execution flow complete
- [x] Coverage calculation explained

### Alignment with Weaver Docs
- [x] OTLP ingestion (gRPC :4317, HTTP :4318)
- [x] Admin interface (:8080, /stop, /health)
- [x] Inactivity timeout
- [x] Streaming vs batch modes
- [x] Advisor types (built-in, OTel, custom)
- [x] Advice levels (violation, improvement, information)
- [x] Statistics structure
- [x] Exit code logic
- [x] Rego policy examples
- [x] jq preprocessor

### clnrm Integration
- [x] OtlpGrpc vs StdoutNdjson explained
- [x] Container.id validation shown
- [x] Test.isolated proof documented
- [x] False positive prevention illustrated
- [x] CI/CD gate implementation
- [x] GitHub Actions examples
- [x] Docker dependency shown

---

**Status:** ✅ COMPLETE - Full suite ready for use

**Next Steps:**
1. Render all diagrams to PNG
2. Embed in markdown documentation
3. Use in presentations/reviews
4. Update as architecture evolves
