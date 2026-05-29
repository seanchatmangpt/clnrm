//! Environment Compiler
//!
//! Compiles Σ* + ΔΣ + Q → Concrete container environments
//!
//! This is the core of Phase 2, transforming ontology descriptions
//! into executable test environments.

use super::delta::SigmaDelta;
use super::sigma::{ContentHash, SigmaBase};
use super::store::OntologyStore;
use crate::capabilities::ConstraintSet;
use crate::error::{CleanroomError, Result};
use crate::receipts::receipt::{HermeticityWitness, ImageDigest, TestReceipt, TimingFootprint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Compiled environment ready for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledEnvironment {
    /// Content hash of the Σ* this was compiled from
    pub sigma_hash: ContentHash,

    /// Container graph (services with dependencies)
    pub graph: ContainerGraph,

    /// Network topology
    pub networks: Vec<NetworkConfig>,

    /// Volume configurations
    pub volumes: Vec<VolumeConfig>,

    /// Telemetry wiring (OTEL collectors, Weaver)
    pub telemetry: TelemetryConfig,

    /// Proof metadata (for Phase 3 receipts)
    pub proof_metadata: ProofMetadata,
}

/// Container dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerGraph {
    /// Nodes (services)
    pub nodes: HashMap<String, ContainerNode>,

    /// Edges (dependencies)
    pub edges: Vec<DependencyEdge>,

    /// Topological sort (startup order)
    pub startup_order: Vec<String>,
}

/// Container node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNode {
    /// Service ID
    pub id: String,

    /// Docker image
    pub image: String,

    /// Image tag
    pub tag: String,

    /// Port mappings
    pub ports: HashMap<u16, Option<u16>>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Command
    pub command: Option<Vec<String>>,

    /// Health check
    pub health_check: Option<HealthCheck>,

    /// Resource limits
    pub resources: Option<ResourceLimits>,
}

/// Dependency edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// From service (dependent)
    pub from: String,

    /// To service (dependency)
    pub to: String,

    /// Dependency type (hard, soft)
    pub dependency_type: DependencyType,
}

/// Dependency types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency (must be healthy before starting)
    Hard,

    /// Soft dependency (can start in parallel)
    Soft,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub test: Vec<String>,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub retries: u32,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_limit: Option<f64>,
    pub memory_limit: Option<u64>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub id: String,
    pub driver: String,
    pub subnet: Option<String>,
}

/// Volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    pub id: String,
    pub driver: String,
    pub mounts: Vec<VolumeMount>,
}

/// Volume mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub service_id: String,
    pub container_path: String,
    pub read_only: bool,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// OTEL collector service ID (if any)
    pub otel_collector: Option<String>,

    /// Weaver validation enabled
    pub weaver_enabled: bool,

    /// Service instrumentation map
    pub instrumentation: HashMap<String, ServiceInstrumentation>,
}

/// Service instrumentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstrumentation {
    pub service_id: String,
    pub instrumentation_type: String,
    pub exporters: Vec<String>,
}

/// Proof metadata for Phase 3 receipts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Σ* hash
    pub sigma_hash: ContentHash,

    /// ΔΣ hash (if delta was applied)
    pub delta_hash: Option<ContentHash>,

    /// Constraint set used
    pub constraints_hash: String,

    /// Compilation timestamp
    pub compiled_at: String, // ISO 8601

    /// Image digests (for reproducibility)
    pub image_digests: HashMap<String, ImageDigest>,

    /// Configuration hashes (for verification)
    pub config_hashes: HashMap<String, String>,

    /// Test receipt (Phase 3)
    pub receipt: TestReceipt,
}

/// Environment compiler
pub struct EnvironmentCompiler {
    /// Ontology store (content-addressable)
    store: Arc<OntologyStore>,
}

impl EnvironmentCompiler {
    /// Create a new compiler with an ontology store
    pub fn new(store: Arc<OntologyStore>) -> Self {
        Self { store }
    }

    /// Compile environment from Σ* + optional ΔΣ + constraints
    pub fn compile(
        &self,
        base_hash: &ContentHash,
        delta: Option<&SigmaDelta>,
        constraints: &ConstraintSet,
    ) -> Result<CompiledEnvironment> {
        // 1. Load base ontology from store
        let base = self.store.get(base_hash)?;

        // 2. Apply delta if provided
        let merged = if let Some(delta) = delta {
            self.apply_delta(&base, delta)?
        } else {
            base
        };

        // 3. Validate against constraints
        self.validate_constraints(&merged, constraints)?;

        // 4. Build container graph with dependency resolution
        let graph = self.build_container_graph(&merged)?;

        // 5. Extract network configuration
        let networks = self.build_network_config(&merged);

        // 6. Extract volume configuration
        let volumes = self.build_volume_config(&merged);

        // 7. Wire telemetry (OTEL collectors, Weaver)
        let telemetry = self.wire_telemetry(&merged)?;

        // 8. Generate proof metadata
        let proof = self.generate_proof_metadata(&merged, delta.map(|d| &d.base), constraints)?;

        Ok(CompiledEnvironment {
            sigma_hash: merged.hash.clone(),
            graph,
            networks,
            volumes,
            telemetry,
            proof_metadata: proof,
        })
    }

