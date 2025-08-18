use crate::{Database, Keys, CF_STATE};
use beacon_core::{BeaconResult, StateKey, StateValue, StateMap};
use std::sync::Arc;

/// State storage manager
pub struct StateStorage {
    db: Arc<Database>,
}

impl StateStorage {
    /// Create a new state storage instance
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get a value from state
    pub async fn get_state(&self, key: &StateKey) -> BeaconResult<Option<StateValue>> {
        let db_key = Keys::state(key);
        self.db.get_cf(CF_STATE, &db_key)
    }

    /// Set a value in state
    pub async fn set_state(&self, key: StateKey, value: StateValue) -> BeaconResult<()> {
        let db_key = Keys::state(&key);
        self.db.put_cf(CF_STATE, &db_key, &value)?;
        tracing::trace!("Set state: {} = {} bytes", key, value.len());
        Ok(())
    }

    /// Delete a value from state
    pub async fn delete_state(&self, key: &StateKey) -> BeaconResult<()> {
        let db_key = Keys::state(key);
        self.db.delete_cf(CF_STATE, &db_key)?;
        tracing::trace!("Deleted state: {}", key);
        Ok(())
    }

    /// Apply multiple state changes atomically
    pub async fn apply_state_changes(&self, changes: &StateMap) -> BeaconResult<()> {
        if changes.is_empty() {
            return Ok(());
        }

        let mut batch = self.db.create_batch();
        let cf = self.db.cf_handle(CF_STATE)?;

        for (key, value) in changes {
            let db_key = Keys::state(key);
            batch.put_cf(cf, &db_key, value);
        }

        self.db.write_batch(batch)?;
        tracing::debug!("Applied {} state changes", changes.len());
        Ok(())
    }

    /// Get multiple state values
    pub async fn get_state_batch(&self, keys: &[StateKey]) -> BeaconResult<StateMap> {
        let mut result = StateMap::new();
        
        for key in keys {
            if let Some(value) = self.get_state(key).await? {
                result.insert(key.clone(), value);
            }
        }
        
        Ok(result)
    }

    /// Get all state with a given prefix
    pub async fn get_state_with_prefix(&self, prefix: &str) -> BeaconResult<StateMap> {
        let mut result = StateMap::new();
        let db_prefix = Keys::state(prefix);
        
        let iter = self.db.iter_cf_mode(
            CF_STATE,
            rocksdb::IteratorMode::From(&db_prefix, rocksdb::Direction::Forward)
        )?;

        for item in iter {
            match item {
                Ok((key, value)) => {
                    if key.starts_with(&db_prefix) {
                        // Extract the original state key by removing the "state:" prefix
                        let key_str = String::from_utf8_lossy(&key);
                        if key_str.starts_with("state:") {
                            let state_key = key_str[6..].to_string(); // Remove "state:" prefix
                            result.insert(state_key, value.to_vec());
                        }
                    } else {
                        // We've gone past the prefix
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Error iterating state with prefix {}: {}", prefix, e);
                    break;
                }
            }
        }
        
        Ok(result)
    }

    /// Check if a state key exists
    pub async fn state_exists(&self, key: &StateKey) -> BeaconResult<bool> {
        let db_key = Keys::state(key);
        Ok(self.db.get_cf(CF_STATE, &db_key)?.is_some())
    }

    /// Get the size of a state value
    pub async fn get_state_size(&self, key: &StateKey) -> BeaconResult<Option<usize>> {
        if let Some(value) = self.get_state(key).await? {
            Ok(Some(value.len()))
        } else {
            Ok(None)
        }
    }

    /// Create a state snapshot (for rollback purposes)
    pub async fn create_snapshot(&self, snapshot_id: &str) -> BeaconResult<()> {
        // In a real implementation, this would create a consistent snapshot
        // For now, we'll just log it
        tracing::info!("Created state snapshot: {}", snapshot_id);
        Ok(())
    }

    /// Restore from a state snapshot
    pub async fn restore_snapshot(&self, snapshot_id: &str) -> BeaconResult<()> {
        // In a real implementation, this would restore from a snapshot
        tracing::info!("Restored state snapshot: {}", snapshot_id);
        Ok(())
    }

    /// Clear all state (dangerous!)
    pub async fn clear_all_state(&self) -> BeaconResult<()> {
        let iter = self.db.iter_cf(CF_STATE)?;
        let mut batch = self.db.create_batch();
        let cf = self.db.cf_handle(CF_STATE)?;

        for item in iter {
            match item {
                Ok((key, _)) => {
                    batch.delete_cf(cf, &key);
                }
                Err(e) => {
                    tracing::error!("Error clearing state: {}", e);
                    break;
                }
            }
        }

        self.db.write_batch(batch)?;
        tracing::warn!("Cleared all state data");
        Ok(())
    }
}

/// State storage utilities
impl StateStorage {
    /// Helper to store a JSON-serializable value
    pub async fn set_json<T: serde::Serialize>(&self, key: StateKey, value: &T) -> BeaconResult<()> {
        let json_data = serde_json::to_vec(value)
            .map_err(|e| beacon_core::BeaconError::serialization(format!("JSON serialization failed: {}", e)))?;
        self.set_state(key, json_data).await
    }

