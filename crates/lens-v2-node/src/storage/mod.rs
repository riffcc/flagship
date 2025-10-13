//! LensStorage - Abstract storage interface for Lens Node
//!
//! This module provides a storage abstraction that can be backed by:
//! - Citadel DHT (distributed, P2P)
//! - RocksDB (local cache)
//! - Hybrid (DHT with RocksDB write-through cache)

pub mod dht_storage;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use dht_storage::{DHTStorage, DHTMetrics};

/// Release metadata stored in DHT
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub id: String,
    pub title: String,
    pub creator: Option<String>,
    pub year: Option<u32>,
    pub category_id: String,
    pub thumbnail_cid: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub schema_version: String,
}

/// Featured release metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaturedMetadata {
    pub release_id: String,
    pub featured_at: u64, // Unix timestamp
    pub priority: u32,
}

/// Category metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Abstract storage interface for Lens Node operations
#[async_trait]
pub trait LensStorage: Send + Sync {
    // ========== Release Operations ==========

    /// Store a release
    async fn put_release(&mut self, release: &ReleaseMetadata) -> Result<()>;

    /// Get a release by ID
    async fn get_release(&self, id: &str) -> Result<Option<ReleaseMetadata>>;

    /// Delete a release by ID
    async fn delete_release(&mut self, id: &str) -> Result<()>;

    /// Check if a release exists
    async fn has_release(&self, id: &str) -> Result<bool>;

    /// List all releases (paginated)
    async fn list_releases(&self, offset: usize, limit: usize) -> Result<Vec<ReleaseMetadata>>;

    // ========== Featured Operations ==========

    /// Add a featured release
    async fn add_featured(&mut self, featured: &FeaturedMetadata) -> Result<()>;

    /// Get all featured releases (sorted by priority)
    async fn list_featured(&self) -> Result<Vec<FeaturedMetadata>>;

    /// Remove a featured release
    async fn remove_featured(&mut self, release_id: &str) -> Result<()>;

    // ========== Category Operations ==========

    /// Store a category
    async fn put_category(&mut self, category: &CategoryMetadata) -> Result<()>;

    /// Get a category by ID
    async fn get_category(&self, id: &str) -> Result<Option<CategoryMetadata>>;

    /// List all categories
    async fn list_categories(&self) -> Result<Vec<CategoryMetadata>>;

    // ========== Query Operations ==========

    /// Search releases by title (substring match)
    async fn search_releases_by_title(&self, query: &str) -> Result<Vec<ReleaseMetadata>>;

    /// Get releases by category
    async fn get_releases_by_category(&self, category_id: &str) -> Result<Vec<ReleaseMetadata>>;

    /// Get releases by tag
    async fn get_releases_by_tag(&self, tag: &str) -> Result<Vec<ReleaseMetadata>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Test Fixtures ==========

    fn create_test_release(id: &str, title: &str) -> ReleaseMetadata {
        ReleaseMetadata {
            id: id.to_string(),
            title: title.to_string(),
            creator: Some("Test Artist".to_string()),
            year: Some(2024),
            category_id: "music".to_string(),
            thumbnail_cid: Some("QmTest123".to_string()),
            description: Some("A test release".to_string()),
            tags: vec!["test".to_string(), "demo".to_string()],
            schema_version: "1.0.0".to_string(),
        }
    }

    fn create_test_featured(release_id: &str, priority: u32) -> FeaturedMetadata {
        FeaturedMetadata {
            release_id: release_id.to_string(),
            featured_at: 1234567890,
            priority,
        }
    }

    fn create_test_category(id: &str, name: &str) -> CategoryMetadata {
        CategoryMetadata {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Test category".to_string()),
        }
    }

    // ========== TDD Tests for LensStorage Trait ==========

