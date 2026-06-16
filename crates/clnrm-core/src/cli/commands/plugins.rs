//! Plugins command implementation
//!
//! Handles listing and management of available service plugins.

use crate::error::{CleanroomError, Result};
use crate::telemetry::cli_helpers::CliPluginsSpanBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

/// Status of a loaded plugin.
#[derive(Debug, Clone)]
pub enum PluginStatus {
    /// Plugin is loaded and running.
    Loaded,
    /// Plugin has been stopped.
    Stopped,
    /// Plugin encountered an error.
    Error(String),
}

/// Metadata and configuration for a discovered plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin version string.
    pub version: String,
    /// Category / type of the plugin (e.g. "database", "llm").
    pub plugin_type: String,
    /// Runtime status.
    pub status: PluginStatus,
    /// Arbitrary key/value configuration entries.
    pub config: HashMap<String, String>,
}

/// Scan `.clnrm/plugins/` in the current working directory for `*.json` files
/// and deserialize each into a `PluginInfo`.
///
/// Returns an empty `Vec` if the directory does not exist.
pub fn list_plugins_info() -> Result<Vec<PluginInfo>> {
    let plugins_dir = PathBuf::from(".clnrm/plugins");

    if !plugins_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();

    let entries = std::fs::read_dir(&plugins_dir).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to read plugins directory '{}': {}",
            plugins_dir.display(),
            e
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            CleanroomError::io_error(format!("Failed to read directory entry: {}", e))
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match load_plugin_from_path(&path) {
                Ok(info) => plugins.push(info),
                Err(_) => {
                    // Skip invalid plugin files silently
                }
            }
        }
    }

    Ok(plugins)
}

/// Internal helper: load a `PluginInfo` from a JSON file at `path`.
fn load_plugin_from_path(path: &Path) -> Result<PluginInfo> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to read plugin file '{}': {}",
            path.display(),
            e
        ))
    })?;

    let raw: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        CleanroomError::serialization_error(format!(
            "Failed to parse plugin JSON '{}': {}",
            path.display(),
            e
        ))
    })?;

    let name = raw
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CleanroomError::validation_error("Plugin JSON missing required field: name")
        })?
        .to_string();

    let version = raw
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CleanroomError::validation_error("Plugin JSON missing required field: version")
        })?
        .to_string();

    let plugin_type = raw
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CleanroomError::validation_error("Plugin JSON missing required field: type")
        })?
        .to_string();

    let mut config = HashMap::new();
    if let Some(cfg_obj) = raw.get("config").and_then(|v| v.as_object()) {
        for (k, v) in cfg_obj {
            if let Some(s) = v.as_str() {
                config.insert(k.clone(), s.to_string());
            }
        }
    }

    let status_str = raw
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("loaded");
    let status = match status_str {
        "stopped" => PluginStatus::Stopped,
        "error" => PluginStatus::Error(
            raw.get("error_message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string(),
        ),
        _ => PluginStatus::Loaded,
    };

    Ok(PluginInfo {
        name,
        version,
        plugin_type,
        status,
        config,
    })
}

/// Load a plugin by reading its JSON config file at `config`.
///
/// Validates that the fields `"name"`, `"version"`, and `"type"` are present.
pub fn load_plugin(name: &str, config: &Path) -> Result<PluginInfo> {
    let info = load_plugin_from_path(config)?;

    if info.name != name {
        return Err(CleanroomError::validation_error(format!(
            "Plugin name mismatch: expected '{}', got '{}'",
            name, info.name
        )));
    }

    Ok(info)
}

/// Remove the plugin's JSON file from `.clnrm/plugins/<name>.json`.
///
/// Returns `CleanroomError::io_error` if the file is not found.
pub fn unload_plugin(name: &str) -> Result<()> {
    let path = PathBuf::from(format!(".clnrm/plugins/{}.json", name));

    if !path.exists() {
        return Err(CleanroomError::io_error(format!(
            "Plugin '{}' not found at '{}'",
            name,
            path.display()
        )));
    }

    std::fs::remove_file(&path).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to remove plugin file '{}': {}",
            path.display(),
            e
        ))
    })
}

/// Return the `PluginInfo` for the named plugin from `.clnrm/plugins/<name>.json`.
pub fn plugin_status(name: &str) -> Result<PluginInfo> {
    let path = PathBuf::from(format!(".clnrm/plugins/{}.json", name));

    if !path.exists() {
        return Err(CleanroomError::io_error(format!(
            "Plugin '{}' not found at '{}'",
            name,
            path.display()
        )));
    }

    load_plugin_from_path(&path)
}

/// List available plugins
pub fn list_plugins() -> Result<()> {
    // Start telemetry span
    let span = CliPluginsSpanBuilder::new().start();

    info!("📦 Available Service Plugins:");

    // List core plugins
    tracing::info!("✅ generic_container (alpine, ubuntu, debian)");
    tracing::info!("✅ surreal_db (database integration)");
    tracing::info!("✅ network_tools (curl, wget, netcat)");

    // List AI/LLM proxy plugins for automated rollout
    tracing::info!("✅ ollama (local AI model integration)");
    tracing::info!("✅ vllm (high-performance LLM inference)");
    tracing::info!("✅ tgi (Hugging Face text generation inference)");

    // List experimental plugins
    tracing::info!("\n🧪 Experimental Plugins (clnrm-ai crate):");
    tracing::info!("🎭 chaos_engine (controlled failure injection, network partitions)");
    tracing::info!("🤖 ai_test_generator (AI-powered test case generation)");

    // List plugin capabilities
    tracing::info!("\n🔧 Plugin Capabilities:");
    tracing::info!("  • Container lifecycle management");
    tracing::info!("  • Service health monitoring");
    tracing::info!("  • Network connectivity testing");
    tracing::info!("  • Database integration testing");
    tracing::info!("  • AI/LLM proxy automated rollout & testing");
    tracing::info!("    ◦ Ollama (local development)");
    tracing::info!("    ◦ vLLM (production inference)");
    tracing::info!("    ◦ TGI (Hugging Face optimized)");
    tracing::info!("  • Chaos engineering (experimental - clnrm-ai crate)");
    tracing::info!("  • AI-powered test generation (experimental - clnrm-ai crate)");
    tracing::info!("  • Custom service plugins");

    tracing::info!("\n💡 Usage:");
    tracing::info!("  clnrm run tests/your-test.toml");
    tracing::info!("  # Plugins are automatically discovered and loaded");
    tracing::info!("\n🚀 LLM Proxy Testing:");
    tracing::info!("  # Test Ollama: endpoint=http://localhost:11434, model=qwen3-coder:30b");
    tracing::info!(
        "  # Test vLLM: endpoint=http://localhost:8000, model=microsoft/DialoGPT-medium"
    );
    tracing::info!(
        "  # Test TGI: endpoint=http://localhost:8080, model_id=microsoft/DialoGPT-medium"
    );

    // Count plugins for telemetry
    let builtin_plugins = 6; // generic_container, surreal_db, network_tools, ollama, vllm, tgi
    let experimental_plugins = 2; // chaos_engine, ai_test_generator
    let total_plugins = builtin_plugins + experimental_plugins;

    // Build plugin type map
    let plugins_by_type = r#"{"generic": 3, "database": 1, "llm": 3, "chaos": 1, "ai": 1}"#;

    // Finish telemetry span with success
    span.finish(
        true,
        total_plugins,
        builtin_plugins,
        0, // No custom plugins discovered
        Some(plugins_by_type.to_string()),
        None,
    );

    Ok(())
}
