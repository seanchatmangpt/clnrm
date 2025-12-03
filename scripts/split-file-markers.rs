//! Post-processor to split rendered template with file markers into multiple files
//! This works around ggen not having template generate command yet

use std::fs;
use std::path::PathBuf;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rendered_file = std::env::args()
        .nth(1)
        .ok_or_else(|| "Usage: split-file-markers <rendered_template> <output_dir>")?;
    let output_dir = std::env::args()
        .nth(2)
        .ok_or_else(|| "Usage: split-file-markers <rendered_template> <output_dir>")?;

    println!("📖 Reading rendered template: {}", rendered_file);
    let content = fs::read_to_string(&rendered_file)?;

    println!("🔍 Splitting by file markers...");
    let file_marker_re = Regex::new(r"\{#\s*FILE:\s*([^\s#]+)\s*#\}")?;

    let mut current_file: Option<PathBuf> = None;
    let mut current_content = String::new();
    let mut files_created = 0;

    for line in content.lines() {
        if let Some(captures) = file_marker_re.captures(line) {
            // Write previous file if exists
            if let Some(path) = current_file.take() {
                let full_path = PathBuf::from(&output_dir).join(&path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&full_path, current_content.trim_end())?;
                println!("   ✅ {}", path.display());
                files_created += 1;
            }
            
            // Start new file
            let file_path = captures.get(1).unwrap().as_str();
            current_file = Some(PathBuf::from(file_path));
            current_content.clear();
        } else if current_file.is_some() {
            // Accumulate content for current file
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Write last file
    if let Some(path) = current_file {
        let full_path = PathBuf::from(&output_dir).join(&path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, current_content.trim_end())?;
        println!("   ✅ {}", path.display());
        files_created += 1;
    }

    println!("✨ Created {} files", files_created);
    Ok(())
}

