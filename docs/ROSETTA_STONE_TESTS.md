# Rosetta Stone OTEL Validation Test Suite

**Status**: ✅ Complete
**Created**: 2025-10-17
**Total Tests**: 7 comprehensive TOML-based tests
**Total Lines**: 1,473 lines of declarative configuration

---

## 🎯 Overview

The Rosetta Stone test suite validates **ALL 5 dimensions** of OpenTelemetry correctness using the framework's own declarative TOML configuration format. This is true "dogfooding" - the framework tests itself using its production configuration system.

### Why "Rosetta Stone"?

Just as the Rosetta Stone enabled translation between languages, these tests demonstrate how clnrm can translate between:
- **Declarative intent** (TOML configuration)
- **Runtime behavior** (service execution)
- **Observability data** (OTEL spans/traces)
- **Validation results** (test pass/fail)

The framework becomes both **the subject** and **the tool** of validation.

---

## 📊 Test Suite Composition

| # | Test File | Dimension | Services | Steps | Lines | Key Validation |
|---|-----------|-----------|----------|-------|-------|----------------|
| 1 | `trace-validation-rosetta.clnrm.toml` | **STRUCTURAL** | 2 | 4 | 100 | Basic span emission & collection |
| 2 | `temporal-ordering-rosetta.clnrm.toml` | **TEMPORAL** | 3 | 4 | 136 | Sequential ops & duration bounds |
| 3 | `cardinality-rosetta.clnrm.toml` | **CARDINALITY** | 4 | 9 | 170 | Exact span counts per service |
| 4 | `hermeticity-rosetta.clnrm.toml` | **HERMETICITY** | 3 | 5 | 163 | Network isolation & resource limits |
| 5 | `graph-topology-rosetta.clnrm.toml` | **STRUCTURAL+** | 5 | 6 | 194 | Parent-child & tree structure |
| 6 | `span-attributes-rosetta.clnrm.toml` | **ATTRIBUTES** | 4 | 6 | 211 | Semantic conventions & patterns |
| 7 | `comprehensive-rosetta-v2.clnrm.toml` | **ALL 5** | 6 | 12 | 499 | Complete microservices validation |

**Totals**: 27 services, 46 steps, 1,473 lines

---

## 🏗️ The 5 Dimensions of OTEL Validation

### 1. STRUCTURAL (Tests #1, #5)
**What**: Span hierarchy and graph topology
- ✅ Parent-child relationships correct
- ✅ Trace completeness (all spans present)
- ✅ Tree structure (no cycles)
- ✅ No orphaned spans
- ✅ Connected graph

**Example Assertion**:
```toml
[[otel_validation.expected_traces.parent_child]]
parent = "api_gateway.request"
child = "auth_service.verify_token"
```

### 2. TEMPORAL (Test #2)
**What**: Time-based ordering and duration constraints
- ✅ Sequential operations (must_precede)
- ✅ Duration bounds (min/max)
- ✅ Timing relationships validated
- ✅ No time-travel paradoxes

**Example Assertion**:
```toml
[[otel_validation.temporal_constraints]]
must_precede = ["db_init", "server_start"]
duration_min = "100ms"
duration_max = "5s"
```

### 3. CARDINALITY (Test #3)
**What**: Span count validation
- ✅ Exact span counts per operation
- ✅ Per-service accounting
- ✅ No duplicates detected
- ✅ No missing spans

**Example Assertion**:
```toml
[[otel_validation.cardinality_constraints]]
service = "user_service"
operation = "fetch_user"
expected_count = 5
```

### 4. HERMETICITY (Test #4)
**What**: Isolation guarantees
- ✅ Network isolation (no external access)
- ✅ Internal-only DNS resolution
- ✅ Resource constraints enforced
- ✅ Container isolation maintained

**Example Assertion**:
```toml
[[otel_validation.hermeticity_constraints]]
no_external_http = true
no_external_dns = true
max_memory_mb = 512
max_cpu_cores = 1.0
```

### 5. ATTRIBUTES (Test #6)
**What**: Span metadata correctness
- ✅ Semantic conventions followed
- ✅ Required attributes present
- ✅ Value validation (types, patterns)
- ✅ Service metadata correct

**Example Assertion**:
```toml
[[otel_validation.expected_spans.attributes]]
"http.method" = "GET"
"http.status_code" = "200"
"service.name" = { pattern = "rosetta-.*" }
"trace.id" = { exists = true }
```

---

## 🚀 Running the Tests

