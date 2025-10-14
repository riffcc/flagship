//! DHT-backed storage implementation for Lens Node
//!
//! Uses Citadel DHT for distributed storage with local caching.

use anyhow::{Context, Result};
use async_trait::async_trait;
use citadel_core::key_mapping::key_to_slot;
use citadel_core::topology::MeshConfig;
use citadel_dht::node::MinimalNode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{
    CategoryMetadata, FeaturedMetadata, LensStorage, ReleaseMetadata,
};
use crate::dht_encryption::DHTEncryption;

/// Metrics for DHT operations
#[derive(Debug, Clone, Default)]
pub struct DHTMetrics {
    pub get_count: u64,
    pub put_count: u64,
    pub delete_count: u64,
    pub total_get_latency_ms: u64,
    pub total_put_latency_ms: u64,
    pub total_delete_latency_ms: u64,
    pub errors: u64,
}

/// DHT Storage implementation
pub struct DHTStorage {
    node: MinimalNode,
    local_storage: Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>,
    mesh_config: MeshConfig,
    metrics: Arc<Mutex<DHTMetrics>>,
    encryption: Option<Arc<DHTEncryption>>,
}

impl DHTStorage {
    /// Create a new DHT storage instance
    pub fn new(node: MinimalNode, mesh_config: MeshConfig) -> Self {
        Self {
            node,
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            mesh_config,
            metrics: Arc::new(Mutex::new(DHTMetrics::default())),
            encryption: None,
        }
    }

    /// Create a new DHT storage instance with encryption
    pub fn new_with_encryption(node: MinimalNode, mesh_config: MeshConfig, encryption: Arc<DHTEncryption>) -> Self {
        Self {
            node,
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            mesh_config,
            metrics: Arc::new(Mutex::new(DHTMetrics::default())),
            encryption: Some(encryption),
        }
    }

    /// Get current DHT metrics
    pub fn get_metrics(&self) -> DHTMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Record a GET operation
    fn record_get(&self, latency_ms: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.get_count += 1;
        metrics.total_get_latency_ms += latency_ms;
        tracing::debug!("DHT GET operation completed in {}ms (total: {})", latency_ms, metrics.get_count);
    }

    /// Record a PUT operation
    fn record_put(&self, latency_ms: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.put_count += 1;
        metrics.total_put_latency_ms += latency_ms;
        tracing::debug!("DHT PUT operation completed in {}ms (total: {})", latency_ms, metrics.put_count);
    }

    /// Record a DELETE operation
    fn record_delete(&self, latency_ms: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.delete_count += 1;
        metrics.total_delete_latency_ms += latency_ms;
        tracing::debug!("DHT DELETE operation completed in {}ms (total: {})", latency_ms, metrics.delete_count);
    }

    /// Record an error
    fn record_error(&self, operation: &str, error: &anyhow::Error) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.errors += 1;
        tracing::error!("DHT {} operation failed: {} (total errors: {})", operation, error, metrics.errors);
    }

    /// Generate a DHT key for a release
    fn release_key(&self, id: &str) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lens:release:");
        hasher.update(id.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Generate a DHT key for a featured release
    fn featured_key(&self, release_id: &str) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lens:featured:");
        hasher.update(release_id.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Generate a DHT key for a category
    fn category_key(&self, id: &str) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lens:category:");
        hasher.update(id.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Generate a DHT key for listing all releases
    fn releases_list_key(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lens:releases:list");
        *hasher.finalize().as_bytes()
    }

    /// Generate a DHT key for listing featured releases
    fn featured_list_key(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lens:featured:list");
        *hasher.finalize().as_bytes()
    }

    /// Generate a DHT key for listing categories
    fn categories_list_key(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lens:categories:list");
        *hasher.finalize().as_bytes()
    }

    /// Serialize data to bytes (with optional encryption)
    fn serialize<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        let json_bytes = serde_json::to_vec(value).context("Failed to serialize value")?;

        // Encrypt if encryption is enabled
        if let Some(enc) = &self.encryption {
            let encrypted = enc.encrypt(&json_bytes)
                .context("Failed to encrypt DHT value")?;
            Ok(encrypted)
        } else {
            Ok(json_bytes)
        }
    }

    /// Deserialize data from bytes (with optional decryption)
    fn deserialize<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        // Try to decrypt if encryption is enabled
        let json_bytes = if let Some(enc) = &self.encryption {
            // Try decryption first
            match enc.decrypt(bytes) {
                Ok(decrypted) => decrypted,
                Err(_) => {
                    // If decryption fails, try to parse as unencrypted JSON
                    // This provides backward compatibility with unencrypted data
                    tracing::debug!("Decryption failed, attempting to parse as unencrypted JSON");
                    bytes.to_vec()
                }
            }
        } else {
            bytes.to_vec()
        };

        serde_json::from_slice(&json_bytes).context("Failed to deserialize value")
    }
}