    // We'll implement a MockStorage to test the trait interface
    struct MockStorage {
        releases: std::collections::HashMap<String, ReleaseMetadata>,
        featured: std::collections::HashMap<String, FeaturedMetadata>,
        categories: std::collections::HashMap<String, CategoryMetadata>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                releases: std::collections::HashMap::new(),
                featured: std::collections::HashMap::new(),
                categories: std::collections::HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl LensStorage for MockStorage {
        async fn put_release(&mut self, release: &ReleaseMetadata) -> Result<()> {
            self.releases.insert(release.id.clone(), release.clone());
            Ok(())
        }

        async fn get_release(&self, id: &str) -> Result<Option<ReleaseMetadata>> {
            Ok(self.releases.get(id).cloned())
        }

        async fn delete_release(&mut self, id: &str) -> Result<()> {
            self.releases.remove(id);
            Ok(())
        }

        async fn has_release(&self, id: &str) -> Result<bool> {
            Ok(self.releases.contains_key(id))
        }

        async fn list_releases(&self, offset: usize, limit: usize) -> Result<Vec<ReleaseMetadata>> {
            let mut releases: Vec<_> = self.releases.values().cloned().collect();
            releases.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(releases.into_iter().skip(offset).take(limit).collect())
        }

        async fn add_featured(&mut self, featured: &FeaturedMetadata) -> Result<()> {
            self.featured.insert(featured.release_id.clone(), featured.clone());
            Ok(())
        }

        async fn list_featured(&self) -> Result<Vec<FeaturedMetadata>> {
            let mut featured: Vec<_> = self.featured.values().cloned().collect();
            featured.sort_by(|a, b| b.priority.cmp(&a.priority));
            Ok(featured)
        }

        async fn remove_featured(&mut self, release_id: &str) -> Result<()> {
            self.featured.remove(release_id);
            Ok(())
        }

        async fn put_category(&mut self, category: &CategoryMetadata) -> Result<()> {
            self.categories.insert(category.id.clone(), category.clone());
            Ok(())
        }

        async fn get_category(&self, id: &str) -> Result<Option<CategoryMetadata>> {
            Ok(self.categories.get(id).cloned())
        }

        async fn list_categories(&self) -> Result<Vec<CategoryMetadata>> {
            let mut categories: Vec<_> = self.categories.values().cloned().collect();
            categories.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(categories)
        }

        async fn search_releases_by_title(&self, query: &str) -> Result<Vec<ReleaseMetadata>> {
            let results: Vec<_> = self.releases.values()
                .filter(|r| r.title.to_lowercase().contains(&query.to_lowercase()))
                .cloned()
                .collect();
            Ok(results)
        }

        async fn get_releases_by_category(&self, category_id: &str) -> Result<Vec<ReleaseMetadata>> {
            let results: Vec<_> = self.releases.values()
                .filter(|r| r.category_id == category_id)
                .cloned()
                .collect();
            Ok(results)
        }

        async fn get_releases_by_tag(&self, tag: &str) -> Result<Vec<ReleaseMetadata>> {
            let results: Vec<_> = self.releases.values()
                .filter(|r| r.tags.contains(&tag.to_string()))
                .cloned()
                .collect();
            Ok(results)
        }
    }

    // ========== Release Tests ==========

