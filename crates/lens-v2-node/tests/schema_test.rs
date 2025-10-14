use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt; // for `oneshot`

// Import the routes module to get access to create_test_app
use lens_node::routes::create_test_app;

/// Test that the schemas list endpoint works
#[tokio::test]
async fn test_list_schemas() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("schemas").is_some());

    let schemas = json["schemas"].as_array().unwrap();
    assert!(schemas.len() > 0);
    assert!(schemas.iter().any(|s| s.as_str() == Some("Release")));
}

/// Test getting the latest version of a schema
#[tokio::test]
async fn test_get_latest_schema() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas/Release")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Release");
    assert_eq!(json["version"]["major"], 1);
    assert_eq!(json["version"]["minor"], 0);
    assert_eq!(json["version"]["patch"], 0);
    assert!(json.get("schema").is_some());
}

/// Test getting all versions of a schema
#[tokio::test]
async fn test_get_schema_versions() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas/Release/versions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["schema_name"], "Release");
    assert!(json.get("versions").is_some());

    let versions = json["versions"].as_array().unwrap();
    assert!(versions.len() > 0);
    assert!(versions.contains(&Value::String("1.0.0".to_string())));
}

/// Test getting a specific schema version
#[tokio::test]
async fn test_get_specific_schema_version() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas/Release/versions/1.0.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Release");
    assert_eq!(json["version"]["major"], 1);
    assert_eq!(json["version"]["minor"], 0);
    assert_eq!(json["version"]["patch"], 0);
}

/// Test getting a non-existent schema
#[tokio::test]
async fn test_get_nonexistent_schema() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas/NonExistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test getting a non-existent version of an existing schema
#[tokio::test]
async fn test_get_nonexistent_version() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas/Release/versions/99.0.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test invalid version format
#[tokio::test]
async fn test_invalid_version_format() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/schemas/Release/versions/invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