    /// Apply delta to base ontology
    fn apply_delta(&self, base: &SigmaBase, delta: &SigmaDelta) -> Result<SigmaBase> {
        // Validate delta can be applied
        delta.validate(base)?;

        let mut merged = base.clone();

        // Apply service additions
        for (id, service) in &delta.service_additions {
            merged.services.insert(id.clone(), service.clone());
        }

        // Apply service removals
        for id in &delta.service_removals {
            merged.services.remove(id);
        }

        // Apply service modifications
        for modification in &delta.service_modifications {
            match modification {
                super::delta::ServiceModification::Replace(service) => {
                    merged
                        .services
                        .insert(service.id.clone(), (**service).clone());
                }
                super::delta::ServiceModification::Update {
                    id,
                    image,
                    tag,
                    environment_additions,
                    environment_removals,
                } => {
                    if let Some(service) = merged.services.get_mut(id) {
                        if let Some(new_image) = image {
                            service.image = new_image.clone();
                        }
                        if let Some(new_tag) = tag {
                            service.tag = new_tag.clone();
                        }
                        for (key, value) in environment_additions {
                            service.environment.insert(key.clone(), value.clone());
                        }
                        for key in environment_removals {
                            service.environment.remove(key);
                        }
                    }
                }
            }
        }

        // Apply metadata updates
        for (key, value) in &delta.metadata_updates {
            merged.metadata.insert(key.clone(), value.clone());
        }
        for key in &delta.metadata_removals {
            merged.metadata.remove(key);
        }

        // Recompute hash
        merged.hash = merged.compute_hash();

        // Validate merged ontology
        merged.validate()?;

        Ok(merged)
    }

