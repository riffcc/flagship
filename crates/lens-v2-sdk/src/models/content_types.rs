use serde::{Deserialize, Serialize};

/// Content type discriminator - extensible enum for all possible content types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    // Media - Video
    Movie,
    TvSeries,
    TvEpisode,
    Video,              // YouTube, Vimeo, etc.
    VideoClip,

    // Media - Audio
    MusicAlbum,
    MusicTrack,
    Playlist,
    Podcast,
    PodcastEpisode,
    Audiobook,
    AudioProduction,    // DAW projects, stems, etc.

    // Publications
    Book,
    Ebook,
    Magazine,
    Comic,
    ScientificPaper,
    Thesis,
    Report,
    Article,

    // Educational
    Course,
    Lesson,
    Tutorial,
    Lecture,
    Workshop,

    // Scientific
    Dataset,
    Experiment,
    Observation,
    Sample,
    Specimen,
    Model,              // Scientific/statistical model

    // Software & Technology
    Software,
    Library,
    Framework,
    Application,
    Game,
    AiModel,
    MachineLearningModel,
    ContainerImage,     // Docker, OCI images
    VirtualMachine,

    // Visual Arts & Design
    Photo,
    PhotoAlbum,
    Artwork,
    Drawing,
    Blueprint,
    CadModel,
    ThreeDModel,
    Animation,

    // Archival & Museum
    MuseumArtifact,
    HistoricalDocument,
    Manuscript,
    ArchivalRecord,
    Collection,

    // Data & Backup
    Backup,
    Archive,
    Snapshot,

    // Other
    Website,
    WebPage,
    Document,
    Presentation,
    Spreadsheet,
    Database,

    // Extensibility: allows any custom type
    Custom(String),
}

/// Creator/contributor role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatorRole {
    Author,
    Artist,
    Director,
    Producer,
    Actor,
    Musician,
    Singer,
    Composer,
    Conductor,
    Editor,
    Photographer,
    Illustrator,
    Translator,
    Narrator,
    Developer,
    Maintainer,
    Contributor,
    Curator,
    Researcher,
    DataCollector,
    Custom(String),
}

/// A creator/contributor with their role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Creator {
    pub name: String,
    pub role: CreatorRole,
    /// Optional identifier (ORCID, ISNI, etc.)
    pub identifier: Option<String>,
}

/// License information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct License {
    /// License name (e.g., "CC-BY-4.0", "MIT", "Apache-2.0")
    pub name: String,
    /// URL to license text
    pub url: Option<String>,
    /// SPDX identifier if applicable
    pub spdx_id: Option<String>,
}

/// File/resource reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier (CID, URL, etc.)
    pub id: String,
    /// MIME type
    pub mime_type: Option<String>,
    /// Size in bytes
    pub size: Option<u64>,
    /// Checksum/hash
    pub checksum: Option<String>,
    /// Purpose (thumbnail, preview, master, etc.)
    pub purpose: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_serialization() {
        let types = vec![
            ContentType::Movie,
            ContentType::MusicAlbum,
            ContentType::ScientificPaper,
            ContentType::AiModel,
            ContentType::Custom("MyCustomType".to_string()),
        ];

        for content_type in types {
            let json = serde_json::to_string(&content_type).unwrap();
            let deserialized: ContentType = serde_json::from_str(&json).unwrap();
            assert_eq!(content_type, deserialized);
        }
    }

    #[test]
    fn test_creator_with_role() {
        let creator = Creator {
            name: "Jane Smith".to_string(),
            role: CreatorRole::Director,
            identifier: Some("https://orcid.org/0000-0001-2345-6789".to_string()),
        };

        let json = serde_json::to_string(&creator).unwrap();
        let deserialized: Creator = serde_json::from_str(&json).unwrap();
        assert_eq!(creator, deserialized);
    }

    #[test]
    fn test_custom_roles() {
        let role = CreatorRole::Custom("VoiceActor".to_string());
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: CreatorRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }
}
