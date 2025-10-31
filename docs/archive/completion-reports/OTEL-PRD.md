# PRD: Telemetry-Only `.clnrm.toml` Schema

## Summary

Define a flat, indentation-free TOML schema that proves correctness via OpenTelemetry spans only. Spans are the single source of truth. No stdout, logs, or mocks.

## Goals

* Hermetic, reproducible, telemetry-only validation.
* Minimal keys. No duplication. Clear invariants.
* Composable expectations over spans, edges, counts, and time.

## Non-Goals

* Runtime implementation details, collectors, drivers.
* Error handling paths.
* UI/CLI behavior.

## Scope

* File format, required/optional keys, and validation rules.
* Acceptance criteria derived from span structure and attributes.

## Personas

* Infra/test engineers who write integration specs.
* CI owners who need deterministic proofs of behavior.

## Core Principles

* Sufficiency: everything computable from spans.
* Orthogonality: config concerns do not overlap.
* Invariance: hermeticity encoded as constraints.
* Minimality: happy path requires only essential keys.

---

## Schema (Authoritative Shape)

> All tables are flat. Use inline arrays and inline tables. No indentation.

### Identity

```toml
[meta]
name="string"
version="string"
description="string"
```

### Telemetry

```toml
[otel]
exporter="stdout"  # or "otlp"
endpoint="string"  # optional for stdout
protocol="string"  # e.g. "http/protobuf" if used
sample_ratio=1.0
resources={ "service.name"="string", "service.version"="string", "env"="string" }
```

### Service Under Test

```toml
[service.<id>]
plugin="generic_container"
image="string"
args=["string","..."]
env={ "KEY"="VALUE","..."="..." }
wait_for_span="string"  # earliest expected root span
```

### Scenario (Span Emitter)

```toml
[[scenario]]
name="string"
service="<id>"
run="string"
artifacts.collect=["spans:logical_handle"]
```

### Expectations: Span Structure

```toml
[[expect.span]]
name="string"
parent="string"           # optional; implies edge constraint
kind="internal"           # enum: internal|server|client|producer|consumer
attrs.all={ "k"="v","..."="..." }  # all must match
attrs.any=["k=v","k2=v2"]          # at least one matches
events.any=["string","..."]        # at least one present
duration_ms={ min=integer, max=integer }
```

### Expectations: Graph Topology

```toml
[expect.graph]
must_include=[["parent_span","child_span"],["p","c2"]]
must_not_cross=[["span_a","span_b"]]
acyclic=true
```

### Expectations: Cardinalities

```toml
[expect.counts]
spans_total={ gte=integer, lte=integer }
events_total={ gte=integer }
errors_total={ eq=0 }
by_name={ "span_name"={ eq=integer }, "span_b"={ gte=integer } }
```

### Expectations: Temporal Windows

```toml
[[expect.window]]
outer="root_span_name"
contains=["child_a","child_b"]
```

### Expectations: Hermeticity

```toml
[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={ "service.name"="string","env"="string" }
span_attrs.forbid_keys=["net.peer.name","db.connection_string","http.url"]
```

---

## Field Rules

* **meta.name**: required, unique per repo.
* **otel.exporter**: required.
* **service.<id>**: at least one required. `<id>` must be referenced by a scenario.
* **scenario**: at least one required; exactly one `service` reference per scenario.
* **expect.***: optional but recommended; if absent, only presence of root span is asserted.

Type constraints:

* Strings are UTF-8.
* Inline arrays preserve order.
* Inline tables are unordered sets.
* Duration bounds use milliseconds, inclusive.

---

## Validation Semantics

1. **Resource Gate**: `otel.resources` must be present; all `expect.hermeticity.resource_attrs.must_match` keys must equal.
2. **Root Presence**: `service.<id>.wait_for_span` must exist in collected spans.
3. **Span Matching**: Each `expect.span` maps to ≥1 concrete span; `attrs.all` are exact equality matches; `attrs.any` is disjunctive.
4. **Graph**: All `must_include` edges exist; `must_not_cross` forbids specified ordering or containment; `acyclic=true` forbids cycles.
5. **Counts**: Global and per-name counts satisfy bounds.
6. **Windows**: `outer`’s [start,end] strictly contains listed spans.
7. **Hermeticity**: `no_external_services=true` implies no spans with forbidden attributes or peers; `span_attrs.forbid_keys` must be absent everywhere.

---

## Minimal Happy-Path Example

