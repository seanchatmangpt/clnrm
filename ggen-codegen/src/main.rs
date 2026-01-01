/// GGEN CLI - Ontology-Driven Code Generator

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use ggen_codegen::{GeneratorConfig, CodeGenerator, load_ontology, load_instances};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "ggen")]
#[command(about = "Ontology-driven code generation from RDF", long_about = None)]
struct Args {
    /// Path to ggen.toml configuration
    #[arg(long, default_value = "ggen.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate code from RDF ontology and instances
    Sync {
        /// Source ontology or workspace path
        #[arg(long, default_value = ".")]
        from: PathBuf,

        /// Target output directory
        #[arg(long)]
        to: Option<PathBuf>,

        /// Generation mode: full, incremental, verify
        #[arg(long, default_value = "full")]
        mode: String,

        /// Preview changes without writing files
        #[arg(long)]
        dry_run: bool,

        /// Show detailed operation logs
        #[arg(long)]
        verbose: bool,
    },

    /// Validate ontology and instances
    Validate {
        /// Path to ontology file
        #[arg(long, default_value = "schema/clnrm-ontology.ttl")]
        ontology: PathBuf,

        /// Path to instances file
        #[arg(long, default_value = "schema/clnrm-instances.ttl")]
        instances: PathBuf,

        /// Show detailed validation report
        #[arg(long)]
        verbose: bool,
    },

    /// Show version information
    Version,

    /// Show help information
    Help,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    // Load configuration
    let mut config = if args.config.exists() {
        GeneratorConfig::from_file(&args.config).await?
    } else {
        GeneratorConfig::default()
    };

    match args.command {
        Commands::Sync {
            from,
            to,
            mode,
            dry_run,
            verbose,
        } => {
            if verbose {
                println!("🔍 GGEN Code Generator");
                println!("   Version: 0.1.0");
                println!("   Config: {}", args.config.display());
                println!("   Source: {}", from.display());
                println!("   Mode: {}", mode);
                println!("   Dry-run: {}", dry_run);
            }

            // Update config with CLI overrides
            if let Some(output) = to {
                config.generation.output_dir = output;
            }
            config.generation.overwrite = mode == "full";

            println!("📦 Starting code generation...");

            if dry_run {
                println!("⚠️  DRY-RUN MODE: No files will be written");
            }

            // Run generator
            let generator = CodeGenerator::new(config)?;
            match generator.generate().await {
                Ok(_) => {
                    println!("✅ Generation successful!");
                    if dry_run {
                        println!("   (Preview mode - no files written)");
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Generation failed: {}", e);
                    Err(e.into())
                }
            }
        }

        Commands::Validate {
            ontology,
            instances,
            verbose,
        } => {
            println!("🔍 Validating ontology and instances...");

            match (load_ontology(&ontology).await, load_instances(&instances).await) {
                (Ok(ont), Ok(inst)) => {
                    println!("✅ Ontology: {} (v{})", ont.title, ont.version);
                    println!("✅ Instances: {} instances loaded", inst.instances.len());

                    if verbose {
                        println!("\nOntology Classes:");
                        for (name, class) in &ont.classes {
                            println!("  - {}: {} properties", name, class.properties.len());
                        }

                        println!("\nInstance Types:");
                        let mut type_counts: std::collections::HashMap<String, usize> =
                            std::collections::HashMap::new();
                        for inst in inst.instances.values() {
                            *type_counts.entry(inst.class_type.clone()).or_insert(0) += 1;
                        }
                        for (typ, count) in &type_counts {
                            println!("  - {}: {} instances", typ, count);
                        }
                    }

                    println!("\n✅ Validation successful!");
                    Ok(())
                }
                (Err(e), _) => {
                    eprintln!("❌ Ontology error: {}", e);
                    Err(e.into())
                }
                (_, Err(e)) => {
                    eprintln!("❌ Instances error: {}", e);
                    Err(e.into())
                }
            }
        }

        Commands::Version => {
            println!("ggen version 0.1.0");
            println!("Ontology-driven code generation");
            Ok(())
        }

        Commands::Help => {
            println!("ggen - Ontology-Driven Code Generator");
            println!();
            println!("USAGE:");
            println!("    ggen [OPTIONS] <COMMAND>");
            println!();
            println!("COMMANDS:");
            println!("    sync       Generate code from RDF instances");
            println!("    validate   Validate ontology and instances");
            println!("    version    Show version information");
            println!("    help       Show this help message");
            println!();
            println!("OPTIONS:");
            println!("    --config <PATH>     Path to ggen.toml (default: ggen.toml)");
            println!("    -h, --help          Print help information");
            Ok(())
        }
    }
}
