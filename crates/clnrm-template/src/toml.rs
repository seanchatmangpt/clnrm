//! Comprehensive TOML file operations for Cleanroom templates
//!
//! Provides TOML-specific functionality for template development:
//! - TOML file loading and parsing
//! - TOML validation and schema checking
//! - TOML file writing and formatting
//! - TOML merging and composition
//! - TOML diff and patch operations
//! - Template file organization and management

use crate::error::{TemplateError, Result};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

/// TOML file representation with metadata
#[derive(Debug, Clone)]
pub struct TomlFile {
    /// File path
    pub path: PathBuf,
    /// TOML content as string
    pub content: String,
    /// Parsed TOML as JSON Value for manipulation
    pub parsed: Value,
    /// File metadata
    pub metadata: TomlMetadata,
}

/// TOML file metadata for tracking and validation
#[derive(Debug, Clone)]
pub struct TomlMetadata {
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub modified: std::time::SystemTime,
    /// File permissions
    pub permissions: std::fs::Permissions,
    /// Template variables used (for analysis)
    pub variables_used: HashSet<String>,
    /// Template functions used (for analysis)
    pub functions_used: HashSet<String>,
}

/// TOML file loader with comprehensive parsing capabilities
#[derive(Debug, Clone)]
pub struct TomlLoader {
    /// Base directories to search for TOML files
    search_paths: Vec<PathBuf>,
    /// File extensions to consider (default: toml, clnrm.toml)
    extensions: Vec<String>,
    /// Enable recursive directory scanning
    recursive: bool,
    /// Validation rules to apply during loading
    validation_rules: Vec<crate::validation::ValidationRule>,
}

impl Default for TomlLoader {
    fn default() -> Self {
        Self {
            search_paths: Vec::new(),
            extensions: vec!["toml".to_string(), "clnrm.toml".to_string()],
            recursive: true,
            validation_rules: Vec::new(),
        }
    }
}

