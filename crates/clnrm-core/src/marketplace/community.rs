//! Community Features and Social Integration
//!
//! Provides community engagement features including ratings, reviews,
//! discussions, and collaborative plugin development.

use crate::error::{CleanroomError, Result};
use crate::marketplace::metadata::PluginMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User review for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    /// Review ID
    pub id: String,
    /// Plugin name
    pub plugin_name: String,
    /// User ID
    pub user_id: String,
    /// Rating (1-5)
    pub rating: u8,
    /// Review title
    pub title: String,
    /// Review content
    pub content: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Helpfulness votes
    pub helpful_votes: u32,
    /// Response from plugin author
    pub author_response: Option<String>,
}

impl PluginReview {
    pub fn new(
        plugin_name: impl Into<String>,
        user_id: impl Into<String>,
        rating: u8,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Self> {
        if rating == 0 || rating > 5 {
            return Err(CleanroomError::validation_error(
                "Rating must be between 1 and 5",
            ));
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            plugin_name: plugin_name.into(),
            user_id: user_id.into(),
            rating,
            title: title.into(),
            content: content.into(),
            created_at: chrono::Utc::now(),
            helpful_votes: 0,
            author_response: None,
        })
    }

    pub fn add_author_response(&mut self, response: impl Into<String>) {
        self.author_response = Some(response.into());
    }

    pub fn add_helpful_vote(&mut self) {
        self.helpful_votes += 1;
    }
}

/// Plugin discussion thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionThread {
    /// Thread ID
    pub id: String,
    /// Plugin name
    pub plugin_name: String,
    /// Thread title
    pub title: String,
    /// Thread author
    pub author_id: String,
    /// Thread posts
    pub posts: Vec<DiscussionPost>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Thread tags
    pub tags: Vec<String>,
    /// View count
    pub views: u32,
    /// Is thread locked
    pub locked: bool,
}

impl DiscussionThread {
    pub fn new(
        plugin_name: impl Into<String>,
        author_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            plugin_name: plugin_name.into(),
            title: title.into(),
            author_id: author_id.into(),
            posts: Vec::new(),
            created_at: now,
            last_activity: now,
            tags: Vec::new(),
            views: 0,
            locked: false,
        }
    }

    pub fn add_post(&mut self, post: DiscussionPost) {
        self.last_activity = chrono::Utc::now();
        self.posts.push(post);
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    pub fn increment_views(&mut self) {
        self.views += 1;
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

/// Discussion post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionPost {
    /// Post ID
    pub id: String,
    /// Author ID
    pub author_id: String,
    /// Post content
    pub content: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Edit timestamp
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Upvotes
    pub upvotes: u32,
    /// Downvotes
    pub downvotes: u32,
}

impl DiscussionPost {
    pub fn new(author_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            author_id: author_id.into(),
            content: content.into(),
            created_at: chrono::Utc::now(),
            edited_at: None,
            upvotes: 0,
            downvotes: 0,
        }
    }

    pub fn edit(&mut self, new_content: impl Into<String>) {
        self.content = new_content.into();
        self.edited_at = Some(chrono::Utc::now());
    }

    pub fn upvote(&mut self) {
        self.upvotes += 1;
    }

    pub fn downvote(&mut self) {
        self.downvotes += 1;
    }
}

/// Community manager for plugin ecosystem
pub struct CommunityManager {
    /// Plugin reviews
    reviews: HashMap<String, Vec<PluginReview>>,
    /// Discussion threads
    discussions: HashMap<String, Vec<DiscussionThread>>,
}

impl CommunityManager {
    pub fn new() -> Self {
        Self {
            reviews: HashMap::new(),
            discussions: HashMap::new(),
        }
    }

    /// Add a review for a plugin
    pub fn add_review(&mut self, review: PluginReview) -> Result<()> {
        self.reviews
            .entry(review.plugin_name.clone())
            .or_default()
            .push(review);
        Ok(())
    }

    /// Get reviews for a plugin
    pub fn get_reviews(&self, plugin_name: &str) -> Vec<PluginReview> {
        self.reviews.get(plugin_name).cloned().unwrap_or_default()
    }

    /// Get average rating for a plugin
    pub fn get_average_rating(&self, plugin_name: &str) -> f64 {
        let reviews = self.get_reviews(plugin_name);
        if reviews.is_empty() {
            return 0.0;
        }

        let total: u32 = reviews.iter().map(|r| r.rating as u32).sum();
        total as f64 / reviews.len() as f64
    }

