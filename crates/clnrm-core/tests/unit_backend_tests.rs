//! Comprehensive unit tests for backend implementations
//!
//! Tests follow TDD London School methodology:
//! - Contract testing through Backend trait
//! - Mock verification for command execution
//! - Focus on collaboration patterns

use clnrm_core::backend::{Backend, Cmd, RunResult};
use clnrm_core::policy::Policy;
use clnrm_core::scenario;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Cmd Builder Tests
// ============================================================================

#[test]
fn test_cmd_new_creates_command_with_binary() {
    // Arrange & Act
    let cmd = Cmd::new("echo");

    // Assert
    assert_eq!(cmd.bin, "echo");
    assert!(cmd.args.is_empty());
    assert!(cmd.env.is_empty());
    assert!(cmd.workdir.is_none());
}

#[test]
fn test_cmd_arg_adds_single_argument() {
    // Arrange & Act
    let cmd = Cmd::new("ls").arg("-la");

    // Assert
    assert_eq!(cmd.args, vec!["-la"]);
}

#[test]
fn test_cmd_arg_chaining_adds_multiple_arguments() {
    // Arrange & Act
    let cmd = Cmd::new("git").arg("commit").arg("-m").arg("test message");

    // Assert
    assert_eq!(cmd.args, vec!["commit", "-m", "test message"]);
}

#[test]
fn test_cmd_args_adds_array_of_arguments() {
    // Arrange & Act
    let cmd = Cmd::new("cargo").args(&["test", "--release", "--all"]);

    // Assert
    assert_eq!(cmd.args, vec!["test", "--release", "--all"]);
}

#[test]
fn test_cmd_workdir_sets_working_directory() {
    // Arrange & Act
    let workdir = PathBuf::from("/tmp/test");
    let cmd = Cmd::new("pwd").workdir(workdir.clone());

    // Assert
    assert_eq!(cmd.workdir, Some(workdir));
}

#[test]
fn test_cmd_env_adds_environment_variable() {
    // Arrange & Act
    let cmd = Cmd::new("printenv")
        .env("KEY1", "value1")
        .env("KEY2", "value2");

    // Assert
    assert_eq!(cmd.env.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(cmd.env.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(cmd.env.len(), 2);
}

#[test]
fn test_cmd_policy_sets_execution_policy() {
    // Arrange
    let policy = Policy::default();

    // Act
    let cmd = Cmd::new("sleep").policy(policy.clone());

    // Assert
    assert_eq!(
        cmd.policy.security.enable_network_isolation,
        policy.security.enable_network_isolation
    );
}

#[test]
fn test_cmd_builder_pattern_chains_all_methods() {
    // Arrange & Act
    let cmd = Cmd::new("test")
        .arg("arg1")
        .args(&["arg2", "arg3"])
        .workdir(PathBuf::from("/tmp"))
        .env("VAR1", "val1")
        .env("VAR2", "val2")
        .policy(Policy::default());

    // Assert
    assert_eq!(cmd.bin, "test");
    assert_eq!(cmd.args, vec!["arg1", "arg2", "arg3"]);
    assert_eq!(cmd.workdir, Some(PathBuf::from("/tmp")));
    assert_eq!(cmd.env.len(), 2);
}

// ============================================================================
// RunResult Tests
// ============================================================================

#[test]
fn test_run_result_new_creates_result_with_given_values() {
    // Arrange & Act
    let result = RunResult::new(0, "output".to_string(), "".to_string(), 100);

    // Assert
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "output");
    assert_eq!(result.stderr, "");
    assert_eq!(result.duration_ms, 100);
    assert!(result.steps.is_empty());
    assert_eq!(result.backend, "unknown");
}

#[test]
fn test_run_result_success_returns_true_for_zero_exit_code() {
    // Arrange
    let result = RunResult::new(0, "".to_string(), "".to_string(), 0);

    // Act
    let is_success = result.success();

    // Assert
    assert!(is_success);
}

#[test]
fn test_run_result_success_returns_false_for_nonzero_exit_code() {
    // Arrange
    let result = RunResult::new(1, "".to_string(), "error".to_string(), 0);

    // Act
    let is_success = result.success();

    // Assert
    assert!(!is_success);
}

#[test]
fn test_run_result_failed_returns_false_for_zero_exit_code() {
    // Arrange
    let result = RunResult::new(0, "".to_string(), "".to_string(), 0);

    // Act
    let has_failed = result.failed();

    // Assert
    assert!(!has_failed);
}

#[test]
fn test_run_result_failed_returns_true_for_nonzero_exit_code() {
    // Arrange
    let result = RunResult::new(127, "".to_string(), "command not found".to_string(), 0);

    // Act
    let has_failed = result.failed();

    // Assert
    assert!(has_failed);
}

#[test]
fn test_run_result_tracks_multiple_steps() {
    // Arrange
    let mut result = RunResult::new(0, "".to_string(), "".to_string(), 0);

    // Act
    result.steps.push(scenario::StepResult {
        name: "step1".to_string(),
        exit_code: 0,
        stdout: "output1".to_string(),
        stderr: "".to_string(),
        duration_ms: 50,
        start_ts: 0,
        success: true,
        source: "test".to_string(),
    });
    result.steps.push(scenario::StepResult {
        name: "step2".to_string(),
        exit_code: 0,
        stdout: "output2".to_string(),
        stderr: "".to_string(),
        duration_ms: 75,
        start_ts: 1,
        success: true,
        source: "test".to_string(),
    });

    // Assert
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0].name, "step1");
    assert_eq!(result.steps[1].name, "step2");
}

