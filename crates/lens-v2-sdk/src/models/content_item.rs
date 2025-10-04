use crate::schema::{SchemaDefinition, SchemaVersion, Versioned};
use super::{ContentType, Creator, License, Resource};
use super::metadata::MetadataContainer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal content item - base for all content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentItem {
    /// Unique identifier (hash/CID)
    pub id: String,

    /// Content type discriminator
    pub content_type: ContentType,

    /// Title/name
    pub title: String,

    /// Description/synopsis
    pub description: Option<String>,

    /// Creators/contributors
    pub creators: Vec<Creator>,

    /// Tags for categorization and search
    pub tags: Vec<String>,

    /// Primary language (ISO 639 code)
    pub language: Option<String>,

    /// License information
    pub license: Option<License>,

    /// Resources (files, thumbnails, etc.)
    pub resources: Vec<Resource>,

    /// Creation timestamp (ISO 8601)
    pub created_at: Option<String>,

    /// Last modified timestamp (ISO 8601)
    pub updated_at: Option<String>,

    /// Publication/release date (ISO 8601)
    pub published_at: Option<String>,

    /// Standard metadata (Dublin Core, Schema.org, etc.)
    #[serde(default)]
    pub metadata: MetadataContainer,

    /// Type-specific fields as JSON
    /// Allows each content type to have unique fields
    #[serde(default)]
    pub type_specific: serde_json::Value,

    /// Custom/arbitrary fields that don't fit elsewhere
    /// Allows complete extensibility without breaking schema
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,

    /// Schema version for this content item
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

impl ContentItem {
    /// Create a new minimal content item
    pub fn new(id: String, content_type: ContentType, title: String) -> Self {
        Self {
            id,
            content_type,
            title,
            description: None,
            creators: Vec::new(),
            tags: Vec::new(),
            language: None,
            license: None,
            resources: Vec::new(),
            created_at: None,
            updated_at: None,
            published_at: None,
            metadata: MetadataContainer::default(),
            type_specific: serde_json::Value::Null,
            custom: HashMap::new(),
            schema_version: default_schema_version(),
        }
    }

    /// Builder: Add description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Builder: Add creator
    pub fn with_creator(mut self, creator: Creator) -> Self {
        self.creators.push(creator);
        self
    }

    /// Builder: Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Builder: Add resource
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    /// Builder: Set type-specific data
    pub fn with_type_specific(mut self, data: serde_json::Value) -> Self {
        self.type_specific = data;
        self
    }

    /// Builder: Add custom field
    pub fn with_custom_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }
}

