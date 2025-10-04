use axum::{extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

/// Content category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCategory {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "metadataSchema")]
    pub metadata_schema: Option<serde_json::Value>,
    #[serde(rename = "siteAddress")]
    pub site_address: String,
}

/// GET /api/v1/content-categories - List all content categories
pub async fn list_categories() -> impl IntoResponse {
    // Return standard categories for now
    // Use the same site address as configured in frontend
    let site_address = "zb2rhkfHMKY7nsrC6QYcuAi1imgAAUXwPM3WYCajL3Evxmq2w".to_string();

    let categories = vec![
        ContentCategory {
            id: "music".to_string(),
            name: "Music".to_string(),
            slug: "music".to_string(),
            metadata_schema: Some(serde_json::json!({
                "artist": "string",
                "album": "string",
                "trackMetadata": "string"
            })),
            site_address: site_address.clone(),
        },
        ContentCategory {
            id: "movies".to_string(),
            name: "Movies".to_string(),
            slug: "movies".to_string(),
            metadata_schema: Some(serde_json::json!({
                "director": "string",
                "releaseYear": "string",
                "duration": "string",
                "classification": "string"
            })),
            site_address: site_address.clone(),
        },
        ContentCategory {
            id: "tv-shows".to_string(),
            name: "TV Shows".to_string(),
            slug: "tv-shows".to_string(),
            metadata_schema: Some(serde_json::json!({
                "seasons": "number",
                "episodes": "number",
                "releaseYear": "string"
            })),
            site_address: site_address.clone(),
        },
        ContentCategory {
            id: "books".to_string(),
            name: "Books".to_string(),
            slug: "books".to_string(),
            metadata_schema: Some(serde_json::json!({
                "author": "string",
                "isbn": "string",
                "publisher": "string",
                "publicationYear": "string"
            })),
            site_address: site_address.clone(),
        },
        ContentCategory {
            id: "audiobooks".to_string(),
            name: "Audiobooks".to_string(),
            slug: "audiobooks".to_string(),
            metadata_schema: Some(serde_json::json!({
                "narrator": "string",
                "author": "string",
                "duration": "string"
            })),
            site_address: site_address.clone(),
        },
        ContentCategory {
            id: "games".to_string(),
            name: "Games".to_string(),
            slug: "games".to_string(),
            metadata_schema: Some(serde_json::json!({
                "platform": "string",
                "developer": "string",
                "releaseYear": "string"
            })),
            site_address,
        },
    ];

    Json(categories)
}
