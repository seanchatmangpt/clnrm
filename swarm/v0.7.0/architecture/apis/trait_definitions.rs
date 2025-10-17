// API Trait Definitions for v0.7.0 DX Features
// All traits are dyn-compatible (sync methods only) per clnrm standards

use std::path::{Path, PathBuf};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during DX operations
#[derive(Debug, thiserror::Error)]
pub enum DxError {
    #[error("File watch error: {0}")]
    WatchError(String),

    #[error("Change detection error: {0}")]
    ChangeDetectionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Template rendering error: {0}")]
    TemplateError(String),

    #[error("TOML parsing error: {0}")]
    TomlError(String),

    #[error("Diff engine error: {0}")]
    DiffError(String),

    #[error("Parallel execution error: {0}")]
    ParallelExecutionError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Timeout error: operation took longer than {0:?}")]
    TimeoutError(Duration),
}

pub type DxResult<T> = Result<T, DxError>;

// ============================================================================
// 1. FileWatcher Trait
// ============================================================================

/// Represents a file change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Iterator over file change events
pub struct WatchStream {
    inner: Box<dyn Iterator<Item = DxResult<FileChangeEvent>> + Send>,
}

impl WatchStream {
    pub fn new(iter: impl Iterator<Item = DxResult<FileChangeEvent>> + Send + 'static) -> Self {
        Self {
            inner: Box::new(iter),
        }
    }
}

impl Iterator for WatchStream {
    type Item = DxResult<FileChangeEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Configuration for file watching behavior
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Debounce duration to prevent duplicate events
    pub debounce_duration: Duration,

    /// Patterns to include (glob patterns)
    pub include_patterns: Vec<String>,

    /// Patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,

    /// Whether to watch recursively
    pub recursive: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(100),
            include_patterns: vec!["**/*.toml".to_string(), "**/*.tera".to_string()],
            exclude_patterns: vec!["**/target/**".to_string(), "**/.git/**".to_string()],
            recursive: true,
        }
    }
}

/// Trait for watching file system changes
/// CRITICAL: All methods MUST be sync (no async) for dyn compatibility
pub trait FileWatcher: Send + Sync {
    /// Start watching the specified paths
    /// Returns an iterator that yields change events
    fn watch(&self, paths: &[PathBuf], config: WatchConfig) -> DxResult<WatchStream>;

    /// Stop watching all paths
    fn stop(&self) -> DxResult<()>;

    /// Check if watcher is currently active
    fn is_active(&self) -> bool;

    /// Get the list of currently watched paths
    fn watched_paths(&self) -> Vec<PathBuf>;
}

// ============================================================================
// 2. ChangeDetector Trait
// ============================================================================

/// Hash of file contents (SHA-256)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileHash(pub String);

impl FileHash {
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Metadata about a file for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub hash: FileHash,
    pub modified_time: std::time::SystemTime,
    pub size: u64,
}

/// Cache of file hashes for change detection
pub struct HashCache {
    /// Map from path to hash and metadata
    cache: std::collections::HashMap<PathBuf, FileMetadata>,
}

impl HashCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, path: &Path) -> Option<&FileMetadata> {
        self.cache.get(path)
    }

    pub fn insert(&mut self, metadata: FileMetadata) {
        self.cache.insert(metadata.path.clone(), metadata);
    }

    pub fn remove(&mut self, path: &Path) -> Option<FileMetadata> {
        self.cache.remove(path)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Trait for detecting file changes using content hashing
/// CRITICAL: All methods MUST be sync (no async) for dyn compatibility
pub trait ChangeDetector: Send + Sync {
    /// Compute hash of file contents
    fn hash_file(&self, path: &Path) -> DxResult<FileHash>;

    /// Check if file has changed since last check
    /// Returns true if file is new or content has changed
    fn has_changed(&self, path: &Path) -> DxResult<bool>;

    /// Update the cache with current file state
    fn update_cache(&self, path: &Path) -> DxResult<FileMetadata>;

    /// Invalidate the entire cache
    fn invalidate_cache(&self) -> DxResult<()>;

    /// Get cached metadata for a file
    fn get_metadata(&self, path: &Path) -> DxResult<Option<FileMetadata>>;

    /// Batch check multiple files for changes
    fn batch_check(&self, paths: &[PathBuf]) -> DxResult<Vec<(PathBuf, bool)>>;
}

// ============================================================================
// 3. DryRunValidator Trait
// ============================================================================

/// Severity level of a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub suggestion: Option<String>,
}

/// Report from validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub path: PathBuf,
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub validation_time_ms: u64,
}

impl ValidationReport {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            is_valid: true,
            issues: Vec::new(),
            validation_time_ms: 0,
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        if issue.severity == Severity::Error {
            self.is_valid = false;
        }
        self.issues.push(issue);
    }

    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
    }
}

/// Configuration for template validation
#[derive(Debug, Clone)]
pub struct TemplateValidationConfig {
    /// Whether to check for undefined variables
    pub check_undefined_vars: bool,

    /// Whether to validate filter syntax
    pub check_filters: bool,

    /// Custom allowed variables
    pub allowed_vars: Vec<String>,

    /// Maximum template depth for includes
    pub max_depth: usize,
}

impl Default for TemplateValidationConfig {
    fn default() -> Self {
        Self {
            check_undefined_vars: true,
            check_filters: true,
            allowed_vars: vec!["test".to_string(), "service".to_string(), "env".to_string()],
            max_depth: 10,
        }
    }
}

/// Trait for validating templates and TOML files without execution
/// CRITICAL: All methods MUST be sync (no async) for dyn compatibility
pub trait DryRunValidator: Send + Sync {
    /// Validate Tera template syntax and structure
    fn validate_template(&self, content: &str, config: TemplateValidationConfig) -> DxResult<ValidationReport>;

