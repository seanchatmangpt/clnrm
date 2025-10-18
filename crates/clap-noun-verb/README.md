# clap-noun-verb

A framework for building composable CLI patterns on top of clap. This crate provides the foundation for creating command-line interfaces with the `noun verb` pattern (e.g., `services status`, `collector up`), similar to how Python's Typer provides a simpler interface over Click.

## Framework Philosophy

**clap-noun-verb** is designed as a **framework** rather than a library of specific compositions. Instead of providing pre-built CLI patterns, it provides the tools and APIs that allow you to compose your own CLI patterns in flexible, extensible ways.

### Key Framework Features

- **Composable Command Structure**: Easy composition of nouns and verbs
- **Framework-Level APIs**: APIs that make it easy to build CLI frameworks
- **Extensible Traits**: Traits that can be easily extended and customized
- **Hierarchical Command Support**: Support for complex nested command structures
- **Type-Safe Composition**: Compile-time verification of command structure
- **Multiple Composition Methods**: Choose the composition style that fits your needs

## Features

- **Trait-based command definition** - `NounCommand` and `VerbCommand` traits for type-safe command structure
- **Builder pattern API** - Ergonomic command registration with method chaining
- **Automatic help generation** - Enhanced help text for noun-verb patterns
- **Type-safe command routing** - Compile-time verification of command structure
- **Zero-cost abstractions** - Thin wrapper over clap with no runtime overhead
- **Convenience macros** - Reduce boilerplate with `noun!` and `verb!` macros

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "0.1.0"
```

## Framework Usage

**clap-noun-verb** provides multiple ways to compose CLI patterns:

### Method 1: Declarative Composition (Simplest)

```rust
use clap_noun_verb::{app, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    let cli = app! {
        name: "myapp",
        about: "My awesome CLI application",
        commands: [
            noun!("services", "Manage services", [
                verb!("status", "Show status", |_args: &VerbArgs| {
                    println!("All services are running");
                    Ok(())
                }),
            ]),
            noun!("collector", "Manage collector", [
                verb!("up", "Start collector", |_args: &VerbArgs| {
                    println!("Starting collector");
                    Ok(())
                }),
            ]),
        ],
    };

    cli.run()
}
```

### Method 2: Builder Pattern (Most Flexible)

```rust
use clap_noun_verb::{Cli, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    let cli = Cli::new()
        .name("myapp")
        .about("My CLI application")
        .noun(noun!("services", "Manage services", [
            verb!("status", "Show status", |_args: &VerbArgs| {
                println!("Services are running");
                Ok(())
            }),
        ]))
        .noun(noun!("collector", "Manage collector", [
            verb!("up", "Start collector", |_args: &VerbArgs| {
                println!("Starting collector");
                Ok(())
            }),
        ]));

    cli.run()
}
```

### Method 3: Command Registry (For Dynamic Composition)

```rust
use clap_noun_verb::{Registry, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    let registry = Registry::new()
        .name("dynamic-app")
        .about("Dynamically composed CLI")
        .register_noun(noun!("services", "Manage services", [
            verb!("status", "Show status", |_args: &VerbArgs| {
                println!("Dynamic services status");
                Ok(())
            }),
        ]));

    registry.run()
}
```

### Method 4: Command Tree (For Hierarchical Composition)

```rust
use clap_noun_verb::{CommandTree, CommandTreeBuilder, patterns, VerbArgs, Result};

fn main() -> Result<()> {
    let tree = CommandTree::from_builder(
        CommandTreeBuilder::new()
            .add_root_with_children(
                "dev",
                "Development tools",
                vec![
                    patterns::noun_verb_pattern(
                        "test",
                        "Testing utilities",
                        vec![
                            ("run".to_string(), "Run tests".to_string(), Box::new(|_args: &VerbArgs| {
                                println!("Running tests...");
                                Ok(())
                            })),
                        ]
                    ),
                ]
            )
    );

    tree.run()
}
```

All methods create the same CLI structure:

```
myapp
├── services
│   └── status
└── collector
    └── up
