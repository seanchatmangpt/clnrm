# clnrm v1.0.1 Architecture - C4 Diagrams

## Overview

This document provides comprehensive C4 architecture diagrams for the clnrm (Cleanroom Testing Framework) v1.0.1. The diagrams span four levels of abstraction:

1. **System Context** - How clnrm fits in the broader ecosystem
2. **Container** - High-level technology choices and system decomposition
3. **Component** - Internal structure of key containers
4. **Code** - Key classes, traits, and data structures

clnrm is a hermetic integration testing framework built in Rust that uses Docker containers for test isolation, supports declarative TOML test definitions, and includes comprehensive OpenTelemetry observability. The framework "eats its own dog food" - it tests itself using its own capabilities.

## Diagram Index

### Level 1: System Context (5 diagrams)
1. [Overall System Context](#diagram-1-overall-system-context)
2. [User Interactions](#diagram-2-user-interactions)
3. [External Systems](#diagram-3-external-systems)
4. [Integration Points](#diagram-4-integration-points)
5. [Deployment Context](#diagram-5-deployment-context)

### Level 2: Container Diagrams (5 diagrams)
6. [Workspace Architecture](#diagram-6-workspace-architecture)
7. [Core Library Containers](#diagram-7-core-library-containers)
8. [CLI Application](#diagram-8-cli-application)
9. [Plugin System](#diagram-9-plugin-system)
10. [Observability Stack](#diagram-10-observability-stack)

### Level 3: Component Diagrams (10 diagrams)
11. [CleanroomEnvironment Components](#diagram-11-cleanroomenvironment-components)
12. [Service Plugin Architecture](#diagram-12-service-plugin-architecture)
13. [Backend Abstraction](#diagram-13-backend-abstraction)
14. [Configuration Loading](#diagram-14-configuration-loading)
15. [Template Processing](#diagram-15-template-processing)
16. [OTEL Integration](#diagram-16-otel-integration)
17. [Validation Framework](#diagram-17-validation-framework)
18. [CLI Command Router](#diagram-18-cli-command-router)
19. [Test Execution Pipeline](#diagram-19-test-execution-pipeline)
20. [Metrics Collection](#diagram-20-metrics-collection)

### Level 4: Code/Class Diagrams (5 diagrams)
21. [Core Traits](#diagram-21-core-traits)
22. [Error Hierarchy](#diagram-22-error-hierarchy)
23. [Data Models](#diagram-23-data-models)
24. [State Management](#diagram-24-state-management)
25. [Execution Flow](#diagram-25-execution-flow)

---

## Level 1: System Context

### Diagram 1: Overall System Context

Shows clnrm in the broader testing ecosystem, including developers, CI/CD systems, and external dependencies.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml

LAYOUT_WITH_LEGEND()

title System Context - clnrm Hermetic Testing Framework v1.0.1

Person(developer, "Developer", "Software engineer writing integration tests")
Person(sre, "SRE/DevOps", "Platform engineer managing test infrastructure")
Person(qa, "QA Engineer", "Quality assurance testing production scenarios")

System(clnrm, "clnrm Framework", "Hermetic integration testing platform with container isolation, declarative TOML tests, and OpenTelemetry observability")

System_Ext(docker, "Docker/Podman", "Container runtime for test isolation")
System_Ext(registry, "Container Registry", "Docker Hub, GHCR, ECR for container images")
System_Ext(otel_collector, "OTEL Collector", "Jaeger, DataDog, New Relic for telemetry")
System_Ext(ci_system, "CI/CD System", "GitHub Actions, GitLab CI, Jenkins")
System_Ext(vcs, "Version Control", "Git repositories with test configurations")

Rel(developer, clnrm, "Writes tests with", "TOML configs, CLI")
Rel(sre, clnrm, "Monitors and deploys", "CI/CD pipelines")
Rel(qa, clnrm, "Validates scenarios", "Test suites")

Rel(clnrm, docker, "Creates isolated containers", "testcontainers-rs")
Rel(clnrm, registry, "Pulls service images", "Docker Registry API")
Rel(clnrm, otel_collector, "Exports telemetry", "OTLP HTTP/gRPC")
Rel(ci_system, clnrm, "Executes test runs", "CLI commands")
Rel(clnrm, vcs, "Reads test configs", ".clnrm.toml files")

note right of clnrm
  **Key Capabilities:**
  - Hermetic test isolation
  - Plugin-based services
  - Declarative TOML tests
  - OTEL observability
  - Self-testing framework
  **Technology:**
  - Rust 2021 Edition
  - Tokio async runtime
  - testcontainers-rs
end note

@enduml
```

**Description**: This diagram shows clnrm as the central system that coordinates hermetic integration testing. Three types of users (developers, SREs, QA engineers) interact with clnrm through its CLI and TOML configuration files. The framework orchestrates Docker containers for test isolation, pulls images from registries, and exports comprehensive telemetry to OTEL collectors. CI/CD systems invoke clnrm for automated testing.

---

### Diagram 2: User Interactions

Focuses on the different user personas and their specific interactions with clnrm.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml

LAYOUT_TOP_DOWN()

title User Interactions - clnrm Framework v1.0.1

Person(app_dev, "Application Developer", "Writes integration tests for features")
Person(platform_eng, "Platform Engineer", "Maintains test infrastructure")
Person(qa_auto, "QA Automation", "Creates end-to-end test scenarios")
Person(ops, "Operations Team", "Monitors test health and performance")

System_Boundary(clnrm_system, "clnrm Testing Platform") {
    System(cli, "CLI Interface", "Command-line tool for test execution")
    System(config, "Configuration System", "TOML-based test definitions")
    System(plugins, "Plugin System", "Extensible service integrations")
    System(reports, "Reporting Engine", "JUnit, JSON, TAP outputs")
}

Rel(app_dev, cli, "Runs tests locally", "clnrm run tests/")
Rel(app_dev, config, "Defines tests", ".clnrm.toml")
Rel(platform_eng, plugins, "Develops custom plugins", "ServicePlugin trait")
Rel(platform_eng, cli, "Manages infrastructure", "clnrm services")
Rel(qa_auto, config, "Authors test scenarios", "TOML scenarios")
Rel(qa_auto, reports, "Analyzes results", "JUnit XML, JSON")
Rel(ops, cli, "Health checks", "clnrm health")
Rel(ops, reports, "Performance metrics", "Metrics API")

note right of app_dev
  **Developer Workflow:**
  1. clnrm init (setup)
  2. Write .clnrm.toml tests
  3. clnrm run tests/
  4. View results
end note

note left of platform_eng
  **Platform Tasks:**
  - Custom plugin development
  - CI/CD integration
  - Resource optimization
  - Security policies
end note

@enduml
```

**Description**: This diagram details how different user personas interact with clnrm's subsystems. Application developers use the CLI and configuration system for daily testing. Platform engineers extend the framework through plugins and manage infrastructure. QA automation engineers author complex scenarios, while operations teams monitor health and performance metrics.

---

### Diagram 3: External Systems

Shows all external systems and services that clnrm integrates with.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml

LAYOUT_WITH_LEGEND()

title External Systems Integration - clnrm v1.0.1

System(clnrm, "clnrm Framework", "Hermetic testing platform")

System_Boundary(container_runtime, "Container Runtimes") {
    System_Ext(docker, "Docker Engine", "Container execution")
    System_Ext(podman, "Podman", "Daemonless containers")
}

System_Boundary(registries, "Container Registries") {
    System_Ext(dockerhub, "Docker Hub", "Public container images")
    System_Ext(ghcr, "GitHub Container Registry", "Private images")
    System_Ext(ecr, "AWS ECR", "Enterprise registry")
}

System_Boundary(observability, "Observability Platforms") {
    System_Ext(jaeger, "Jaeger", "Distributed tracing")
    System_Ext(datadog, "DataDog", "APM and metrics")
    System_Ext(newrelic, "New Relic", "Full-stack observability")
    System_Ext(prometheus, "Prometheus", "Metrics collection")
}

System_Boundary(ci_cd, "CI/CD Platforms") {
    System_Ext(github_actions, "GitHub Actions", "Automated workflows")
    System_Ext(gitlab_ci, "GitLab CI", "Pipeline automation")
    System_Ext(jenkins, "Jenkins", "Build automation")
}

System_Boundary(databases, "Test Databases") {
    System_Ext(postgres, "PostgreSQL", "Relational database")
    System_Ext(redis, "Redis", "Cache and queue")
    System_Ext(surrealdb, "SurrealDB", "Multi-model database")
}

Rel(clnrm, docker, "Creates/manages containers", "testcontainers-rs")
Rel(clnrm, podman, "Alternative runtime", "Compatible API")
Rel(clnrm, dockerhub, "Pulls base images", "Registry API v2")
Rel(clnrm, ghcr, "Pulls private images", "OCI standard")
Rel(clnrm, ecr, "Enterprise images", "AWS SDK")
Rel(clnrm, jaeger, "Exports traces", "OTLP gRPC")
Rel(clnrm, datadog, "Sends telemetry", "OTLP HTTP")
Rel(clnrm, newrelic, "APM data", "OTLP")
Rel(clnrm, prometheus, "Metrics endpoint", "/metrics")
Rel(github_actions, clnrm, "Executes tests", "clnrm run")
Rel(gitlab_ci, clnrm, "Pipeline stage", "CLI")
Rel(jenkins, clnrm, "Build step", "Shell")
Rel(clnrm, postgres, "Test database", "GenericContainerPlugin")
Rel(clnrm, redis, "Cache service", "Plugin")
Rel(clnrm, surrealdb, "Native plugin", "SurrealDbPlugin")

note bottom of clnrm
  **Integration Patterns:**
  - Container API via testcontainers-rs
  - OTLP for all telemetry exports
  - OCI standard for registries
  - Exit codes for CI/CD
  - JUnit XML for reporting
end note

@enduml
```

**Description**: This comprehensive view shows all external systems clnrm integrates with. Container runtimes (Docker, Podman) provide isolation. Multiple registries supply container images. Observability platforms (Jaeger, DataDog, New Relic, Prometheus) receive telemetry via OTLP. CI/CD systems execute tests through the CLI. Test databases run as containerized services through plugins.

---

### Diagram 4: Integration Points

Focuses on technical integration mechanisms and protocols.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml

LAYOUT_TOP_DOWN()

title Integration Points & Protocols - clnrm v1.0.1

System(clnrm_core, "clnrm Core", "Test orchestration")

System_Boundary(protocols, "Integration Protocols") {
    Component(docker_api, "Docker API", "Container management", "REST + Unix socket")
    Component(otlp_http, "OTLP HTTP", "Telemetry export", "Protocol Buffers")
    Component(otlp_grpc, "OTLP gRPC", "Telemetry stream", "gRPC")
    Component(registry_api, "Registry API v2", "Image pulls", "Docker Registry HTTP API")
}

System_Ext(docker_daemon, "Docker Daemon", "Container runtime")
System_Ext(otel_collector, "OTEL Collector", "Telemetry receiver")
System_Ext(registry, "Container Registry", "Image storage")
System_Ext(github, "GitHub Actions", "CI/CD runner")

Rel(clnrm_core, docker_api, "testcontainers-rs", "Rust SDK")
Rel(docker_api, docker_daemon, "HTTP + Unix socket", "/var/run/docker.sock")
Rel(clnrm_core, otlp_http, "opentelemetry-otlp", "HTTP POST")
Rel(clnrm_core, otlp_grpc, "opentelemetry-otlp", "gRPC stream")
Rel(otlp_http, otel_collector, "Export spans/metrics", ":4318/v1/traces")
Rel(otlp_grpc, otel_collector, "Export spans/metrics", ":4317")
Rel(clnrm_core, registry_api, "reqwest", "HTTPS")
Rel(registry_api, registry, "Pull manifests/layers", "Token auth")
Rel(github, clnrm_core, "Shell execution", "Exit codes + stdout")

note right of docker_api
  **testcontainers-rs 0.25**
  - Blocking + async APIs
  - Container lifecycle
  - Port mapping
  - Volume mounts
  - Network creation
end note

note right of otlp_http
  **opentelemetry 0.31.0**
  - Traces, metrics, logs
  - Batch export
  - Retry logic
  - Compression support
end note

note bottom of clnrm_core
  **Output Formats:**
  - JUnit XML (CI/CD)
  - JSON (programmatic)
  - TAP (Test Anything Protocol)
  - Human-readable (console)
end note

@enduml
```

**Description**: This diagram details the technical protocols and APIs used for integrations. Docker API communication via testcontainers-rs, OTLP HTTP/gRPC for telemetry export, Docker Registry API v2 for image pulls, and standard CLI interfaces for CI/CD. Each integration uses industry-standard protocols with proper authentication and error handling.

---

### Diagram 5: Deployment Context

Shows how clnrm is deployed across different environments.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Deployment.puml

LAYOUT_WITH_LEGEND()

title Deployment Context - clnrm v1.0.1

Deployment_Node(dev_machine, "Developer Workstation", "macOS/Linux/Windows") {
    Deployment_Node(dev_runtime, "Local Runtime", "Docker Desktop") {
        Container(dev_clnrm, "clnrm CLI", "Rust binary", "Homebrew installed")
        Container(dev_containers, "Test Containers", "Docker", "Isolated test services")
    }
    Deployment_Node(dev_otel, "Local OTEL", "Docker Compose") {
        Container(dev_jaeger, "Jaeger UI", "Docker", "Trace visualization")
    }
}

Deployment_Node(ci_runner, "CI/CD Runner", "GitHub Actions, GitLab Runner") {
    Deployment_Node(ci_container, "Runner Container", "Ubuntu 22.04") {
        Container(ci_clnrm, "clnrm Binary", "Rust", "Cached binary")
        Container(ci_tests, "Test Containers", "Docker-in-Docker", "Ephemeral services")
    }
}

Deployment_Node(production, "Production Environment", "Kubernetes Cluster") {
    Deployment_Node(test_pod, "Test Pod", "Kubernetes Pod") {
        Container(prod_clnrm, "clnrm", "Container", "Scheduled test runs")
        Container(prod_services, "Test Services", "Sidecar containers", "Service mesh")
    }
    Deployment_Node(obs_stack, "Observability", "Kubernetes Services") {
        ContainerDb(otel_collector, "OTEL Collector", "Kubernetes Deployment", "Centralized telemetry")
        ContainerDb(prometheus, "Prometheus", "StatefulSet", "Metrics storage")
        Container(grafana, "Grafana", "Deployment", "Dashboards")
    }
}

Rel(dev_clnrm, dev_containers, "Creates/manages", "Docker API")
Rel(dev_clnrm, dev_jaeger, "Exports traces", "OTLP HTTP :4318")
Rel(ci_clnrm, ci_tests, "Orchestrates", "testcontainers")
Rel(prod_clnrm, prod_services, "Tests", "Service mesh")
Rel(prod_clnrm, otel_collector, "Telemetry", "OTLP gRPC :4317")
Rel(otel_collector, prometheus, "Metrics", "Remote write")

note right of dev_machine
  **Local Development:**
  - brew install clnrm
  - clnrm init && clnrm run
  - Instant feedback loop
  - Full observability locally
end note

note right of ci_runner
  **CI/CD Execution:**
  - Automated on PR/merge
  - Parallel test execution
  - JUnit XML reporting
  - Docker layer caching
end note

note right of production
  **Production Testing:**
  - Smoke tests post-deploy
  - Chaos engineering
  - Canary validation
  - SLO monitoring
end note

@enduml
```

**Description**: This deployment diagram shows clnrm across three environments. On developer workstations, it's installed via Homebrew and uses Docker Desktop for local testing. In CI/CD, it runs in containerized runners with Docker-in-Docker for test isolation. In production Kubernetes clusters, it runs as scheduled pods with sidecar services, exporting telemetry to centralized OTEL collectors.

---

## Level 2: Container Diagrams

### Diagram 6: Workspace Architecture

Shows the Cargo workspace structure with all 4 crates and their relationships.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

LAYOUT_WITH_LEGEND()

title Cargo Workspace Architecture - clnrm v1.0.1

System_Boundary(workspace, "clnrm Workspace") {
    Container(clnrm_bin, "clnrm", "Rust binary crate", "CLI application with command routing and user interaction")

    Container(clnrm_core, "clnrm-core", "Rust library crate", "Core framework: CleanroomEnvironment, plugins, OTEL, validation, configuration")

    Container(clnrm_shared, "clnrm-shared", "Rust library crate", "Shared utilities and common types across all crates")

    Container(clnrm_ai, "clnrm-ai", "Rust library crate", "EXPERIMENTAL: AI-powered test generation and optimization (isolated)")
}

ContainerDb(cargo_toml, "Cargo.toml", "Workspace config", "Version 1.0.1, Edition 2021")

Rel(clnrm_bin, clnrm_core, "Uses", "clnrm_core::*")
Rel(clnrm_bin, clnrm_shared, "Uses", "Shared utilities")
Rel(clnrm_core, clnrm_shared, "Uses", "Common types")
Rel(clnrm_ai, clnrm_core, "Extends", "Optional features")

Rel_R(cargo_toml, clnrm_bin, "Builds", "Default member")
Rel_R(cargo_toml, clnrm_core, "Builds", "Default member")
Rel_R(cargo_toml, clnrm_shared, "Builds", "Default member")
Rel_D(cargo_toml, clnrm_ai, "Excluded", "Not in default-members")

note right of clnrm_core
  **Production-Ready Core:**
  - 120+ source files
  - Comprehensive test suite
  - OpenTelemetry integration
  - Plugin architecture
  - Zero unwrap/expect
  - AAA test pattern
end note

note right of clnrm_ai
  **Experimental Isolation:**
  - Not built by default
  - cargo build (excludes AI)
  - cargo build -p clnrm-ai (explicit)
  - Prevents experimental code
    from affecting production
end note

note bottom of workspace
  **Workspace Benefits:**
  - Unified dependency management
  - Shared build artifacts
  - Consistent versioning
  - Modular architecture
  - Clear separation of concerns
end note

@enduml
```

**Description**: The Cargo workspace consists of 4 crates. The `clnrm` binary crate provides the CLI. `clnrm-core` is the production-ready library with all framework functionality. `clnrm-shared` contains common utilities. `clnrm-ai` is intentionally excluded from default builds to isolate experimental features. The workspace uses resolver = "2" and maintains unified versioning at 1.0.1.

---

### Diagram 7: Core Library Containers

Breaks down the internal structure of the clnrm-core library.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

LAYOUT_TOP_DOWN()

title clnrm-core Library Structure - v1.0.1

System_Boundary(clnrm_core, "clnrm-core Library") {
    Container(cleanroom, "CleanroomEnvironment", "Core module", "Test orchestration, service registry, lifecycle management")

    Container(backend, "Backend Abstraction", "Trait + Implementations", "TestcontainerBackend, MockBackend for fast testing")

    Container(services, "Service Plugins", "Plugin system", "GenericContainer, SurrealDB, Ollama, vLLM, TGI, Chaos plugins")

    Container(config, "Configuration System", "TOML parser", "TestConfig, StepConfig, ServiceConfig with validation")

    Container(template, "Template Engine", "Tera integration", "Macro library, deterministic rendering, context management")

    Container(telemetry, "Telemetry Module", "OpenTelemetry", "Traces, metrics, logs with OTLP export")

    Container(validation, "Validation Framework", "OTEL validators", "Span, trace, count, graph, hermeticity validators")

    Container(reporting, "Reporting Engine", "Multi-format", "JUnit XML, JSON, TAP, human-readable outputs")

    Container(cli_impl, "CLI Implementation", "Command handlers", "Run, init, health, plugins, self-test, validate commands")

    Container(error, "Error System", "Result types", "CleanroomError with structured context and error chaining")
}

Rel(cleanroom, backend, "Uses", "Container operations")
Rel(cleanroom, services, "Manages", "Plugin lifecycle")
Rel(cleanroom, telemetry, "Instruments", "Spans + metrics")
Rel(config, template, "Renders", "TOML templates")
Rel(cli_impl, cleanroom, "Orchestrates", "Test execution")
Rel(cli_impl, config, "Loads", "TOML files")
Rel(cli_impl, reporting, "Generates", "Test reports")
Rel(validation, telemetry, "Validates", "OTEL data")
Rel(services, backend, "Executes via", "Container API")
Rel_U(error, cleanroom, "Returns", "Result<T>")
Rel_U(error, services, "Propagates", "Errors")
Rel_U(error, config, "Validation errors", "Result<T>")

note right of cleanroom
  **Core Engine:**
  - ServiceRegistry
  - CleanroomEnvironment
  - ServiceHandle
  - HealthStatus
  - Arc<RwLock> for concurrency
end note

note left of telemetry
  **OTEL Features:**
  - otel-traces
  - otel-metrics
  - otel-logs
  - otel-otlp
  - otel-stdout
end note

@enduml
```

**Description**: The clnrm-core library is organized into 10 major containers. CleanroomEnvironment is the central orchestrator. Backend abstraction enables both real containers (testcontainers) and mock backends. Service plugins provide extensibility. Configuration handles TOML parsing. Template engine supports macro libraries. Telemetry integrates OTEL. Validation ensures test correctness. Reporting generates multiple output formats. CLI implementation handles commands. Error system provides structured error handling throughout.

---

### Diagram 8: CLI Application

Details the clnrm binary's architecture and command structure.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

LAYOUT_WITH_LEGEND()

title clnrm CLI Application - v1.0.1

System_Boundary(cli_app, "clnrm CLI Binary") {
    Container(main, "main.rs", "Entry point", "Initializes telemetry and runs CLI router")

    Container(cli_router, "CLI Router", "clap-based", "Parses commands and routes to handlers")

    Container_Boundary(commands, "Command Handlers") {
        Component(run_cmd, "run", "Test execution", "Executes tests from TOML files")
        Component(init_cmd, "init", "Initialization", "Creates .clnrm.toml scaffold")
        Component(health_cmd, "health", "Health check", "Validates Docker and dependencies")
        Component(self_test_cmd, "self-test", "Framework validation", "Tests framework with framework")
        Component(plugins_cmd, "plugins", "Plugin listing", "Shows available service plugins")
        Component(validate_cmd, "validate", "Config validation", "Validates TOML syntax")
        Component(report_cmd, "report", "Report generation", "Generates HTML/JSON reports")
        Component(services_cmd, "services", "Service management", "Lists and manages services")
        Component(template_cmd, "template", "Template operations", "Renders and validates templates")
    }

    Container(cli_telemetry, "CLI Telemetry", "CliTelemetry", "Optional OTEL for CLI observability")
}

System_Ext(clnrm_core_lib, "clnrm-core", "Core library")

Rel(main, cli_telemetry, "Initializes", "from_env()")
Rel(main, cli_router, "Starts", "run_cli_with_telemetry()")
Rel(cli_router, run_cmd, "Routes", "clnrm run")
Rel(cli_router, init_cmd, "Routes", "clnrm init")
Rel(cli_router, health_cmd, "Routes", "clnrm health")
Rel(cli_router, self_test_cmd, "Routes", "clnrm self-test")
Rel(cli_router, plugins_cmd, "Routes", "clnrm plugins")
Rel(cli_router, validate_cmd, "Routes", "clnrm validate")
Rel(cli_router, report_cmd, "Routes", "clnrm report")
Rel(cli_router, services_cmd, "Routes", "clnrm services")
Rel(cli_router, template_cmd, "Routes", "clnrm template")

Rel(run_cmd, clnrm_core_lib, "Uses", "CleanroomEnvironment")
Rel(init_cmd, clnrm_core_lib, "Uses", "Config templates")
Rel(health_cmd, clnrm_core_lib, "Checks", "Docker connection")
Rel(self_test_cmd, clnrm_core_lib, "Tests", "Framework capabilities")
Rel(validate_cmd, clnrm_core_lib, "Validates", "TOML parsing")

note right of main
  **CLI Features:**
  - Zero unwrap/expect
  - Proper error handling
  - Optional telemetry
  - Exit code propagation
  - Colored output
end note

note bottom of commands
  **Command Patterns:**
  - All async handlers
  - Result<()> returns
  - Structured logging
  - Progress indicators
  - Human-friendly errors
end note

@enduml
```

**Description**: The CLI application uses clap for command parsing with 9 main commands. The main.rs entry point initializes optional telemetry and starts the CLI router. Each command handler is async, returns Result<()>, and delegates to clnrm-core for functionality. Commands include run (test execution), init (scaffolding), health (Docker validation), self-test (dogfooding), plugins (discovery), validate (TOML checks), report (output generation), services (management), and template (rendering).

---

### Diagram 9: Plugin System

Shows the plugin architecture and available service plugins.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

LAYOUT_WITH_LEGEND()

title Service Plugin System - clnrm v1.0.1

System_Boundary(plugin_system, "Plugin Architecture") {
    Container(service_plugin_trait, "ServicePlugin Trait", "Core abstraction", "Defines start(), stop(), health_check(), name() - all sync methods")

    Container(service_registry, "ServiceRegistry", "Plugin manager", "Registers and manages plugin instances")

    Container_Boundary(core_plugins, "Built-in Plugins") {
        Component(generic_plugin, "GenericContainerPlugin", "Any Docker image", "Universal container support")
        Component(surrealdb_plugin, "SurrealDbPlugin", "SurrealDB", "Multi-model database")
        Component(ollama_plugin, "OllamaPlugin", "Ollama", "Local LLM inference")
        Component(vllm_plugin, "VllmPlugin", "vLLM", "Fast LLM serving")
        Component(tgi_plugin, "TgiPlugin", "Text Generation Inference", "HuggingFace TGI")
        Component(chaos_plugin, "ChaosEnginePlugin", "Chaos engineering", "Failure injection")
        Component(otel_plugin, "OtelCollectorPlugin", "OTEL Collector", "Telemetry aggregation")
    }

    Container(service_factory, "ServiceFactory", "Plugin discovery", "Creates plugins from configuration")

    Container(service_handle, "ServiceHandle", "Instance handle", "Manages running service instances")
}

Rel(service_registry, service_plugin_trait, "Manages", "Box<dyn ServicePlugin>")
Rel(generic_plugin, service_plugin_trait, "Implements", "Trait methods")
Rel(surrealdb_plugin, service_plugin_trait, "Implements", "Native support")
Rel(ollama_plugin, service_plugin_trait, "Implements", "LLM proxy")
Rel(vllm_plugin, service_plugin_trait, "Implements", "LLM proxy")
Rel(tgi_plugin, service_plugin_trait, "Implements", "LLM proxy")
Rel(chaos_plugin, service_plugin_trait, "Implements", "Chaos ops")
Rel(otel_plugin, service_plugin_trait, "Implements", "Observability")
Rel(service_factory, core_plugins, "Creates", "Plugin instances")
Rel(service_registry, service_handle, "Returns", "Active services")

note right of service_plugin_trait
  **Critical Design:**
  - All methods MUST be sync
  - Maintains dyn compatibility
  - Use tokio::task::block_in_place
    for async operations internally
  - Send + Sync + Debug bounds
end note

note bottom of core_plugins
  **Plugin Capabilities:**
  - Start/stop lifecycle
  - Health monitoring
  - Port mapping
  - Environment variables
  - Volume mounts
  - Network configuration
end note

note left of service_factory
  **Discovery:**
  - TOML [services] section
  - Dynamic instantiation
  - Configuration validation
  - Error propagation
end note

@enduml
```

**Description**: The plugin system centers on the ServicePlugin trait with sync methods for dyn compatibility. ServiceRegistry manages plugin instances. Seven built-in plugins support various services: GenericContainerPlugin for any Docker image, SurrealDbPlugin with native support, three LLM inference plugins (Ollama, vLLM, TGI), ChaosEnginePlugin for failure injection, and OtelCollectorPlugin for telemetry. ServiceFactory creates plugins from TOML configuration. ServiceHandle tracks running instances.

---

### Diagram 10: Observability Stack

Details the comprehensive OpenTelemetry integration.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

LAYOUT_TOP_DOWN()

title Observability Stack - OpenTelemetry Integration v1.0.1

System_Boundary(observability, "clnrm Observability") {
    Container(telemetry_init, "Telemetry Initialization", "init.rs", "TelemetryBuilder, TelemetryHandle, provider setup")

    Container(telemetry_config, "Telemetry Config", "config.rs", "TelemetryConfig, ExporterConfig, sampling, resources")

    Container_Boundary(signal_types, "Signal Types") {
        Component(traces, "Traces", "Spans", "Distributed tracing with parent-child relationships")
        Component(metrics, "Metrics", "Counters/Gauges", "Test duration, success rate, resource usage")
        Component(logs, "Logs", "Log records", "Structured logging with context")
    }

    Container_Boundary(exporters, "Exporters") {
        Component(otlp_http, "OTLP HTTP", "HTTP/Protobuf", "Port 4318 for Jaeger, DataDog, etc")
        Component(otlp_grpc, "OTLP gRPC", "gRPC stream", "Port 4317 for high-throughput")
        Component(stdout_exp, "Stdout Exporter", "Console", "Development debugging")
        Component(jaeger_exp, "Jaeger Exporter", "Legacy", "Direct Jaeger protocol")
        Component(zipkin_exp, "Zipkin Exporter", "HTTP", "Zipkin-compatible tracing")
    }

    Container(validation_system, "OTEL Validation", "validation/otel.rs", "OtelValidator, span/trace assertions")

    Container(stdout_parser, "Stdout Parser", "otel/stdout_parser.rs", "Parses OTEL stdout for testing")
}

System_Ext(otel_collector, "OTEL Collector", "Telemetry aggregator")
System_Ext(jaeger, "Jaeger", "Tracing backend")
System_Ext(prometheus, "Prometheus", "Metrics storage")

Rel(telemetry_init, telemetry_config, "Loads", "Configuration")
Rel(telemetry_init, traces, "Creates", "TracerProvider")
Rel(telemetry_init, metrics, "Creates", "MeterProvider")
Rel(telemetry_init, logs, "Creates", "LoggerProvider")

Rel(traces, otlp_http, "Exports via", "Batch processor")
Rel(traces, otlp_grpc, "Exports via", "Stream")
Rel(traces, stdout_exp, "Exports via", "Console")
Rel(traces, jaeger_exp, "Exports via", "UDP")
Rel(traces, zipkin_exp, "Exports via", "HTTP")

Rel(metrics, otlp_http, "Exports via", "Periodic reader")
Rel(metrics, prometheus, "Scrapes from", "/metrics endpoint")

Rel(validation_system, traces, "Validates", "Span assertions")
Rel(validation_system, stdout_parser, "Uses", "Test validation")

Rel(otlp_http, otel_collector, "Sends to", ":4318")
Rel(otlp_grpc, otel_collector, "Sends to", ":4317")
Rel(jaeger_exp, jaeger, "Direct send", ":14268")

note right of telemetry_init
  **Feature Flags:**
  - otel-traces
  - otel-metrics
  - otel-logs
  - otel-otlp
  - otel-stdout
  All optional and composable
end note

note left of validation_system
  **Validation Features:**
  - SpanAssertion (name, attrs)
  - TraceAssertion (completeness)
  - Count validators
  - Graph validators
  - Hermeticity checks
end note

note bottom of observability
  **Key Principles:**
  - Zero performance impact when disabled
  - Proper resource shutdown
  - Configurable sampling
  - Batch export optimization
  - Error resilience
end note

@enduml
```

**Description**: The observability stack provides comprehensive OpenTelemetry integration with traces, metrics, and logs. TelemetryBuilder initializes providers based on feature flags. Five exporter types support various backends: OTLP HTTP/gRPC for modern systems, stdout for development, and legacy Jaeger/Zipkin protocols. The validation system ensures telemetry correctness through assertions. Stdout parser enables testing. All features are optional via cargo flags, ensuring zero overhead when disabled.

---

## Level 3: Component Diagrams

### Diagram 11: CleanroomEnvironment Components

Internal structure of the core orchestration engine.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_WITH_LEGEND()

title CleanroomEnvironment Internal Components - v1.0.1

Container_Boundary(cleanroom_env, "CleanroomEnvironment") {
    Component(environment_struct, "CleanroomEnvironment", "Main struct", "Orchestrates test execution and service lifecycle")

    Component(service_registry_comp, "ServiceRegistry", "Plugin manager", "HashMap<String, Box<dyn ServicePlugin>>")

    Component(active_services, "Active Services", "Runtime state", "HashMap<String, ServiceHandle>")

    Component(backend_field, "Backend", "Container runtime", "Arc<dyn Backend> - TestcontainerBackend or MockBackend")

    Component(test_context, "Test Context", "Execution state", "Current test metadata, timing, resources")

    Component(lifecycle_mgr, "Lifecycle Manager", "Service control", "start_service(), stop_service(), cleanup()")

    Component(execution_engine, "Execution Engine", "Command runner", "execute_command(), run_step(), collect_results()")

    Component(health_checker, "Health Checker", "Service monitoring", "Periodic health checks for active services")

    Component(telemetry_ctx, "Telemetry Context", "OTEL integration", "Span creation, attribute setting, tracing")
}

Component(config_loader, "Configuration", "External", "TOML test definitions")
Component(plugins, "Service Plugins", "External", "Registered plugin implementations")
Component(backend_impl, "Backend Impl", "External", "Testcontainer or Mock")

Rel(environment_struct, service_registry_comp, "Contains", "Arc<RwLock<ServiceRegistry>>")
Rel(environment_struct, active_services, "Maintains", "Arc<RwLock<HashMap>>")
Rel(environment_struct, backend_field, "Uses", "Arc<dyn Backend>")
Rel(environment_struct, test_context, "Tracks", "Current state")
Rel(environment_struct, telemetry_ctx, "Instruments", "Spans + metrics")

Rel(lifecycle_mgr, service_registry_comp, "Queries", "Get plugin")
Rel(lifecycle_mgr, plugins, "Invokes", "start()/stop()")
Rel(lifecycle_mgr, active_services, "Updates", "Add/remove handles")

Rel(execution_engine, backend_impl, "Executes", "Container commands")
Rel(execution_engine, active_services, "Targets", "Service containers")
Rel(execution_engine, telemetry_ctx, "Records", "Execution traces")

Rel(health_checker, active_services, "Monitors", "Health status")
Rel(health_checker, plugins, "Calls", "health_check()")

Rel(config_loader, environment_struct, "Initializes", "Test configuration")

note right of environment_struct
  **Concurrency Model:**
  - Arc<RwLock<>> for shared state
  - Async methods for I/O
  - Tokio runtime integration
  - Graceful shutdown
end note

note left of lifecycle_mgr
  **Hermetic Isolation:**
  - Each test gets fresh instance
  - Services start on-demand
  - Automatic cleanup on drop
  - No cross-test pollution
end note

@enduml
```

**Description**: CleanroomEnvironment is the core orchestration engine. It maintains a ServiceRegistry of plugins and Active Services for running instances. The Backend field abstracts container operations. Test Context tracks execution state. Lifecycle Manager handles service start/stop operations. Execution Engine runs commands in containers. Health Checker monitors service status. Telemetry Context instruments all operations. The design ensures hermetic isolation with Arc<RwLock<>> for safe concurrency.

---

### Diagram 12: Service Plugin Architecture

Details the plugin lifecycle and extension mechanism.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_TOP_DOWN()

title Service Plugin Architecture - Lifecycle & Extension v1.0.1

Container_Boundary(plugin_arch, "Plugin System") {
    Component(plugin_trait, "ServicePlugin Trait", "Core abstraction", "fn start() -> Result<ServiceHandle>\nfn stop(handle) -> Result<()>\nfn health_check(&handle) -> HealthStatus\nfn name() -> &str")

    Component(plugin_lifecycle, "Plugin Lifecycle", "State machine", "Registered → Started → Running → Stopped → Cleaned")

    Component_Boundary(plugin_impls, "Plugin Implementations") {
        Component(generic_impl, "GenericContainerPlugin", "Universal", "image: String, env_vars, ports, volumes")
        Component(db_impl, "SurrealDbPlugin", "Database", "Native SurrealDB support with connection pooling")
        Component(llm_impl, "LLM Plugins", "AI services", "Ollama, vLLM, TGI with model configuration")
    }

    Component(plugin_config, "Plugin Configuration", "TOML", "[services.name]\ntype = \"generic\"\nimage = \"alpine\"")

    Component(plugin_factory, "Plugin Factory", "Instantiation", "Creates plugins from config with validation")

    Component(plugin_registry, "Plugin Registry", "Storage", "HashMap<String, Box<dyn ServicePlugin>>")

    Component(readiness_probe, "Readiness Probe", "Health", "TCP/HTTP/Command-based health checks")
}

Component(backend_api, "Backend API", "External", "Container operations")
Component(docker, "Docker", "External", "Container runtime")

Rel(plugin_factory, plugin_config, "Reads", "TOML parsing")
Rel(plugin_factory, plugin_impls, "Instantiates", "Plugin creation")
Rel(plugin_factory, plugin_registry, "Registers", "Add to registry")

Rel(plugin_trait, plugin_lifecycle, "Defines", "State transitions")
Rel(generic_impl, plugin_trait, "Implements", "Trait methods")
Rel(db_impl, plugin_trait, "Implements", "Specialized DB")
Rel(llm_impl, plugin_trait, "Implements", "AI services")

Rel(plugin_impls, backend_api, "Uses", "Container operations")
Rel(backend_api, docker, "Calls", "Docker API")

Rel(readiness_probe, plugin_impls, "Checks", "Service health")
Rel(plugin_registry, plugin_trait, "Stores", "Box<dyn ServicePlugin>")

note right of plugin_trait
  **Critical: Sync Methods**
  pub trait ServicePlugin:
    Send + Sync + Debug {

    fn start(&self) -> Result<Handle>;
    // NOT async - maintains dyn

    fn stop(&self, handle) -> Result<()>;
    // Use block_in_place internally
  }
end note

note left of plugin_lifecycle
  **Lifecycle States:**
  1. Registered (in registry)
  2. Started (container created)
  3. Running (health checks pass)
  4. Stopped (graceful shutdown)
  5. Cleaned (resources freed)
end note

note bottom of plugin_impls
  **Extension Pattern:**
  1. Implement ServicePlugin
  2. Add to services/mod.rs
  3. Register in ServiceRegistry
  4. Configure in TOML
  5. Automatic discovery
end note

@enduml
```

**Description**: The plugin architecture uses the ServicePlugin trait with sync methods for dyn compatibility. Plugin Lifecycle tracks state transitions from registration to cleanup. Three implementation categories exist: GenericContainerPlugin for any image, specialized database plugins like SurrealDB, and LLM service plugins. Plugin Factory creates instances from TOML configuration. Plugin Registry stores Box<dyn ServicePlugin>. Readiness Probe ensures services are healthy. The pattern enables easy extension while maintaining type safety.

---

### Diagram 13: Backend Abstraction

Shows the container backend abstraction and implementations.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_WITH_LEGEND()

title Backend Abstraction - Container Operations v1.0.1

Container_Boundary(backend_system, "Backend Abstraction Layer") {
    Component(backend_trait, "Backend Trait", "Core abstraction", "async fn run(cmd: Cmd) -> Result<RunResult>")

    Component_Boundary(implementations, "Implementations") {
        Component(testcontainer_backend, "TestcontainerBackend", "Production", "Uses testcontainers-rs for real Docker containers")
        Component(mock_backend, "MockBackend", "Testing", "Fast in-memory simulation for unit tests")
    }

    Component(cmd_struct, "Cmd", "Command spec", "bin, args, workdir, env, policy constraints")

    Component(run_result, "RunResult", "Execution result", "exit_code, stdout, stderr, duration_ms, steps")

    Component(volume_mount, "VolumeMount", "Storage", "host_path, container_path, read_only")

    Component(capabilities, "Capabilities", "Feature flags", "network_isolation, volume_mounts, privileged_mode")
}

System_Ext(testcontainers_lib, "testcontainers-rs 0.25", "Rust library")
System_Ext(docker_daemon, "Docker Daemon", "Container runtime")

Rel(backend_trait, testcontainer_backend, "Implemented by", "Production path")
Rel(backend_trait, mock_backend, "Implemented by", "Test path")

Rel(testcontainer_backend, testcontainers_lib, "Uses", "Container API")
Rel(testcontainers_lib, docker_daemon, "Manages", "Container lifecycle")

Rel(backend_trait, cmd_struct, "Accepts", "Command input")
Rel(backend_trait, run_result, "Returns", "Execution output")

Rel(cmd_struct, volume_mount, "Includes", "Volume spec")
Rel(testcontainer_backend, capabilities, "Supports", "Feature checks")
Rel(mock_backend, capabilities, "Simulates", "Limited features")

note right of backend_trait
  **Backend Trait:**
  pub trait Backend: Send + Sync {
    async fn run(
      &self,
      cmd: Cmd
    ) -> Result<RunResult>;

    fn capabilities(&self)
      -> Capabilities;
  }
end note

note left of testcontainer_backend
  **Production Features:**
  - Real container execution
  - Port mapping
  - Volume mounts
  - Network isolation
  - Log streaming
  - Resource limits
end note

note bottom of mock_backend
  **Fast Testing:**
  - Instant responses
  - No Docker dependency
  - Configurable behavior
  - Predictable results
  - Property-based testing
end note

note right of run_result
  **Rich Results:**
  - Exit codes
  - Stdout/stderr streams
  - Duration tracking
  - Step breakdown
  - Backend info
  - Concurrency flags
end note

@enduml
```

**Description**: The Backend trait abstracts container operations with async methods. TestcontainerBackend is the production implementation using testcontainers-rs 0.25 for real Docker operations. MockBackend provides fast in-memory simulation for unit tests. Cmd structures specify commands with policies. RunResult captures comprehensive execution data. VolumeMount supports persistent storage. Capabilities flag available features. This abstraction enables both production execution and fast testing without Docker overhead.

---

### Diagram 14: Configuration Loading

Details the TOML configuration parsing and validation system.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_TOP_DOWN()

title Configuration Loading System - TOML Parsing v1.0.1

Container_Boundary(config_system, "Configuration System") {
    Component(config_loader, "Configuration Loader", "loader.rs", "load_cleanroom_config(), discover TOML files")

    Component_Boundary(config_types, "Configuration Types") {
        Component(test_config, "TestConfig", "Main config", "test.metadata, services, steps, assertions")
        Component(step_config, "StepConfig", "Step definition", "name, command, expected_output_regex, service")
        Component(service_config, "ServiceConfig", "Service definition", "type, image, ports, environment, volumes")
        Component(cleanroom_config, "CleanroomConfig", "Project config", "Global settings, defaults, policies")
        Component(scenario_config, "ScenarioConfig", "Scenario", "Multi-step test scenarios with dependencies")
    }

    Component(config_deserializers, "Custom Deserializers", "deserializers.rs", "Parse shell commands, durations, regex patterns")

    Component(config_validator, "Configuration Validator", "Validation", "Required fields, type checking, dependency resolution")

    Component(template_resolver, "Template Resolver", "Template support", "Resolve {{ variables }} in TOML files")

    Component(toml_parser, "TOML Parser", "toml crate", "Parse .clnrm.toml files")
}

System_Ext(filesystem, "Filesystem", ".clnrm.toml files")
System_Ext(tera_engine, "Tera Engine", "Template rendering")

Rel(config_loader, filesystem, "Reads", "TOML files")
Rel(config_loader, toml_parser, "Parses", "TOML syntax")
Rel(toml_parser, config_deserializers, "Uses", "Custom types")
Rel(toml_parser, test_config, "Creates", "Parsed config")

Rel(test_config, step_config, "Contains", "Vec<StepConfig>")
Rel(test_config, service_config, "Contains", "HashMap<String, ServiceConfig>")
Rel(cleanroom_config, test_config, "Includes", "Multiple tests")

Rel(config_loader, template_resolver, "Applies", "Variable substitution")
Rel(template_resolver, tera_engine, "Uses", "Template rendering")

Rel(config_loader, config_validator, "Validates", "Parsed config")
Rel(config_validator, test_config, "Checks", "Required fields")
Rel(config_validator, step_config, "Validates", "Command syntax")
Rel(config_validator, service_config, "Verifies", "Service references")

note right of test_config
  **TestConfig Structure:**
  [test.metadata]
  name = "my_test"
  description = "Test desc"

  [services.db]
  type = "generic_container"
  image = "postgres:15"

  [[steps]]
  name = "setup"
  command = ["psql", "-c", "..."]
  service = "db"
end note

note left of config_validator
  **Validation Rules:**
  - Required fields present
  - Service references valid
  - Command syntax correct
  - Regex patterns compile
  - Duration formats valid
  - Dependencies resolvable
end note

note bottom of config_deserializers
  **Custom Parsing:**
  - Shell command strings
  - Duration (30s, 5m, 1h)
  - Regex patterns
  - Environment variables
  - Port mappings
  - Volume mounts
end note

@enduml
```

**Description**: The configuration system loads and validates TOML test definitions. Configuration Loader discovers .clnrm.toml files and uses the toml crate for parsing. Five configuration types define tests: TestConfig (main), StepConfig (test steps), ServiceConfig (services), CleanroomConfig (project globals), and ScenarioConfig (complex scenarios). Custom Deserializers handle shell commands, durations, and regex. Configuration Validator ensures required fields and type correctness. Template Resolver supports {{ variable }} substitution via Tera. This enables declarative testing without code.

---

### Diagram 15: Template Processing

Shows the Tera template engine integration with macro library.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_WITH_LEGEND()

title Template Processing System - Tera Integration v1.0.1

Container_Boundary(template_system, "Template Processing") {
    Component(template_renderer, "TemplateRenderer", "Main interface", "render(), render_string(), load_templates()")

    Component(tera_instance, "Tera Instance", "Template engine", "Tera 1.19 with custom functions and filters")

    Component_Boundary(template_features, "Template Features") {
        Component(template_context, "TemplateContext", "Context data", "Variables, functions, test data for rendering")
        Component(macro_library, "Macro Library", "Reusable macros", "Common patterns: with_database, with_cache, with_web_server")
        Component(custom_functions, "Custom Functions", "Template funcs", "now(), uuid(), fake_data(), random_string()")
        Component(custom_filters, "Custom Filters", "Data transforms", "uppercase, lowercase, hash, regex_match")
    }

    Component(determinism_engine, "Determinism Engine", "Reproducibility", "Seeded RNG for deterministic fake data")

    Component(template_validator, "Template Validator", "Syntax check", "Validates template syntax before rendering")

    Component(template_cache, "Template Cache", "Performance", "Caches compiled templates")
}

System_Ext(tera_lib, "Tera 1.19", "Template library")
System_Ext(fake_lib, "Fake 2.9", "Fake data generation")
System_Ext(toml_files, "TOML Files", "Template sources")

Rel(template_renderer, tera_instance, "Uses", "Template operations")
Rel(tera_instance, tera_lib, "Built on", "Tera library")

Rel(template_renderer, template_context, "Accepts", "Context data")
Rel(template_renderer, template_validator, "Validates", "Syntax")
Rel(template_renderer, template_cache, "Caches", "Compiled templates")

Rel(tera_instance, macro_library, "Includes", "Reusable macros")
Rel(tera_instance, custom_functions, "Registers", "Template functions")
Rel(tera_instance, custom_filters, "Registers", "Data filters")

Rel(custom_functions, determinism_engine, "Uses", "Seeded RNG")
Rel(custom_functions, fake_lib, "Generates", "Fake data")

Rel(toml_files, template_renderer, "Renders", "Template variables")

note right of macro_library
  **Macro Examples:**
  {% macro with_database(name, image) %}
  [services.{{ name }}]
  type = "generic_container"
  image = "{{ image }}"
  {% endmacro %}

  {{ with_database("db", "postgres:15") }}
end note

note left of custom_functions
  **Template Functions:**
  - now() - Current timestamp
  - uuid() - Generate UUID v4
  - fake_data(type) - Fake data
  - random_string(len) - Random
  - env(var) - Environment var
end note

note bottom of determinism_engine
  **Deterministic Rendering:**
  - Seeded random generation
  - Reproducible fake data
  - Consistent UUIDs
  - Deterministic timestamps
  - Essential for test reliability
end note

note right of template_renderer
  **Usage:**
  let mut renderer = TemplateRenderer::new();
  let mut context = TemplateContext::new();
  context.insert("db_image", "postgres:15");

  let result = renderer.render(
    "test.toml.tera",
    &context
  )?;
end note

@enduml
```

**Description**: The template system uses Tera 1.19 for powerful template rendering in TOML files. TemplateRenderer is the main interface backed by a Tera Instance. Template Context provides variables. Macro Library contains reusable patterns like with_database. Custom Functions add utility features (uuid, now, fake_data). Custom Filters transform data. Determinism Engine ensures reproducible rendering with seeded RNG. Template Validator checks syntax. Template Cache improves performance. This enables DRY TOML configurations with variable substitution and macros.

---

### Diagram 16: OTEL Integration

Details the OpenTelemetry implementation across the framework.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_TOP_DOWN()

title OpenTelemetry Integration Components - v1.0.1

Container_Boundary(otel_system, "OpenTelemetry System") {
    Component(telemetry_builder, "TelemetryBuilder", "Initialization", "Builds and configures OTEL providers")

    Component(telemetry_handle, "TelemetryHandle", "Lifecycle", "Manages telemetry shutdown and state")

    Component_Boundary(providers, "Signal Providers") {
        Component(tracer_provider, "TracerProvider", "Tracing", "Creates spans for operations")
        Component(meter_provider, "MeterProvider", "Metrics", "Creates meters for measurements")
        Component(logger_provider, "LoggerProvider", "Logs", "Structured log records")
    }

    Component_Boundary(processors, "Processors") {
        Component(batch_processor, "BatchSpanProcessor", "Batching", "Batches spans before export")
        Component(simple_processor, "SimpleSpanProcessor", "Immediate", "Exports spans immediately (testing)")
        Component(validation_processor, "ValidationSpanProcessor", "Testing", "Collects spans for validation")
    }

    Component_Boundary(exporters_comp, "Exporters") {
        Component(otlp_exporter, "OtlpExporter", "OTLP", "HTTP/gRPC to collectors")
        Component(stdout_exporter, "StdoutExporter", "Console", "JSON to stdout")
        Component(jaeger_exporter, "JaegerExporter", "Jaeger", "Legacy protocol")
    }

    Component(resource_builder, "ResourceBuilder", "Metadata", "service.name, version, environment")

    Component(sampling_config, "SamplingConfig", "Sampling", "AlwaysOn, TraceIdRatio, ParentBased")
}

System_Ext(global_otel, "opentelemetry::global", "Global registry")
System_Ext(tracing_otel, "tracing-opentelemetry", "Tracing bridge")

Rel(telemetry_builder, tracer_provider, "Creates", "With config")
Rel(telemetry_builder, meter_provider, "Creates", "With config")
Rel(telemetry_builder, logger_provider, "Creates", "With config")
Rel(telemetry_builder, resource_builder, "Builds", "Resource metadata")
Rel(telemetry_builder, sampling_config, "Applies", "Sampling rules")
Rel(telemetry_builder, telemetry_handle, "Returns", "Handle")

Rel(tracer_provider, batch_processor, "Uses", "Default")
Rel(tracer_provider, simple_processor, "Uses", "Testing")
Rel(tracer_provider, validation_processor, "Uses", "Validation")

Rel(batch_processor, otlp_exporter, "Exports to", "OTLP")
Rel(simple_processor, stdout_exporter, "Exports to", "Console")
Rel(batch_processor, jaeger_exporter, "Exports to", "Jaeger")

Rel(tracer_provider, global_otel, "Registers", "Global provider")
Rel(global_otel, tracing_otel, "Bridges", "tracing crate")

note right of telemetry_builder
  **Builder Pattern:**
  TelemetryBuilder::new(config)
    .with_tracer_provider()
    .with_meter_provider()
    .with_logger_provider()
    .init()?

  Returns TelemetryHandle for shutdown
end note

note left of providers
  **Signal Types:**
  - Traces: Distributed tracing
    (spans with parent/child)
  - Metrics: Measurements
    (counters, gauges, histograms)
  - Logs: Structured events
    (with trace context)
end note

note bottom of exporters_comp
  **Exporter Configuration:**
  - OTLP: Production standard
  - Stdout: Development/debugging
  - Jaeger: Legacy compatibility
  All exporters support batching
  and retry logic
end note

@enduml
```

**Description**: The OTEL integration is built around TelemetryBuilder which creates three signal providers: TracerProvider for spans, MeterProvider for metrics, and LoggerProvider for structured logs. Three processor types handle span export: BatchSpanProcessor for production batching, SimpleSpanProcessor for immediate export in tests, and ValidationSpanProcessor for test assertions. Three exporter types export to different backends: OtlpExporter for OTLP HTTP/gRPC, StdoutExporter for console output, and JaegerExporter for legacy Jaeger. ResourceBuilder adds service metadata. SamplingConfig controls sampling rates.

---

### Diagram 17: Validation Framework

Shows the comprehensive validation system including OTEL validation.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_WITH_LEGEND()

title Validation Framework - Comprehensive Verification v1.0.1

Container_Boundary(validation_framework, "Validation System") {
    Component(validation_orchestrator, "ValidationOrchestrator", "Coordinator", "Runs all validators and aggregates results")

    Component_Boundary(otel_validators, "OTEL Validators") {
        Component(span_validator, "SpanValidator", "Span checks", "Validates span attributes, timing, status")
        Component(count_validator, "CountValidator", "Count checks", "Verifies expected span counts")
        Component(graph_validator, "GraphValidator", "Relationships", "Validates parent-child span relationships")
        Component(hermeticity_validator, "HermeticityValidator", "Isolation", "Ensures no cross-test span pollution")
        Component(order_validator, "OrderValidator", "Sequence", "Validates span ordering and timing")
        Component(status_validator, "StatusValidator", "Status codes", "Checks span status (OK, Error, Unset)")
        Component(window_validator, "WindowValidator", "Time windows", "Validates timing constraints")
    }

    Component(shape_validator, "ShapeValidator", "Structure", "Validates test result structure")

    Component(prd_expectations, "PrdExpectations", "Requirements", "Product requirements verification")

    Component(validation_report, "ValidationReport", "Results", "Aggregated validation results with failures")

    Component(otel_validator_main, "OtelValidator", "OTEL API", "Main API for OTEL validation")
}

System_Ext(otel_data, "OTEL Data", "Spans, traces, metrics")
System_Ext(test_results, "Test Results", "Execution outcomes")

Rel(validation_orchestrator, span_validator, "Runs", "Span validation")
Rel(validation_orchestrator, count_validator, "Runs", "Count checks")
Rel(validation_orchestrator, graph_validator, "Runs", "Relationship checks")
Rel(validation_orchestrator, hermeticity_validator, "Runs", "Isolation checks")
Rel(validation_orchestrator, order_validator, "Runs", "Ordering checks")
Rel(validation_orchestrator, status_validator, "Runs", "Status checks")
Rel(validation_orchestrator, window_validator, "Runs", "Timing checks")
Rel(validation_orchestrator, shape_validator, "Runs", "Structure validation")
Rel(validation_orchestrator, prd_expectations, "Checks", "Requirements")
Rel(validation_orchestrator, validation_report, "Generates", "Final report")

Rel(otel_validator_main, span_validator, "Uses", "Span assertions")
Rel(otel_validator_main, count_validator, "Uses", "Count assertions")
Rel(otel_validator_main, graph_validator, "Uses", "Trace assertions")

Rel(otel_data, span_validator, "Validates", "Span data")
Rel(otel_data, count_validator, "Validates", "Span counts")
Rel(otel_data, graph_validator, "Validates", "Trace graphs")
Rel(test_results, shape_validator, "Validates", "Result structure")

note right of otel_validators
  **Validator Hierarchy:**
  Each validator implements:
  - validate() -> Result<ValidationResult>
  - Specific validation logic
  - Error collection
  - Detailed failure messages
end note

note left of hermeticity_validator
  **Hermeticity Checks:**
  - No shared state between tests
  - Unique trace IDs per test
  - No span leakage
  - Proper resource cleanup
  - Timestamp boundaries
end note

note bottom of validation_report
  **ValidationReport:**
  - Overall pass/fail
  - Individual validator results
  - Detailed failure messages
  - Span/trace IDs for debugging
  - Execution metadata
end note

note right of otel_validator_main
  **Usage Example:**
  let validator = OtelValidator::new();

  let assertion = SpanAssertion {
    name: "db.query",
    attributes: hashmap!{
      "db.operation" => "SELECT"
    },
    required: true,
    min_duration_ms: Some(1.0),
  };

  let result = validator
    .validate_span_real(&assertion)?;
end note

@enduml
```

**Description**: The validation framework ensures test correctness through comprehensive checks. ValidationOrchestrator coordinates seven OTEL validators: SpanValidator for span attributes, CountValidator for expected counts, GraphValidator for parent-child relationships, HermeticityValidator for isolation, OrderValidator for sequencing, StatusValidator for status codes, and WindowValidator for timing. ShapeValidator checks result structure. PrdExpectations verifies requirements. ValidationReport aggregates all results. OtelValidator provides the main API for OTEL validation assertions. Each validator returns detailed failure information.

---

### Diagram 18: CLI Command Router

Details the CLI command parsing and routing system.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_TOP_DOWN()

title CLI Command Router - clap-based Routing v1.0.1

Container_Boundary(cli_router_system, "CLI Router") {
    Component(cli_parser, "CLI Parser", "clap 4.5", "Parses command-line arguments into Commands enum")

    Component(commands_enum, "Commands Enum", "Command variants", "Run, Init, Health, SelfTest, Plugins, Validate, Report, Services, Template")

    Component_Boundary(command_handlers, "Command Handlers") {
        Component(run_handler, "RunCommand", "Test execution", "Loads TOML, creates environment, runs tests")
        Component(init_handler, "InitCommand", "Initialization", "Creates .clnrm.toml scaffold")
        Component(health_handler, "HealthCommand", "Health checks", "Validates Docker, dependencies")
        Component(self_test_handler, "SelfTestCommand", "Dogfooding", "Tests framework capabilities")
        Component(plugins_handler, "PluginsCommand", "Plugin listing", "Lists available plugins")
        Component(validate_handler, "ValidateCommand", "Config validation", "Validates TOML syntax")
        Component(report_handler, "ReportCommand", "Report generation", "Generates HTML/JSON reports")
        Component(services_handler, "ServicesCommand", "Service mgmt", "Manages services")
        Component(template_handler, "TemplateCommand", "Template ops", "Renders templates")
    }

    Component(cli_telemetry_init, "CLI Telemetry", "Observability", "Optional OTEL for CLI operations")

    Component(error_handler, "Error Handler", "Error display", "User-friendly error formatting")

    Component(progress_display, "Progress Display", "User feedback", "Progress bars, spinners")
}

System_Ext(clnrm_core, "clnrm-core", "Core library")
System_Ext(terminal, "Terminal", "User interaction")

Rel(terminal, cli_parser, "Executes", "clnrm <command>")
Rel(cli_parser, commands_enum, "Parses to", "Command variant")
Rel(cli_parser, cli_telemetry_init, "Initializes", "Optional telemetry")

Rel(commands_enum, run_handler, "Routes", "Run command")
Rel(commands_enum, init_handler, "Routes", "Init command")
Rel(commands_enum, health_handler, "Routes", "Health command")
Rel(commands_enum, self_test_handler, "Routes", "SelfTest command")
Rel(commands_enum, plugins_handler, "Routes", "Plugins command")
Rel(commands_enum, validate_handler, "Routes", "Validate command")
Rel(commands_enum, report_handler, "Routes", "Report command")
Rel(commands_enum, services_handler, "Routes", "Services command")
Rel(commands_enum, template_handler, "Routes", "Template command")

Rel(run_handler, clnrm_core, "Uses", "CleanroomEnvironment")
Rel(init_handler, clnrm_core, "Uses", "Config templates")
Rel(health_handler, clnrm_core, "Uses", "Health checks")
Rel(self_test_handler, clnrm_core, "Uses", "Self-testing")
Rel(validate_handler, clnrm_core, "Uses", "Config validation")

Rel(run_handler, progress_display, "Shows", "Test progress")
Rel(run_handler, error_handler, "Handles", "Errors")
Rel(error_handler, terminal, "Displays", "User-friendly errors")

note right of commands_enum
  **Commands Enum:**
  #[derive(Parser)]
  enum Commands {
    Run(RunCommand),
    Init(InitCommand),
    Health(HealthCommand),
    SelfTest(SelfTestCommand),
    Plugins(PluginsCommand),
    Validate(ValidateCommand),
    Report(ReportCommand),
    Services(ServicesCommand),
    Template(TemplateCommand),
  }
end note

note left of run_handler
  **Run Command:**
  - Discovers .clnrm.toml files
  - Creates CleanroomEnvironment
  - Registers services
  - Executes test steps
  - Collects results
  - Generates reports
  - Proper error handling
end note

note bottom of cli_telemetry_init
  **CLI Observability:**
  - Optional OTEL tracing
  - Command duration metrics
  - Error tracking
  - Environment from env vars
  - Graceful degradation
end note

@enduml
```

**Description**: The CLI router uses clap 4.5 for parsing. CLI Parser parses arguments into Commands enum with 9 variants. Each command routes to a dedicated handler: RunCommand executes tests, InitCommand scaffolds projects, HealthCommand validates setup, SelfTestCommand dogfoods the framework, PluginsCommand lists available plugins, ValidateCommand checks TOML syntax, ReportCommand generates outputs, ServicesCommand manages services, and TemplateCommand handles templates. CLI Telemetry provides optional observability. Error Handler formats user-friendly errors. Progress Display shows real-time feedback. All handlers delegate to clnrm-core.

---

### Diagram 19: Test Execution Pipeline

Shows the end-to-end test execution flow.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_WITH_LEGEND()

title Test Execution Pipeline - End-to-End Flow v1.0.1

Container_Boundary(execution_pipeline, "Test Execution Pipeline") {
    Component(test_discovery, "Test Discovery", "File scanning", "Finds .clnrm.toml files using walkdir")

    Component(config_loading, "Config Loading", "TOML parsing", "Parses and validates TestConfig")

    Component(environment_setup, "Environment Setup", "Initialization", "Creates CleanroomEnvironment per test")

    Component(service_startup, "Service Startup", "Service init", "Starts required services via plugins")

    Component(step_executor, "Step Executor", "Command execution", "Runs test steps sequentially or concurrently")

    Component(assertion_checker, "Assertion Checker", "Verification", "Validates expected outputs, exit codes")

    Component(artifact_collector, "Artifact Collector", "Artifacts", "Collects logs, metrics, traces")

    Component(result_aggregator, "Result Aggregator", "Results", "Aggregates step results into test result")

    Component(cleanup_manager, "Cleanup Manager", "Teardown", "Stops services, removes containers")

    Component(report_generator, "Report Generator", "Reporting", "Generates JUnit, JSON, TAP, human outputs")
}

System_Ext(filesystem, "Filesystem", ".clnrm.toml files")
System_Ext(docker, "Docker", "Container runtime")
System_Ext(otel_backend, "OTEL Backend", "Telemetry export")

Rel(test_discovery, filesystem, "Scans", "Recursive search")
Rel(test_discovery, config_loading, "Passes", "TOML files")
Rel(config_loading, environment_setup, "Provides", "TestConfig")
Rel(environment_setup, service_startup, "Triggers", "Service start")
Rel(service_startup, docker, "Creates", "Containers")
Rel(service_startup, step_executor, "Ready for", "Execution")
Rel(step_executor, docker, "Executes in", "Containers")
Rel(step_executor, assertion_checker, "Results to", "Validation")
Rel(assertion_checker, artifact_collector, "Triggers", "Collection")
Rel(artifact_collector, docker, "Collects from", "Containers")
Rel(artifact_collector, otel_backend, "Exports", "Telemetry")
Rel(assertion_checker, result_aggregator, "Sends", "Step results")
Rel(result_aggregator, cleanup_manager, "Triggers", "Cleanup")
Rel(cleanup_manager, docker, "Removes", "Containers")
Rel(result_aggregator, report_generator, "Provides", "Test results")

note right of test_discovery
  **Discovery Strategy:**
  - Recursive directory walk
  - Match "*.clnrm.toml" pattern
  - Respect .gitignore rules
  - Parallel file reading
end note

note left of environment_setup
  **Hermetic Isolation:**
  - Fresh CleanroomEnvironment
    per test
  - No shared state
  - Unique container names
  - Isolated networks
  - Independent lifecycles
end note

note bottom of step_executor
  **Execution Modes:**
  - Sequential (default)
  - Concurrent (with deps)
  - Fail-fast (on error)
  - Continue-on-error
  - Timeout enforcement
end note

note right of artifact_collector
  **Artifacts:**
  - Container logs
  - OTEL traces/metrics
  - Screenshots (if enabled)
  - Network dumps
  - State snapshots
end note

@enduml
```

**Description**: The test execution pipeline orchestrates end-to-end testing. Test Discovery scans for .clnrm.toml files. Config Loading parses and validates TOML. Environment Setup creates hermetic CleanroomEnvironment instances. Service Startup launches containerized services via plugins. Step Executor runs test steps with configurable concurrency. Assertion Checker validates outputs and exit codes. Artifact Collector gathers logs, traces, and metrics. Result Aggregator combines step results. Cleanup Manager tears down containers. Report Generator produces multiple output formats. Each test runs in complete isolation with automatic cleanup.

---

### Diagram 20: Metrics Collection

Details the metrics and performance tracking system.

```plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

LAYOUT_TOP_DOWN()

title Metrics Collection System - Performance Tracking v1.0.1

Container_Boundary(metrics_system, "Metrics Collection") {
    Component(simple_metrics, "SimpleMetrics", "Basic counters", "In-memory counters and gauges")

    Component_Boundary(otel_metrics, "OTEL Metrics") {
        Component(meter_provider, "MeterProvider", "OTEL metrics", "Creates meters for instrumentation")
        Component(test_counter, "Test Counter", "Counter", "Tracks test_runs_total{status}")
        Component(test_duration, "Test Duration", "Histogram", "test_duration_seconds")
        Component(container_gauge, "Container Gauge", "Gauge", "active_containers")
        Component(step_counter, "Step Counter", "Counter", "test_steps_total{status}")
    }

    Component(metrics_exporter, "Metrics Exporter", "Export", "Prometheus endpoint, OTLP push")

    Component(performance_tracker, "Performance Tracker", "Timing", "Tracks operation durations")

    Component(resource_monitor, "Resource Monitor", "Resources", "CPU, memory, disk usage")

    Component(metrics_aggregator, "Metrics Aggregator", "Aggregation", "Combines metrics from multiple sources")
}

System_Ext(prometheus, "Prometheus", "Metrics storage")
System_Ext(otel_collector, "OTEL Collector", "Metrics pipeline")

Rel(simple_metrics, test_counter, "Updates", "Test counts")
Rel(simple_metrics, test_duration, "Records", "Duration")
Rel(simple_metrics, container_gauge, "Sets", "Active count")
Rel(simple_metrics, step_counter, "Increments", "Step counts")

Rel(meter_provider, test_counter, "Creates", "Counter metric")
Rel(meter_provider, test_duration, "Creates", "Histogram metric")
Rel(meter_provider, container_gauge, "Creates", "Gauge metric")
Rel(meter_provider, step_counter, "Creates", "Counter metric")

Rel(performance_tracker, test_duration, "Records", "Timing data")
Rel(resource_monitor, container_gauge, "Updates", "Resource usage")

Rel(otel_metrics, metrics_aggregator, "Sends", "OTEL metrics")
Rel(simple_metrics, metrics_aggregator, "Sends", "Simple metrics")
Rel(metrics_aggregator, metrics_exporter, "Aggregates", "All metrics")

Rel(metrics_exporter, prometheus, "Scrape endpoint", "/metrics")
Rel(metrics_exporter, otel_collector, "Push", "OTLP")

note right of simple_metrics
  **SimpleMetrics:**
  - Fast in-memory tracking
  - No external dependencies
  - Thread-safe Arc<RwLock>
  - Used for basic counts
  - Fallback when OTEL disabled
end note

note left of otel_metrics
  **OTEL Metric Types:**
  - Counter: Monotonic increase
    (test_runs_total)
  - Histogram: Distribution
    (test_duration_seconds)
  - Gauge: Current value
    (active_containers)
end note

note bottom of metrics_exporter
  **Export Formats:**
  - Prometheus: Pull-based
    (HTTP /metrics endpoint)
  - OTLP: Push-based
    (HTTP/gRPC to collector)
  - JSON: File-based
    (local storage)
end note

note right of performance_tracker
  **Tracked Operations:**
  - Test execution time
  - Container startup time
  - Service health check time
  - Config loading time
  - Template rendering time
end note

@enduml
```

**Description**: The metrics system provides comprehensive performance tracking. SimpleMetrics offers basic in-memory counters and gauges as a fallback. OTEL Metrics use MeterProvider to create instrumented metrics: Test Counter tracks runs by status, Test Duration histograms measure execution time, Container Gauge monitors active containers, and Step Counter tracks step execution. Performance Tracker times operations. Resource Monitor tracks system resources. Metrics Aggregator combines all sources. Metrics Exporter supports Prometheus scrape endpoints and OTLP push to collectors. This enables both local monitoring and centralized observability.

---

## Level 4: Code/Class Diagrams

### Diagram 21: Core Traits

Shows the key trait definitions and their relationships.

```plantuml
@startuml

title Core Traits - clnrm v1.0.1

interface ServicePlugin {
    +fn name(&self) -> &str
    +fn start(&self) -> Result<ServiceHandle>
    +fn stop(&self, handle: ServiceHandle) -> Result<()>
    +fn health_check(&self, handle: &ServiceHandle) -> HealthStatus
}

interface Backend {
    +async fn run(&self, cmd: Cmd) -> Result<RunResult>
    +fn capabilities(&self) -> Capabilities
}

interface Cache {
    +async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>
    +async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>
    +async fn invalidate(&self, key: &str) -> Result<()>
    +async fn stats(&self) -> Result<CacheStats>
}

interface Formatter {
    +fn format(&self, suite: &TestSuite) -> Result<String>
    +fn file_extension(&self) -> &str
}

class GenericContainerPlugin {
    -name: String
    -image: String
    -env_vars: HashMap<String, String>
    -ports: Vec<u16>
    -volumes: Vec<VolumeMount>
}

class SurrealDbPlugin {
    -name: String
    -version: String
    -port: u16
    -data_dir: Option<PathBuf>
}

class TestcontainerBackend {
    -docker: Arc<Docker>
    -containers: Arc<RwLock<HashMap<String, Container>>>
}

class MockBackend {
    -responses: HashMap<String, RunResult>
    -call_count: AtomicUsize
}

class FileCache {
    -cache_dir: PathBuf
    -max_size: usize
}

class MemoryCache {
    -store: Arc<RwLock<HashMap<String, Vec<u8>>>>
    -max_size: usize
}

class JunitFormatter {
}

class JsonFormatter {
}

class HumanFormatter {
}

ServicePlugin <|.. GenericContainerPlugin : implements
ServicePlugin <|.. SurrealDbPlugin : implements
Backend <|.. TestcontainerBackend : implements
Backend <|.. MockBackend : implements
Cache <|.. FileCache : implements
Cache <|.. MemoryCache : implements
Formatter <|.. JunitFormatter : implements
Formatter <|.. JsonFormatter : implements
Formatter <|.. HumanFormatter : implements

note right of ServicePlugin
  **Critical Design:**
  All methods are sync to maintain
  dyn ServicePlugin compatibility.
  Use block_in_place internally
  for async operations.

  Bounds: Send + Sync + Debug
end note

note left of Backend
  **Backend Abstraction:**
  Enables both real Docker
  containers (TestcontainerBackend)
  and fast mock testing
  (MockBackend) without changes
  to test code.
end note

note bottom of Cache
  **Caching Strategy:**
  - FileCache: Persistent on disk
  - MemoryCache: Fast in-memory
  - LRU eviction policies
  - Configurable size limits
end note

note top of Formatter
  **Output Formats:**
  - JUnit XML (CI/CD)
  - JSON (programmatic)
  - Human-readable (console)
  - TAP (Test Anything Protocol)
end note

@enduml
```

**Description**: This class diagram shows the four core trait hierarchies. ServicePlugin defines the plugin interface with sync methods - implemented by GenericContainerPlugin and SurrealDbPlugin. Backend abstracts container operations - TestcontainerBackend for production and MockBackend for testing. Cache provides caching - FileCache persists to disk while MemoryCache uses RAM. Formatter enables multiple output formats - JunitFormatter, JsonFormatter, and HumanFormatter. All traits use proper Rust bounds (Send + Sync) for concurrency.

---

### Diagram 22: Error Hierarchy

Details the comprehensive error handling system.

```plantuml
@startuml

title Error Hierarchy - Structured Error Handling v1.0.1

class CleanroomError {
    +kind: ErrorKind
    +message: String
    +context: Option<String>
    +source: Option<String>
    +timestamp: DateTime<Utc>

    +fn new(kind: ErrorKind, message: String) -> Self
    +fn with_context(self, context: String) -> Self
    +fn with_source<E: Error>(self, source: E) -> Self
    +fn container_error(message: String) -> Self
    +fn network_error(message: String) -> Self
    +fn timeout_error(message: String) -> Self
    +fn configuration_error(message: String) -> Self
    +fn validation_error(message: String) -> Self
    +fn service_error(message: String) -> Self
    +fn internal_error(message: String) -> Self
    +fn template_error(message: String) -> Self
}

enum ErrorKind {
    ContainerError
    NetworkError
    ResourceLimitExceeded
    Timeout
    ConfigurationError
    PolicyViolation
    DeterministicError
    CoverageError
    SnapshotError
    TracingError
    RedactionError
    ReportError
    IoError
    SerializationError
    ValidationError
    ServiceError
    InternalError
    TemplateError
}

class Result<T> <<type alias>> {
    std::result::Result<T, CleanroomError>
}

class ErrorContext {
    +fn add_context<T>(result: Result<T>, context: String) -> Result<T>
}

CleanroomError --> ErrorKind : has

note right of CleanroomError
  **Error Chaining:**
  - Preserves source errors
  - Adds contextual information
  - Timestamps for debugging
  - Structured for logging

  **Usage:**
  Err(CleanroomError::container_error(
    "Failed to start container"
  ).with_context(
    format!("Image: {}", image)
  ).with_source(err))
end note

note left of ErrorKind
  **18 Error Categories:**
  - Container operations
  - Network failures
  - Resource limits
  - Timeouts
  - Configuration issues
  - Policy violations
  - Service failures
  - Template rendering
  - Validation errors
  - Internal errors
end note

note bottom of Result
  **Result Type:**
  All fallible functions
  return Result<T> which is
  std::result::Result<T, CleanroomError>

  NO .unwrap() or .expect()
  in production code!

  Use ? operator for propagation.
end note

@enduml
```

**Description**: The error system is built around CleanroomError which contains an ErrorKind enum with 18 categories, plus message, context, source, and timestamp fields. CleanroomError provides constructor methods for each error kind and fluent methods for adding context and source errors. Result<T> is a type alias for std::result::Result<T, CleanroomError> used throughout the codebase. ErrorContext provides utility functions. The design enables error chaining, contextual information, and structured logging while enforcing zero unwrap/expect in production code.

---

### Diagram 23: Data Models

Shows the configuration and runtime data structures.

```plantuml
@startuml

title Data Models - Configuration & Runtime Structures v1.0.1

class TestConfig {
    +metadata: TestMetadata
    +services: HashMap<String, ServiceConfig>
    +steps: Vec<StepConfig>
    +assertions: Option<AssertionConfig>
    +otel: Option<OtelConfig>
    +determinism: Option<DeterminismConfig>
}

class TestMetadata {
    +name: String
    +description: Option<String>
    +tags: Vec<String>
    +timeout_seconds: Option<u64>
}

class ServiceConfig {
    +service_type: String
    +image: Option<String>
    +ports: Vec<u16>
    +environment: HashMap<String, String>
    +volumes: Vec<VolumeMount>
    +health_check: Option<HealthCheckConfig>
}

class StepConfig {
    +name: String
    +command: Vec<String>
    +service: Option<String>
    +expected_exit_code: i32
    +expected_output_regex: Option<String>
    +timeout_seconds: Option<u64>
}

class AssertionConfig {
    +container_should_have_executed_commands: Option<usize>
    +execution_should_be_hermetic: Option<bool>
    +should_produce_telemetry: Option<bool>
}

class ServiceHandle {
    +id: String
    +service_name: String
    +metadata: HashMap<String, String>
}

class ExecutionResult {
    +step_name: String
    +exit_code: i32
    +stdout: String
    +stderr: String
    +duration_ms: u64
    +success: bool
}

class TestResult {
    +test_name: String
    +status: TestStatus
    +duration_ms: u64
    +steps: Vec<ExecutionResult>
    +errors: Vec<String>
}

enum TestStatus {
    Passed
    Failed
    Skipped
    Error
}

class CleanroomConfig {
    +project_name: String
    +version: String
    +tests: Vec<TestConfig>
    +global_services: HashMap<String, ServiceConfig>
    +default_timeout: u64
}

TestConfig --> TestMetadata : contains
TestConfig --> ServiceConfig : contains many
TestConfig --> StepConfig : contains many
TestConfig --> AssertionConfig : optional
TestResult --> TestStatus : has
TestResult --> ExecutionResult : contains many
CleanroomConfig --> TestConfig : contains many

note right of TestConfig
  **TOML Structure:**
  [test.metadata]
  name = "integration_test"

  [services.db]
  type = "generic_container"
  image = "postgres:15"

  [[steps]]
  name = "setup"
  command = ["psql", "-c", "..."]
end note

note left of ServiceHandle
  **Runtime State:**
  ServiceHandle represents
  a running service instance
  with unique ID and metadata.

  Returned by ServicePlugin.start()
  Used for stop() and health_check()
end note

note bottom of TestResult
  **Test Results:**
  Comprehensive result structure
  with hierarchical steps,
  timing data, and error messages.

  Serialized to JUnit XML,
  JSON, or human-readable formats.
end note

@enduml
```

**Description**: The data model centers on TestConfig which defines tests in TOML. TestConfig contains TestMetadata (name, description, tags), ServiceConfig for services, StepConfig for test steps, AssertionConfig for validation, and optional OTEL/determinism configs. ServiceHandle represents running service instances. ExecutionResult captures step outcomes. TestResult aggregates test execution with TestStatus enum. CleanroomConfig provides project-level configuration. These structures enable declarative testing and comprehensive result tracking.

---

### Diagram 24: State Management

Shows state management and concurrency patterns.

```plantuml
@startuml

title State Management - Concurrency & Lifecycle v1.0.1

class CleanroomEnvironment {
    -backend: Arc<dyn Backend>
    -services: Arc<RwLock<ServiceRegistry>>
    -active_services: Arc<RwLock<HashMap<String, ServiceHandle>>>
    -telemetry_state: Option<TelemetryState>
    -test_context: Arc<RwLock<TestContext>>

    +async fn new() -> Result<Self>
    +async fn register_service(&self, plugin: Box<dyn ServicePlugin>) -> Result<()>
    +async fn start_service(&self, name: &str) -> Result<ServiceHandle>
    +async fn stop_service(&self, handle: ServiceHandle) -> Result<()>
    +async fn execute_command(&self, handle: &ServiceHandle, cmd: &[&str]) -> Result<ExecutionResult>
}

class ServiceRegistry {
    -plugins: HashMap<String, Box<dyn ServicePlugin>>
    -active_services: HashMap<String, ServiceHandle>

    +fn new() -> Self
    +fn register_plugin(&mut self, plugin: Box<dyn ServicePlugin>)
    +fn get_plugin(&self, name: &str) -> Option<&dyn ServicePlugin>
    +fn list_plugins(&self) -> Vec<String>
}

class TelemetryState {
    -handle: TelemetryHandle
    -tracer: Tracer
    -meter: Meter
    -active_spans: Arc<RwLock<HashMap<String, Span>>>

    +fn new(config: TelemetryConfig) -> Result<Self>
    +fn create_span(&self, name: &str) -> Span
    +fn record_metric(&self, name: &str, value: f64)
}

class TestContext {
    -test_name: String
    -start_time: Instant
    -resources: HashMap<String, String>
    -environment_vars: HashMap<String, String>

    +fn new(test_name: String) -> Self
    +fn elapsed(&self) -> Duration
}

class CacheManager {
    -caches: HashMap<String, Arc<dyn Cache>>
    -default_cache: Arc<dyn Cache>

    +async fn get_cache(&self, name: &str) -> Arc<dyn Cache>
    +async fn invalidate_all(&self) -> Result<()>
}

note right of CleanroomEnvironment
  **Concurrency Patterns:**
  - Arc<RwLock<T>> for shared state
  - Multiple readers, single writer
  - Tokio async runtime
  - Graceful shutdown on drop

  **Thread Safety:**
  All state is Send + Sync
  Safe concurrent access
  No data races
end note

note left of ServiceRegistry
  **Service Lifecycle:**
  1. Register plugin
  2. Start service (creates handle)
  3. Service runs (health checks)
  4. Stop service (cleanup)
  5. Remove from registry
end note

note bottom of TelemetryState
  **OTEL State:**
  - Global tracer/meter providers
  - Per-test span tracking
  - Metric aggregation
  - Automatic export
  - Proper shutdown
end note

note top of CacheManager
  **Cache Management:**
  - Multiple named caches
  - File-based persistence
  - Memory-based speed
  - Configurable eviction
  - Statistics tracking
end note

@enduml
```

**Description**: State management uses Arc<RwLock<T>> for safe concurrent access. CleanroomEnvironment maintains core state: backend abstraction, service registry, active services, telemetry state, and test context. ServiceRegistry stores plugins and active services. TelemetryState manages OTEL providers and active spans. TestContext tracks per-test execution state. CacheManager handles multiple caches. The design ensures thread safety with proper Rust concurrency patterns (Arc for shared ownership, RwLock for interior mutability) and follows async/await patterns with Tokio.

---

### Diagram 25: Execution Flow

State machine diagram showing test execution lifecycle.

```plantuml
@startuml

title Test Execution Flow - State Machine v1.0.1

[*] --> Initialized : clnrm run tests/

state Initialized {
    [*] --> DiscoveringTests
    DiscoveringTests --> LoadingConfig : .clnrm.toml found
    LoadingConfig --> ValidatingConfig : TOML parsed
    ValidatingConfig --> CreatingEnvironment : Config valid
}

state CreatingEnvironment {
    [*] --> InitializingBackend
    InitializingBackend --> RegisteringPlugins : Backend ready
    RegisteringPlugins --> InitializingTelemetry : Plugins registered
    InitializingTelemetry --> EnvironmentReady : OTEL initialized
}

Initialized --> CreatingEnvironment : Per test

state TestExecution {
    [*] --> StartingServices
    StartingServices --> WaitingForHealth : Services started
    WaitingForHealth --> ExecutingSteps : All healthy

    state ExecutingSteps {
        [*] --> ExecutingStep1
        ExecutingStep1 --> ExecutingStep2 : Step 1 success
        ExecutingStep2 --> ExecutingStep3 : Step 2 success
        ExecutingStep3 --> [*] : All steps done

        ExecutingStep1 --> StepFailed : Step 1 failed
        ExecutingStep2 --> StepFailed : Step 2 failed
        ExecutingStep3 --> StepFailed : Step 3 failed
    }

    ExecutingSteps --> CheckingAssertions : Steps completed
    StepFailed --> CheckingAssertions : Continue on error
    CheckingAssertions --> TestPassed : Assertions pass
    CheckingAssertions --> TestFailed : Assertions fail
}

CreatingEnvironment --> TestExecution : Environment ready

state Cleanup {
    [*] --> StoppingServices
    StoppingServices --> RemovingContainers : Services stopped
    RemovingContainers --> CollectingArtifacts : Containers removed
    CollectingArtifacts --> ExportingTelemetry : Artifacts collected
    ExportingTelemetry --> CleanupComplete : Telemetry exported
}

TestExecution --> Cleanup : Test completed
TestPassed --> Cleanup
TestFailed --> Cleanup

state Reporting {
    [*] --> AggregatingResults
    AggregatingResults --> FormattingOutput : Results aggregated
    FormattingOutput --> WritingReports : Formatted
    WritingReports --> DisplayingResults : Reports written
    DisplayingResults --> [*] : Complete
}

Cleanup --> Reporting : Cleanup complete

Reporting --> [*] : Exit with status code

note right of TestExecution
  **Hermetic Isolation:**
  Each test gets:
  - Fresh CleanroomEnvironment
  - New service instances
  - Isolated containers
  - Unique trace IDs
  - Clean state
end note

note left of Cleanup
  **Cleanup Guarantees:**
  - Services always stopped
  - Containers always removed
  - Resources always freed
  - Artifacts collected
  - No resource leaks

  Cleanup runs even on:
  - Test failure
  - Timeout
  - Panic/error
  - User cancellation
end note

note bottom of Reporting
  **Report Formats:**
  - JUnit XML (CI/CD)
  - JSON (programmatic)
  - TAP (compatibility)
  - Human-readable (console)

  Exit codes:
  - 0: All tests passed
  - 1: Some tests failed
  - 2: Runtime error
end note

@enduml
```

**Description**: The execution flow state machine shows the complete test lifecycle. Initialization discovers tests and loads configuration. Environment creation initializes backend, registers plugins, and sets up telemetry. Test execution starts services, waits for health, executes steps sequentially or concurrently, and checks assertions. Cleanup always runs - stopping services, removing containers, collecting artifacts, and exporting telemetry. Reporting aggregates results and generates outputs in multiple formats. The design ensures hermetic isolation per test and proper cleanup even on failures.

---

## Architecture Patterns

### Key Patterns Identified

1. **Plugin Architecture (Extensibility)**
   - ServicePlugin trait with sync methods
   - Dynamic registration via Box<dyn ServicePlugin>
   - Factory pattern for instantiation from TOML
   - Enables users to add custom services without modifying core

2. **Backend Abstraction (Testability)**
   - Backend trait abstracts container operations
   - TestcontainerBackend for production (real Docker)
   - MockBackend for fast unit testing (no Docker)
   - Swap implementations without changing test code

3. **Hermetic Isolation (Reliability)**
   - Each test gets fresh CleanroomEnvironment instance
   - No shared state between tests
   - Unique container names and network isolation
   - Automatic cleanup on drop via RAII

4. **Declarative Configuration (Simplicity)**
   - TOML-based test definitions
   - Template support with Tera (macros, variables)
   - Zero-code test authoring for common scenarios
   - Version-controlled test configurations

5. **Observability-First (Production-Ready)**
   - OpenTelemetry integration throughout
   - Traces, metrics, and logs for all operations
   - Multiple exporter support (OTLP, Jaeger, stdout)
   - Validation framework ensures telemetry correctness

6. **Error Propagation (Reliability)**
   - Result<T, CleanroomError> everywhere
   - Zero unwrap/expect in production code
   - Error chaining with context and source
   - Structured errors for logging and debugging

7. **Async Runtime Integration (Performance)**
   - Tokio for async I/O operations
   - Sync trait methods (block_in_place internally)
   - Arc<RwLock<T>> for shared state
   - Efficient concurrent execution

8. **Dogfooding (Self-Testing)**
   - Framework tests itself using its own capabilities
   - self-test command validates functionality
   - Examples demonstrate real usage
   - Documentation backed by working code

9. **Builder Pattern (Configuration)**
   - TelemetryBuilder for OTEL setup
   - Fluent API for configuration
   - Compile-time validation where possible
   - Sensible defaults with override capability

10. **Factory Pattern (Instantiation)**
    - ServiceFactory creates plugins from config
    - Plugin discovery and registration
    - Type-safe plugin instantiation
    - Error handling during creation

---

## Technology Stack

### Core Technologies by Layer

#### **Language & Runtime**
- **Rust 2021 Edition** - Memory safety, zero-cost abstractions
- **Tokio 1.0** - Async runtime for I/O operations
- **Cargo Workspace** - Multi-crate project organization

#### **Container Orchestration**
- **testcontainers-rs 0.25** - Container lifecycle management
- **Docker/Podman** - Container runtime (user choice)
- **testcontainers-modules 0.13** - Pre-built service modules

#### **Configuration & Parsing**
- **toml 0.9** - TOML parsing and serialization
- **toml_edit 0.22** - TOML manipulation
- **Tera 1.19** - Template engine for TOML
- **serde 1.0** - Serialization framework
- **clap 4.5** - CLI argument parsing

#### **Observability (OpenTelemetry)**
- **opentelemetry 0.31.0** - Core OTEL API
- **opentelemetry_sdk 0.31.0** - SDK implementation
- **opentelemetry-otlp 0.31.0** - OTLP exporter (HTTP/gRPC)
- **opentelemetry-jaeger 0.22.0** - Legacy Jaeger protocol
- **opentelemetry-zipkin 0.31.0** - Zipkin protocol
- **opentelemetry-stdout 0.31.0** - Console exporter
- **tracing 0.1** - Structured logging
- **tracing-opentelemetry 0.32.0** - OTEL bridge
- **tracing-subscriber 0.3** - Log subscriber

#### **Testing & Validation**
- **proptest 1.4** - Property-based testing (160K+ cases)
- **criterion 0.5** - Benchmarking with statistical analysis
- **insta 1.34** - Snapshot testing
- **junit-report 0.8** - JUnit XML generation

#### **Utilities**
- **anyhow 1.0** - Error handling convenience
- **regex 1.0** - Regular expressions for output matching
- **walkdir 2.5** - Recursive directory traversal
- **tempfile 3.0** - Temporary file/directory management
- **uuid 1.0** - UUID generation for unique IDs
- **chrono 0.4** - Date/time handling
- **hostname 0.4** - System hostname detection
- **reqwest 0.12** - HTTP client (OTLP, registry)
- **fake 2.9** - Fake data generation for templates

#### **Storage & Caching**
- **sha2 0.10** - SHA-256 hashing for cache keys
- **glob 0.3** - File pattern matching
- **globset 0.4** - Efficient glob matching

#### **Reporting**
- **quick-xml 0.31** - XML generation for JUnit
- **serde_json 1.0** - JSON serialization for reports

---

## Architectural Decision Records

### ADR-001: Workspace Structure with AI Isolation

**Decision**: Use Cargo workspace with clnrm-ai crate excluded from default-members.

**Rationale**:
- Experimental AI features should not affect production stability
- cargo build excludes AI by default
- cargo build -p clnrm-ai explicitly builds experimental code
- Clear separation of concerns

### ADR-002: Sync ServicePlugin Trait Methods

**Decision**: All ServicePlugin trait methods are sync (not async).

**Rationale**:
- Maintains `dyn ServicePlugin` compatibility
- Enables Box<dyn ServicePlugin> storage
- Use tokio::task::block_in_place internally for async ops
- Avoids async trait limitations in Rust

### ADR-003: Backend Abstraction for Testability

**Decision**: Abstract container operations behind Backend trait.

**Rationale**:
- Enables MockBackend for fast unit tests (no Docker)
- TestcontainerBackend for production (real containers)
- Tests run 100x faster with MockBackend
- Same test code works with both backends

### ADR-004: TOML-based Declarative Testing

**Decision**: Use TOML for test definitions instead of Rust code.

**Rationale**:
- Non-programmers can write tests
- Version-controlled test configurations
- Template support for DRY configurations
- Clear separation of test logic and framework

### ADR-005: OpenTelemetry 0.31.0 with Feature Flags

**Decision**: Use latest OTEL 0.31.0 with optional feature flags.

**Rationale**:
- Zero overhead when disabled
- Modular features (traces, metrics, logs)
- Multiple exporter support (OTLP, Jaeger, stdout)
- Production-ready observability

### ADR-006: Hermetic Isolation via Fresh Environments

**Decision**: Create fresh CleanroomEnvironment per test.

**Rationale**:
- No shared state between tests
- Reliable, reproducible test execution
- Parallel test execution without conflicts
- Automatic cleanup via RAII

### ADR-007: Error Handling with Result<T, CleanroomError>

**Decision**: All fallible operations return Result<T, CleanroomError>.

**Rationale**:
- Zero unwrap/expect in production code
- Structured errors with context and source
- Proper error propagation via ? operator
- User-friendly error messages

### ADR-008: Dogfooding Self-Test Command

**Decision**: Implement clnrm self-test to test framework with framework.

**Rationale**:
- Validates core functionality
- Demonstrates real usage patterns
- Catches regressions early
- Documentation backed by working code

---

## Summary

This comprehensive C4 architecture document provides 25 detailed diagrams across four abstraction levels, documenting the clnrm v1.0.1 hermetic testing framework. The architecture demonstrates:

- **Modular Design**: Clear separation via Cargo workspace (4 crates)
- **Extensibility**: Plugin architecture with ServicePlugin trait
- **Testability**: Backend abstraction (Testcontainer vs Mock)
- **Observability**: Comprehensive OpenTelemetry integration
- **Reliability**: Hermetic isolation and proper error handling
- **Usability**: Declarative TOML testing with templates
- **Production Quality**: Zero unwrap/expect, AAA test patterns

The framework successfully implements the "eat your own dog food" principle, testing itself using its own capabilities while maintaining FAANG-level code standards.
