# Behavior Coverage Design for clnrm

## Problem Statement

How do we determine what percentage of a system's behaviors are actually covered by clnrm tests? We need heuristics that go beyond code coverage to measure **behavioral coverage** - the extent to which system behaviors are validated.

## Design Philosophy

**Behavior Coverage** ≠ Code Coverage

- **Code Coverage**: Which lines of code were executed
- **Behavior Coverage**: Which system behaviors were validated

### Key Insight: OTEL Spans as Behavior Proof

Since clnrm is OTEL-first, we can use spans as **behavioral evidence**:
- Each span represents an observable behavior
- Span attributes capture behavioral context
- Span events capture state transitions
- Span relationships capture workflow patterns

## Behavior Coverage Dimensions

### 1. API Surface Coverage

**Metric**: What percentage of API endpoints/commands have been tested?

```toml
[behavior.api_surface]
# Define the complete API surface
total_endpoints = [
  "GET /users",
  "POST /users",
  "GET /users/:id",
  "PUT /users/:id",
  "DELETE /users/:id",
  "GET /health",
  "GET /metrics"
]

# Tests that exercise each endpoint
covered_by = {
  "GET /users" = ["test_list_users.clnrm.toml"],
  "POST /users" = ["test_create_user.clnrm.toml"],
  "GET /users/:id" = ["test_get_user.clnrm.toml"]
}

# Calculation: 3/7 = 42.9% API surface coverage
```

### 2. State Transition Coverage

**Metric**: What percentage of state transitions have been validated?

```toml
[behavior.state_transitions]
# Define valid state transitions for an entity
entity = "Order"
states = ["pending", "confirmed", "shipped", "delivered", "cancelled"]

transitions = [
  { from = "pending", to = "confirmed", trigger = "payment_received" },
  { from = "confirmed", to = "shipped", trigger = "warehouse_dispatch" },
  { from = "shipped", to = "delivered", trigger = "customer_signature" },
  { from = "pending", to = "cancelled", trigger = "timeout" },
  { from = "confirmed", to = "cancelled", trigger = "user_cancel" }
]

# Tests that validate each transition
covered = [
  { from = "pending", to = "confirmed", test = "test_order_payment.clnrm.toml" },
  { from = "confirmed", to = "shipped", test = "test_order_shipment.clnrm.toml" }
]

# Calculation: 2/5 = 40% state transition coverage
```

### 3. Error Scenario Coverage

**Metric**: What percentage of error conditions are tested?

```toml
[behavior.error_scenarios]
# Define expected error behaviors
errors = [
  { code = 400, scenario = "invalid_email_format" },
  { code = 401, scenario = "missing_auth_token" },
  { code = 403, scenario = "insufficient_permissions" },
  { code = 404, scenario = "user_not_found" },
  { code = 409, scenario = "duplicate_email" },
  { code = 500, scenario = "database_connection_failed" },
  { code = 503, scenario = "service_unavailable" }
]

# Tests that validate each error
covered = [
  { scenario = "invalid_email_format", test = "test_validation_errors.clnrm.toml" },
  { scenario = "user_not_found", test = "test_404_errors.clnrm.toml" }
]

# Calculation: 2/7 = 28.6% error scenario coverage
```

### 4. Data Flow Coverage

**Metric**: What percentage of data flows through the system are validated?

```toml
[behavior.data_flows]
# Define critical data paths
flows = [
  { name = "user_registration", steps = ["validate", "hash_password", "save_db", "send_email"] },
  { name = "user_login", steps = ["lookup_user", "verify_password", "create_session", "return_token"] },
  { name = "password_reset", steps = ["validate_email", "generate_token", "send_email", "update_db"] }
]

# Tests that validate each flow end-to-end
covered = [
  { flow = "user_registration", test = "test_registration_flow.clnrm.toml" }
]

# Calculation: 1/3 = 33.3% data flow coverage
```

### 5. Integration Point Coverage

**Metric**: What percentage of external integrations are tested?

```toml
[behavior.integrations]
# Define external dependencies
integrations = [
  { name = "postgres", type = "database", operations = ["read", "write", "transaction"] },
  { name = "redis", type = "cache", operations = ["get", "set", "delete", "expire"] },
  { name = "stripe", type = "payment", operations = ["create_charge", "refund"] },
  { name = "sendgrid", type = "email", operations = ["send_email", "send_template"] }
]

# Tests that validate each integration
covered = [
  { integration = "postgres", operations = ["read", "write"], test = "test_database.clnrm.toml" },
  { integration = "redis", operations = ["get", "set"], test = "test_cache.clnrm.toml" }
]

# Calculation: 2/4 integrations tested = 50%
# Operations: 4/9 operations = 44.4%
```

