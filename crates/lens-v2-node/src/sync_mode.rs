//! Sync Mode Configuration
//!
//! Defines sync modes: TGPQL (current) and UBTS (unified transactions)

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Sync mode for the node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    /// TGPQL mode - TGP Query Language with separate block types
    /// - WantList queries
    /// - Separate release/admin/featured blocks
    /// - Current production mode
    TGPQL,

    /// UBTS mode - Unified Block Transaction System
    /// - All operations are transactions in blocks
    /// - Single block type with transaction list
    /// - Cleaner, more extensible architecture
    UBTS,
}

impl Default for SyncMode {
    fn default() -> Self {
        // Default to TGPQL for backward compatibility
        Self::TGPQL
    }
}

impl FromStr for SyncMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tgpql" => Ok(Self::TGPQL),
            "ubts" => Ok(Self::UBTS),
            _ => Err(format!("Unknown sync mode: {}. Valid modes: tgpql, ubts", s)),
        }
    }
}

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TGPQL => write!(f, "TGPQL"),
            Self::UBTS => write!(f, "UBTS"),
        }
    }
}

impl SyncMode {
    /// Get a description of this sync mode
    pub fn description(&self) -> &'static str {
        match self {
            Self::TGPQL => "TGP Query Language - WantList-based sync with separate block types",
            Self::UBTS => "Unified Block Transaction System - All operations as transactions",
        }
    }

    /// Check if this mode is TGPQL
    pub fn is_tgpql(&self) -> bool {
        matches!(self, Self::TGPQL)
    }

    /// Check if this mode is UBTS
    pub fn is_ubts(&self) -> bool {
        matches!(self, Self::UBTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_mode_from_str() {
        assert_eq!("tgpql".parse::<SyncMode>().unwrap(), SyncMode::TGPQL);
        assert_eq!("TGPQL".parse::<SyncMode>().unwrap(), SyncMode::TGPQL);
        assert_eq!("ubts".parse::<SyncMode>().unwrap(), SyncMode::UBTS);
        assert_eq!("UBTS".parse::<SyncMode>().unwrap(), SyncMode::UBTS);
        assert!("invalid".parse::<SyncMode>().is_err());
    }

    #[test]
    fn test_sync_mode_display() {
        assert_eq!(SyncMode::TGPQL.to_string(), "TGPQL");
        assert_eq!(SyncMode::UBTS.to_string(), "UBTS");
    }

    #[test]
    fn test_sync_mode_default() {
        assert_eq!(SyncMode::default(), SyncMode::TGPQL);
    }
}
