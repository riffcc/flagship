use axum::{extract::Json, response::IntoResponse};
use serde::Serialize;

/// Legacy structure type for backwards compatibility
/// Structures are deprecated in Lens V2 - use releases instead
#[derive(Debug, Clone, Serialize)]
pub struct Structure {
    pub id: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
}

/// GET /api/v1/structures - List structures (legacy endpoint)
/// Returns empty array as structures are deprecated in Lens V2
pub async fn list_structures() -> impl IntoResponse {
    tracing::debug!("Legacy structures endpoint called - returning empty array (use /releases instead)");

    // Return empty array - structures are deprecated in favor of releases
    Json(Vec::<Structure>::new())
}

/// GET /api/v1/structures/:id - Get structure by ID (legacy endpoint)
/// Returns 404 as structures are deprecated in Lens V2
pub async fn get_structure() -> impl IntoResponse {
    tracing::debug!("Legacy structure/:id endpoint called - returning empty (use /releases/:id instead)");

    // Return empty object
    Json(serde_json::json!({}))
}
