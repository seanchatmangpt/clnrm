//! CLI for gVisor migration tool

use anyhow::Result;
use clap::{Parser, Subcommand};
use clnrm_migrate::MigrationEngine;
use std::path::PathBuf;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "clnrm-migrate")]
#[command(about = "Migrate testcontainers configs to gVisor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan codebase for testcontainers services
    Scan {
        /// Root directory to scan
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Output file for scan results
        #[arg(short, long, default_value = "scan-results.json")]
        output: PathBuf,
    },

    /// Convert configurations to gVisor format
    Convert {
        /// Root directory to scan
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Output directory for converted configs
        #[arg(short, long, default_value = "./migration-output")]
        output: PathBuf,
    },

    /// Validate gVisor configurations
    Validate {
        /// Configuration file to validate
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Run full migration pipeline
    All {
        /// Root directory to scan
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "./migration-output")]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { root, output } => {
            println!("🔍 Scanning {} for testcontainers services...", root.display());
            let mut engine = MigrationEngine::new();
            let discoveries = engine.scanner.scan(&root)?;

            let json = serde_json::to_string_pretty(&discoveries)?;
            std::fs::write(&output, json)?;

            println!("✅ Found {} services", discoveries.len());
            println!("📄 Results written to {}", output.display());
        }

        Commands::Convert { root, output } => {
            println!("🔄 Converting configurations...");
            let mut engine = MigrationEngine::new();

            let discoveries = engine.scanner.scan(&root)?;
            let conversions = engine.converter.convert_all(&discoveries)?;

            engine.converter.write_configs(&conversions, &output)?;

            println!("✅ Converted {} services", conversions.len());
            println!("📄 Configs written to {}", output.display());
        }

        Commands::Validate { config } => {
            println!("✅ Validating {}...", config.display());
            let content = std::fs::read_to_string(&config)?;
            let _parsed: toml::Value = toml::from_str(&content)?;
            println!("✅ Configuration is valid!");
        }

        Commands::All { root, output } => {
            println!("🚀 Running full migration pipeline...");
            println!("📂 Root directory: {}", root.display());
            println!("📂 Output directory: {}", output.display());
            println!();

            let mut engine = MigrationEngine::new();
            let report = engine.migrate(&root, &output)?;

            println!();
            println!("📊 Migration Summary:");
            println!("  Total services: {}", report.total_services);
            println!("  Converted: {}", report.converted_services);
            println!("  Errors: {}", report.validation_errors);
            println!("  Warnings: {}", report.validation_warnings);
            println!();

            if report.validation_errors == 0 {
                println!("✅ Migration completed successfully!");
            } else {
                println!("⚠️  Migration completed with errors. Review the report.");
            }

            println!();
            println!("📄 Report: {}/migration-report.md", output.display());
            println!("⚙️  Config: {}/gvisor-services.toml", output.display());
        }
    }

    Ok(())
}
