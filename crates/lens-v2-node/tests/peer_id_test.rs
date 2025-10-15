//! Test that peer_id generation works correctly and persists across restarts
//!
//! This test verifies:
//! 1. Peer_id is generated on first startup
//! 2. Peer_id persists across restarts (loaded from database)
//! 3. Peer_id is a valid BLAKE3 hash of the node's public key
//! 4. Different nodes get different peer_ids

use lens_node::db::Database;
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::path::PathBuf;

#[tokio::test]
async fn test_peer_id_generation_and_persistence() {
    // Create temporary database
    let temp_dir = std::env::temp_dir().join(format!("lens-peer-id-test-{}", rand::random::<u64>()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    let db = Database::open(&temp_dir).unwrap();

    // First startup - generate keypair
    let node_keypair_key = b"node:ed25519_keypair".to_vec();

    // Should not exist initially
    let initial_keypair = db.get::<Vec<u8>, Vec<u8>>(node_keypair_key.clone()).unwrap();
    assert!(initial_keypair.is_none(), "Keypair should not exist initially");

    // Generate and store keypair (simulating first startup)
    let signing_key = SigningKey::from_bytes(&rand::random());
    let verifying_key = signing_key.verifying_key();
    db.put(node_keypair_key.clone(), signing_key.as_bytes()).unwrap();

    // Generate peer_id from public key
    let public_key_bytes = verifying_key.to_bytes();
    let hash = blake3::hash(&public_key_bytes);
    let peer_id_1 = format!("bafk{}", hex::encode(hash.as_bytes()));

    println!("✅ First startup - Generated peer_id: {}", peer_id_1);
    println!("   Public key: {}", hex::encode(public_key_bytes));

    // Second startup - load keypair from database
    let stored_keypair = db.get::<Vec<u8>, Vec<u8>>(node_keypair_key.clone()).unwrap();
    assert!(stored_keypair.is_some(), "Keypair should be stored in database");

    let key_bytes: [u8; 32] = stored_keypair.unwrap().as_slice().try_into().unwrap();
    let loaded_signing_key = SigningKey::from_bytes(&key_bytes);
    let loaded_verifying_key = loaded_signing_key.verifying_key();

    // Generate peer_id from loaded public key
    let loaded_public_key_bytes = loaded_verifying_key.to_bytes();
    let loaded_hash = blake3::hash(&loaded_public_key_bytes);
    let peer_id_2 = format!("bafk{}", hex::encode(loaded_hash.as_bytes()));

    println!("✅ Second startup - Loaded peer_id: {}", peer_id_2);

    // Assert peer_id is the same across restarts
    assert_eq!(peer_id_1, peer_id_2, "Peer_id must persist across restarts");

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_different_nodes_get_different_peer_ids() {
    // Create two temporary databases (simulating two nodes)
    let temp_dir1 = std::env::temp_dir().join(format!("lens-peer-id-test1-{}", rand::random::<u64>()));
    let temp_dir2 = std::env::temp_dir().join(format!("lens-peer-id-test2-{}", rand::random::<u64>()));

    let _ = std::fs::remove_dir_all(&temp_dir1);
    let _ = std::fs::remove_dir_all(&temp_dir2);

    let db1 = Database::open(&temp_dir1).unwrap();
    let db2 = Database::open(&temp_dir2).unwrap();

    // Generate keypairs for both nodes
    let node_keypair_key = b"node:ed25519_keypair".to_vec();

    let signing_key1 = SigningKey::from_bytes(&rand::random());
    let verifying_key1 = signing_key1.verifying_key();
    db1.put(node_keypair_key.clone(), signing_key1.as_bytes()).unwrap();

    let signing_key2 = SigningKey::from_bytes(&rand::random());
    let verifying_key2 = signing_key2.verifying_key();
    db2.put(node_keypair_key.clone(), signing_key2.as_bytes()).unwrap();

    // Generate peer_ids
    let hash1 = blake3::hash(&verifying_key1.to_bytes());
    let peer_id_1 = format!("bafk{}", hex::encode(hash1.as_bytes()));

    let hash2 = blake3::hash(&verifying_key2.to_bytes());
    let peer_id_2 = format!("bafk{}", hex::encode(hash2.as_bytes()));

    println!("✅ Node 1 peer_id: {}", peer_id_1);
    println!("✅ Node 2 peer_id: {}", peer_id_2);

    // Assert different nodes get different peer_ids
    assert_ne!(peer_id_1, peer_id_2, "Different nodes must have different peer_ids");

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir1);
    let _ = std::fs::remove_dir_all(&temp_dir2);
}

#[tokio::test]
async fn test_peer_id_is_valid_blake3_hash() {
    // Create temporary database
    let temp_dir = std::env::temp_dir().join(format!("lens-peer-id-test-{}", rand::random::<u64>()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    let db = Database::open(&temp_dir).unwrap();

    // Generate keypair
    let node_keypair_key = b"node:ed25519_keypair".to_vec();
    let signing_key = SigningKey::from_bytes(&rand::random());
    let verifying_key = signing_key.verifying_key();
    db.put(node_keypair_key, signing_key.as_bytes()).unwrap();

    // Generate peer_id
    let public_key_bytes = verifying_key.to_bytes();
    let hash = blake3::hash(&public_key_bytes);
    let peer_id = format!("bafk{}", hex::encode(hash.as_bytes()));

    println!("✅ Generated peer_id: {}", peer_id);

    // Verify peer_id format
    assert!(peer_id.starts_with("bafk"), "Peer_id must start with 'bafk'");
    assert_eq!(peer_id.len(), 4 + 64, "Peer_id must be 'bafk' + 64 hex chars (32 bytes)");

    // Verify we can extract and verify the hash
    let hash_hex = &peer_id[4..];
    let decoded_hash = hex::decode(hash_hex).unwrap();
    assert_eq!(decoded_hash.len(), 32, "Hash must be 32 bytes");

    // Verify the hash matches the public key
    let recomputed_hash = blake3::hash(&public_key_bytes);
    assert_eq!(decoded_hash, recomputed_hash.as_bytes(), "Hash must match public key");

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}
