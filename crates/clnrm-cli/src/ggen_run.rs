use clnrm_core::{GenGenServiceLoader, GenGenConfigBuilder, ServiceRegistry};
use std::error::Error;

#[derive(clap::Parser)]
pub struct GenGenRunArgs {
    #[arg(value_name = "TEST", help = "Test to run: simple-echo, comprehensive, database")]
    pub test_name: String,

    #[arg(long, help = "Load services from ggen")]
    pub use_ggen_services: bool,
}

pub async fn run_ggen_test(args: GenGenRunArgs) -> Result<(), Box<dyn Error>> {
    let config = match args.test_name.as_str() {
        "simple-echo" => GenGenConfigBuilder::build_simple_echo_test()?,
        "comprehensive" => GenGenConfigBuilder::build_comprehensive_test()?,
        "database" => GenGenConfigBuilder::build_database_test()?,
        _ => return Err(format!("Unknown test: {}", args.test_name).into()),
    };

    println!("Running test: {}", args.test_name);
    println!("Scenarios: {}", config.scenarios.len());

    for scenario in &config.scenarios {
        println!("\nScenario: {}", scenario.name);
        println!("  Steps: {}", scenario.steps.len());

        for step in &scenario.steps {
            println!("  - {}", step.name);
            println!("    Command: {:?}", step.command);
            if let Some(timeout) = step.timeout {
                println!("    Timeout: {:?}", timeout);
            }
            if let Some(retries) = step.retries {
                println!("    Retries: {}", retries);
            }
        }
    }

    if args.use_ggen_services {
        println!("\nLoading services from ggen...");
        let services = GenGenServiceLoader::load_services()?;
        println!("Loaded {} services:", services.len());

        for service in services {
            println!("  - {}", service.name());
        }

        let mut registry = ServiceRegistry::new();
        registry = registry.with_ggen_plugins()?;
        println!("ServiceRegistry populated with {} plugins", registry.plugins.len());
    }

    println!("\n✓ Test configuration loaded successfully");
    Ok(())
}
