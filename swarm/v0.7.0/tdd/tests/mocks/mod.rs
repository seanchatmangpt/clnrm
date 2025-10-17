/// Mock infrastructure for London School TDD
/// Provides test doubles for external dependencies

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Mock file watcher for testing file change detection
#[derive(Clone)]
pub struct MockFileWatcher {
    changes: Arc<Mutex<Vec<(String, Instant)>>>,
    delay: Duration,
}

impl MockFileWatcher {
    pub fn new() -> Self {
        Self {
            changes: Arc::new(Mutex::new(Vec::new())),
            delay: Duration::from_millis(50),
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn trigger_change(&self, path: &str) {
        let mut changes = self.changes.lock().unwrap();
        changes.push((path.to_string(), Instant::now()));
    }

    pub fn get_changes(&self) -> Vec<(String, Instant)> {
        self.changes.lock().unwrap().clone()
    }

    pub fn change_count(&self) -> usize {
        self.changes.lock().unwrap().len()
    }

    pub fn last_change_time(&self) -> Option<Instant> {
        self.changes.lock().unwrap().last().map(|(_, time)| *time)
    }
}

impl Default for MockFileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock template renderer for testing template processing
#[derive(Clone)]
pub struct MockTemplateRenderer {
    calls: Arc<Mutex<Vec<RenderCall>>>,
    should_fail: Arc<Mutex<bool>>,
    render_duration: Duration,
}

#[derive(Clone, Debug)]
pub struct RenderCall {
    pub template_path: String,
    pub timestamp: Instant,
    pub success: bool,
}

impl MockTemplateRenderer {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
            render_duration: Duration::from_millis(100),
        }
    }

    pub fn with_render_duration(mut self, duration: Duration) -> Self {
        self.render_duration = duration;
        self
    }

    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    pub fn render(&self, template_path: &str) -> Result<String, String> {
        std::thread::sleep(self.render_duration);

        let should_fail = *self.should_fail.lock().unwrap();
        let success = !should_fail;

        self.calls.lock().unwrap().push(RenderCall {
            template_path: template_path.to_string(),
            timestamp: Instant::now(),
            success,
        });

        if should_fail {
            Err("Rendering failed".to_string())
        } else {
            Ok(format!("Rendered: {}", template_path))
        }
    }

    pub fn was_called(&self) -> bool {
        !self.calls.lock().unwrap().is_empty()
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    pub fn get_calls(&self) -> Vec<RenderCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn last_render_time(&self) -> Option<Instant> {
        self.calls.lock().unwrap().last().map(|call| call.timestamp)
    }
}

impl Default for MockTemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock TOML parser for testing configuration validation
#[derive(Clone)]
pub struct MockTomlParser {
    parse_results: Arc<Mutex<HashMap<String, Result<ParsedToml, String>>>>,
}

#[derive(Clone, Debug)]
pub struct ParsedToml {
    pub has_meta: bool,
    pub has_otel: bool,
    pub services: Vec<String>,
    pub scenarios: Vec<String>,
}

impl MockTomlParser {
    pub fn new() -> Self {
        Self {
            parse_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_result(&self, content: &str, result: Result<ParsedToml, String>) {
        self.parse_results.lock().unwrap().insert(content.to_string(), result);
    }

    pub fn parse(&self, content: &str) -> Result<ParsedToml, String> {
        self.parse_results
            .lock()
            .unwrap()
            .get(content)
            .cloned()
            .unwrap_or_else(|| Ok(ParsedToml {
                has_meta: true,
                has_otel: true,
                services: vec!["test_service".to_string()],
                scenarios: vec!["test_scenario".to_string()],
            }))
    }
}

impl Default for MockTomlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock formatter for testing template formatting
#[derive(Clone)]
pub struct MockFormatter {
    formatted_content: Arc<Mutex<HashMap<String, String>>>,
}

impl MockFormatter {
    pub fn new() -> Self {
        Self {
            formatted_content: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_formatted(&self, original: &str, formatted: &str) {
        self.formatted_content
            .lock()
            .unwrap()
            .insert(original.to_string(), formatted.to_string());
    }

    pub fn format(&self, content: &str) -> String {
        self.formatted_content
            .lock()
            .unwrap()
            .get(content)
            .cloned()
            .unwrap_or_else(|| content.to_string())
    }

    pub fn needs_formatting(&self, content: &str) -> bool {
        let formatted = self.format(content);
        content != formatted
    }
}

impl Default for MockFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock trace differ for testing trace comparison
#[derive(Clone)]
pub struct MockTraceDiffer {
    differences: Arc<Mutex<Vec<TraceDifference>>>,
}

#[derive(Clone, Debug)]
pub struct TraceDifference {
    pub span_name: String,
    pub difference_type: DifferenceType,
    pub details: String,
}

#[derive(Clone, Debug)]
pub enum DifferenceType {
    MissingSpan,
    ExtraSpan,
    AttributeChanged,
    DurationChanged,
}

impl MockTraceDiffer {
    pub fn new() -> Self {
        Self {
            differences: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_difference(&self, diff: TraceDifference) {
        self.differences.lock().unwrap().push(diff);
    }

    pub fn get_differences(&self) -> Vec<TraceDifference> {
        self.differences.lock().unwrap().clone()
    }

    pub fn has_differences(&self) -> bool {
        !self.differences.lock().unwrap().is_empty()
    }

    pub fn difference_count(&self) -> usize {
        self.differences.lock().unwrap().len()
    }
}

impl Default for MockTraceDiffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_file_watcher_tracks_changes() {
        let watcher = MockFileWatcher::new();
        assert_eq!(watcher.change_count(), 0);

        watcher.trigger_change("test.toml");
        assert_eq!(watcher.change_count(), 1);

        watcher.trigger_change("test2.toml");
        assert_eq!(watcher.change_count(), 2);
    }

    #[test]
    fn test_mock_renderer_tracks_calls() {
        let renderer = MockTemplateRenderer::new();
        assert!(!renderer.was_called());

        renderer.render("test.tera").unwrap();
        assert!(renderer.was_called());
        assert_eq!(renderer.call_count(), 1);
    }

    #[test]
    fn test_mock_renderer_can_fail() {
        let renderer = MockTemplateRenderer::new();
        renderer.set_should_fail(true);

        let result = renderer.render("test.tera");
        assert!(result.is_err());
    }
}
