//! System diagnostic and status commands

#![allow(clippy::unused_unit)]

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Run system health check
#[verb("doctor")]
pub fn doctor() -> Result<crate::doctor::DoctorReport> {
    Ok(crate::doctor::run_diagnostics())
}
