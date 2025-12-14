//! Stress command implementation

use clap::Args;
use clnrm_core::error::Result;

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
        println!("📄 Generating example stress test configuration...");
        let example = clnrm_core::cli::commands::stress::generate_stress_config_example();
        println!("{}", example);
        println!("");
        println!("💡 Copy the above configuration to a .toml file and run:");
        println!("   clnrm stress path/to/config.toml");
        return Ok(());
    }

    if let Some(config_path) = &args.load_config {
        println!("📂 Loading and validating configuration: {}", config_path);

        let config = clnrm_core::cli::commands::stress::load_stress_config(
            &std::path::PathBuf::from(config_path),
        )
        .map_err(|e| {
            clnrm_core::error::CleanroomError::config_error(format!(
                "Failed to load stress config '{}': {}",
                config_path, e
            ))
        })?;

        println!("✅ Configuration loaded successfully");
        println!("📊 Configuration summary:");
        println!("   - Containers: {}", config.containers.len());
        println!("   - Test count: {}", config.test_count);
        println!("   - Span depth: {}", config.span_depth);
        println!("   - Max containers: {}", config.max_containers);
        println!("   - Concurrency: {}", config.concurrency);
        println!(
            "   - Max memory: {} MB",
            config.max_memory_mb.unwrap_or(1024)
        );
        println!("   - Timeout: {}s", config.timeout_secs.unwrap_or(300));
        println!("   - Fail fast: {}", config.fail_fast);

        return Ok(());
    }

    if let Some(config_path) = &args.config {
        println!("🏃 Running stress test with config: {}", config_path);

        // Load configuration
        let config = clnrm_core::cli::commands::stress::load_stress_config(
            &std::path::PathBuf::from(config_path),
        )
        .map_err(|e| {
            clnrm_core::error::CleanroomError::config_error(format!(
                "Failed to load stress config '{}': {}",
                config_path, e
            ))
        })?;

        println!("🚀 Starting stress test execution...");
        println!("   Containers: {}", config.containers.len());
        println!(
            "   Total tests: {}",
            config.test_count * config.containers.len()
        );
        println!("   Concurrency: {}", config.concurrency);
        println!("");

        // Execute stress test using core functionality
        clnrm_core::cli::commands::stress::run_stress_test(
            config.containers,
            config.test_count,
            config.span_depth,
            config.max_containers,
            config.concurrency,
            config.max_memory_mb,
            config.timeout_secs,
            config.fail_fast,
            config.output_dir,
        )
        .await?;

        println!("");
        println!("✅ Stress test completed successfully!");
        println!("📊 Results saved to output directory (if configured)");
        return Ok(());
    }

    // No valid arguments provided - show help
    println!("❓ No arguments provided.");
    println!("");
    println!("Usage examples:");
    println!("  clnrm stress --generate-example          # Generate example config");
    println!("  clnrm stress --load-config config.toml    # Validate config without running");
    println!("  clnrm stress config.toml                  # Run stress test");
    println!("");

    Err(clnrm_core::error::CleanroomError::config_error(
        "No arguments provided. Use --generate-example, --load-config <file>, or provide a config file",
    ))
}
