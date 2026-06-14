//! Cache management for test execution

use crate::cache::{Cache, CacheManager};
use crate::cli::types::CliTestResult;
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Filter tests that have changed since last cache update
///
/// Returns only test files whose raw content has changed.
/// Note: We use raw content for caching, not rendered templates, because
/// template rendering requires vars from the parsed TOML (chicken-and-egg problem).
pub async fn filter_changed_tests(
    test_files: &[PathBuf],
    cache_manager: &CacheManager,
) -> Result<Vec<PathBuf>> {
    let mut changed_tests = Vec::new();

    for test_file in test_files {
        // Read raw file content (don't render templates)
        let content = std::fs::read_to_string(test_file).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read test file '{}': {}",
                test_file.display(),
                e
            ))
        })?;

        // Check if file has changed based on raw content
        if cache_manager.has_changed(test_file, &content)? {
            changed_tests.push(test_file.clone());
        }
    }

    Ok(changed_tests)
}

/// Update cache for test results
///
/// Updates cache hashes for successfully executed tests using raw content.
pub async fn update_cache_for_results(
    results: &[CliTestResult],
    cache_manager: &CacheManager,
) -> Result<()> {
    for result in results {
        // Only update cache for passed tests
        if result.passed {
            // Reconstruct the file path from test name
            // This assumes test names match file names (which they should)
            let test_path = PathBuf::from(&result.name);

            // Check if file exists and update cache
            if test_path.exists() {
                let content = tokio::fs::read_to_string(&test_path).await.map_err(|e| {
                    CleanroomError::io_error(format!(
                        "Failed to read test file '{}': {}",
                        test_path.display(),
                        e
                    ))
                })?;

                // Update cache with raw content
                cache_manager.update(&test_path, &content)?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Extended cache types
// ---------------------------------------------------------------------------

/// A composite key that uniquely identifies a test file's cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    /// SHA-256 hex digest of the test file contents.
    pub test_file_hash: String,
    /// SHA-256 hex digest of the configuration (e.g. CLNRM_CONFIG env var).
    pub config_hash: String,
    /// SHA-256 hex digest of relevant environment variables.
    pub env_hash: String,
}

/// The outcome stored in a cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheResult {
    /// The test passed.
    Pass,
    /// The test failed; the string holds a brief reason.
    Fail(String),
    /// The test was skipped.
    Skipped,
}

/// A single cache entry stored on disk as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The key that identifies this entry.
    pub key: CacheKey,
    /// The result of the test run.
    pub result: CacheResult,
    /// Unix timestamp (ms) when this entry was created.
    pub created_at_ms: u64,
    /// Unix timestamp (ms) after which this entry is considered stale.
    pub expires_at_ms: u64,
}

/// High-level interface for the test-result cache.
pub struct TestCache {
    /// Directory where `.json` cache files are stored.
    pub cache_dir: PathBuf,
    /// Time-to-live in seconds for each entry.
    pub ttl_secs: u64,
}

impl TestCache {
    /// Create a new `TestCache` that stores entries under `cache_dir`.
    pub fn new(cache_dir: PathBuf, ttl_secs: u64) -> Self {
        Self { cache_dir, ttl_secs }
    }

    /// Compute a `CacheKey` for the given test file.
    ///
    /// * `test_file_hash` – SHA-256 of the file's byte content.
    /// * `config_hash`    – SHA-256 of the `CLNRM_CONFIG` env var value, or an
    ///                      empty string if unset.
    /// * `env_hash`       – always an empty SHA-256 digest for now (placeholder).
    pub fn compute_key(test_file: &Path) -> Result<CacheKey> {
        let content = std::fs::read(test_file).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read test file for cache key '{}': {}",
                test_file.display(),
                e
            ))
        })?;

        let test_file_hash = {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hex::encode(hasher.finalize())
        };

        let config_hash = {
            let mut hasher = Sha256::new();
            let val = std::env::var("CLNRM_CONFIG").unwrap_or_default();
            hasher.update(val.as_bytes());
            hex::encode(hasher.finalize())
        };

        let env_hash = {
            let mut hasher = Sha256::new();
            hasher.update(b"");
            hex::encode(hasher.finalize())
        };

        Ok(CacheKey {
            test_file_hash,
            config_hash,
            env_hash,
        })
    }

    /// Return the cache file path for `key`.
    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key.test_file_hash))
    }

    /// Current time as Unix milliseconds.
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Look up a cache entry.  Returns `None` if the entry is missing or
    /// expired.
    pub fn lookup(&self, key: &CacheKey) -> Option<CacheEntry> {
        let path = self.entry_path(key);
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        let now = Self::now_ms();
        if entry.expires_at_ms > now {
            Some(entry)
        } else {
            None
        }
    }

    /// Persist a cache entry.  Errors are silently ignored (best-effort).
    pub fn store(&self, key: CacheKey, result: CacheResult) {
        let now = Self::now_ms();
        let entry = CacheEntry {
            key: key.clone(),
            result,
            created_at_ms: now,
            expires_at_ms: now + self.ttl_secs * 1000,
        };

        let path = self.entry_path(&key);

        // Best-effort: create the directory if needed
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(&entry) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Delete the cache file for `key`.  Errors are silently ignored.
    pub fn invalidate(&self, key: &CacheKey) {
        let path = self.entry_path(key);
        let _ = std::fs::remove_file(&path);
    }

    /// Delete all expired entries and return the count of deleted files.
    pub fn clear_expired(&self) -> usize {
        let now = Self::now_ms();
        let mut deleted = 0;

        let entries = match std::fs::read_dir(&self.cache_dir) {
            Ok(e) => e,
            Err(_) => return 0,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                    if cache_entry.expires_at_ms <= now {
                        if std::fs::remove_file(&path).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
        }

        deleted
    }
}
