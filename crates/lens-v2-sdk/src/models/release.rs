use crate::schema::{SchemaDefinition, SchemaVersion, Versioned};
use serde::{Deserialize, Serialize};

/// A release in the Lens ecosystem (album, movie, TV series, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Release {
    /// Unique identifier (hash)
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Creator/artist/director
    pub creator: Option<String>,

    /// Release year
    pub year: Option<u32>,

    /// Category ID
    pub category_id: String,

    /// Content Identifier (CID) for thumbnail
    pub thumbnail_cid: Option<String>,

    /// Description/synopsis
    pub description: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Schema version this release was serialized with
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

impl Versioned for Release {
    fn schema_name() -> &'static str {
        "Release"
    }

    fn schema_version() -> SchemaVersion {
        SchemaVersion::new(1, 0, 0)
    }

    fn schema_definition() -> SchemaDefinition {
        SchemaDefinition {
            name: Self::schema_name().to_string(),
            version: Self::schema_version(),
            schema: serde_json::json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "Release",
                "description": "A release in the Lens ecosystem (album, movie, TV series, etc.)",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Unique identifier (hash)"
                    },
                    "title": {
                        "type": "string",
                        "description": "Human-readable title"
                    },
                    "creator": {
                        "type": ["string", "null"],
                        "description": "Creator/artist/director"
                    },
                    "year": {
                        "type": ["integer", "null"],
                        "description": "Release year"
                    },
                    "category_id": {
                        "type": "string",
                        "description": "Category identifier"
                    },
                    "thumbnail_cid": {
                        "type": ["string", "null"],
                        "description": "Content Identifier (CID) for thumbnail"
                    },
                    "description": {
                        "type": ["string", "null"],
                        "description": "Description or synopsis"
                    },
                    "tags": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Tags for categorization"
                    },
                    "schema_version": {
                        "type": "string",
                        "description": "Schema version this release was serialized with",
                        "default": "1.0.0"
                    }
                },
                "required": ["id", "title", "category_id", "tags"]
            }),
            migration_hints: None,
        }
    }

    fn supports_version(version: &SchemaVersion) -> bool {
        // We support any 1.x.x version
        version.major == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_serialization() {
        let release = Release {
            id: "test123".to_string(),
            title: "Test Release".to_string(),
            creator: Some("Test Artist".to_string()),
            year: Some(2024),
            category_id: "music".to_string(),
            thumbnail_cid: Some("QmTest123".to_string()),
            description: Some("A test release".to_string()),
            tags: vec!["test".to_string(), "demo".to_string()],
            schema_version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&release).unwrap();
        let deserialized: Release = serde_json::from_str(&json).unwrap();

        assert_eq!(release, deserialized);
    }

    #[test]
    fn test_schema_definition() {
        let schema = Release::schema_definition();
        assert_eq!(schema.name, "Release");
        assert_eq!(schema.version, SchemaVersion::new(1, 0, 0));
        assert!(schema.schema.get("properties").is_some());
    }

    #[test]
    fn test_version_support() {
        assert!(Release::supports_version(&SchemaVersion::new(1, 0, 0)));
        assert!(Release::supports_version(&SchemaVersion::new(1, 1, 0)));
        assert!(Release::supports_version(&SchemaVersion::new(1, 2, 5)));
        assert!(!Release::supports_version(&SchemaVersion::new(2, 0, 0)));
    }

    #[test]
    fn test_deserialize_without_schema_version() {
        // Test backwards compatibility - old data without schema_version field
        let json = r#"{
            "id": "test123",
            "title": "Test Release",
            "creator": "Test Artist",
            "year": 2024,
            "category_id": "music",
            "thumbnail_cid": "QmTest123",
            "description": "A test release",
            "tags": ["test", "demo"]
        }"#;

        let release: Release = serde_json::from_str(json).unwrap();
        assert_eq!(release.schema_version, "1.0.0"); // Default value
    }
}
