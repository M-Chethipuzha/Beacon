use rocksdb::{DB, Options, ColumnFamily, ColumnFamilyDescriptor, WriteBatch};
use std::path::Path;
use std::sync::Arc;
use beacon_core::{BeaconError, BeaconResult};
use tracing::{debug, info};

/// Database column families
pub const CF_BLOCKS: &str = "blocks";
pub const CF_TRANSACTIONS: &str = "transactions";
pub const CF_STATE: &str = "state";
pub const CF_METADATA: &str = "metadata";
pub const CF_INDICES: &str = "indices";

/// Database configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    /// Path to the database directory
    pub path: String,
    /// Enable compression
    pub enable_compression: bool,
    /// Cache size in MB
    pub cache_size: usize,
    /// Write buffer size in MB
    pub write_buffer_size: usize,
    /// Maximum number of open files
    pub max_open_files: i32,
    /// Enable statistics
    pub enable_statistics: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "./beacon_data".to_string(),
            enable_compression: true,
            cache_size: 256, // 256 MB
            write_buffer_size: 64, // 64 MB
            max_open_files: 1000,
            enable_statistics: true,
        }
    }
}

/// Database wrapper for RocksDB
pub struct Database {
    db: Arc<DB>,
    config: DatabaseConfig,
}

impl Database {
    /// Open or create a new database
    pub fn open(config: DatabaseConfig) -> BeaconResult<Self> {
        info!("Opening database at: {}", config.path);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&config.path)
            .map_err(|e| BeaconError::storage(format!("Failed to create database directory: {}", e)))?;

        // Configure RocksDB options
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        
        // Set cache size
        if config.cache_size > 0 {
            let cache = rocksdb::Cache::new_lru_cache(config.cache_size * 1024 * 1024);
            db_opts.set_row_cache(&cache);
        }
        
        // Set write buffer size
        db_opts.set_write_buffer_size(config.write_buffer_size * 1024 * 1024);
        
        // Set max open files
        db_opts.set_max_open_files(config.max_open_files);
        
        // Enable compression
        if config.enable_compression {
            db_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        }
        
        // Enable statistics
        if config.enable_statistics {
            db_opts.enable_statistics();
        }

        // Configure column families
        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_BLOCKS, Options::default()),
            ColumnFamilyDescriptor::new(CF_TRANSACTIONS, Options::default()),
            ColumnFamilyDescriptor::new(CF_STATE, Options::default()),
            ColumnFamilyDescriptor::new(CF_METADATA, Options::default()),
            ColumnFamilyDescriptor::new(CF_INDICES, Options::default()),
        ];

        // Open database with column families
        let db = DB::open_cf_descriptors(&db_opts, &config.path, cfs)
            .map_err(|e| BeaconError::storage(format!("Failed to open database: {}", e)))?;

        info!("Database opened successfully");

        Ok(Self {
            db: Arc::new(db),
            config,
        })
    }

    /// Get a reference to the underlying RocksDB instance
    pub fn inner(&self) -> &DB {
        &self.db
    }

    /// Get a column family handle
    pub fn cf_handle(&self, cf_name: &str) -> BeaconResult<&ColumnFamily> {
        self.db
            .cf_handle(cf_name)
            .ok_or_else(|| BeaconError::storage(format!("Column family '{}' not found", cf_name)))
    }

    /// Put a key-value pair in the default column family
    pub fn put(&self, key: &[u8], value: &[u8]) -> BeaconResult<()> {
        self.db
            .put(key, value)
            .map_err(|e| BeaconError::storage(format!("Failed to put data: {}", e)))
    }

    /// Put a key-value pair in a specific column family
    pub fn put_cf(&self, cf_name: &str, key: &[u8], value: &[u8]) -> BeaconResult<()> {
        let cf = self.cf_handle(cf_name)?;
        self.db
            .put_cf(cf, key, value)
            .map_err(|e| BeaconError::storage(format!("Failed to put data in CF '{}': {}", cf_name, e)))
    }

    /// Get a value by key from the default column family
    pub fn get(&self, key: &[u8]) -> BeaconResult<Option<Vec<u8>>> {
        self.db
            .get(key)
            .map_err(|e| BeaconError::storage(format!("Failed to get data: {}", e)))
    }

    /// Get a value by key from a specific column family
    pub fn get_cf(&self, cf_name: &str, key: &[u8]) -> BeaconResult<Option<Vec<u8>>> {
        let cf = self.cf_handle(cf_name)?;
        self.db
            .get_cf(cf, key)
            .map_err(|e| BeaconError::storage(format!("Failed to get data from CF '{}': {}", cf_name, e)))
    }

    /// Delete a key from the default column family
    pub fn delete(&self, key: &[u8]) -> BeaconResult<()> {
        self.db
            .delete(key)
            .map_err(|e| BeaconError::storage(format!("Failed to delete data: {}", e)))
    }

    /// Delete a key from a specific column family
    pub fn delete_cf(&self, cf_name: &str, key: &[u8]) -> BeaconResult<()> {
        let cf = self.cf_handle(cf_name)?;
        self.db
            .delete_cf(cf, key)
            .map_err(|e| BeaconError::storage(format!("Failed to delete data from CF '{}': {}", cf_name, e)))
    }

    /// Create a write batch for atomic operations
    pub fn create_batch(&self) -> WriteBatch {
        WriteBatch::default()
    }

    /// Write a batch atomically
    pub fn write_batch(&self, batch: WriteBatch) -> BeaconResult<()> {
        self.db
            .write(batch)
            .map_err(|e| BeaconError::storage(format!("Failed to write batch: {}", e)))
    }

    /// Create an iterator over a column family
    pub fn iter_cf(&self, cf_name: &str) -> BeaconResult<rocksdb::DBIteratorWithThreadMode<'_, DB>> {
        let cf = self.cf_handle(cf_name)?;
        Ok(self.db.iterator_cf(cf, rocksdb::IteratorMode::Start))
    }

    /// Create an iterator with a specific mode
    pub fn iter_cf_mode(
        &self,
        cf_name: &str,
        mode: rocksdb::IteratorMode,
    ) -> BeaconResult<rocksdb::DBIteratorWithThreadMode<'_, DB>> {
        let cf = self.cf_handle(cf_name)?;
        Ok(self.db.iterator_cf(cf, mode))
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Option<String> {
        self.db.property_value("rocksdb.stats").ok().flatten()
    }

    /// Compact a specific column family
    pub fn compact_cf(&self, cf_name: &str) -> BeaconResult<()> {
        let cf = self.cf_handle(cf_name)?;
        self.db
            .compact_range_cf(cf, None::<&[u8]>, None::<&[u8]>);
        debug!("Compacted column family: {}", cf_name);
        Ok(())
    }

    /// Compact all column families
    pub fn compact_all(&self) -> BeaconResult<()> {
        let cf_names = vec![CF_BLOCKS, CF_TRANSACTIONS, CF_STATE, CF_METADATA, CF_INDICES];
        
        for cf_name in cf_names {
            self.compact_cf(cf_name)?;
        }
        
        info!("Compacted all column families");
        Ok(())
    }

    /// Create a checkpoint (backup)
    pub fn create_checkpoint<P: AsRef<Path>>(&self, path: P) -> BeaconResult<()> {
        let checkpoint = rocksdb::checkpoint::Checkpoint::new(&self.db)
            .map_err(|e| BeaconError::storage(format!("Failed to create checkpoint object: {}", e)))?;
        
        checkpoint
            .create_checkpoint(path)
            .map_err(|e| BeaconError::storage(format!("Failed to create checkpoint: {}", e)))?;
        
        info!("Created database checkpoint");
        Ok(())
    }

    /// Get database size information
    pub fn get_size_info(&self) -> BeaconResult<DatabaseSizeInfo> {
        let mut total_size = 0u64;
        let mut cf_sizes = std::collections::HashMap::new();

        let cf_names = vec![CF_BLOCKS, CF_TRANSACTIONS, CF_STATE, CF_METADATA, CF_INDICES];
        
        for cf_name in cf_names {
            if let Ok(Some(size_str)) = self.db.property_value_cf(
                self.cf_handle(cf_name)?,
                "rocksdb.total-sst-files-size"
            ) {
                if let Ok(size) = size_str.parse::<u64>() {
                    cf_sizes.insert(cf_name.to_string(), size);
                    total_size += size;
                }
            }
        }

        Ok(DatabaseSizeInfo {
            total_size,
            cf_sizes,
        })
    }

    /// Perform database maintenance
    pub async fn maintenance(&self) -> BeaconResult<()> {
        info!("Starting database maintenance");

        // Compact all column families
        self.compact_all()?;

        // Log statistics
        if let Some(stats) = self.get_stats() {
            debug!("Database statistics:\n{}", stats);
        }

        // Log size information
        let size_info = self.get_size_info()?;
        info!("Database total size: {} MB", size_info.total_size / (1024 * 1024));

        info!("Database maintenance completed");
        Ok(())
    }

    /// Close the database
    pub fn close(self) {
        // Database will be closed when the Arc is dropped
        info!("Database closed");
    }
}

