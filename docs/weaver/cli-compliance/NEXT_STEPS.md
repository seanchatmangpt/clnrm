# CLI Instrumentation - Next Steps

**For the next agent (Code Analyzer or Backend Dev)**

---

## Immediate Action Items

### 1. Instrument `clnrm plugins` (1 hour)

**File**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/plugins.rs`

**Current**:
```rust
pub fn list_plugins() -> Result<()> {
    info!("📦 Available Service Plugins:");
    // ... println! statements
    Ok(())
}
```

**Required**:
```rust
use opentelemetry::trace::Tracer;
use tracing::instrument;

#[instrument(name = "plugin.list", skip_all)]
pub fn list_plugins() -> Result<()> {
    let tracer = opentelemetry::global::tracer("clnrm.cli");

    let plugins = vec![
        "generic_container", "surreal_db", "network_tools",
        "ollama", "vllm", "tgi"
    ];

    tracer.in_span("plugin.list", |cx| {
        cx.span().set_attribute("plugin.count", plugins.len() as i64);
        cx.span().set_attribute("plugin.categories", "core,llm,experimental");

        info!("📦 Available Service Plugins:");
        // ... existing display logic

        Ok(())
    })
}
```

**Schema to create**: `registry/cli/plugin_commands.yaml`

---

### 2. Instrument `clnrm health` (2 hours)

**File**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/health.rs`

**Add metrics export**:
```rust
pub async fn system_health_check(verbose: bool) -> Result<()> {
    let meter = opentelemetry::global::meter("clnrm.health");
    let health_gauge = meter.f64_gauge("health.score").init();
    let warnings_counter = meter.i64_counter("health.warnings").init();
    let errors_counter = meter.i64_counter("health.errors").init();

    // ... existing health checks

    // Export metrics
    health_gauge.record(health_percentage as f64, &[
        KeyValue::new("health.status", get_health_status(health_percentage)),
    ]);
    warnings_counter.add(warnings.len() as i64, &[]);
    errors_counter.add(errors.len() as i64, &[]);

    Ok(())
}
```

**Add Docker health check**:
```rust
async fn check_docker_health() -> Result<()> {
    let output = Command::new("docker")
        .args(&["info", "--format", "{{.ServerVersion}}"])
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(())
    } else {
        Err(CleanroomError::service_error("Docker daemon not running"))
    }
}

// In system_health_check:
println!("\n🐳 Docker Integration");
println!("─────────────────────────────────────");
total_checks += 1;
match check_docker_health().await {
    Ok(_) => {
        println!("  ✅ Docker: Available");
        health_score += 1;
    }
    Err(e) => {
        println!("  ❌ Docker: Unavailable");
        warnings.push(format!("Docker daemon: {}", e));
    }
}
```

**Schema to create**: `registry/cli/health_check.yaml`

---

### 3. Instrument `clnrm services status` (1 hour)

**File**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/services_noun_verb.rs`

**Add span**:
```rust
#[instrument(name = "services.status", skip_all)]
async fn show_service_status() -> Result<()> {
    let tracer = opentelemetry::global::tracer("clnrm.services");

    tracer.in_span("services.status", |cx| {
        println!("📊 Service Status:");

        let environment = CleanroomEnvironment::new().await?;
        let services = environment.services().await;

        let active_count = services.active_services().len();
        cx.span().set_attribute("service.count", active_count as i64);

        if active_count > 0 {
            let names: Vec<_> = services.active_services()
                .values()
                .map(|h| h.service_name.clone())
                .collect();
            cx.span().set_attribute("service.names", format!("{:?}", names));
        }

        // ... existing display logic
        Ok(())
    })
}
```

**Schema**: Use existing `registry/core/plugin_system.yaml`, add `services.status` span.

---

### 4. Fix `clnrm collector status` (2 hours)

**File**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/collector.rs`

**Add fallback Docker detection**:
```rust
/// Find OTEL collector container directly via Docker API
fn find_otel_collector_container() -> Result<Option<String>> {
    let output = Command::new("docker")
        .args(&["ps", "--filter", "name=otel-collector", "--format", "{{.ID}}"])
        .output()?;

    if output.status.success() {
        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !container_id.is_empty() {
            Ok(Some(container_id))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[instrument(name = "collector.status_check", skip_all)]
pub async fn show_collector_status() -> Result<()> {
    let tracer = opentelemetry::global::tracer("clnrm.collector");

    tracer.in_span("collector.status_check", |cx| {
        match CollectorState::load()? {
            Some(state) => {
                let running = is_container_running(&state.container_id)?;

                cx.span().set_attribute("container.id", state.container_id.clone());
                cx.span().set_attribute("container.state", if running { "running" } else { "stopped" });
                cx.span().set_attribute("collector.managed_by", "clnrm");

                // ... existing display logic
            }
            None => {
                // Fallback: check Docker directly
                if let Some(container_id) = find_otel_collector_container()? {
                    cx.span().set_attribute("container.id", container_id.clone());
                    cx.span().set_attribute("container.state", "running");
                    cx.span().set_attribute("collector.managed_by", "external");

                    println!("✅ OTEL collector found (externally managed)");
                    println!("   Container ID: {}", container_id);

                    // TODO: Detect ports via Docker inspect
                } else {
                    cx.span().set_attribute("container.state", "not_found");

                    println!("❌ No OTEL collector is running");
                    println!("\n💡 Start a collector: clnrm collector up");
                }
            }
        }
        Ok(())
    })
}
```

