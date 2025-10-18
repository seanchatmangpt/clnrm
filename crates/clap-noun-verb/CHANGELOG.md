# Changelog

All notable changes to clap-noun-verb will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of clap-noun-verb crate
- Trait-based command definition with `NounCommand` and `VerbCommand`
- Builder pattern API with `NounVerbCli`
- Convenience macros `noun!` and `verb!`
- Comprehensive error handling with `NounVerbError`
- Command routing with `CommandRouter`
- Examples demonstrating real-world usage
- Full documentation and README

### Features
- **Noun-Verb Pattern**: Clean abstraction for `noun verb` CLI structure
- **Type Safety**: Compile-time verification of command structure
- **Zero-Cost Abstractions**: Thin wrapper over clap with no runtime overhead
- **Ergonomic API**: Method chaining and macro-based command registration
- **Automatic Help**: Enhanced help text generation for noun-verb patterns
- **Error Handling**: Comprehensive error types with context and source information

### Examples
- **Basic Example**: Simple noun-verb CLI with services and collector commands
- **Services Example**: Detailed services management CLI
- **Collector Example**: OpenTelemetry collector management CLI

### Documentation
- Comprehensive README with usage examples
- API documentation with trait definitions
- Migration guide from direct clap usage
- Contributing guidelines
- Design philosophy and rationale

## [0.1.0] - 2024-01-01

### Added
- Initial release
- Core noun-verb CLI framework
- Trait-based command system
- Builder pattern API
- Macro convenience functions
- Error handling system
- Command routing
- Examples and documentation

### Technical Details
- Built on clap 4.5
- Uses thiserror for error handling
- Static string lifetimes for trait compatibility
- Dyn-compatible trait design
- Comprehensive test coverage