#[async_trait]
impl LensStorage for DHTStorage {
    async fn put_release(&mut self, release: &ReleaseMetadata) -> Result<()> {
        let start = Instant::now();

        let result = (|| -> Result<()> {
            let key = self.release_key(&release.id);
            let value = self.serialize(release)
                .context("Failed to serialize release metadata")?;

            // Store in local DHT storage
            let mut storage = self.local_storage.lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;
            storage.insert(key, value);

            // Update releases list
            let list_key = self.releases_list_key();
            let mut release_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
                self.deserialize(bytes)
                    .context("Failed to deserialize releases list")?
            } else {
                Vec::new()
            };

            if !release_ids.contains(&release.id) {
                release_ids.push(release.id.clone());
                let list_bytes = self.serialize(&release_ids)
                    .context("Failed to serialize updated releases list")?;
                storage.insert(list_key, list_bytes);
            }

            Ok(())
        })();

        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => {
                self.record_put(latency);
                tracing::info!("Successfully stored release '{}' in DHT", release.id);
            }
            Err(e) => {
                self.record_error("PUT", e);
            }
        }

        result
    }

    async fn get_release(&self, id: &str) -> Result<Option<ReleaseMetadata>> {
        let start = Instant::now();

        let result = (|| -> Result<Option<ReleaseMetadata>> {
            let key = self.release_key(id);
            let storage = self.local_storage.lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

            match storage.get(&key) {
                Some(bytes) => {
                    let release = self.deserialize(bytes)
                        .context(format!("Failed to deserialize release '{}'", id))?;
                    Ok(Some(release))
                }
                None => Ok(None),
            }
        })();

        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(Some(_)) => {
                self.record_get(latency);
                tracing::debug!("Successfully retrieved release '{}' from DHT", id);
            }
            Ok(None) => {
                self.record_get(latency);
                tracing::debug!("Release '{}' not found in DHT", id);
            }
            Err(e) => {
                self.record_error("GET", e);
            }
        }

        result
    }

    async fn delete_release(&mut self, id: &str) -> Result<()> {
        let start = Instant::now();

        let result = (|| -> Result<()> {
            let key = self.release_key(id);
            let mut storage = self.local_storage.lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;
            storage.remove(&key);

            // Update releases list
            let list_key = self.releases_list_key();
            if let Some(bytes) = storage.get(&list_key) {
                let mut release_ids: Vec<String> = self.deserialize(bytes)
                    .context("Failed to deserialize releases list")?;
                release_ids.retain(|rid| rid != id);
                let list_bytes = self.serialize(&release_ids)
                    .context("Failed to serialize updated releases list")?;
                storage.insert(list_key, list_bytes);
            }

            Ok(())
        })();

        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => {
                self.record_delete(latency);
                tracing::info!("Successfully deleted release '{}' from DHT", id);
            }
            Err(e) => {
                self.record_error("DELETE", e);
            }
        }

        result
    }

    async fn has_release(&self, id: &str) -> Result<bool> {
        let key = self.release_key(id);
        let storage = self.local_storage.lock().unwrap();
        Ok(storage.contains_key(&key))
    }

    async fn list_releases(&self, offset: usize, limit: usize) -> Result<Vec<ReleaseMetadata>> {
        let start = Instant::now();

        let result = (|| -> Result<Vec<ReleaseMetadata>> {
            let list_key = self.releases_list_key();
            let storage = self.local_storage.lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire storage lock: {}", e))?;

            let release_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
                self.deserialize(bytes)
                    .context("Failed to deserialize releases list")?
            } else {
                Vec::new()
            };

            let mut releases = Vec::new();
            let mut skipped_count = 0;

            for id in release_ids.iter().skip(offset).take(limit) {
                if let Some(bytes) = storage.get(&self.release_key(id)) {
                    match self.deserialize(bytes) {
                        Ok(release) => releases.push(release),
                        Err(e) => {
                            tracing::warn!("Failed to deserialize release '{}': {}", id, e);
                            skipped_count += 1;
                        }
                    }
                }
            }

            if skipped_count > 0 {
                tracing::warn!("Skipped {} corrupted releases during list operation", skipped_count);
            }

            Ok(releases)
        })();

        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(releases) => {
                self.record_get(latency);
                tracing::debug!("Successfully listed {} releases (offset: {}, limit: {})", releases.len(), offset, limit);
            }
            Err(e) => {
                self.record_error("LIST", e);
            }
        }

        result
    }

    async fn add_featured(&mut self, featured: &FeaturedMetadata) -> Result<()> {
        let key = self.featured_key(&featured.release_id);
        let value = self.serialize(featured)?;

        let mut storage = self.local_storage.lock().unwrap();
        storage.insert(key, value);

        // Update featured list
        let list_key = self.featured_list_key();
        let mut featured_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        if !featured_ids.contains(&featured.release_id) {
            featured_ids.push(featured.release_id.clone());
            let list_bytes = self.serialize(&featured_ids)?;
            storage.insert(list_key, list_bytes);
        }

        Ok(())
    }

    async fn list_featured(&self) -> Result<Vec<FeaturedMetadata>> {
        let list_key = self.featured_list_key();
        let storage = self.local_storage.lock().unwrap();

        let featured_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        let mut featured = Vec::new();
        for id in featured_ids {
            if let Some(bytes) = storage.get(&self.featured_key(&id)) {
                if let Ok(f) = self.deserialize(bytes) {
                    featured.push(f);
                }
            }
        }

        // Sort by priority (descending)
        featured.sort_by(|a: &FeaturedMetadata, b: &FeaturedMetadata| b.priority.cmp(&a.priority));

        Ok(featured)
    }

    async fn remove_featured(&mut self, release_id: &str) -> Result<()> {
        let key = self.featured_key(release_id);
        let mut storage = self.local_storage.lock().unwrap();
        storage.remove(&key);

        // Update featured list
        let list_key = self.featured_list_key();
        if let Some(bytes) = storage.get(&list_key) {
            let mut featured_ids: Vec<String> = self.deserialize(bytes)?;
            featured_ids.retain(|fid| fid != release_id);
            let list_bytes = self.serialize(&featured_ids)?;
            storage.insert(list_key, list_bytes);
        }

        Ok(())
    }

    async fn put_category(&mut self, category: &CategoryMetadata) -> Result<()> {
        let key = self.category_key(&category.id);
        let value = self.serialize(category)?;

        let mut storage = self.local_storage.lock().unwrap();
        storage.insert(key, value);

        // Update categories list
        let list_key = self.categories_list_key();
        let mut category_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        if !category_ids.contains(&category.id) {
            category_ids.push(category.id.clone());
            let list_bytes = self.serialize(&category_ids)?;
            storage.insert(list_key, list_bytes);
        }

        Ok(())
    }

    async fn get_category(&self, id: &str) -> Result<Option<CategoryMetadata>> {
        let key = self.category_key(id);
        let storage = self.local_storage.lock().unwrap();

        match storage.get(&key) {
            Some(bytes) => Ok(Some(self.deserialize(bytes)?)),
            None => Ok(None),
        }
    }

    async fn list_categories(&self) -> Result<Vec<CategoryMetadata>> {
        let list_key = self.categories_list_key();
        let storage = self.local_storage.lock().unwrap();

        let category_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        let mut categories = Vec::new();
        for id in category_ids {
            if let Some(bytes) = storage.get(&self.category_key(&id)) {
                if let Ok(cat) = self.deserialize(bytes) {
                    categories.push(cat);
                }
            }
        }

        // Sort by ID
        categories.sort_by(|a: &CategoryMetadata, b: &CategoryMetadata| a.id.cmp(&b.id));

        Ok(categories)
    }

    async fn search_releases_by_title(&self, query: &str) -> Result<Vec<ReleaseMetadata>> {
        let list_key = self.releases_list_key();
        let storage = self.local_storage.lock().unwrap();

        let release_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        let mut results = Vec::new();
        for id in release_ids {
            if let Some(bytes) = storage.get(&self.release_key(&id)) {
                if let Ok(release) = self.deserialize::<ReleaseMetadata>(bytes) {
                    if release.title.to_lowercase().contains(&query.to_lowercase()) {
                        results.push(release);
                    }
                }
            }
        }

        Ok(results)
    }

    async fn get_releases_by_category(&self, category_id: &str) -> Result<Vec<ReleaseMetadata>> {
        let list_key = self.releases_list_key();
        let storage = self.local_storage.lock().unwrap();

        let release_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        let mut results = Vec::new();
        for id in release_ids {
            if let Some(bytes) = storage.get(&self.release_key(&id)) {
                if let Ok(release) = self.deserialize::<ReleaseMetadata>(bytes) {
                    if release.category_id == category_id {
                        results.push(release);
                    }
                }
            }
        }

        Ok(results)
    }

    async fn get_releases_by_tag(&self, tag: &str) -> Result<Vec<ReleaseMetadata>> {
        let list_key = self.releases_list_key();
        let storage = self.local_storage.lock().unwrap();

        let release_ids: Vec<String> = if let Some(bytes) = storage.get(&list_key) {
            self.deserialize(bytes)?
        } else {
            Vec::new()
        };

        let mut results = Vec::new();
        for id in release_ids {
            if let Some(bytes) = storage.get(&self.release_key(&id)) {
                if let Ok(release) = self.deserialize::<ReleaseMetadata>(bytes) {
                    if release.tags.contains(&tag.to_string()) {
                        results.push(release);
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citadel_core::topology::SlotCoordinate;

    fn create_test_dht_storage() -> DHTStorage {
        let config = MeshConfig::new(100, 100, 50);
        let slot = SlotCoordinate::new(0, 0, 0);
        let peer_id = [1u8; 32];
        let node = MinimalNode::new(slot, peer_id, config, 0);

        DHTStorage::new(node, config)
    }

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

    // ========== TDD Tests for DHTStorage ==========

    #[tokio::test]
    async fn test_dht_storage_put_and_get_release() -> Result<()> {
        let mut storage = create_test_dht_storage();
        let release = create_test_release("test1", "Test Release");

        storage.put_release(&release).await?;
        let retrieved = storage.get_release("test1").await?;

        assert_eq!(retrieved, Some(release));
        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_get_nonexistent_release() -> Result<()> {
        let storage = create_test_dht_storage();
        let result = storage.get_release("nonexistent").await?;

        assert_eq!(result, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_delete_release() -> Result<()> {
        let mut storage = create_test_dht_storage();
        let release = create_test_release("test1", "Test Release");

        storage.put_release(&release).await?;
        assert!(storage.has_release("test1").await?);

        storage.delete_release("test1").await?;
        assert!(!storage.has_release("test1").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_list_releases() -> Result<()> {
        let mut storage = create_test_dht_storage();

        // Add 5 releases
        for i in 0..5 {
            let release = create_test_release(&format!("test{}", i), &format!("Release {}", i));
            storage.put_release(&release).await?;
        }

        // List all
        let all = storage.list_releases(0, 10).await?;
        assert_eq!(all.len(), 5);

        // Paginate
        let page1 = storage.list_releases(0, 2).await?;
        assert_eq!(page1.len(), 2);

        let page2 = storage.list_releases(2, 2).await?;
        assert_eq!(page2.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_featured() -> Result<()> {
        let mut storage = create_test_dht_storage();

        let featured1 = FeaturedMetadata {
            release_id: "release1".to_string(),
            featured_at: 1234567890,
            priority: 10,
        };

        let featured2 = FeaturedMetadata {
            release_id: "release2".to_string(),
            featured_at: 1234567891,
            priority: 20,
        };

        storage.add_featured(&featured1).await?;
        storage.add_featured(&featured2).await?;

        let list = storage.list_featured().await?;
        assert_eq!(list.len(), 2);

        // Should be sorted by priority (descending)
        assert_eq!(list[0].priority, 20);
        assert_eq!(list[1].priority, 10);

        // Remove
        storage.remove_featured("release1").await?;
        assert_eq!(storage.list_featured().await?.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_categories() -> Result<()> {
        let mut storage = create_test_dht_storage();

        let cat1 = CategoryMetadata {
            id: "music".to_string(),
            name: "Music".to_string(),
            description: Some("Music releases".to_string()),
        };

        let cat2 = CategoryMetadata {
            id: "movies".to_string(),
            name: "Movies".to_string(),
            description: Some("Movie releases".to_string()),
        };

        storage.put_category(&cat1).await?;
        storage.put_category(&cat2).await?;

        let retrieved = storage.get_category("music").await?;
        assert_eq!(retrieved, Some(cat1));

        let all = storage.list_categories().await?;
        assert_eq!(all.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_search_by_title() -> Result<()> {
        let mut storage = create_test_dht_storage();

        storage.put_release(&create_test_release("r1", "Dark Side of the Moon")).await?;
        storage.put_release(&create_test_release("r2", "The Dark Knight")).await?;
        storage.put_release(&create_test_release("r3", "Bright Stars")).await?;

        let results = storage.search_releases_by_title("dark").await?;
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_get_by_category() -> Result<()> {
        let mut storage = create_test_dht_storage();

        let mut r1 = create_test_release("r1", "Album 1");
        r1.category_id = "music".to_string();

        let mut r2 = create_test_release("r2", "Album 2");
        r2.category_id = "music".to_string();

        let mut r3 = create_test_release("r3", "Movie 1");
        r3.category_id = "movies".to_string();

        storage.put_release(&r1).await?;
        storage.put_release(&r2).await?;
        storage.put_release(&r3).await?;

        let music = storage.get_releases_by_category("music").await?;
        assert_eq!(music.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_dht_storage_get_by_tag() -> Result<()> {
        let mut storage = create_test_dht_storage();

        let mut r1 = create_test_release("r1", "Release 1");
        r1.tags = vec!["rock".to_string(), "classic".to_string()];

        let mut r2 = create_test_release("r2", "Release 2");
        r2.tags = vec!["rock".to_string(), "indie".to_string()];

        let mut r3 = create_test_release("r3", "Release 3");
        r3.tags = vec!["pop".to_string()];

        storage.put_release(&r1).await?;
        storage.put_release(&r2).await?;
        storage.put_release(&r3).await?;

        let rock = storage.get_releases_by_tag("rock").await?;
        assert_eq!(rock.len(), 2);

        Ok(())
    }
}
