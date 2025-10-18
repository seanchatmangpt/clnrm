//! Template caching and hot-reload system
//!
//! Provides caching for compiled templates and hot-reload functionality
//! for development and dynamic template loading.

use crate::error::{TemplateError, Result};
use crate::renderer::TemplateRenderer;
use crate::context::TemplateContext;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Template cache for compiled templates and metadata
///
/// Caches compiled Tera templates and tracks file modification times
/// for hot-reload functionality.
#[derive(Debug)]
pub struct TemplateCache {
    /// Cached compiled templates (template_name -> compiled_template)
    templates: Arc<RwLock<HashMap<String, CachedTemplate>>>,
    /// File modification times for hot-reload
    file_mtimes: Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Hot-reload enabled
    hot_reload: bool,
    /// Cache TTL (time-to-live)
    ttl: Duration,
}

/// Cached template with metadata
#[derive(Debug, Clone)]
struct CachedTemplate {
    /// Template content
    content: String,
    /// Last modification time
    modified: SystemTime,
    /// Compilation time
    compiled_at: SystemTime,
    /// Template size (for cache management)
    size: usize,
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Templates evicted due to TTL
    pub evictions: u64,
    /// Total cache size (bytes)
    pub total_size: usize,
    /// Number of templates in cache
    pub template_count: usize,
}