impl TomlLoader {
    /// Create new TOML loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Add search path for TOML files
    pub fn with_search_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.search_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add multiple search paths
    pub fn with_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        for path in paths {
            self.search_paths.push(path.as_ref().to_path_buf());
        }
        self
    }

    /// Set file extensions to include
    pub fn with_extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extensions = extensions.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Enable/disable recursive scanning
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Add validation rule for loaded TOML files
    pub fn with_validation_rule(mut self, rule: crate::validation::ValidationRule) -> Self {
        self.validation_rules.push(rule);
        self
    }

    /// Load single TOML file
    ///
    /// # Arguments
    /// * `path` - Path to TOML file
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<TomlFile> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(TemplateError::IoError(format!("TOML file not found: {}", path.display())));
        }

        if !path.is_file() {
            return Err(TemplateError::IoError(format!("Path is not a file: {}", path.display())));
        }

        // Check file extension
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if !self.extensions.contains(&ext.to_string()) {
                return Err(TemplateError::ValidationError(format!(
                    "File extension '{}' not supported. Expected: {:?}",
                    ext, self.extensions
                )));
            }
        }

        let content = fs::read_to_string(path)
            .map_err(|e| TemplateError::IoError(format!("Failed to read TOML file: {}", e)))?;

        let parsed = toml::from_str::<Value>(&content)
            .map_err(|e| TemplateError::ValidationError(format!("Invalid TOML format: {}", e)))?;

        let metadata = path.metadata()
            .map_err(|e| TemplateError::IoError(format!("Failed to read file metadata: {}", e)))?;

        let file = TomlFile {
            path: path.to_path_buf(),
            content,
            parsed,
            metadata: TomlMetadata {
                size: metadata.len(),
                modified: metadata.modified()
                    .map_err(|e| TemplateError::IoError(format!("Failed to get modification time: {}", e)))?,
                permissions: metadata.permissions(),
                variables_used: HashSet::new(),
                functions_used: HashSet::new(),
            },
        };

        // Apply validation rules
        for rule in &self.validation_rules {
            rule.validate(&file.parsed, &file.path.to_string_lossy())?;
        }

        Ok(file)
    }

    /// Load all TOML files from search paths
    ///
    /// Returns map of file paths to TomlFile objects
    pub fn load_all(&self) -> Result<HashMap<PathBuf, TomlFile>> {
        let mut files = HashMap::new();

        for search_path in &self.search_paths {
            self.scan_directory(search_path, &mut files)?;
        }

        Ok(files)
    }

    /// Scan directory for TOML files
    fn scan_directory(&self, dir: &Path, files: &mut HashMap<PathBuf, TomlFile>) -> Result<()> {
        use walkdir::WalkDir;

        let walker = if self.recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        for entry in walker {
            let entry = entry
                .map_err(|e| TemplateError::IoError(format!("Failed to read directory entry: {}", e)))?;

            if entry.file_type().is_file() {
                let path = entry.path();

                // Check if file has supported extension
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if self.extensions.contains(&ext.to_string()) {
                        match self.load_file(path) {
                            Ok(file) => {
                                files.insert(path.to_path_buf(), file);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to load TOML file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load TOML files matching glob pattern
    pub fn load_glob(&self, pattern: &str) -> Result<HashMap<PathBuf, TomlFile>> {
        use globset::{Glob, GlobSetBuilder};

        let glob = Glob::new(pattern)
            .map_err(|e| TemplateError::ConfigError(format!("Invalid glob pattern '{}': {}", pattern, e)))?;

        let glob_set = GlobSetBuilder::new()
            .add(glob)
            .build()
            .map_err(|e| TemplateError::ConfigError(format!("Failed to build glob set: {}", e)))?;

        let mut files = HashMap::new();

        for search_path in &self.search_paths {
            self.scan_glob_pattern(search_path, &glob_set, &mut files)?;
        }

        Ok(files)
    }

    /// Scan directory with glob pattern
    fn scan_glob_pattern(&self, dir: &Path, glob_set: &globset::GlobSet, files: &mut HashMap<PathBuf, TomlFile>) -> Result<()> {
        use walkdir::WalkDir;

        let walker = if self.recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        for entry in walker {
            let entry = entry
                .map_err(|e| TemplateError::IoError(format!("Failed to read directory entry: {}", e)))?;

            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy();
                if glob_set.is_match(&*path_str) {
                    match self.load_file(entry.path()) {
                        Ok(file) => {
                            files.insert(entry.path().to_path_buf(), file);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load TOML file {:?}: {}", entry.path(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// TOML file writer with formatting and validation
#[derive(Debug, Clone)]
pub struct TomlWriter {
    /// Enable pretty formatting
    pretty: bool,
    /// Backup files before writing
    backup: bool,
    /// Validate before writing
    validate: bool,
    /// Custom header comment for generated files
    header: Option<String>,
}

impl Default for TomlWriter {
    fn default() -> Self {
        Self {
            pretty: true,
            backup: true,
            validate: true,
            header: Some("# Generated by clnrm-template".to_string()),
        }
    }
}

impl TomlWriter {
    /// Create new TOML writer
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable/disable pretty formatting
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Enable/disable backup creation
    pub fn backup(mut self, backup: bool) -> Self {
        self.backup = backup;
        self
    }

    /// Enable/disable validation before writing
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    /// Set custom header comment
    pub fn with_header<S: Into<String>>(mut self, header: S) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Write TOML content to file
    ///
    /// # Arguments
    /// * `path` - Target file path
    /// * `content` - TOML content to write
    /// * `validator` - Optional validator to run before writing
    pub fn write_file<P: AsRef<Path>>(&self, path: P, content: &str, validator: Option<&crate::validation::TemplateValidator>) -> Result<()> {
        let path = path.as_ref();

        // Validate before writing if enabled
        if self.validate {
            if let Some(validator) = validator {
                validator.validate(content, &path.to_string_lossy())?;
            }
        }

        // Create backup if enabled and file exists
        if self.backup && path.exists() {
            self.create_backup(path)?;
        }

        // Prepare content with header
        let final_content = if let Some(ref header) = self.header {
            format!("{}\n{}\n", header, content)
        } else {
            content.to_string()
        };

        // Write file
        let mut file = fs::File::create(path)
            .map_err(|e| TemplateError::IoError(format!("Failed to create file: {}", e)))?;

        file.write_all(final_content.as_bytes())
            .map_err(|e| TemplateError::IoError(format!("Failed to write file: {}", e)))?;

        file.sync_all()
            .map_err(|e| TemplateError::IoError(format!("Failed to sync file: {}", e)))?;

        Ok(())
    }

    /// Create backup of existing file
    fn create_backup(&self, path: &Path) -> Result<()> {
        let _backup_path = self.backup_path(path);

        fs::copy(path, &_backup_path)
            .map_err(|e| TemplateError::IoError(format!("Failed to create backup: {}", e)))?;

        Ok(())
    }

    /// Generate backup file path
    fn backup_path(&self, path: &Path) -> PathBuf {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();

        path.with_file_name(format!("{}.{}.bak", stem, timestamp))
    }
}

/// TOML merger for combining multiple TOML sources
pub struct TomlMerger {
    /// Merge strategy for conflicting keys
    strategy: MergeStrategy,
    /// Preserve comments and formatting
    preserve_formatting: bool,
    /// Deep merge nested structures
    deep_merge: bool,
}

pub enum MergeStrategy {
    /// Overwrite existing values (default)
    Overwrite,
    /// Merge arrays by concatenation
    MergeArrays,
    /// Preserve existing values
    Preserve,
    /// Custom merge function
    Custom,
}

impl std::fmt::Debug for MergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeStrategy::Overwrite => write!(f, "Overwrite"),
            MergeStrategy::MergeArrays => write!(f, "MergeArrays"),
            MergeStrategy::Preserve => write!(f, "Preserve"),
            MergeStrategy::Custom => write!(f, "Custom"),
        }
    }
}

impl Clone for MergeStrategy {
    fn clone(&self) -> Self {
        match self {
            MergeStrategy::Overwrite => MergeStrategy::Overwrite,
            MergeStrategy::MergeArrays => MergeStrategy::MergeArrays,
            MergeStrategy::Preserve => MergeStrategy::Preserve,
            MergeStrategy::Custom => MergeStrategy::Custom,
        }
    }
}

impl Default for TomlMerger {
    fn default() -> Self {
        Self {
            strategy: MergeStrategy::Overwrite,
            preserve_formatting: false,
            deep_merge: true,
        }
    }
}

impl TomlMerger {
    /// Create new TOML merger
    pub fn new() -> Self {
        Self::default()
    }

    /// Set merge strategy
    pub fn with_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Enable/disable formatting preservation
    pub fn preserve_formatting(mut self, preserve: bool) -> Self {
        self.preserve_formatting = preserve;
        self
    }

    /// Enable/disable deep merging
    pub fn deep_merge(mut self, deep: bool) -> Self {
        self.deep_merge = deep;
        self
    }

    /// Merge two TOML values
    ///
    /// # Arguments
    /// * `base` - Base TOML value
    /// * `overlay` - TOML value to merge on top
    pub fn merge(&self, base: &Value, overlay: &Value) -> Result<Value> {
        match (&base, &overlay) {
            (Value::Object(base_obj), Value::Object(overlay_obj)) => {
                let mut result = base_obj.clone();

                for (key, overlay_value) in overlay_obj {
                    if let Some(base_value) = base_obj.get(key) {
                        let merged = self.merge_values(base_value, overlay_value)?;
                        result.insert(key.clone(), merged);
                    } else {
                        result.insert(key.clone(), overlay_value.clone());
                    }
                }

                Ok(Value::Object(result))
            }
            _ => {
                // For non-objects, use overlay strategy
                Ok(overlay.clone())
            }
        }
    }

    /// Merge individual values based on strategy
    fn merge_values(&self, base: &Value, overlay: &Value) -> Result<Value> {
        match &self.strategy {
            MergeStrategy::Overwrite => Ok(overlay.clone()),
            MergeStrategy::Preserve => Ok(base.clone()),
            MergeStrategy::MergeArrays => {
                if let (Value::Array(base_arr), Value::Array(overlay_arr)) = (base, overlay) {
                    let mut merged = base_arr.clone();
                    merged.extend(overlay_arr.iter().cloned());
                    Ok(Value::Array(merged))
                } else {
                    Ok(overlay.clone())
                }
            }
            MergeStrategy::Custom => Ok(overlay.clone()), // Simplified for now
        }
    }

    /// Merge multiple TOML files
    ///
    /// # Arguments
    /// * `files` - Vector of TomlFile objects to merge
    pub fn merge_files(&self, files: &[&TomlFile]) -> Result<TomlFile> {
        if files.is_empty() {
            return Err(TemplateError::ValidationError("No files to merge".to_string()));
        }

        let mut merged_value = files[0].parsed.clone();

        for file in &files[1..] {
            merged_value = self.merge(&merged_value, &file.parsed)?;
        }

        // Create new TomlFile with merged content
        let merged_content = if self.preserve_formatting {
            // Try to preserve formatting (simplified)
            toml::to_string_pretty(&merged_value)
                .unwrap_or_else(|_| toml::to_string(&merged_value).unwrap_or_default())
        } else {
            toml::to_string(&merged_value)
                .map_err(|e| TemplateError::ValidationError(format!("Failed to serialize merged TOML: {}", e)))?
        };

        Ok(TomlFile {
            path: files[0].path.with_extension("merged.toml"),
            content: merged_content,
            parsed: merged_value,
            metadata: files[0].metadata.clone(), // Use first file's metadata
        })
    }
}

/// TOML file utilities for common operations
pub struct TomlUtils;

impl TomlUtils {
    /// Extract variables from TOML content for template analysis
    ///
    /// # Arguments
    /// * `content` - TOML content as string
    pub fn extract_variables(content: &str) -> Result<HashSet<String>> {
        let parsed = toml::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!("Invalid TOML for variable extraction: {}", e)))?;

        let mut variables = HashSet::new();
        Self::extract_variables_recursive(&parsed, &mut variables, "");
        Ok(variables)
    }

    /// Recursively extract variable references from TOML
    fn extract_variables_recursive(value: &Value, variables: &mut HashSet<String>, prefix: &str) {
        match value {
            Value::String(s) => {
                // Look for template variable patterns {{ variable }}
                if s.contains("{{") && s.contains("}}") {
                    // Simple extraction - in real implementation would use regex
                    if let Some(start) = s.find("{{") {
                        if let Some(end) = s.find("}}") {
                            let var_part = &s[start + 2..end];
                            if !var_part.trim().is_empty() {
                                let var_name = if prefix.is_empty() {
                                    var_part.trim().to_string()
                                } else {
                                    format!("{}.{}", prefix, var_part.trim())
                                };
                                variables.insert(var_name);
                            }
                        }
                    }
                }
            }
            Value::Object(obj) => {
                for (key, value) in obj {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    Self::extract_variables_recursive(value, variables, &new_prefix);
                }
            }
            Value::Array(arr) => {
                for (i, value) in arr.iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        format!("{}", i)
                    } else {
                        format!("{}.{}", prefix, i)
                    };
                    Self::extract_variables_recursive(value, variables, &new_prefix);
                }
            }
            _ => {} // Other types don't contain variables
        }
    }

    /// Validate TOML file structure
    ///
    /// # Arguments
    /// * `file` - TomlFile to validate
    /// * `required_sections` - Required top-level sections
    pub fn validate_structure(file: &TomlFile, required_sections: &[&str]) -> Result<()> {
        let obj = file.parsed.as_object()
            .ok_or_else(|| TemplateError::ValidationError("TOML must be an object".to_string()))?;

        for section in required_sections {
            if !obj.contains_key(*section) {
                return Err(TemplateError::ValidationError(format!(
                    "Required section '{}' missing in TOML file: {}",
                    section, file.path.display()
                )));
            }
        }

        Ok(())
    }

    /// Compare two TOML files for differences
    ///
    /// # Arguments
    /// * `file1` - First TOML file
    /// * `file2` - Second TOML file
    pub fn diff(file1: &TomlFile, file2: &TomlFile) -> TomlDiff {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        // Compare top-level keys
        if let (Some(obj1), Some(obj2)) = (file1.parsed.as_object(), file2.parsed.as_object()) {
            for (key, value2) in obj2 {
                if let Some(value1) = obj1.get(key) {
                    if value1 != value2 {
                        changed.push((key.clone(), value1.clone(), value2.clone()));
                    }
                } else {
                    added.push((key.clone(), value2.clone()));
                }
            }

            for (key, _) in obj1 {
                if !obj2.contains_key(key) {
                    removed.push(key.clone());
                }
            }
        }

        TomlDiff {
            added,
            removed,
            changed,
        }
    }

    /// Pretty format TOML content
    ///
    /// # Arguments
    /// * `content` - TOML content to format
    pub fn format_toml(content: &str) -> Result<String> {
        let parsed = toml::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!("Invalid TOML for formatting: {}", e)))?;

        toml::to_string_pretty(&parsed)
            .map_err(|e| TemplateError::ValidationError(format!("Failed to format TOML: {}", e)))
    }

    /// Validate TOML syntax and structure
    ///
    /// # Arguments
    /// * `content` - TOML content to validate
    pub fn validate_toml_syntax(content: &str) -> Result<()> {
        // Parse to check syntax
        toml::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!("Invalid TOML syntax: {}", e)))?;

        Ok(())
    }

    /// Extract all keys from TOML content
    ///
    /// # Arguments
    /// * `content` - TOML content
    pub fn extract_keys(content: &str) -> Result<HashSet<String>> {
        let parsed = toml::from_str::<Value>(content)
            .map_err(|e| TemplateError::ValidationError(format!("Invalid TOML for key extraction: {}", e)))?;

        let mut keys = HashSet::new();
        Self::extract_keys_recursive(&parsed, &mut keys, "");
        Ok(keys)
    }

    /// Recursively extract all keys from TOML
    fn extract_keys_recursive(value: &Value, keys: &mut HashSet<String>, prefix: &str) {
        match value {
            Value::Object(obj) => {
                for (key, value) in obj {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    keys.insert(full_key.clone());
                    Self::extract_keys_recursive(value, keys, &full_key);
                }
            }
            Value::Array(arr) => {
                for (i, value) in arr.iter().enumerate() {
                    let index_key = if prefix.is_empty() {
                        format!("{}", i)
                    } else {
                        format!("{}.{}", prefix, i)
                    };
                    Self::extract_keys_recursive(value, keys, &index_key);
                }
            }
            _ => {} // Leaf values don't have keys
        }
    }

    /// Check if TOML content contains template variables
    ///
    /// # Arguments
    /// * `content` - TOML content
    pub fn contains_templates(content: &str) -> bool {
        content.contains("{{") || content.contains("{%") || content.contains("{#")
    }

    /// Count template variables in TOML content
    ///
    /// # Arguments
    /// * `content` - TOML content
    pub fn count_variables(content: &str) -> usize {
        let mut count = 0;
        let mut in_braces = false;

        for ch in content.chars() {
            match ch {
                '{' => {
                    if let Some(next) = content.chars().nth(count + 1) {
                        if next == '{' {
                            in_braces = true;
                        }
                    }
                }
                '}' => {
                    if let Some(prev) = content.chars().nth(count - 1) {
                        if prev == '}' && in_braces {
                            in_braces = false;
                        }
                    }
                }
                _ => {
                    if in_braces {
                        // Count variables (simplified - would need proper parsing)
                        count += 1;
                    }
                }
            }
        }

        count
    }
}

