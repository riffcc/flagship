# ContentItem: Universal Schema for All Content Types

The `ContentItem` schema is designed to handle **literally any kind of content** with extreme flexibility and extensibility.

## Supported Content Types

### Media - Video
- `movie` - Feature films
- `tv_series` - Television series
- `tv_episode` - Individual TV episodes
- `video` - Online videos (YouTube, Vimeo, etc.)
- `video_clip` - Short video clips

### Media - Audio
- `music_album` - Music albums
- `music_track` - Individual music tracks
- `playlist` - Curated playlists
- `podcast` - Podcast series
- `podcast_episode` - Individual podcast episodes
- `audiobook` - Audiobook recordings
- `audio_production` - DAW projects, stems, production files

### Publications
- `book` - Physical books
- `ebook` - Digital books
- `magazine` - Magazine publications
- `comic` - Comic books and graphic novels
- `scientific_paper` - Research papers
- `thesis` - Academic theses
- `report` - Reports and white papers
- `article` - Articles and essays

### Educational
- `course` - Educational courses
- `lesson` - Individual lessons
- `tutorial` - Tutorial content
- `lecture` - Lecture recordings
- `workshop` - Workshop materials

### Scientific
- `dataset` - Scientific datasets
- `experiment` - Experiment records
- `observation` - Observational data
- `sample` - Sample data
- `specimen` - Physical specimens (with Darwin Core support)
- `model` - Scientific/statistical models

### Software & Technology
- `software` - Software applications
- `library` - Code libraries
- `framework` - Development frameworks
- `application` - Applications
- `game` - Video games
- `ai_model` - AI/ML models
- `machine_learning_model` - ML models
- `container_image` - Docker/OCI images
- `virtual_machine` - VM images

### Visual Arts & Design
- `photo` - Photographs
- `photo_album` - Photo albums/collections
- `artwork` - Artwork and digital art
- `drawing` - Drawings and sketches
- `blueprint` - Technical blueprints
- `cad_model` - CAD models
- `three_d_model` - 3D models
- `animation` - Animations

### Archival & Museum
- `museum_artifact` - Museum artifacts
- `historical_document` - Historical documents
- `manuscript` - Manuscripts
- `archival_record` - Archival records
- `collection` - Collections

### Data & Backup
- `backup` - System backups
- `archive` - Archives
- `snapshot` - System snapshots

### Other
- `website` - Websites
- `web_page` - Web pages
- `document` - Generic documents
- `presentation` - Presentations
- `spreadsheet` - Spreadsheets
- `database` - Databases

### Custom Types
- `custom("YourType")` - Any custom content type you define!

## Standard Metadata Support

The `metadata` field supports multiple standard metadata schemas:

### Dublin Core
Industry standard for general metadata (libraries, archives, museums).

```rust
StandardMetadata::DublinCore {
    title: Some("Example Dataset".to_string()),
    creator: Some(vec!["Jane Doe".to_string()]),
    subject: Some(vec!["Science".to_string()]),
    description: Some("An example dataset".to_string()),
    publisher: Some("Example Org".to_string()),
    date: Some("2024-01-01".to_string()),
    dc_type: Some("Dataset".to_string()),
    format: Some("application/json".to_string()),
    identifier: Some("doi:10.1234/example".to_string()),
    language: Some("en".to_string()),
    rights: Some("CC-BY-4.0".to_string()),
    // ... other Dublin Core fields
}
```

### Schema.org
Web-focused structured data standard.

```rust
StandardMetadata::SchemaOrg {
    context: "https://schema.org".to_string(),
    schema_type: "Dataset".to_string(),
    properties: {
        "name": "My Dataset",
        "description": "A comprehensive dataset",
        "keywords": ["science", "research"]
    }
}
```

### DataCite
Research data citation standard (for scientific datasets).

```rust
StandardMetadata::DataCite {
    doi: Some("10.1234/example".to_string()),
    creators: vec![...],
    titles: vec![...],
    publisher: "University Press".to_string(),
    publication_year: 2024,
    resource_type: ...,
    subjects: Some(vec!["Biology", "Genetics"]),
}
```

### Darwin Core
Biodiversity data standard (for specimens and observations).

```rust
StandardMetadata::DarwinCore {
    scientific_name: Some("Panthera leo".to_string()),
    kingdom: Some("Animalia".to_string()),
    phylum: Some("Chordata".to_string()),
    class: Some("Mammalia".to_string()),
    // ... other taxonomic fields
}
```

### PREMIS
Digital preservation standard.

```rust
StandardMetadata::Premis {
    object_id: "obj-123".to_string(),
    object_category: "file".to_string(),
    preservation_level: Some("full".to_string()),
}
```

### Custom Metadata
Any metadata scheme not explicitly supported:

