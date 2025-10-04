use super::{SchemaDefinition, SchemaVersion};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for storing and retrieving schema definitions
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    /// Map of schema name -> version -> definition
    schemas: Arc<RwLock<HashMap<String, HashMap<SchemaVersion, SchemaDefinition>>>>,
}

impl SchemaRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a schema definition
    pub fn register(&self, schema: SchemaDefinition) -> Result<(), String> {
        let mut schemas = self
            .schemas
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let versions = schemas
            .entry(schema.name.clone())
            .or_insert_with(HashMap::new);

        if versions.contains_key(&schema.version) {
            return Err(format!(
                "Schema {} version {} is already registered",
                schema.name, schema.version
            ));
        }

        versions.insert(schema.version.clone(), schema);
        Ok(())
    }

    /// Get a specific schema version
    pub fn get(
        &self,
        name: &str,
        version: &SchemaVersion,
    ) -> Result<Option<SchemaDefinition>, String> {
        let schemas = self
            .schemas
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        Ok(schemas
            .get(name)
            .and_then(|versions| versions.get(version))
            .cloned())
    }

    /// Get the latest version of a schema
    pub fn get_latest(&self, name: &str) -> Result<Option<SchemaDefinition>, String> {
        let schemas = self
            .schemas
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        Ok(schemas.get(name).and_then(|versions| {
            versions
                .keys()
                .max()
                .and_then(|max_version| versions.get(max_version))
                .cloned()
        }))
    }

    /// Get all versions of a schema
    pub fn get_all_versions(&self, name: &str) -> Result<Vec<SchemaVersion>, String> {
        let schemas = self
            .schemas
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        Ok(schemas
            .get(name)
            .map(|versions| {
                let mut versions: Vec<_> = versions.keys().cloned().collect();
                versions.sort();
                versions
            })
            .unwrap_or_default())
    }

    /// List all registered schema names
    pub fn list_schemas(&self) -> Result<Vec<String>, String> {
        let schemas = self
            .schemas
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        Ok(schemas.keys().cloned().collect())
    }

    /// Export all schemas as JSON
    pub fn export_all(&self) -> Result<serde_json::Value, String> {
        let schemas = self
            .schemas
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        let mut export = serde_json::Map::new();
        for (name, versions) in schemas.iter() {
            let mut version_map = serde_json::Map::new();
            for (version, definition) in versions.iter() {
                version_map.insert(
                    version.to_string(),
                    serde_json::to_value(definition)
                        .map_err(|e| format!("Failed to serialize schema: {}", e))?,
                );
            }
            export.insert(name.clone(), serde_json::Value::Object(version_map));
        }

        Ok(serde_json::Value::Object(export))
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema(name: &str, version: SchemaVersion) -> SchemaDefinition {
        SchemaDefinition {
            name: name.to_string(),
            version,
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                }
            }),
            migration_hints: None,
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = SchemaRegistry::new();
        let schema = create_test_schema("Release", SchemaVersion::new(1, 0, 0));

        registry.register(schema.clone()).unwrap();

        let retrieved = registry
            .get("Release", &SchemaVersion::new(1, 0, 0))
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.name, "Release");
        assert_eq!(retrieved.version, SchemaVersion::new(1, 0, 0));
    }

    #[test]
    fn test_register_duplicate_version() {
        let registry = SchemaRegistry::new();
        let schema = create_test_schema("Release", SchemaVersion::new(1, 0, 0));

        registry.register(schema.clone()).unwrap();
        let result = registry.register(schema.clone());

        assert!(result.is_err());
    }

    #[test]
    fn test_get_latest() {
        let registry = SchemaRegistry::new();

        registry
            .register(create_test_schema("Release", SchemaVersion::new(1, 0, 0)))
            .unwrap();
        registry
            .register(create_test_schema("Release", SchemaVersion::new(1, 1, 0)))
            .unwrap();
        registry
            .register(create_test_schema("Release", SchemaVersion::new(2, 0, 0)))
            .unwrap();

        let latest = registry.get_latest("Release").unwrap().unwrap();
        assert_eq!(latest.version, SchemaVersion::new(2, 0, 0));
    }

    #[test]
    fn test_get_all_versions() {
        let registry = SchemaRegistry::new();

        registry
            .register(create_test_schema("Release", SchemaVersion::new(1, 0, 0)))
            .unwrap();
        registry
            .register(create_test_schema("Release", SchemaVersion::new(2, 0, 0)))
            .unwrap();
        registry
            .register(create_test_schema("Release", SchemaVersion::new(1, 1, 0)))
            .unwrap();

        let versions = registry.get_all_versions("Release").unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0], SchemaVersion::new(1, 0, 0));
        assert_eq!(versions[1], SchemaVersion::new(1, 1, 0));
        assert_eq!(versions[2], SchemaVersion::new(2, 0, 0));
    }

    #[test]
    fn test_list_schemas() {
        let registry = SchemaRegistry::new();

        registry
            .register(create_test_schema("Release", SchemaVersion::new(1, 0, 0)))
            .unwrap();
        registry
            .register(create_test_schema("Track", SchemaVersion::new(1, 0, 0)))
            .unwrap();

        let mut schemas = registry.list_schemas().unwrap();
        schemas.sort();

        assert_eq!(schemas.len(), 2);
        assert!(schemas.contains(&"Release".to_string()));
        assert!(schemas.contains(&"Track".to_string()));
    }
}
