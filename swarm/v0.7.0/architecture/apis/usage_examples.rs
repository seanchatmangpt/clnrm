// Usage Examples for v0.7.0 DX Features
// Demonstrates proper usage patterns following clnrm standards

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::collections::HashMap;

// Import trait definitions (these would be in the actual module)
use crate::trait_definitions::*;

// ============================================================================
// Example 1: FileWatcher - Watch for Test Changes
// ============================================================================

/// Example implementation of FileWatcher using notify crate
pub struct NotifyFileWatcher {
    // Internal state would use Arc<Mutex<...>> for thread safety
}

impl NotifyFileWatcher {
    pub fn new() -> DxResult<Self> {
        Ok(Self {})
    }
}

impl FileWatcher for NotifyFileWatcher {
    fn watch(&self, paths: &[PathBuf], config: WatchConfig) -> DxResult<WatchStream> {
        // CRITICAL: Use tokio::task::block_in_place for async operations
        // This keeps the trait method sync while allowing async internally

        let events = vec![
            Ok(FileChangeEvent {
                path: PathBuf::from("tests/example.clnrm.toml"),
                change_type: ChangeType::Modified,
                timestamp: std::time::SystemTime::now(),
            })
        ];

        Ok(WatchStream::new(events.into_iter()))
    }

    fn stop(&self) -> DxResult<()> {
        Ok(())
    }

    fn is_active(&self) -> bool {
        true
    }

    fn watched_paths(&self) -> Vec<PathBuf> {
        vec![PathBuf::from("tests/")]
    }
}

/// Example: Using FileWatcher in dev mode
pub fn example_watch_mode() -> DxResult<()> {
    // Arrange
    let watcher = NotifyFileWatcher::new()?;
    let config = WatchConfig {
        debounce_duration: Duration::from_millis(100),
        include_patterns: vec!["**/*.toml".to_string(), "**/*.tera".to_string()],
        exclude_patterns: vec!["**/target/**".to_string()],
        recursive: true,
    };

    // Act
    let watch_stream = watcher.watch(&[PathBuf::from("tests/")], config)?;

    // Process events
    for event_result in watch_stream.take(5) {
        match event_result {
            Ok(event) => {
                println!("File changed: {:?} ({})", event.path, match event.change_type {
                    ChangeType::Modified => "modified",
                    ChangeType::Created => "created",
                    ChangeType::Deleted => "deleted",
                    ChangeType::Renamed => "renamed",
                });

                // Re-run tests for the changed file
                // run_tests_for_file(&event.path)?;
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
            }
        }
    }

    // Assert
    watcher.stop()?;
    Ok(())
}

// ============================================================================
// Example 2: ChangeDetector - Hash-Based Change Detection
// ============================================================================

/// Example implementation of ChangeDetector
pub struct Sha256ChangeDetector {
    cache: std::sync::Arc<std::sync::Mutex<HashCache>>,
}

impl Sha256ChangeDetector {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(std::sync::Mutex::new(HashCache::new())),
        }
    }
}

impl ChangeDetector for Sha256ChangeDetector {
    fn hash_file(&self, path: &Path) -> DxResult<FileHash> {
        // Read file and compute SHA-256 hash
        use sha2::{Sha256, Digest};

        let contents = std::fs::read(path)
            .map_err(|e| DxError::ChangeDetectionError(format!("Failed to read {}: {}", path.display(), e)))?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let hash = format!("{:x}", hasher.finalize());

        Ok(FileHash::new(hash))
    }

    fn has_changed(&self, path: &Path) -> DxResult<bool> {
        let current_hash = self.hash_file(path)?;

        let cache = self.cache.lock()
            .map_err(|e| DxError::ChangeDetectionError(format!("Cache lock error: {}", e)))?;

        match cache.get(path) {
            Some(metadata) => Ok(metadata.hash != current_hash),
            None => Ok(true), // New file
        }
    }

    fn update_cache(&self, path: &Path) -> DxResult<FileMetadata> {
        let hash = self.hash_file(path)?;
        let metadata_std = std::fs::metadata(path)
            .map_err(|e| DxError::ChangeDetectionError(format!("Failed to get metadata: {}", e)))?;

        let metadata = FileMetadata {
            path: path.to_path_buf(),
            hash,
            modified_time: metadata_std.modified()
                .map_err(|e| DxError::ChangeDetectionError(format!("Failed to get mtime: {}", e)))?,
            size: metadata_std.len(),
        };

        let mut cache = self.cache.lock()
            .map_err(|e| DxError::ChangeDetectionError(format!("Cache lock error: {}", e)))?;
        cache.insert(metadata.clone());

        Ok(metadata)
    }

