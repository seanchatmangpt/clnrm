//! Test script to verify CLI command fixes work
//! This tests the validate, lint, and dry-run commands directly

use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing CLI command fixes...");

    // Test v2.0.0 format file
    let v2_file = PathBuf::from("examples/advanced-features/env-vars-test.clnrm.toml");

    if !v2_file.exists() {
        println!("❌ Test file missing: {}", v2_file.display());
        return Ok(());
    }

    // Test validate command
    println!("\n1️⃣ Testing validate command...");
    match clnrm_core::cli::commands::validate::validate_config(&v2_file) {
        Ok(_) => println!("✅ validate: PASSED"),
        Err(e) => println!("❌ validate: FAILED - {}", e),
    }

    // Test lint command
    println!("\n2️⃣ Testing lint command...");
    match clnrm_core::cli::commands::lint::lint_files(vec![&v2_file], "human", false) {
        Ok(_) => println!("✅ lint: PASSED"),
        Err(e) => println!("❌ lint: FAILED - {}", e),
    }

    // Test dry-run command
    println!("\n3️⃣ Testing dry-run command...");
    match clnrm_core::cli::commands::dry_run::dry_run_validate(vec![&v2_file], false) {
        Ok(results) => {
            let passed = results.iter().all(|r| r.valid);
            if passed {
                println!("✅ dry-run: PASSED");
            } else {
                println!("❌ dry-run: FAILED - {} errors", results.iter().map(|r| r.error_count).sum::<usize>());
            }
        }
        Err(e) => println!("❌ dry-run: FAILED - {}", e),
    }

    // Test v1.x format rejection
    println!("\n4️⃣ Testing v1.x format rejection...");
    let v1_file = PathBuf::from("examples/behaviors.clnrm.toml");

    if v1_file.exists() {
        match clnrm_core::cli::commands::validate::validate_config(&v1_file) {
            Ok(_) => println!("❌ v1.x rejection: FAILED - should have rejected v1.x format"),
            Err(e) => {
                if e.to_string().contains("v2.0.0 format") {
                    println!("✅ v1.x rejection: PASSED - correctly rejected v1.x format");
                } else {
                    println!("❌ v1.x rejection: FAILED - wrong error: {}", e);
                }
            }
        }
    } else {
        println!("⚠️ v1.x test file not found");
    }

    println!("\n🎉 CLI fixes test completed!");
    Ok(())
}
