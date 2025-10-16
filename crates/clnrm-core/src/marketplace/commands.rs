//! Marketplace CLI Commands
//!
//! Implements the CLI interface for the plugin marketplace functionality.

use crate::error::Result;
use crate::marketplace::{Marketplace, MarketplaceConfig};
use clap::{Parser, Subcommand};

/// Update operation result
#[derive(Debug, Clone)]
pub enum UpdateResult {
    Updated(String, semver::Version),
    NoUpdate(String),
    Failed(String, String),
}

impl std::fmt::Display for UpdateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateResult::Updated(name, version) => {
                write!(f, "✅ {} updated to {}", name, version)
            }
            UpdateResult::NoUpdate(name) => {
                write!(f, "✅ {} is already up to date", name)
            }
            UpdateResult::Failed(name, error) => {
                write!(f, "❌ {} failed to update: {}", name, error)
            }
        }
    }
}

/// Marketplace CLI commands
#[derive(Parser)]
#[command(name = "marketplace")]
#[command(about = "Plugin marketplace for Cleanroom")]
pub struct MarketplaceCommands {
    #[command(subcommand)]
    pub command: MarketplaceSubcommands,
}

/// Marketplace subcommands
#[derive(Subcommand)]
pub enum MarketplaceSubcommands {
    /// Search for plugins
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Install a plugin
    Install {
        /// Plugin name to install
        #[arg(value_name = "PLUGIN")]
        plugin: String,

        /// Specific version to install
        #[arg(short, long)]
        version: Option<String>,

        /// Force installation even if dependencies conflict
        #[arg(short, long)]
        force: bool,
    },

