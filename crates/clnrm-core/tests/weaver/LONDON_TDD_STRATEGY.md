# London TDD Strategy for Weaver-First Refactor

**Author**: TDD London School Swarm Agent
**Date**: 2025-10-30
**Version**: 1.0.0
**Status**: Ready for Implementation

---

## Executive Summary

This document defines the mock-driven test strategy for clnrm's Weaver-first refactor using London School TDD principles. The strategy prioritizes **interaction testing over state testing**, using mocks derived from OTel semantic convention schemas to verify contracts between components.

**Key Principle**: Mocks represent contracts defined by schemas, not implementation details. Tests verify that components collaborate correctly according to telemetry contracts.

---

## 1. Schema-Driven Contract Analysis

### 1.1 Core Telemetry Contracts

From registry analysis, we identify **4 primary contracts**:

#### Contract 1: Test Execution (test_execution.yaml)

**Producer**: CleanroomEnvironment
**Consumer**: Weaver validation
**Contract Attributes** (REQUIRED):

```rust
pub struct TestExecutionContract {
    test_name: String,                    // REQUIRED
    test_suite: String,                   // REQUIRED
    test_isolated: bool,                  // REQUIRED (must be true)
    test_result: TestResult,              // REQUIRED (pass/fail/error)
    test_duration_ms: f64,                // REQUIRED (must be > 0)
    test_start_timestamp: i64,            // REQUIRED
    test_end_timestamp: i64,              // REQUIRED
    container_id: String,                 // REQUIRED (proves container ran)
    container_image_name: String,         // REQUIRED
    container_exit_code: i32,             // REQUIRED
    test_cleanup_performed: bool,         // REQUIRED (must be true)
}
```

**Validation Rule**: `end_timestamp - start_timestamp == duration_ms`

#### Contract 2: Container Lifecycle (container_lifecycle.yaml)

**Producer**: Backend (TestcontainerBackend)
**Consumer**: Weaver validation
**Contract Attributes** (REQUIRED):

```rust
pub struct ContainerLifecycleContract {
    container_id: String,                 // REQUIRED (PRIMARY KEY)
    container_image: String,              // REQUIRED
    container_state: ContainerState,      // REQUIRED (final must be 'destroyed')
    container_created_at: String,         // REQUIRED (ISO 8601)
    container_started_at: String,         // REQUIRED (ISO 8601)
    container_destroyed_at: String,       // REQUIRED (ISO 8601)
    container_backend: String,            // REQUIRED
    cleanup_success: bool,                // REQUIRED (must be true)
    cleanup_orphaned_resources: i32,      // RECOMMENDED (must be 0)
}
```

**Validation Rule**: `destroyed_at` MUST exist (missing = resource leak)

#### Contract 3: Plugin Execution (plugin_system.yaml)

**Producer**: ServicePlugin implementations
**Consumer**: Weaver validation
**Contract Attributes** (REQUIRED):

```rust
pub struct PluginExecutionContract {
    plugin_name: String,                  // REQUIRED
    plugin_type: String,                  // REQUIRED
    plugin_state: PluginState,            // REQUIRED
    service_name: String,                 // REQUIRED
    service_type: String,                 // REQUIRED
    container_id: String,                 // REQUIRED (links to container)
    plugin_health_check_performed: bool,  // REQUIRED
    plugin_health_check_passed: bool,     // REQUIRED
}
```

**State Transition**: registered → starting → running → healthy → stopping → stopped

#### Contract 4: Test Events (test_events.yaml)

**Producer**: Test execution flow
**Consumer**: Weaver validation
**Contract Events**:

1. `test.started` → `test.completed` (matched by test.name + container.id)
2. `test.started` → `test.failed` (matched by test.name + container.id)
3. `container.leaked` (CRITICAL - should NEVER occur)
4. `isolation.violation` (CRITICAL - should NEVER occur)

**Validation Rule**: Every `started` event must have corresponding `completed` or `failed` event

---

## 2. Mock Design Strategy

### 2.1 London School Principles Applied

