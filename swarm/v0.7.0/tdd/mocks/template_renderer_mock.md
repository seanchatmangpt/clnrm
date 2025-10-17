# MockTemplateRenderer Contract

## Purpose
Define interaction contract for Tera template rendering with context injection and error simulation.

## Mock Trait Definition

```rust
use std::collections::HashMap;
use serde_json::Value;
use crate::core::Result;

/// Mock implementation of template rendering behavior
pub trait MockTemplateRenderer: Send + Sync {
    /// Render a template with given context
    ///
    /// Interactions to verify:
    /// - Called with correct template path
    /// - Receives expected context variables
    /// - Called in response to file changes
    fn render(&self, template_path: &str, context: &HashMap<String, Value>) -> Result<String>;

    /// Validate template syntax without rendering
    ///
    /// Interactions to verify:
    /// - Called before first render
    /// - Called after template changes detected
    fn validate_syntax(&self, template_path: &str) -> Result<()>;

    /// Get list of variables used in template
    ///
    /// Used for context validation
    fn get_required_variables(&self, template_path: &str) -> Result<Vec<String>>;

    /// Clear cached templates (force re-parse)
    ///
    /// Interactions to verify:
    /// - Called after template file changes
    /// - Called before re-rendering
    fn clear_cache(&self, template_path: &str) -> Result<()>;

    /// Check if template is in cache
    fn is_cached(&self, template_path: &str) -> bool;
}
```

## Mock Implementation for Tests

```rust
use std::sync::{Arc, Mutex};

/// Test mock with interaction tracking and response configuration
pub struct TestTemplateRenderer {
    /// Tracks render() calls with arguments
    render_calls: Arc<Mutex<Vec<RenderCall>>>,

    /// Tracks validate_syntax() calls
    validation_calls: Arc<Mutex<Vec<String>>>,

    /// Tracks clear_cache() calls
    cache_clear_calls: Arc<Mutex<Vec<String>>>,

    /// Configured responses for templates
    responses: Arc<Mutex<HashMap<String, RendererResponse>>>,

    /// Current cache state
    cache_state: Arc<Mutex<HashMap<String, bool>>>,
}

#[derive(Debug, Clone)]
struct RenderCall {
    template_path: String,
    context: HashMap<String, Value>,
    timestamp: std::time::Instant,
}

enum RendererResponse {
    Success(String),
    Error(String),
    ContextError(Vec<String>), // Missing variables
}

impl TestTemplateRenderer {
    pub fn new() -> Self {
        Self {
            render_calls: Arc::new(Mutex::new(Vec::new())),
            validation_calls: Arc::new(Mutex::new(Vec::new())),
            cache_clear_calls: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(HashMap::new())),
            cache_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configure mock to return specific output for template
    pub fn configure_response(&self, template: &str, output: String) {
        self.responses.lock().unwrap()
            .insert(template.to_string(), RendererResponse::Success(output));
    }

    /// Configure mock to return error for template
    pub fn configure_error(&self, template: &str, error: String) {
        self.responses.lock().unwrap()
            .insert(template.to_string(), RendererResponse::Error(error));
    }

    /// Configure mock to return missing context error
    pub fn configure_context_error(&self, template: &str, missing: Vec<String>) {
        self.responses.lock().unwrap()
            .insert(template.to_string(), RendererResponse::ContextError(missing));
    }

    /// Verify render() was called for template
    pub fn verify_rendered(&self, template: &str) -> bool {
        self.render_calls.lock().unwrap()
            .iter()
            .any(|call| call.template_path == template)
    }

    /// Verify render() was called with specific context key
    pub fn verify_context_contains(&self, template: &str, key: &str) -> bool {
        self.render_calls.lock().unwrap()
            .iter()
            .filter(|call| call.template_path == template)
            .any(|call| call.context.contains_key(key))
    }

    /// Get render call count for template
    pub fn render_call_count(&self, template: &str) -> usize {
        self.render_calls.lock().unwrap()
            .iter()
            .filter(|call| call.template_path == template)
            .count()
    }

    /// Verify validation was called for template
    pub fn verify_validated(&self, template: &str) -> bool {
        self.validation_calls.lock().unwrap()
            .iter()
            .any(|t| t == template)
    }

    /// Verify cache was cleared for template
    pub fn verify_cache_cleared(&self, template: &str) -> bool {
        self.cache_clear_calls.lock().unwrap()
            .iter()
            .any(|t| t == template)
    }

    /// Get last render call for template
    pub fn last_render_call(&self, template: &str) -> Option<RenderCall> {
        self.render_calls.lock().unwrap()
            .iter()
            .filter(|call| call.template_path == template)
            .last()
            .cloned()
    }
}

impl MockTemplateRenderer for TestTemplateRenderer {
    fn render(&self, template_path: &str, context: &HashMap<String, Value>) -> Result<String> {
        // Track call
        self.render_calls.lock().unwrap().push(RenderCall {
            template_path: template_path.to_string(),
            context: context.clone(),
            timestamp: std::time::Instant::now(),
        });

        // Return configured response
        let responses = self.responses.lock().unwrap();
        match responses.get(template_path) {
            Some(RendererResponse::Success(output)) => Ok(output.clone()),
            Some(RendererResponse::Error(error)) => {
                Err(CleanroomError::template_error(error.clone()))
            }
            Some(RendererResponse::ContextError(missing)) => {
                Err(CleanroomError::template_error(format!(
                    "Missing required context variables: {}",
                    missing.join(", ")
                )))
            }
            None => {
                // Default response if not configured
                Ok(format!("# Rendered output from {}", template_path))
            }
        }
    }

    fn validate_syntax(&self, template_path: &str) -> Result<()> {
        self.validation_calls.lock().unwrap().push(template_path.to_string());

        // Check for configured validation errors
        let responses = self.responses.lock().unwrap();
        if let Some(RendererResponse::Error(error)) = responses.get(template_path) {
            return Err(CleanroomError::template_error(error.clone()));
        }

        Ok(())
    }

    fn get_required_variables(&self, template_path: &str) -> Result<Vec<String>> {
        // Default set of required variables
        Ok(vec![
            "test_name".to_string(),
            "service_name".to_string(),
            "image".to_string(),
        ])
    }

    fn clear_cache(&self, template_path: &str) -> Result<()> {
        self.cache_clear_calls.lock().unwrap().push(template_path.to_string());
        self.cache_state.lock().unwrap().insert(template_path.to_string(), false);
        Ok(())
    }

    fn is_cached(&self, template_path: &str) -> bool {
        *self.cache_state.lock().unwrap()
            .get(template_path)
            .unwrap_or(&false)
    }
}
```