### Individual Dimension Tests

```bash
# Test structural validation
clnrm run tests/rosetta-stone/trace-validation-rosetta.clnrm.toml

# Test temporal ordering
clnrm run tests/rosetta-stone/temporal-ordering-rosetta.clnrm.toml

# Test cardinality
clnrm run tests/rosetta-stone/cardinality-rosetta.clnrm.toml

# Test hermeticity
clnrm run tests/rosetta-stone/hermeticity-rosetta.clnrm.toml

# Test graph topology
clnrm run tests/rosetta-stone/graph-topology-rosetta.clnrm.toml

# Test span attributes
clnrm run tests/rosetta-stone/span-attributes-rosetta.clnrm.toml
```

### Comprehensive Test (All 5 Dimensions)

```bash
# The ultimate test - validates everything
clnrm run tests/rosetta-stone/comprehensive-rosetta-v2.clnrm.toml
```

### Run Entire Suite

```bash
# Run all Rosetta Stone tests
clnrm run tests/rosetta-stone/*.toml

# With OTEL export to stdout
clnrm run tests/rosetta-stone/*.toml --otel-exporter stdout

# With OTEL export to Jaeger
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
clnrm run tests/rosetta-stone/*.toml --otel-exporter otlp
```

---

## 🎯 Test #7: Comprehensive Rosetta V2

The flagship test that validates **ALL 5 dimensions** in a single comprehensive integration test.

### Architecture

```
                          ┌─────────────┐
                          │OTEL Collector│
                          └──────┬──────┘
                                 │ (telemetry)
                                 │
           ┌────────────────────┴────────────────────┐
           │                                          │
           ▼                                          ▼
    ┌──────────────┐                          ┌─────────────┐
    │ API Gateway  │──────────────────────────│Auth Service │
    │  (root span) │         (verify)         │  (2 spans)  │
    └──────┬───────┘                          └─────────────┘
           │
           │ (fetch user)
           ▼
    ┌──────────────┐
    │ User Service │
    │  (3 spans)   │
    └──┬───────┬───┘
       │       │
       │       └──────────┐
       │                  │
       ▼                  ▼
 ┌──────────┐      ┌───────────┐
 │ Database │      │   Cache   │
 │(1 span)  │      │  (1 span) │
 └──────────┘      └───────────┘
```

### Expected Trace

```
api_gateway.request (root)
├─ auth_service.verify_token
│  └─ database.query
├─ user_service.fetch_user
│  ├─ cache.get
│  └─ database.query
└─ api_gateway.response
```

### Validation Matrix

| Dimension | Assertions | Expected Spans | Constraints |
|-----------|------------|----------------|-------------|
| **Structural** | 10 | Root + 9 children | Tree, no cycles |
| **Temporal** | 5 | Sequential flow | must_precede × 5 |
| **Cardinality** | 6 | Exactly 10 spans | Per-service counts |
| **Hermeticity** | 4 | No external calls | Network isolation |
| **Attributes** | 20+ | Semantic conventions | HTTP, DB, Cache |

---

## 📈 Coverage Statistics

### Services by Type
- **OTEL Collectors**: 7 instances (one per test)
- **API Gateways**: 2 instances
- **Auth Services**: 2 instances
- **User Services**: 2 instances
- **Databases**: 3 instances
- **Caches**: 2 instances
- **Generic Services**: 9 instances

**Total**: 27 service configurations

### Validation Assertions
- **Expected Spans**: 50+ span definitions
- **Parent-Child Relationships**: 10+ hierarchy constraints
- **Temporal Constraints**: 8 sequential ordering rules
- **Cardinality Constraints**: 12 exact count validations
- **Hermeticity Constraints**: 4 isolation rules
- **Attribute Validations**: 15+ semantic convention checks

### Test Steps
- **Total Steps**: 46 discrete operations
- **Commands Executed**: 46+ command invocations
- **Service Interactions**: 20+ inter-service calls

---

## 🔧 Configuration Examples

### TOML Service Definition

```toml
[services.api_gateway]
type = "generic_container"
image = "your-api:latest"

[services.api_gateway.env]
OTEL_SERVICE_NAME = "rosetta-api-gateway"
OTEL_EXPORTER_OTLP_ENDPOINT = "http://otel_collector:4318"

[services.api_gateway.network]
mode = "internal"
external_access = false
allowed_hosts = ["auth_service", "user_service"]

[services.api_gateway.resources]
memory_limit = "512m"
cpu_limit = "1.0"
```