**1. Mock External Dependencies, Not Internal Collaborators**

```rust
// ✅ CORRECT - Mock external process (Weaver)
mock_weaver_process
    .expect_start()
    .times(1)
    .returning(|| Ok(WeaverCoordination { ... }));

// ❌ WRONG - Don't mock internal logic
// We don't mock CleanroomEnvironment's private methods
```

**2. Verify Interactions, Not State**

```rust
// ✅ CORRECT - Verify the conversation between components
assert!(mock_backend.create_container.called_with("alpine:latest"));
assert!(mock_weaver.receive_span.called_after(mock_backend.create_container));

// ❌ WRONG - Don't test internal state
// assert!(environment.internal_state == State::Running);
```

**3. Design Mocks from Schemas, Not Implementations**

```rust
// ✅ CORRECT - Mock represents telemetry contract
pub trait TelemetryReceiver {
    fn receive_span(&mut self, span: Span) -> Result<()>;
    fn validate_attributes(&self, required: Vec<&str>) -> Result<()>;
}

// Schema-driven mock
impl TelemetryReceiver for MockWeaver {
    fn receive_span(&mut self, span: Span) -> Result<()> {
        // Validate against schema contract
        self.validate_required_attributes(&span)?;
        self.spans.push(span);
        Ok(())
    }
}
```

### 2.2 Mock Component Design

#### Mock 1: WeaverProcessMock

**Purpose**: Simulate Weaver process lifecycle without spawning actual process
**Contract**: Process management (start/stop/report)

```rust
pub struct WeaverProcessMock {
    pub start_calls: Vec<WeaverConfig>,
    pub stop_calls: usize,
    pub coordination_response: Option<WeaverCoordination>,
    pub report_response: ValidationReport,
    pub failure_mode: Option<FailureMode>,
}

impl WeaverProcessMock {
    pub fn expect_start(&mut self) -> &mut Self {
        // Record expected call
        self
    }

    pub fn with_coordination(&mut self, coord: WeaverCoordination) -> &mut Self {
        self.coordination_response = Some(coord);
        self
    }

    pub fn with_report(&mut self, report: ValidationReport) -> &mut Self {
        self.report_response = report;
        self
    }

    pub fn simulate_failure(&mut self, mode: FailureMode) -> &mut Self {
        self.failure_mode = Some(mode);
        self
    }
}

pub enum FailureMode {
    ProcessCrash,
    PortUnavailable,
    InvalidReport,
    ZeroSamples,
}
```

#### Mock 2: DockerBackendMock

**Purpose**: Simulate Docker operations without actual containers
**Contract**: Container lifecycle (create/start/stop/destroy)

```rust
pub struct DockerBackendMock {
    pub containers: HashMap<String, ContainerMockState>,
    pub create_calls: Vec<String>,  // image names
    pub destroy_calls: Vec<String>, // container IDs
    pub expected_lifecycle: Vec<ContainerState>,
}

pub struct ContainerMockState {
    pub id: String,
    pub image: String,
    pub state: ContainerState,
    pub created_at: String,
    pub started_at: Option<String>,
    pub destroyed_at: Option<String>,
}

impl DockerBackendMock {
    pub fn expect_create(&mut self, image: &str) -> &mut Self {
        self.expected_lifecycle.push(ContainerState::Creating);
        self
    }

    pub fn will_return_container(&mut self, id: &str) -> &mut Self {
        let container = ContainerMockState {
            id: id.to_string(),
            state: ContainerState::Running,
            created_at: chrono::Utc::now().to_rfc3339(),
            ..Default::default()
        };
        self.containers.insert(id.to_string(), container);
        self
    }

    pub fn verify_destroyed(&self, container_id: &str) -> bool {
        self.containers
            .get(container_id)
            .map(|c| c.destroyed_at.is_some())
            .unwrap_or(false)
    }
}
```

#### Mock 3: OTELExporterMock

**Purpose**: Capture telemetry without actual OTLP export
**Contract**: Telemetry collection (spans/metrics/events)

