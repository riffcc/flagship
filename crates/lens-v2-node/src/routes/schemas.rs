use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use lens_v2_sdk::{Release, SchemaRegistry, SchemaVersion, Versioned};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Application state containing the schema registry
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<SchemaRegistry>,
}

/// Response for listing all schemas
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemasListResponse {
    pub schemas: Vec<String>,
}

/// Response for listing versions of a schema
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaVersionsResponse {
    pub schema_name: String,
    pub versions: Vec<String>,
}

/// Handler to list all registered schemas
pub async fn list_schemas(State(state): State<AppState>) -> impl IntoResponse {
    match state.registry.list_schemas() {
        Ok(schemas) => {
            let response = SchemasListResponse { schemas };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = serde_json::json!({ "error": e });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Handler to get all versions of a specific schema
pub async fn get_schema_versions(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
) -> impl IntoResponse {
    match state.registry.get_all_versions(&schema_name) {
        Ok(versions) => {
            let versions: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
            let response = SchemaVersionsResponse {
                schema_name,
                versions,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = serde_json::json!({ "error": e });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Handler to get a specific schema version
pub async fn get_schema(
    State(state): State<AppState>,
    Path((schema_name, version_str)): Path<(String, String)>,
) -> impl IntoResponse {
    let version = match SchemaVersion::parse(&version_str) {
        Ok(v) => v,
        Err(e) => {
            let error = serde_json::json!({ "error": format!("Invalid version: {}", e) });
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };

    match state.registry.get(&schema_name, &version) {
        Ok(Some(schema)) => {
            let json = serde_json::to_value(&schema).unwrap();
            (StatusCode::OK, Json(json))
        }
        Ok(None) => {
            let error = serde_json::json!({
                "error": format!("Schema {} version {} not found", schema_name, version_str)
            });
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = serde_json::json!({ "error": e });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Handler to get the latest version of a schema
pub async fn get_latest_schema(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
) -> impl IntoResponse {
    match state.registry.get_latest(&schema_name) {
        Ok(Some(schema)) => {
            let json = serde_json::to_value(&schema).unwrap();
            (StatusCode::OK, Json(json))
        }
        Ok(None) => {
            let error = serde_json::json!({
                "error": format!("Schema {} not found", schema_name)
            });
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = serde_json::json!({ "error": e });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Initialize the schema registry with built-in schemas
pub fn initialize_registry() -> SchemaRegistry {
    let registry = SchemaRegistry::new();

    // Register built-in schemas
    if let Err(e) = registry.register(Release::schema_definition()) {
        eprintln!("Failed to register Release schema: {}", e);
    }

    // Future: Add more schemas here as they're implemented
    // registry.register(Track::schema_definition()).ok();
    // registry.register(Account::schema_definition()).ok();

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_registry() {
        let registry = initialize_registry();
        let schemas = registry.list_schemas().unwrap();
        assert!(schemas.contains(&"Release".to_string()));
    }

    #[test]
    fn test_registry_has_release_schema() {
        let registry = initialize_registry();
        let latest = registry.get_latest("Release").unwrap();
        assert!(latest.is_some());
        let schema = latest.unwrap();
        assert_eq!(schema.version, SchemaVersion::new(1, 0, 0));
    }
}
