//! Poka-Yoke (Error-Proofing) Mechanisms
//!
//! This module implements error-proofing mechanisms to prevent the highest-priority
//! failure modes identified in the FMEA audit. These mechanisms make failures
//! impossible or immediately detectable.
//!
//! # Architecture
//!
//! The poka-yoke module follows a layered architecture:
//!
//! ```
//! ┌─────────────────────────────────────────┐
//! │     Trait Abstractions (traits.rs)      │
//! │  CliValidator, TomlValidator, etc.       │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │   Default Implementations (impls.rs)     │
//! │  Concrete validators for production      │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │   Global Instances (globals.rs)         │
//! │  Shared validators across codebase       │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Poka-Yoke Principles
//!
//! 1. **Prevention**: Make errors impossible through design
//! 2. **Detection**: Make errors immediately obvious when they occur
//! 3. **Fail-Fast**: Detect errors as early as possible
//! 4. **Clear Errors**: Provide actionable error messages

pub mod traits;
pub mod impls;
pub mod globals;

#[cfg(test)]
mod tests;

// Re-export for convenience
pub use traits::*;
pub use impls::*;
pub use globals::*;

