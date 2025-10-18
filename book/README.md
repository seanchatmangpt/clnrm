# Cleanroom Testing Framework - Advanced Users Guide

This mdbook provides comprehensive guidance for advanced users of the Cleanroom Testing Framework (clnrm). It covers plugin development, advanced testing patterns, template mastery, and production deployment strategies.

## Building the Book

### Prerequisites

- [mdbook](https://rust-lang.github.io/mdBook/) installed
- Rust toolchain (for running examples)

### Build Commands

```bash
# Install mdbook (if not already installed)
cargo install mdbook

# Build the book
mdbook build book/

# Serve the book locally for development
mdbook serve book/ --open

# Watch for changes and rebuild automatically
mdbook watch book/
```

### Viewing the Book

After building, open `book/book/index.html` in your browser, or use:

```bash
mdbook serve book/ --open
```

## Running Examples

All examples in this book are validated and runnable. To test them:

```bash
# Run all mdbook examples
bash tests/mdbook-examples/validate-all-examples.sh

# Run specific example categories
cargo test --test mdbook-examples-plugin-development
cargo test --test mdbook-examples-advanced-patterns
```

## Contributing

When adding new content to this book:

1. Follow core team standards (no unwrap/expect in production examples)
2. Include runnable examples in `tests/mdbook-examples/`
3. Validate all examples work with `clnrm`
4. Update this README if adding new build steps

## Core Team Standards

This book follows FAANG-level quality standards:

- ✅ Zero unwrap()/expect() in production examples
- ✅ All traits dyn-compatible
- ✅ Proper error handling with CleanroomError
- ✅ OTEL instrumentation in examples
- ✅ AAA pattern in test examples
- ✅ Honest documentation (only v1.0.1 features)

## Links

- [Main clnrm README](../README.md)
- [Core Team Standards](../CLAUDE.md)
- [Architecture Documentation](../docs/architecture/)
- [Examples](../examples/)
