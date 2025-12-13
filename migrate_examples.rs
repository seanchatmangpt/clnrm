//! Migrate all v1.x examples to v2.0.0 format

use std::fs;
use std::path::Path;
use std::collections::HashMap;

fn migrate_toml_content(content: &str) -> String {
    let mut lines = content.lines().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Convert [test.metadata] to [test]
        if line == "[test.metadata]" {
            lines[i] = "[test]".to_string();
        }

        // Convert [services.X] to [containers.X]
        else if line.starts_with("[services.") && line.ends_with("]") {
            let container_name = &line[9..line.len()-1]; // Remove [services. and ]
            lines[i] = format!("[containers.{}]", container_name);
        }

        // Remove type = "generic_container" lines
        else if line.contains("type = \"generic_container\"") {
            lines.remove(i);
            continue; // Don't increment i since we removed an element
        }

        // Convert command = [...] to exec = [...]
        else if line.starts_with("command = ") {
            lines[i] = line.replace("command = ", "exec = ");
        }

        // Convert expected_output_regex to assert.stdout_contains
        else if line.starts_with("expected_output_regex = ") {
            let value = line.split("= ").nth(1).unwrap_or("");
            lines[i] = format!("assert.stdout_contains = {}", value);
        }

        // Add container = "name" to [[steps]] sections
        else if line == "[[steps]]" {
            // Look ahead to find the container name from previous [containers.X] section
            let mut container_name = String::new();
            let mut j = i;
            while j > 0 {
                j -= 1;
                let prev_line = lines[j].trim();
                if prev_line.starts_with("[containers.") && prev_line.ends_with("]") {
                    container_name = prev_line[12..prev_line.len()-1].to_string(); // Remove [containers. and ]
                    break;
                }
            }

            // Insert container field after [[steps]]
            if !container_name.is_empty() {
                i += 1; // Move past [[steps]]
                lines.insert(i, format!("container = \"{}\"", container_name));
            }
        }

        i += 1;
    }

    lines.join("\n")
}

fn migrate_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Migrating: {}", path.display());

    let content = fs::read_to_string(path)?;
    let migrated = migrate_toml_content(&content);

    // Only write if content actually changed
    if migrated != content {
        fs::write(path, migrated)?;
        println!("✅ Migrated: {}", path.display());
    } else {
        println!("⚪ No changes needed: {}", path.display());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting example migration from v1.x to v2.0.0 format...");

    // Find all .clnrm.toml files in examples/
    let examples_dir = Path::new("examples");
    let mut migrated_count = 0;
    let mut total_count = 0;

    for entry in walkdir::WalkDir::new(examples_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") &&
           path.to_string_lossy().contains(".clnrm.toml") {

            total_count += 1;
            if let Err(e) = migrate_file(path) {
                eprintln!("❌ Failed to migrate {}: {}", path.display(), e);
            } else {
                migrated_count += 1;
            }
        }
    }

    println!("\n🎉 Migration complete!");
    println!("📊 Migrated: {}/{} files", migrated_count, total_count);

    Ok(())
}