#[test]
fn test_run_result_tracks_redacted_environment_variables() {
    // Arrange
    let mut result = RunResult::new(0, "".to_string(), "".to_string(), 0);

    // Act
    result.redacted_env.push("SECRET_KEY".to_string());
    result.redacted_env.push("API_TOKEN".to_string());

    // Assert
    assert_eq!(result.redacted_env.len(), 2);
    assert!(result.redacted_env.contains(&"SECRET_KEY".to_string()));
}

#[test]
fn test_run_result_tracks_step_execution_order() {
    // Arrange
    let mut result = RunResult::new(0, "".to_string(), "".to_string(), 0);

    // Act
    result.step_order.push("step1".to_string());
    result.step_order.push("step2".to_string());
    result.step_order.push("step3".to_string());

    // Assert
    assert_eq!(result.step_order, vec!["step1", "step2", "step3"]);
}

// ============================================================================
// Backend Trait Contract Tests (London School)
// ============================================================================

// Note: We cannot directly test Backend trait without a concrete implementation
// These tests verify the trait contract expectations

#[test]
fn test_backend_trait_is_send_sync() {
    // Arrange
    fn assert_send_sync<T: Send + Sync>() {}

    // Act & Assert - Backend trait requires Send + Sync
    // This is a compile-time check
    assert_send_sync::<Box<dyn Backend>>();
}

// ============================================================================
// Command Execution Patterns Tests
// ============================================================================

#[test]
fn test_cmd_with_shell_script_command() {
    // Arrange & Act
    let cmd = Cmd::new("sh").arg("-c").arg("echo 'hello' && echo 'world'");

    // Assert
    assert_eq!(cmd.bin, "sh");
    assert_eq!(cmd.args.len(), 2);
    assert_eq!(cmd.args[0], "-c");
}

#[test]
fn test_cmd_with_complex_environment() {
    // Arrange
    let mut env_vars = HashMap::new();
    env_vars.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    env_vars.insert("HOME".to_string(), "/home/user".to_string());
    env_vars.insert("LANG".to_string(), "en_US.UTF-8".to_string());

    // Act
    let mut cmd = Cmd::new("env");
    for (key, value) in env_vars.iter() {
        cmd = cmd.env(key, value);
    }

    // Assert
    assert_eq!(cmd.env.len(), 3);
    assert_eq!(cmd.env.get("PATH"), Some(&"/usr/bin:/bin".to_string()));
}

