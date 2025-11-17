//! Evidence Graph Mining Pipeline
//!
//! A comprehensive agent-driven pipeline for mining evidence from source code and documentation
//! that supports claims about the graph-universe thesis and its organ systems (KNHK, μ-kernel,
//! CTT, clnrm, CNV, nomrg, DFLSS, AHI).
//!
//! # Overview
//!
//! The pipeline has 7 phases:
//!
//! 1. **Repository Discovery**: Catalog repositories, enumerate files, classify by type/language
//! 2. **File Classification**: Categorize files (code, doc, config, test, example)
//! 3. **Token Extraction**: Build TokenIndex with frequencies and positions
//! 4. **Concept Matching**: Apply rule-based scoring against 15 concepts
//! 5. **Excerpt Extraction**: Identify relevant line ranges, score locally
//! 6. **Evidence Synthesis**: Generate EvidenceNodes, deduplicate clusters
//! 7. **Graph Construction**: Build nodes/edges, infer relationships, generate outputs
//!
//! # Concepts
//!
//! The pipeline tracks **15 core concepts** across **7 domains**:
//!
//! - **Universe** (3): C_GRAPH_UNIVERSE_PRIMARY, C_CODE_AS_PROJECTION, C_RECEIPTS_AND_PROOFS
//! - **Timing** (2): C_MU_KERNEL_PHYSICS, C_TIMING_BOUNDS_ENFORCED
//! - **Knowledge** (3): C_KNHK_GRAPH_PRIMARY, C_DFLSS_FLOW, C_AHI_GOVERNANCE
//! - **Verification** (2): C_CTT_12_PHASE_VERIFICATION, C_CLNRM_HERMETIC_TESTING
//! - **Surface** (3): C_CNV_AGENT_CLI, C_NOMRG_GRAPH_OVERLAY, C_GGEN_PROJECTION_ENGINE
//!
//! # Outputs
//!
//! Three artifacts are generated:
//!
//! - `evidence_graph.json`: Complete graph with nodes (concept, evidence, system) and edges
//! - `concept_coverage.json`: Per-concept statistics with evidence counts and strength ranges
//! - `concept_gaps.json`: Concepts with insufficient or missing evidence
//!
//! # Usage
//!
//! ```ignore
//! use evidence_graph::{Pipeline, PipelineConfig};
//!
//! let config = PipelineConfig::default();
//! let mut pipeline = Pipeline::new(config);
//! pipeline.run("/home/user/clnrm").await?;
//!
//! let graph = pipeline.evidence_graph();
//! let coverage = pipeline.coverage_report();
//! let gaps = pipeline.gaps_report();
//! ```

pub mod concepts;
pub mod outputs;
pub mod schemas;
pub mod phase1_discovery;
pub mod phase3_tokenization;
pub mod phase4_matching;
pub mod phase5_excerpts;
pub mod phase6_synthesis;
pub mod phase7_graphconstruction;
pub mod orchestrator;

pub use concepts::ConceptRegistry;
pub use outputs::*;
pub use schemas::*;
pub use orchestrator::{Pipeline, PipelineConfig};
