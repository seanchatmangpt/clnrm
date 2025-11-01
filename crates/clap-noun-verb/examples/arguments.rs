//! Example demonstrating argument extraction and global args access

use clap::Arg;
use clap_noun_verb::{noun, run_cli, verb, Result, VerbArgs};

fn main() -> Result<()> {
    run_cli(|cli| {
        cli.name("example")
            .about("Example CLI with argument extraction")
            .global_args(vec![
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::Count)
                    .help("Increase verbosity"),
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Configuration file"),
            ])
            .noun(noun!("services", "Manage services", [
                verb!("logs", "Show logs for a service", |args: &VerbArgs| {
                    // Extract verb-specific arguments
                    let service = args.get_one_str("service")?;
                    let lines = args.get_one_opt::<usize>("lines").unwrap_or(50);
                    
                    // Access global arguments from parent
                    let verbose = args.get_global_flag_count("verbose");
                    let config = args.get_global_str("config");
                    
                    // Use the arguments
                    if verbose > 0 {
                        println!("[Verbose level: {}] Showing {} lines of logs for service '{}'", verbose, lines, service);
                    } else {
                        println!("Showing {} lines of logs for service '{}'", lines, service);
                    }
                    
                    if let Some(config_file) = config {
                        println!("Using config file: {}", config_file);
                    }
                    
                    println!("Log entries for {}:", service);
                    for i in 1..=lines.min(10) {
                        println!("  [{}] Log entry {}", i, i);
                    }
                    
                    Ok(())
                }, args: [
                    Arg::new("service").required(true).help("Service name"),
                    Arg::new("lines").short('n').long("lines").default_value("50").help("Number of lines to show"),
                ]),
                verb!("restart", "Restart a service", |args: &VerbArgs| {
                    // Extract service name
                    let service = args.get_one_str("service")?;
                    let force = args.is_flag_set("force");
                    
                    // Access global verbose flag
                    let verbose = args.is_global_flag_set("verbose");
                    
                    if verbose {
                        println!("[Verbose] Restarting service '{}' (force: {})", service, force);
                    } else {
                        println!("Restarting service '{}'", service);
                    }
                    
                    if force {
                        println!("Force restarting...");
                    } else {
                        println!("Graceful restart...");
                    }
                    
                    println!("✓ Service '{}' restarted", service);
                    Ok(())
                }, args: [
                    Arg::new("service").required(true).help("Service name"),
                    Arg::new("force").short('f').long("force").help("Force restart"),
                ]),
                verb!("deploy", "Deploy a service", |args: &VerbArgs| {
                    // Extract multiple arguments
                    let service = args.get_one_str("service")?;
                    let image = args.get_one_str_opt("image");
                    let config_path = args.get_path_opt("config");
                    
                    // Access global args
                    let verbose = args.get_global_flag_count("verbose");
                    
                    if verbose > 1 {
                        println!("[DEBUG] Deploying service '{}'", service);
                        if let Some(ref img) = image {
                            println!("[DEBUG] Using image: {}", img);
                        }
                        if let Some(ref cfg) = config_path {
                            println!("[DEBUG] Config path: {}", cfg.display());
                        }
                    }
                    
                    println!("Deploying service '{}'", service);
                    if let Some(img) = image {
                        println!("  Image: {}", img);
                    }
                    if let Some(cfg) = config_path {
                        println!("  Config: {}", cfg.display());
                    }
                    
                    println!("✓ Service '{}' deployed", service);
                    Ok(())
                }, args: [
                    Arg::new("service").required(true).help("Service name"),
                    Arg::new("image").long("image").help("Container image"),
                    Arg::new("config").long("config").value_name("FILE").help("Deployment config file"),
                ]),
            ]))
    })
}
