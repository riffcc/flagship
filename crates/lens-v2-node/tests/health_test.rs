use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt; // for `oneshot`

// Import the routes module to get access to create_test_app
use lens_node::routes::create_test_app;

/// Test that the health endpoint returns 200 OK
#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test that the health endpoint returns valid JSON
#[tokio::test]
async fn test_health_endpoint_returns_json() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("status").is_some());
    assert_eq!(json["status"], "healthy");
}

/// Test that the health endpoint includes version information
#[tokio::test]
async fn test_health_endpoint_includes_version() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("version").is_some());
    // Version should be a non-empty string
    assert!(!json["version"].as_str().unwrap().is_empty());
}