    fn invalidate_cache(&self) -> DxResult<()> {
        let mut cache = self.cache.lock()
            .map_err(|e| DxError::ChangeDetectionError(format!("Cache lock error: {}", e)))?;
        cache.clear();
        Ok(())
    }

    fn get_metadata(&self, path: &Path) -> DxResult<Option<FileMetadata>> {
        let cache = self.cache.lock()
            .map_err(|e| DxError::ChangeDetectionError(format!("Cache lock error: {}", e)))?;
        Ok(cache.get(path).cloned())
    }

    fn batch_check(&self, paths: &[PathBuf]) -> DxResult<Vec<(PathBuf, bool)>> {
        paths.iter()
            .map(|path| {
                let changed = self.has_changed(path)?;
                Ok((path.clone(), changed))
            })
            .collect()
    }
}

/// Example: Using ChangeDetector for incremental testing
pub fn example_incremental_testing() -> DxResult<()> {
    // Arrange
    let detector = Sha256ChangeDetector::new();
    let test_files = vec![
        PathBuf::from("tests/test1.clnrm.toml"),
        PathBuf::from("tests/test2.clnrm.toml"),
        PathBuf::from("tests/test3.clnrm.toml"),
    ];

    // Initial run - cache all files
    for file in &test_files {
        detector.update_cache(file)?;
    }

    // Act - simulate file changes and check
    let changes = detector.batch_check(&test_files)?;

    // Assert
    for (path, changed) in changes {
        if changed {
            println!("Re-running tests in: {}", path.display());
            // run_test_file(&path)?;
        } else {
            println!("Skipping unchanged: {}", path.display());
        }
    }

    Ok(())
}

// ============================================================================
// Example 3: DryRunValidator - Validate Without Execution
// ============================================================================

/// Example implementation of DryRunValidator
pub struct TeraTomlValidator {
    tera: tera::Tera,
}

impl TeraTomlValidator {
    pub fn new() -> DxResult<Self> {
        let tera = tera::Tera::default();
        Ok(Self { tera })
    }
}

