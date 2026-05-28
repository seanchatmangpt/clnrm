# Gall Testing Specification (GTS)

## 1. Philosophy & Definition

**Gall's Law:** *"A complex system that works is invariably found to have evolved from a simple system that worked."*

**Gall Testing** is an integration testing methodology derived from this principle. Unlike traditional end-to-end (E2E) tests that attempt to validate a complex, fully integrated architecture (which are brittle and prone to cascading failures), and unlike unit tests (which often mock away critical behavioral reality), a **Gall Test** validates the **highest-order simple systems**.

A Gall Test proves that a foundational subsystem (a primitive building block) functions flawlessly in isolation under real-world constraints. By mathematically proving that the foundational blocks work, the probability of the complex aggregate failing due to structural instability drops near zero.

## 2. Criteria for a Gall Subsystem

To qualify for a Gall Test, a subsystem must meet the following criteria:

1.  **High Cohesion, Low Coupling:** The subsystem must be instantiable without bringing up the entire framework. It should not require the master `CleanroomEnvironment` or a running daemon to validate its core logic.
2.  **State Transformation:** It must take a raw input, process it, and yield a deterministic output (e.g., Raw String -> Parsed Config, or Resource Request -> Acquired Lock).
3.  **Foundational Criticality:** If this subsystem fails, the entire framework is guaranteed to fail or act unpredictably.
4.  **No Mocks for Internal State:** The test must execute the actual production code path for that subsystem. Mocks are only permitted for external boundaries (like network calls or daemon APIs).

## 3. The Anatomy of a Gall Test

Gall Tests should be organized into a dedicated suite (e.g., `tests/gall/`) to differentiate them from granular unit tests and brittle E2E tests.

A standard Gall Test follows the **A.I.M.** structure:

*   **A - Isolate (Arrange):** Instantiate the simple subsystem completely severed from the master orchestrator. Provide it with strict, realistic configurations.
*   **I - Ignite (Act):** Push the subsystem to its boundary limits. Feed it edge-case data, concurrent requests, or malformed inputs that a user might provide.
*   **M - Measure (Assert):** Validate not just the happy path, but the *failure modes*. A Gall Test must prove that the subsystem fails gracefully and securely when abused.

## 4. Identified Gall Subsystems in `clnrm`

Based on the architectural audit, the following subsystems require immediate Gall Testing:

| Subsystem | Target | Validation Goal |
| :--- | :--- | :--- |
| **Configuration Parsing Engine** | `crates/clnrm-core/src/config/` | Prove TOML deserialization, fallback logic, and strict schema validation work without booting a container. |
| **Template Variable Engine** | `crates/clnrm-template/` | Prove `TemplateRenderer` perfectly replaces variables and handles missing keys deterministically. |
| **OTEL Semantic Conventions** | `crates/clnrm-core/src/telemetry/` | Prove `SpanBuilder` generates exact `KeyValue` matches for the `weaver` schema without needing an OTLP endpoint. |
| **Deterministic Port Allocator** | `crates/clnrm-core/src/telemetry/live_check/` | Prove 100 concurrent threads requesting ports do not receive duplicates and locks are freed. |
| **Service Plugin Registry** | `crates/clnrm-core/src/cleanroom.rs` | Prove state transitions (Registered -> Active -> Stopped) function correctly in isolated memory. |

## 5. Implementation Standard

All Gall Tests must adhere to the following standards:

1.  **Zero Flakiness:** Because they test isolated logic without external daemons, a Gall Test must pass 10,000 times out of 10,000 runs.
2.  **Sub-100ms Execution:** Gall tests must be lightning fast.
3.  **Error Signal Clarity:** If a Gall Test fails, it should immediately pinpoint the exact line of code in the subsystem that broke, preventing hours of debugging in E2E suites.

---
*By enforcing Gall Testing, we ensure the framework is built on concrete, not sand.*