```

## Advanced Usage

### Nested Noun-Verb Commands

For more complex CLI hierarchies, you can create nested command structures:

```rust
use clap_noun_verb::{run_cli, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    run_cli("myapp", |cli| {
        cli.about("Advanced CLI with nested commands")
            .noun(noun!("dev", "Development tools", {
                noun!("test", "Testing utilities", [
                    verb!("run", "Run tests", |_args: &VerbArgs| {
                        println!("Running tests...");
                        Ok(())
                    }),
                    verb!("watch", "Watch for changes", |_args: &VerbArgs| {
                        println!("Watching for changes...");
                        Ok(())
                    }),
                ]),
                noun!("lint", "Code linting", [
                    verb!("check", "Check code style", |_args: &VerbArgs| {
                        println!("Checking code style...");
                        Ok(())
                    }),
                    verb!("fix", "Auto-fix issues", |_args: &VerbArgs| {
                        println!("Auto-fixing issues...");
                        Ok(())
                    }),
                ]),
            }))
            .noun(noun!("ai", "AI-powered tools", {
                noun!("orchestrate", "Test orchestration", [
                    verb!("run", "Run orchestrated tests", |_args: &VerbArgs| {
                        println!("Running orchestrated tests...");
                        Ok(())
                    }),
                    verb!("predict", "Predict failures", |_args: &VerbArgs| {
                        println!("Predicting failures...");
                        Ok(())
                    }),
                ]),
                noun!("analyze", "Code analysis", [
                    verb!("performance", "Analyze performance", |_args: &VerbArgs| {
                        println!("Analyzing performance...");
                        Ok(())
                    }),
                    verb!("quality", "Analyze code quality", |_args: &VerbArgs| {
                        println!("Analyzing code quality...");
                        Ok(())
                    }),
                ]),
            }))
    })
}
```

This creates commands like:
- `myapp dev test run`
- `myapp dev lint check`
- `myapp ai orchestrate predict`
- `myapp ai analyze quality`

### Using the Builder Pattern Directly

```rust
use clap_noun_verb::{NounVerbCli, noun, verb, VerbArgs};

fn main() -> clap_noun_verb::Result<()> {
    let cli = NounVerbCli::new("myapp")
        .about("My awesome CLI application")
        .noun(noun!("services", "Manage services", [
            verb!("status", "Show status", |_args: &VerbArgs| {
                println!("Services are running");
                Ok(())
            }),
        ]));

    cli.run()
}
```

### Custom Command Implementation

For more control, you can implement the traits directly:

```rust
use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs, Result};

struct ServicesCommand;

impl NounCommand for ServicesCommand {
    fn name(&self) -> &'static str { "services" }
    fn about(&self) -> &'static str { "Manage application services" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(StatusCommand)]
    }
}

struct StatusCommand;

impl VerbCommand for StatusCommand {
    fn name(&self) -> &'static str { "status" }
    fn about(&self) -> &'static str { "Show service status" }
    fn run(&self, _args: &VerbArgs) -> Result<()> {
        println!("All services are running");
        Ok(())
    }
}
```

### Accessing Command Arguments

The `VerbArgs` struct provides access to parsed arguments:

```rust
use clap_noun_verb::{verb, VerbArgs};

verb!("logs", "Show logs for a service", |args: &VerbArgs| {
    // Access context data
    if let Some(service) = args.get_context("service") {
        println!("Showing logs for service: {}", service);
    }
    
    // Access clap matches for custom arguments
    if args.matches.get_flag("follow") {
        println!("Following logs...");
    }
    
    Ok(())
})
```

## Framework Examples

The crate includes examples demonstrating different composition approaches:

- **Framework Example** (`examples/framework.rs`) - Demonstrates all composition methods (declarative, builder, registry, tree)
- **Basic Example** (`examples/basic.rs`) - Simple noun-verb CLI with services and collector commands
- **Services Example** (`examples/services.rs`) - More detailed services management CLI
- **Collector Example** (`examples/collector.rs`) - OpenTelemetry collector management CLI
- **Nested Example** (`examples/nested.rs`) - Complex nested command hierarchies

Run the examples:

```bash
# Framework composition examples
cargo run --example framework

