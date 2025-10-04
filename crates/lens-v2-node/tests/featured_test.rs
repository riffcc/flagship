use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt; // for `oneshot`

// Import the routes module to get access to create_test_app
#[path = "../src/routes/mod.rs"]
mod routes;

/// Test that we can update a featured release's promoted status
#[tokio::test]
async fn test_update_featured_release_promoted() {
    let app = routes::create_test_app();

    // First, get the list of featured releases
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/featured-releases")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let featured_releases: Vec<Value> = serde_json::from_slice(&body).unwrap();

    // Skip if no releases
    if featured_releases.is_empty() {
        return;
    }

    let first_release = &featured_releases[0];
    let release_id = first_release["id"].as_str().unwrap();

    // Update the promoted status
    let update_data = json!({
        "id": release_id,
        "promoted": true,
        "tags": ["music", "featured"],
        "priority": 100
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/featured-releases/{}", release_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], release_id);
}

/// Test that updating featured release with invalid ID returns 404
#[tokio::test]
async fn test_update_featured_release_not_found() {
    let app = routes::create_test_app();

    let update_data = json!({
        "id": "00000000-0000-0000-0000-000000000000",
        "promoted": true,
        "tags": ["test"],
        "priority": 50
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/featured-releases/00000000-0000-0000-0000-000000000000")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test that we can update tags on a featured release
#[tokio::test]
async fn test_update_featured_release_tags() {
    let app = routes::create_test_app();

    // First, get the list of featured releases
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/featured-releases")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let featured_releases: Vec<Value> = serde_json::from_slice(&body).unwrap();

    // Skip if no releases
    if featured_releases.is_empty() {
        return;
    }

    let first_release = &featured_releases[0];
    let release_id = first_release["id"].as_str().unwrap();

    // Update with new tags
    let new_tags = vec!["custom-tag-1", "custom-tag-2", "trending"];
    let update_data = json!({
        "id": release_id,
        "promoted": false,
        "tags": new_tags,
        "priority": 75
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/featured-releases/{}", release_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