### 6. Span Coverage (OTEL-First Approach)

**Metric**: What percentage of expected spans are observed?

```toml
[behavior.span_coverage]
# Define expected span inventory from production
expected_spans = [
  "http.request",
  "db.query",
  "cache.get",
  "email.send",
  "payment.charge",
  "user.authenticate",
  "order.create",
  "order.ship"
]

# Spans observed in test execution
observed_spans = [
  "http.request",
  "db.query",
  "cache.get",
  "user.authenticate"
]

# Calculation: 4/8 = 50% span coverage
```

## Composite Behavior Coverage Score

### Weighted Formula

```
Behavior Coverage = Σ (dimension_coverage × dimension_weight)

Default weights:
- API Surface: 20%
- State Transitions: 20%
- Error Scenarios: 15%
- Data Flows: 20%
- Integration Points: 15%
- Span Coverage: 10%
```

### Example Calculation

```
API Surface:       42.9% × 0.20 = 8.58%
State Transitions: 40.0% × 0.20 = 8.00%
Error Scenarios:   28.6% × 0.15 = 4.29%
Data Flows:        33.3% × 0.20 = 6.66%
Integrations:      50.0% × 0.15 = 7.50%
Span Coverage:     50.0% × 0.10 = 5.00%
                              ─────────
Total Behavior Coverage:      40.03%
```

## Implementation Strategy

### 1. Behavior Manifest File

Create `behaviors.clnrm.toml` to define complete system behavior inventory:

```toml
[system]
name = "my-api"
version = "1.0.0"

[dimensions.api_surface]
weight = 0.20
endpoints = [
  "GET /users",
  "POST /users",
  # ... complete list
]

[dimensions.state_transitions]
weight = 0.20
[[dimensions.state_transitions.entities]]
name = "Order"
states = ["pending", "confirmed", "shipped"]
transitions = [
  { from = "pending", to = "confirmed" }
]

[dimensions.error_scenarios]
weight = 0.15
scenarios = [
  { code = 400, name = "invalid_input" },
  { code = 404, name = "not_found" }
]

[dimensions.data_flows]
weight = 0.20
flows = [
  { name = "user_registration", steps = ["validate", "save", "notify"] }
]

[dimensions.integrations]
weight = 0.15
services = [
  { name = "postgres", operations = ["read", "write"] },
  { name = "redis", operations = ["get", "set"] }
]

[dimensions.span_coverage]
weight = 0.10
expected_spans = [
  "http.request",
  "db.query"
]
```

### 2. Test Metadata

Add coverage metadata to test TOML files:

```toml
[test.metadata]
name = "test_user_creation"

[test.covers]
api_endpoints = ["POST /users"]
state_transitions = [
  { entity = "User", from = null, to = "active" }
]
error_scenarios = ["invalid_email_format", "duplicate_email"]
data_flows = ["user_registration"]
integrations = [
  { service = "postgres", operations = ["write"] },
  { service = "sendgrid", operations = ["send_email"] }
]
expected_spans = ["http.request", "db.insert", "email.send"]
```

### 3. Coverage Tracking in CleanroomEnvironment

```rust
pub struct BehaviorCoverage {
    pub api_endpoints_covered: HashSet<String>,
    pub state_transitions_covered: HashSet<(String, String, String)>, // (entity, from, to)
    pub error_scenarios_covered: HashSet<String>,
    pub data_flows_covered: HashSet<String>,
    pub integrations_covered: HashMap<String, HashSet<String>>, // service -> operations
    pub spans_observed: HashSet<String>,
}

impl CleanroomEnvironment {
    pub async fn calculate_behavior_coverage(&self) -> Result<BehaviorCoverageReport> {
        // Load behaviors.clnrm.toml
        let manifest = load_behavior_manifest()?;

        // Aggregate coverage from all executed tests
        let coverage = self.aggregate_coverage().await?;

        // Calculate per-dimension coverage
        let api_coverage = coverage.api_endpoints_covered.len() as f64
            / manifest.api_surface.len() as f64;

        // ... calculate other dimensions

        // Calculate weighted total
        let total_coverage =
            api_coverage * manifest.weights.api_surface +
            state_coverage * manifest.weights.state_transitions +
            // ... other dimensions
            ;

        Ok(BehaviorCoverageReport {
            total_coverage,
            dimensions: vec![
                DimensionCoverage { name: "API Surface", coverage: api_coverage },
                // ...
            ],
            uncovered_behaviors: manifest.find_uncovered(&coverage),
        })
    }
}
```