/// Database size information
#[derive(Debug)]
pub struct DatabaseSizeInfo {
    pub total_size: u64,
    pub cf_sizes: std::collections::HashMap<String, u64>,
}

/// Database key builders for consistent key formatting
pub struct Keys;

impl Keys {
    /// Block key: "block:{index}"
    pub fn block(index: u64) -> Vec<u8> {
        format!("block:{:020}", index).into_bytes()
    }

    /// Block hash key: "block_hash:{hash}"
    pub fn block_hash(hash: &str) -> Vec<u8> {
        format!("block_hash:{}", hash).into_bytes()
    }

    /// Transaction key: "tx:{tx_id}"
    pub fn transaction(tx_id: &str) -> Vec<u8> {
        format!("tx:{}", tx_id).into_bytes()
    }

    /// Transaction by block key: "tx_block:{block_index}:{tx_index}"
    pub fn transaction_by_block(block_index: u64, tx_index: usize) -> Vec<u8> {
        format!("tx_block:{:020}:{:010}", block_index, tx_index).into_bytes()
    }

    /// State key: "state:{key}"
    pub fn state(key: &str) -> Vec<u8> {
        format!("state:{}", key).into_bytes()
    }

    /// Metadata key: "meta:{key}"
    pub fn metadata(key: &str) -> Vec<u8> {
        format!("meta:{}", key).into_bytes()
    }

    /// Index key: "index:{type}:{value}"
    pub fn index(index_type: &str, value: &str) -> Vec<u8> {
        format!("index:{}:{}", index_type, value).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = DatabaseConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let db = Database::open(config).unwrap();

        // Test basic put/get
        let key = b"test_key";
        let value = b"test_value";
        
        db.put(key, value).unwrap();
        let retrieved = db.get(key).unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // Test column family operations
        db.put_cf(CF_BLOCKS, key, value).unwrap();
        let retrieved = db.get_cf(CF_BLOCKS, key).unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // Test delete
        db.delete(key).unwrap();
        let retrieved = db.get(key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_key_builders() {
        assert_eq!(Keys::block(123), b"block:00000000000000000123".to_vec());
        assert_eq!(Keys::transaction("tx123"), b"tx:tx123".to_vec());
        assert_eq!(Keys::state("balance:addr1"), b"state:balance:addr1".to_vec());
    }
}