```rust
StandardMetadata::Custom {
    schema: "https://myorg.com/metadata-schema".to_string(),
    data: serde_json::json!({
        "custom_field": "value",
        "nested": {
            "data": "works too"
        }
    })
}
```

## Extensibility

### Type-Specific Fields
Use `type_specific` for fields unique to a content type:

```rust
ContentItem::new(...)
    .with_type_specific(serde_json::json!({
        "duration_seconds": 7200,
        "resolution": "3840x2160",
        "frame_rate": 60.0,
        "audio_channels": 5.1
    }))
```

### Custom Fields
Use `custom` for arbitrary data that doesn't fit elsewhere:

```rust
ContentItem::new(...)
    .with_custom_field("internal_id".to_string(), serde_json::json!(12345))
    .with_custom_field("priority".to_string(), serde_json::json!("high"))
    .with_custom_field("metadata_quality_score".to_string(), serde_json::json!(0.95))
```

## Version Negotiation

Content items include a `schema_version` field allowing peers to:
- Exchange data even with version drift
- Automatically negotiate compatible versions
- Gracefully handle missing fields
- Support protocol evolution over time

## Example: Scientific Dataset

```rust
use lens_v2_sdk::*;

let dataset = ContentItem::new(
    "QmExampleCID123".to_string(),
    ContentType::Dataset,
    "Global Climate Data 2024".to_string(),
)
.with_description("Comprehensive global climate observations for 2024".to_string())
.with_creator(Creator {
    name: "Dr. Jane Smith".to_string(),
    role: CreatorRole::Researcher,
    identifier: Some("https://orcid.org/0000-0001-2345-6789".to_string()),
})
.with_tag("climate".to_string())
.with_tag("2024".to_string())
.with_tag("global".to_string())
.with_resource(Resource {
    id: "QmDataFile123".to_string(),
    mime_type: Some("application/netcdf".to_string()),
    size: Some(1024 * 1024 * 1024), // 1GB
    checksum: Some("sha256:abc123...".to_string()),
    purpose: Some("data".to_string()),
});

// Add DataCite metadata
dataset.metadata.add(StandardMetadata::DataCite {
    doi: Some("10.5281/zenodo.1234567".to_string()),
    creators: vec![...],
    // ... DataCite fields
});

// Add custom fields
dataset = dataset
    .with_custom_field("measurement_method".to_string(),
        serde_json::json!("satellite"))
    .with_custom_field("quality_control_level".to_string(),
        serde_json::json!("Level 3"));
```

## Example: AI Model

```rust
let model = ContentItem::new(
    "QmModelCID456".to_string(),
    ContentType::AiModel,
    "GPT-Style Language Model".to_string(),
)
.with_description("A transformer-based language model".to_string())
.with_creator(Creator {
    name: "AI Research Lab".to_string(),
    role: CreatorRole::Developer,
    identifier: None,
})
.with_type_specific(serde_json::json!({
    "model_architecture": "transformer",
    "parameters": "7B",
    "training_data": "web corpus",
    "context_length": 8192,
    "quantization": "fp16"
}))
.with_custom_field("inference_framework".to_string(),
    serde_json::json!(["pytorch", "onnx"]));
```

## Example: Museum Artifact

```rust
let artifact = ContentItem::new(
    "QmArtifactCID789".to_string(),
    ContentType::MuseumArtifact,
    "Ancient Greek Amphora".to_string(),
)
.with_description("Red-figure amphora from 5th century BCE".to_string())
.with_creator(Creator {
    name: "Unknown Ancient Greek Potter".to_string(),
    role: CreatorRole::Artist,
    identifier: None,
})
.with_type_specific(serde_json::json!({
    "period": "Classical Greek",
    "circa": "-450",
    "material": ["clay", "pigment"],
    "dimensions": {
        "height_cm": 45,
        "diameter_cm": 30
    },
    "condition": "Restored, minor cracks",
    "provenance": "Excavated in Athens, 1890"
}));

// Add Dublin Core metadata for museum systems
artifact.metadata.add(StandardMetadata::DublinCore {
    title: Some("Ancient Greek Amphora".to_string()),
    creator: Some(vec!["Unknown".to_string()]),
    date: Some("-450".to_string()),
    dc_type: Some("PhysicalObject".to_string()),
    format: Some("ceramic".to_string()),
    // ... more Dublin Core fields
});
```

## Philosophy

The ContentItem schema is designed to be:

1. **Universal** - Handle any content type, present or future
2. **Extensible** - Custom types, fields, and metadata without breaking changes
3. **Standard-Compliant** - Support existing metadata standards
4. **Future-Proof** - Version negotiation allows protocol evolution
5. **P2P-Friendly** - Designed for decentralized networks with version drift
6. **Developer-Friendly** - Builder pattern, type safety, comprehensive tests