### 4. CLI Commands

```bash
# Initialize behavior manifest
clnrm coverage init

# Analyze behavior coverage
clnrm coverage analyze

# Generate coverage report
clnrm coverage report --format html

# Show uncovered behaviors
clnrm coverage gaps

# Validate manifest completeness
clnrm coverage validate-manifest
```

### 5. Coverage Report Output

```
Behavior Coverage Report
========================

Overall Coverage: 40.03% 🟡

Dimension Breakdown:
┌─────────────────────┬──────────┬─────────┬──────────┐
│ Dimension           │ Coverage │ Weight  │ Score    │
├─────────────────────┼──────────┼─────────┼──────────┤
│ API Surface         │ 42.9%    │ 20%     │ 8.58%    │
│ State Transitions   │ 40.0%    │ 20%     │ 8.00%    │
│ Error Scenarios     │ 28.6%    │ 15%     │ 4.29%    │
│ Data Flows          │ 33.3%    │ 20%     │ 6.66%    │
│ Integration Points  │ 50.0%    │ 15%     │ 7.50%    │
│ Span Coverage       │ 50.0%    │ 10%     │ 5.00%    │
└─────────────────────┴──────────┴─────────┴──────────┘

Top Uncovered Behaviors:
1. DELETE /users/:id (API Surface)
2. Order: confirmed -> cancelled (State Transition)
3. 500: database_connection_failed (Error Scenario)
4. password_reset flow (Data Flow)
5. stripe payment integration (Integration)

Recommendations:
→ Add tests for DELETE operations (4 endpoints uncovered)
→ Add error injection tests for 500-series errors
→ Implement integration tests for payment gateway
```

## Advanced Heuristics

### 1. Behavior Complexity Weighting

Not all behaviors are equal. Weight by complexity:

```toml
[behavior.api_surface.endpoints.complex]
"POST /orders" = { weight = 3.0, reason = "multi-step transaction" }
"GET /users" = { weight = 1.0, reason = "simple read" }

# Weighted coverage: (covered_weight / total_weight)
```

### 2. Criticality Scoring

Mark critical behaviors that MUST be tested:

```toml
[behavior.critical]
must_cover = [
  { type = "api", name = "POST /payments", reason = "financial transaction" },
  { type = "flow", name = "user_authentication", reason = "security critical" }
]

# Fail CI if critical behaviors uncovered
```

### 3. Production Span Discovery

Auto-discover behaviors from production OTEL data:

```bash
# Import production spans to create behavior manifest
clnrm coverage discover --source jaeger --endpoint http://jaeger:16686

# Generates behaviors.clnrm.toml from observed production behavior
```

### 4. Differential Coverage

Track coverage changes between versions:

```bash
# Compare coverage between versions
clnrm coverage diff --baseline v1.0.0 --current v1.1.0

# Output: +5.3% behavior coverage (3 new behaviors covered)
```

## Benefits

1. **Objective Metric**: Quantifiable measure of test quality beyond code coverage
2. **Gap Identification**: Precisely identifies untested behaviors
3. **Prioritization**: Weight critical behaviors higher
4. **Trend Tracking**: Monitor coverage over time
5. **Compliance**: Prove regulatory requirements are tested
6. **OTEL-Native**: Leverages existing observability infrastructure

## Next Steps

1. Implement `BehaviorCoverage` struct in clnrm-core
2. Create `behaviors.clnrm.toml` schema
3. Add `[test.covers]` section to TOML test format
4. Implement `clnrm coverage` CLI commands
5. Create HTML coverage report generator
6. Add CI/CD integration examples

## References

- OTEL Semantic Conventions: https://opentelemetry.io/docs/specs/semconv/
- Test Coverage Metrics: https://martinfowler.com/bliki/TestCoverage.html
- Behavior-Driven Development: https://cucumber.io/docs/bdd/
