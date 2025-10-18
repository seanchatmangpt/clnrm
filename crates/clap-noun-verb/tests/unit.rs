//! Unit tests for clap-noun-verb modules

use clap_noun_verb::{
    noun, verb, Cli, Registry, VerbArgs, Result, NounCommand, VerbCommand,
    NounContext, VerbContext, CommandTree, CommandTreeBuilder, patterns
};

#[test]
fn test_noun_command_trait() -> Result<()> {
    struct TestNoun;

    impl NounCommand for TestNoun {
        fn name(&self) -> &'static str { "test-noun" }
        fn about(&self) -> &'static str { "Test noun command" }
        fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
            vec![Box::new(TestVerb)]
        }
    }

    struct TestVerb;

    impl VerbCommand for TestVerb {
        fn name(&self) -> &'static str { "test-verb" }
        fn about(&self) -> &'static str { "Test verb command" }
        fn run(&self, _args: &VerbArgs) -> Result<()> {
            Ok(())
        }
    }

    let noun = TestNoun;
    assert_eq!(noun.name(), "test-noun");
    assert_eq!(noun.about(), "Test noun command");
    assert_eq!(noun.verbs().len(), 1);
    assert_eq!(noun.verbs()[0].name(), "test-verb");

    Ok(())
}

#[test]
fn test_verb_command_trait() -> Result<()> {
    struct TestVerb {
        name: String,
        about: String,
    }

    impl VerbCommand for TestVerb {
        fn name(&self) -> &'static str { "test-verb" }
        fn about(&self) -> &'static str { "Test verb command" }
        fn run(&self, _args: &VerbArgs) -> Result<()> {
            Ok(())
        }
    }

    let verb = TestVerb {
        name: "test".to_string(),
        about: "test".to_string(),
    };

    assert_eq!(verb.name(), "test-verb");
    assert_eq!(verb.about(), "Test verb command");

    Ok(())
}

#[test]
fn test_verb_args_context() -> Result<()> {
    let context = VerbContext::new("test-verb")
        .with_noun("test-noun")
        .with_data("key1", "value1")
        .with_data("key2", "value2");

    assert_eq!(context.verb, "test-verb");
    assert_eq!(context.noun, Some("test-noun".to_string()));
    assert_eq!(context.get_data("key1"), Some(&"value1".to_string()));
    assert_eq!(context.get_data("key2"), Some(&"value2".to_string()));
    assert_eq!(context.get_data("key3"), None);

    Ok(())
}

#[test]
fn test_verb_args_creation() -> Result<()> {
    let args = VerbArgs::new(clap::ArgMatches::default())
        .add_context("test-key", "test-value");

    assert_eq!(args.verb(), "");
    assert_eq!(args.noun(), None);
    assert_eq!(args.get_context("test-key"), Some(&"test-value".to_string()));

    Ok(())
}

#[test]
fn test_registry_configuration() -> Result<()> {
    let registry = Registry::new()
        .name("test-app")
        .about("Test application")
        .version("1.0.0");

    let command = registry.build_command();
    assert_eq!(command.get_name(), "test-app");
    assert_eq!(command.get_about().unwrap(), "Test application");
    assert_eq!(command.get_version().unwrap(), "1.0.0");

    Ok(())
}

#[test]
fn test_registry_noun_management() -> Result<()> {
    let mut registry = Registry::new();

    // Test adding nouns
    registry = registry.register_noun(noun!("test1", "Test command 1", [
        verb!("action1", "Action 1", |_args: &VerbArgs| { Ok(()) }),
    ]));

    registry = registry.register_noun(noun!("test2", "Test command 2", [
        verb!("action2", "Action 2", |_args: &VerbArgs| { Ok(()) }),
    ]));

    assert_eq!(registry.noun_names().len(), 2);
    assert!(registry.has_noun("test1"));
    assert!(registry.has_noun("test2"));

    // Test removing nouns
    let removed = registry.remove_noun("test1");
    assert!(removed.is_some());
    assert!(!registry.has_noun("test1"));
    assert!(registry.has_noun("test2"));

    Ok(())
}