    /// List installed plugins
    List {
        /// Show only installed plugins
        #[arg(short, long)]
        installed: bool,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Get information about a plugin
    Info {
        /// Plugin name
        #[arg(value_name = "PLUGIN")]
        plugin: String,
    },

    /// Update plugins
    Update {
        /// Update all plugins
        #[arg(short, long)]
        all: bool,

        /// Specific plugin to update
        #[arg(value_name = "PLUGIN")]
        plugin: Option<String>,
    },

    /// Rate a plugin
    Rate {
        /// Plugin name
        #[arg(value_name = "PLUGIN")]
        plugin: String,

        /// Rating (1-5)
        #[arg(value_name = "RATING")]
        rating: u8,
    },

    /// Add a review for a plugin
    Review {
        /// Plugin name
        #[arg(value_name = "PLUGIN")]
        plugin: String,

        /// Review text
        #[arg(value_name = "REVIEW")]
        review: String,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin name to uninstall
        #[arg(value_name = "PLUGIN")]
        plugin: String,

        /// Force uninstallation even if other plugins depend on it
        #[arg(short, long)]
        force: bool,
    },

    /// Show marketplace statistics
    Stats {
        /// Plugin name for detailed stats
        #[arg(value_name = "PLUGIN")]
        plugin: Option<String>,
    },
}

/// Execute marketplace commands
pub async fn execute_marketplace_command(
    marketplace: &Marketplace,
    command: MarketplaceSubcommands,
) -> Result<()> {
    match command {
        MarketplaceSubcommands::Search {
            query,
            category,
            limit,
        } => {
            let results = marketplace.search(&query).await?;

            println!("🔍 Search results for '{}':", query);
            println!("Found {} plugins", results.len());

            for plugin in results.iter().take(limit) {
                if let Some(ref cat) = category {
                    if plugin.primary_category().map(|c| c.to_string()) != Some(cat.clone()) {
                        continue;
                    }
                }

                println!(
                    "  📦 {} v{} - {}",
                    plugin.name, plugin.version, plugin.description
                );
                println!(
                    "     by {} | ⭐ {:.1}/5.0",
                    plugin.author, plugin.community.average_rating
                );
            }
        }

        MarketplaceSubcommands::Install {
            plugin,
            version,
            force,
        } => {
            println!("📦 Installing plugin: {}", plugin);

            if let Some(ref ver) = version {
                println!("  Version: {}", ver);
            }

            if force {
                println!("  Force mode: enabled");
            }

            match marketplace.install(&plugin).await {
                Ok(installed) => {
                    println!("✅ Plugin '{}' installed successfully", installed.name);
                    println!("  Version: {}", installed.version);
                    println!("  Author: {}", installed.author);

                    // Record installation in registry
                    let _ = marketplace.registry.record_installation(&plugin).await;
                }
                Err(e) => {
                    if force {
                        println!("❌ Installation failed: {}", e);
                        return Err(e);
                    } else {
                        println!("❌ Installation failed: {}", e);
                        println!("💡 Try --force if you want to ignore dependency conflicts");
                        return Err(e);
                    }
                }
            }
        }

        MarketplaceSubcommands::List {
            installed,
            category,
        } => {
            if installed {
                let plugins = marketplace.list_installed()?;
                println!("📋 Installed plugins ({}):", plugins.len());

                for plugin in plugins {
                    if let Some(ref cat) = category {
                        if plugin.primary_category().map(|c| c.to_string()) != Some(cat.clone()) {
                            continue;
                        }
                    }

                    println!(
                        "  📦 {} v{} - {}",
                        plugin.name, plugin.version, plugin.description
                    );
                    println!(
                        "     by {} | ⭐ {:.1}/5.0 | Downloads: {}",
                        plugin.author,
                        plugin.community.average_rating,
                        plugin.community.download_count
                    );
                }
            } else {
                let plugins = marketplace.search("").await?;
                println!("📋 Available plugins ({}):", plugins.len());

                for plugin in plugins {
                    if let Some(ref cat) = category {
                        if plugin.primary_category().map(|c| c.to_string()) != Some(cat.clone()) {
                            continue;
                        }
                    }

                    println!(
                        "  📦 {} v{} - {}",
                        plugin.name, plugin.version, plugin.description
                    );
                    println!(
                        "     by {} | ⭐ {:.1}/5.0",
                        plugin.author, plugin.community.average_rating
                    );
                }
            }
        }

        MarketplaceSubcommands::Info { plugin } => match marketplace.get_plugin_info(&plugin) {
            Ok(metadata) => {
                println!("📦 Plugin Information: {}", metadata.name);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Version: {}", metadata.version);
                println!("Author: {}", metadata.author);
                println!("License: {}", metadata.license);
                println!("Description: {}", metadata.description);

                if let Some(ref homepage) = metadata.homepage {
                    println!("Homepage: {}", homepage);
                }

                if let Some(ref repo) = metadata.repository {
                    println!("Repository: {}", repo);
                }

                println!("Keywords: {}", metadata.keywords.join(", "));

                if !metadata.capabilities.is_empty() {
                    println!("Capabilities:");
                    for capability in &metadata.capabilities {
                        println!(
                            "  • {} ({}) - {}",
                            capability.name, capability.category, capability.description
                        );
                    }
                }

                if !metadata.dependencies.is_empty() {
                    println!("Dependencies:");
                    for dep in &metadata.dependencies {
                        println!("  • {} {}", dep.name, dep.version_constraint);
                    }
                }

                println!("Community Stats:");
                println!(
                    "  ⭐ Rating: {:.1}/5.0 ({} reviews)",
                    metadata.community.average_rating, metadata.community.rating_count
                );
                println!("  📥 Downloads: {}", metadata.community.download_count);
                println!(
                    "  📅 Created: {}",
                    metadata.community.created_at.format("%Y-%m-%d")
                );
                println!(
                    "  🔄 Updated: {}",
                    metadata.community.updated_at.format("%Y-%m-%d")
                );
            }
            Err(e) => {
                println!("❌ Plugin '{}' not found: {}", plugin, e);
            }
        },

        MarketplaceSubcommands::Update { all, plugin } => {
            if all {
                println!("🔄 Updating all installed plugins...");
                match marketplace.update_all().await {
                    Ok(results) => {
                        for result in results {
                            println!("  {}", result);
                        }
                    }
                    Err(e) => {
                        println!("❌ Update failed: {}", e);
                    }
                }
            } else if let Some(ref plugin_name) = plugin {
                println!("🔄 Updating plugin: {}", plugin_name);
                match marketplace.update_plugin(plugin_name).await {
                    Ok(result) => {
                        println!("  {}", result);
                    }
                    Err(e) => {
                        println!("❌ Update failed: {}", e);
                    }
                }
            } else {
                println!("❌ Please specify --all or provide a plugin name");
            }
        }

        MarketplaceSubcommands::Rate { plugin, rating } => {
            match marketplace.rate_plugin(&plugin, rating).await {
                Ok(_) => {
                    println!("✅ Rated plugin '{}' with {} stars", plugin, rating);
                }
                Err(e) => {
                    println!("❌ Failed to rate plugin: {}", e);
                }
            }
        }

        MarketplaceSubcommands::Review { plugin, review } => {
            match marketplace.review_plugin(&plugin, &review).await {
                Ok(_) => {
                    println!("✅ Added review for plugin '{}'", plugin);
                    println!("Review: {}", review);
                }
                Err(e) => {
                    println!("❌ Failed to add review: {}", e);
                }
            }
        }

        MarketplaceSubcommands::Uninstall { plugin, force } => {
            println!("🗑️  Uninstalling plugin: {}", plugin);

            if force {
                println!("  Force mode: enabled");
            }

            match marketplace.registry.remove_plugin(&plugin).await {
                Ok(_) => {
                    println!("✅ Plugin '{}' uninstalled successfully", plugin);
                }
                Err(e) => {
                    if force {
                        println!("❌ Uninstallation failed: {}", e);
                        return Err(e);
                    } else {
                        println!("❌ Uninstallation failed: {}", e);
                        println!("💡 Try --force if you want to ignore dependency conflicts");
                        return Err(e);
                    }
                }
            }
        }

        MarketplaceSubcommands::Stats { plugin } => {
            if let Some(ref plugin_name) = plugin {
                match marketplace.get_plugin_stats(plugin_name) {
                    Ok(stats) => {
                        println!("📊 Plugin Statistics: {}", stats.metadata.name);
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        println!("Community:");
                        println!(
                            "  ⭐ Average Rating: {:.1}/5.0",
                            stats.community.average_rating
                        );
                        println!("  👥 Rating Count: {}", stats.community.rating_count);
                        println!("  📥 Total Downloads: {}", stats.community.download_count);
                        println!(
                            "  📅 Created: {}",
                            stats.community.created_at.format("%Y-%m-%d")
                        );
                        println!(
                            "  🔄 Last Updated: {}",
                            stats.community.updated_at.format("%Y-%m-%d")
                        );

                        println!("Usage:");
                        println!("  📦 Installations: {}", stats.usage_stats.installations);
                        println!("  🔄 Active: {}", stats.usage_stats.active_installations);
                        println!("  📊 Daily Usage: {:.1}", stats.usage_stats.daily_usage);
                        println!("  🚀 Peak Usage: {}", stats.usage_stats.peak_usage);
                        println!(
                            "  ⏱️  Avg Session: {:.1}s",
                            stats.usage_stats.avg_session_duration
                        );
                        println!("  ❌ Error Rate: {:.1}%", stats.usage_stats.error_rate);

                        println!("Performance:");
                        println!(
                            "  ⚡ Startup Time: {:.1}ms",
                            stats.performance_metrics.avg_startup_time_ms
                        );
                        println!(
                            "  💾 Memory Usage: {:.1}MB",
                            stats.performance_metrics.avg_memory_usage_mb
                        );
                        println!(
                            "  🖥️  CPU Usage: {:.1}%",
                            stats.performance_metrics.avg_cpu_usage_percent
                        );
                        println!(
                            "  📈 P95 Response: {:.1}ms",
                            stats.performance_metrics.p95_response_time_ms
                        );
                        println!(
                            "  ✅ Reliability: {:.1}/100",
                            stats.performance_metrics.reliability_score
                        );
                    }
                    Err(e) => {
                        println!("❌ Failed to get stats for '{}': {}", plugin_name, e);
                    }
                }
            } else {
                println!("❌ Please specify a plugin name for detailed stats");
                println!("💡 Usage: clnrm marketplace stats <plugin-name>");
            }
        }
    }

    Ok(())
}

/// Initialize marketplace with sample plugins for demonstration
pub async fn initialize_sample_marketplace(config: &MarketplaceConfig) -> Result<Marketplace> {
    let marketplace = Marketplace::new(config.clone())?;

    // Register some sample plugins
    let mut postgres_plugin = crate::marketplace::metadata::PluginMetadata::new(
        "postgres-plugin",
        "1.0.0",
        "PostgreSQL database testing plugin",
        "Cleanroom Team",
    )?;

    postgres_plugin.keywords = vec![
        "database".to_string(),
        "postgresql".to_string(),
        "sql".to_string(),
    ];
    postgres_plugin
        .capabilities
        .push(crate::marketplace::metadata::standard_capabilities::database_capability());

    let mut redis_plugin = crate::marketplace::metadata::PluginMetadata::new(
        "redis-plugin",
        "1.2.0",
        "Redis cache and session testing plugin",
        "Community Contributor",
    )?;

    redis_plugin.keywords = vec![
        "cache".to_string(),
        "redis".to_string(),
        "session".to_string(),
    ];
    redis_plugin
        .capabilities
        .push(crate::marketplace::metadata::PluginCapability::new(
            "cache",
            crate::marketplace::metadata::PluginCategory::Storage,
            "Provides Redis cache testing capabilities",
        ));

    let mut ai_plugin = crate::marketplace::metadata::PluginMetadata::new(
        "ai-testing-plugin",
        "0.5.0",
        "AI model testing and validation plugin",
        "AI Testing Community",
    )?;

    ai_plugin.keywords = vec![
        "ai".to_string(),
        "machine-learning".to_string(),
        "testing".to_string(),
    ];
    ai_plugin
        .capabilities
        .push(crate::marketplace::metadata::standard_capabilities::ai_ml_capability());

    // Register plugins
    marketplace
        .registry
        .register_plugin(postgres_plugin)
        .await?;
    marketplace.registry.register_plugin(redis_plugin).await?;
    marketplace.registry.register_plugin(ai_plugin).await?;

    println!("✅ Sample marketplace initialized with 3 plugins");
    Ok(marketplace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::MarketplaceConfig;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_marketplace_commands() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("clnrm-test-{}", uuid::Uuid::new_v4()));
        let config = MarketplaceConfig {
            registry_urls: vec![],
            cache_dir: temp_dir.join("cache"),
            install_dir: temp_dir.join("plugins"),
            community_enabled: false,
            auto_update: false,
        };
        let marketplace = Marketplace::new(config)?;

        // Test search functionality
        let _results = marketplace.search("test").await?;
        // Should find our test plugins or return empty results

        // Test list functionality
        let installed = marketplace.list_installed()?;
        assert!(installed.is_empty()); // No plugins installed initially

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
