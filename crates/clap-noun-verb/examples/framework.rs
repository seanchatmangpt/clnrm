//! Framework-level composition example
//!
//! This example demonstrates how to use clap-noun-verb as a framework
//! for building composable CLI patterns, similar to how users would
//! build their own CLI frameworks on top of it.

use clap_noun_verb::{
    app, command_group, command_tree, noun, verb, Cli, Registry, Tree, VerbArgs, Result,
    NounCommand, VerbCommand, CommandTree, CommandTreeBuilder, patterns
};

fn main() -> Result<()> {
    // Method 1: Using the declarative app! macro
    println!("=== Method 1: Declarative app! macro ===");
    let cli = app! {
        name: "myapp",
        about: "My awesome CLI application built with clap-noun-verb framework",
        commands: [
            noun!("services", "Manage application services", [
                verb!("status", "Show status of all services", |_args: &VerbArgs| {
                    println!("📊 Service Status:");
                    println!("  web-server: Running (port 8080)");
                    println!("  database: Running (port 5432)");
                    println!("  redis: Running (port 6379)");
                    Ok(())
                }),
                verb!("logs", "Show logs for a service", |args: &VerbArgs| {
                    if let Some(service) = args.matches.get_one::<String>("service") {
                        println!("📄 Logs for service: {}", service);
                    } else {
                        println!("📄 Recent service logs:");
                        println!("  [2024-01-01 10:00:00] INFO: Service started");
                        println!("  [2024-01-01 10:00:01] INFO: Listening on port 8080");
                    }
                    Ok(())
                }),
                verb!("restart", "Restart a service", |args: &VerbArgs| {
                    if let Some(service) = args.matches.get_one::<String>("service") {
                        println!("🔄 Restarting service: {}", service);
                    } else {
                        println!("🔄 Restarting all services...");
                    }
                    Ok(())
                }),
            ]),
            noun!("collector", "Manage OpenTelemetry collector", [
                verb!("up", "Start the collector", |_args: &VerbArgs| {
                    println!("Starting OpenTelemetry Collector...");
                    println!("✓ Collector started on ports:");
                    println!("  HTTP: 4318");
                    println!("  gRPC: 4317");
                    Ok(())
                }),
                verb!("down", "Stop the collector", |_args: &VerbArgs| {
                    println!("Stopping OpenTelemetry Collector...");
                    println!("✓ Collector stopped");
                    Ok(())
                }),
                verb!("status", "Show collector status", |_args: &VerbArgs| {
                    println!("Collector Status:");
                    println!("  State: Running");
                    println!("  HTTP endpoint: http://localhost:4318");
                    println!("  gRPC endpoint: http://localhost:4317");
                    Ok(())
                }),
            ]),
        ],
    };

    println!("Built CLI structure: {:?}", cli.command_structure());

    // Method 2: Using the registry for programmatic composition
    println!("\n=== Method 2: Programmatic composition with Registry ===");
    let registry = Registry::new()
        .name("composed-app")
        .about("CLI application built with programmatic composition")
        .register_noun(services_command())
        .register_noun(collector_command());

    println!("Registry structure: {:?}", registry.command_structure());

    // Method 3: Using the command tree for hierarchical composition
    println!("\n=== Method 3: Hierarchical composition with CommandTree ===");
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
                            ("watch".to_string(), "Watch for changes".to_string(), Box::new(|_args: &VerbArgs| {
                                println!("Watching for test changes...");
                                Ok(())
                            })),
                        ]
                    ),
                    patterns::noun_verb_pattern(
                        "lint",
                        "Code linting",
                        vec![
                            ("check".to_string(), "Check code style".to_string(), Box::new(|_args: &VerbArgs| {
                                println!("Checking code style...");
                                Ok(())
                            })),
                            ("fix".to_string(), "Auto-fix issues".to_string(), Box::new(|_args: &VerbArgs| {
                                println!("Auto-fixing linting issues...");
                                Ok(())
                            })),
                        ]
                    ),
                ]
            )
            .add_root_with_handler(
                "version",
                "Show version information",
                |_args: &VerbArgs| {
                    println!("composed-app v1.0.0");
                    Ok(())
                }
            )
    );

    println!("Tree structure:");
    for path in tree.roots[0].command_paths() {
        println!("  {}", path.join(" "));
    }

    // Method 4: Custom command implementation for advanced use cases
    println!("\n=== Method 4: Custom command implementation ===");
    let custom_cli = Cli::new()
        .name("custom-app")
        .about("CLI with custom command implementations")
        .noun(custom_services_command())
        .noun(custom_collector_command());

    println!("Custom CLI structure: {:?}", custom_cli.command_structure());

    Ok(())
}

