# Diataxis Documentation Restructure Plan

## Overview

This plan reorganizes clnrm documentation using the [Diataxis framework](https://diataxis.fr/), which divides documentation into 4 modes based on two axes:
- **Procedural** ↔ **Conceptual** (how users approach)
- **Specific** ↔ **General** (what the content covers)

This creates 4 categories:
1. **Tutorials** - Learning-oriented, hands-on, specific examples
2. **How-to Guides** - Task-oriented, practical, addressing specific problems
3. **Reference** - Information-oriented, technical, general lookup
4. **Explanation** - Understanding-oriented, conceptual, general principles

## Current State Assessment

**Problems with current structure (177 files):**
- ❌ No clear Diataxis organization
- ❌ 20+ agent reports mixed with user documentation
- ❌ Version duplication (v1.2.1, v1.3.0, v1.4.0 docs)
- ❌ Historical reports clutter main docs
- ❌ Users don't know where to start
- ❌ Scattered configuration examples
- ❌ No clear conceptual guides

**What works well:**
- ✅ Comprehensive technical content exists
- ✅ Good CLI reference (CLI_GUIDE.md)
- ✅ TOML configuration documented
- ✅ Performance info available (PERFORMANCE_TUNING.md)

## Target Documentation Structure

```
/docs
  /tutorials                    # Learn by doing
    /01-getting-started        # Your first test
    /02-container-pooling      # Enable 80% speedup
    /03-weaver-validation      # Ensure behavior correctness
    /04-custom-plugins         # Extend clnrm
    /05-otel-integration       # Add observability
  /how-to                       # Problem-solving guides
    /parallel-execution        # Run tests concurrently
    /ci-cd-integration         # Use in GitHub Actions
    /migrations                # Upgrade from previous versions
    /performance-optimization  # Tune for your workload
    /troubleshooting           # Solve common problems
    /custom-plugins            # Create service plugins
    /weaver-schemas            # Write validation schemas
    /docker-alternatives       # Use Podman or other backends
  /reference                    # Technical lookup
    /cli                       # Command-line interface
    /toml-schema               # Configuration format
    /api                       # Rust API documentation
    /environment-variables     # Configuration via env vars
    /plugins                   # Built-in plugins
    /otel-attributes           # OpenTelemetry span attributes
  /explanation                  # Understand concepts
    /architecture              # System design overview
    /weaver-validation         # Why schema validation matters
    /container-pooling         # How pooling works
    /concurrency-model         # Concurrent test execution
    /plugin-system             # Plugin architecture
    /hermiticity              # Test isolation principles
    /false-positives          # The fake-green problem
  /README.md                    # Main entry point (rewritten)
  /index.md                     # Documentation hub
  /GETTING_STARTED.md           # Quick 5-minute start
/README.md                      # Project root (rewritten)
/CHANGELOG.md                   # Release notes
/ARCHITECTURE.md                # (moved from docs/architecture/)
/archive/                       # Historical documents
```

## Documentation Organization

### 1. TUTORIALS (Learning-oriented, hands-on)

**Purpose:** Guide users step-by-step through concrete examples to understand clnrm.

#### Tutorial 1: Getting Started (15 minutes)
- **Goals:** User runs first test successfully, understands basic concepts
- **Sections:**
  - Quick install (brew/cargo)
  - Initialize project (`clnrm init`)
  - Write simple TOML test
  - Run test (`clnrm run`)
  - Understand results
- **Outcome:** User has working test, ready for advanced features

#### Tutorial 2: Container Pooling Performance (10 minutes)
- **Goals:** User enables pooling, sees 80% speedup
- **Sections:**
  - Understand pooling benefit (2-5s → 0.1-0.5ms)
  - Enable with `CLNRM_ENABLE_POOLING=1`
  - Configure pool size, idle timeout
  - Benchmark before/after
  - Profile with provided tools
- **Outcome:** User optimized test suite, 10x throughput improvement

#### Tutorial 3: Weaver Validation (15 minutes)
- **Goals:** User adds schema validation, catches false positives
- **Sections:**
  - Understand false-positive problem
  - Install/configure Weaver
  - Add `[weaver]` section to TOML
  - Write first schema in registry
  - Run validation (`--live-check`)
  - Interpret validation report
- **Outcome:** User understands why behavior validation matters, has schema validation working

#### Tutorial 4: Custom Service Plugins (20 minutes)
- **Goals:** User creates simple plugin for custom service
- **Sections:**
  - Understand ServicePlugin trait
  - Copy/modify example plugin
  - Register in plugin registry
  - Use in test TOML
  - Test plugin behavior
  - Publish plugin
- **Outcome:** User can extend clnrm with custom services

#### Tutorial 5: OpenTelemetry Setup (15 minutes)
- **Goals:** User exports telemetry to observability backend
- **Sections:**
  - Understand OTEL importance
  - Configure OTLP endpoint
  - Add resource attributes
  - Export to Jaeger/DataDog/New Relic
  - Inspect traces in UI
  - Understand sampling and propagators
- **Outcome:** User can integrate with observability platform

### 2. HOW-TO GUIDES (Task-oriented, practical)

**Purpose:** Answer "How do I do X?" with practical, copy-paste solutions.

#### Execution & Performance
- **Run tests in parallel** - Use `--parallel --jobs N`
- **Optimize container startup** - Container pooling config
- **Scale to 1000 concurrent tests** - Resource requirements, tuning
- **Profile test performance** - Built-in metrics, flamegraph
- **Reduce memory footprint** - Pool sizing, cleanup
- **Monitor test health** - Metrics collection, dashboards

#### Integration & CI/CD
- **Integrate with GitHub Actions** - Workflow example
- **Integrate with GitLab CI** - Pipeline example
- **Integrate with Jenkins** - Jenkinsfile example
- **Generate JUnit XML reports** - CI system compatibility
- **Fail on first error** - `-x` flag behavior
- **Collect test artifacts** - Logs, traces, screenshots

#### Configuration & Customization
- **Configure different backends** - Docker, Podman, testcontainers
- **Use environment variables** - `CLNRM_*` var reference
- **Template variables** - Tera syntax, examples
- **Custom TOML schemas** - Extend base schema
- **Multi-environment testing** - Dev/staging/prod configs
- **Hermetic testing patterns** - Isolation best practices

#### Troubleshooting
- **Fix Docker connection issues** - Daemon, socket, permissions
- **Debug test failures** - Log levels, trace output
- **Handle flaky tests** - Timeout tuning, retry logic
- **Fix memory leaks** - Container cleanup, pool limits
- **Understand timing failures** - Span duration assertions
- **Handle schema validation failures** - Common mistakes

#### Upgrade & Migration
- **Migrate v1.3 to v1.4** - Breaking changes, TOML updates
- **Migrate v1.4.0 to v1.4.1** - Pool improvements, config changes
- **Handle deprecated features** - Backwards compatibility notes
- **Update custom plugins** - API changes, trait updates

#### Advanced Topics
- **Write Weaver schemas** - OpenTelemetry semantic conventions
- **Custom validators** - Add validation rules
- **Plugin composition** - Multi-service orchestration
- **Stress testing** - Load testing patterns, limits
- **Determinism validation** - Reproducible results

### 3. REFERENCE (Information-oriented, technical)

**Purpose:** Look up specific technical details, complete specifications.

#### CLI Reference
- **Commands overview** - All `clnrm` commands listed
- **Command reference** - Each command with options, examples
- **Flags reference** - All flags and short forms
- **Exit codes** - What each exit code means
- **Help text** - Complete CLI help as reference

#### Configuration Reference
- **TOML schema** - Complete configuration structure
  - `[meta]` section
  - `[services]` section (service definitions)
  - `[[scenario]]` section (test steps)
  - `[expect.*]` sections (all validation types)
  - `[weaver]` section
  - `[otel]` section
- **Variable reference** - All template variables available
- **Attributes reference** - OpenTelemetry attribute names

#### API Reference
- **Crate documentation** - Rustdoc for `clnrm-core`
- **Trait reference** - ServicePlugin, Backend, Validator
- **Configuration types** - Serde-based config structures
- **Error types** - All error variants
- **Module organization** - Where to find what

#### Environment Variables
- **Configuration vars** - `CLNRM_*` variables
- **Pooling vars** - `CLNRM_ENABLE_POOLING`, etc.
- **OTEL vars** - `OTEL_EXPORTER_OTLP_*`, etc.
- **Testing vars** - `RUST_LOG`, `RUST_BACKTRACE`

#### Built-in Plugins
- **GenericContainer** - Run any Docker image
- **SurrealDB** - Database service plugin
- **LLM Plugins** - Ollama, vLLM, TGI
- **ChaosEngine** - Chaos engineering plugin
- **ServiceManager** - Orchestrate multiple services

### 4. EXPLANATION (Understanding-oriented, conceptual)

**Purpose:** Help users understand "why?" and design principles.

#### Architecture & Design
- **System architecture** - Component overview, interactions
- **Test execution flow** - How a test runs from TOML to completion
- **Container management** - Docker integration, lifecycle
- **Plugin system** - Why plugins, extensibility model
- **Configuration system** - TOML loading, defaults, precedence

#### Core Concepts
- **Weaver schema validation** - Why behavior validation matters
  - Problem: False positives in traditional testing
  - Solution: Schema-first validation against telemetry
  - Benefits: Catches fake-green tests, ensures correct behavior
  - Integration: How Weaver validates runtime telemetry
- **Container pooling** - Why 80% faster startup
  - Problem: 2-5s startup overhead per test
  - Solution: Pre-warm containers, FIFO queue, background health checks
  - Benefits: 0.1-0.5ms acquisition, 10x throughput, 92%+ hit rate
  - Trade-offs: Memory usage, resource management
- **Concurrency model** - Semaphore-based execution
  - Problem: Uncontrolled concurrency overwhelms system
  - Solution: Semaphore-based fairness, job limiting
  - Benefits: Predictable resource usage, fair scheduling
  - Tuning: Job count, timeouts, backpressure
- **Hermiticity** - Test isolation principles
  - Problem: Tests affecting each other, hidden dependencies
  - Solution: Docker isolation, filesystem separation
  - Validation: Telemetry checking for external calls
  - Best practices: Service discovery patterns

#### Advanced Concepts
- **Determinism engine** - Reproducible test results
- **OTEL instrumentation** - What gets traced, how
- **Stress testing** - Behavior under load
- **Performance characteristics** - Scaling limits, bottlenecks

## Implementation Roadmap

### Phase 1: Foundation (Days 1-2)
1. ✅ Create Diataxis directory structure
2. Rewrite main README with Diataxis guidance
3. Create documentation/index.md hub
4. Create GETTING_STARTED.md (5-minute start)

### Phase 2: Tutorials (Days 3-4)
1. Getting Started (15 min)
2. Container Pooling (10 min)
3. Weaver Validation (15 min)
4. Custom Plugins (20 min)
5. OTEL Integration (15 min)

### Phase 3: How-To Guides (Days 5-6)
1. Execution & Performance (5 guides)
2. Integration & CI/CD (6 guides)
3. Configuration (6 guides)
4. Troubleshooting (6 guides)
5. Advanced Topics (5 guides)

### Phase 4: Reference (Days 7-8)
1. CLI Reference (extract/update)
2. Configuration Reference (comprehensive)
3. API Reference (Rustdoc)
4. Environment Variables (all)
5. Plugins (all built-in plugins)

### Phase 5: Explanation (Days 9-10)
1. Architecture overview
2. Weaver validation deep-dive
3. Container pooling deep-dive
4. Concurrency model
5. Plugin system

### Phase 6: Cleanup & Polish (Days 11-12)
1. Archive old documentation
2. Update links throughout
3. Create migration guide (old → new structure)
4. Validation and testing

## File Movement Strategy

### Keep (Archive)
```
docs/archive/               # All historical documents
  /reports/                # Agent reports, completion reports
  /versions/               # Version-specific docs
  /releases/               # Release notes archive
```

### Update
```
README.md                   # Rewrite with Diataxis framing
docs/index.md             # Main hub
docs/GETTING_STARTED.md   # New 5-min start
```

### Migrate
```
docs/CLI_GUIDE.md         → docs/reference/cli.md
docs/TOML_REFERENCE.md    → docs/reference/toml-schema.md
docs/PERFORMANCE_TUNING.md → docs/how-to/performance-optimization.md
docs/CONTAINER_POOLING.md → docs/explanation/container-pooling.md
docs/TESTING.md           → docs/how-to/testing-patterns.md
docs/SECURITY.md          → docs/reference/security.md (keep in root)
```

### Create New
```
docs/tutorials/01-getting-started/
docs/tutorials/02-container-pooling/
docs/tutorials/03-weaver-validation/
docs/tutorials/04-custom-plugins/
docs/tutorials/05-otel-integration/
docs/how-to/parallel-execution.md
docs/how-to/ci-cd-integration.md
docs/explanation/architecture.md
docs/explanation/weaver-validation.md
... (complete list above)
```

## Success Criteria

✅ **Organization**
- [ ] All 4 Diataxis quadrants have clear, separate sections
- [ ] No agent reports in user docs
- [ ] Old docs in `/archive/` with README explaining
- [ ] Clear navigation between sections

✅ **Content Quality**
- [ ] Each tutorial has learning objectives and "what you'll do"
- [ ] Each how-to has clear problem statement and solution
- [ ] Each reference section is complete and up-to-date
- [ ] Each explanation has conceptual diagrams or analogies

✅ **User Experience**
- [ ] New user can find "getting started" in <10 seconds
- [ ] User can find "how to do X" in <30 seconds
- [ ] User can find technical specs in <20 seconds
- [ ] README clearly directs to appropriate section

✅ **Maintainability**
- [ ] Clear ownership (which doc type to add)
- [ ] Consistent structure (all tutorials follow same format)
- [ ] Automated link checking (no broken references)
- [ ] Version update process documented

## Diataxis Checklist for Each Document

### Tutorial Checklist
- [ ] Concrete learning objectives
- [ ] Step-by-step instructions (no skipping)
- [ ] Real, working example throughout
- [ ] Only essential info (no tangents)
- [ ] "What you'll do" section at start
- [ ] Estimated time to complete
- [ ] Next tutorial to learn in sequence

### How-To Checklist
- [ ] Clear problem statement
- [ ] Prerequisites listed
- [ ] Step-by-step solution
- [ ] One specific task per guide
- [ ] Copy-paste friendly code/config
- [ ] Related how-tos linked
- [ ] When to use (vs other approaches)

### Reference Checklist
- [ ] Complete, accurate information
- [ ] Consistent structure/format
- [ ] Examples where helpful
- [ ] Cross-referenced
- [ ] Searchable
- [ ] No narrative/explanation (facts only)
- [ ] Version notes if applicable

### Explanation Checklist
- [ ] Big picture perspective
- [ ] "Why this matters" section
- [ ] No step-by-step procedures
- [ ] Includes design rationale
- [ ] Conceptual diagrams helpful
- [ ] Links to how-to guides
- [ ] Discusses trade-offs
- [ ] Accessible to learners but deep for experts

## Next Steps

1. Review and approve this plan
2. Create directory structure: `/docs/tutorials/`, `/docs/how-to/`, etc.
3. Start Phase 1: Rewrite README and create index
4. Execute phases 2-5 systematically
5. Validate structure with user testing
6. Publish updated documentation

---

## References

- [Diataxis Framework](https://diataxis.fr/) - Official documentation
- [Divio's Implementation](https://docs.divio.com/) - Real-world example
- Current clnrm docs: `/home/user/clnrm/docs/`
- Codebase analysis: `CODEBASE_STRUCTURE_ANALYSIS.md`
