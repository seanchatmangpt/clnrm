# CLI Schema Quick Reference

**Status:** Architecture Complete - Ready for Implementation
**Commands:** 11 uninstrumented CLI commands (48% coverage gap)
**Target:** 96% CLI coverage after implementation

## Schema File Structure

```
registry/
├── cli/
│   ├── initialization.yaml          # init
│   ├── health_check.yaml           # health
│   ├── plugin_operations.yaml      # plugins
│   ├── service_management.yaml     # services, collector
│   ├── project_operations.yaml     # fmt, render, record
│   ├── image_operations.yaml       # pull
│   └── tdd_workflow.yaml           # red-green, repro
├── cli_metrics/
│   └── cli_metrics.yaml            # 4 metrics
└── cli_events/
    └── cli_events.yaml             # 5 events
```

## Commands by Category

| Category | Commands | Schema File | Priority |
|----------|----------|-------------|----------|
| **Initialization** | init | initialization.yaml | HIGH |
| **Health** | health | health_check.yaml | HIGH |
| **Plugin** | plugins | plugin_operations.yaml | MEDIUM |
| **Service Mgmt** | services, collector | service_management.yaml | MEDIUM |
| **Project Ops** | fmt, render, record | project_operations.yaml | MEDIUM |
| **Image Ops** | pull | image_operations.yaml | LOW |
| **TDD** | red-green, repro | tdd_workflow.yaml | LOW |

## Universal CLI Attributes

Every CLI span MUST have:

```yaml
cli.command              # string, required - Command name
operation.duration_ms    # double, required - Must be > 0
operation.success        # boolean, required - Final result
error.type              # string, conditionally_required - Only on failure
error.message           # string, conditionally_required - Only on failure
```

## Command Span Summary

### init - Project Initialization
```yaml
Span: clnrm.cli.init
Proves: Project initialization with config generation
Critical Attributes:
  - project.path (proves target location)
  - config.generated (proves .clnrm.toml created)
  - config.path (cannot exist without file write)
```

### health - System Health Check
```yaml
Span: clnrm.cli.health
Proves: System health verification
Critical Attributes:
  - health.overall (healthy/degraded/unhealthy)
  - health.checks_total (must equal passed + failed)
  - docker.available (proves Docker check)
```

### plugins - Plugin Discovery
```yaml
Span: clnrm.cli.plugins
Proves: Plugin listing/discovery
Critical Attributes:
  - plugins.discovered (total count)
  - plugins.by_type (JSON map)
  - plugins.builtin + plugins.custom = plugins.discovered
```

### services - Service Management
```yaml
Span: clnrm.cli.services
Proves: Service status/logs/restart operations
Critical Attributes:
  - service.operation (status/logs/restart)
  - service.name (for logs/restart)
  - services.running + stopped + error = total
```

### collector - OTEL Collector Management
```yaml
Span: clnrm.cli.collector
Proves: Collector lifecycle management
Critical Attributes:
  - collector.operation (up/down/status/logs)
  - collector.running (current state)
  - collector.http_port (for up operation)
```

### fmt - Template Formatting
```yaml
Span: clnrm.cli.fmt
Proves: Template formatting operation
Critical Attributes:
  - files.formatted + unchanged + errors = input count
  - check_mode.enabled (whether --check used)
  - idempotency.verified (if --verify used)
```

### render - Template Rendering
```yaml
Span: clnrm.cli.render
Proves: Tera template rendering
Critical Attributes:
  - template.path (template file)
  - variables.count (variables provided)
  - output.size_bytes (proves rendering occurred)
```

### record - Baseline Recording
```yaml
Span: clnrm.cli.record
Proves: Test baseline capture for reproducibility
Critical Attributes:
  - tests.recorded (count)
  - baseline.digest (SHA-256)
  - baseline.path (output file)
```

### pull - Image Pre-pulling
```yaml
Span: clnrm.cli.pull
Proves: Docker image pre-pulling
Critical Attributes:
  - images.pulled + failed + skipped = discovered
  - parallel.enabled (parallel mode)
  - parallel.jobs (worker count)

Child Span: clnrm.cli.pull.image
  - image.name (full name)
  - image.digest (SHA256 after pull)
  - pull.duration_ms (individual pull time)
```

### red-green - TDD Workflow Validation
```yaml
Span: clnrm.cli.red_green
Proves: Red->Green TDD cycle validation
Critical Attributes:
  - tdd.expected_state (red/green)
  - tdd.actual_state (actual result)
  - tdd.validation_passed (expected == actual)
```

### repro - Test Reproduction
```yaml
Span: clnrm.cli.repro
Proves: Test reproduction from baseline
Critical Attributes:
  - baseline.digest (baseline SHA-256)
  - digest.verified (match result)
  - tests.diverged (different results)
```