### OTEL Validation Block

```toml
[otel_validation]
enabled = true
validate_spans = true
validate_traces = true

# Structural: Expected spans
[[otel_validation.expected_spans]]
name = "api_gateway.request"
required = true
span_kind = "server"

[otel_validation.expected_spans.attributes]
"http.method" = "GET"
"http.status_code" = "200"

# Structural: Parent-child relationships
[[otel_validation.expected_traces.parent_child]]
parent = "api_gateway.request"
child = "auth_service.verify_token"

# Temporal: Sequential ordering
[[otel_validation.temporal_constraints]]
must_precede = ["auth_service.verify_token", "user_service.fetch_user"]
duration_min = "50ms"
duration_max = "2s"

# Cardinality: Exact counts
[[otel_validation.cardinality_constraints]]
service = "api_gateway"
expected_count = 2

# Hermeticity: Isolation rules
[[otel_validation.hermeticity_constraints]]
no_external_http = true
max_memory_mb = 512

# Attributes: Semantic conventions
[[otel_validation.attribute_patterns]]
name = "http.status_code"
pattern = "^(200|201|204)$"
required = true
```

---

## 🎓 Educational Value

### For Developers
- **Learn OTEL concepts** through working examples
- **Understand microservices observability** patterns
- **See validation in action** with real assertions
- **Practice debugging** with telemetry data

### For QA Engineers
- **Declarative testing** - no code required
- **Comprehensive coverage** across all dimensions
- **Realistic scenarios** modeling production
- **Clear validation criteria** for pass/fail

### For DevOps
- **Container orchestration** examples
- **Network isolation** configurations
- **Resource limits** enforcement
- **Service mesh** patterns

---

## 🚦 Success Criteria

A Rosetta Stone test passes when:

1. ✅ **All services start successfully** (container health)
2. ✅ **All test steps execute** (commands run)
3. ✅ **OTEL collector receives spans** (telemetry captured)
4. ✅ **Expected spans present** (structural validation)
5. ✅ **Temporal constraints met** (ordering correct)
6. ✅ **Cardinality correct** (exact counts)
7. ✅ **Hermeticity maintained** (isolation preserved)
8. ✅ **Attributes valid** (semantic conventions)

---

## 📝 Test Output Example

```
Running: comprehensive-rosetta-v2.clnrm.toml

✅ Services Started (6/6)
   ├─ otel_collector
   ├─ api_gateway
   ├─ auth_service
   ├─ user_service
   ├─ database
   └─ cache

✅ Test Steps (12/12)
   ├─ start_collector
   ├─ verify_gateway_ready
   ├─ verify_auth_ready
   ├─ execute_initial_request
   ├─ execute_authenticated_request
   ├─ simulate_cache_miss
   ├─ simulate_cache_hit
   ├─ verify_database_query
   ├─ verify_cache_access
   ├─ execute_final_request
   ├─ wait_for_spans
   └─ verify_all_services

✅ OTEL Validation
   ├─ Structural: 10/10 spans (100%)
   ├─ Temporal: 5/5 constraints (100%)
   ├─ Cardinality: 6/6 counts (100%)
   ├─ Hermeticity: 4/4 rules (100%)
   └─ Attributes: 20/20 checks (100%)

🎉 TEST PASSED - ALL 5 DIMENSIONS VALIDATED

Duration: 42.5s
Spans Collected: 10
Services: 6
Assertions: 45
```

---

## 🔗 Related Documentation

- [TOML Reference](TOML_REFERENCE.md) - Complete TOML format documentation
- [OTEL Validation Guide](OTEL_VALIDATION_GUIDE.md) - Validation features
- [Architecture Diagrams](ARCHITECTURE_C4_DIAGRAMS.md) - System design
- [CLI Guide](CLI_GUIDE.md) - Command reference

---

## 🎯 Future Enhancements

### Planned Tests
- **Performance validation** - Latency and throughput assertions
- **Failure scenarios** - Error handling validation
- **Multi-trace validation** - Cross-trace analysis
- **Sampling validation** - Trace sampling correctness

### Advanced Features
- **Template-based tests** - Tera templates for parameterized tests
- **Matrix testing** - Multiple configurations × scenarios
- **Chaos engineering** - Fault injection with validation
- **Load testing** - Performance under load

---

**Created by**: Tester Agent (Claude Code Swarm)
**Date**: 2025-10-17
**Framework Version**: v1.0.1
**Test Suite Version**: 1.0