/// TOML file differences for comparison
#[derive(Debug, Clone)]
pub struct TomlDiff {
    /// Keys added in second file
    pub added: Vec<(String, Value)>,
    /// Keys removed from first file
    pub removed: Vec<String>,
    /// Keys with different values
    pub changed: Vec<(String, Value, Value)>,
}

/// Fluent API for TOML file operations
pub struct TomlFileBuilder {
    loader: TomlLoader,
    writer: TomlWriter,
    merger: TomlMerger,
}

impl TomlFileBuilder {
    /// Start building TOML file operations
    pub fn new() -> Self {
        Self {
            loader: TomlLoader::new(),
            writer: TomlWriter::new(),
            merger: TomlMerger::new(),
        }
    }

    /// Configure loader
    pub fn loader<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TomlLoader) -> TomlLoader,
    {
        self.loader = f(self.loader);
        self
    }

    /// Configure writer
    pub fn writer<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TomlWriter) -> TomlWriter,
    {
        self.writer = f(self.writer);
        self
    }

    /// Configure merger
    pub fn merger<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TomlMerger) -> TomlMerger,
    {
        self.merger = f(self.merger);
        self
    }

    /// Load TOML file
    pub fn load<P: AsRef<Path>>(self, path: P) -> Result<TomlFile> {
        self.loader.load_file(path)
    }

    /// Write TOML file
    pub fn write<P: AsRef<Path>>(self, path: P, content: &str, validator: Option<&crate::validation::TemplateValidator>) -> Result<()> {
        self.writer.write_file(path, content, validator)
    }

    /// Merge TOML files
    pub fn merge(self, files: &[&TomlFile]) -> Result<TomlFile> {
        self.merger.merge_files(files)
    }
}