impl TemplateCache {
    /// Create new template cache
    ///
    /// # Arguments
    /// * `hot_reload` - Enable hot-reload for file changes
    /// * `ttl` - Cache time-to-live duration
    pub fn new(hot_reload: bool, ttl: Duration) -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            file_mtimes: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            hot_reload,
            ttl,
        }
    }

    /// Create cache with default settings (1 hour TTL, hot-reload enabled)
    pub fn default() -> Self {
        Self::new(true, Duration::from_secs(3600))
    }

    /// Get template from cache or compile if not cached/missing
    ///
    /// # Arguments
    /// * `template_name` - Name of template
    /// * `template_content` - Template content
    /// * `file_path` - Optional file path for hot-reload
    pub fn get_or_compile(&self, template_name: &str, template_content: &str, file_path: Option<&Path>) -> Result<String> {
        // Check if template is in cache and still valid
        if let Some(cached) = self.templates.read().unwrap().get(template_name) {
            if self.is_cache_valid(cached, file_path)? {
                // Cache hit
                self.record_hit();
                return Ok(cached.content.clone());
            }
        }

        // Cache miss or invalid - compile template
        self.record_miss();

        let compiled = self.compile_template(template_content)?;

        // Cache the compiled template
        self.cache_template(template_name, template_content, &compiled)?;

        // Update file modification time for hot-reload
        if let Some(path) = file_path {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(mtime) = metadata.modified() {
                    self.file_mtimes.write().unwrap().insert(path.to_path_buf(), mtime);
                }
            }
        }

        Ok(compiled)
    }

    /// Check if cached template is still valid
    fn is_cache_valid(&self, cached: &CachedTemplate, file_path: Option<&Path>) -> Result<bool> {
        // Check TTL
        let age = SystemTime::now().duration_since(cached.compiled_at)
            .unwrap_or(Duration::from_secs(0));

        if age > self.ttl {
            return Ok(false);
        }

        // Check file modification time if hot-reload is enabled
        if self.hot_reload {
            if let Some(path) = file_path {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(mtime) = metadata.modified() {
                        if let Some(cached_mtime) = self.file_mtimes.read().unwrap().get(path) {
                            if mtime > *cached_mtime {
                                return Ok(false); // File was modified
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    /// Compile template content
    fn compile_template(&self, content: &str) -> Result<String> {
        // For now, just return the content as-is
        // In a real implementation, this would compile Tera templates
        Ok(content.to_string())
    }

    /// Cache compiled template
    fn cache_template(&self, name: &str, content: &str, compiled: &str) -> Result<()> {
        let now = SystemTime::now();
        let cached = CachedTemplate {
            content: compiled.to_string(),
            modified: now,
            compiled_at: now,
            size: compiled.len(),
        };

        // Update cache
        self.templates.write().unwrap().insert(name.to_string(), cached);

        // Update stats
        let mut stats = self.stats.write().unwrap();
        stats.total_size += compiled.len();
        stats.template_count += 1;

        Ok(())
    }

    /// Record cache hit
    fn record_hit(&self) {
        self.stats.write().unwrap().hits += 1;
    }

    /// Record cache miss
    fn record_miss(&self) {
        self.stats.write().unwrap().misses += 1;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Clear cache
    pub fn clear(&self) {
        self.templates.write().unwrap().clear();
        self.file_mtimes.write().unwrap().clear();

        let mut stats = self.stats.write().unwrap();
        stats.total_size = 0;
        stats.template_count = 0;
        stats.evictions = 0;
    }

    /// Evict expired templates
    pub fn evict_expired(&self) -> usize {
        let now = SystemTime::now();
        let mut templates = self.templates.write().unwrap();
        let mut file_mtimes = self.file_mtimes.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let initial_count = templates.len();
        templates.retain(|_name, cached| {
            let age = now.duration_since(cached.compiled_at).unwrap_or(Duration::from_secs(0));
            if age > self.ttl {
                // Template expired
                stats.total_size -= cached.size;
                stats.evictions += 1;
                false
            } else {
                true
            }
        });

        // Clean up file modification times for non-existent templates
        file_mtimes.retain(|path, _| {
            // Check if file still exists
            path.exists()
        });

        stats.template_count = templates.len();
        initial_count - templates.len()
    }
}

/// Cached template renderer with hot-reload support
///
/// Combines TemplateRenderer with TemplateCache for optimal performance
/// and development experience.
pub struct CachedRenderer {
    /// Base template renderer
    renderer: TemplateRenderer,
    /// Template cache
    cache: TemplateCache,
}

impl CachedRenderer {
    /// Create new cached renderer
    ///
    /// # Arguments
    /// * `context` - Template context
    /// * `hot_reload` - Enable hot-reload
    pub fn new(context: TemplateContext, hot_reload: bool) -> Result<Self> {
        let renderer = TemplateRenderer::new()?.with_context(context);
        let cache = TemplateCache::new(hot_reload, Duration::from_secs(3600));

        Ok(Self { renderer, cache })
    }

    /// Render template with caching
    ///
    /// # Arguments
    /// * `template` - Template content
    /// * `name` - Template name for caching
    /// * `file_path` - Optional file path for hot-reload
    pub fn render_cached(&mut self, template: &str, name: &str, file_path: Option<&Path>) -> Result<String> {
        // Try to get from cache first
        if let Ok(cached) = self.cache.get_or_compile(name, template, file_path) {
            return Ok(cached);
        }

        // Fall back to direct rendering if caching fails
        self.renderer.render_str(template, name)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Clear template cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Evict expired templates from cache
    pub fn evict_expired(&self) -> usize {
        self.cache.evict_expired()
    }

    /// Access the underlying template renderer
    pub fn renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    /// Access the underlying template renderer mutably
    pub fn renderer_mut(&mut self) -> &mut TemplateRenderer {
        &mut self.renderer
    }
}

/// Hot-reload watcher for template files
///
/// Monitors template directories for file changes and triggers cache invalidation.
/// Useful for development environments where templates change frequently.
pub struct HotReloadWatcher {
    /// Watched directories
    watched_dirs: Vec<PathBuf>,
    /// Cache to invalidate
    cache: Arc<TemplateCache>,
    /// File watcher (simplified implementation)
    _watcher: Option<Box<dyn Watcher>>,
}

impl HotReloadWatcher {
    /// Create new hot-reload watcher
    ///
    /// # Arguments
    /// * `cache` - Template cache to invalidate on changes
    pub fn new(cache: Arc<TemplateCache>) -> Self {
        Self {
            watched_dirs: Vec::new(),
            cache,
            _watcher: None,
        }
    }

    /// Add directory to watch
    ///
    /// # Arguments
    /// * `path` - Directory path to watch for template changes
    pub fn watch_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.watched_dirs.push(path.as_ref().to_path_buf());
        self
    }

    /// Start watching for file changes
    ///
    /// This is a simplified implementation. In a real implementation,
    /// this would use a proper file watcher like `notify` or `inotify`.
    pub fn start(self) -> Result<()> {
        // For now, just log that we're watching
        // Real implementation would set up file system watchers
        Ok(())
    }

    /// Stop watching (no-op in simplified implementation)
    pub fn stop(&self) -> Result<()> {
        Ok(())
    }
}

// Placeholder trait for file watcher
trait Watcher {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_cache_basic() {
        let cache = TemplateCache::default();
        let template = "Hello {{ name }}";

        // First access should be a miss
        let result = cache.get_or_compile("test", template, None).unwrap();
        assert_eq!(result, template);

        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);

        // Second access should be a hit
        let result = cache.get_or_compile("test", template, None).unwrap();
        assert_eq!(result, template);

        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_cached_renderer() {
        let context = TemplateContext::with_defaults();
        let mut renderer = CachedRenderer::new(context, false).unwrap();

        let template = "service = \"{{ svc }}\"";
        let result = renderer.render_cached(template, "test", None).unwrap();
        assert_eq!(result, template);

        let stats = renderer.cache_stats();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = TemplateCache::new(false, Duration::from_millis(1));

        // Add template
        cache.get_or_compile("test", "Hello", None).unwrap();

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(10));

        // Should be evicted
        let evicted = cache.evict_expired();
        assert_eq!(evicted, 1);
    }
}