## Metrics Quick Reference

### clnrm.cli.command.duration
- Type: Histogram (ms)
- Purpose: CLI command execution time distribution
- Attributes: cli.command, operation.success

### clnrm.cli.command.count
- Type: Counter ({invocation})
- Purpose: CLI command invocation count
- Attributes: cli.command, operation.success, exit.code

### clnrm.cli.file.operations
- Type: Counter ({file})
- Purpose: File operations count
- Attributes: cli.command, operation.type, operation.success

### clnrm.cli.image.pull.size
- Type: Histogram (By)
- Purpose: Image pull size distribution
- Attributes: image.registry, pull.success

## Events Quick Reference

| Event | When | Critical Attributes |
|-------|------|-------------------|
| `clnrm.cli.command.started` | Command starts | cli.command, cli.args |
| `clnrm.cli.command.completed` | Command succeeds | cli.command, duration_ms, exit.code |
| `clnrm.cli.command.failed` | Command fails | cli.command, error.type, error.message |
| `clnrm.cli.config.missing` | Config not found | cli.command, config.path, config.type |
| `clnrm.cli.validation.failed` | Weaver validation fails | cli.command, validation.errors |

## Validation Patterns

### Command Proof Pattern
```yaml
MUST HAVE:
  - cli.command exists
  - operation.duration_ms > 0
  - operation.success is set
```

### Error Handling Pattern
```yaml
IF operation.success = false THEN:
  - error.type MUST exist
  - error.message MUST exist
```

### Count Balance Pattern
```yaml
FOR commands with counts (services, plugins, pull):
  - Subcounts MUST sum to total count
  - Example: services.running + stopped + error = total
```

### File Operation Pattern
```yaml
FOR file operations (fmt, render, record):
  - Input count MUST be tracked
  - Output result MUST be tracked
  - Success/failure ratio MUST be calculable
```

## Implementation Priority

### Phase 1: High Priority (Week 1)
1. **init** - Most critical, foundation for all projects
2. **health** - Critical for validation and debugging

### Phase 2: Medium Priority (Week 2)
3. **plugins** - Core feature listing
4. **services** - Service management operations
5. **collector** - OTEL infrastructure management
6. **fmt** - Template formatting (v0.7.0 feature)
7. **render** - Template rendering (v0.7.0 feature)
8. **record** - Baseline recording (v0.7.0 feature)

### Phase 3: Low Priority (Week 3)
9. **pull** - Image pre-pulling optimization
10. **red-green** - TDD workflow validation
11. **repro** - Test reproduction

## Validation Commands

### Validate Schema Files
```bash
weaver registry check -r registry/
```

### Generate Code
```bash
weaver generate \
  --registry registry/ \
  --template rust \
  --output crates/clnrm-core/src/telemetry/generated/cli/
```

### Live-Check During Execution
```bash
# Terminal 1: Start collector
docker run -p 4318:4318 otel/opentelemetry-collector

# Terminal 2: Run commands
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
  cargo run --features otel -- init

# Terminal 3: Validate
weaver registry live-check \
  --registry registry/ \
  --endpoint http://localhost:4318/v1/traces
```

## Testing Checklist Per Command

- [ ] Schema YAML created
- [ ] Weaver validation passes
- [ ] Rust types generated
- [ ] Span builder implemented
- [ ] Command instrumented
- [ ] Unit tests written
- [ ] Integration tests written
- [ ] Weaver live-check passes
- [ ] Documentation updated
- [ ] CI/CD configured

## Success Metrics

| Metric | Before | After |
|--------|--------|-------|
| CLI commands instrumented | 11/23 (48%) | 22/23 (96%) |
| Schema files | 3 | 10 |
| Weaver coverage | Core only | Full CLI |
| CI/CD validation | Manual | Automated |

## Common Pitfalls

1. **Missing duration_ms**: Every span MUST track duration
2. **Unset operation.success**: Every span MUST set final result
3. **Missing error attributes**: Failures MUST have error.type and error.message
4. **Count imbalance**: Subcounts must sum to total (services, plugins, pull)
5. **Missing conditionally_required**: Error attributes required when operation.success = false

## References

- **Full Architecture**: `docs/weaver/CLI_SCHEMA_ARCHITECTURE.md` (1105 lines)
- **Existing Schemas**: `registry/core/`, `registry/metrics/`, `registry/events/`
- **Validation Strategy**: `registry/VALIDATION_STRATEGY.md`
- **Implementation Guide**: Section 4 of Architecture document

---

**Status:** Ready for schema file creation and implementation
**Estimated Effort:** 3-4 weeks full implementation
**Risk Level:** LOW (following proven patterns)
