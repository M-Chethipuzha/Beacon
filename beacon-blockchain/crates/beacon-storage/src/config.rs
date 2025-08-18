use crate::DatabaseConfig;
use serde::{Deserialize, Serialize};

/// Storage system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Enable automatic compaction
    pub auto_compaction: bool,
    /// Compaction interval in seconds
    pub compaction_interval: u64,
    /// Enable background sync
    pub background_sync: bool,
    /// Sync interval in seconds
    pub sync_interval: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            auto_compaction: true,
            compaction_interval: 3600, // 1 hour
            background_sync: true,
            sync_interval: 300, // 5 minutes
        }
    }
}

impl StorageConfig {
    /// Validate the storage configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.compaction_interval == 0 {
            return Err("Compaction interval must be greater than 0".to_string());
        }

        if self.sync_interval == 0 {
            return Err("Sync interval must be greater than 0".to_string());
        }

        Ok(())
    }
}
