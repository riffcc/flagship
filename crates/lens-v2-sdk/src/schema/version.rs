use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Semantic version for schemas (major.minor.patch)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string (e.g., "1.2.3")
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", s));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(Self::new(major, minor, patch))
    }

    /// Check if this version is compatible with another version
    /// Compatible if major versions match and this version >= other version
    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        if self.major != other.major {
            return false;
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Greater => true,
            Ordering::Equal => self.patch >= other.patch,
            Ordering::Less => false,
        }
    }

    /// Check if this version can read data written by another version
    /// Reading is possible if major versions match
    pub fn can_read(&self, other: &SchemaVersion) -> bool {
        self.major == other.major
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for SchemaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchemaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parsing_invalid() {
        assert!(SchemaVersion::parse("1.2").is_err());
        assert!(SchemaVersion::parse("1.2.3.4").is_err());
        assert!(SchemaVersion::parse("a.b.c").is_err());
    }

    #[test]
    fn test_version_ordering() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 1, 0);
        let v3 = SchemaVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_1_0 = SchemaVersion::new(1, 1, 0);
        let v1_1_1 = SchemaVersion::new(1, 1, 1);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);

        // Same major, newer version is compatible
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(v1_1_1.is_compatible_with(&v1_1_0));

        // Same major, older version is not compatible
        assert!(!v1_0_0.is_compatible_with(&v1_1_0));

        // Different major versions are not compatible
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    }

    #[test]
    fn test_can_read() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_1_0 = SchemaVersion::new(1, 1, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);

        // Same major version can read each other
        assert!(v1_0_0.can_read(&v1_1_0));
        assert!(v1_1_0.can_read(&v1_0_0));

        // Different major versions cannot read
        assert!(!v1_0_0.can_read(&v2_0_0));
        assert!(!v2_0_0.can_read(&v1_0_0));
    }

    #[test]
    fn test_display() {
        let v = SchemaVersion::new(1, 2, 3);
        assert_eq!(format!("{}", v), "1.2.3");
    }
}