    #[tokio::test]
    async fn test_put_and_get_release() -> Result<()> {
        let mut storage = MockStorage::new();
        let release = create_test_release("test1", "Test Release");

        storage.put_release(&release).await?;
        let retrieved = storage.get_release("test1").await?;

        assert_eq!(retrieved, Some(release));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_nonexistent_release() -> Result<()> {
        let storage = MockStorage::new();
        let result = storage.get_release("nonexistent").await?;

        assert_eq!(result, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_release() -> Result<()> {
        let mut storage = MockStorage::new();
        let release = create_test_release("test1", "Test Release");

        storage.put_release(&release).await?;
        assert!(storage.has_release("test1").await?);

        storage.delete_release("test1").await?;
        assert!(!storage.has_release("test1").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_has_release() -> Result<()> {
        let mut storage = MockStorage::new();
        let release = create_test_release("test1", "Test Release");

        assert!(!storage.has_release("test1").await?);
        storage.put_release(&release).await?;
        assert!(storage.has_release("test1").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_releases_pagination() -> Result<()> {
        let mut storage = MockStorage::new();

        // Add 5 releases
        for i in 0..5 {
            let release = create_test_release(&format!("test{}", i), &format!("Release {}", i));
            storage.put_release(&release).await?;
        }

        // Get first 2
        let page1 = storage.list_releases(0, 2).await?;
        assert_eq!(page1.len(), 2);

        // Get next 2
        let page2 = storage.list_releases(2, 2).await?;
        assert_eq!(page2.len(), 2);

        // Get last page
        let page3 = storage.list_releases(4, 2).await?;
        assert_eq!(page3.len(), 1);

        Ok(())
    }

    // ========== Featured Tests ==========

    #[tokio::test]
    async fn test_add_and_list_featured() -> Result<()> {
        let mut storage = MockStorage::new();
        let featured1 = create_test_featured("release1", 10);
        let featured2 = create_test_featured("release2", 20);

        storage.add_featured(&featured1).await?;
        storage.add_featured(&featured2).await?;

        let list = storage.list_featured().await?;
        assert_eq!(list.len(), 2);

        // Should be sorted by priority (descending)
        assert_eq!(list[0].priority, 20);
        assert_eq!(list[1].priority, 10);

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_featured() -> Result<()> {
        let mut storage = MockStorage::new();
        let featured = create_test_featured("release1", 10);

        storage.add_featured(&featured).await?;
        assert_eq!(storage.list_featured().await?.len(), 1);

        storage.remove_featured("release1").await?;
        assert_eq!(storage.list_featured().await?.len(), 0);

        Ok(())
    }

    // ========== Category Tests ==========

    #[tokio::test]
    async fn test_put_and_get_category() -> Result<()> {
        let mut storage = MockStorage::new();
        let category = create_test_category("music", "Music");

        storage.put_category(&category).await?;
        let retrieved = storage.get_category("music").await?;

        assert_eq!(retrieved, Some(category));
        Ok(())
    }

    #[tokio::test]
    async fn test_list_categories() -> Result<()> {
        let mut storage = MockStorage::new();

        storage.put_category(&create_test_category("music", "Music")).await?;
        storage.put_category(&create_test_category("movies", "Movies")).await?;
        storage.put_category(&create_test_category("books", "Books")).await?;

        let categories = storage.list_categories().await?;
        assert_eq!(categories.len(), 3);

        Ok(())
    }

    // ========== Query Tests ==========

    #[tokio::test]
    async fn test_search_releases_by_title() -> Result<()> {
        let mut storage = MockStorage::new();

        storage.put_release(&create_test_release("r1", "Dark Side of the Moon")).await?;
        storage.put_release(&create_test_release("r2", "The Dark Knight")).await?;
        storage.put_release(&create_test_release("r3", "Bright Stars")).await?;

        let results = storage.search_releases_by_title("dark").await?;
        assert_eq!(results.len(), 2); // "Dark Side" and "Dark Knight"

        Ok(())
    }

    #[tokio::test]
    async fn test_get_releases_by_category() -> Result<()> {
        let mut storage = MockStorage::new();

        let mut r1 = create_test_release("r1", "Album 1");
        r1.category_id = "music".to_string();

        let mut r2 = create_test_release("r2", "Album 2");
        r2.category_id = "music".to_string();

        let mut r3 = create_test_release("r3", "Movie 1");
        r3.category_id = "movies".to_string();

        storage.put_release(&r1).await?;
        storage.put_release(&r2).await?;
        storage.put_release(&r3).await?;

        let music_releases = storage.get_releases_by_category("music").await?;
        assert_eq!(music_releases.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_releases_by_tag() -> Result<()> {
        let mut storage = MockStorage::new();

        let mut r1 = create_test_release("r1", "Release 1");
        r1.tags = vec!["rock".to_string(), "classic".to_string()];

        let mut r2 = create_test_release("r2", "Release 2");
        r2.tags = vec!["rock".to_string(), "indie".to_string()];

        let mut r3 = create_test_release("r3", "Release 3");
        r3.tags = vec!["pop".to_string()];

        storage.put_release(&r1).await?;
        storage.put_release(&r2).await?;
        storage.put_release(&r3).await?;

        let rock_releases = storage.get_releases_by_tag("rock").await?;
        assert_eq!(rock_releases.len(), 2);

        Ok(())
    }
}
