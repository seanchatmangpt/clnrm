//! Chaos engineering orchestration module
//!
//! Provides orchestration layer between TOML configuration and ChaosEnginePlugin,
//! mapping declarative chaos experiments to executable scenarios.

pub mod orchestrator;

pub use orchestrator::ChaosOrchestrator;
