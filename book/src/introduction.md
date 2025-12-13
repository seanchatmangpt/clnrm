# Introduction

Welcome to the **Advanced Users Guide** for the Cleanroom Testing Framework v2.0.0 (clnrm). This guide is designed for developers who want to extend clnrm's capabilities, implement complex testing patterns, and deploy clnrm in production environments.

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
- ✅ **Honest documentation** - only features that actually work in v2.0.0
- ✅ **Runnable examples** - every code sample can be executed
- ✅ **Self-testing** - examples are validated by clnrm itself
- ✅ **Verification commands** - clear steps to validate examples
- ✅ **No false positives** - incomplete features use `unimplemented!()` with clear messages

## What's New in v2.0.0: Breaking Changes for Production-Ready Execution

**clnrm v2.0.0** delivers production-ready container execution with breaking changes that solve critical environment variable persistence and execution model issues.

### 🎯 The Execution Model Problem

**v1.x Execution Model (Broken):**
```
Container Creation → Environment Variables Set → Container Destroyed
     ↓
New Container Creation → Environment Variables Lost → Commands Fail
```

**v2.0.0 Execution Model (Fixed):**
```
Container Creation → Environment Variables Set → Commands Execute in Same Container
     ↓
Environment Variables Persist → Commands Work Correctly → Proper Isolation
```

### ✅ Production-Ready Features (v2.0.0)

- **Container Persistence**: Commands execute via `docker exec` into running containers
- **Environment Variable Persistence**: Env vars persist across all steps in the same container
- **Simplified Configuration**: Removed `type = "generic_container"` field
- **Clean Architecture**: `[containers.X]` instead of `[services.X]`
- **Deterministic Execution**: Same container instance used for all commands
- **Backward Compatibility**: Migration guide provided for v1.x users

### 🚀 Production-Ready Implementation (v2.0.0)

- **Docker Exec Semantics**: Commands run in existing containers, not new ones
- **Environment Continuity**: Variables set in container creation persist
- **Configuration Simplification**: Streamlined TOML format
- **Migration Path**: Clear upgrade path from v1.x configurations

### ❌ Known Limitations (v2.0.0)

- **Plugin System**: Core plugin architecture maintained
- **Template System**: Tera templates with macro library
- **Container Execution**: Commands run in isolated containers with lifecycle tracking
- **Multi-Service Testing**: Orchestrate multiple services with dependency management

## How to Use This Guide

### 1. Start with Migration
If you're upgrading from v1.x, start with the [Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md) to understand breaking changes.

### 2. Learn Plugin Development
Once migrated, learn how to create custom service plugins in the [Plugin Development](plugin-development/README.md) section.

### 3. Master Advanced Patterns
Explore [Advanced Testing Patterns](advanced-patterns/README.md) for complex testing scenarios.

### 4. Deploy in Production
Finally, learn [Production Deployment](production-deployment/README.md) strategies for CI/CD and enterprise use.

## Running Examples

Every example in this guide is validated and runnable:

```bash
# Validate all examples
clnrm validate examples/

# Run specific examples
clnrm run examples/ --format json
```

## Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/seanchatmangpt/clnrm/issues)
- **Migration Guide**: See [docs/V2_0_0_MIGRATION_GUIDE.md](../docs/V2_0_0_MIGRATION_GUIDE.md) for upgrade help
- **Architecture Docs**: See [docs/V2_0_0_ARCHITECTURE.md](../docs/V2_0_0_ARCHITECTURE.md) for deep technical details
- **Examples**: See [examples/](../examples/) for working code samples

## Contributing

Contributions to this guide are welcome! When contributing:

1. Follow core team standards (no unwrap/expect in production code)
2. Include runnable examples in `examples/`
3. Validate all examples work with clnrm v2.0.0
4. Update this introduction if adding new major sections

---

**Ready to get started?** Begin with the [Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md) to upgrade from v1.x.