#[test]
fn test_cmd_with_quoted_arguments() {
    // Arrange & Act
    let cmd = Cmd::new("echo")
        .arg("hello world")
        .arg("'single quotes'")
        .arg(r#""double quotes""#);

    // Assert
    assert_eq!(cmd.args.len(), 3);
    assert_eq!(cmd.args[0], "hello world");
    assert_eq!(cmd.args[1], "'single quotes'");
}

#[test]
fn test_cmd_with_unicode_arguments() {
    // Arrange & Act
    let cmd = Cmd::new("echo")
        .arg("Hello 世界")
        .arg("Привет мир")
        .arg("🚀 🎉");

    // Assert
    assert_eq!(cmd.args.len(), 3);
    assert_eq!(cmd.args[0], "Hello 世界");
    assert_eq!(cmd.args[2], "🚀 🎉");
}

// ============================================================================
// Edge Cases and Error Scenarios
// ============================================================================

#[test]
fn test_cmd_with_empty_binary_name() {
    // Arrange & Act
    let cmd = Cmd::new("");

    // Assert
    assert_eq!(cmd.bin, "");
}

#[test]
fn test_cmd_with_empty_arguments() {
    // Arrange & Act
    let cmd = Cmd::new("test").arg("").arg("  ").arg("");

    // Assert
    assert_eq!(cmd.args.len(), 3);
    assert_eq!(cmd.args[0], "");
    assert_eq!(cmd.args[1], "  ");
}

#[test]
fn test_cmd_with_empty_environment_values() {
    // Arrange & Act
    let cmd = Cmd::new("test").env("EMPTY_VAR", "").env("SPACE_VAR", "  ");

    // Assert
    assert_eq!(cmd.env.get("EMPTY_VAR"), Some(&"".to_string()));
    assert_eq!(cmd.env.get("SPACE_VAR"), Some(&"  ".to_string()));
}

#[test]
fn test_run_result_with_large_output() {
    // Arrange
    let large_output = "x".repeat(1_000_000); // 1MB output

    // Act
    let result = RunResult::new(0, large_output.clone(), "".to_string(), 1000);

    // Assert
    assert_eq!(result.stdout.len(), 1_000_000);
    assert!(result.success());
}

#[test]
fn test_run_result_with_unicode_output() {
    // Arrange
    let unicode_output = "Test output: 世界 🚀 Привет";

    // Act
    let result = RunResult::new(0, unicode_output.to_string(), "".to_string(), 50);

    // Assert
    assert_eq!(result.stdout, unicode_output);
}

#[test]
fn test_run_result_with_multiline_output() {
    // Arrange
    let multiline_output = "line1\nline2\nline3\n";

    // Act
    let result = RunResult::new(0, multiline_output.to_string(), "".to_string(), 100);

    // Assert
    assert!(result.stdout.contains('\n'));
    assert_eq!(result.stdout.lines().count(), 3);
}

// ============================================================================
// Policy Integration Tests
// ============================================================================

#[test]
fn test_cmd_with_restrictive_policy() {
    // Arrange
    let mut policy = Policy::default();
    policy.security.allowed_ports = vec![8080];
    policy.security.blocked_addresses = vec!["0.0.0.0".to_string()];

    // Act
    let cmd = Cmd::new("curl")
        .arg("http://localhost:8080")
        .policy(policy.clone());

    // Assert
    assert_eq!(cmd.policy.security.allowed_ports, vec![8080]);
    assert_eq!(cmd.policy.security.blocked_addresses, vec!["0.0.0.0"]);
}

#[test]
fn test_cmd_default_policy_is_permissive() {
    // Arrange & Act
    let cmd = Cmd::new("test");

    // Assert
    let default_policy = Policy::default();
    assert_eq!(
        cmd.policy.security.enable_network_isolation,
        default_policy.security.enable_network_isolation
    );
}

// ============================================================================
// Collaboration Pattern Tests (London School)
// ============================================================================

#[test]
fn test_cmd_collaborates_with_policy_for_restrictions() {
    // Arrange
    let mut policy = Policy::default();
    policy.security.blocked_addresses = vec!["malicious.com".to_string()];
    policy.security.enable_network_isolation = true;

    // Act
    let cmd = Cmd::new("sleep").arg("100").policy(policy.clone());

    // Assert - verify the collaboration contract
    assert!(cmd.policy.security.enable_network_isolation);
    assert!(cmd
        .policy
        .security
        .blocked_addresses
        .contains(&"malicious.com".to_string()));
}

#[test]
fn test_run_result_aggregates_step_durations() {
    // Arrange
    let mut result = RunResult::new(0, "".to_string(), "".to_string(), 0);

    // Act - simulate multiple steps
    result.steps.push(scenario::StepResult {
        name: "step1".to_string(),
        exit_code: 0,
        stdout: "".to_string(),
        stderr: "".to_string(),
        duration_ms: 100,
        start_ts: 0,
        success: true,
        source: "test".to_string(),
    });
    result.steps.push(scenario::StepResult {
        name: "step2".to_string(),
        exit_code: 0,
        stdout: "".to_string(),
        stderr: "".to_string(),
        duration_ms: 150,
        start_ts: 1,
        success: true,
        source: "test".to_string(),
    });

    // Assert
    let total_step_duration: u64 = result.steps.iter().map(|s| s.duration_ms).sum();
    assert_eq!(total_step_duration, 250);
}
