//! Site Identity Management
//!
//! Provides unique identification for each Lens Node instance through SiteID and SiteKey.
//! This enables defederation and tracking of content origins across the distributed network.

use anyhow::{Context, Result};
use blake3::Hasher;
use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::db::Database;

/// Unique identifier for a Lens Node site
/// Format: "site-" + first 16 hex chars of Blake3 hash
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteId(String);

impl SiteId {
    /// Generate a new SiteID from hostname, timestamp, and random salt
    pub fn generate() -> Self {
        let hostname = hostname::get()
            .unwrap_or_else(|_| "unknown".into())
            .to_string_lossy()
            .to_string();

        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        let mut rng = rand::thread_rng();
        let mut salt = [0u8; 32];
        rng.fill_bytes(&mut salt);

        let mut hasher = Hasher::new();
        hasher.update(hostname.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&salt);

        let hash = hasher.finalize();
        let hex = hex::encode(&hash.as_bytes()[..8]); // First 8 bytes = 16 hex chars

        Self(format!("site-{}", hex))
    }

    /// Parse a SiteID from a string
    pub fn from_string(s: String) -> Result<Self> {
        if !s.starts_with("site-") {
            anyhow::bail!("SiteID must start with 'site-'");
        }
        if s.len() != 21 { // "site-" (5) + 16 hex chars
            anyhow::bail!("SiteID must be 21 characters long");
        }
        Ok(Self(s))
    }

    /// Get the string representation of the SiteID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ed25519 key pair for site signing and verification
#[derive(Clone)]
pub struct SiteKey {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl SiteKey {
    /// Generate a new ed25519 key pair
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create from existing signing key bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            anyhow::bail!("Signing key must be 32 bytes");
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Get the signing key as bytes
    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Get the verifying key as bytes
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Get the public key in base58 format for API responses
    pub fn public_key_base58(&self) -> String {
        bs58::encode(self.verifying_key.as_bytes()).into_string()
    }
}

/// Complete site identity including ID, keys, and optional friendly name
#[derive(Clone)]
pub struct SiteIdentity {
    pub site_id: SiteId,
    pub site_key: SiteKey,
    pub site_name: Option<String>,
}

impl SiteIdentity {
    /// Initialize or load site identity from database
    /// If no identity exists, generates a new one
    pub async fn initialize(db: &Database, site_name: Option<String>) -> Result<Self> {
        // Try to load existing identity
        if let Some(identity) = Self::load(db).await? {
            tracing::info!("Loaded existing site identity: {}", identity.site_id);
            return Ok(identity);
        }

        // Generate new identity
        let site_id = SiteId::generate();
        let site_key = SiteKey::generate();

        tracing::info!("Generated new site identity: {}", site_id);

        let identity = Self {
            site_id,
            site_key,
            site_name,
        };

        // Persist to database
        identity.save(db).await?;

        Ok(identity)
    }

    /// Load site identity from database
    async fn load(db: &Database) -> Result<Option<Self>> {
        // Load SiteID
        let site_id_str: Option<String> = db.get("site:id")?;
        let site_id = match site_id_str {
            Some(s) => SiteId::from_string(s)?,
            None => return Ok(None),
        };

        // Load private key
        let private_key_bytes: Option<Vec<u8>> = db.get("site:private_key")?;
        let site_key = match private_key_bytes {
            Some(bytes) => SiteKey::from_bytes(&bytes)?,
            None => return Ok(None),
        };

        // Load optional site name
        let site_name: Option<String> = db.get("site:name")?;

        Ok(Some(Self {
            site_id,
            site_key,
            site_name,
        }))
    }

    /// Save site identity to database
    async fn save(&self, db: &Database) -> Result<()> {
        db.put("site:id", &self.site_id.as_str())?;
        db.put("site:private_key", &self.site_key.signing_key_bytes().to_vec())?;

        if let Some(ref name) = self.site_name {
            db.put("site:name", name)?;
        }

        Ok(())
    }