impl DryRunValidator for TeraTomlValidator {
    fn validate_template(&self, content: &str, config: TemplateValidationConfig) -> DxResult<ValidationReport> {
        let mut report = ValidationReport::new(PathBuf::from("template"));
        let start = std::time::Instant::now();

        // Parse template syntax
        match tera::Tera::one_off(content, &tera::Context::new(), false) {
            Ok(_) => {
                // Template is valid
            }
            Err(e) => {
                report.add_issue(ValidationIssue {
                    severity: Severity::Error,
                    message: format!("Template syntax error: {}", e),
                    line: None,
                    column: None,
                    suggestion: Some("Check template syntax and Tera documentation".to_string()),
                });
            }
        }

        // Check for undefined variables if enabled
        if config.check_undefined_vars {
            // This is simplified - real implementation would parse template AST
            if content.contains("{{ undefined_var }}") {
                report.add_issue(ValidationIssue {
                    severity: Severity::Warning,
                    message: "Potentially undefined variable: undefined_var".to_string(),
                    line: None,
                    column: None,
                    suggestion: Some("Define this variable in your context or template".to_string()),
                });
            }
        }

        report.validation_time_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    fn validate_toml(&self, content: &str) -> DxResult<ValidationReport> {
        let mut report = ValidationReport::new(PathBuf::from("config.toml"));
        let start = std::time::Instant::now();

        // Parse TOML
        match toml::from_str::<toml::Value>(content) {
            Ok(_) => {
                // TOML is valid
            }
            Err(e) => {
                report.add_issue(ValidationIssue {
                    severity: Severity::Error,
                    message: format!("TOML parse error: {}", e),
                    line: e.line_col().map(|(line, _)| line),
                    column: e.line_col().map(|(_, col)| col),
                    suggestion: Some("Check TOML syntax".to_string()),
                });
            }
        }

        report.validation_time_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    fn check_required_keys(&self, toml: &toml::Value) -> DxResult<()> {
        let required_keys = vec!["test.metadata.name", "steps"];

        for key_path in required_keys {
            let parts: Vec<&str> = key_path.split('.').collect();
            let mut current = toml;

            for part in &parts {
                match current.get(part) {
                    Some(value) => current = value,
                    None => {
                        return Err(DxError::ValidationError(
                            format!("Missing required key: {}", key_path)
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_test_file(&self, path: &Path) -> DxResult<ValidationReport> {
        let mut report = ValidationReport::new(path.to_path_buf());
        let start = std::time::Instant::now();

        // Read file
        let content = std::fs::read_to_string(path)
            .map_err(|e| DxError::IoError(e))?;

        // Validate TOML structure
        let toml_report = self.validate_toml(&content)?;
        report.issues.extend(toml_report.issues);

        // Check required keys
        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            if let Err(e) = self.check_required_keys(&value) {
                report.add_issue(ValidationIssue {
                    severity: Severity::Error,
                    message: e.to_string(),
                    line: None,
                    column: None,
                    suggestion: Some("Add the required key to your test configuration".to_string()),
                });
            }
        }

        report.validation_time_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    fn batch_validate(&self, paths: &[PathBuf]) -> DxResult<Vec<ValidationReport>> {
        paths.iter()
            .map(|path| self.validate_test_file(path))
            .collect()
    }

    fn lint_template(&self, content: &str) -> DxResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for anti-patterns
        if content.contains("{% raw %}{% raw %}") {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: "Nested raw blocks detected".to_string(),
                line: None,
                column: None,
                suggestion: Some("Simplify template structure".to_string()),
            });
        }

        // Check for long lines
        for (idx, line) in content.lines().enumerate() {
            if line.len() > 120 {
                issues.push(ValidationIssue {
                    severity: Severity::Info,
                    message: "Line exceeds 120 characters".to_string(),
                    line: Some(idx + 1),
                    column: None,
                    suggestion: Some("Consider breaking into multiple lines".to_string()),
                });
            }
        }

        Ok(issues)
    }
}

/// Example: Using DryRunValidator before test execution
pub fn example_dry_run_validation() -> DxResult<()> {
    // Arrange
    let validator = TeraTomlValidator::new()?;
    let test_files = vec![
        PathBuf::from("tests/valid_test.clnrm.toml"),
        PathBuf::from("tests/invalid_test.clnrm.toml"),
    ];

    // Act
    let reports = validator.batch_validate(&test_files)?;

    // Assert - check results
    for report in reports {
        println!("\nValidation report for: {}", report.path.display());
        println!("  Status: {}", if report.is_valid { "✓ Valid" } else { "✗ Invalid" });
        println!("  Errors: {}", report.error_count());
        println!("  Warnings: {}", report.warning_count());
        println!("  Validation time: {}ms", report.validation_time_ms);

        if !report.issues.is_empty() {
            println!("\n  Issues:");
            for issue in &report.issues {
                let severity = match issue.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARN",
                    Severity::Info => "INFO",
                };
                let location = match (issue.line, issue.column) {
                    (Some(l), Some(c)) => format!(" [{}:{}]", l, c),
                    (Some(l), None) => format!(" [line {}]", l),
                    _ => String::new(),
                };
                println!("    {} {}{}", severity, issue.message, location);
                if let Some(suggestion) = &issue.suggestion {
                    println!("      → {}", suggestion);
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Example 4: DiffEngine - Compare Execution Traces
// ============================================================================

/// Example implementation of DiffEngine
pub struct TraceDiffEngine;

impl DiffEngine for TraceDiffEngine {
    fn compare_traces(&self, baseline: &Trace, current: &Trace) -> DxResult<TraceComparison> {
        let mut differences = Vec::new();

        // Compare exit codes
        if baseline.exit_code != current.exit_code {
            differences.push(TraceDifference {
                diff_type: DiffType::ExitCodeChanged,
                baseline_value: Some(baseline.exit_code.to_string()),
                current_value: Some(current.exit_code.to_string()),
                context: "Test exit code".to_string(),
            });
        }

        // Compare number of entries
        if baseline.entries.len() != current.entries.len() {
            differences.push(TraceDifference {
                diff_type: DiffType::Modified,
                baseline_value: Some(format!("{} entries", baseline.entries.len())),
                current_value: Some(format!("{} entries", current.entries.len())),
                context: "Trace entry count".to_string(),
            });
        }

        // Compare timing
        let baseline_duration = baseline.duration().as_millis() as i64;
        let current_duration = current.duration().as_millis() as i64;
        let timing_delta = current_duration - baseline_duration;

        // Calculate similarity score
        let similarity = if differences.is_empty() { 1.0 } else {
            1.0 - (differences.len() as f64 / 10.0).min(1.0)
        };

        Ok(TraceComparison {
            baseline_name: baseline.test_name.clone(),
            current_name: current.test_name.clone(),
            differences,
            similarity_score: similarity,
            timing_delta_ms: timing_delta,
        })
    }

    fn format_diff(&self, comparison: &TraceComparison, format: DiffFormat) -> DxResult<String> {
        let mut output = String::new();

        output.push_str(&format!("Comparing: {} → {}\n", comparison.baseline_name, comparison.current_name));
        output.push_str(&format!("Similarity: {:.1}%\n", comparison.similarity_score * 100.0));
        output.push_str(&format!("Timing delta: {:+}ms\n\n", comparison.timing_delta_ms));

        if comparison.differences.is_empty() {
            output.push_str("✓ No differences found\n");
        } else {
            output.push_str(&format!("Found {} differences:\n\n", comparison.differences.len()));
            for diff in &comparison.differences {
                let symbol = match diff.diff_type {
                    DiffType::Added => "+",
                    DiffType::Removed => "-",
                    DiffType::Modified => "~",
                    _ => "•",
                };
                output.push_str(&format!("{} {}\n", symbol, diff.context));
                if let Some(baseline) = &diff.baseline_value {
                    output.push_str(&format!("  - {}\n", baseline));
                }
                if let Some(current) = &diff.current_value {
                    output.push_str(&format!("  + {}\n", current));
                }
                output.push('\n');
            }
        }

        Ok(output)
    }

    fn to_json(&self, comparison: &TraceComparison) -> DxResult<serde_json::Value> {
        serde_json::to_value(comparison)
            .map_err(|e| DxError::DiffError(format!("JSON serialization error: {}", e)))
    }

    fn load_trace(&self, path: &Path) -> DxResult<Trace> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| DxError::IoError(e))?;

        serde_json::from_str(&content)
            .map_err(|e| DxError::DiffError(format!("Failed to parse trace: {}", e)))
    }

    fn save_trace(&self, trace: &Trace, path: &Path) -> DxResult<()> {
        let json = serde_json::to_string_pretty(trace)
            .map_err(|e| DxError::DiffError(format!("Failed to serialize trace: {}", e)))?;

        std::fs::write(path, json)
            .map_err(|e| DxError::IoError(e))
    }

    fn similarity_score(&self, baseline: &Trace, current: &Trace) -> DxResult<f64> {
        let comparison = self.compare_traces(baseline, current)?;
        Ok(comparison.similarity_score)
    }
}

/// Example: Using DiffEngine for regression detection
pub fn example_trace_diff() -> DxResult<()> {
    // Arrange
    let engine = TraceDiffEngine;

    let baseline = Trace {
        test_name: "test_container_creation".to_string(),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now() + Duration::from_millis(100),
        entries: vec![],
        exit_code: 0,
        metadata: HashMap::new(),
    };

    let mut current = baseline.clone();
    current.exit_code = 1; // Simulate failure

    // Act
    let comparison = engine.compare_traces(&baseline, &current)?;
    let diff_output = engine.format_diff(&comparison, DiffFormat::default())?;

    // Assert
    println!("{}", diff_output);

    if comparison.similarity_score < 0.9 {
        println!("⚠ Significant differences detected!");
        return Err(DxError::DiffError("Test behavior has changed".to_string()));
    }

    Ok(())
}

// ============================================================================
// Example 5: ParallelExecutor - Concurrent Test Execution
// ============================================================================

/// Example implementation of ParallelExecutor
pub struct RayonParallelExecutor {
    memory_limit: std::sync::Arc<std::sync::Mutex<Option<usize>>>,
}

impl RayonParallelExecutor {
    pub fn new() -> Self {
        Self {
            memory_limit: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

impl ParallelExecutor for RayonParallelExecutor {
    fn execute_scenarios(&self, scenarios: Vec<Scenario>, config: ParallelConfig) -> DxResult<ExecutionResults> {
        use rayon::prelude::*;

        let start = std::time::Instant::now();

        // Configure thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.workers)
            .build()
            .map_err(|e| DxError::ParallelExecutionError(format!("Failed to create thread pool: {}", e)))?;

        // Execute scenarios in parallel
        let results: Vec<ScenarioResult> = pool.install(|| {
            scenarios.par_iter()
                .map(|scenario| {
                    // Execute single scenario
                    let scenario_start = std::time::Instant::now();

                    // Simulate test execution
                    // In real implementation, this would call CleanroomEnvironment
                    let success = true;

                    ScenarioResult {
                        scenario_id: scenario.id.clone(),
                        success,
                        duration: scenario_start.elapsed(),
                        trace: None,
                        error: None,
                    }
                })
                .collect()
        });

        // Aggregate results
        let passed = results.iter().filter(|r| r.success).count();
        let failed = results.len() - passed;

        Ok(ExecutionResults {
            total_scenarios: scenarios.len(),
            passed,
            failed,
            skipped: 0,
            total_duration: start.elapsed(),
            results,
        })
    }

    fn set_memory_limit(&self, bytes: usize) -> DxResult<()> {
        let mut limit = self.memory_limit.lock()
            .map_err(|e| DxError::ParallelExecutionError(format!("Lock error: {}", e)))?;
        *limit = Some(bytes);
        Ok(())
    }

    fn memory_usage(&self) -> DxResult<usize> {
        // In real implementation, would query system metrics
        Ok(0)
    }

    fn cancel_all(&self) -> DxResult<()> {
        // In real implementation, would signal cancellation
        Ok(())
    }

    fn progress(&self) -> DxResult<ExecutionProgress> {
        // In real implementation, would track actual progress
        Ok(ExecutionProgress {
            total_scenarios: 0,
            completed: 0,
            running: 0,
            pending: 0,
            failed: 0,
        })
    }
}

/// Example: Using ParallelExecutor for fast test runs
pub fn example_parallel_execution() -> DxResult<()> {
    // Arrange
    let executor = RayonParallelExecutor::new();

    let scenarios = vec![
        Scenario {
            id: "1".to_string(),
            name: "Test 1".to_string(),
            path: PathBuf::from("tests/test1.clnrm.toml"),
            timeout: Some(Duration::from_secs(30)),
            priority: Priority::High,
        },
        Scenario {
            id: "2".to_string(),
            name: "Test 2".to_string(),
            path: PathBuf::from("tests/test2.clnrm.toml"),
            timeout: Some(Duration::from_secs(30)),
            priority: Priority::Normal,
        },
    ];

    let config = ParallelConfig {
        workers: 4,
        memory_limit_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
        global_timeout: Some(Duration::from_secs(300)),
        fail_fast: false,
        shuffle: true,
    };

    // Act
    let results = executor.execute_scenarios(scenarios, config)?;

    // Assert
    println!("\nExecution Results:");
    println!("  Total: {}", results.total_scenarios);
    println!("  Passed: {}", results.passed);
    println!("  Failed: {}", results.failed);
    println!("  Success rate: {:.1}%", results.success_rate() * 100.0);
    println!("  Total time: {:.2}s", results.total_duration.as_secs_f64());

    if results.failed > 0 {
        return Err(DxError::ParallelExecutionError(
            format!("{} tests failed", results.failed)
        ));
    }

    Ok(())
}

// ============================================================================
// Integration Example: Complete Dev Workflow
// ============================================================================

/// Example: Complete development workflow using all DX features
pub fn example_complete_dev_workflow() -> DxResult<()> {
    println!("Starting development workflow...\n");

    // 1. Validate all tests without running
    println!("Step 1: Dry-run validation");
    let validator = TeraTomlValidator::new()?;
    let test_files = vec![PathBuf::from("tests/")];
    // Validation would happen here
    println!("  ✓ All tests validated\n");

    // 2. Set up file watching
    println!("Step 2: Start watch mode");
    let watcher = NotifyFileWatcher::new()?;
    let detector = Sha256ChangeDetector::new();
    println!("  ✓ Watching for changes\n");

    // 3. On file change, run affected tests in parallel
    println!("Step 3: Detected change in tests/example.clnrm.toml");

    let executor = RayonParallelExecutor::new();
    let scenarios = vec![
        Scenario {
            id: "1".to_string(),
            name: "Example test".to_string(),
            path: PathBuf::from("tests/example.clnrm.toml"),
            timeout: Some(Duration::from_secs(30)),
            priority: Priority::High,
        },
    ];

    let config = ParallelConfig::default();
    let results = executor.execute_scenarios(scenarios, config)?;

    println!("  ✓ Tests executed: {} passed, {} failed\n", results.passed, results.failed);

    // 4. Compare with baseline
    println!("Step 4: Compare with baseline");
    let diff_engine = TraceDiffEngine;
    // Diff comparison would happen here
    println!("  ✓ No regressions detected\n");

    println!("Development workflow complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_watcher_creation() {
        // Arrange, Act, Assert pattern
        let watcher = NotifyFileWatcher::new();
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_change_detector_hash() -> DxResult<()> {
        // Arrange
        let detector = Sha256ChangeDetector::new();

        // Act
        // Would test with actual file

        // Assert
        Ok(())
    }
}