/// Custom services command implementation
fn services_command() -> impl NounCommand {
    noun!("services", "Manage application services", [
        verb!("status", "Show status of all services", |_args: &VerbArgs| {
            println!("📊 Service Status:");
            println!("  web-server: Running (port 8080)");
            println!("  database: Running (port 5432)");
            println!("  redis: Running (port 6379)");
            Ok(())
        }),
        verb!("logs", "Show logs for a service", |args: &VerbArgs| {
            println!("📄 Service Logs:");
            if let Some(service) = args.matches.get_one::<String>("service") {
                println!("  Logs for service: {}", service);
            } else {
                println!("  [2024-01-01 10:00:00] INFO: Service started");
                println!("  [2024-01-01 10:00:01] INFO: Listening on port 8080");
            }
            Ok(())
        }),
        verb!("restart", "Restart a service", |args: &VerbArgs| {
            if let Some(service) = args.matches.get_one::<String>("service") {
                println!("🔄 Restarting service: {}", service);
            } else {
                println!("🔄 Restarting all services...");
            }
            Ok(())
        }),
    ])
}

/// Custom collector command implementation
fn collector_command() -> impl NounCommand {
    noun!("collector", "Manage OpenTelemetry collector", [
        verb!("up", "Start the collector", |_args: &VerbArgs| {
            println!("Starting OpenTelemetry Collector...");
            println!("✓ Collector started on ports:");
            println!("  HTTP: 4318");
            println!("  gRPC: 4317");
            Ok(())
        }),
        verb!("down", "Stop the collector", |_args: &VerbArgs| {
            println!("Stopping OpenTelemetry Collector...");
            println!("✓ Collector stopped");
            Ok(())
        }),
        verb!("status", "Show collector status", |_args: &VerbArgs| {
            println!("Collector Status:");
            println!("  State: Running");
            println!("  HTTP endpoint: http://localhost:4318");
            println!("  gRPC endpoint: http://localhost:4317");
            Ok(())
        }),
    ])
}

/// Advanced custom services command with nested structure
fn custom_services_command() -> impl NounCommand {
    noun!("services", "Manage application services", {
        noun!("database", "Database service management", [
            verb!("status", "Check database status", |_args: &VerbArgs| {
                println!("🗄️ Database Status: Running (PostgreSQL 15)");
                Ok(())
            }),
            verb!("backup", "Create database backup", |_args: &VerbArgs| {
                println!("💾 Creating database backup...");
                Ok(())
            }),
        ]),
        noun!("web", "Web service management", [
            verb!("status", "Check web server status", |_args: &VerbArgs| {
                println!("🌐 Web Server Status: Running (Nginx)");
                Ok(())
            }),
            verb!("reload", "Reload web server configuration", |_args: &VerbArgs| {
                println!("🔄 Reloading web server configuration...");
                Ok(())
            }),
        ]),
    })
}

/// Advanced custom collector command
fn custom_collector_command() -> impl NounCommand {
    noun!("collector", "Manage OpenTelemetry collector", [
        verb!("up", "Start the collector", |_args: &VerbArgs| {
            println!("🚀 Starting OpenTelemetry Collector...");
            println!("✓ Collector started successfully");
            Ok(())
        }),
        verb!("down", "Stop the collector", |_args: &VerbArgs| {
            println!("⏹️ Stopping OpenTelemetry Collector...");
            println!("✓ Collector stopped");
            Ok(())
        }),
        verb!("config", "Manage collector configuration", |args: &VerbArgs| {
            if args.matches.get_flag("validate") {
                println!("✅ Collector configuration is valid");
            } else {
                println!("📋 Collector configuration:");
                println!("  exporters:");
                println!("    - stdout");
                println!("    - otlp-http");
            }
            Ok(())
        }),
    ])
}
