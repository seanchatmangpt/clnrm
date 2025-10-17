# Rosetta Stone: OTEL Validation Test Suite

**7 Comprehensive TOML-based tests validating all 5 dimensions of OpenTelemetry correctness**

This test suite demonstrates the framework's self-testing capabilities through declarative TOML configuration. Each test validates a specific dimension of OTEL functionality while the comprehensive test validates all dimensions together.

## Test Suite Overview

| Test File | Dimension | Services | Steps | Total Lines | Key Validations |
|-----------|-----------|----------|-------|-------------|-----------------|
| `trace-validation-rosetta.clnrm.toml` | STRUCTURAL | 2 | 4 | 100 | Basic span emission, collector capture |
| `temporal-ordering-rosetta.clnrm.toml` | TEMPORAL | 3 | 4 | 136 | Sequential operations, duration constraints |
| `cardinality-rosetta.clnrm.toml` | CARDINALITY | 4 | 9 | 170 | Exact span counts per service |
| `hermeticity-rosetta.clnrm.toml` | HERMETICITY | 3 | 5 | 163 | Network isolation, resource limits |
| `graph-topology-rosetta.clnrm.toml` | STRUCTURAL (advanced) | 5 | 6 | 194 | Parent-child relationships, tree structure |
| `span-attributes-rosetta.clnrm.toml` | ATTRIBUTES | 4 | 6 | 211 | Semantic conventions, required attributes |
| `comprehensive-rosetta-v2.clnrm.toml` | **ALL 5 DIMENSIONS** | 6 | 12 | 499 | Complete microservices validation |

**Total:** 1,473 lines of declarative test configuration

## The 5 Dimensions of OTEL Validation

### 1. STRUCTURAL
- Span hierarchy (parent-child relationships)
- Trace completeness
- Graph topology (tree structure, no cycles)
- No orphaned spans

### 2. TEMPORAL
- Sequential ordering (must_precede constraints)
- Duration bounds (min/max)
- Timing relationships
- No time-travel paradoxes

### 3. CARDINALITY
- Exact span counts
- Per-service accounting
- No duplicates
- No missing spans

### 4. HERMETICITY
- Network isolation (no external access)
- Internal-only DNS
- Resource constraints
- Container isolation

### 5. ATTRIBUTES
- Semantic conventions (OpenTelemetry standards)
- Required attributes present
- Value validation (types, patterns, ranges)
- Service metadata

## Running the Tests

```bash
# Run individual test
clnrm run tests/rosetta-stone/trace-validation-rosetta.clnrm.toml

# Run all rosetta stone tests
clnrm run tests/rosetta-stone/

# Run with OTEL export to stdout
clnrm run tests/rosetta-stone/comprehensive-rosetta-v2.clnrm.toml --otel-exporter stdout

# Run with OTEL export to collector
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
clnrm run tests/rosetta-stone/ --features otel
```

## Example Span Validations

### Test #1: Basic Trace Validation
```toml
[[otel_validation.expected_spans]]
name = "app.operation.start"
required = true

[otel_validation.expected_spans.attributes]
"service.name" = "rosetta-basic-app"
"operation.type" = "execute"
```

### Test #2: Temporal Ordering
```toml
[[otel_validation.temporal_constraints]]
span_name = "db.initialize"
must_precede = ["server.start"]
reason = "Database must initialize before server starts"

[[otel_validation.expected_spans]]
name = "db.initialize"
min_duration_ms = 2000
max_duration_ms = 3000
```

### Test #3: Cardinality
```toml
[[otel_validation.span_count_constraints]]
service_name = "rosetta-service-a"
span_name = "service.operation"
exact_count = 5
reason = "Service A performs exactly 5 operations"
```

### Test #4: Hermeticity
```toml
[[otel_validation.hermeticity_constraints]]
no_external_network = true
internal_dns_only = true
resource_limits_enforced = true

[[otel_validation.expected_spans]]
name = "network.access.blocked"
[otel_validation.expected_spans.attributes]
"network.allowed" = "false"
```

### Test #5: Graph Topology
```toml
[[otel_validation.expected_traces.parent_child]]
parent = "gateway.request"
child = "auth.validate"

[[otel_validation.graph_topology_constraints]]
no_orphaned_spans = true
no_cycles = true
single_root = true
max_depth = 3
```

### Test #6: Span Attributes
```toml
[[otel_validation.attribute_constraints]]
attribute_name = "http.status_code"
data_type = "integer"
valid_range = [200, 599]

[[otel_validation.attribute_constraints]]
attribute_name = "service.version"
pattern = "^\\d+\\.\\d+\\.\\d+$"  # Semantic versioning
```

### Test #7: Comprehensive (All 5 Dimensions)

**Architecture:**
```
API Gateway → Auth Service → User Service → Database
                                          ↘ Cache
```

**10 Expected Spans:**
1. `gateway.request` (root, 2x)
2. `auth.validate` (2x)
3. `user.fetch` (2x)
4. `cache.lookup` (2x)
5. `db.query` (1x - cache miss)
6. `cache.set` (1x - update cache)

**Validations:**
- STRUCTURAL: Parent-child relationships, tree integrity
- TEMPORAL: db.initialize → cache.warm → gateway.request
- CARDINALITY: Exactly 10 spans total
- HERMETICITY: No external network, resource limits enforced
- ATTRIBUTES: HTTP, DB, and cache semantic conventions

## Key Features

### Dogfooding Pattern
The framework tests itself using its own TOML configuration format. This ensures:
- Configuration format is expressive enough for real-world scenarios
- OTEL validation logic works correctly
- Self-testing increases confidence

### Declarative Validation
No code required - all validation logic is declarative:
```toml
[otel_validation]
enabled = true
validate_spans = true
validate_traces = true
validate_temporal_ordering = true
validate_cardinality = true
validate_hermeticity = true
validate_attributes = true
```

### Realistic Scenarios
Tests model real microservices architectures:
- API gateways with authentication
- Database queries with caching
- Sequential service dependencies
- Network isolation and resource constraints

## Integration with Framework

These tests validate the OTEL implementation in:
- `crates/clnrm-core/src/telemetry/init.rs` - OTEL initialization
- `crates/clnrm-core/src/validation/otel.rs` - Validation logic
- `crates/clnrm-core/tests/otel_validation_integration.rs` - Integration tests

## Expected Outcomes

When all tests pass:
- ✅ Spans are correctly emitted and captured
- ✅ Temporal ordering is maintained
- ✅ Span counts are accurate
- ✅ Tests run in hermetic isolation
- ✅ Attributes follow semantic conventions
- ✅ Graph topology is valid (no cycles, orphans)

## Debugging Failed Tests

If a test fails, check:

1. **OTEL Collector logs**: `docker logs <collector-container>`
2. **Span export**: Run with `--otel-exporter stdout`
3. **Validation output**: Framework reports which constraint failed
4. **Service logs**: Check individual service container logs

## Next Steps

1. Implement TOML parsing for OTEL validation constraints
2. Build validation engine that checks all 5 dimensions
3. Add OTEL collector plugin to service registry
4. Run tests and iterate on implementation

## References

- OpenTelemetry Semantic Conventions: https://opentelemetry.io/docs/specs/semconv/
- TOML Format: https://toml.io/
- Framework OTEL Guide: `/docs/OTEL_VALIDATION.md`
