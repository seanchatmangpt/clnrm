# How-To Guides - Solve Specific Problems

How-to guides are **task-oriented, practical solutions** for concrete problems. Use these when you want to accomplish something specific.

## Quick Navigation

**Know what you want to do? Find it here:**

### Execution & Performance
- **[Run tests in parallel](./parallel-execution.md)** — Execute multiple tests concurrently for faster feedback
- **[Enable container pooling](./container-pooling-setup.md)** — Get 80% faster test startup with pre-warmed containers
- **[Optimize for your workload](./performance-tuning.md)** — Tune pool size, concurrency, and timeouts
- **[Stress test your setup](./stress-testing.md)** — Load test with hundreds/thousands of concurrent tests
- **[Monitor test performance](./performance-monitoring.md)** — Collect and analyze metrics

### CI/CD Integration
- **[Use with GitHub Actions](./github-actions.md)** — Integrate clnrm into GitHub workflows
- **[Use with GitLab CI](./gitlab-ci.md)** — Integrate clnrm into GitLab pipelines
- **[Use with Jenkins](./jenkins.md)** — Integrate clnrm into Jenkins jobs
- **[Generate test reports](./test-reporting.md)** — Create JUnit XML, HTML reports
- **[Fail fast in CI](./ci-fail-fast.md)** — Stop on first failure for quick feedback

### Configuration & Customization
- **[Use environment variables](./environment-variables.md)** — Configure via `CLNRM_*` and `OTEL_*` vars
- **[Template variables](./template-variables.md)** — Parameterize TOML with Tera templates
- **[Multi-environment testing](./multi-environment.md)** — Test dev/staging/prod configurations
- **[Use different backends](./container-backends.md)** — Docker, Podman, testcontainers
- **[Configure OTEL export](./otel-configuration.md)** — Export to Jaeger, DataDog, New Relic

### Testing Patterns
- **[Test with databases](./database-testing.md)** — PostgreSQL, MongoDB, SurrealDB patterns
- **[Test APIs](./api-testing.md)** — HTTP endpoint testing patterns
- **[Test microservices](./microservice-testing.md)** — Multi-service orchestration
- **[Test with custom services](./custom-service-testing.md)** — Write service plugins
- **[Hermetic testing patterns](./hermetic-patterns.md)** — Best practices for isolated tests

### Advanced Topics
- **[Write Weaver schemas](./weaver-schemas.md)** — Define OpenTelemetry validation schemas
- **[Custom validation](./custom-validators.md)** — Extend validation beyond built-in rules
- **[Plugin development](./plugin-development.md)** — Create and register service plugins
- **[Determinism testing](./determinism-testing.md)** — Ensure reproducible test results
- **[Chaos engineering](./chaos-engineering.md)** — Test resilience with failure injection

### Troubleshooting
- **[Fix Docker issues](./troubleshooting/docker.md)** — Docker daemon, socket, networking
- **[Debug test failures](./troubleshooting/debug.md)** — Logging, tracing, inspection tools
- **[Handle flaky tests](./troubleshooting/flaky-tests.md)** — Timeout tuning, retry logic
- **[Fix validation failures](./troubleshooting/validation.md)** — Schema and expectation errors
- **[Resolve performance issues](./troubleshooting/performance.md)** — Memory leaks, timeouts, bottlenecks

### Upgrading & Migration
- **[Migrate from v1.3 to v1.4](./migrate-v1.3-to-v1.4.md)** — Breaking changes, new features
- **[Migrate from v1.4.0 to v1.4.1](./migrate-v1.4.0-to-v1.4.1.md)** — Pool improvements
- **[Deprecated features](./deprecated-features.md)** — What's no longer supported

---

## What Makes a How-To Guide

How-to guides in Diataxis are:
- ✅ **Task-oriented** — Solve a specific problem, not teach a concept
- ✅ **Practical** — Copy-paste solutions, real code/config
- ✅ **Assume knowledge** — Readers know clnrm basics
- ✅ **Answer "How do I..."** — Focused on the task, not the "why"
- ✅ **Immediate value** — Reader can accomplish goal after reading

They're **NOT**:
- ❌ Tutorials (those teach from scratch)
- ❌ Reference (those look up details)
- ❌ Explanations (those teach concepts)

---

## How to Use These Guides

1. **Find your task** — Use the quick navigation above or search
2. **Read the problem statement** — Understand the scenario
3. **Follow the steps** — Copy/paste code and commands
4. **Customize for your needs** — Adapt the solution
5. **See also** section — Related guides and reference docs

---

## Don't See What You Need?

- **General question?** → Check [Tutorials](../tutorials/) or [Explanations](../explanation/)
- **Need technical details?** → Check [Reference Docs](../reference/)
- **New to clnrm?** → Start with [Getting Started](../GETTING_STARTED.md)
- **Have a feature request?** → [Open an issue](https://github.com/seanchatmangpt/clnrm/issues)

---

## Guide Categories at a Glance

| Category | Purpose | Examples |
|----------|---------|----------|
| **Execution & Performance** | Run and optimize tests | Parallel, pooling, stress testing |
| **CI/CD Integration** | Integrate with automation | GitHub Actions, Jenkins, GitLab |
| **Configuration** | Customize behavior | Environment variables, templates, backends |
| **Testing Patterns** | Test specific things | Databases, APIs, microservices |
| **Advanced** | Go beyond basics | Schemas, validators, plugins |
| **Troubleshooting** | Fix problems | Errors, failures, debugging |
| **Migration** | Upgrade versions | v1.3→v1.4, breaking changes |

---

**Ready to get something done?** Pick a guide above and get started!
