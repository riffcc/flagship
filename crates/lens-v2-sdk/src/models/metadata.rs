use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard metadata schemas support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "standard", rename_all = "snake_case")]
pub enum StandardMetadata {
    /// Dublin Core metadata
    /// https://www.dublincore.org/specifications/dublin-core/dcmi-terms/
    DublinCore {
        /// Title
        title: Option<String>,
        /// Creator
        creator: Option<Vec<String>>,
        /// Subject/keywords
        subject: Option<Vec<String>>,
        /// Description
        description: Option<String>,
        /// Publisher
        publisher: Option<String>,
        /// Contributor
        contributor: Option<Vec<String>>,
        /// Date
        date: Option<String>,
        /// Type
        #[serde(rename = "type")]
        dc_type: Option<String>,
        /// Format
        format: Option<String>,
        /// Identifier
        identifier: Option<String>,
        /// Source
        source: Option<String>,
        /// Language
        language: Option<String>,
        /// Relation
        relation: Option<String>,
        /// Coverage
        coverage: Option<String>,
        /// Rights
        rights: Option<String>,
    },

    /// Schema.org structured data
    /// https://schema.org/
    SchemaOrg {
        /// @context
        context: String,
        /// @type
        #[serde(rename = "type")]
        schema_type: String,
        /// Properties as key-value pairs
        properties: HashMap<String, serde_json::Value>,
    },

    /// DataCite metadata schema (for research data)
    /// https://schema.datacite.org/
    DataCite {
        /// DOI
        doi: Option<String>,
        /// Creators
        creators: Vec<DataCiteCreator>,
        /// Titles
        titles: Vec<DataCiteTitle>,
        /// Publisher
        publisher: String,
        /// Publication year
        publication_year: u32,
        /// Resource type
        resource_type: DataCiteResourceType,
        /// Subjects
        subjects: Option<Vec<String>>,
        /// Additional properties
        #[serde(flatten)]
        additional: HashMap<String, serde_json::Value>,
    },

    /// Darwin Core (for biodiversity/specimens)
    /// https://dwc.tdwg.org/
    DarwinCore {
        /// Scientific name
        scientific_name: Option<String>,
        /// Kingdom
        kingdom: Option<String>,
        /// Phylum
        phylum: Option<String>,
        /// Class
        class: Option<String>,
        /// Order
        order: Option<String>,
        /// Family
        family: Option<String>,
        /// Genus
        genus: Option<String>,
        /// Additional DwC terms
        #[serde(flatten)]
        additional: HashMap<String, serde_json::Value>,
    },

    /// PREMIS (digital preservation)
    /// https://www.loc.gov/standards/premis/
    Premis {
        /// Object identifier
        object_id: String,
        /// Object category
        object_category: String,
        /// Preservation level
        preservation_level: Option<String>,
        /// Additional PREMIS data
        #[serde(flatten)]
        additional: HashMap<String, serde_json::Value>,
    },

    /// Custom/arbitrary metadata
    /// Allows any metadata scheme not explicitly supported
    Custom {
        /// Schema identifier (URL or name)
        schema: String,
        /// Metadata as arbitrary JSON
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCiteCreator {
    pub name: String,
    pub name_type: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub name_identifiers: Option<Vec<NameIdentifier>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameIdentifier {
    pub name_identifier: String,
    pub name_identifier_scheme: String,
    pub scheme_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCiteTitle {
    pub title: String,
    pub title_type: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCiteResourceType {
    pub resource_type_general: String,
    pub resource_type: Option<String>,
}

/// Container for multiple metadata standards
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MetadataContainer {
    /// Multiple metadata standards can coexist
    pub standards: Vec<StandardMetadata>,
}

impl MetadataContainer {
    pub fn new() -> Self {
        Self {
            standards: Vec::new(),
        }
    }

    pub fn add(&mut self, metadata: StandardMetadata) {
        self.standards.push(metadata);
    }

    pub fn get_dublin_core(&self) -> Option<&StandardMetadata> {
        self.standards.iter().find(|m| matches!(m, StandardMetadata::DublinCore { .. }))
    }

    pub fn get_schema_org(&self) -> Option<&StandardMetadata> {
        self.standards.iter().find(|m| matches!(m, StandardMetadata::SchemaOrg { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dublin_core_metadata() {
        let dc = StandardMetadata::DublinCore {
            title: Some("Example Dataset".to_string()),
            creator: Some(vec!["Jane Doe".to_string()]),
            subject: Some(vec!["Science".to_string(), "Data".to_string()]),
            description: Some("An example dataset".to_string()),
            publisher: Some("Example Org".to_string()),
            contributor: None,
            date: Some("2024-01-01".to_string()),
            dc_type: Some("Dataset".to_string()),
            format: Some("application/json".to_string()),
            identifier: Some("doi:10.1234/example".to_string()),
            source: None,
            language: Some("en".to_string()),
            relation: None,
            coverage: None,
            rights: Some("CC-BY-4.0".to_string()),
        };

        let json = serde_json::to_string(&dc).unwrap();
        let deserialized: StandardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(dc, deserialized);
    }

    #[test]
    fn test_custom_metadata() {
        let custom = StandardMetadata::Custom {
            schema: "https://example.com/my-schema".to_string(),
            data: serde_json::json!({
                "custom_field": "value",
                "another_field": 42
            }),
        };

        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: StandardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn test_metadata_container() {
        let mut container = MetadataContainer::new();

        container.add(StandardMetadata::DublinCore {
            title: Some("Test".to_string()),
            creator: None,
            subject: None,
            description: None,
            publisher: None,
            contributor: None,
            date: None,
            dc_type: None,
            format: None,
            identifier: None,
            source: None,
            language: None,
            relation: None,
            coverage: None,
            rights: None,
        });

        assert!(container.get_dublin_core().is_some());
        assert!(container.get_schema_org().is_none());
    }

    #[test]
    fn test_datacite_metadata() {
        let datacite = StandardMetadata::DataCite {
            doi: Some("10.1234/example".to_string()),
            creators: vec![DataCiteCreator {
                name: "Smith, John".to_string(),
                name_type: Some("Personal".to_string()),
                given_name: Some("John".to_string()),
                family_name: Some("Smith".to_string()),
                name_identifiers: Some(vec![NameIdentifier {
                    name_identifier: "0000-0001-2345-6789".to_string(),
                    name_identifier_scheme: "ORCID".to_string(),
                    scheme_uri: Some("https://orcid.org".to_string()),
                }]),
            }],
            titles: vec![DataCiteTitle {
                title: "Example Research Data".to_string(),
                title_type: None,
                lang: Some("en".to_string()),
            }],
            publisher: "Example University".to_string(),
            publication_year: 2024,
            resource_type: DataCiteResourceType {
                resource_type_general: "Dataset".to_string(),
                resource_type: Some("Experimental Data".to_string()),
            },
            subjects: Some(vec!["Biology".to_string(), "Genetics".to_string()]),
            additional: HashMap::new(),
        };

        let json = serde_json::to_string(&datacite).unwrap();
        let deserialized: StandardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(datacite, deserialized);
    }
}
