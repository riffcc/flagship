//! DHT Encryption Module
//!
//! Provides encryption/decryption for DHT stored values using ChaCha20-Poly1305.
//!
//! ## Encryption Scheme
//!
//! - **Algorithm**: ChaCha20-Poly1305 (authenticated encryption)
//! - **Key Derivation**: BLAKE3(SiteKey || salt || "lens:dht:v1")
//! - **Nonce**: 96-bit random nonce (stored with ciphertext)
//! - **Format**: `[nonce:12][ciphertext][tag:16]`
//!
//! ## Site Modes
//!
//! - **Normal Mode**: DHT values are shareable, encryption is optional
//! - **Enterprise Mode**: DHT values are private, encryption is required
//!
//! ## Key Management
//!
//! - SiteKey is generated on first run and stored in RocksDB
//! - Key is derived from SiteKey using BLAKE3 with a salt
//! - Salt is randomly generated and stored with encrypted data

use anyhow::{Context, Result, bail};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Size of the nonce in bytes (96 bits)
const NONCE_SIZE: usize = 12;

/// Size of the authentication tag in bytes (128 bits)
const TAG_SIZE: usize = 16;

/// Encryption salt stored in database
const ENCRYPTION_SALT_KEY: &str = "site:encryption_salt";

/// Site key stored in database (32 bytes)
const SITE_KEY_KEY: &str = "site:key";

/// Site mode stored in database
const SITE_MODE_KEY: &str = "site:mode";

/// Site operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SiteMode {
    /// Normal mode - DHT shareable, encryption optional
    Normal,
    /// Enterprise mode - DHT private, encryption required
    Enterprise,
}

impl SiteMode {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(SiteMode::Normal),
            "enterprise" => Ok(SiteMode::Enterprise),
            _ => bail!("Invalid site mode: {}. Must be 'normal' or 'enterprise'", s),
        }
    }

    /// Check if encryption is required for this mode
    pub fn requires_encryption(&self) -> bool {
        match self {
            SiteMode::Normal => false,
            SiteMode::Enterprise => true,
        }
    }
}

impl Default for SiteMode {
    fn default() -> Self {
        SiteMode::Normal
    }
}

/// DHT encryption context
pub struct DHTEncryption {
    site_key: [u8; 32],
    salt: [u8; 16],
    mode: SiteMode,
}

impl DHTEncryption {
    /// Initialize from database, generating keys if they don't exist
    pub fn init_or_generate(db: &crate::db::Database, mode: SiteMode) -> Result<Self> {
        // Get or generate site key
        let site_key: [u8; 32] = match db.get::<_, Vec<u8>>(SITE_KEY_KEY)? {
            Some(key) => {
                if key.len() != 32 {
                    bail!("Invalid site key length: expected 32 bytes, got {}", key.len());
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&key);
                tracing::info!("Loaded existing SiteKey from database");
                arr
            }
            None => {
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                db.put(SITE_KEY_KEY, &key.to_vec())?;
                tracing::info!("Generated new SiteKey and stored in database");
                key
            }
        };

        // Get or generate encryption salt
        let salt: [u8; 16] = match db.get::<_, Vec<u8>>(ENCRYPTION_SALT_KEY)? {
            Some(s) => {
                if s.len() != 16 {
                    bail!("Invalid encryption salt length: expected 16 bytes, got {}", s.len());
                }
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&s);
                tracing::info!("Loaded existing encryption salt from database");
                arr
            }
            None => {
                let mut s = [0u8; 16];
                OsRng.fill_bytes(&mut s);
                db.put(ENCRYPTION_SALT_KEY, &s.to_vec())?;
                tracing::info!("Generated new encryption salt and stored in database");
                s
            }
        };

        // Store site mode
        db.put(SITE_MODE_KEY, &mode)?;
        tracing::info!("Site mode set to: {:?}", mode);

