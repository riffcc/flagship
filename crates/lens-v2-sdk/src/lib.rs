/// Versioned schema system for flexible P2P data exchange
pub mod schema;

/// Core data models
pub mod models;

// Re-export commonly used types
pub use schema::{SchemaDefinition, SchemaRegistry, SchemaVersion, Versioned, VersionNegotiation};
pub use models::*;
