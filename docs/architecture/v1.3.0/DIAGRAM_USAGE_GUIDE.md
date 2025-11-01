# C4 Diagram Usage Guide - clnrm v1.3.0

## Quick Start

### Generate All Diagrams
```bash
# Install PlantUML (requires Java)
brew install plantuml

# Generate PNG diagrams
plantuml /tmp/v1.3.0-c4-diagrams.puml

# Generates 7 diagrams in current directory:
# 1. clnrm-v1.3.0-system-context.png
# 2. clnrm-v1.3.0-container.png
# 3. clnrm-v1.3.0-telemetry-components.png
# 4. clnrm-v1.3.0-state-machine.png
# 5. clnrm-v1.3.0-test-execution-sequence.png
# 6. clnrm-v1.3.0-cicd-deployment.png
# 7. clnrm-v1.3.0-production-deployment.png
```

### Alternative: Generate SVG (Vector Format)
```bash
plantuml -tsvg /tmp/v1.3.0-c4-diagrams.puml
```

### Alternative: View in PlantUML Online
1. Copy contents of `/tmp/v1.3.0-c4-diagrams.puml`
2. Paste into https://www.plantuml.com/plantuml/uml/
3. View diagrams interactively

---

## Diagram Reference

### 1. System Context Diagram
**File:** `clnrm-v1.3.0-system-context.png`
**Use When:** Explaining clnrm to stakeholders, new team members, or executives

**Shows:**
- clnrm in the ecosystem (developers, CI/CD, SRE)
- External systems (Weaver, Docker, OTLP collectors, GitHub)
- Critical relationships and data flows

**Key Insight:** Weaver is the single source of truth for telemetry correctness

**Audience:** All stakeholders (technical and non-technical)

---

### 2. Container Diagram
**File:** `clnrm-v1.3.0-container.png`
**Use When:** Designing new features, onboarding developers, architecture reviews

**Shows:**
- Internal architecture (CLI, Core, Config, Telemetry, Backend, Plugins)
- Technology choices (Rust, TOML, OpenTelemetry SDK, testcontainers-rs)
- Component responsibilities and interactions

**Key Insight:** Clear separation of concerns enables extensibility

**Audience:** Developers, architects, technical leads

---

### 3. Component Diagram (Telemetry Layer)
**File:** `clnrm-v1.3.0-telemetry-components.png`
**Use When:** Debugging Weaver integration, implementing validation features

**Shows:**
- Detailed telemetry layer components
- WeaverController, WeaverProcessManager, LiveCheckOrchestrator
- Port discovery, health checks, validation engine
- State management and OTLP integration

**Key Insight:** Multi-tier port discovery supports 40 concurrent processes

**Audience:** Core team developers, SRE, performance engineers

---

### 4. State Machine Diagram
**File:** `clnrm-v1.3.0-state-machine.png`
**Use When:** Debugging startup/shutdown issues, adding error handling

**Shows:**
- LiveCheckOrchestrator state transitions
- States: Uninitialized → Starting → WeaverRunning → Stopping → Completed
- Error paths: PortExhaust, HealthTimeout, ProcessCrash
- Recovery strategies

**Key Insight:** Graceful degradation with fallback to force-kill

**Audience:** Core team developers, troubleshooting

---

### 5. Sequence Diagram (Test Execution)
**File:** `clnrm-v1.3.0-test-execution-sequence.png`
**Use When:** Understanding end-to-end flow, onboarding, debugging

**Shows:**
- Complete test execution flow from CLI to validation
- Weaver-first pattern (start Weaver → init OTEL → run tests → stop Weaver)
- Interactions between CLI, Core, Weaver, OTEL, Docker
- Timing of health checks, telemetry export, report generation

**Key Insight:** OTEL is flushed BEFORE stopping Weaver to ensure telemetry delivery

**Audience:** All developers, especially new team members

---

### 6. Deployment Diagram (CI/CD)
**File:** `clnrm-v1.3.0-cicd-deployment.png`
**Use When:** Setting up CI/CD pipelines, GitHub Actions workflows

**Shows:**
- GitHub Actions runner architecture
- Installation steps (Rust, Weaver, Docker)
- Test execution environment (clnrm, Weaver, test containers)
- Artifact upload on failure
- Observability backend export (Jaeger, DataDog, Grafana)

**Key Insight:** Validation reports uploaded as artifacts for debugging

**Audience:** DevOps, SRE, CI/CD engineers

---

### 7. Deployment Diagram (Production)
**File:** `clnrm-v1.3.0-production-deployment.png`
**Use When:** Designing production validation, continuous testing strategy

**Shows:**
- Kubernetes deployment architecture
- Validation namespace (clnrm, Weaver)
- Application namespace (API, database, cache)
- Observability namespace (OTEL collector, Jaeger, Prometheus)
- External SaaS export (DataDog, New Relic, Honeycomb)