```rust
pub struct OTELExporterMock {
    pub spans: Vec<SpanData>,
    pub metrics: Vec<MetricData>,
    pub events: Vec<EventData>,
    pub export_calls: usize,
}

impl OTELExporterMock {
    pub fn expect_span(&mut self, contract: TestExecutionContract) -> &mut Self {
        // Store expected span attributes from contract
        self
    }

    pub fn verify_contract(&self, contract: &TestExecutionContract) -> Result<()> {
        // Verify received spans match contract requirements
        let span = self.spans.iter().find(|s| s.name == contract.test_name)
            .ok_or_else(|| Error::MissingSpan(contract.test_name.clone()))?;

        // Verify REQUIRED attributes
        assert!(span.attributes.contains_key("test.isolated"));
        assert!(span.attributes.contains_key("container.id"));
        assert!(span.attributes.get("test.duration_ms").unwrap() > 0.0);

        Ok(())
    }

    pub fn find_matching_events(&self, test_name: &str) -> (Option<Event>, Option<Event>) {
        let started = self.events.iter()
            .find(|e| e.name == "test.started" && e.attributes["test.name"] == test_name);
        let completed = self.events.iter()
            .find(|e| e.name == "test.completed" && e.attributes["test.name"] == test_name);

        (started.cloned(), completed.cloned())
    }
}
```

#### Mock 4: PortDiscoveryMock

**Purpose**: Simulate port availability without network binding
**Contract**: Port allocation (find/reserve/release)

```rust
pub struct PortDiscoveryMock {
    pub available_ports: Vec<u16>,
    pub allocated_ports: HashMap<u16, String>, // port -> service
    pub find_calls: usize,
}

impl PortDiscoveryMock {
    pub fn with_available_ports(&mut self, ports: Vec<u16>) -> &mut Self {
        self.available_ports = ports;
        self
    }

    pub fn simulate_exhaustion(&mut self) -> &mut Self {
        self.available_ports.clear();
        self
    }

    pub fn expect_fallback(&mut self, primary: Range<u16>, fallback: Range<u16>) -> &mut Self {
        // Simulate primary range exhausted, fallback succeeds
        self.available_ports = (fallback.start..fallback.end).collect();
        self
    }
}
```

### 2.3 Schema-Driven Test Fixtures

**Fixture Builder Pattern** - Generate test data from schema definitions:

```rust
pub struct ContractFixtures;

impl ContractFixtures {
    /// Generate valid test execution contract from schema
    pub fn valid_test_execution() -> TestExecutionContract {
        TestExecutionContract {
            test_name: "test_container_creation".to_string(),
            test_suite: "integration_tests".to_string(),
            test_isolated: true,  // Schema REQUIRES true
            test_result: TestResult::Pass,
            test_duration_ms: 125.5,  // Schema REQUIRES > 0
            test_start_timestamp: 1730250000000,
            test_end_timestamp: 1730250125500,
            container_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            container_image_name: "alpine:latest".to_string(),
            container_exit_code: 0,
            test_cleanup_performed: true,  // Schema REQUIRES true
        }
    }

    /// Generate INVALID contract (missing required attributes)
    pub fn invalid_test_execution_missing_container_id() -> TestExecutionContract {
        let mut contract = Self::valid_test_execution();
        contract.container_id = "".to_string();  // VIOLATES schema
        contract
    }

    /// Generate INVALID contract (test.isolated = false)
    pub fn invalid_test_execution_not_isolated() -> TestExecutionContract {
        let mut contract = Self::valid_test_execution();
        contract.test_isolated = false;  // VIOLATES schema
        contract
    }

    /// Generate container lifecycle with resource leak
    pub fn container_lifecycle_leaked() -> ContainerLifecycleContract {
        ContainerLifecycleContract {
            container_id: "leaked-container-123".to_string(),
            container_image: "postgres:15".to_string(),
            container_state: ContainerState::Running,  // Never destroyed!
            container_created_at: "2025-10-30T14:23:45.123Z".to_string(),
            container_started_at: "2025-10-30T14:23:46.456Z".to_string(),
            container_destroyed_at: "".to_string(),  // MISSING - VIOLATES schema
            container_backend: "testcontainers".to_string(),
            cleanup_success: false,  // VIOLATES schema
            cleanup_orphaned_resources: 1,  // Should be 0
        }
    }

    /// Generate plugin execution with health check failure
    pub fn plugin_execution_unhealthy() -> PluginExecutionContract {
        PluginExecutionContract {
            plugin_name: "postgres".to_string(),
            plugin_type: "database".to_string(),
            plugin_state: PluginState::Error,
            service_name: "test_postgres".to_string(),
            service_type: "database".to_string(),
            container_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            plugin_health_check_performed: true,
            plugin_health_check_passed: false,  // Health check FAILED
        }
    }
}
```

