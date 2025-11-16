# Explanations - Understand Concepts

Explanations are **understanding-oriented guides** about "why" and design principles. Use these to deepen your knowledge and understanding of clnrm.

## Conceptual Guides

### Architecture & Design
- **[System Architecture](./architecture.md)** — How clnrm works end-to-end
  - Component overview
  - Data flow through the system
  - Key abstractions (traits, plugins)
  - Design patterns and principles
  - Why each component exists

- **[Plugin System](./plugin-system.md)** — Why plugins and how they work
  - Plugin philosophy (extensibility over hardcoding)
  - ServicePlugin trait design
  - Plugin lifecycle (start, stop, health check)
  - Plugin discovery and registration
  - Creating custom plugins

### Core Concepts
- **[Weaver Schema Validation](./weaver-validation.md)** — Why behavior validation matters
  - The false-positive problem in testing
  - How schema validation works
  - Why OpenTelemetry is the source of truth
  - How Weaver catches fake-green tests
  - Integration with test execution

- **[Container Pooling](./container-pooling.md)** — How 80% speedup works
  - Why startup was slow (2-5 seconds)
  - How pooling solves it (0.1-0.5ms acquisition)
  - Pre-warming strategy and FIFO queue
  - Background health check worker
  - Hit rate optimization (92%+ target)
  - Trade-offs (memory vs. speed)

- **[Concurrency Model](./concurrency.md)** — How parallel tests work
  - Why uncontrolled concurrency is bad
  - Semaphore-based fairness model
  - Job limiting and backpressure
  - Lock-free hot paths (DashMap)
  - Scaling limits and bottlenecks

### Principles & Best Practices
- **[Hermiticity](./hermiticity.md)** — Test isolation principles
  - What hermiticity means
  - Why isolation matters
  - Docker as isolation mechanism
  - Hermetic test patterns
  - Validating hermiticity through telemetry

- **[Determinism](./determinism.md)** — Reproducible test results
  - Non-determinism sources
  - Deterministic test design
  - Random seed control
  - Timing-dependent tests
  - Validation approaches

- **[Performance Characteristics](./performance.md)** — Understanding scaling
  - Startup overhead breakdown
  - Throughput limits
  - Resource requirements
  - Scaling laws and bottlenecks
  - Optimization opportunities

### Advanced Topics
- **[OpenTelemetry Integration](./otel-integration.md)** — Observability architecture
  - What telemetry gets emitted
  - Export formats (OTLP, stdout)
  - Resource attributes and context
  - Span structure and relationships
  - Trace propagation

- **[False Positives in Testing](./false-positives.md)** — The problem clnrm solves
  - Types of false positives
  - Why traditional testing fails
  - How schema validation helps
  - Detecting fake-green tests
  - Building more reliable test suites

---

## Learning Paths

### For New Users
Start here to understand clnrm deeply:
1. [System Architecture](./architecture.md) — Understand components
2. [Plugin System](./plugin-system.md) — Understand extensibility
3. [Weaver Validation](./weaver-validation.md) — Understand behavior validation

### For Performance-Focused Users
Learn how to optimize:
1. [Container Pooling](./container-pooling.md) — How pooling works
2. [Concurrency Model](./concurrency.md) — How parallel tests work
3. [Performance Characteristics](./performance.md) — Scaling and limits

### For Advanced Users
Master the advanced topics:
1. [Hermiticity](./hermiticity.md) — Test isolation patterns
2. [Determinism](./determinism.md) — Reproducible tests
3. [OpenTelemetry Integration](./otel-integration.md) — Deep observability

---

## What Makes an Explanation

Explanations in Diataxis are:
- ✅ **Understanding-oriented** — Goal is comprehension, not action
- ✅ **Conceptual** — Teach ideas and principles, not procedures
- ✅ **Broad context** — Explain "why" not just "what"
- ✅ **Discuss trade-offs** — Show alternatives and decisions
- ✅ **No procedures** — No step-by-step instructions
- ✅ **Depth and nuance** — For users who want to understand deeply

They're **NOT**:
- ❌ Tutorials (those teach step-by-step)
- ❌ How-to guides (those solve specific problems)
- ❌ Reference (those look up details)

---

## Quick Reference

**When to read each explanation:**

| Explanation | Read When You Want To... |
|-------------|------------------------|
| **System Architecture** | Understand how clnrm works holistically |
| **Plugin System** | Know how to create or extend plugins |
| **Weaver Validation** | Understand why behavior validation matters |
| **Container Pooling** | Grasp how performance works |
| **Concurrency Model** | Learn about parallel test execution |
| **Hermiticity** | Master test isolation |
| **Determinism** | Ensure reproducible results |
| **Performance Characteristics** | Understand scaling and limits |
| **OTEL Integration** | Know what telemetry is emitted |
| **False Positives** | Understand the testing problem clnrm solves |

---

## Reading Patterns

### The Curious Developer
1. Start with [System Architecture](./architecture.md)
2. Jump to topics that interest you
3. Follow "see also" links

### The Pragmatist
1. Read the explanation for your specific need
2. Apply knowledge to your use case
3. Reference [How-To Guides](../how-to/) for practical steps

### The Theorist
1. Read explanations in order (learning paths above)
2. Study design decisions and trade-offs
3. Explore related concepts deeply

---

## Key Concepts at a Glance

**Weaver Validation**: Schema-first validation catches fake-green tests that traditional testing misses by validating actual runtime behavior.

**Container Pooling**: Pre-warmed containers reduce test startup from 2-5s to 0.1-0.5ms through FIFO queue management and background health checks.

**Hermiticity**: Tests run in isolated Docker containers with no cross-test pollution, validatable through telemetry inspection.

**Determinism**: Reproducible test results through controlled randomness, seed management, and timing-independent assertions.

**Concurrency**: Semaphore-based fairness limiting with lock-free hot paths enables 10x throughput (500-1000 concurrent tests).

---

## Next Steps

1. **Choose a topic** — Pick an explanation above that interests you
2. **Understand the concept** — Read and think about the ideas
3. **Apply the knowledge** — Use [How-To Guides](../how-to/) for practical steps
4. **Refer to details** — Use [Reference Docs](../reference/) for exact syntax

---

## See Also

- **Need step-by-step instructions?** → [Tutorials](../tutorials/) and [How-To Guides](../how-to/)
- **Need to look up details?** → [Reference Docs](../reference/)
- **Ready to learn more?** → Pick an explanation above

---

**Ready to deepen your understanding?** Pick a topic above and explore!
