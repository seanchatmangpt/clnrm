//! Integration tests for clap-noun-verb framework

use clap_noun_verb::{
    app, command_group, command_tree, noun, verb, Cli, Registry, Tree, VerbArgs, Result,
    NounCommand, VerbCommand, CommandTree, CommandTreeBuilder, patterns
};

#[test]
fn test_basic_noun_verb_cli() -> Result<()> {
    let cli = app! {
        name: "test-app",
        about: "Test CLI application",
        commands: [
            noun!("services", "Manage services", [
                verb!("status", "Show status", |_args: &VerbArgs| {
                    println!("Services are running");
                    Ok(())
                }),
            ]),
        ],
    };

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "services"));

    Ok(())
}

#[test]
fn test_registry_functionality() -> Result<()> {
    let registry = Registry::new()
        .name("registry-test")
        .about("Registry test application")
        .register_noun(noun!("test", "Test commands", [
            verb!("run", "Run test", |_args: &VerbArgs| {
                println!("Running test");
                Ok(())
            }),
        ]));

    let structure = registry.command_structure();
    assert!(structure.contains_key("test"));
    assert_eq!(structure.get("test").unwrap().len(), 1);
    assert!(structure.get("test").unwrap().contains(&"run".to_string()));

    Ok(())
}

#[test]
fn test_command_tree_hierarchy() -> Result<()> {
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
                                println!("Watching for changes...");
                                Ok(())
                            })),
                        ]
                    ),
                ]
            )
    );

    let paths = tree.roots[0].command_paths();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|path| path == &vec!["dev".to_string(), "test".to_string(), "run".to_string()]));
    assert!(paths.iter().any(|path| path == &vec!["dev".to_string(), "test".to_string(), "watch".to_string()]));

    Ok(())
}

#[test]
fn test_nested_command_routing() -> Result<()> {
    let cli = Cli::new()
        .name("nested-test")
        .about("Nested command test")
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
        }));

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "dev"));

    Ok(())
}

#[test]
fn test_custom_command_implementation() -> Result<()> {
    struct CustomServicesCommand;

    impl NounCommand for CustomServicesCommand {
        fn name(&self) -> &'static str { "custom-services" }
        fn about(&self) -> &'static str { "Custom services implementation" }
        fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
            vec![Box::new(CustomStatusCommand)]
        }
    }

    struct CustomStatusCommand;

    impl VerbCommand for CustomStatusCommand {
        fn name(&self) -> &'static str { "status" }
        fn about(&self) -> &'static str { "Show custom status" }
        fn run(&self, _args: &VerbArgs) -> Result<()> {
            println!("Custom status: All systems operational");
            Ok(())
        }
    }

    let cli = Cli::new()
        .name("custom-test")
        .about("Custom command test")
        .noun(CustomServicesCommand);

    let structure = cli.command_structure();
    assert!(structure.contains_key("custom-services"));
    assert!(structure.get("custom-services").unwrap().contains(&"status".to_string()));

    Ok(())
}

#[test]
fn test_verb_args_context() -> Result<()> {
    let cli = app! {
        name: "context-test",
        about: "Context test application",
        commands: [
            noun!("test", "Test commands", [
                verb!("with-context", "Command with context", |args: &VerbArgs| {
                    let verb_name = args.verb();
                    let noun_name = args.noun();

                    assert_eq!(verb_name, "with-context");
                    assert_eq!(noun_name, Some("test"));

                    if let Some(custom) = args.get_context("custom") {
                        assert_eq!(custom, "test-value");
                    }

                    println!("Context test passed");
                    Ok(())
                }),
            ]),
        ],
    };

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "test"));

    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    let cli = app! {
        name: "error-test",
        about: "Error handling test",
        commands: [
            noun!("test", "Test commands", [
                verb!("error", "Command that errors", |_args: &VerbArgs| {
                    Err(clap_noun_verb::NounVerbError::execution_error("Test error"))
                }),
            ]),
        ],
    };

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "test"));

    Ok(())
}

#[test]
fn test_cli_builder_method_chaining() -> Result<()> {
    let cli = Cli::new()
        .name("method-chain-test")
        .about("Method chaining test")
        .noun(noun!("first", "First command group", [
            verb!("action", "First action", |_args: &VerbArgs| {
                println!("First action executed");
                Ok(())
            }),
        ]))
        .noun(noun!("second", "Second command group", [
            verb!("action", "Second action", |_args: &VerbArgs| {
                println!("Second action executed");
                Ok(())
            }),
        ]));

    let structure = cli.command_structure();
    assert!(structure.contains_key("first"));
    assert!(structure.contains_key("second"));
    assert_eq!(structure.get("first").unwrap().len(), 1);
    assert_eq!(structure.get("second").unwrap().len(), 1);

    Ok(())
}

#[test]
fn test_command_group_macro() -> Result<()> {
    let group = command_group!("test-group", "Test command group", [
        verb!("first", "First command", |_args: &VerbArgs| {
            println!("First command");
            Ok(())
        }),
        verb!("second", "Second command", |_args: &VerbArgs| {
            println!("Second command");
            Ok(())
        }),
    ]);

    // The macro should create a noun command
    assert_eq!(group.name(), "test-group");
    assert_eq!(group.about(), "Test command group");
    assert_eq!(group.verbs().len(), 2);

    Ok(())
}

#[test]
fn test_command_tree_macro() -> Result<()> {
    let tree = command_tree!(
        Cli::new()
            .name("tree-test")
            .about("Tree test")
        => noun!("root", "Root command", [
            verb!("leaf", "Leaf command", |_args: &VerbArgs| {
                println!("Leaf command");
                Ok(())
            }),
        ])
    );

    let structure = tree.command_structure();
    assert!(structure.contains_key("root"));
    assert!(structure.get("root").unwrap().contains(&"leaf".to_string()));

    Ok(())
}

#[test]
fn test_registry_introspection() -> Result<()> {
    let registry = Registry::new()
        .name("introspection-test")
        .about("Introspection test")
        .register_noun(noun!("services", "Service management", [
            verb!("status", "Show status", |_args: &VerbArgs| { Ok(()) }),
            verb!("restart", "Restart service", |_args: &VerbArgs| { Ok(()) }),
        ]))
        .register_noun(noun!("config", "Configuration management", [
            verb!("get", "Get config value", |_args: &VerbArgs| { Ok(()) }),
            verb!("set", "Set config value", |_args: &VerbArgs| { Ok(()) }),
        ]));

    // Test introspection methods
    assert_eq!(registry.noun_names().len(), 2);
    assert!(registry.has_noun("services"));
    assert!(registry.has_noun("config"));

    let structure = registry.command_structure();
    assert_eq!(structure.len(), 2);
    assert_eq!(structure.get("services").unwrap().len(), 2);
    assert_eq!(structure.get("config").unwrap().len(), 2);

    Ok(())
}

#[test]
fn test_verb_args_functionality() -> Result<()> {
    let cli = app! {
        name: "args-test",
        about: "Arguments test",
        commands: [
            noun!("test", "Test commands", [
                verb!("with-args", "Command with arguments", |args: &VerbArgs| {
                    // Test that we can access clap matches
                    let matches = &args.matches;

                    // Test context access
                    let verb_name = args.verb();
                    let noun_name = args.noun();

                    assert_eq!(verb_name, "with-args");
                    assert_eq!(noun_name, Some("test"));

                    println!("Arguments test passed");
                    Ok(())
                }),
            ]),
        ],
    };

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "test"));

    Ok(())
}