---

## 3. Test Structure by Phase

### Phase 1: WeaverController Lifecycle Tests

**Focus**: Process management interactions
**Mocks**: WeaverProcessMock
**Pattern**: Arrange (mock setup) → Act (controller method) → Assert (verify interactions)

```
tests/weaver/
├── phase1_weaver_lifecycle/
│   ├── mod.rs
│   ├── test_startup.rs           # WeaverController::start_and_coordinate
│   ├── test_shutdown.rs          # WeaverController::stop_and_report
│   ├── test_port_discovery.rs    # Port allocation with fallback
│   ├── test_coordination.rs      # Coordination metadata flow
│   ├── test_health_check.rs      # Wait for ready timeout
│   └── test_failure_modes.rs     # Process crash, port conflicts
```

**Example Test Structure**:

```rust
// test_startup.rs
#[test]
fn test_weaver_startup_discovers_ports_and_coordinates() {
    // ARRANGE - Setup mocks
    let mut mock_process = WeaverProcessMock::new();
    mock_process
        .expect_start()
        .with_coordination(WeaverCoordination {
            weaver_pid: 12345,
            otlp_grpc_port: 4317,
            admin_port: 8080,
            ready_at: Instant::now(),
        });

    let config = WeaverConfig::default();
    let mut controller = WeaverController::with_mock(config, mock_process);

    // ACT - Execute coordination
    let coord = controller.start_and_coordinate()
        .expect("Coordination should succeed");

    // ASSERT - Verify interactions
    assert_eq!(coord.otlp_grpc_port, 4317);
    assert_eq!(mock_process.start_calls.len(), 1);
    assert!(controller.coordination().is_some());
}
```

### Phase 2: Coordination Pattern Tests

**Focus**: Weaver-first initialization order
**Mocks**: WeaverProcessMock + OTELExporterMock
**Pattern**: Verify correct sequencing of Weaver → OTEL → Tests

```
tests/weaver/
├── phase2_coordination/
│   ├── mod.rs
│   ├── test_weaver_first_order.rs     # Weaver starts before OTEL
│   ├── test_port_handoff.rs           # Weaver port → OTEL config
│   ├── test_ready_wait.rs             # OTEL waits for Weaver ready
│   ├── test_shutdown_order.rs         # OTEL flush → Weaver stop
│   └── test_orphan_cleanup.rs         # Old process cleanup
```

**Example Test Structure**:

```rust
// test_weaver_first_order.rs
#[test]
fn test_weaver_must_start_before_otel_initialization() {
    // ARRANGE
    let mut mock_weaver = WeaverProcessMock::new();
    let mut mock_otel = OTELExporterMock::new();

    let start_order = Arc::new(Mutex::new(Vec::new()));
    let order_clone = Arc::clone(&start_order);

    mock_weaver
        .expect_start()
        .with_callback(move || {
            order_clone.lock().unwrap().push("weaver_start");
        });

    let order_clone2 = Arc::clone(&start_order);
    mock_otel
        .expect_init()
        .with_callback(move || {
            order_clone2.lock().unwrap().push("otel_init");
        });

    // ACT
    let coord = controller.start_and_coordinate().unwrap();
    let _otel_guard = init_otel_with_mock(coord.otlp_grpc_port, mock_otel);

    // ASSERT - Verify Weaver started BEFORE OTEL
    let order = start_order.lock().unwrap();
    assert_eq!(order[0], "weaver_start");
    assert_eq!(order[1], "otel_init");
}
```

