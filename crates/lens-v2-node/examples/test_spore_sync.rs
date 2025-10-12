use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=========================================");
    println!("SPORE Block Sync Test with Ed25519");
    println!("=========================================\n");

    // Generate Ed25519 keypair
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let public_key_hex = hex::encode(verifying_key.to_bytes());
    let public_key_full = format!("ed25119p/{}", public_key_hex);

    println!("Generated Ed25519 Keypair:");
    println!("  Public Key: {}", public_key_full);
    println!("  Private Key: {}", hex::encode(signing_key.to_bytes()));
    println!();

    // Wait for nodes to be ready
    println!("Waiting for nodes to start...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client = reqwest::Client::new();

    // Step 1: Authorize admin on node 0
    println!("Step 1: Authorizing admin on Node 0");
    let timestamp = chrono::Utc::now().timestamp_millis();
    let body = json!({
        "publicKey": public_key_full
    });
    let body_str = serde_json::to_string(&body)?;

    // Sign: timestamp:body
    let message = format!("{}:{}", timestamp, body_str);
    let signature = signing_key.sign(message.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    let response = client
        .post("http://localhost:6002/api/v1/admin/authorize")
        .header("Content-Type", "application/json")
        .header("X-Public-Key", &public_key_full)
        .header("X-Signature", signature_hex)
        .header("X-Timestamp", timestamp.to_string())
        .body(body_str.clone())
        .send()
        .await?;

    println!("  Response: {}\n", response.text().await?);

    // Wait for admin sync
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Step 2: Create release on node 0
    println!("Step 2: Creating release on Node 0");
    let timestamp = chrono::Utc::now().timestamp_millis();
    let body = json!({
        "name": "SPORE Test Release",
        "categoryId": "cat-test",
        "categorySlug": "test",
        "contentCID": "QmTestSPORE123456",
        "version": "1.0.0"
    });
    let body_str = serde_json::to_string(&body)?;

    let message = format!("{}:{}", timestamp, body_str);
    let signature = signing_key.sign(message.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    let response = client
        .post("http://localhost:6002/api/v1/releases")
        .header("Content-Type", "application/json")
        .header("X-Public-Key", &public_key_full)
        .header("X-Signature", signature_hex)
        .header("X-Timestamp", timestamp.to_string())
        .body(body_str)
        .send()
        .await?;

    let release_response = response.text().await?;
    println!("  Response: {}\n", release_response);

    let release_json: serde_json::Value = serde_json::from_str(&release_response)?;
    let release_id = release_json["id"].as_str().unwrap_or("null");
    println!("  Release ID: {}\n", release_id);

    // Wait for SPORE propagation
    println!("Waiting for SPORE propagation...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Step 3: Check if release synced to other nodes
    println!("Step 3: Checking synchronization\n");

    for (node_num, port) in [(1, 6003), (2, 6004)] {
        let url = format!("http://localhost:{}/api/v1/releases", port);
        let response = client.get(&url).send().await?;
        let releases: serde_json::Value = response.json().await?;

        let has_release = releases.as_array()
            .map(|arr| arr.iter().any(|r| r["id"] == release_id))
            .unwrap_or(false);

        if has_release {
            println!("  ✅ Release synced to Node {}", node_num);
        } else {
            println!("  ❌ Release NOT synced to Node {}", node_num);
            println!("     Releases on Node {}: {}", node_num, releases);
        }
    }

    // Step 4: Check sync status
    println!("\nStep 4: Final sync status\n");
    for port in [6002, 6003, 6004] {
        let url = format!("http://localhost:{}/api/v1/ready", port);
        let response = client.get(&url).send().await?;
        let status: serde_json::Value = response.json().await?;
        println!("  Node {}: {}", port, status);
    }

    println!("\n=========================================");
    println!("Test Complete!");
    println!("=========================================\n");

    Ok(())
}
