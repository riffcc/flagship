use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status of the service
    pub status: String,
    /// Version information
    pub version: String,
}

/// Health check endpoint handler
///
/// Returns basic health status and version information
/// This endpoint is used for monitoring and load balancer health checks
pub async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    (StatusCode::OK, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_healthy() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
