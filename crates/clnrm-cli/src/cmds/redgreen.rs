//! Red-green command implementation
//!
//! Validates Test-Driven Development (TDD) workflow by ensuring:
//! - RED: Tests fail before implementation (proper failing test)
//! - GREEN: Tests pass after implementation (working solution)
//!
//! Follows 80/20 principle: Focus on core TDD validation with clear feedback.

use clap::Args;
use clnrm_core::cli::commands::run_red_green_validation;
use clnrm_core::error::Result;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct RedGreenArgs {
    /// Test files or directories to validate
    #[arg(value_name = "PATH")]
    pub paths: Vec<String>,

    /// Expect red state (tests should fail)
    #[arg(long)]
    pub expect: Option<String>,

    /// Verify red state (deprecated, use --expect red)
    #[arg(long)]
    pub verify_red: bool,

    /// Verify green state (deprecated, use --expect green)
    #[arg(long)]
    pub verify_green: bool,
}

/// TDD state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TddState {
    Red,
    Green,
}

impl TddState {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "red" => Ok(TddState::Red),
            "green" => Ok(TddState::Green),
            _ => Err(clnrm_core::error::CleanroomError::validation_error(
                format!("Invalid TDD state: {}. Must be 'red' or 'green'", s)
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TddState::Red => "red",
            TddState::Green => "green",
        }
    }
}

/// Run the red-green command
///
/// # Arguments
/// * `paths` - Test files or directories to validate
/// * `expect` - Expected TDD state ("red" or "green")
/// * `verify_red` - Legacy flag for red state validation
/// * `verify_green` - Legacy flag for green state validation
///
/// # Returns
/// * `Result<()>` - Success if TDD state validation passes, error with details if fails
///
/// # Core Team Standards
/// - Validates proper TDD workflow (RED before GREEN)
/// - Clear feedback for development process
/// - No false positives in workflow validation
pub async fn run(args: &RedGreenArgs) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Arrange: Convert inputs and validate
    let paths: Vec<PathBuf> = args.paths.iter().map(PathBuf::from).collect();

    if paths.is_empty() {
        return Err(clnrm_core::error::CleanroomError::config_error(
            "No test paths provided for red-green validation"
        ));
    }

    // Handle expect flag (new API) vs legacy flags
    let (verify_red, verify_green) = if let Some(expect) = &args.expect {
        match expect.as_str() {
            "red" => (true, false),
            "green" => (false, true),
            _ => {
                return Err(clnrm_core::error::CleanroomError::config_error(
                    format!("Invalid expect value: {}. Use 'red' or 'green'", expect)
                ));
            }
        }
    } else {
        // Use legacy flags if expect not provided
        (args.verify_red, args.verify_green)
    };

    if !verify_red && !verify_green {
        println!("💡 Red-Green TDD Workflow Validation");
        println!("");
        println!("Validates Test-Driven Development workflow:");
        println!("  --expect red   - Verify tests fail before implementation");
        println!("  --expect green - Verify tests pass after implementation");
        println!("");
        println!("Example: clnrm redgreen --expect red tests/");
        return Ok(());
    }

    // Act: Run TDD validation
    run_red_green_validation(&paths, verify_red, verify_green).await
}