#[test]
fn test_registry_command_structure() -> Result<()> {
    let registry = Registry::new()
        .register_noun(noun!("services", "Service management", [
            verb!("status", "Show status", |_args: &VerbArgs| { Ok(()) }),
            verb!("restart", "Restart service", |_args: &VerbArgs| { Ok(()) }),
        ]))
        .register_noun(noun!("config", "Configuration management", [
            verb!("get", "Get config", |_args: &VerbArgs| { Ok(()) }),
        ]));

    let structure = registry.command_structure();

    assert_eq!(structure.len(), 2);
    assert!(structure.contains_key("services"));
    assert!(structure.contains_key("config"));

    let services_verbs = structure.get("services").unwrap();
    assert_eq!(services_verbs.len(), 2);
    assert!(services_verbs.contains(&"status".to_string()));
    assert!(services_verbs.contains(&"restart".to_string()));

    let config_verbs = structure.get("config").unwrap();
    assert_eq!(config_verbs.len(), 1);
    assert!(config_verbs.contains(&"get".to_string()));

    Ok(())
}

#[test]
fn test_command_tree_basic() -> Result<()> {
    let tree = CommandTree::new();

    assert_eq!(tree.root_names().len(), 0);

    Ok(())
}

#[test]
fn test_command_tree_builder() -> Result<()> {
    let tree = CommandTree::from_builder(
        CommandTreeBuilder::new()
            .add_root_with_handler(
                "version",
                "Show version",
                |_args: &VerbArgs| {
                    println!("Version 1.0.0");
                    Ok(())
                }
            )
    );

    assert_eq!(tree.root_names().len(), 1);
    assert_eq!(tree.root_names()[0], "version");

    Ok(())
}

#[test]
fn test_command_tree_nested() -> Result<()> {
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
                                Ok(())
                            })),
                        ]
                    ),
                ]
            )
    );

    assert_eq!(tree.root_names().len(), 1);
    assert_eq!(tree.root_names()[0], "dev");

    let paths = tree.roots[0].command_paths();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], vec!["dev", "test", "run"]);

    Ok(())
}

#[test]
fn test_cli_builder_basic() -> Result<()> {
    let cli = Cli::new()
        .name("test-cli")
        .about("Test CLI");

    let command = cli.build_command();
    assert_eq!(command.get_name(), "test-cli");
    assert_eq!(command.get_about().unwrap(), "Test CLI");

    Ok(())
}

#[test]
fn test_cli_builder_with_nouns() -> Result<()> {
    let cli = Cli::new()
        .name("multi-test")
        .about("Multi-command test")
        .noun(noun!("cmd1", "Command 1", [
            verb!("action1", "Action 1", |_args: &VerbArgs| { Ok(()) }),
        ]))
        .noun(noun!("cmd2", "Command 2", [
            verb!("action2", "Action 2", |_args: &VerbArgs| { Ok(()) }),
        ]));

    let structure = cli.command_structure();
    assert_eq!(structure.len(), 2);
    assert!(structure.contains_key("cmd1"));
    assert!(structure.contains_key("cmd2"));

    Ok(())
}

#[test]
fn test_cli_builder_introspection() -> Result<()> {
    let cli = Cli::new()
        .name("introspection-test")
        .about("Introspection test")
        .noun(noun!("test", "Test command", [
            verb!("action", "Test action", |_args: &VerbArgs| { Ok(()) }),
        ]));

    assert!(cli.has_command("test"));
    assert!(!cli.has_command("nonexistent"));

    Ok(())
}

