//! Test binary for noun-verb CLI integration (v5.3.0)
//!
//! This binary uses clap-noun-verb v5.3.0's auto-discovery mechanism.
//! Commands are registered via #[noun] and #[verb] proc macros.

// Force inclusion of noun-verb command modules for linkme discovery
// The modules contain #[distributed_slice] registrations that must be linked
extern crate clnrm_core;

fn main() -> clap_noun_verb::Result<()> {
    // Force the linker to include the noun-verb modules by referencing their types
    // This ensures the linkme distributed slices are not eliminated
    let _ = std::any::TypeId::of::<clnrm_core::cli::commands::collector_noun_verb::CollectorStatusOutput>();
    let _ = std::any::TypeId::of::<clnrm_core::cli::commands::services_noun_verb::ServiceStatusOutput>();

    // v5.3.0 auto-discovers all commands registered via linkme distributed slices
    clap_noun_verb::run()
}
