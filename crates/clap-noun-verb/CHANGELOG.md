# Changelog

All notable changes to clap-noun-verb will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-12-19

### Added - v1.0.0 Release

- **API Stability**: All public APIs are now stable and guaranteed within the same major version
- **Enhanced Documentation**: Comprehensive API documentation with examples for all public types and traits
- **Edge Case Testing**: Expanded test coverage for error paths, validation, and edge cases
- **Publishing Metadata**: Complete Cargo.toml metadata for crates.io publication

### Changed

- **Version**: Bumped to 1.0.0 to signal API stability
- **Documentation**: Added API stability guarantees and examples to all public APIs
- **Metadata**: Added documentation and homepage links to Cargo.toml

### Technical Details

- All public APIs marked as stable
- Comprehensive test coverage (32+ tests including edge cases)
- All examples compile and run
- Ready for production use

## [0.1.0] - 2024-12-19

### Added
- Initial release of clap-noun-verb crate
- Trait-based command definition with `NounCommand` and `VerbCommand`
- Builder pattern API with `CliBuilder`
- Convenience macros `noun!` and `verb!`
- Comprehensive error handling with `NounVerbError`
- Command routing with `CommandRouter` and `CommandRegistry`
- Command tree support for hierarchical composition
- Examples demonstrating real-world usage
- Full documentation and README

### Features
- **Noun-Verb Pattern**: Clean abstraction for `noun verb` CLI structure
- **Type Safety**: Compile-time verification of command structure
- **Zero-Cost Abstractions**: Thin wrapper over clap with no runtime overhead
- **Ergonomic API**: Method chaining and macro-based command registration
- **Automatic Help**: Enhanced help text generation for noun-verb patterns
- **Error Handling**: Comprehensive error types with context and source information
- **Argument Extraction Helpers**: Type-safe argument extraction from `VerbArgs`
- **Verb Argument Support**: Define arguments directly in verb macros
- **Command Structure Validation**: Validate command structure for duplicate names and conflicts
- **Global Arguments Access**: Verbs can access global arguments like `--verbose` and `--config` from parent commands
- **PathBuf Convenience Methods**: Specialized helpers for path arguments
- **Auto-Validation Option**: Optional automatic structure validation on build/run

### API Highlights

#### Argument Extraction Helpers
- `get_one_str(name)` - Get required string argument
- `get_one_str_opt(name)` - Get optional string argument
- `get_one<T>(name)` - Get required typed argument
- `get_one_opt<T>(name)` - Get optional typed argument
- `get_many<T>(name)` - Get required multiple values
- `get_many_opt<T>(name)` - Get optional multiple values
- `is_flag_set(name)` - Check if flag is set
- `get_flag_count(name)` - Get flag count (for -v, -vv, -vvv patterns)
- `get_path(name)` - Get required PathBuf argument (convenience method)
- `get_path_opt(name)` - Get optional PathBuf argument (convenience method)

#### Global Arguments Access
- `get_global<T>(name)` - Get global argument of any type
- `get_global_str(name)` - Get global string argument
- `is_global_flag_set(name)` - Check if global flag is set
- `get_global_flag_count(name)` - Get global flag count (for -v, -vv, -vvv patterns)

#### Verb Argument Support
Verbs can now define arguments directly in the macro:
```rust
verb!("logs", "Show logs", |args: &VerbArgs| {
    let service = args.get_one_str("service")?;
    let verbose = args.get_global_flag_count("verbose");
    Ok(())
}, args: [
    clap::Arg::new("service").required(true),
    clap::Arg::new("lines").short('n').long("lines").default_value("50"),
]);
```

#### Command Structure Validation
The `CommandRegistry` now has a `validate()` method that checks for:
- Duplicate noun names
- Empty nouns (no verbs or sub-nouns)
- Duplicate verb names within a noun
- Duplicate sub-noun names within a noun
- Verb/sub-noun name conflicts

### Examples
- **Basic Example**: Simple noun-verb CLI with services and collector commands
- **Services Example**: Detailed services management CLI
- **Collector Example**: OpenTelemetry collector management CLI
- **Framework Example**: Comprehensive framework usage demonstration

### Documentation
- Comprehensive README with usage examples
- API documentation with trait definitions
- Migration guide from direct clap usage
- Contributing guidelines
- Design philosophy and rationale

### Technical Details
- Built on clap 4.5
- Uses thiserror for error handling
- Static string lifetimes for trait compatibility
- Dyn-compatible trait design
- Comprehensive test coverage (32 tests)
- All examples compile and run