# Basic usage examples
cargo run --example basic -- --help
cargo run --example services -- services status
cargo run --example collector -- collector up

# Advanced nested patterns
cargo run --example nested -- dev test run
cargo run --example nested -- ai orchestrate predict
```

## API Reference

### Framework Types

- **`Cli`** - Main builder for creating composable CLI applications
- **`Registry`** - Central registry for dynamic command composition
- **`Tree`** - Tree-based structure for hierarchical command organization
- **`NounCommand`** - Trait for defining noun commands (composable units)
- **`VerbCommand`** - Trait for defining verb commands (actions on nouns)
- **`NounContext`** - Context information passed to noun commands
- **`VerbContext`** - Context information passed to verb commands
- **`VerbArgs`** - Arguments and context passed to verb commands

### Composition Methods

1. **Declarative** - `app!` macro for simple composition
2. **Builder** - `Cli` for flexible composition
3. **Registry** - `Registry` for dynamic composition
4. **Tree** - `CommandTree` for hierarchical composition

### Macros

- **`app!(name, about, commands)`** - Declarative CLI composition
- **`noun!(name, about, [verbs...])`** - Create a noun command with verbs
- **`verb!(name, about, handler)`** - Create a verb command with handler
- **`command_group!(name, about, [verbs...])`** - Create a command group
- **`command_tree!(builder => commands...)`** - Compose commands into a tree

### Error Types

The crate uses `thiserror` for comprehensive error handling:

- **`NounVerbError::CommandNotFound`** - When a noun command is not found
- **`NounVerbError::VerbNotFound`** - When a verb command is not found for a noun
- **`NounVerbError::InvalidStructure`** - When the command structure is invalid
- **`NounVerbError::ExecutionError`** - When command execution fails
- **`NounVerbError::ArgumentError`** - When argument parsing fails

## Design Philosophy

### Why Framework-Based?

**clap-noun-verb** is designed as a **framework** that enables composition rather than providing specific compositions:

1. **Composable by Design** - Users compose their own CLI patterns
2. **Multiple Composition Methods** - Choose the approach that fits your needs
3. **Extensible Architecture** - Easy to extend and customize for specific use cases
4. **Framework-Level APIs** - APIs that make it easy to build CLI frameworks

### Why Noun-Verb Pattern?

The noun-verb pattern provides several benefits:

1. **Intuitive Structure** - Commands naturally group related functionality
2. **Scalable Organization** - Easy to add new verbs to existing nouns
3. **Consistent UX** - Users learn one pattern and can apply it everywhere
4. **Type Safety** - Compile-time verification of command structure

### Why Trait-Based Architecture?

- **Extensibility** - Easy to add new command types without modifying core
- **Type Safety** - Compile-time verification of command implementations
- **Testability** - Commands can be easily mocked and tested
- **Composability** - Commands can be combined and reused

### Why Multiple Composition Methods?

Different projects have different needs:

- **Declarative** - For simple, static CLI structures
- **Builder** - For flexible, programmatic composition
- **Registry** - For dynamic, runtime composition
- **Tree** - For complex, hierarchical command structures

## Framework vs Direct clap Usage

### Direct clap (verbose enum-based):

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Services {
        #[command(subcommand)]
        command: ServiceCommands,
    },
}

#[derive(Subcommand)]
enum ServiceCommands {
    Status,
    Logs { service: String },
    Restart { service: String },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Services { command } => match command {
            ServiceCommands::Status => println!("Services running"),
            ServiceCommands::Logs { service } => println!("Logs for {}", service),
            ServiceCommands::Restart { service } => println!("Restarting {}", service),
        },
    }
}
```

### With clap-noun-verb Framework (composable):

