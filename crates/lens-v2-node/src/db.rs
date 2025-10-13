use anyhow::{Context, Result};
use rocksdb::{DB, Options, IteratorMode};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// RocksDB database wrapper for lens-node persistence
#[derive(Clone)]
pub struct Database {
    db: Arc<DB>,
}

impl Database {
    /// Open or create a RocksDB database at the specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let path_ref = path.as_ref();
        tracing::info!("Opening RocksDB database at: {}", path_ref.display());

        let db = DB::open(&opts, path_ref)
            .context("Failed to open RocksDB database")?;

        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Store a value with the given key
    pub fn put<K, V>(&self, key: K, value: &V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        let serialized = serde_json::to_vec(value)
            .context("Failed to serialize value")?;

        self.db.put(key, serialized)
            .context("Failed to put value in database")?;

        Ok(())
    }

    /// Get a value by key
    pub fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: AsRef<[u8]>,
        V: for<'de> Deserialize<'de>,
    {
        match self.db.get(key)? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .context("Failed to deserialize value")?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Delete a value by key
    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<()> {
        self.db.delete(key)
            .context("Failed to delete value from database")?;
        Ok(())
    }

    /// Check if a key exists
    pub fn exists<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        Ok(self.db.get(key)?.is_some())
    }

    /// Iterate over all keys with a given prefix
    pub fn iter_prefix<V>(&self, prefix: &[u8]) -> Result<Vec<(String, V)>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let mut results = Vec::new();

        let iter = self.db.iterator(IteratorMode::From(prefix, rocksdb::Direction::Forward));

        for item in iter {
            let (key, value) = item?;

            // Check if key starts with prefix
            if !key.starts_with(prefix) {
                break;
            }

            let key_str = String::from_utf8_lossy(&key).to_string();
            let deserialized: V = serde_json::from_slice(&value)
                .context("Failed to deserialize value during iteration")?;

            results.push((key_str, deserialized));
        }

        Ok(results)
    }

    /// Get all values with a given prefix
    pub fn get_all_with_prefix<V>(&self, prefix: &str) -> Result<Vec<V>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let items = self.iter_prefix::<V>(prefix.as_bytes())?;
        Ok(items.into_iter().map(|(_, v)| v).collect())
    }

    /// Count items with a given prefix
    pub fn count_prefix(&self, prefix: &[u8]) -> Result<usize> {
        let mut count = 0;
        let iter = self.db.iterator(IteratorMode::From(prefix, rocksdb::Direction::Forward));

        for item in iter {
            let (key, _) = item?;
            if !key.starts_with(prefix) {
                break;
            }
            count += 1;
        }

        Ok(count)
    }
}

/// Key prefixes for different data types
pub mod prefixes {
    /// Release data: "release:{id}"
    pub const RELEASE: &str = "release:";

    /// Featured release data: "featured:{id}"
    pub const FEATURED: &str = "featured:";

    /// Featured release entity data: "featured_release:{id}"
    pub const FEATURED_RELEASE: &str = "featured_release:";

    /// Category data: "category:{id}"
    pub const CATEGORY: &str = "category:";

    /// Structure data: "structure:{id}"
    pub const STRUCTURE: &str = "structure:";

    /// Schema data: "schema:{id}"
    pub const SCHEMA: &str = "schema:";

    /// Authorization transaction data: "authorization:{uuid}"
    pub const AUTHORIZATION: &str = "authorization:";

    /// Delete transaction data: "delete_tx:{ubts_block_id}"
    pub const DELETE_TRANSACTION: &str = "delete_tx:";

    /// Update transaction data: "update_tx:{ubts_block_id}"
    pub const UPDATE_TRANSACTION: &str = "update_tx:";
}

/// Helper to create a key with prefix
pub fn make_key(prefix: &str, id: &str) -> String {
    format!("{}{}", prefix, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: String,
        name: String,
    }

    #[test]
    fn test_database_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db = Database::open(temp_dir.path())?;

        // Test put and get
        let test_data = TestData {
            id: "test1".to_string(),
            name: "Test Item".to_string(),
        };

        db.put("test:1", &test_data)?;
        let retrieved: Option<TestData> = db.get("test:1")?;
        assert_eq!(retrieved, Some(test_data.clone()));

        // Test exists
        assert!(db.exists("test:1")?);
        assert!(!db.exists("test:2")?);

        // Test delete
        db.delete("test:1")?;
        let deleted: Option<TestData> = db.get("test:1")?;
        assert_eq!(deleted, None);

        Ok(())
    }

    #[test]
    fn test_prefix_iteration() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db = Database::open(temp_dir.path())?;

        // Insert multiple items with same prefix
        for i in 1..=5 {
            let data = TestData {
                id: format!("item{}", i),
                name: format!("Test Item {}", i),
            };
            db.put(format!("release:{}", i), &data)?;
        }

        // Get all with prefix
        let all: Vec<TestData> = db.get_all_with_prefix("release:")?;
        assert_eq!(all.len(), 5);

        // Count with prefix
        let count = db.count_prefix(b"release:")?;
        assert_eq!(count, 5);

        Ok(())
    }
}
