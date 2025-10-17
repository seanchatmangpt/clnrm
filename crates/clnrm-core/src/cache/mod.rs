//! Cache module for change-aware test execution
//!
//! Provides file hashing and cache management to skip unchanged test scenarios,
//! enabling 10x faster iteration by only rerunning tests when their configs change.
//!
//! ## Architecture
//! Pipeline: Render → Hash → Load cache → Compare → Run (if changed) → Update cache
//!
//! ## Cache Structure
//! File: `~/.clnrm/cache/hashes.json`
//! ```json
//! {
//!   "version": "1.0.0",
//!   "hashes": {
//!     "tests/api.clnrm.toml": "abc123...",
//!     "tests/db.clnrm.toml": "def456..."
//!   },
//!   "last_updated": "2025-10-17T12:34:56Z"
//! }
//! ```
//!
//! ## London School TDD Design
//!
//! The cache subsystem follows London School TDD principles:
//! - **Trait-based abstraction**: `Cache` trait defines collaboration contract
//! - **Mockable interface**: Supports test doubles for behavior verification
//! - **Multiple backends**: FileCache (persistent), MemoryCache (testing)
//! - **Interaction testing**: Focus on how components collaborate

pub mod cache_trait;
pub mod file_cache;
pub mod hash;
pub mod memory_cache;

pub use cache_trait::{BoxedCache, Cache, CacheStats};
pub use file_cache::FileCache;
pub use memory_cache::MemoryCache;

// Legacy alias for backward compatibility
pub use file_cache::FileCache as CacheManager;
