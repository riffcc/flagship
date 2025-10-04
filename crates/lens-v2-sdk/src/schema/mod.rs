pub mod version;
pub mod registry;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use version::SchemaVersion;
pub use registry::SchemaRegistry;

/// A JSON schema definition for a data model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDefinition {
    /// Schema name (e.g., "Release", "Track", "Account")
    pub name: String,

    /// Schema version
    pub version: SchemaVersion,

    /// JSON Schema specification
    pub schema: serde_json::Value,

    /// Migration hints for upgrading from previous versions
    pub migration_hints: Option<HashMap<SchemaVersion, MigrationHint>>,
}

/// Hints for migrating data between schema versions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationHint {
    /// Source version
    pub from_version: SchemaVersion,

    /// Target version
    pub to_version: SchemaVersion,

    /// Field mappings: old_field -> new_field
    pub field_mappings: HashMap<String, String>,

    /// Default values for new fields
    pub default_values: HashMap<String, serde_json::Value>,

    /// Fields that were removed
    pub removed_fields: Vec<String>,
}

/// Trait for types that have versioned schemas
pub trait Versioned {
    /// Get the schema name for this type
    fn schema_name() -> &'static str;

    /// Get the current schema version
    fn schema_version() -> SchemaVersion;

    /// Get the JSON schema definition
    fn schema_definition() -> SchemaDefinition;

    /// Check if this type can deserialize from a given version
    fn supports_version(version: &SchemaVersion) -> bool;
}

/// Result of a version negotiation between peers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionNegotiation {
    /// Schema name being negotiated
    pub schema_name: String,

    /// Versions supported by local peer
    pub local_versions: Vec<SchemaVersion>,

    /// Versions supported by remote peer
    pub remote_versions: Vec<SchemaVersion>,

    /// Agreed-upon version (highest compatible)
    pub agreed_version: Option<SchemaVersion>,
}

impl VersionNegotiation {
    /// Negotiate the best version between two peers
    pub fn negotiate(
        schema_name: String,
        local_versions: Vec<SchemaVersion>,
        remote_versions: Vec<SchemaVersion>,
    ) -> Self {
        // Find the highest version that both peers support
        let mut agreed_version = None;

        for local_ver in local_versions.iter().rev() {
            if remote_versions.contains(local_ver) {
                agreed_version = Some(local_ver.clone());
                break;
            }
        }

        Self {
            schema_name,
            local_versions,
            remote_versions,
            agreed_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_negotiation_exact_match() {
        let local = vec![
            SchemaVersion::new(1, 0, 0),
            SchemaVersion::new(2, 0, 0),
            SchemaVersion::new(3, 0, 0),
        ];
        let remote = vec![
            SchemaVersion::new(1, 0, 0),
            SchemaVersion::new(2, 0, 0),
            SchemaVersion::new(3, 0, 0),
        ];

        let result = VersionNegotiation::negotiate("Release".to_string(), local, remote);
        assert_eq!(result.agreed_version, Some(SchemaVersion::new(3, 0, 0)));
    }

    #[test]
    fn test_version_negotiation_partial_overlap() {
        let local = vec![
            SchemaVersion::new(2, 0, 0),
            SchemaVersion::new(3, 0, 0),
        ];
        let remote = vec![
            SchemaVersion::new(1, 0, 0),
            SchemaVersion::new(2, 0, 0),
        ];

        let result = VersionNegotiation::negotiate("Release".to_string(), local, remote);
        assert_eq!(result.agreed_version, Some(SchemaVersion::new(2, 0, 0)));
    }

    #[test]
    fn test_version_negotiation_no_overlap() {
        let local = vec![SchemaVersion::new(3, 0, 0)];
        let remote = vec![SchemaVersion::new(1, 0, 0)];

        let result = VersionNegotiation::negotiate("Release".to_string(), local, remote);
        assert_eq!(result.agreed_version, None);
    }
}
