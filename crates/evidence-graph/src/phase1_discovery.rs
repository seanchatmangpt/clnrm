//! Phase 1: Repository Discovery and Cataloging
//!
//! Discovers and catalogs repositories, enumerates all files, and classifies them
//! by type, language, and location.

use crate::schemas::{CatalogStats, FileDescriptor, RepoCatalog, RepoDescriptor};
use std::path::Path;
use walkdir::WalkDir;

/// Repository discovery engine
pub struct RepositoryDiscovery {
    /// Root path to scan
    pub root_path: String,
    /// Discovered repositories
    pub repositories: Vec<RepoDescriptor>,
}

impl RepositoryDiscovery {
    /// Create a new discovery engine
    pub fn new(root_path: impl Into<String>) -> Self {
        Self {
            root_path: root_path.into(),
            repositories: Vec::new(),
        }
    }

    /// Discover repositories based on directory structure
    pub fn discover_repositories(&mut self) -> anyhow::Result<()> {
        let root = Path::new(&self.root_path);

        // Look for common repo markers
        let repo_markers = vec!["Cargo.toml", ".git", "package.json", "go.mod"];

        // Scan immediate subdirectories
        if root.exists() && root.is_dir() {
            for entry in std::fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Check if this looks like a repo
                    let dir_name = path.file_name().unwrap().to_string_lossy();

                    // Look for repo markers
                    for marker in &repo_markers {
                        if path.join(marker).exists() {
                            let repo_id = dir_name.to_string();
                            self.repositories.push(RepoDescriptor {
                                repo_id: repo_id.clone(),
                                origin: path.to_string_lossy().to_string(),
                                likely_domains: self.infer_domains(&repo_id),
                                priority: 0.8,
                                discovered_at: chrono::Utc::now()
                                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                            });
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Infer domains based on repo name
    fn infer_domains(&self, repo_id: &str) -> Vec<String> {
        let lower = repo_id.to_lowercase();
        let mut domains = Vec::new();

        if lower.contains("knhk") || lower.contains("knowledge") {
            domains.push("knowledge_graph".to_string());
        }
        if lower.contains("mu-kernel") || lower.contains("kernel") {
            domains.push("timing_kernel".to_string());
        }
        if lower.contains("ctt") || lower.contains("chicago") {
            domains.push("verification".to_string());
        }
        if lower.contains("clnrm") {
            domains.push("testing".to_string());
        }
        if lower.contains("cnv") {
            domains.push("cli_surface".to_string());
        }
        if lower.contains("nomrg") {
            domains.push("graph_overlay".to_string());
        }
        if lower.contains("ggen") {
            domains.push("code_generation".to_string());
        }
        if lower.contains("ahi") || lower.contains("autonomic") {
            domains.push("governance".to_string());
        }

        if domains.is_empty() {
            domains.push("unknown".to_string());
        }

        domains
    }
}

/// File discovery and classification
pub struct FileDiscovery;

impl FileDiscovery {
    /// Enumerate all files in a directory
    pub fn enumerate_files(root_path: &str) -> anyhow::Result<Vec<FileDescriptor>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Skip hidden files and common exclusions
            if should_skip(&relative_path) {
                continue;
            }

            let (kind, language) = classify_file(&relative_path);

            // Count lines
            let line_count = if kind != "binary" {
                std::fs::read_to_string(path)
                    .map(|content| content.lines().count())
                    .unwrap_or(0)
            } else {
                0
            };

            files.push(FileDescriptor {
                repo_id: extract_repo_id(&relative_path),
                path: relative_path,
                kind,
                language,
                line_count,
                is_test: is_test_file(&entry.path()),
            });
        }

        Ok(files)
    }
}

/// Classify a file by kind and language
fn classify_file(path: &str) -> (String, String) {
    let lower = path.to_lowercase();

    // Determine kind
    let kind = if lower.contains("test") || lower.ends_with(".test.rs") {
        "test"
    } else if lower.contains("example") {
        "example"
    } else if lower.contains("doc") || lower.ends_with(".md") {
        "doc"
    } else if lower.contains("src/") || lower.contains("lib/") || lower.contains("crates/") {
        "code"
    } else if is_config_file(&lower) {
        "config"
    } else {
        "other"
    }
    .to_string();

    // Determine language
    let language = if lower.ends_with(".rs") {
        "rust"
    } else if lower.ends_with(".md") {
        "markdown"
    } else if lower.ends_with(".toml") {
        "toml"
    } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
        "yaml"
    } else if lower.ends_with(".json") {
        "json"
    } else if lower.ends_with(".go") {
        "go"
    } else if lower.ends_with(".py") {
        "python"
    } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
        "typescript"
    } else if lower.ends_with(".js") || lower.ends_with(".jsx") {
        "javascript"
    } else if lower.ends_with(".sh") {
        "bash"
    } else {
        "other"
    }
    .to_string();