    /// Create a discussion thread
    pub fn create_discussion(&mut self, thread: DiscussionThread) -> Result<String> {
        let thread_id = thread.id.clone();
        self.discussions
            .entry(thread.plugin_name.clone())
            .or_default()
            .push(thread);
        Ok(thread_id)
    }

    /// Get discussions for a plugin
    pub fn get_discussions(&self, plugin_name: &str) -> Vec<DiscussionThread> {
        self.discussions
            .get(plugin_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Get a specific discussion thread
    pub fn get_discussion(&self, plugin_name: &str, thread_id: &str) -> Option<DiscussionThread> {
        self.discussions
            .get(plugin_name)?
            .iter()
            .find(|t| t.id == thread_id)
            .cloned()
    }

    /// Add a post to a discussion thread
    pub fn add_discussion_post(
        &mut self,
        plugin_name: &str,
        thread_id: &str,
        post: DiscussionPost,
    ) -> Result<()> {
        if let Some(threads) = self.discussions.get_mut(plugin_name) {
            if let Some(thread) = threads.iter_mut().find(|t| t.id == thread_id) {
                if thread.locked {
                    return Err(CleanroomError::validation_error("Thread is locked"));
                }
                thread.add_post(post);
                return Ok(());
            }
        }

        Err(CleanroomError::validation_error("Thread not found"))
    }

    /// Get trending plugins based on community engagement
    pub fn get_trending_plugins(&self, metadata_list: &[PluginMetadata]) -> Vec<PluginMetadata> {
        let mut scored: Vec<_> = metadata_list
            .iter()
            .map(|metadata| {
                let reviews = self.get_reviews(&metadata.name);
                let discussions = self.get_discussions(&metadata.name);

                // Calculate engagement score
                let review_score = reviews.len() as f64 * 10.0;
                let discussion_score = discussions.len() as f64 * 5.0;
                let rating_score = self.get_average_rating(&metadata.name) * 20.0;

                let total_score = review_score + discussion_score + rating_score;

                (metadata.clone(), total_score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().map(|(metadata, _)| metadata).collect()
    }

    /// Get most helpful reviews
    pub fn get_most_helpful_reviews(&self, plugin_name: &str, limit: usize) -> Vec<PluginReview> {
        let mut reviews = self.get_reviews(plugin_name);
        reviews.sort_by(|a, b| b.helpful_votes.cmp(&a.helpful_votes));
        reviews.into_iter().take(limit).collect()
    }

    /// Get active discussions (sorted by recent activity)
    pub fn get_active_discussions(&self, plugin_name: &str, limit: usize) -> Vec<DiscussionThread> {
        let mut discussions = self.get_discussions(plugin_name);
        discussions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        discussions.into_iter().take(limit).collect()
    }
}

impl Default for CommunityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_review_creation() -> Result<()> {
        let review = PluginReview::new(
            "test-plugin",
            "user123",
            5,
            "Great plugin!",
            "This plugin works perfectly for my use case.",
        )?;

        assert_eq!(review.rating, 5);
        assert_eq!(review.plugin_name, "test-plugin");
        assert_eq!(review.helpful_votes, 0);

        Ok(())
    }

    #[test]
    fn test_invalid_rating() {
        let result = PluginReview::new(
            "test-plugin",
            "user123",
            6, // Invalid rating
            "Test",
            "Content",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_discussion_thread() {
        let mut thread = DiscussionThread::new("test-plugin", "user123", "How to use this plugin?");

        assert_eq!(thread.posts.len(), 0);
        assert!(!thread.locked);

        let post = DiscussionPost::new("user456", "Here's how you use it...");
        thread.add_post(post);

        assert_eq!(thread.posts.len(), 1);
    }

    #[test]
    fn test_community_manager() -> Result<()> {
        let mut manager = CommunityManager::new();

        let review = PluginReview::new("test-plugin", "user123", 5, "Excellent", "Great work")?;

        manager.add_review(review)?;

        let reviews = manager.get_reviews("test-plugin");
        assert_eq!(reviews.len(), 1);

        let avg_rating = manager.get_average_rating("test-plugin");
        assert_eq!(avg_rating, 5.0);

        Ok(())
    }

    #[test]
    fn test_locked_thread() -> Result<()> {
        let mut manager = CommunityManager::new();

        let mut thread = DiscussionThread::new("test-plugin", "user123", "Test thread");
        thread.lock();

        let thread_id = thread.id.clone();
        manager.create_discussion(thread)?;

        let post = DiscussionPost::new("user456", "Test post");
        let result = manager.add_discussion_post("test-plugin", &thread_id, post);

        assert!(result.is_err());

        Ok(())
    }
}
