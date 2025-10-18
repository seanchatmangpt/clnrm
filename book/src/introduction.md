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
- ✅ **Honest documentation** - only features that actually work in v1.0.1
- ✅ **Runnable examples** - every code sample can be executed
- ✅ **Self-testing** - examples are validated by clnrm itself
- ✅ **Verification commands** - clear steps to validate examples

## What Actually Works (v1.0.1)

This guide documents **only** features that are verified to work in clnrm v1.0.1:

### ✅ Fully Working Features
- **Plugin System**: Custom service plugins with container integration
- **Template System**: Tera templates with macro library
- **Container Execution**: Commands run in isolated containers
- **OTEL Integration**: Span creation and basic validation
- **Multi-Service Testing**: Orchestrate multiple services
- **CI/CD Integration**: GitHub Actions and GitLab CI patterns

### 🚧 Partially Working Features
- **Advanced OTEL Validation**: Basic span validation works, advanced analysis in progress
- **Chaos Engineering**: Framework exists, advanced patterns being developed
- **Performance Testing**: Basic metrics collection, regression detection in progress

### ❌ Not Yet Implemented
- **AI Features**: Available in experimental `clnrm-ai` crate only
- **Hot Reload**: Planned for future versions
- **Advanced Reporting**: Basic JSON/XML works, advanced formats planned

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
