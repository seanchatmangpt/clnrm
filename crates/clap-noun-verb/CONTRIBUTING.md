# Contributing to clap-noun-verb

Thank you for your interest in contributing to clap-noun-verb! This document provides guidelines and information for contributors.

## Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/seanchatmangpt/clnrm.git
   cd clnrm/crates/clap-noun-verb
   ```

2. **Install dependencies**:
   ```bash
   cargo build
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Run examples**:
   ```bash
   cargo run --example basic -- --help
   cargo run --example services -- services status
   cargo run --example collector -- collector up
   ```

## Code Style

### Rust Conventions

- Follow standard Rust formatting with `cargo fmt`
- Use `cargo clippy` for linting
- Write comprehensive documentation for public APIs
- Use meaningful variable and function names

### Error Handling

- Use `thiserror` for error types
- Provide context and source information in errors
- Never use `unwrap()` or `expect()` in production code
- Always use proper `Result<T, E>` types

### Testing

- Write unit tests for all public functions
- Include integration tests for command routing
- Test error cases and edge conditions
- Use descriptive test names that explain what is being tested

## Architecture Guidelines

### Trait Design

- Keep traits `dyn` compatible (no async trait methods)
- Use static string lifetimes for trait methods
- Provide sensible defaults where possible
- Document trait contracts clearly

### Macro Design

- Keep macros simple and focused
- Provide clear error messages for macro misuse
- Use proper hygiene to avoid name conflicts
- Document macro syntax and behavior

### Builder Pattern

- Use method chaining for ergonomic APIs
- Provide sensible defaults
- Allow configuration at any point in the chain
- Make the API intuitive and discoverable

## Pull Request Process

1. **Fork the repository** and create a feature branch
2. **Write tests** for any new functionality
3. **Update documentation** as needed
4. **Run the full test suite** to ensure nothing is broken
5. **Submit a pull request** with a clear description

### Pull Request Guidelines

- **Title**: Use a clear, descriptive title
- **Description**: Explain what changes were made and why
- **Tests**: Include test cases for new functionality
- **Documentation**: Update relevant documentation
- **Breaking Changes**: Clearly mark any breaking changes

## Issue Reporting

When reporting issues, please include:

- **Rust version**: `rustc --version`
- **Crate version**: Version of clap-noun-verb being used
- **OS**: Operating system and version
- **Reproduction steps**: Clear steps to reproduce the issue
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Code example**: Minimal code that reproduces the issue

## Feature Requests

When requesting features, please include:

- **Use case**: Why is this feature needed?
- **Proposed API**: How should the feature be used?
- **Alternatives**: What alternatives have you considered?
- **Implementation ideas**: Any thoughts on how to implement it?

## Release Process

Releases are managed through semantic versioning:

- **Major version** (1.0.0): Breaking changes
- **Minor version** (0.1.0): New features, backward compatible
- **Patch version** (0.0.1): Bug fixes, backward compatible

### Release Checklist

- [ ] Update version numbers in Cargo.toml
- [ ] Update CHANGELOG.md with new features and fixes
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create release tag
- [ ] Publish to crates.io

## Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the Rust community guidelines

### Communication

- Use clear, professional language
- Provide context for questions and suggestions
- Be patient with newcomers
- Share knowledge and best practices

## Development Tools

### Recommended Tools

- **rust-analyzer**: IDE support for Rust
- **cargo-watch**: Automatic recompilation during development
- **cargo-expand**: Expand macros for debugging
- **cargo-audit**: Security vulnerability scanning

### Useful Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run examples
cargo run --example basic

# Check documentation
cargo doc --open

# Audit dependencies
cargo audit
```

## License

By contributing to clap-noun-verb, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions about contributing, please:

- Open an issue for discussion
- Check existing issues and pull requests
- Review the documentation and examples
- Ask in the project discussions

Thank you for contributing to clap-noun-verb!