### Phase 3: OTEL Integration Tests

**Focus**: Telemetry contract verification
**Mocks**: OTELExporterMock + DockerBackendMock
**Pattern**: Execute test → Capture telemetry → Verify against schema contracts

```
tests/weaver/
├── phase3_otel_integration/
│   ├── mod.rs
│   ├── test_contract_test_execution.rs      # Verify test_execution schema
│   ├── test_contract_container_lifecycle.rs # Verify container_lifecycle schema
│   ├── test_contract_plugin_execution.rs    # Verify plugin_system schema
│   ├── test_contract_events.rs              # Verify test_events schema
│   ├── test_zero_samples_detection.rs       # Catch no-telemetry false positives
│   └── test_attribute_validation.rs         # Required vs optional attributes
```

**Example Test Structure**:

```rust
// test_contract_test_execution.rs
#[test]
fn test_execution_emits_all_required_attributes() {
    // ARRANGE - Setup mocks
    let mut mock_otel = OTELExporterMock::new();
    let mut mock_backend = DockerBackendMock::new();
    mock_backend
        .will_return_container("test-container-123");

    let expected_contract = ContractFixtures::valid_test_execution();

    // ACT - Execute test
    let env = CleanroomEnvironment::with_mocks(mock_backend, mock_otel.clone());
    env.run_test("test_container_creation").await?;

    // ASSERT - Verify telemetry matches contract
    mock_otel.verify_contract(&expected_contract)?;

    // CRITICAL ASSERTIONS from schema
    let span = mock_otel.find_span("test_container_creation").unwrap();
    assert!(span.attributes["test.isolated"] == true);  // REQUIRED
    assert!(span.attributes["container.id"] != "");     // REQUIRED
    assert!(span.attributes["test.duration_ms"] > 0.0); // REQUIRED > 0
    assert!(span.attributes["test.cleanup_performed"] == true); // REQUIRED
}
```

### Phase 4: End-to-End Docker Validation

**Focus**: Full integration with real Docker containers
**Mocks**: None (integration tests)
**Pattern**: Weaver → OTEL → Real Docker → Validate telemetry

```
tests/weaver/
├── phase4_e2e_docker/
│   ├── mod.rs
│   ├── test_docker_container_lifecycle.rs   # Full Docker lifecycle
│   ├── test_docker_plugin_execution.rs      # Real service plugins
│   ├── test_docker_isolation_proof.rs       # Hermetic isolation validation
│   ├── test_docker_cleanup_verification.rs  # Resource leak detection
│   └── test_docker_weaver_validation.rs     # Weaver validates real Docker telemetry
```

**Example Test Structure**:

```rust
// test_docker_weaver_validation.rs
#[tokio::test]
#[ignore = "Requires Docker and Weaver installation"]
async fn test_weaver_validates_real_docker_container_creation() {
    // ARRANGE - Start REAL Weaver
    let config = WeaverConfig::default();
    let mut controller = WeaverController::new(config);
    let coord = controller.start_and_coordinate()
        .expect("Weaver should start");

    // Initialize REAL OTEL pointing to Weaver
    let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
    let _otel_guard = init_otel(OtelConfig {
        export: Export::OtlpGrpc { endpoint: &endpoint },
        ..Default::default()
    })?;

    // ACT - Run REAL test with REAL Docker
    let env = CleanroomEnvironment::new().await?;
    env.run_test_scenario("tests/basic.clnrm.toml").await?;

    // Flush telemetry
    drop(_otel_guard);
    std::thread::sleep(Duration::from_millis(500));

    // ASSERT - Get Weaver validation report
    let report = controller.stop_and_report()?;

    // CRITICAL: Weaver MUST receive telemetry
    assert!(report.sample_count > 0, "Weaver received ZERO samples - validation is invalid!");

    // CRITICAL: No schema violations
    assert_eq!(report.violations, 0, "Weaver detected {} schema violations", report.violations);
    assert_eq!(report.status, ValidationStatus::Success);
}
```