    (kind, language)
}

/// Check if a file is a configuration file
fn is_config_file(path: &str) -> bool {
    path.contains("cargo.toml")
        || path.contains("package.json")
        || path.contains(".env")
        || path.contains("config")
        || path.contains(".clnrm.toml")
        || path.contains("registry")
}

/// Check if a file should be skipped
fn should_skip(path: &str) -> bool {
    let lower = path.to_lowercase();

    lower.starts_with(".")
        || lower.contains("/.git/")
        || lower.contains("node_modules")
        || lower.contains("target/")
        || lower.contains("/dist/")
        || lower.contains("__pycache__")
        || lower.ends_with(".lock")
        || lower.ends_with(".swp")
        || lower.ends_with(".tmp")
}

/// Check if a file is a test file
fn is_test_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("test") || path_str.contains("spec")
}

/// Extract repository ID from a file path
fn extract_repo_id(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();

    // Look for common patterns
    for (i, part) in parts.iter().enumerate() {
        if *part == "crates" && i + 1 < parts.len() {
            return parts[i + 1].to_string();
        }
    }

    // Otherwise use first component
    parts.first().unwrap_or(&"unknown").to_string()
}

/// Build a complete catalog from a root path
pub fn build_catalog(root_path: &str) -> anyhow::Result<RepoCatalog> {
    // Phase 1: Discover repositories
    let mut discovery = RepositoryDiscovery::new(root_path);
    discovery.discover_repositories()?;

    // Phase 2: Enumerate all files
    let files = FileDiscovery::enumerate_files(root_path)?;

    // Calculate statistics
    let total_code_files = files.iter().filter(|f| f.kind == "code").count();
    let total_doc_files = files.iter().filter(|f| f.kind == "doc").count();
    let total_lines_of_code: usize = files.iter().filter(|f| f.kind == "code").map(|f| f.line_count).sum();

    let stats = CatalogStats {
        total_repos: discovery.repositories.len(),
        total_files: files.len(),
        total_code_files,
        total_doc_files,
        total_lines_of_code,
    };

    Ok(RepoCatalog {
        repositories: discovery.repositories,
        files,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_domains() {
        let discovery = RepositoryDiscovery::new(".");
        let domains = discovery.infer_domains("knhk");
        assert!(domains.contains(&"knowledge_graph".to_string()));
    }

    #[test]
    fn test_classify_rust_code() {
        let (kind, lang) = classify_file("src/main.rs");
        assert_eq!(lang, "rust");
        assert_eq!(kind, "code");
    }

    #[test]
    fn test_classify_markdown_doc() {
        let (kind, lang) = classify_file("docs/README.md");
        assert_eq!(lang, "markdown");
        assert_eq!(kind, "doc");
    }

    #[test]
    fn test_should_skip_git_dir() {
        assert!(should_skip(".git/config"));
        assert!(should_skip(".github/workflows/ci.yml"));
    }

    #[test]
    fn test_extract_repo_id() {
        assert_eq!(extract_repo_id("crates/clnrm/src/main.rs"), "clnrm");
        assert_eq!(extract_repo_id("docs/README.md"), "docs");
    }
}