```toml
[meta]
name="otel_self_validation"
version="1.0"
description="Telemetry-only proof"

[otel]
exporter="otlp"
endpoint="http://collector:4318"
protocol="http/protobuf"
sample_ratio=1.0
resources={ "service.name"="clnrm","service.version"="0.5.0","env"="ci" }

[service.clnrm]
plugin="generic_container"
image="example/clnrm:latest"
args=["self-test","--otel-exporter","otlp","--otel-endpoint","http://collector:4318"]
env={ "OTEL_TRACES_EXPORTER"="otlp","OTEL_EXPORTER_OTLP_ENDPOINT"="http://collector:4318" }
wait_for_span="clnrm.run"

[[scenario]]
name="otel_only_proof"
service="clnrm"
run="clnrm run --otel-exporter otlp --otel-endpoint http://collector:4318"
artifacts.collect=["spans:default"]

[[expect.span]]
name="clnrm.run"
kind="internal"
attrs.all={ "result"="pass" }
children.any=["clnrm.step:hello_world"]
duration_ms={ min=10, max=600000 }

[[expect.span]]
name="clnrm.step:hello_world"
parent="clnrm.run"
kind="internal"
attrs.any=["step.name=hello_world","status=ok"]
events.any=["container.start","container.exec","container.stop"]
duration_ms={ min=1, max=120000 }

[expect.graph]
must_include=[["clnrm.run","clnrm.step:hello_world"]]
acyclic=true

[expect.counts]
spans_total={ gte=2, lte=200 }
errors_total={ eq=0 }
by_name={ "clnrm.run"={ eq=1 } }

[[expect.window]]
outer="clnrm.run"
contains=["clnrm.step:hello_world"]

[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={ "service.name"="clnrm","env"="ci" }
span_attrs.forbid_keys=["net.peer.name","db.connection_string","http.url"]
```

---

## Backwards/Forwards Compatibility

* Unknown keys are ignored.
* New expectation blocks must default to non-enforcing when absent.
* Versioning via `meta.version`.

## Metrics of Success

* Zero flakiness across identical runs.
* Stable pass/fail across environments.
* Span graph diffs isolate all regressions.

## Acceptance Criteria

* A file that conforms to the schema can validate correctness using spans alone.
* Removing any declared constraint changes validation outcome predictably.
* Adding spans or attributes outside constraints does not produce false failures unless violating hermeticity or counts.

## Open Questions

* Standardized enum set for `kind`.
* Reserved attribute names for `attrs.any` string shorthand.

---

## Implementation Status

**Framework**: clnrm v0.5.0
**Implementation Date**: 2025-10-16
**Status**: ✅ COMPLETE

### Rust Implementation Mapping

| PRD Section | Rust Module | Status |
|-------------|-------------|--------|
| meta | config::TestMetadata | ✅ Complete |
| otel | telemetry::OtelConfig | ✅ Complete |
| service.<id> | config::ServiceConfig | ✅ Complete |
| scenario | config::StepConfig | ✅ Complete |
| expect.span | validation::span_validator | ✅ Complete |
| expect.graph | validation::graph_validator | ✅ Complete |
| expect.counts | validation::count_validator | ✅ Complete |
| expect.window | validation::window_validator | ✅ Complete |
| expect.hermeticity | validation::hermeticity_validator | ✅ Complete |

### Key Implementation Files

- `crates/clnrm-core/src/validation/graph_validator.rs` - Graph topology validation
- `crates/clnrm-core/src/validation/count_validator.rs` - Cardinality validation
- `crates/clnrm-core/src/validation/window_validator.rs` - Temporal window validation
- `crates/clnrm-core/src/validation/hermeticity_validator.rs` - Isolation validation
- `crates/clnrm-core/src/validation/orchestrator.rs` - Unified validation runner
- `crates/clnrm-core/src/validation/span_validator.rs` - Span-level assertions
- `crates/clnrm-core/src/config.rs` - TOML configuration parsing

### Test Coverage

- ✅ Unit tests for all validators
- ✅ Integration tests for PRD minimal happy path
- ✅ Property-based tests for edge cases
- ✅ Self-test using clnrm's own telemetry

### Usage Example

See `tests/self-test/clnrm-otel-validation.clnrm.toml` for complete working example
matching the PRD minimal happy path.

### Validation Flow

1. Parse `.clnrm.toml` → `TestConfig` with `OtelValidationSection`
2. Execute test → Collect spans via OTLP
3. Load spans → `SpanValidator::from_file()`
4. Build expectations → `PrdExpectations::from_config()`
5. Run validation → `expectations.validate_all(spans)`
6. Generate report → `ValidationReport` with pass/fail details

### Compliance Notes

- ✅ All TOML schema fields supported
- ✅ Validation semantics match PRD spec exactly
- ✅ Minimal happy path example passes
- ✅ Zero false positives
- ✅ Hermetic and reproducible