#[test]
fn test_noun_context_creation() -> Result<()> {
    let context = NounContext::new("test-noun")
        .with_data("key1", "value1")
        .with_data("key2", "value2");

    assert_eq!(context.noun, "test-noun");
    assert_eq!(context.get_data("key1"), Some(&"value1".to_string()));
    assert_eq!(context.get_data("key2"), Some(&"value2".to_string()));
    assert_eq!(context.get_data("key3"), None);

    Ok(())
}

#[test]
fn test_verb_context_creation() -> Result<()> {
    let context = VerbContext::new("test-verb")
        .with_noun("test-noun")
        .with_data("key1", "value1");

    assert_eq!(context.verb, "test-verb");
    assert_eq!(context.noun, Some("test-noun".to_string()));
    assert_eq!(context.get_data("key1"), Some(&"value1".to_string()));

    Ok(())
}

#[test]
fn test_macro_expansion() -> Result<()> {
    // Test that macros expand correctly
    let test_noun = noun!("test-noun", "Test noun", [
        verb!("test-verb", "Test verb", |_args: &VerbArgs| { Ok(()) }),
    ]);

    assert_eq!(test_noun.name(), "test-noun");
    assert_eq!(test_noun.about(), "Test noun");
    assert_eq!(test_noun.verbs().len(), 1);

    Ok(())
}

#[test]
fn test_error_types() -> Result<()> {
    // Test error creation
    let cmd_error = clap_noun_verb::NounVerbError::command_not_found("missing-command");
    let verb_error = clap_noun_verb::NounVerbError::verb_not_found("services", "missing-verb");
    let structure_error = clap_noun_verb::NounVerbError::invalid_structure("Invalid structure");
    let exec_error = clap_noun_verb::NounVerbError::execution_error("Execution failed");
    let arg_error = clap_noun_verb::NounVerbError::argument_error("Invalid arguments");

    // Test that errors have the expected messages
    assert!(cmd_error.to_string().contains("Command 'missing-command' not found"));
    assert!(verb_error.to_string().contains("Verb 'missing-verb' not found for noun 'services'"));
    assert!(structure_error.to_string().contains("Invalid structure"));
    assert!(exec_error.to_string().contains("Execution failed"));
    assert!(arg_error.to_string().contains("Invalid arguments"));

    Ok(())
}

#[test]
fn test_patterns_helper() -> Result<()> {
    let pattern = patterns::noun_verb_pattern(
        "test-noun",
        "Test noun pattern",
        vec![
            ("verb1".to_string(), "Verb 1".to_string(), Box::new(|_args: &VerbArgs| {
                Ok(())
            })),
            ("verb2".to_string(), "Verb 2".to_string(), Box::new(|_args: &VerbArgs| {
                Ok(())
            })),
        ]
    );

    assert_eq!(pattern.name(), "test-noun");
    assert_eq!(pattern.about(), "Test noun pattern");
    assert_eq!(pattern.verbs().len(), 2);

    Ok(())
}

#[test]
fn test_build_cli_function() -> Result<()> {
    let (command, structure) = clap_noun_verb::build_cli(|cli| {
        cli.name("build-test")
           .about("Build test CLI")
           .noun(noun!("test", "Test command", [
               verb!("action", "Test action", |_args: &VerbArgs| { Ok(()) }),
           ]))
    });

    assert_eq!(command.get_name(), "build-test");
    assert_eq!(command.get_about().unwrap(), "Build test CLI");
    assert_eq!(structure.len(), 1);
    assert!(structure.contains_key("test"));

    Ok(())
}

#[test]
fn test_run_cli_function() -> Result<()> {
    // This test would normally run the CLI, but for testing we just verify
    // that the function exists and can be called with a simple builder
    let result = clap_noun_verb::run_cli(|cli| {
        cli.name("run-test")
           .about("Run test CLI")
           .noun(noun!("test", "Test command", [
               verb!("help", "Show help", |_args: &VerbArgs| {
                   println!("Help command");
                   Ok(())
               }),
           ]))
    });

    // The function should return an error when run without proper args
    // but should not panic
    assert!(result.is_err());

    Ok(())
}