**Key Insight:** Production telemetry MUST conform to schemas (validated in real-time)

**Audience:** SRE, platform engineers, architects

---

## Common Use Cases

### Use Case 1: Onboarding New Developer
**Diagrams to Share:**
1. System Context → "Here's where clnrm fits in the ecosystem"
2. Sequence Diagram → "Here's how a test executes end-to-end"
3. Container Diagram → "Here's the internal architecture you'll work with"

**Key Messages:**
- Tests can lie; telemetry schemas don't
- Weaver validation is the source of truth
- Zero samples = failed test (prevents false positives)

---

### Use Case 2: Debugging Weaver Integration Issue
**Diagrams to Use:**
1. Component Diagram → "Understand telemetry layer components"
2. State Machine → "Identify which state transition is failing"
3. Sequence Diagram → "Verify timing of operations (OTEL flush, health check, etc.)"

**Key Areas to Check:**
- Port discovery (multi-tier fallback working?)
- Health check (timeout? process crash?)
- OTLP export (endpoint correct? telemetry reaching Weaver?)
- Report parsing (JSON format valid? sample_count > 0?)

---

### Use Case 3: Setting Up CI/CD Pipeline
**Diagrams to Use:**
1. Deployment Diagram (CI/CD) → "Here's the target architecture"
2. Sequence Diagram → "Here's the execution flow to implement"

**Key Steps:**
1. Install Rust toolchain
2. Install Weaver (`cargo install weaver-cli`)
3. Run tests with validation (`clnrm run tests/ --validate`)
4. Upload artifacts on failure (`validation_output/`)
5. Render GitHub annotations (optional)

---

### Use Case 4: Production Deployment Design
**Diagrams to Use:**
1. Deployment Diagram (Production) → "Here's the Kubernetes architecture"
2. Component Diagram → "Here's how Weaver coordinates"

**Key Decisions:**
- Namespace isolation (validation vs application)
- Observability backend (internal vs external SaaS)
- Validation schedule (5 minutes? 15 minutes?)
- Alerting strategy (PagerDuty? Slack?)

---

### Use Case 5: Architecture Review
**Diagrams to Present:**
1. System Context → "Here's the business context"
2. Container Diagram → "Here's the internal architecture"
3. Component Diagram → "Here's the critical telemetry layer"
4. Deployment Diagram (Production) → "Here's how we deploy"

**Key Talking Points:**
- Validation hierarchy (Weaver > Compilation > Tests)
- Multi-tier port discovery (40 concurrent processes)
- Zero-sample prevention (no false positives)
- Type-safe builders (compile-time schema enforcement)

---

## Diagram Maintenance

### When to Update Diagrams

| Change | Diagrams to Update | Priority |
|--------|-------------------|----------|
| New external system integration | System Context | High |
| New internal container/crate | Container Diagram | High |
| New telemetry component | Component Diagram | Medium |
| New state in orchestrator | State Machine | Medium |
| New deployment environment | Deployment Diagram | High |
| Bug fix (no architecture change) | None | N/A |

### Update Process
1. Edit `/tmp/v1.3.0-c4-diagrams.puml` (source of truth)
2. Regenerate diagrams: `plantuml /tmp/v1.3.0-c4-diagrams.puml`
3. Commit both `.puml` source and `.png` outputs
4. Update documentation to reference new diagrams

---

## Advanced Usage

### Customize Diagram Style
```plantuml
' Add at top of .puml file
skinparam backgroundColor #EEEBDC
skinparam handwritten true
```

### Export to PDF
```bash
plantuml -tpdf /tmp/v1.3.0-c4-diagrams.puml
```

### Include in Markdown
```markdown
![System Context](./clnrm-v1.3.0-system-context.png)
```

### Include in mdBook
```markdown
# Architecture

![System Context](../diagrams/clnrm-v1.3.0-system-context.png)
```

---

## Troubleshooting

### Issue: PlantUML Not Generating Diagrams
**Solution:**
```bash
# Check Java installed
java -version

# If not installed
brew install openjdk@11

# Retry PlantUML
plantuml -version
```

### Issue: Diagrams Look Different Than Expected
**Solution:**
- Ensure using official C4-PlantUML stdlib
- Check PlantUML version (`plantuml -version`)
- Try online editor: https://www.plantuml.com/plantuml/uml/

### Issue: Can't View .puml Files in Editor
**Solution:**
- Install PlantUML plugin for VS Code, IntelliJ, or Vim
- Use online editor as fallback

---

## Resources

- **PlantUML:** https://plantuml.com/
- **C4 Model:** https://c4model.com/
- **C4-PlantUML:** https://github.com/plantuml-stdlib/C4-PlantUML
- **clnrm Documentation:** `docs/`

---

**Last Updated:** 2025-10-31
**Diagram Version:** v1.3.0
**Maintainer:** Architecture Evaluator #12