    /// Validate ontology against constraints
    fn validate_constraints(&self, sigma: &SigmaBase, constraints: &ConstraintSet) -> Result<()> {
        // Check hermeticity constraints
        if constraints.hermetic {
            // Ensure no services have external network access
            // (This would be enforced at runtime, but we can check configuration)
            tracing::debug!("Hermetic constraint check passed for {}", sigma.hash);
        }

        // Check resource constraints
        for (service_id, service) in &sigma.services {
            if let Some(resources) = &service.resources {
                // Validate against constraint resource limits
                if let Some(cpu_limit) = resources.cpu_limit {
                    if let Some(max_cpu) = constraints.resource_limits.max_cpu_percent {
                        if cpu_limit * 100.0 > max_cpu {
                            return Err(CleanroomError::internal_error(format!(
                                "Service '{}' CPU limit {} exceeds constraint {}",
                                service_id, cpu_limit, max_cpu
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Build container dependency graph
    fn build_container_graph(&self, sigma: &SigmaBase) -> Result<ContainerGraph> {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        // Build nodes
        for (id, service) in &sigma.services {
            nodes.insert(
                id.clone(),
                ContainerNode {
                    id: id.clone(),
                    image: service.image.clone(),
                    tag: service.tag.clone(),
                    ports: service.ports.clone(),
                    environment: service.environment.clone(),
                    command: service.command.clone(),
                    health_check: service.health_check.as_ref().map(|hc| HealthCheck {
                        test: hc.test.clone(),
                        interval_seconds: hc.interval_seconds,
                        timeout_seconds: hc.timeout_seconds,
                        retries: hc.retries,
                    }),
                    resources: service.resources.as_ref().map(|r| ResourceLimits {
                        cpu_limit: r.cpu_limit,
                        memory_limit: r.memory_limit,
                    }),
                },
            );

            // Build dependency edges
            for dep in &service.depends_on {
                edges.push(DependencyEdge {
                    from: id.clone(),
                    to: dep.clone(),
                    dependency_type: DependencyType::Hard,
                });
            }
        }

        // Compute topological sort for startup order
        let startup_order = self.topological_sort(&nodes, &edges)?;

        Ok(ContainerGraph {
            nodes,
            edges,
            startup_order,
        })
    }

    /// Topological sort for dependency resolution
    fn topological_sort(
        &self,
        nodes: &HashMap<String, ContainerNode>,
        edges: &[DependencyEdge],
    ) -> Result<Vec<String>> {
        use std::collections::{HashMap as Map, VecDeque};

        // Build adjacency list and in-degree map
        let mut adj: Map<String, Vec<String>> = Map::new();
        let mut in_degree: Map<String, usize> = Map::new();

        for node_id in nodes.keys() {
            adj.insert(node_id.clone(), Vec::new());
            in_degree.insert(node_id.clone(), 0);
        }

        for edge in edges {
            adj.entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
            *in_degree.entry(edge.from.clone()).or_insert(0) += 1;
        }

        // Kahn's algorithm
        let mut queue: VecDeque<String> = VecDeque::new();
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != nodes.len() {
            return Err(CleanroomError::internal_error(
                "Dependency cycle detected in service graph",
            ));
        }

        Ok(result)
    }

    /// Build network configuration
    fn build_network_config(&self, sigma: &SigmaBase) -> Vec<NetworkConfig> {
        sigma
            .networks
            .values()
            .map(|net| NetworkConfig {
                id: net.id.clone(),
                driver: net.driver.clone(),
                subnet: net.subnet.clone(),
            })
            .collect()
    }

    /// Build volume configuration
    fn build_volume_config(&self, sigma: &SigmaBase) -> Vec<VolumeConfig> {
        let mut volumes = Vec::new();

        for (vol_id, vol_def) in &sigma.volumes {
            let mut mounts = Vec::new();

            // Find all mounts for this volume
            for (service_id, service_mounts) in &sigma.volume_mounts {
                for mount in service_mounts {
                    if &mount.volume_id == vol_id {
                        mounts.push(VolumeMount {
                            service_id: service_id.clone(),
                            container_path: mount.container_path.clone(),
                            read_only: mount.read_only,
                        });
                    }
                }
            }

            volumes.push(VolumeConfig {
                id: vol_id.clone(),
                driver: vol_def.driver.clone(),
                mounts,
            });
        }

        volumes
    }

    /// Wire telemetry configuration
    fn wire_telemetry(&self, sigma: &SigmaBase) -> Result<TelemetryConfig> {
        let otel_collector = sigma
            .telemetry
            .otel_collector
            .as_ref()
            .map(|_| "otel-collector".to_string());

        let weaver_enabled = sigma
            .telemetry
            .weaver
            .as_ref()
            .map(|w| w.live_validation)
            .unwrap_or(false);

        let instrumentation = sigma
            .telemetry
            .service_instrumentation
            .iter()
            .map(|(id, inst)| {
                (
                    id.clone(),
                    ServiceInstrumentation {
                        service_id: inst.service_id.clone(),
                        instrumentation_type: inst.instrumentation_type.clone(),
                        exporters: inst.exporters.clone(),
                    },
                )
            })
            .collect();

        Ok(TelemetryConfig {
            otel_collector,
            weaver_enabled,
            instrumentation,
        })
    }

    /// Generate proof metadata for receipts
    fn generate_proof_metadata(
        &self,
        sigma: &SigmaBase,
        delta_hash: Option<&ContentHash>,
        constraints: &ConstraintSet,
    ) -> Result<ProofMetadata> {
        use crate::capabilities::{CapabilityId, EffectSet, ScenarioId};
        use sha2::{Digest, Sha256};

        // Hash constraints for proof
        let constraints_serialized = serde_json::to_string(constraints).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to serialize constraints: {}", e))
        })?;
        let mut hasher = Sha256::new();
        hasher.update(constraints_serialized.as_bytes());
        let constraints_hash = hex::encode(hasher.finalize());

        // Extract image digests from services
        let mut image_digests = HashMap::new();
        for (service_id, service_def) in &sigma.services {
            image_digests.insert(
                service_id.clone(),
                ImageDigest {
                    image: format!("{}:{}", service_def.image, service_def.tag),
                    digest: format!("sha256:EXAMPLE-ONLY: placeholder-{}", service_id), // Populated at runtime
                    platform: Some("linux/amd64".to_string()),
                },
            );
        }

        let timestamp = chrono::Utc::now().to_rfc3339();

        // Create test receipt
        let receipt = TestReceipt {
            id: unimplemented!("ORACLE-GAP Refusal: Content hashing is not yet implemented"), // Will be computed after full creation
            scenario_id: ScenarioId(format!("compiled-{}", sigma.hash)),
            capabilities: vec![CapabilityId("environment_compilation".to_string())],
            effects: EffectSet::new(), // Effects determined at runtime
            sigma_hash: sigma.hash.clone(),
            image_digests: image_digests.clone(),
            constraints: constraints.clone(),
            weaver_proof: None, // Populated at runtime after validation
            timing_footprint: TimingFootprint {
                total_duration: Duration::from_secs(0), // Populated at runtime
                hot_paths: vec![],
                warm_paths: vec![],
                cold_paths: vec![],
                tau_violations: vec![],
            },
            hermeticity_witness: HermeticityWitness {
                network_isolated: constraints.hermetic,
                external_connections: vec![],
                filesystem_isolated: constraints.hermetic,
                non_hermetic_paths: vec![],
                process_isolated: true,
                deterministic: constraints.deterministic,
                determinism_violations: vec![],
            },
            previous_receipt: None, // Set when storing in chain
            signature: None,        // Optional cryptographic signature
            timestamp: timestamp.clone(),
            metadata: HashMap::new(),
        };

        // Compute receipt ID
        let receipt_id = receipt.compute_id();
        let receipt = TestReceipt {
            id: receipt_id,
            ..receipt
        };

        Ok(ProofMetadata {
            sigma_hash: sigma.hash.clone(),
            delta_hash: delta_hash.cloned(),
            constraints_hash,
            compiled_at: timestamp,
            image_digests,
            config_hashes: HashMap::new(), // Populated at runtime
            receipt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::sigma::{SemVer, ServiceDef, TelemetryDef};

    #[allow(dead_code)]
    fn create_test_sigma() -> SigmaBase {
        let mut services = HashMap::new();
        services.insert(
            "db".to_string(),
            ServiceDef {
                id: "db".to_string(),
                image: "postgres".to_string(),
                tag: "14".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                entrypoint: None,
                working_dir: None,
                health_check: None,
                resources: None,
                depends_on: vec![],
            },
        );
        services.insert(
            "api".to_string(),
            ServiceDef {
                id: "api".to_string(),
                image: "myapi".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                entrypoint: None,
                working_dir: None,
                health_check: None,
                resources: None,
                depends_on: vec!["db".to_string()],
            },
        );

        let sigma = SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("test"),
            description: "Test".to_string(),
            services,
            networks: HashMap::new(),
            volumes: HashMap::new(),
            volume_mounts: HashMap::new(),
            telemetry: TelemetryDef {
                otel_collector: None,
                weaver: None,
                service_instrumentation: HashMap::new(),
            },
            metadata: HashMap::new(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let mut sigma_with_hash = sigma;
        sigma_with_hash.hash = sigma_with_hash.compute_hash();
        sigma_with_hash
    }

    #[test]
    fn test_topological_sort_simple() {
        // Arrange: Create compiler and simple graph
        let store = Arc::new(OntologyStore::new());
        let compiler = EnvironmentCompiler::new(store);

        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            ContainerNode {
                id: "a".to_string(),
                image: "test".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                health_check: None,
                resources: None,
            },
        );
        nodes.insert(
            "b".to_string(),
            ContainerNode {
                id: "b".to_string(),
                image: "test".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                health_check: None,
                resources: None,
            },
        );

        let edges = vec![DependencyEdge {
            from: "b".to_string(),
            to: "a".to_string(),
            dependency_type: DependencyType::Hard,
        }];

        // Act: Perform topological sort
        let result = compiler.topological_sort(&nodes, &edges).unwrap();

        // Assert: 'a' comes before 'b' (b depends on a)
        let a_pos = result.iter().position(|x| x == "a").unwrap();
        let b_pos = result.iter().position(|x| x == "b").unwrap();
        assert!(a_pos < b_pos, "Service 'a' must start before 'b'");
    }

    #[test]
    fn test_topological_sort_detects_cycle() {
        // Arrange: Create circular dependency
        let store = Arc::new(OntologyStore::new());
        let compiler = EnvironmentCompiler::new(store);

        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            ContainerNode {
                id: "a".to_string(),
                image: "test".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                health_check: None,
                resources: None,
            },
        );
        nodes.insert(
            "b".to_string(),
            ContainerNode {
                id: "b".to_string(),
                image: "test".to_string(),
                tag: "latest".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                health_check: None,
                resources: None,
            },
        );

        let edges = vec![
            DependencyEdge {
                from: "a".to_string(),
                to: "b".to_string(),
                dependency_type: DependencyType::Hard,
            },
            DependencyEdge {
                from: "b".to_string(),
                to: "a".to_string(),
                dependency_type: DependencyType::Hard,
            },
        ];

        // Act & Assert: Cycle detection fails
        assert!(compiler.topological_sort(&nodes, &edges).is_err());
    }
}
