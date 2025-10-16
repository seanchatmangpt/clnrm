//! Plugin Discovery System
//!
//! Provides intelligent plugin search, filtering, and recommendation capabilities
//! for the marketplace ecosystem.

use crate::error::{CleanroomError, Result};
use crate::marketplace::{MarketplaceConfig, metadata::*};
use std::collections::HashMap;

/// Plugin discovery engine
pub struct PluginDiscovery {
    config: MarketplaceConfig,
    /// Cached search index
    search_index: HashMap<String, Vec<PluginMetadata>>,
}

impl PluginDiscovery {
    /// Create new discovery engine
    pub fn new(config: &MarketplaceConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            search_index: HashMap::new(),
        })
    }

    /// Search for plugins by query string
    pub async fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        // TODO: Implement actual search against remote registries
        // For now, return mock results for demonstration

        let mock_plugins = self.generate_mock_plugins();

        if query.is_empty() {
            return Ok(mock_plugins);
        }

        // Simple keyword matching
        let query_lower = query.to_lowercase();
        let results: Vec<PluginMetadata> = mock_plugins
            .into_iter()
            .filter(|plugin| {
                plugin.name.to_lowercase().contains(&query_lower)
                    || plugin.description.to_lowercase().contains(&query_lower)
                    || plugin.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    /// Search plugins by category
    pub async fn search_by_category(&self, category: &PluginCategory) -> Result<Vec<PluginMetadata>> {
        let all_plugins = self.search_plugins("").await?;

        let results: Vec<PluginMetadata> = all_plugins
            .into_iter()
            .filter(|plugin| {
                plugin.capabilities.iter().any(|cap| &cap.category == category)
            })
            .collect();

        Ok(results)
    }

    /// Get plugin metadata from remote registry
    pub async fn get_plugin_metadata(&self, name: &str) -> Result<PluginMetadata> {
        // TODO: Fetch from remote registry
        // For now, search in mock data
        let plugins = self.search_plugins(name).await?;

        plugins.into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| CleanroomError::validation_error(format!("Plugin '{}' not found", name)))
    }

    /// Get plugin recommendations based on installed plugins
    pub async fn get_recommendations(&self, installed_plugins: &[PluginMetadata]) -> Result<Vec<PluginMetadata>> {
        let all_plugins = self.search_plugins("").await?;

        // Simple recommendation: suggest plugins in same categories
        let mut recommended = Vec::new();
        let installed_names: Vec<_> = installed_plugins.iter().map(|p| p.name.as_str()).collect();

        for plugin in all_plugins {
            if installed_names.contains(&plugin.name.as_str()) {
                continue;
            }

            // Recommend if shares categories with installed plugins
            for installed in installed_plugins {
                if let (Some(cat1), Some(cat2)) = (plugin.primary_category(), installed.primary_category()) {
                    if cat1 == cat2 {
                        recommended.push(plugin.clone());
                        break;
                    }
                }
            }
        }

        // Sort by quality score
        recommended.sort_by(|a, b| {
            b.quality_score().partial_cmp(&a.quality_score()).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(recommended.into_iter().take(10).collect())
    }

    /// Get trending plugins
    pub async fn get_trending(&self, limit: usize) -> Result<Vec<PluginMetadata>> {
        let mut plugins = self.search_plugins("").await?;

        // Sort by download count and recent activity
        plugins.sort_by(|a, b| {
            let score_a = a.community.download_count as f64 + a.community.average_rating * 1000.0;
            let score_b = b.community.download_count as f64 + b.community.average_rating * 1000.0;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(plugins.into_iter().take(limit).collect())
    }

    /// Get plugins by keyword
    pub async fn search_by_keyword(&self, keyword: &str) -> Result<Vec<PluginMetadata>> {
        let all_plugins = self.search_plugins("").await?;

        let keyword_lower = keyword.to_lowercase();
        let results: Vec<PluginMetadata> = all_plugins
            .into_iter()
            .filter(|plugin| {
                plugin.keywords.iter().any(|k| k.to_lowercase() == keyword_lower)
            })
            .collect();

        Ok(results)
    }

    /// Get plugins by author
    pub async fn search_by_author(&self, author: &str) -> Result<Vec<PluginMetadata>> {
        let all_plugins = self.search_plugins("").await?;

        let author_lower = author.to_lowercase();
        let results: Vec<PluginMetadata> = all_plugins
            .into_iter()
            .filter(|plugin| plugin.author.to_lowercase().contains(&author_lower))
            .collect();

        Ok(results)
    }

    /// Get most popular plugins
    pub async fn get_popular(&self, limit: usize) -> Result<Vec<PluginMetadata>> {
        let mut plugins = self.search_plugins("").await?;

        plugins.sort_by(|a, b| {
            b.community.download_count.cmp(&a.community.download_count)
        });

        Ok(plugins.into_iter().take(limit).collect())
    }

    /// Get highest rated plugins
    pub async fn get_top_rated(&self, limit: usize) -> Result<Vec<PluginMetadata>> {
        let mut plugins = self.search_plugins("").await?;

        plugins.sort_by(|a, b| {
            b.community.average_rating
                .partial_cmp(&a.community.average_rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(plugins.into_iter().take(limit).collect())
    }

    /// Generate mock plugins for demonstration
    fn generate_mock_plugins(&self) -> Vec<PluginMetadata> {
        let mut plugins = Vec::new();

        // PostgreSQL plugin
        if let Ok(mut plugin) = PluginMetadata::new(
            "postgres-plugin",
            "1.2.3",
            "Production-ready PostgreSQL testing plugin with advanced features",
            "Cleanroom Core Team"
        ) {
            plugin.keywords = vec!["database".to_string(), "postgresql".to_string(), "sql".to_string()];
            plugin.capabilities.push(standard_capabilities::database_capability());
            plugin.homepage = Some("https://plugins.cleanroom.dev/postgres".to_string());
            plugin.repository = Some("https://github.com/cleanroom/postgres-plugin".to_string());
            plugin.community.average_rating = 4.8;
            plugin.community.rating_count = 156;
            plugin.community.download_count = 5420;
            plugins.push(plugin);
        }

        // Redis plugin
        if let Ok(mut plugin) = PluginMetadata::new(
            "redis-plugin",
            "2.0.1",
            "High-performance Redis testing with cache and session management",
            "Community Maintainers"
        ) {
            plugin.keywords = vec!["cache".to_string(), "redis".to_string(), "session".to_string()];
            plugin.capabilities.push(standard_capabilities::cache_capability());
            plugin.homepage = Some("https://plugins.cleanroom.dev/redis".to_string());
            plugin.community.average_rating = 4.6;
            plugin.community.rating_count = 89;
            plugin.community.download_count = 3210;
            plugins.push(plugin);
        }

        // Kafka plugin
        if let Ok(mut plugin) = PluginMetadata::new(
            "kafka-plugin",
            "1.5.0",
            "Apache Kafka streaming and message queue testing plugin",
            "Enterprise Solutions Inc"
        ) {
            plugin.keywords = vec!["kafka".to_string(), "streaming".to_string(), "messages".to_string()];
            plugin.capabilities.push(standard_capabilities::message_queue_capability());
            plugin.community.average_rating = 4.4;
            plugin.community.rating_count = 67;
            plugin.community.download_count = 2100;
            plugins.push(plugin);
        }

        // AI Testing plugin
        if let Ok(mut plugin) = PluginMetadata::new(
            "ai-testing-plugin",
            "0.8.2",
            "Comprehensive AI/ML model testing and validation framework",
            "AI Testing Collective"
        ) {
            plugin.keywords = vec!["ai".to_string(), "ml".to_string(), "testing".to_string(), "validation".to_string()];
            plugin.capabilities.push(standard_capabilities::ai_ml_capability());
            plugin.homepage = Some("https://plugins.cleanroom.dev/ai-testing".to_string());
            plugin.community.average_rating = 4.9;
            plugin.community.rating_count = 234;
            plugin.community.download_count = 8900;
            plugins.push(plugin);
        }

        // MongoDB plugin
        if let Ok(mut plugin) = PluginMetadata::new(
            "mongodb-plugin",
            "1.1.0",
            "MongoDB NoSQL database testing with document validation",
            "Database Community"
        ) {
            plugin.keywords = vec!["database".to_string(), "mongodb".to_string(), "nosql".to_string()];
            plugin.capabilities.push(standard_capabilities::database_capability());
            plugin.community.average_rating = 4.3;
            plugin.community.rating_count = 45;
            plugin.community.download_count = 1500;
            plugins.push(plugin);
        }

        plugins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_plugins() -> Result<()> {
        let config = MarketplaceConfig::default();
        let discovery = PluginDiscovery::new(&config)?;

        let results = discovery.search_plugins("database").await?;
        assert!(!results.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_category() -> Result<()> {
        let config = MarketplaceConfig::default();
        let discovery = PluginDiscovery::new(&config)?;

        let results = discovery.search_by_category(&PluginCategory::Database).await?;
        assert!(!results.is_empty());

        for plugin in results {
            assert!(plugin.capabilities.iter().any(|cap| matches!(cap.category, PluginCategory::Database)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_trending() -> Result<()> {
        let config = MarketplaceConfig::default();
        let discovery = PluginDiscovery::new(&config)?;

        let trending = discovery.get_trending(5).await?;
        assert!(!trending.is_empty());
        assert!(trending.len() <= 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_recommendations() -> Result<()> {
        let config = MarketplaceConfig::default();
        let discovery = PluginDiscovery::new(&config)?;

        let mut installed = PluginMetadata::new(
            "test-db",
            "1.0.0",
            "Test database",
            "Test"
        )?;
        installed.capabilities.push(standard_capabilities::database_capability());

        let recommendations = discovery.get_recommendations(&[installed]).await?;
        // Should recommend other database plugins
        assert!(!recommendations.is_empty());

        Ok(())
    }
}
