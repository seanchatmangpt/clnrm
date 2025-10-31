# Introduction

Welcome to the **Advanced Users Guide** for the Cleanroom Testing Framework (clnrm). This guide is designed for developers who want to extend clnrm's capabilities, implement complex testing patterns, and deploy clnrm in production environments.

## What You'll Learn

This guide covers four main areas that provide the highest value for advanced users:

### 🔌 Plugin Development
Learn how to create custom service plugins, integrate with external systems, and extend clnrm's capabilities. This is where you'll get the most value - extending the framework to meet your specific needs.

### 🧪 Advanced Testing Patterns
Master multi-service orchestration, chaos engineering, OTEL validation, and performance testing. These patterns enable testing of complex distributed systems.

### 📝 Template System Mastery
Deep dive into Tera templates, macro libraries, and variable resolution. Create reusable, maintainable test configurations.

### 🚀 Production Deployment
Deploy clnrm in CI/CD pipelines, optimize performance, and implement enterprise-scale patterns.

## Prerequisites

Before diving into this guide, you should have:

- **Basic clnrm knowledge**: Familiarity with `.clnrm.toml` files and basic commands
- **Rust experience**: Understanding of Rust syntax and concepts (for plugin development)
- **Container knowledge**: Basic understanding of Docker/Podman
- **Testing experience**: Familiarity with integration testing concepts

## Core Team Standards

This guide follows FAANG-level quality standards established by the clnrm core team:

### Code Quality Standards
- ✅ **Zero unwrap()/expect()** in production examples
- ✅ **All traits dyn-compatible** for object safety
- ✅ **Proper error handling** with CleanroomError
- ✅ **OTEL instrumentation** in all examples
- ✅ **AAA pattern** in test examples
- ✅ **Descriptive naming** following conventions

### Documentation Standards
- ✅ **Honest documentation** - only features that actually work in v1.2.1
- ✅ **Runnable examples** - every code sample can be executed
- ✅ **Self-testing** - examples are validated by clnrm itself
- ✅ **Verification commands** - clear steps to validate examples
- ✅ **No false positives** - incomplete features use `unimplemented!()` with clear messages

## What's New in v1.2.1: Production-Ready Schema Validation

**clnrm v1.2.1** delivers a production-ready approach to testing: **Weaver-based schema validation as the source of truth**. This solves the false positive problem that plagues traditional testing with complete implementation of health checks, test coverage, and validation gates.

### 🎯 The False Positive Problem

```
Traditional Testing:
  Test passes ✅ → Assumes feature works → FALSE POSITIVE ❌
  └─ Tests validate test code, not production behavior

clnrm v1.2.1 with Weaver:
  Test passes ✅ + Weaver validates schema ✅ → TRUE POSITIVE ✅
  └─ Schema validation proves actual runtime behavior
```

### ✅ Production-Ready Features (v1.2.1)

- **Weaver Live-Check Integration**: Real-time schema validation with HTTP health checks (replaces hardcoded sleeps)
- **Schema Registry**: 207 validated schema files, comprehensive semantic conventions
- **False Positive Detection**: Zero-sample validation prevents tests passing without telemetry
- **Health Check Integration**: Proper HTTP health checks via admin port (exponential backoff, 30s timeout)
- **Complete Test Coverage**: Phase 2 and Phase 3 coordination tests fully implemented
- **CI/CD Reliability**: Removed test masking (`|| true` patterns), failures actually fail CI
- **Config-Based Execution**: Dynamic image selection from cleanroom config (no hardcoded defaults)
- **Plugin System**: Custom service plugins with container integration
- **Template System**: Tera templates with macro library
- **Container Execution**: Commands run in isolated containers with lifecycle tracking
- **Multi-Service Testing**: Orchestrate multiple services with dependency management
- **CI/CD Integration**: GitHub Actions gates with Weaver validation

### 🚀 Production-Ready Implementation (v1.2.1)

- **WeaverController**: Complete implementation with health checks (900+ lines, production-ready)
- **Schema Registry**: `registry/` with comprehensive semantic conventions
- **80/20 Validation**: 4 critical attributes prove 80% of functionality
- **Zero False Positives**: Sample count validation prevents fake-green tests
- **Live Telemetry Validation**: Runtime OTLP validation against schemas with proper coordination
- **Test Coverage**: Phase 2 (coordination) and Phase 3 (OTEL integration) tests complete

### 📊 Validation Hierarchy

```
1. Weaver Schema Validation (HIGHEST AUTHORITY)
   ├─ Runtime telemetry MUST match schemas
   ├─ Exit code 1 = FAIL BUILD
   └─ Source of truth for production readiness

2. Compilation + Type Safety (SECOND AUTHORITY)
   ├─ Code must compile
   ├─ Type-safe telemetry builders
   └─ Zero clippy warnings

3. Traditional Tests (SUPPORTING EVIDENCE)
   ├─ Can have false positives
   ├─ Not sole source of truth
   └─ Validated by Weaver
```

### ❌ Known Limitations (v1.2.1)

- **AI Features**: Available in experimental `clnrm-ai` crate only
- **Hot Reload**: Planned for future versions
- **Marketplace Features**: Experimental, uses `unimplemented!()` for incomplete operations
- **Phase 4 E2E Tests**: Docker integration tests stubbed (require Docker environment)

## How to Use This Guide

### 1. Start with Plugin Development
If you want to extend clnrm, start with the [Plugin Development](plugin-development/README.md) section. This provides the highest value and most practical benefits.

### 2. Learn Advanced Patterns
Once you understand plugins, explore [Advanced Testing Patterns](advanced-patterns/README.md) for complex testing scenarios.

### 3. Master Templates
For reusable configurations, dive into [Template System Mastery](template-mastery/README.md).

### 4. Deploy in Production
Finally, learn [Production Deployment](production-deployment/README.md) strategies for CI/CD and enterprise use.

## Running Examples

Every example in this guide is validated and runnable:

```bash
# Validate all examples
bash tests/mdbook-examples/validate-all-examples.sh

# Run specific examples
cargo test --test mdbook-examples-plugin-development
```

## Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/seanchatmangpt/clnrm/issues)
- **Core Team Standards**: See [CLAUDE.md](../CLAUDE.md) for development guidelines
- **Architecture Docs**: See [docs/architecture/](../docs/architecture/) for deep technical details
- **Examples**: See [examples/](../examples/) for working code samples

## Contributing

Contributions to this guide are welcome! When contributing:

1. Follow core team standards (no unwrap/expect in production code)
2. Include runnable examples in `tests/mdbook-examples/`
3. Validate all examples work with `clnrm`
4. Update this introduction if adding new major sections

---

**Ready to get started?** Begin with [Plugin Development](plugin-development/README.md) to learn how to extend clnrm's capabilities.