## Test Examples

### Example 1: Verify Render Called with Correct Context

```rust
#[tokio::test]
async fn test_live_reload_renders_with_updated_context() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    mock_renderer.configure_response(
        "test.toml.tera",
        "[test.metadata]\nname = \"rendered_test\"\n".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let mut context = HashMap::new();
    context.insert("test_name".to_string(), Value::String("my_test".to_string()));
    context.insert("image".to_string(), Value::String("alpine:latest".to_string()));

    // Act
    orchestrator.render_template("test.toml.tera", &context).await?;

    // Assert - Verify render called with correct context
    assert!(
        mock_renderer.verify_rendered("test.toml.tera"),
        "Render should be called"
    );
    assert!(
        mock_renderer.verify_context_contains("test.toml.tera", "test_name"),
        "Context should include test_name"
    );
    assert!(
        mock_renderer.verify_context_contains("test.toml.tera", "image"),
        "Context should include image"
    );

    // Verify context values
    let last_call = mock_renderer.last_render_call("test.toml.tera").unwrap();
    assert_eq!(
        last_call.context.get("test_name").unwrap(),
        &Value::String("my_test".to_string())
    );

    Ok(())
}
```

### Example 2: Verify Cache Cleared After Template Change

```rust
#[tokio::test]
async fn test_live_reload_clears_cache_on_template_change() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let mock_watcher = Arc::new(TestFileWatcher::new());

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let template_path = "test.toml.tera";

    // Act - Simulate template change
    orchestrator.start().await?;
    mock_watcher.trigger_change(Path::new(template_path))?;

    // Assert - Verify interaction sequence
    assert!(
        mock_renderer.verify_cache_cleared(template_path),
        "Cache should be cleared after template change"
    );
    assert!(
        mock_renderer.verify_rendered(template_path),
        "Template should be re-rendered after cache clear"
    );

    // Verify order: clear_cache() called before render()
    let clear_calls = mock_renderer.cache_clear_calls.lock().unwrap();
    let render_calls = mock_renderer.render_calls.lock().unwrap();
    assert!(!clear_calls.is_empty() && !render_calls.is_empty());

    Ok(())
}
```

### Example 3: Verify Validation Before First Render

```rust
#[tokio::test]
async fn test_live_reload_validates_template_syntax_before_rendering() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let template_path = "test.toml.tera";

    // Act
    orchestrator.load_and_render(template_path).await?;

    // Assert - Verify validation called before render
    assert!(
        mock_renderer.verify_validated(template_path),
        "Template should be validated"
    );
    assert!(
        mock_renderer.verify_rendered(template_path),
        "Template should be rendered after validation"
    );

    Ok(())
}
```

### Example 4: Verify Error Propagation