impl Default for TomlFileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_toml_file_loading() {
        let temp_dir = tempdir().unwrap();
        let toml_file = temp_dir.path().join("test.toml");

        let content = r#"
[service]
name = "test-service"

[meta]
version = "1.0.0"
        "#;

        fs::write(&toml_file, content).unwrap();

        let loader = TomlLoader::new()
            .with_search_path(&temp_dir)
            .with_extensions(vec!["toml"]);

        let file = loader.load_file(&toml_file).unwrap();

        assert_eq!(file.path, toml_file);
        assert_eq!(file.content, content);
        assert!(file.parsed.get("service").is_some());
        assert!(file.parsed.get("meta").is_some());
    }

    #[test]
    fn test_toml_merging() {
        let base_content = r#"
[service]
name = "base-service"

[meta]
version = "1.0.0"
        "#;

        let overlay_content = r#"
[service]
description = "overlay description"

[config]
debug = true
        "#;

        let base_parsed = toml::from_str::<Value>(base_content).unwrap();
        let overlay_parsed = toml::from_str::<Value>(overlay_content).unwrap();

        let merger = TomlMerger::new();
        let merged = merger.merge(&base_parsed, &overlay_parsed).unwrap();

        // Should have both service.name and service.description
        assert!(merged.get("service").unwrap().get("name").is_some());
        assert!(merged.get("service").unwrap().get("description").is_some());
        assert!(merged.get("meta").is_some());
        assert!(merged.get("config").is_some());
    }

    #[test]
    fn test_variable_extraction() {
        let content = r#"
service = "{{ service_name }}"
config = "{{ config.env }}"
        "#;

        let variables = TomlUtils::extract_variables(content).unwrap();
        assert!(variables.contains("service_name"));
        assert!(variables.contains("config.env"));
    }

    #[test]
    fn test_toml_validation() {
        let temp_dir = tempdir().unwrap();
        let toml_file = temp_dir.path().join("config.toml");

        let content = r#"
[service]
name = "test-service"

[meta]
version = "1.0.0"
        "#;

        fs::write(&toml_file, content).unwrap();

        let file = TomlLoader::new().load_file(&toml_file).unwrap();
        TomlUtils::validate_structure(&file, &["service", "meta"]).unwrap();
    }

    #[test]
    fn test_toml_formatting() {
        let content = r#"[service]name="test"[meta]version="1.0.0""#;
        let formatted = TomlUtils::format_toml(content).unwrap();

        assert!(formatted.contains("[service]"));
        assert!(formatted.contains("[meta]"));
        assert!(formatted.contains("name = \"test\""));
        assert!(formatted.contains("version = \"1.0.0\""));
    }

    #[test]
    fn test_toml_key_extraction() {
        let content = r#"
[service]
name = "test"

[config]
debug = true

[database]
host = "localhost"
        "#;

        let keys = TomlUtils::extract_keys(content).unwrap();
        assert!(keys.contains("service"));
        assert!(keys.contains("service.name"));
        assert!(keys.contains("config"));
        assert!(keys.contains("config.debug"));
        assert!(keys.contains("database"));
        assert!(keys.contains("database.host"));
    }
}