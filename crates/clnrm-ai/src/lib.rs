//! # Cleanroom AI - Autonomic Intelligence Module
//!
//! This crate contains AI and autonomic intelligence features for the
//! Cleanroom Testing Framework, including:
//!
//! - AI-powered test orchestration
//! - Predictive failure analysis
//! - Intelligent optimization
//! - Real-time monitoring with anomaly detection
//!
//! ## Features
//!
//! This is an **experimental** module that provides AI capabilities through
//! Ollama integration. All features gracefully degrade to rule-based fallbacks
//! when AI services are unavailable.

pub mod commands;
pub mod services;

// Re-export main types
pub use commands::{
    ai_monitor, ai_optimize, ai_orchestrate, ai_predict, ai_real,
};
pub use services::{
    ai_intelligence::AIIntelligenceService,
    ai_test_generator::{AITestGeneratorConfig, AITestGeneratorPlugin},
};