**Schema**: Use existing `registry/core/container_lifecycle.yaml`, verify `collector.status_check` span defined.

---

## Schema Definitions Needed

### `registry/cli/plugin_commands.yaml`

```yaml
groups:
  - id: clnrm.cli.plugins
    type: span
    brief: "CLI plugin management operations"
    attributes:
      - id: plugin.count
        type: int
        brief: "Number of plugins listed"
        requirement_level: required
      - id: plugin.categories
        type: string
        brief: "Comma-separated plugin categories"
        examples: ["core,llm,experimental"]

spans:
  - id: plugin.list
    brief: "List available plugins"
    attributes:
      - ref: plugin.count
      - ref: plugin.categories
```

### `registry/cli/health_check.yaml`

```yaml
groups:
  - id: clnrm.cli.health
    type: metric
    brief: "System health check metrics"
    attributes:
      - id: health.score
        type: double
        brief: "Health score percentage (0-100)"
        requirement_level: required
      - id: health.status
        type: string
        brief: "Health status description"
        examples: ["EXCELLENT", "GOOD", "DEGRADED"]
      - id: health.warnings
        type: int
        brief: "Number of warnings detected"
      - id: health.errors
        type: int
        brief: "Number of errors detected"

metrics:
  - id: health.score
    brief: "Overall system health score"
    instrument: gauge
    unit: "percent"
```

### `registry/cli/service_commands.yaml`

```yaml
groups:
  - id: clnrm.cli.services
    type: span
    brief: "CLI service management operations"
    attributes:
      - id: service.count
        type: int
        brief: "Number of active services"
      - id: service.names
        type: string
        brief: "JSON array of service names"
        examples: ['["postgres", "redis"]']

spans:
  - id: services.status
    brief: "Query service status"
    attributes:
      - ref: service.count
      - ref: service.names
```

---

## Testing the Instrumentation

### 1. Build with OTEL features

```bash
cargo build --release --features otel
```

### 2. Set OTLP endpoint

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
```

### 3. Run commands and capture telemetry

```bash
# Terminal 1: Start collector (if not running)
docker-compose -f docker/docker-compose.yml up -d

# Terminal 2: Run instrumented commands
cargo run -p clnrm -- plugins
cargo run -p clnrm -- health --verbose
cargo run -p clnrm -- services status
cargo run -p clnrm -- collector status
```

### 4. Verify telemetry in Jaeger

Open http://localhost:16686 and search for:
- Service: `clnrm.cli`
- Operations: `plugin.list`, `services.status`, `collector.status_check`

### 5. Run Weaver live-check

```bash
weaver registry live-check \
  --registry registry/ \
  --otlp-endpoint http://localhost:4318 \
  --command "cargo run -p clnrm -- plugins" \
  --expected-span "plugin.list"
```

---

## Success Criteria

✅ All 4 commands emit OTEL spans
✅ Schemas created for CLI operations
✅ Weaver live-check passes
✅ Telemetry visible in Jaeger/Grafana
✅ No performance regression (<5ms overhead)

---

## Coordination

When complete, run hooks:

```bash
npx claude-flow@alpha hooks pre-task --description "Instrument CLI commands with OTEL"
npx claude-flow@alpha hooks post-edit --file "crates/clnrm-core/src/cli/commands/plugins.rs" --memory-key "hive/cli/instrumentation"
npx claude-flow@alpha hooks notify --message "CLI instrumentation: 4/4 commands instrumented"
npx claude-flow@alpha hooks post-task --task-id "instrument-cli-commands"
```

---

## Estimated Effort

- **Phase 1**: 6 hours (instrumentation)
- **Phase 2**: 4 hours (schema creation)
- **Phase 3**: 2 hours (testing + validation)
- **Total**: 12 hours (1.5 days)

---

## Questions?

Contact the **Backend-Dev** agent or check:
- `/Users/sac/clnrm/docs/weaver/cli-compliance/SERVICE_COMMANDS_VALIDATION.md`
- `/Users/sac/clnrm/docs/weaver/cli-compliance/VALIDATION_SUMMARY.md`
