//! CLI integration with noun-verb pattern (v5.3.2 compatibility)
//!
//! This module provides integration with clap-noun-verb v5.3.2 for services and collector
//! management commands. Services and collector verbs are defined with proc macros and
//! auto-discovered via linkme distributed slices.

use crate::error::{CleanroomError, Result};

/// Run CLI with noun-verb pattern for services and collector commands (v5.3.2)
///
/// This uses the v5.3.2 API with auto-discovery via linkme. The services and collector
/// nouns should be defined with #[noun] macros and will be automatically registered.
pub fn run_noun_verb_cli() -> Result<()> {
    // v5.3.2 uses auto-discovery via linkme - no manual registration needed
    clap_noun_verb::run()
        .map_err(|e| CleanroomError::internal_error(format!("CLI execution failed: {}", e)))
}
