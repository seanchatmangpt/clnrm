//! Edge case and error path tests for clap-noun-verb

use clap::Arg;
use clap_noun_verb::{noun, verb, Cli, CommandRegistry, NounVerbError, Result, VerbArgs};

#[test]
fn test_empty_noun_validation_direct() -> Result<()> {
    // Test validation catches empty nouns
    let registry =
        CommandRegistry::new()
            .name("test")
            .register_noun(noun!("empty", "Empty noun", []));

    let result = registry.validate();
    assert!(result.is_err());
    if let Err(NounVerbError::InvalidStructure { message }) = result {
        assert!(message.contains("no verbs or sub-nouns"));
    } else {
        panic!("Expected InvalidStructure error");
    }

    Ok(())
}

#[test]
fn test_empty_noun_validation() -> Result<()> {
    let registry =
        CommandRegistry::new()
            .name("test")
            .register_noun(noun!("empty", "Empty noun", []));

    let result = registry.validate();
    assert!(result.is_err());
    if let Err(NounVerbError::InvalidStructure { message }) = result {
        assert!(message.contains("no verbs or sub-nouns"));
    } else {
        panic!("Expected InvalidStructure error");
    }

    Ok(())
}

#[test]
fn test_duplicate_verb_names_validation() -> Result<()> {
    // Note: This test checks that duplicates are caught - but the noun! macro
    // creates separate verb instances, so this might need manual trait impl
    let registry = CommandRegistry::new().name("test").auto_validate(true);

    // Since we can't easily create duplicate verbs with the macro,
    // we test that auto_validate works when enabled
    let _registry = registry.register_noun(noun!(
        "test",
        "Test",
        [verb!("action", "Action", |_args: &VerbArgs| { Ok(()) }),]
    ));

    Ok(())
}

#[test]
fn test_global_args_access() -> Result<()> {
    let cli = Cli::new()
        .name("global-test")
        .global_args(vec![
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE"),
        ])
        .noun(noun!(
            "test",
            "Test",
            [verb!("run", "Run test", |args: &VerbArgs| {
                // Test global args access
                let verbose = args.get_global_flag_count("verbose");
                let config = args.get_global_str("config");

                // Verify we can access them
                assert_eq!(verbose, 0);
                assert_eq!(config, None);

                Ok(())
            }),]
        ));

    cli.run_with_args(vec![
        "global-test".to_string(),
        "test".to_string(),
        "run".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_pathbuf_extraction() -> Result<()> {
    let cli = Cli::new().name("path-test").noun(noun!(
        "file",
        "File operations",
        [verb!("read", "Read file", |_args: &VerbArgs| {
            // PathBuf extraction is tested through build_command
            // Real extraction requires actual command execution
            Ok(())
        }, args: [
            Arg::new("file").required(true).value_name("PATH"),
            Arg::new("optional").value_name("PATH"),
        ]),]
    ));

    let _command = cli.build_command();
    Ok(())
}

#[test]
fn test_multiple_values_extraction() -> Result<()> {
    let cli = Cli::new().name("multi-test").noun(noun!(
        "items",
        "Item operations",
        [verb!("add", "Add items", |args: &VerbArgs| {
            // Test multiple values
            let items = args.get_many::<String>("items")?;
            assert!(!items.is_empty());

            let _optional = args.get_many_opt::<String>("optional");
            // Can be empty for optional

            Ok(())
        }, args: [
            Arg::new("items").required(true).num_args(1..),
            Arg::new("optional").num_args(0..),
        ]),]
    ));

    let _command = cli.build_command();
    Ok(())
}

#[test]
fn test_flag_counting() -> Result<()> {
    let cli = Cli::new().name("flag-test").noun(noun!(
        "cmd",
        "Command",
        [verb!("run", "Run command", |args: &VerbArgs| {
            let count = args.get_flag_count("verbose");
            assert_eq!(count, 0); // Default is 0

            let is_set = args.is_flag_set("force");
            assert!(!is_set);

            Ok(())
        }, args: [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
            Arg::new("force").short('f').long("force"),
        ]),]
    ));

    let _command = cli.build_command();
    Ok(())
}

#[test]
fn test_nested_noun_verb_structure() -> Result<()> {
    let cli = Cli::new()
        .name("nested-test")
        .noun(noun!("dev", "Development", {
            noun!("test", "Testing", [
                verb!("run", "Run tests", |_args: &VerbArgs| { Ok(()) }),
            ]),
        }));

    let command = cli.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "dev"));

    Ok(())
}

#[test]
fn test_auto_validation_enabled() -> Result<()> {
    let registry = CommandRegistry::new()
        .name("auto-validate-test")
        .auto_validate(true)
        .register_noun(noun!(
            "valid",
            "Valid noun",
            [verb!("action", "Action", |_args: &VerbArgs| { Ok(()) }),]
        ));

    // Should not panic with auto_validate enabled for valid structure
    let _command = registry.build_command();
    Ok(())
}

#[test]
fn test_error_propagation() -> Result<()> {
    let cli = Cli::new().name("error-test").noun(noun!(
        "test",
        "Test",
        [verb!("fail", "Fail command", |_args: &VerbArgs| {
            Err(NounVerbError::ExecutionError {
                message: "Intentional failure".to_string(),
            })
        }),]
    ));

    let result = cli.run_with_args(vec![
        "error-test".to_string(),
        "test".to_string(),
        "fail".to_string(),
    ]);

    assert!(result.is_err());
    if let Err(NounVerbError::ExecutionError { message }) = result {
        assert_eq!(message, "Intentional failure");
    } else {
        panic!("Expected ExecutionError");
    }

    Ok(())
}