        Ok(Self {
            site_key,
            salt,
            mode,
        })
    }

    /// Load from database (must already exist)
    pub fn load(db: &crate::db::Database) -> Result<Self> {
        let site_key: [u8; 32] = match db.get::<_, Vec<u8>>(SITE_KEY_KEY)? {
            Some(key) => {
                if key.len() != 32 {
                    bail!("Invalid site key length: expected 32 bytes, got {}", key.len());
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&key);
                arr
            }
            None => bail!("Site key not found in database. Run init first."),
        };

        let salt: [u8; 16] = match db.get::<_, Vec<u8>>(ENCRYPTION_SALT_KEY)? {
            Some(s) => {
                if s.len() != 16 {
                    bail!("Invalid encryption salt length: expected 16 bytes, got {}", s.len());
                }
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&s);
                arr
            }
            None => bail!("Encryption salt not found in database. Run init first."),
        };

        let mode: SiteMode = db.get(SITE_MODE_KEY)?.unwrap_or_default();

        Ok(Self {
            site_key,
            salt,
            mode,
        })
    }

    /// Get the current site mode
    pub fn mode(&self) -> SiteMode {
        self.mode
    }

    /// Derive encryption key from site key and salt
    fn derive_key(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.site_key);
        hasher.update(&self.salt);
        hasher.update(b"lens:dht:v1");
        *hasher.finalize().as_bytes()
    }

    /// Encrypt data for DHT storage
    ///
    /// Returns: [nonce:12][ciphertext][tag:16]
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = self.derive_key();
        let cipher = ChaCha20Poly1305::new(&key.into());

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data from DHT storage
    ///
    /// Expects: [nonce:12][ciphertext][tag:16]
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < NONCE_SIZE + TAG_SIZE {
            bail!(
                "Encrypted data too short: expected at least {} bytes, got {}",
                NONCE_SIZE + TAG_SIZE,
                encrypted.len()
            );
        }

        let key = self.derive_key();
        let cipher = ChaCha20Poly1305::new(&key.into());

        // Extract nonce
        let nonce = Nonce::from_slice(&encrypted[..NONCE_SIZE]);

        // Extract ciphertext (includes tag)
        let ciphertext = &encrypted[NONCE_SIZE..];

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Get the site key as hex (for sharing in normal mode)
    pub fn site_key_hex(&self) -> String {
        hex::encode(self.site_key)
    }

    /// Get the encryption salt as hex
    pub fn salt_hex(&self) -> String {
        hex::encode(self.salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (crate::db::Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = crate::db::Database::open(temp_dir.path()).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_site_mode_parsing() {
        assert_eq!(SiteMode::from_str("normal").unwrap(), SiteMode::Normal);
        assert_eq!(SiteMode::from_str("Normal").unwrap(), SiteMode::Normal);
        assert_eq!(SiteMode::from_str("NORMAL").unwrap(), SiteMode::Normal);
        assert_eq!(SiteMode::from_str("enterprise").unwrap(), SiteMode::Enterprise);
        assert_eq!(SiteMode::from_str("Enterprise").unwrap(), SiteMode::Enterprise);
        assert!(SiteMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_site_mode_requires_encryption() {
        assert!(!SiteMode::Normal.requires_encryption());
        assert!(SiteMode::Enterprise.requires_encryption());
    }

    #[test]
    fn test_init_or_generate_creates_keys() {
        let (db, _temp) = create_test_db();

        let enc = DHTEncryption::init_or_generate(&db, SiteMode::Normal).unwrap();

        // Should have generated keys
        assert_ne!(enc.site_key, [0u8; 32]);
        assert_ne!(enc.salt, [0u8; 16]);
        assert_eq!(enc.mode, SiteMode::Normal);
    }

    #[test]
    fn test_init_or_generate_loads_existing_keys() {
        let (db, _temp) = create_test_db();

        // First init
        let enc1 = DHTEncryption::init_or_generate(&db, SiteMode::Normal).unwrap();
        let key1 = enc1.site_key;
        let salt1 = enc1.salt;

        // Second init should load same keys
        let enc2 = DHTEncryption::init_or_generate(&db, SiteMode::Enterprise).unwrap();
        assert_eq!(enc2.site_key, key1);
        assert_eq!(enc2.salt, salt1);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (db, _temp) = create_test_db();
        let enc = DHTEncryption::init_or_generate(&db, SiteMode::Enterprise).unwrap();

        let plaintext = b"Hello, World! This is a test message.";

        // Encrypt
        let encrypted = enc.encrypt(plaintext).unwrap();

        // Should be longer than plaintext (nonce + ciphertext + tag)
        assert!(encrypted.len() > plaintext.len());
        assert_eq!(encrypted.len(), NONCE_SIZE + plaintext.len() + TAG_SIZE);

        // Decrypt
        let decrypted = enc.decrypt(&encrypted).unwrap();

        // Should match original
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let (db, _temp) = create_test_db();
        let enc = DHTEncryption::init_or_generate(&db, SiteMode::Enterprise).unwrap();

        let plaintext = b"Same message";

        // Encrypt twice
        let encrypted1 = enc.encrypt(plaintext).unwrap();
        let encrypted2 = enc.encrypt(plaintext).unwrap();

        // Should be different due to random nonces
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to same plaintext
        assert_eq!(enc.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(enc.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_invalid_data_fails() {
        let (db, _temp) = create_test_db();
        let enc = DHTEncryption::init_or_generate(&db, SiteMode::Enterprise).unwrap();

        // Too short
        let too_short = vec![0u8; 10];
        assert!(enc.decrypt(&too_short).is_err());

        // Wrong key (corrupt data)
        let plaintext = b"Test message";
        let mut encrypted = enc.encrypt(plaintext).unwrap();
        encrypted[NONCE_SIZE] ^= 0xFF; // Corrupt first byte of ciphertext
        assert!(enc.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_site_key_hex() {
        let (db, _temp) = create_test_db();
        let enc = DHTEncryption::init_or_generate(&db, SiteMode::Normal).unwrap();

        let hex = enc.site_key_hex();
        assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars

        // Should be valid hex
        assert!(hex::decode(&hex).is_ok());
    }

    #[test]
    fn test_load_without_init_fails() {
        let (db, _temp) = create_test_db();

        // Should fail because no keys exist
        assert!(DHTEncryption::load(&db).is_err());
    }

    #[test]
    fn test_load_after_init() {
        let (db, _temp) = create_test_db();

        // Init first
        let enc1 = DHTEncryption::init_or_generate(&db, SiteMode::Enterprise).unwrap();

        // Load should work
        let enc2 = DHTEncryption::load(&db).unwrap();

        assert_eq!(enc2.site_key, enc1.site_key);
        assert_eq!(enc2.salt, enc1.salt);
        assert_eq!(enc2.mode, enc1.mode);
    }

    #[test]
    fn test_encryption_with_different_keys() {
        let (db1, _temp1) = create_test_db();
        let (db2, _temp2) = create_test_db();

        let enc1 = DHTEncryption::init_or_generate(&db1, SiteMode::Enterprise).unwrap();
        let enc2 = DHTEncryption::init_or_generate(&db2, SiteMode::Enterprise).unwrap();

        let plaintext = b"Secret message";

        // Encrypt with first key
        let encrypted = enc1.encrypt(plaintext).unwrap();

        // Decrypt with second key should fail
        assert!(enc2.decrypt(&encrypted).is_err());

        // Decrypt with first key should work
        assert_eq!(enc1.decrypt(&encrypted).unwrap(), plaintext);
    }
}