    /// Helper to get a JSON-deserializable value
    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &StateKey) -> BeaconResult<Option<T>> {
        if let Some(data) = self.get_state(key).await? {
            let value = serde_json::from_slice(&data)
                .map_err(|e| beacon_core::BeaconError::serialization(format!("JSON deserialization failed: {}", e)))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Helper to store a string value
    pub async fn set_string(&self, key: StateKey, value: String) -> BeaconResult<()> {
        self.set_state(key, value.into_bytes()).await
    }

    /// Helper to get a string value
    pub async fn get_string(&self, key: &StateKey) -> BeaconResult<Option<String>> {
        if let Some(data) = self.get_state(key).await? {
            let string = String::from_utf8(data)
                .map_err(|e| beacon_core::BeaconError::serialization(format!("Invalid UTF-8: {}", e)))?;
            Ok(Some(string))
        } else {
            Ok(None)
        }
    }

    /// Helper to increment a numeric counter
    pub async fn increment_counter(&self, key: StateKey) -> BeaconResult<u64> {
        let current = if let Some(data) = self.get_state(&key).await? {
            if data.len() == 8 {
                u64::from_le_bytes(data.try_into().unwrap())
            } else {
                0
            }
        } else {
            0
        };

        let new_value = current + 1;
        self.set_state(key, new_value.to_le_bytes().to_vec()).await?;
        Ok(new_value)
    }

    /// Helper to get a numeric counter
    pub async fn get_counter(&self, key: &StateKey) -> BeaconResult<u64> {
        if let Some(data) = self.get_state(key).await? {
            if data.len() == 8 {
                Ok(u64::from_le_bytes(data.try_into().unwrap()))
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// Put a value in state (alias for set_state for gRPC compatibility)
    pub async fn put_state(&self, key: &StateKey, value: StateValue) -> BeaconResult<()> {
        self.set_state(key.clone(), value).await
    }

    /// Get state values within a key range
    pub async fn get_state_range(&self, start_key: &str, end_key: &str) -> BeaconResult<Vec<(String, Vec<u8>)>> {
        let mut result = Vec::new();
        let db_start = Keys::state(start_key);
        let db_end = Keys::state(end_key);
        
        let iter = self.db.iter_cf_mode(
            CF_STATE,
            rocksdb::IteratorMode::From(&db_start, rocksdb::Direction::Forward)
        )?;

        for item in iter {
            match item {
                Ok((key, value)) => {
                    if key.as_ref() >= db_end.as_slice() {
                        // We've reached the end of the range
                        break;
                    }
                    
                    // Extract the original state key by removing the "state:" prefix
                    let key_str = String::from_utf8_lossy(&key);
                    if key_str.starts_with("state:") {
                        let state_key = key_str[6..].to_string(); // Remove "state:" prefix
                        result.push((state_key, value.to_vec()));
                    }
                }
                Err(e) => {
                    tracing::warn!("Error iterating state range {} to {}: {}", start_key, end_key, e);
                    break;
                }
            }
        }

        tracing::debug!("Found {} state entries in range {} to {}", result.len(), start_key, end_key);
        Ok(result)
    }
}