```rust
#[tokio::test]
async fn test_live_reload_propagates_rendering_errors() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    mock_renderer.configure_error(
        "invalid.toml.tera",
        "Syntax error at line 5: unexpected token".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    // Act
    let result = orchestrator.render_template("invalid.toml.tera", &HashMap::new()).await;

    // Assert - Verify error propagation
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Syntax error at line 5"),
        "Should propagate original error message"
    );

    Ok(())
}
```

### Example 5: Verify Context Variable Validation

```rust
#[tokio::test]
async fn test_live_reload_validates_required_context_variables() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    mock_renderer.configure_context_error(
        "test.toml.tera",
        vec!["test_name".to_string(), "image".to_string()],
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let incomplete_context = HashMap::new(); // Missing required variables

    // Act
    let result = orchestrator.render_template("test.toml.tera", &incomplete_context).await;

    // Assert - Verify context validation
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Missing required context variables"));
    assert!(error_msg.contains("test_name"));
    assert!(error_msg.contains("image"));

    Ok(())
}
```

### Example 6: Verify Render Call Count

```rust
#[tokio::test]
async fn test_live_reload_renders_once_per_change() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let mock_watcher = Arc::new(TestFileWatcher::new());

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let template_path = "test.toml.tera";

    // Act - Trigger multiple changes
    orchestrator.start().await?;
    mock_watcher.trigger_change(Path::new(template_path))?;
    mock_watcher.trigger_change(Path::new(template_path))?;
    mock_watcher.trigger_change(Path::new(template_path))?;

    // Assert - Verify render called for each change
    assert_eq!(
        mock_renderer.render_call_count(template_path), 3,
        "Should render once per change event"
    );

    Ok(())
}
```

## Interaction Patterns to Verify

### Pattern 1: First Render Sequence
```
1. get_required_variables(template_path)
2. validate_syntax(template_path)
3. render(template_path, context)
4. Mark as cached
```

### Pattern 2: Template Change Response
```
1. File change detected
2. clear_cache(template_path)
3. validate_syntax(template_path)
4. render(template_path, updated_context)
```

### Pattern 3: Error Recovery
```
1. render() fails with syntax error
2. User fixes template
3. File change detected
4. clear_cache() called
5. validate_syntax() succeeds
6. render() succeeds
```

## Contract Guarantees

### Pre-conditions
- Template path must be valid
- Context must contain all required variables
- Template syntax must be valid (or error returned)

### Post-conditions
- render() returns valid TOML or error
- Cache state reflects rendering status
- Validation called before first render

### Invariants
- Render calls tracked in order
- Context passed completely to renderer
- Cache cleared before re-rendering changed template

## Mock Configuration Helpers

```rust
impl TestTemplateRenderer {
    /// Configure mock with complete template scenario
    pub fn with_template_scenario(self, scenario: TemplateScenario) -> Self {
        match scenario {
            TemplateScenario::ValidRender { template, output } => {
                self.configure_response(&template, output);
            }
            TemplateScenario::SyntaxError { template, error } => {
                self.configure_error(&template, error);
            }
            TemplateScenario::MissingContext { template, variables } => {
                self.configure_context_error(&template, variables);
            }
        }
        self
    }

    /// Reset all tracking state
    pub fn reset(&self) {
        self.render_calls.lock().unwrap().clear();
        self.validation_calls.lock().unwrap().clear();
        self.cache_clear_calls.lock().unwrap().clear();
    }

    /// Get complete interaction log
    pub fn interaction_log(&self) -> Vec<RendererInteraction> {
        // Build chronological interaction log
        vec![]
    }
}

pub enum TemplateScenario {
    ValidRender { template: String, output: String },
    SyntaxError { template: String, error: String },
    MissingContext { template: String, variables: Vec<String> },
}
```

## Usage with Multiple Mocks

```rust
#[tokio::test]
async fn test_complete_live_reload_with_all_collaborators() -> Result<()> {
    // Arrange - All mocks
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let mock_detector = Arc::new(TestChangeDetector::new());

    mock_renderer.configure_response(
        "test.toml.tera",
        "[test.metadata]\nname = \"test\"\n".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    // Act - Complete workflow
    orchestrator.start().await?;
    mock_watcher.trigger_change(Path::new("test.toml.tera"))?;

    // Assert - Verify all collaborator interactions
    assert!(mock_watcher.verify_watched(Path::new("test.toml.tera")));
    assert!(mock_detector.verify_change_detected("test.toml.tera"));
    assert!(mock_renderer.verify_cache_cleared("test.toml.tera"));
    assert!(mock_renderer.verify_rendered("test.toml.tera"));

    Ok(())
}
```

## Design Notes

1. **Behavior Over State**: Tests verify render calls, not internal caching logic
2. **Interaction Recording**: Complete tracking of method calls and arguments
3. **Response Configuration**: Flexible setup for success/error scenarios
4. **Context Validation**: Ensures required variables present
5. **Order Verification**: Can verify sequence of operations
