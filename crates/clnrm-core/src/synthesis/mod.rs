//! Dark-Matter Exploration Mode (Phase 5)
//!
//! Autonomic scenario synthesis for exploring untested capability combinations,
//! service configurations, and hermeticity boundaries.
//!
//! ## Overview
//!
//! The synthesis framework automatically generates test scenarios to fill
//! coverage gaps identified through analysis of historical test receipts.
//!
//! ### Coverage Introspection
//!
//! The `CoverageAnalyzer` examines historical test receipts (Γₜ) to identify:
//!
//! - **Capability Gaps**: Combinations of capabilities never tested together
//! - **Ontology Gaps**: Service configurations never executed
//! - **Hermeticity Gaps**: Isolation boundaries never validated
//!
//! ### Scenario Synthesis
//!
//! The `ScenarioSynthesizer` generates new scenarios to fill identified gaps:
//!
//! - **Coverage Scenarios**: Fill capability/ontology/hermeticity gaps
//! - **Adversarial Scenarios**: Chaos testing variants (network delays, failures, etc.)
//! - **Constraint Solving**: Ensures synthesized scenarios are valid
//!
//! ## Usage
//!
//! ```rust,no_run
//! use clnrm_core::synthesis::{CoverageAnalyzer, ScenarioSynthesizer};
//! use clnrm_core::backend::capabilities::BackendCapabilityRegistry;
//! use clnrm_core::environment::store::OntologyStore;
//! use clnrm_core::receipts::store::ReceiptStore;
//! use std::sync::Arc;
//!
//! # fn example() -> clnrm_core::error::Result<()> {
//! // Set up stores
//! let capabilities = Arc::new(BackendCapabilityRegistry::new());
//! let ontologies = Arc::new(OntologyStore::new());
//! let receipts = Arc::new(ReceiptStore::new());
//!
//! // Create coverage analyzer
//! let analyzer = Arc::new(CoverageAnalyzer::new(
//!     capabilities.clone(),
//!     ontologies.clone(),
//!     receipts.clone(),
//! ));
//!
//! // Find coverage gaps
//! let capability_gaps = analyzer.find_capability_gaps()?;
//! let ontology_gaps = analyzer.find_ontology_gaps()?;
//! let hermeticity_gaps = analyzer.find_hermeticity_gaps()?;
//!
//! // Create synthesizer
//! let synthesizer = ScenarioSynthesizer::new(
//!     analyzer,
//!     capabilities.clone(),
//!     ontologies.clone(),
//! );
//!
//! // Generate scenarios to fill gaps
//! let scenarios = synthesizer.synthesize_all_gaps()?;
//!
//! println!("Generated {} scenarios to fill coverage gaps", scenarios.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Adversarial Testing
//!
//! Generate chaos testing variants of existing scenarios:
//!
//! ```rust,no_run
//! use clnrm_core::synthesis::ScenarioSynthesizer;
//! use clnrm_core::capabilities::CapabilityScenarioBuilder;
//! # use clnrm_core::backend::capabilities::BackendCapabilityRegistry;
//! # use clnrm_core::environment::store::OntologyStore;
//! # use clnrm_core::receipts::store::ReceiptStore;
//! # use clnrm_core::synthesis::CoverageAnalyzer;
//! # use std::sync::Arc;
//!
//! # fn example() -> clnrm_core::error::Result<()> {
//! # let capabilities = Arc::new(BackendCapabilityRegistry::new());
//! # let ontologies = Arc::new(OntologyStore::new());
//! # let receipts = Arc::new(ReceiptStore::new());
//! # let analyzer = Arc::new(CoverageAnalyzer::new(
//! #     capabilities.clone(),
//! #     ontologies.clone(),
//! #     receipts,
//! # ));
//! # let synthesizer = ScenarioSynthesizer::new(
//! #     analyzer,
//! #     capabilities,
//! #     ontologies,
//! # );
//! // Create baseline scenario
//! let baseline = CapabilityScenarioBuilder::new("api_test", "API Test")
//!     .capability("hermetic_execution")
//!     .build();
//!
//! // Generate adversarial variants
//! let variants = synthesizer.synthesize_adversarial(&baseline)?;
//!
//! // Variants include:
//! // - Network delay injection
//! // - Resource exhaustion
//! // - Partial failure scenarios
//! # Ok(())
//! # }
//! ```
//!
//! ## Integration with Chicago TDD
//!
//! The synthesis framework integrates with Chicago TDD by:
//!
//! - Analyzing effect budgets from test receipts
//! - Identifying untested effect combinations
//! - Generating scenarios that exercise new effect patterns
//! - Validating synthesized scenarios against capability registry
//!
//! ## Future Work
//!
//! - **Constraint Solver Integration**: Use SMT solvers for complex scenario generation
//! - **Machine Learning**: Learn from past test results to guide synthesis
//! - **Scenario IR**: Mutable, strongly-typed scenario representation
//! - **TOML Rendering**: Export synthesized scenarios for human review
//! - **AHI Integration**: Closed-loop with A = μ(O) universe

pub mod coverage;
pub mod synthesizer;

// Re-export commonly used types
pub use coverage::{CapabilityGap, CoverageAnalyzer, HermeticityGap, OntologyGap};
pub use synthesizer::{
    AdversarialCondition, ScenarioSynthesizer, SynthesisConfig, SynthesisVariant,
};