    /// Validate TOML content and structure
    fn validate_toml(&self, content: &str) -> DxResult<ValidationReport>;

    /// Check that TOML contains all required keys for test configuration
    fn check_required_keys(&self, toml: &Value) -> DxResult<()>;

    /// Validate a complete test file (TOML + embedded templates)
    fn validate_test_file(&self, path: &Path) -> DxResult<ValidationReport>;

    /// Batch validate multiple files
    fn batch_validate(&self, paths: &[PathBuf]) -> DxResult<Vec<ValidationReport>>;

    /// Check for common anti-patterns and suggest improvements
    fn lint_template(&self, content: &str) -> DxResult<Vec<ValidationIssue>>;
}

// ============================================================================
// 4. DiffEngine Trait
// ============================================================================

/// Represents a single execution trace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub timestamp: std::time::SystemTime,
    pub event_type: TraceEventType,
    pub message: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceEventType {
    ServiceStart,
    ServiceStop,
    CommandExecution,
    OutputCapture,
    Assertion,
    Error,
}

/// Complete execution trace for a test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub test_name: String,
    pub start_time: std::time::SystemTime,
    pub end_time: std::time::SystemTime,
    pub entries: Vec<TraceEntry>,
    pub exit_code: i32,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Trace {
    pub fn duration(&self) -> Duration {
        self.end_time
            .duration_since(self.start_time)
            .unwrap_or(Duration::ZERO)
    }
}

/// Comparison result between two traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceComparison {
    pub baseline_name: String,
    pub current_name: String,
    pub differences: Vec<TraceDifference>,
    pub similarity_score: f64, // 0.0 = completely different, 1.0 = identical
    pub timing_delta_ms: i64,
}

/// A single difference between traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDifference {
    pub diff_type: DiffType,
    pub baseline_value: Option<String>,
    pub current_value: Option<String>,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
    TimingDelta,
    OutputMismatch,
    ExitCodeChanged,
}

/// Format options for diff output
#[derive(Debug, Clone)]
pub struct DiffFormat {
    pub show_context: bool,
    pub context_lines: usize,
    pub colorize: bool,
    pub show_metadata: bool,
}

impl Default for DiffFormat {
    fn default() -> Self {
        Self {
            show_context: true,
            context_lines: 3,
            colorize: true,
            show_metadata: true,
        }
    }
}

/// Trait for comparing execution traces
/// CRITICAL: All methods MUST be sync (no async) for dyn compatibility
pub trait DiffEngine: Send + Sync {
    /// Compare two execution traces
    fn compare_traces(&self, baseline: &Trace, current: &Trace) -> DxResult<TraceComparison>;

    /// Format comparison for human-readable output
    fn format_diff(&self, comparison: &TraceComparison, format: DiffFormat) -> DxResult<String>;

    /// Convert comparison to JSON
    fn to_json(&self, comparison: &TraceComparison) -> DxResult<Value>;

    /// Load trace from file
    fn load_trace(&self, path: &Path) -> DxResult<Trace>;

    /// Save trace to file
    fn save_trace(&self, trace: &Trace, path: &Path) -> DxResult<()>;

    /// Compute similarity score between traces (0.0-1.0)
    fn similarity_score(&self, baseline: &Trace, current: &Trace) -> DxResult<f64>;
}

// ============================================================================
// 5. ParallelExecutor Trait
// ============================================================================

/// A test scenario to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub timeout: Option<Duration>,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/// Result of a single scenario execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub success: bool,
    pub duration: Duration,
    pub trace: Option<Trace>,
    pub error: Option<String>,
}

/// Aggregate results from parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResults {
    pub total_scenarios: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration: Duration,
    pub results: Vec<ScenarioResult>,
}

impl ExecutionResults {
    pub fn success_rate(&self) -> f64 {
        if self.total_scenarios == 0 {
            0.0
        } else {
            self.passed as f64 / self.total_scenarios as f64
        }
    }
}

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads
    pub workers: usize,

    /// Memory limit per worker in bytes
    pub memory_limit_bytes: Option<usize>,

    /// Global timeout for all executions
    pub global_timeout: Option<Duration>,

    /// Whether to fail fast on first error
    pub fail_fast: bool,

    /// Whether to shuffle scenarios for better load balancing
    pub shuffle: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            workers: num_cpus::get(),
            memory_limit_bytes: None,
            global_timeout: None,
            fail_fast: false,
            shuffle: true,
        }
    }
}

/// Trait for parallel test execution
/// CRITICAL: All methods MUST be sync (no async) for dyn compatibility
pub trait ParallelExecutor: Send + Sync {
    /// Execute scenarios in parallel
    /// Uses tokio::task::block_in_place internally for async operations
    fn execute_scenarios(&self, scenarios: Vec<Scenario>, config: ParallelConfig) -> DxResult<ExecutionResults>;

    /// Set memory limit for worker processes
    fn set_memory_limit(&self, bytes: usize) -> DxResult<()>;

    /// Get current memory usage
    fn memory_usage(&self) -> DxResult<usize>;

    /// Cancel all running scenarios
    fn cancel_all(&self) -> DxResult<()>;

    /// Get progress information
    fn progress(&self) -> DxResult<ExecutionProgress>;
}

/// Progress information for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    pub total_scenarios: usize,
    pub completed: usize,
    pub running: usize,
    pub pending: usize,
    pub failed: usize,
}

impl ExecutionProgress {
    pub fn completion_percentage(&self) -> f64 {
        if self.total_scenarios == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total_scenarios as f64) * 100.0
        }
    }
}