impl Versioned for ContentItem {
    fn schema_name() -> &'static str {
        "ContentItem"
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
                "title": "ContentItem",
                "description": "Universal content item supporting all content types with extensible metadata",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Unique identifier (hash/CID)"
                    },
                    "content_type": {
                        "type": "string",
                        "description": "Content type discriminator",
                        "examples": ["movie", "music_album", "scientific_paper", "ai_model"]
                    },
                    "title": {
                        "type": "string",
                        "description": "Title/name of the content"
                    },
                    "description": {
                        "type": ["string", "null"],
                        "description": "Description or synopsis"
                    },
                    "creators": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "role": {"type": "string"},
                                "identifier": {"type": ["string", "null"]}
                            },
                            "required": ["name", "role"]
                        },
                        "description": "Creators and contributors"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags for categorization and search"
                    },
                    "language": {
                        "type": ["string", "null"],
                        "description": "Primary language (ISO 639 code)"
                    },
                    "license": {
                        "type": ["object", "null"],
                        "description": "License information"
                    },
                    "resources": {
                        "type": "array",
                        "description": "Associated resources (files, thumbnails, etc.)"
                    },
                    "created_at": {
                        "type": ["string", "null"],
                        "format": "date-time",
                        "description": "Creation timestamp (ISO 8601)"
                    },
                    "updated_at": {
                        "type": ["string", "null"],
                        "format": "date-time",
                        "description": "Last modified timestamp (ISO 8601)"
                    },
                    "published_at": {
                        "type": ["string", "null"],
                        "format": "date-time",
                        "description": "Publication/release date (ISO 8601)"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Standard metadata (Dublin Core, Schema.org, DataCite, etc.)"
                    },
                    "type_specific": {
                        "description": "Type-specific fields as JSON (extensible)"
                    },
                    "custom": {
                        "type": "object",
                        "description": "Custom/arbitrary fields (fully extensible)"
                    },
                    "schema_version": {
                        "type": "string",
                        "description": "Schema version",
                        "default": "1.0.0"
                    }
                },
                "required": ["id", "content_type", "title"]
            }),
            migration_hints: None,
        }
    }

    fn supports_version(version: &SchemaVersion) -> bool {
        version.major == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreatorRole, metadata::StandardMetadata};

    #[test]
    fn test_content_item_creation() {
        let item = ContentItem::new(
            "test123".to_string(),
            ContentType::Movie,
            "Test Movie".to_string(),
        );

        assert_eq!(item.id, "test123");
        assert_eq!(item.content_type, ContentType::Movie);
        assert_eq!(item.title, "Test Movie");
        assert_eq!(item.schema_version, "1.0.0");
    }

    #[test]
    fn test_content_item_builder() {
        let item = ContentItem::new(
            "test456".to_string(),
            ContentType::MusicAlbum,
            "Test Album".to_string(),
        )
        .with_description("A test music album".to_string())
        .with_creator(Creator {
            name: "Test Artist".to_string(),
            role: CreatorRole::Musician,
            identifier: None,
        })
        .with_tag("rock".to_string())
        .with_tag("2024".to_string())
        .with_custom_field("custom_rating".to_string(), serde_json::json!(4.5));

        assert_eq!(item.description, Some("A test music album".to_string()));
        assert_eq!(item.creators.len(), 1);
        assert_eq!(item.tags.len(), 2);
        assert_eq!(item.custom.get("custom_rating"), Some(&serde_json::json!(4.5)));
    }

    #[test]
    fn test_content_item_serialization() {
        let item = ContentItem::new(
            "test789".to_string(),
            ContentType::ScientificPaper,
            "Research Paper".to_string(),
        )
        .with_description("Important research".to_string());

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ContentItem = serde_json::from_str(&json).unwrap();

        assert_eq!(item, deserialized);
    }

    #[test]
    fn test_content_item_with_metadata() {
        let mut item = ContentItem::new(
            "test_dc".to_string(),
            ContentType::Dataset,
            "Scientific Dataset".to_string(),
        );

        item.metadata.add(StandardMetadata::DublinCore {
            title: Some("Scientific Dataset".to_string()),
            creator: Some(vec!["Dr. Smith".to_string()]),
            subject: Some(vec!["Science".to_string()]),
            description: Some("A dataset".to_string()),
            publisher: Some("University".to_string()),
            contributor: None,
            date: Some("2024-01-01".to_string()),
            dc_type: Some("Dataset".to_string()),
            format: Some("application/json".to_string()),
            identifier: Some("doi:10.1234/test".to_string()),
            source: None,
            language: Some("en".to_string()),
            relation: None,
            coverage: None,
            rights: Some("CC-BY-4.0".to_string()),
        });

        assert!(item.metadata.get_dublin_core().is_some());

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ContentItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }

    #[test]
    fn test_schema_definition() {
        let schema = ContentItem::schema_definition();
        assert_eq!(schema.name, "ContentItem");
        assert_eq!(schema.version, SchemaVersion::new(1, 0, 0));
        assert!(schema.schema.get("properties").is_some());
    }

    #[test]
    fn test_diverse_content_types() {
        let types_to_test = vec![
            (ContentType::Movie, "Inception"),
            (ContentType::TvSeries, "Breaking Bad"),
            (ContentType::MusicAlbum, "Dark Side of the Moon"),
            (ContentType::Podcast, "Serial"),
            (ContentType::Book, "1984"),
            (ContentType::ScientificPaper, "On the Origin of Species"),
            (ContentType::Course, "Introduction to Rust"),
            (ContentType::Dataset, "Climate Data 2024"),
            (ContentType::AiModel, "GPT-4"),
            (ContentType::ContainerImage, "nginx:latest"),
            (ContentType::Photo, "Sunset.jpg"),
            (ContentType::MuseumArtifact, "Ancient Vase"),
            (ContentType::Backup, "System Backup 2024-01-01"),
            (ContentType::Custom("NFT".to_string()), "Bored Ape #1234"),
        ];

        for (content_type, title) in types_to_test {
            let item = ContentItem::new(
                format!("id_{}", title.replace(' ', "_")),
                content_type.clone(),
                title.to_string(),
            );

            let json = serde_json::to_string(&item).unwrap();
            let deserialized: ContentItem = serde_json::from_str(&json).unwrap();
            assert_eq!(item.content_type, deserialized.content_type);
            assert_eq!(item.title, deserialized.title);
        }
    }
}