---

## 4. Integration Test Patterns

### 4.1 The "London Double Loop" Pattern

**Outer Loop**: Integration test with real components
**Inner Loop**: Unit tests with mocks

```rust
// Outer loop - Integration test
#[tokio::test]
async fn integration_full_weaver_lifecycle() {
    let controller = WeaverController::new(WeaverConfig::default());
    let coord = controller.start_and_coordinate()?;
    // ... real execution ...
    let report = controller.stop_and_report()?;
    assert_eq!(report.violations, 0);
}

// Inner loop - Unit tests with mocks (fast feedback)
#[test]
fn unit_weaver_controller_handles_process_crash() {
    let mut mock = WeaverProcessMock::new();
    mock.simulate_failure(FailureMode::ProcessCrash);

    let controller = WeaverController::with_mock(WeaverConfig::default(), mock);
    let result = controller.start_and_coordinate();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("prematurely"));
}
```

### 4.2 Contract Verification Pattern

**Test that components honor telemetry contracts**:

```rust
#[test]
fn verify_container_lifecycle_contract_honored() {
    // ARRANGE - Create contract expectation from schema
    let expected_contract = ContractFixtures::valid_container_lifecycle();

    let mut mock_otel = OTELExporterMock::new();
    let backend = TestcontainerBackend::with_otel(mock_otel.clone());

    // ACT - Execute container lifecycle
    let container = backend.create_container("alpine:latest")?;
    backend.start_container(&container)?;
    backend.destroy_container(&container)?;

    // ASSERT - Verify telemetry matches contract
    let span = mock_otel.find_span("container_lifecycle").unwrap();

    // Schema REQUIRED attributes
    assert!(span.attributes.contains_key("container.created_at"));
    assert!(span.attributes.contains_key("container.started_at"));
    assert!(span.attributes.contains_key("container.destroyed_at"));

    // Schema VALIDATION rule
    assert_eq!(span.attributes["cleanup.success"], true);
    assert_eq!(span.attributes["container.state"], "destroyed");
}
```

### 4.3 Zero-Sample Detection Pattern

**Prevent false positives from missing telemetry**:

```rust
#[test]
fn test_weaver_report_fails_on_zero_samples() {
    // ARRANGE - Mock Weaver that receives NO telemetry
    let mut mock_weaver = WeaverProcessMock::new();
    mock_weaver.with_report(ValidationReport {
        sample_count: 0,  // CRITICAL: Zero samples
        status: ValidationStatus::Success,  // Should be overridden
        violations: 0,
        ..Default::default()
    });

    // ACT
    let mut controller = WeaverController::with_mock(WeaverConfig::default(), mock_weaver);
    controller.start_and_coordinate()?;
    let report = controller.stop_and_report()?;

    // ASSERT - Zero samples MUST result in failure
    assert_eq!(report.status, ValidationStatus::Failure);
    // Controller should override status when sample_count == 0
}
```

---

## 5. Mock Lifecycle Management

### 5.1 Mock Cleanup Strategy

**Ensure mocks don't leak state between tests**:

```rust
impl Drop for WeaverProcessMock {
    fn drop(&mut self) {
        // Verify all expectations were met
        if !self.verify_expectations() {
            panic!("WeaverProcessMock: Unfulfilled expectations");
        }
    }
}

impl WeaverProcessMock {
    fn verify_expectations(&self) -> bool {
        // Check that expected calls were made
        self.expected_starts == self.start_calls.len()
    }
}
```

### 5.2 Mock State Isolation

**Each test gets fresh mock instances**:

```rust
pub struct TestFixture {
    pub weaver_mock: WeaverProcessMock,
    pub backend_mock: DockerBackendMock,
    pub otel_mock: OTELExporterMock,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            weaver_mock: WeaverProcessMock::new(),
            backend_mock: DockerBackendMock::new(),
            otel_mock: OTELExporterMock::new(),
        }
    }
}

#[test]
fn test_with_fresh_fixture() {
    let fixture = TestFixture::new();  // Fresh mocks every test
    // ...
}
```

---

## 6. Test Organization

### Directory Structure

```
crates/clnrm-core/tests/weaver/
├── LONDON_TDD_STRATEGY.md           # This document
├── mod.rs                            # Module declarations
├── fixtures/                         # Schema-driven test fixtures
│   ├── mod.rs
│   ├── contracts.rs                  # Contract builders from schemas
│   └── builders.rs                   # Test data builders
├── mocks/                            # Mock implementations
│   ├── mod.rs
│   ├── weaver_process.rs             # WeaverProcessMock
│   ├── docker_backend.rs             # DockerBackendMock
│   ├── otel_exporter.rs              # OTELExporterMock
│   └── port_discovery.rs             # PortDiscoveryMock
├── phase1_weaver_lifecycle/          # Phase 1 tests
│   ├── mod.rs
│   ├── test_startup.rs
│   ├── test_shutdown.rs
│   ├── test_port_discovery.rs
│   ├── test_coordination.rs
│   ├── test_health_check.rs
│   └── test_failure_modes.rs
├── phase2_coordination/              # Phase 2 tests
│   ├── mod.rs
│   ├── test_weaver_first_order.rs
│   ├── test_port_handoff.rs
│   ├── test_ready_wait.rs
│   ├── test_shutdown_order.rs
│   └── test_orphan_cleanup.rs
├── phase3_otel_integration/          # Phase 3 tests
│   ├── mod.rs
│   ├── test_contract_test_execution.rs
│   ├── test_contract_container_lifecycle.rs
│   ├── test_contract_plugin_execution.rs
│   ├── test_contract_events.rs
│   ├── test_zero_samples_detection.rs
│   └── test_attribute_validation.rs
└── phase4_e2e_docker/                # Phase 4 tests
    ├── mod.rs
    ├── test_docker_container_lifecycle.rs
    ├── test_docker_plugin_execution.rs
    ├── test_docker_isolation_proof.rs
    ├── test_docker_cleanup_verification.rs
    └── test_docker_weaver_validation.rs
```

### Test Naming Convention

**Pattern**: `test_<component>_<behavior>_<outcome>`

Examples:
- `test_weaver_startup_discovers_ports_and_coordinates`
- `test_weaver_shutdown_emits_validation_report`
- `test_docker_backend_creates_container_with_lifecycle_span`
- `test_otel_exporter_validates_required_attributes`

---

## 7. Success Criteria

### Phase 1 Success Criteria
- ✅ All WeaverController lifecycle methods have unit tests with mocks
- ✅ Port discovery with fallback fully tested
- ✅ Process crash scenarios handled gracefully
- ✅ Health check timeout tested
- ✅ Coordination metadata verified

### Phase 2 Success Criteria
- ✅ Weaver-first initialization order verified
- ✅ Port handoff from Weaver → OTEL tested
- ✅ Orphan process cleanup verified
- ✅ Shutdown order (OTEL flush → Weaver stop) enforced

### Phase 3 Success Criteria
- ✅ All 4 schema contracts have verification tests
- ✅ Required vs optional attributes distinguished
- ✅ Zero-sample detection prevents false positives
- ✅ Event pairing (started → completed) validated

### Phase 4 Success Criteria
- ✅ Real Docker containers emit correct telemetry
- ✅ Weaver validates real Docker lifecycle
- ✅ Resource leaks detected via missing `destroyed_at`
- ✅ Hermetic isolation proven via telemetry

---

## 8. Anti-Patterns to Avoid

### ❌ Anti-Pattern 1: Testing Implementation Details

