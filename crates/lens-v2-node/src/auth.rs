use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

const MAX_TIMESTAMP_SKEW_MS: i64 = 300000; // 5 minutes

/// Verify Ed25519 signature on a request
///
/// The signature should be computed as: sign(timestamp + ":" + body)
/// Headers required:
/// - X-Public-Key: ed25519p/{hex_public_key}
/// - X-Signature: {hex_signature}
/// - X-Timestamp: {unix_timestamp_ms}
pub fn verify_request_signature(
    headers: &HeaderMap,
    body: &Bytes,
) -> Result<String, axum::response::Response> {
    // Extract public key
    let public_key_header = headers
        .get("X-Public-Key")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing X-Public-Key header"
                })),
            )
                .into_response()
        })?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid X-Public-Key header encoding"
                })),
            )
                .into_response()
        })?;

    // Extract signature
    let signature_hex = headers
        .get("X-Signature")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing X-Signature header"
                })),
            )
                .into_response()
        })?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid X-Signature header encoding"
                })),
            )
                .into_response()
        })?;

    // Extract timestamp
    let timestamp_str = headers
        .get("X-Timestamp")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing X-Timestamp header"
                })),
            )
                .into_response()
        })?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid X-Timestamp header encoding"
                })),
            )
                .into_response()
        })?;

    // Parse timestamp
    let timestamp: i64 = timestamp_str.parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid timestamp format"
            })),
        )
            .into_response()
    })?;

    // Verify timestamp is recent (prevent replay attacks)
    let now = chrono::Utc::now().timestamp_millis();
    let time_diff = (now - timestamp).abs();
    if time_diff > MAX_TIMESTAMP_SKEW_MS {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Request timestamp too old or too far in future"
            })),
        )
            .into_response());
    }

    // Parse public key (format: ed25519p/{hex})
    let public_key_hex = if let Some(hex) = public_key_header.strip_prefix("ed25519p/") {
        hex
    } else {
        tracing::error!("Public key header '{}' missing ed25519p/ prefix", public_key_header);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Public key must be in format ed25519p/{hex}"
            })),
        )
            .into_response());
    };

    // Decode public key from hex
    let public_key_bytes = hex::decode(public_key_hex).map_err(|e| {
        tracing::error!("Failed to decode public key hex '{}': {:?}", public_key_hex, e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid public key hex encoding"
            })),
        )
            .into_response()
    })?;

    tracing::debug!("Decoded public key: {} bytes", public_key_bytes.len());

    // Create verifying key
    let verifying_key = VerifyingKey::try_from(public_key_bytes.as_slice()).map_err(|e| {
        tracing::error!("Failed to create VerifyingKey from {} bytes: {:?}", public_key_bytes.len(), e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid Ed25519 public key"
            })),
        )
            .into_response()
    })?;

    // Decode signature from hex
    let signature_bytes = hex::decode(signature_hex).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid signature hex encoding"
            })),
        )
            .into_response()
    })?;

    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid Ed25519 signature"
            })),
        )
            .into_response()
    })?;

    // Reconstruct the message that was signed: timestamp:body
    let message = format!("{}:{}", timestamp_str, String::from_utf8_lossy(body));

    // Verify signature
    verifying_key.verify(message.as_bytes(), &signature).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid signature - request verification failed"
            })),
        )
            .into_response()
    })?;

    // Signature is valid, return the public key
    Ok(public_key_header.to_string())
}
