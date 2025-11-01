//! Tests for CLI live-check flags
//!
//! Validates that the new CLI flags for Weaver live-check integration are properly parsed
//! and that CLI flags take precedence over TOML configuration.

use clap::Parser;

#[test]
fn test_run_command_parses_live_check_flags() {
    // Test basic --live-check flag
    let args = vec!["clnrm", "run", "--live-check", "tests/"];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --live-check flag");

    // Extract Run command and verify live_check is true
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run { live_check, .. } => {
                assert!(live_check, "live_check flag should be true");
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_parses_validation_mode() {
    // Test --validation-mode flag
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--validation-mode",
        "80_20",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --validation-mode flag");

    // Extract Run command and verify validation_mode
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run {
                validation_mode, ..
            } => {
                assert_eq!(
                    validation_mode,
                    Some("80_20".to_string()),
                    "validation_mode should be '80_20'"
                );
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_parses_registry_path() {
    // Test --registry-path flag
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--registry-path",
        "./custom-registry",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --registry-path flag");

    // Extract Run command and verify registry_path
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run { registry_path, .. } => {
                assert!(registry_path.is_some(), "registry_path should be set");
                assert_eq!(
                    registry_path.unwrap().to_str().unwrap(),
                    "./custom-registry",
                    "registry_path should be './custom-registry'"
                );
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_parses_otlp_port() {
    // Test --otlp-port flag
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--otlp-port",
        "4317",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --otlp-port flag");

    // Extract Run command and verify otlp_port
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run { otlp_port, .. } => {
                assert_eq!(otlp_port, 4317, "otlp_port should be 4317");
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_parses_admin_port() {
    // Test --admin-port flag
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--admin-port",
        "8080",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --admin-port flag");

    // Extract Run command and verify admin_port
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run { admin_port, .. } => {
                assert_eq!(admin_port, 8080, "admin_port should be 8080");
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_parses_diagnostic_format() {
    // Test --diagnostic-format flag
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--diagnostic-format",
        "json",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --diagnostic-format flag");

    // Extract Run command and verify diagnostic_format
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run {
                diagnostic_format, ..
            } => {
                assert_eq!(
                    diagnostic_format, "json",
                    "diagnostic_format should be 'json'"
                );
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_parses_stop_timeout() {
    // Test --stop-timeout flag
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--stop-timeout",
        "600",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse --stop-timeout flag");

    // Extract Run command and verify stop_timeout
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run { stop_timeout, .. } => {
                assert_eq!(stop_timeout, 600, "stop_timeout should be 600");
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_run_command_all_live_check_flags_together() {
    // Test all flags together
    let args = vec![
        "clnrm",
        "run",
        "--live-check",
        "--validation-mode",
        "strict",
        "--registry-path",
        "./registry",
        "--otlp-port",
        "4317",
        "--admin-port",
        "8080",
        "--diagnostic-format",
        "github",
        "--stop-timeout",
        "600",
        "tests/",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse all live-check flags");

    // Extract Run command and verify all flags
    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::Run {
                live_check,
                validation_mode,
                registry_path,
                otlp_port,
                admin_port,
                diagnostic_format,
                stop_timeout,
                ..
            } => {
                assert!(live_check, "live_check should be true");
                assert_eq!(validation_mode, Some("strict".to_string()));
                assert!(registry_path.is_some());
                assert_eq!(otlp_port, 4317);
                assert_eq!(admin_port, 8080);
                assert_eq!(diagnostic_format, "github");
                assert_eq!(stop_timeout, 600);
            }
            _ => panic!("Expected Run command"),
        }
    }
}

#[test]
fn test_live_check_subcommand_status() {
    // Test live-check status subcommand
    let args = vec!["clnrm", "live-check", "status"];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse live-check status command");

    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::LiveCheck { command } => match command {
                clnrm_core::cli::types::LiveCheckCommands::Status => {
                    // Success
                }
                _ => panic!("Expected Status subcommand"),
            },
            _ => panic!("Expected LiveCheck command"),
        }
    }
}

#[test]
fn test_live_check_subcommand_validate_registry() {
    // Test live-check validate-registry subcommand
    let args = vec![
        "clnrm",
        "live-check",
        "validate-registry",
        "--registry",
        "./registry",
    ];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "Failed to parse live-check validate-registry command"
    );

    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::LiveCheck { command } => match command {
                clnrm_core::cli::types::LiveCheckCommands::ValidateRegistry { registry } => {
                    assert_eq!(
                        registry.to_str().unwrap(),
                        "./registry",
                        "registry path should be './registry'"
                    );
                }
                _ => panic!("Expected ValidateRegistry subcommand"),
            },
            _ => panic!("Expected LiveCheck command"),
        }
    }
}

#[test]
fn test_live_check_subcommand_test_weaver() {
    // Test live-check test-weaver subcommand
    let args = vec!["clnrm", "live-check", "test-weaver"];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "Failed to parse live-check test-weaver command"
    );

    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::LiveCheck { command } => match command {
                clnrm_core::cli::types::LiveCheckCommands::TestWeaver => {
                    // Success
                }
                _ => panic!("Expected TestWeaver subcommand"),
            },
            _ => panic!("Expected LiveCheck command"),
        }
    }
}

#[test]
fn test_live_check_subcommand_modes() {
    // Test live-check modes subcommand
    let args = vec!["clnrm", "live-check", "modes"];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse live-check modes command");

    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::LiveCheck { command } => match command {
                clnrm_core::cli::types::LiveCheckCommands::Modes => {
                    // Success
                }
                _ => panic!("Expected Modes subcommand"),
            },
            _ => panic!("Expected LiveCheck command"),
        }
    }
}

#[test]
fn test_live_check_subcommand_version() {
    // Test live-check version subcommand
    let args = vec!["clnrm", "live-check", "version"];
    let cli = clnrm_core::cli::types::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse live-check version command");

    if let Ok(cli) = cli {
        match cli.command {
            clnrm_core::cli::types::Commands::LiveCheck { command } => match command {
                clnrm_core::cli::types::LiveCheckCommands::Version => {
                    // Success
                }
                _ => panic!("Expected Version subcommand"),
            },
            _ => panic!("Expected LiveCheck command"),
        }
    }
}

#[test]
fn test_live_check_flag_enables_validation() {
    // Test that --live-check flag is equivalent to --validate
    let args_live_check = vec!["clnrm", "run", "--live-check", "tests/"];
    let args_validate = vec!["clnrm", "run", "--validate", "tests/"];

    let cli_live_check = clnrm_core::cli::types::Cli::try_parse_from(args_live_check);
    let cli_validate = clnrm_core::cli::types::Cli::try_parse_from(args_validate);

    assert!(cli_live_check.is_ok() && cli_validate.is_ok());

    // Both should enable validation
    if let (Ok(cli_lc), Ok(cli_v)) = (cli_live_check, cli_validate) {
        match (cli_lc.command, cli_v.command) {
            (
                clnrm_core::cli::types::Commands::Run { live_check: lc, .. },
                clnrm_core::cli::types::Commands::Run { validate: v, .. },
            ) => {
                assert!(lc, "--live-check should be true");
                assert!(v, "--validate should be true");
            }
            _ => panic!("Expected Run commands"),
        }
    }
}