```rust
// ❌ WRONG - Testing internal state
#[test]
fn test_controller_internal_state() {
    let controller = WeaverController::new(config);
    assert_eq!(controller.internal_flag, false);  // BAD!
}

// ✅ CORRECT - Testing observable behavior
#[test]
fn test_controller_coordination_provides_ports() {
    let controller = WeaverController::new(config);
    let coord = controller.start_and_coordinate()?;
    assert!(coord.otlp_grpc_port > 0);  // Observable contract
}
```

### ❌ Anti-Pattern 2: Mocking Everything

```rust
// ❌ WRONG - Over-mocking kills integration confidence
let mock_env = MockCleanroomEnvironment::new();
let mock_backend = MockBackend::new();
let mock_plugin = MockPlugin::new();
// Too many mocks = testing mock interactions, not real system

// ✅ CORRECT - Mock external boundaries only
let mock_weaver = WeaverProcessMock::new();  // External process
let real_backend = TestcontainerBackend::new();  // Real implementation
```

### ❌ Anti-Pattern 3: State-Based Assertions on Mocks

```rust
// ❌ WRONG - Checking mock internal state
assert_eq!(mock.internal_counter, 3);

// ✅ CORRECT - Verifying interactions
assert_eq!(mock.start_calls.len(), 1);
assert!(mock.stop_calls > 0);
```

### ❌ Anti-Pattern 4: Ignoring Schema Contracts

```rust
// ❌ WRONG - Testing without schema validation
#[test]
fn test_container_lifecycle() {
    backend.create_container("alpine")?;
    backend.destroy_container("alpine")?;
    // No verification of telemetry contract!
}

// ✅ CORRECT - Validate against schema contract
#[test]
fn test_container_lifecycle_honors_schema() {
    let expected = ContractFixtures::valid_container_lifecycle();
    backend.create_container("alpine")?;
    backend.destroy_container("alpine")?;
    mock_otel.verify_contract(&expected)?;
}
```

---

## 9. Next Steps

### Implementation Order

1. **Create Mock Infrastructure** (`mocks/` directory)
   - WeaverProcessMock
   - DockerBackendMock
   - OTELExporterMock
   - PortDiscoveryMock

2. **Create Fixtures** (`fixtures/` directory)
   - Contract builders from schemas
   - Valid/invalid contract generators
   - Test data builders

3. **Implement Phase 1 Tests** (WeaverController lifecycle)
   - Start with startup tests
   - Add shutdown and cleanup tests
   - Verify port discovery
   - Test failure modes

4. **Implement Phase 2 Tests** (Coordination patterns)
   - Weaver-first ordering
   - Port handoff verification
   - Shutdown sequence

5. **Implement Phase 3 Tests** (OTEL integration)
   - Contract verification tests
   - Zero-sample detection
   - Attribute validation

6. **Implement Phase 4 Tests** (E2E Docker validation)
   - Real Docker + Weaver integration
   - Full lifecycle validation
   - Resource leak detection

### Integration with CI/CD

```yaml
# .github/workflows/weaver-tests.yml
- name: Run Phase 1-3 (Unit + Integration)
  run: cargo test --test weaver -- --skip phase4

- name: Run Phase 4 (E2E with Docker)
  run: cargo test --test weaver phase4_e2e_docker
  env:
    DOCKER_AVAILABLE: "true"
    WEAVER_INSTALLED: "true"
```

---

## 10. Conclusion

This London TDD strategy provides a **contract-driven, mock-first approach** to testing clnrm's Weaver integration. By deriving mocks from OTel semantic convention schemas, we ensure tests verify actual telemetry contracts rather than implementation details.

**Key Principles**:
1. Mocks represent schemas, not implementations
2. Verify interactions, not state
3. Four-phase approach: lifecycle → coordination → integration → E2E
4. Zero-sample detection prevents false positives
5. Contract fixtures ensure schema compliance

This strategy enables **rapid TDD iteration** with fast mock-based tests while maintaining **high confidence** through schema-driven contracts and E2E Docker validation.

---

**Status**: Ready for Implementation
**Next Agent**: Coder Agent (implement mocks and Phase 1 tests)
**Coordination**: Store strategy in swarm memory for all agents
