use clnrm_core::cleanroom::ServicePlugin;
use clnrm_core::services::chaos_engine::{ChaosConfig, ChaosEnginePlugin, ChaosScenario};
use clnrm_core::services::generic::GenericContainerPlugin;
use clnrm_core::services::ollama::{OllamaConfig, OllamaPlugin};
use clnrm_core::services::otel_collector::OtelCollectorPlugin;
use clnrm_core::services::surrealdb::SurrealDbPlugin;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We create a runtime ONLY if we need it for async calls in OUR main,
    // but the plugins themselves will create their own runtimes or use block_in_place.
    // However, since we are calling plugin.start() which internally creates runtimes,
    // we should NOT have an active tokio runtime in this thread.

    let plugins: Vec<(String, Box<dyn ServicePlugin>)> = vec![
        (
            "OtelCollector".to_string(),
            Box::new(OtelCollectorPlugin::new("otel")),
        ),
        (
            "Generic (Postgres)".to_string(),
            Box::new(GenericContainerPlugin::new("postgres", "postgres:15")),
        ),
        (
            "Generic (Redis)".to_string(),
            Box::new(GenericContainerPlugin::new("redis", "redis:7")),
        ),
        ("SurrealDB".to_string(), Box::new(SurrealDbPlugin::new())),
        (
            "Ollama".to_string(),
            Box::new(OllamaPlugin::new(
                "ollama",
                OllamaConfig {
                    endpoint: "http://localhost:11434".to_string(),
                    default_model: "llama3".to_string(),
                    timeout_seconds: 1,
                },
            )),
        ),
        (
            "ChaosEngine".to_string(),
            Box::new(ChaosEnginePlugin::with_config(
                "chaos",
                ChaosConfig {
                    failure_rate: 0.1,
                    latency_ms: 10,
                    network_partition_rate: 0.0,
                    memory_pressure_mb: 0,
                    cpu_stress_percent: 0,
                    scenarios: vec![ChaosScenario::LatencySpikes {
                        duration_secs: 1,
                        max_latency_ms: 10,
                    }],
                },
            )),
        ),
    ];

    println!("\n# Agent 4 (Mura - Unevenness): Startup Time Measurement\n");
    println!("| Plugin | Startup Time (ms) | Status | Target (<100ms) |");
    println!("|--------|-------------------|--------|-----------------|");

    let mut times = Vec::new();

    for (display_name, plugin) in plugins {
        let start = Instant::now();
        // We measure the start() call which is responsible for initializing the service.
        // For plugins like SurrealDB and ChaosEngine, this involves internal runtimes.
        let result = plugin.start();
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;
        times.push(ms);

        let status = if result.is_ok() { "✅ OK" } else { "❌ FAIL" };
        let target = if ms < 100.0 {
            "✅ PASS"
        } else {
            "❌ INC_MURA"
        };

        println!(
            "| {:<17} | {:>17.2} | {:<6} | {:<15} |",
            display_name, ms, status, target
        );
    }

    let mean = times.iter().sum::<f64>() / times.len() as f64;
    let variance = times.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / times.len() as f64;
    let std_dev = variance.sqrt();

    println!("\n## Statistics");
    println!("- Mean Startup Time: {:.2} ms", mean);
    println!("- Variance: {:.2}", variance);
    println!("- Standard Deviation: {:.2} ms", std_dev);

    if std_dev > 50.0 {
        println!("\n### Mura Waste Detected!");
        println!("High variance in startup times indicates 'Inconsistent Capability'.");
        println!(
            "Some plugins (like ChaosEngine and potentially SurrealDB/Ollama) are 'Incapable'"
        );
        println!("of meeting the sub-100ms TAU execution time targets.");
    }

    Ok(())
}
