//! Stress command implementation

use clap::Args;
use clnrm_core::cli::commands::stress::{
    generate_stress_config_example, load_stress_config, run_stress_test,
};
use clnrm_core::error::Result;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct StressArgs {
    /// Stress test configuration file
    #[arg(value_name = "CONFIG")]
    pub config: Option<String>,

    /// Generate example configuration
    #[arg(long)]
    pub generate_example: bool,

    /// Load and validate configuration
    #[arg(long)]
    pub load_config: Option<String>,
}

/// Run the stress command
///
/// # Arguments
/// * `config` - Path to stress test configuration file
/// * `generate_example` - Generate example configuration and exit
/// * `load_config` - Load and validate configuration without running tests
///
/// # Returns
/// * `Result<()>` - Success if stress test completes, error if tests fail
///
/// # Core Team Standards
/// - Deterministic chaos injection (seeded random)
/// - Comprehensive telemetry collection
/// - Clear pass/fail reporting with performance metrics
pub async fn run(args: &StressArgs) -> Result<()> {
    println!("⚡ Stress Testing");
    println!("=================");
    println!("");

    if args.generate_example {
        println!("📄 Example stress test configuration:\n");
        let example = generate_stress_config_example();
        println!("{}", example);
        return Ok(());
    }

    if let Some(config_path) = &args.load_config {
        println!("📂 Loading configuration: {}", config_path);
        let path = PathBuf::from(config_path);
        let config = load_stress_config(&path)?;
        println!("✓ Configuration validated successfully!\n");
        println!("Configuration summary:");
        println!("  Containers: {:?}", config.containers);
        println!("  Test count per container: {}", config.test_count);
        println!("  Span depth: {}", config.span_depth);
        println!("  Max containers: {}", config.limits.max_containers);
        println!("  Concurrency: {}", config.concurrency);
        println!("  Total permutations: {}", config.total_permutations());
        return Ok(());
    }

    if let Some(config_path) = &args.config {
        println!("🏃 Running stress test with config: {}", config_path);
        let path = PathBuf::from(config_path);
        let config = load_stress_config(&path)?;

        // Run stress test with loaded configuration
        run_stress_test(
            config.containers,
            config.test_count,
            config.span_depth,
            config.limits.max_containers,
            config.concurrency,
            Some(config.limits.max_memory_mb),
            Some(config.test_timeout.as_secs()),
            config.fail_fast,
            config.output_dir,
        )
        .await?;

        return Ok(());
    }

    // No valid arguments provided
    Err(clnrm_core::error::CleanroomError::config_error(
        "No arguments provided. Use --generate-example, --load-config <file>, or <config-file>",
    ))
}