**Option 1: Declarative**
```rust
use clap_noun_verb::{app, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    let cli = app! {
        name: "myapp",
        about: "My CLI application",
        commands: [
            noun!("services", "Manage services", [
                verb!("status", "Show status", |_args: &VerbArgs| {
                    println!("Services running");
                    Ok(())
                }),
                verb!("logs", "Show logs", |args: &VerbArgs| {
                    println!("Showing logs");
                    Ok(())
                }),
            ]),
        ],
    };
    cli.run()
}
```

**Option 2: Builder Pattern**
```rust
use clap_noun_verb::{Cli, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    let cli = Cli::new()
        .name("myapp")
        .about("My CLI application")
        .noun(noun!("services", "Manage services", [
            verb!("status", "Show status", |_args: &VerbArgs| {
                println!("Services running");
                Ok(())
            }),
        ]));

    cli.run()
}
```

**Option 3: Registry (Dynamic)**
```rust
use clap_noun_verb::{Registry, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    let registry = Registry::new()
        .name("myapp")
        .about("Dynamically composed CLI")
        .register_noun(noun!("services", "Manage services", [
            verb!("status", "Show status", |_args: &VerbArgs| {
                println!("Dynamic services");
                Ok(())
            }),
        ]));

    registry.run()
}
```

The framework approach provides:
- **Multiple composition styles** - Choose what fits your project
- **Better organization** - Commands grouped by functionality
- **Easier maintenance** - Less boilerplate, clearer structure
- **Framework extensibility** - Easy to extend and customize

## CLI Patterns for Porting

Based on analysis of the clnrm codebase, here are CLI patterns that could benefit from the noun-verb approach:

### Current clnrm Patterns

**Services Management:**
```rust
// Current: services status|logs|restart|ai-manage
// Could be: services status, services logs, services restart, services ai-manage
```

**Collector Management:**
```rust
// Current: collector up|down|status|logs
// Could be: collector up, collector down, collector status, collector logs
```

**Report Generation:**
```rust
// Current: report --input --output --format
// Could be: report generate, report view, report export
```

**Template Operations:**
```rust
// Current: template <template_name> --output
// Could be: template generate, template list, template validate
```

**AI Commands:**
```rust
// Current: ai-orchestrate, ai-predict, ai-optimize, ai-real, ai-monitor
// Could be: ai orchestrate, ai predict, ai optimize, ai real, ai monitor
```

**Trace Operations:**
```rust
// Current: analyze, diff, graph, spans, repro
// Could be: trace analyze, trace diff, trace graph, trace spans, trace repro
```

**Development Tools:**
```rust
// Current: dev, dry-run, fmt, lint
// Could be: dev test, dev lint, dev format, dev run
```

### Typer-Inspired Patterns

The crate supports patterns similar to Python's Typer:

```rust
// Single-level nouns
noun!("services", "Manage services", [
    verb!("status", "Show status", handler),
    verb!("restart", "Restart service", handler),
])

// Nested nouns (compound commands)
noun!("dev", "Development tools", {
    noun!("test", "Testing utilities", [
        verb!("run", "Run tests", handler),
        verb!("watch", "Watch changes", handler),
    ]),
    noun!("lint", "Code linting", [
        verb!("check", "Check code", handler),
        verb!("fix", "Auto-fix issues", handler),
    ]),
})
```

This creates intuitive command structures like:
- `myapp services status`
- `myapp dev test run`
- `myapp dev lint check`

## Migration Guide

### From Direct clap

1. **Replace enum-based commands** with noun-verb structure
2. **Convert command handlers** to functions that take `VerbArgs`
3. **Use macros** to reduce boilerplate
4. **Update error handling** to use `NounVerbError`

### From Other CLI Frameworks

- **From structopt**: Similar builder pattern, but with noun-verb organization
- **From argh**: More structured approach with trait-based commands
- **From clap-derive**: Simpler syntax with automatic help generation

## Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Python's [Typer](https://typer.tiangolo.com/) library
- Built on top of the excellent [clap](https://crates.io/crates/clap) crate
- Error handling powered by [thiserror](https://crates.io/crates/thiserror)
