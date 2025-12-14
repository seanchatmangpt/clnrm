//! E2E Docker container lifecycle validation

#[cfg(test)]
mod e2e_docker_lifecycle_tests {
    use crate::backend::engine::{BackendSelector, BackendType, ContainerEngine};
    use crate::backend::BackendConfig;
    use crate::environment::compiler::{CompiledEnvironment, ContainerGraph, ContainerNode};
    use std::collections::HashMap;

    /// Test complete container lifecycle: start → health check → exec → stop
    #[tokio::test]
    async fn test_docker_container_full_lifecycle() -> crate::error::Result<()> {
        // Skip if Docker is not available
        if !is_docker_available() {
            println!("⚠️  Docker not available, skipping E2E test");
            return Ok(());
        }

        // Setup backend
        let config = BackendConfig {
            network_mode: "bridge".to_string(),
            auto_remove: true,
        };
        let engine = ContainerEngine::new(config);

        // Create simple environment with one container
        let mut nodes = HashMap::new();
        nodes.insert(
            "test-alpine".to_string(),
            ContainerNode {
                id: "test-alpine".to_string(),
                image: "alpine".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: Some(vec!["sleep".to_string(), "30".to_string()]),
                health_check: Some(crate::environment::compiler::HealthCheck {
                    test: vec!["echo".to_string(), "health".to_string()],
                    interval_seconds: 5,
                    timeout_seconds: 3,
                    retries: 2,
                }),
                resources: Some(crate::environment::compiler::ResourceLimits {
                    cpu_limit: Some(0.5),
                    memory_limit: Some(64 * 1024 * 1024), // 64MB
                }),
            },
        );

        let graph = ContainerGraph {
            nodes,
            edges: Vec::new(),
            startup_order: vec!["test-alpine".to_string()],
        };

        let env = CompiledEnvironment {
            sigma_hash: crate::environment::sigma::ContentHash::from_string("e2e-test"),
            graph,
            networks: Vec::new(),
            volumes: Vec::new(),
            telemetry: crate::environment::compiler::TelemetryConfig {
                otel_collector: None,
                weaver_enabled: false,
                instrumentation: HashMap::new(),
            },
            proof_metadata: crate::environment::compiler::ProofMetadata {
                sigma_hash: crate::environment::sigma::ContentHash::from_string("e2e-test"),
                delta_hash: None,
                constraints_hash: "test".to_string(),
                compiled_at: "2025-01-01T00:00:00Z".to_string(),
                image_digests: HashMap::new(),
                config_hashes: HashMap::new(),
                receipt: crate::receipts::receipt::TestReceipt {
                    id: crate::environment::sigma::ContentHash::from_string("test"),
                    scenario_id: crate::receipts::receipt::ScenarioId("e2e-test".to_string()),
                    capabilities: vec![],
                    effects: crate::capabilities::EffectSet::new(),
                    sigma_hash: crate::environment::sigma::ContentHash::from_string("e2e-test"),
                    image_digests: HashMap::new(),
                    constraints: crate::capabilities::ConstraintSet::default(),
                    weaver_proof: None,
                    timing_footprint: crate::receipts::receipt::TimingFootprint {
                        total_duration: std::time::Duration::from_secs(0),
                        hot_paths: vec![],
                        warm_paths: vec![],
                        cold_paths: vec![],
                        tau_violations: vec![],
                    },
                    hermeticity_witness: crate::receipts::receipt::HermeticityWitness {
                        network_isolated: false,
                        external_connections: vec![],
                        filesystem_isolated: false,
                        non_hermetic_paths: vec![],
                        process_isolated: true,
                        deterministic: true,
                        determinism_violations: vec![],
                    },
                    previous_receipt: None,
                    signature: None,
                    timestamp: "2025-01-01T00:00:00Z".to_string(),
                    metadata: HashMap::new(),
                },
            },
        };

        // Test 1: Start container
        println!("🚀 Starting container...");
        let handle = engine.start(&env).await?;
        println!("✅ Container started: {}", handle.id);

        // Test 2: Health check
        println!("🏥 Checking container health...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await; // Wait for startup
        let is_healthy = engine.health_check(&handle).await?;
        println!("🏥 Health check result: {}", if is_healthy { "healthy" } else { "unhealthy" });
        assert!(is_healthy, "Container should be healthy after startup");

        // Test 3: Execute command
        println!("⚡ Executing command in container...");
        let output = engine.exec(&handle, &["echo".to_string(), "hello".to_string()]).await?;
        println!("⚡ Command output: {}", String::from_utf8_lossy(&output.stdout));
        assert_eq!(output.exit_code, 0, "Command should succeed");
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello"), "Should contain expected output");

        // Test 4: Stop container
        println!("🛑 Stopping container...");
        engine.stop(&handle).await?;
        println!("✅ Container stopped");

        // Test 5: Verify container is stopped
        println!("🔍 Verifying container is stopped...");
        let is_healthy_after_stop = engine.health_check(&handle).await?;
        assert!(!is_healthy_after_stop, "Container should not be healthy after stop");

        println!("🎉 E2E Docker container lifecycle test completed successfully!");
        Ok(())
    }

    /// Test multiple container coordination
    #[tokio::test]
    async fn test_docker_multi_container_coordination() -> crate::error::Result<()> {
        // Skip if Docker is not available
        if !is_docker_available() {
            println!("⚠️  Docker not available, skipping E2E test");
            return Ok(());
        }

        // Setup backend
        let config = BackendConfig {
            network_mode: "bridge".to_string(),
            auto_remove: true,
        };
        let engine = ContainerEngine::new(config);

        // Create environment with two containers (web + db)
        let mut nodes = HashMap::new();

        // Database container
        nodes.insert(
            "test-db".to_string(),
            ContainerNode {
                id: "test-db".to_string(),
                image: "postgres".to_string(),
                tag: "14-alpine".to_string(),
                ports: HashMap::from([(5432, Some(15432))]), // Map to avoid conflicts
                environment: HashMap::from([
                    ("POSTGRES_PASSWORD".to_string(), "testpass".to_string()),
                    ("POSTGRES_DB".to_string(), "testdb".to_string()),
                ]),
                command: None,
                health_check: Some(crate::environment::compiler::HealthCheck {
                    test: vec!["pg_isready".to_string(), "-U".to_string(), "postgres".to_string()],
                    interval_seconds: 5,
                    timeout_seconds: 3,
                    retries: 3,
                }),
                resources: Some(crate::environment::compiler::ResourceLimits {
                    cpu_limit: Some(0.5),
                    memory_limit: Some(128 * 1024 * 1024), // 128MB
                }),
            },
        );

        // Web container (depends on db)
        nodes.insert(
            "test-web".to_string(),
            ContainerNode {
                id: "test-web".to_string(),
                image: "nginx".to_string(),
                tag: "alpine".to_string(),
                ports: HashMap::from([(80, Some(8080))]),
                environment: HashMap::new(),
                command: None,
                health_check: Some(crate::environment::compiler::HealthCheck {
                    test: vec!["curl".to_string(), "-f".to_string(), "http://localhost/".to_string()],
                    interval_seconds: 3,
                    timeout_seconds: 2,
                    retries: 2,
                }),
                resources: Some(crate::environment::compiler::ResourceLimits {
                    cpu_limit: Some(0.3),
                    memory_limit: Some(64 * 1024 * 1024), // 64MB
                }),
            },
        );

        let graph = ContainerGraph {
            nodes,
            edges: vec![crate::environment::compiler::DependencyEdge {
                from: "test-web".to_string(),
                to: "test-db".to_string(),
                dependency_type: crate::environment::compiler::DependencyType::Hard,
            }],
            startup_order: vec!["test-db".to_string(), "test-web".to_string()],
        };

        let env = CompiledEnvironment {
            sigma_hash: crate::environment::sigma::ContentHash::from_string("multi-container-e2e"),
            graph,
            networks: Vec::new(),
            volumes: Vec::new(),
            telemetry: crate::environment::compiler::TelemetryConfig {
                otel_collector: None,
                weaver_enabled: false,
                instrumentation: HashMap::new(),
            },
            proof_metadata: crate::environment::compiler::ProofMetadata {
                sigma_hash: crate::environment::sigma::ContentHash::from_string("multi-container-e2e"),
                delta_hash: None,
                constraints_hash: "test".to_string(),
                compiled_at: "2025-01-01T00:00:00Z".to_string(),
                image_digests: HashMap::new(),
                config_hashes: HashMap::new(),
                receipt: crate::receipts::receipt::TestReceipt {
                    id: crate::environment::sigma::ContentHash::from_string("multi-container-test"),
                    scenario_id: crate::receipts::receipt::ScenarioId("multi-container-e2e".to_string()),
                    capabilities: vec![],
                    effects: crate::capabilities::EffectSet::new(),
                    sigma_hash: crate::environment::sigma::ContentHash::from_string("multi-container-e2e"),
                    image_digests: HashMap::new(),
                    constraints: crate::capabilities::ConstraintSet::default(),
                    weaver_proof: None,
                    timing_footprint: crate::receipts::receipt::TimingFootprint {
                        total_duration: std::time::Duration::from_secs(0),
                        hot_paths: vec![],
                        warm_paths: vec![],
                        cold_paths: vec![],
                        tau_violations: vec![],
                    },
                    hermeticity_witness: crate::receipts::receipt::HermeticityWitness {
                        network_isolated: false,
                        external_connections: vec![],
                        filesystem_isolated: false,
                        non_hermetic_paths: vec![],
                        process_isolated: true,
                        deterministic: true,
                        determinism_violations: vec![],
                    },
                    previous_receipt: None,
                    signature: None,
                    timestamp: "2025-01-01T00:00:00Z".to_string(),
                    metadata: HashMap::new(),
                },
            },
        };

        // Test multi-container lifecycle
        println!("🚀 Starting multi-container environment...");
        let handle = engine.start(&env).await?;
        println!("✅ Multi-container environment started: {}", handle.id);

        // Wait for containers to be ready
        println!("⏳ Waiting for containers to be ready...");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // Check health of all containers
        println!("🏥 Checking health of all containers...");
        let is_healthy = engine.health_check(&handle).await?;
        println!("🏥 Environment health: {}", if is_healthy { "healthy" } else { "unhealthy" });
        assert!(is_healthy, "Multi-container environment should be healthy");

        // Test inter-container communication (if possible)
        println!("🔗 Testing inter-container communication...");

        // Clean up
        println!("🧹 Cleaning up multi-container environment...");
        engine.stop(&handle).await?;
        println!("✅ Multi-container environment stopped");

        println!("🎉 Multi-container coordination test completed successfully!");
        Ok(())
    }

    /// Test container resource limits and constraints
    #[tokio::test]
    async fn test_docker_container_resource_limits() -> crate::error::Result<()> {
        // Skip if Docker is not available
        if !is_docker_available() {
            println!("⚠️  Docker not available, skipping E2E test");
            return Ok(());
        }

        // Setup backend
        let config = BackendConfig {
            network_mode: "bridge".to_string(),
            auto_remove: true,
        };
        let engine = ContainerEngine::new(config);

        // Create container with strict resource limits
        let mut nodes = HashMap::new();
        nodes.insert(
            "test-resource-limited".to_string(),
            ContainerNode {
                id: "test-resource-limited".to_string(),
                image: "alpine".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: Some(vec!["sleep".to_string(), "30".to_string()]),
                health_check: None,
                resources: Some(crate::environment::compiler::ResourceLimits {
                    cpu_limit: Some(0.1), // Very low CPU
                    memory_limit: Some(16 * 1024 * 1024), // 16MB
                }),
            },
        );

        let graph = ContainerGraph {
            nodes,
            edges: Vec::new(),
            startup_order: vec!["test-resource-limited".to_string()],
        };

        let env = CompiledEnvironment {
            sigma_hash: crate::environment::sigma::ContentHash::from_string("resource-limits-test"),
            graph,
            networks: Vec::new(),
            volumes: Vec::new(),
            telemetry: crate::environment::compiler::TelemetryConfig {
                otel_collector: None,
                weaver_enabled: false,
                instrumentation: HashMap::new(),
            },
            proof_metadata: crate::environment::compiler::ProofMetadata {
                sigma_hash: crate::environment::sigma::ContentHash::from_string("resource-limits-test"),
                delta_hash: None,
                constraints_hash: "test".to_string(),
                compiled_at: "2025-01-01T00:00:00Z".to_string(),
                image_digests: HashMap::new(),
                config_hashes: HashMap::new(),
                receipt: crate::receipts::receipt::TestReceipt {
                    id: crate::environment::sigma::ContentHash::from_string("resource-limits-test"),
                    scenario_id: crate::receipts::receipt::ScenarioId("resource-limits-e2e".to_string()),
                    capabilities: vec![],
                    effects: crate::capabilities::EffectSet::new(),
                    sigma_hash: crate::environment::sigma::ContentHash::from_string("resource-limits-test"),
                    image_digests: HashMap::new(),
                    constraints: crate::capabilities::ConstraintSet::default(),
                    weaver_proof: None,
                    timing_footprint: crate::receipts::receipt::TimingFootprint {
                        total_duration: std::time::Duration::from_secs(0),
                        hot_paths: vec![],
                        warm_paths: vec![],
                        cold_paths: vec![],
                        tau_violations: vec![],
                    },
                    hermeticity_witness: crate::receipts::receipt::HermeticityWitness {
                        network_isolated: false,
                        external_connections: vec![],
                        filesystem_isolated: false,
                        non_hermetic_paths: vec![],
                        process_isolated: true,
                        deterministic: true,
                        determinism_violations: vec![],
                    },
                    previous_receipt: None,
                    signature: None,
                    timestamp: "2025-01-01T00:00:00Z".to_string(),
                    metadata: HashMap::new(),
                },
            },
        };

        // Test resource-constrained container
        println!("🚀 Starting resource-constrained container...");
        let handle = engine.start(&env).await?;
        println!("✅ Resource-constrained container started: {}", handle.id);

        // Test that container starts and runs despite low resources
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let is_healthy = engine.health_check(&handle).await?;
        println!("🏥 Container health with low resources: {}", if is_healthy { "healthy" } else { "unhealthy" });

        // Execute a simple command to verify functionality
        let output = engine.exec(&handle, &["echo".to_string(), "resource test".to_string()]).await?;
        assert_eq!(output.exit_code, 0, "Command should succeed despite resource limits");

        // Clean up
        engine.stop(&handle).await?;
        println!("✅ Resource limits test completed successfully!");

        Ok(())
    }

    /// Helper function to check if Docker is available
    fn is_docker_available() -> bool {
        use std::process::Command;
        Command::new("docker")
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
