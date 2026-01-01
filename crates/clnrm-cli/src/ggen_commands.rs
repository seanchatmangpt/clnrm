use clnrm_core::GenGenServiceLoader;
use clap::Parser;
use std::error::Error;

#[derive(Parser)]
pub struct GenGenArgs {
    #[command(subcommand)]
    pub command: GenGenCommand,
}

#[derive(Parser)]
pub enum GenGenCommand {
    ListServices,
    TestService { service: String },
    CheckHealth { service: String },
}

pub async fn handle_ggen(args: GenGenArgs) -> Result<(), Box<dyn Error>> {
    match args.command {
        GenGenCommand::ListServices => list_services().await,
        GenGenCommand::TestService { service } => test_service(&service).await,
        GenGenCommand::CheckHealth { service } => check_health(&service).await,
    }
}

async fn list_services() -> Result<(), Box<dyn Error>> {
    let services = GenGenServiceLoader::load_services()?;
    println!("Available services from ggen:");
    for service in services {
        println!("  - {}", service.name());
    }
    Ok(())
}

async fn test_service(service_name: &str) -> Result<(), Box<dyn Error>> {
    let services = GenGenServiceLoader::load_services()?;

    for service in services {
        if service.name() == service_name {
            match service.start() {
                Ok(handle) => {
                    println!("✓ Service '{}' started", service_name);
                    println!("  ID: {}", handle.id);
                    println!("  Metadata:");
                    for (k, v) in &handle.metadata {
                        println!("    {}: {}", k, v);
                    }
                    let _ = service.stop(handle);
                    return Ok(());
                }
                Err(e) => {
                    return Err(format!("Failed to start service: {}", e).into());
                }
            }
        }
    }

    Err(format!("Service '{}' not found", service_name).into())
}

async fn check_health(service_name: &str) -> Result<(), Box<dyn Error>> {
    let services = GenGenServiceLoader::load_services()?;

    for service in services {
        if service.name() == service_name {
            match service.start() {
                Ok(handle) => {
                    let health = service.health_check(&handle);
                    println!("Service '{}' health: {:?}", service_name, health);
                    let _ = service.stop(handle);
                    return Ok(());
                }
                Err(e) => {
                    return Err(format!("Failed to check health: {}", e).into());
                }
            }
        }
    }

    Err(format!("Service '{}' not found", service_name).into())
}
