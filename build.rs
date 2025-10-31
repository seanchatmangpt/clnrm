// Build script for clnrm
// Regenerates telemetry code from semantic convention schemas

use std::process::Command;
use std::path::Path;

fn main() {
    // Only regenerate if weaver is available and schemas exist
    let registry_path = Path::new("registry");
    let templates_path = Path::new("templates/registry/rust");

    if !registry_path.exists() || !templates_path.exists() {
        println!("cargo:warning=Skipping code generation: registry or templates not found");
        return;
    }

    // Check if weaver is installed
    let weaver_check = Command::new("weaver")
        .arg("--version")
        .output();

    match weaver_check {
        Ok(output) if output.status.success() => {
            println!("cargo:rerun-if-changed=registry/");
            println!("cargo:rerun-if-changed=templates/registry/rust/");

            // Generate telemetry code
            let status = Command::new("weaver")
                .args(&[
                    "registry",
                    "generate",
                    "rust",
                    "--registry",
                    "registry/",
                    "--templates",
                    "templates/registry/rust/",
                    "--output",
                    "crates/clnrm-core/src/telemetry/",
                ])
                .status();

            match status {
                Ok(exit_status) if exit_status.success() => {
                    println!("cargo:warning=Successfully generated telemetry code");
                }
                Ok(exit_status) => {
                    println!("cargo:warning=Weaver generation failed with status: {}", exit_status);
                }
                Err(e) => {
                    println!("cargo:warning=Failed to execute weaver: {}", e);
                }
            }
        }
        _ => {
            println!("cargo:warning=Weaver not found. Install with: cargo install weaver-cli");
            println!("cargo:warning=Skipping telemetry code generation");
        }
    }
}