    /// Get site info for API response
    pub fn to_info_response(&self, version: &str) -> SiteInfoResponse {
        SiteInfoResponse {
            site_id: self.site_id.as_str().to_string(),
            public_key: self.site_key.public_key_base58(),
            site_name: self.site_name.clone(),
            version: version.to_string(),
        }
    }
}

/// Response structure for GET /api/v1/site/info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInfoResponse {
    pub site_id: String,
    pub public_key: String,
    pub site_name: Option<String>,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_site_id_generation() {
        let id1 = SiteId::generate();
        let id2 = SiteId::generate();

        // Should be different
        assert_ne!(id1, id2);

        // Should have correct format
        assert!(id1.as_str().starts_with("site-"));
        assert_eq!(id1.as_str().len(), 21); // "site-" + 16 hex chars
    }

    #[test]
    fn test_site_id_from_string() {
        let valid_id = "site-a3f2e8b9c1d4f5e6";
        let id = SiteId::from_string(valid_id.to_string()).unwrap();
        assert_eq!(id.as_str(), valid_id);

        // Invalid: wrong prefix
        assert!(SiteId::from_string("node-abc123".to_string()).is_err());

        // Invalid: wrong length
        assert!(SiteId::from_string("site-abc".to_string()).is_err());
    }

    #[test]
    fn test_site_key_generation() {
        let key1 = SiteKey::generate();
        let key2 = SiteKey::generate();

        // Should be different
        assert_ne!(key1.signing_key_bytes(), key2.signing_key_bytes());
        assert_ne!(key1.verifying_key_bytes(), key2.verifying_key_bytes());

        // Public key should be base58 encoded
        let pk = key1.public_key_base58();
        assert!(!pk.is_empty());
    }

    #[test]
    fn test_site_key_from_bytes() {
        let original = SiteKey::generate();
        let bytes = original.signing_key_bytes();

        let restored = SiteKey::from_bytes(&bytes).unwrap();

        // Should have same keys
        assert_eq!(original.signing_key_bytes(), restored.signing_key_bytes());
        assert_eq!(original.verifying_key_bytes(), restored.verifying_key_bytes());
    }

    #[tokio::test]
    async fn test_site_identity_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();

        // First initialization should create new identity
        let identity1 = SiteIdentity::initialize(&db, Some("Test Node".to_string()))
            .await
            .unwrap();

        assert!(identity1.site_id.as_str().starts_with("site-"));
        assert_eq!(identity1.site_name, Some("Test Node".to_string()));

        // Second initialization should load existing identity
        let identity2 = SiteIdentity::initialize(&db, Some("Different Name".to_string()))
            .await
            .unwrap();

        assert_eq!(identity1.site_id, identity2.site_id);
        assert_eq!(identity1.site_key.signing_key_bytes(), identity2.site_key.signing_key_bytes());
        // Note: site_name from DB takes precedence
        assert_eq!(identity2.site_name, Some("Test Node".to_string()));
    }

    #[tokio::test]
    async fn test_site_identity_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();

        let identity = SiteIdentity::initialize(&db, None).await.unwrap();

        // Verify stored in database
        let stored_id: String = db.get("site:id").unwrap().unwrap();
        assert_eq!(stored_id, identity.site_id.as_str());

        let stored_key: Vec<u8> = db.get("site:private_key").unwrap().unwrap();
        assert_eq!(stored_key, identity.site_key.signing_key_bytes().to_vec());
    }

    #[tokio::test]
    async fn test_site_info_response() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();

        let identity = SiteIdentity::initialize(&db, Some("My Lens Node".to_string()))
            .await
            .unwrap();

        let response = identity.to_info_response("0.5.7");

        assert!(response.site_id.starts_with("site-"));
        assert!(!response.public_key.is_empty());
        assert_eq!(response.site_name, Some("My Lens Node".to_string()));
        assert_eq!(response.version, "0.5.7");
    }
}
