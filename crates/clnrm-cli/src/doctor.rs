use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct DoctorReport {
    pub status: String,
    pub pathologies: Vec<Pathology>,
}

#[derive(Serialize)]
pub struct Pathology {
    pub name: String,
    pub severity: String,
    pub description: String,
    pub repair_mode: String,
}

impl DoctorReport {
    pub fn new() -> Self {
        Self {
            status: "Healthy".to_string(),
            pathologies: Vec::new(),
        }
    }

    pub fn add_pathology(
        &mut self,
        name: &str,
        severity: &str,
        description: &str,
        repair_mode: &str,
    ) {
        self.pathologies.push(Pathology {
            name: name.to_string(),
            severity: severity.to_string(),
            description: description.to_string(),
            repair_mode: repair_mode.to_string(),
        });

        if severity == "Critical" {
            self.status = "Unhealthy".to_string();
        }
    }
}

pub fn run_diagnostics() -> DoctorReport {
    let mut report = DoctorReport::new();

    // 1. Environment State: Build
    let build_output = Command::new("cargo")
        .arg("check")
        .arg("--workspace")
        .output();

    match build_output {
        Ok(output) if !output.status.success() => {
            report.add_pathology(
                "EnvironmentBuildPathology",
                "Critical",
                "Workspace fails to compile.",
                "Manual: Check cargo check output and fix Rust compilation errors.",
            );
        }
        Err(e) => {
            report.add_pathology(
                "EnvironmentBuildPathology",
                "Critical",
                &format!("Failed to execute cargo: {}", e),
                "Manual: Check cargo installation.",
            );
        }
        _ => {}
    }

    // 2. Environment State: Tests
    let test_output = Command::new("cargo")
        .arg("test")
        .arg("--workspace")
        .arg("--lib")
        .arg("--tests")
        .output();

    match test_output {
        Ok(output) if !output.status.success() => {
            report.add_pathology(
                "TimingTruthPathology",
                "Critical",
                "Test suite is failing, compromising reproducibility truth.",
                "Manual: Run cargo test and resolve failing invariants.",
            );
        }
        Err(e) => {
            report.add_pathology(
                "TimingTruthPathology",
                "Critical",
                &format!("Failed to execute cargo test: {}", e),
                "Manual: Check cargo installation.",
            );
        }
        _ => {}
    }

    // 3. Deployability Truth: Publish
    let publish_output = Command::new("cargo")
        .arg("publish")
        .arg("--dry-run")
        .arg("--allow-dirty")
        .arg("-p")
        .arg("clnrm-core")
        .output();

    match publish_output {
        Ok(output) if !output.status.success() => {
            report.add_pathology(
                "DeployabilityTruthPathology",
                "Critical",
                "Crate fails publishability dry-run.",
                "Manual: Resolve Cargo.toml dependency issues or uncommitted changes.",
            );
        }
        Err(e) => {
            report.add_pathology(
                "DeployabilityTruthPathology",
                "Critical",
                &format!("Failed to execute cargo publish: {}", e),
                "Manual: Check cargo installation.",
            );
        }
        _ => {}
    }

    // 4. Anti-Lie Truth: Source Code Analysis
    let crates_dir = Path::new("crates");
    let mut legacy_calls = 0;
    let mut testcontainers_refs = 0;
    let mut migration_todos = 0;
    let mut unwrap_calls = 0;

    if crates_dir.exists() {
        for entry in WalkDir::new(crates_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                // Skip doctor.rs to avoid false positives from the check itself
                if entry.path().to_string_lossy().contains("doctor.rs") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines() {
                        if line.contains("Command::new(\"docker\")") {
                            legacy_calls += 1;
                        }
                        if line.contains("testcontainers") && !line.trim().starts_with("//") {
                            testcontainers_refs += 1;
                        }
                        if line.contains("GapMarker:")
                            || line.contains("GapMarker ")
                            || line.contains("todo!")
                            || line.contains("unimplemented!")
                            || line.contains("ORACLE-GAP Refusal")
                        {
                            migration_todos += 1;
                        }
                        if line.contains(".unwrap()")
                            && !entry.path().to_string_lossy().contains("tests")
                            && !line.trim().starts_with("//")
                        {
                            unwrap_calls += 1;
                        }
                    }
                }
            }
        }
    }

    if legacy_calls > 0 {
        report.add_pathology(
            "LegacyDependencyPathology",
            "Critical",
            &format!(
                "Found {} legacy Docker CLI calls violating v3 hermetic isolation mandate.",
                legacy_calls
            ),
            "Manual: Migrate logic to gVisor/runsc executor.",
        );
    }

    if testcontainers_refs > 0 {
        report.add_pathology(
            "AntiLieTruthPathology",
            "Critical",
            &format!(
                "Found {} lingering references to deprecated 'testcontainers' library.",
                testcontainers_refs
            ),
            "Manual: Purge remaining testcontainers code paths.",
        );
    }

    if migration_todos > 0 {
        report.add_pathology(
            "OracleGapPathology",
            "Critical",
            &format!("Found {} pending Oracle Gaps (GapMarkers/unimplemented/refusals). System is incomplete.", migration_todos),
            "Manual: Implement the missing logic defined in the Oracle Gaps.",
        );
    }

    if unwrap_calls > 0 {
        report.add_pathology(
            "LatentInstabilityPathology",
            "Warning",
            &format!(
                "Found {} unsafe unwraps in core library code.",
                unwrap_calls
            ),
            "Auto/Manual: Convert unwraps to structured CleanroomError propagation.",
        );
    }

    // Print human-readable report if not structured output
    println!("🏥 CLNRM Doctor - Epistemic Diagnostics 🏥\n");
    println!("Status: {}\n", report.status);

    if report.pathologies.is_empty() {
        println!("✅ System is fully lawful, truthful, and deployable.");
    } else {
        for p in &report.pathologies {
            println!("🔴 [{}] {}", p.severity, p.name);
            println!("   Reason: {}", p.description);
            println!("   Repair: {}\n", p.repair_mode);
        }
    }

    // We can fail the process if critical
    if report.status == "Unhealthy" {
        std::process::exit(1);
    }

    report
}
