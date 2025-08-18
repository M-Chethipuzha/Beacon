use crate::{Database, Keys, CF_BLOCKS};
use beacon_core::{BeaconResult, Block, BlockIndex};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Blockchain storage manager
pub struct BlockchainStorage {
    db: Arc<Database>,
}

impl BlockchainStorage {
    /// Create a new blockchain storage instance
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Store a block
    pub async fn store_block(&self, block: &Block) -> BeaconResult<()> {
        let block_data = bincode::serialize(block)?;
        let block_key = Keys::block(block.header.index);
        let hash_key = Keys::block_hash(&block.hash);

        // Create a batch to store both the block and its hash index atomically
        let mut batch = self.db.create_batch();
        batch.put_cf(self.db.cf_handle(CF_BLOCKS)?, &block_key, &block_data);
        batch.put_cf(self.db.cf_handle(CF_BLOCKS)?, &hash_key, &block.header.index.to_le_bytes());

        self.db.write_batch(batch)?;
        tracing::debug!("Stored block {} with hash {}", block.header.index, block.hash);

        Ok(())
    }

    /// Get a block by index
    pub async fn get_block_by_index(&self, index: BlockIndex) -> BeaconResult<Option<Block>> {
        let key = Keys::block(index);
        if let Some(data) = self.db.get_cf(CF_BLOCKS, &key)? {
            let block: Block = bincode::deserialize(&data)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    /// Get a block by hash
    pub async fn get_block_by_hash(&self, hash: &str) -> BeaconResult<Option<Block>> {
        let hash_key = Keys::block_hash(hash);
        if let Some(index_data) = self.db.get_cf(CF_BLOCKS, &hash_key)? {
            let index = BlockIndex::from_le_bytes(
                index_data.try_into()
                    .map_err(|_| beacon_core::BeaconError::storage("Invalid block index data"))?
            );
            self.get_block_by_index(index).await
        } else {
            Ok(None)
        }
    }

    /// Get the latest block
    pub async fn get_latest_block(&self) -> BeaconResult<Option<Block>> {
        let latest_index = self.get_latest_block_index().await?;
        if let Some(index) = latest_index {
            self.get_block_by_index(index).await
        } else {
            Ok(None)
        }
    }

    /// Get the latest block index
    pub async fn get_latest_block_index(&self) -> BeaconResult<Option<BlockIndex>> {
        // Iterate backwards through possible block indices
        // In a real implementation, we'd store this as metadata
        let mut iter = self.db.iter_cf_mode(CF_BLOCKS, rocksdb::IteratorMode::End)?;
        
        while let Some(result) = iter.next() {
            match result {
                Ok((key, _)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    if key_str.starts_with("block:") {
                        let index_str = &key_str[6..]; // Remove "block:" prefix
                        if let Ok(index) = index_str.parse::<BlockIndex>() {
                            return Ok(Some(index));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error iterating blocks: {}", e);
                    break;
                }
            }
        }
        
        Ok(None)
    }

    /// Get block count
    pub async fn get_block_count(&self) -> BeaconResult<u64> {
        if let Some(latest_index) = self.get_latest_block_index().await? {
            Ok(latest_index + 1) // Index is 0-based
        } else {
            Ok(0)
        }
    }

    /// Check if a block exists
    pub async fn block_exists(&self, index: BlockIndex) -> BeaconResult<bool> {
        let key = Keys::block(index);
        Ok(self.db.get_cf(CF_BLOCKS, &key)?.is_some())
    }

    /// Get multiple blocks in a range
    pub async fn get_blocks_range(&self, start: BlockIndex, count: u32) -> BeaconResult<Vec<Block>> {
        let mut blocks = Vec::new();
        
        for i in 0..count {
            let index = start + i as u64;
            if let Some(block) = self.get_block_by_index(index).await? {
                blocks.push(block);
            } else {
                break; // Stop if we reach a non-existent block
            }
        }
        
        Ok(blocks)
    }

    /// Store the genesis block
    pub async fn store_genesis_block(&self, network_id: &str) -> BeaconResult<Block> {
        let genesis = Block::genesis(network_id);
        self.store_block(&genesis).await?;
        tracing::info!("Stored genesis block for network: {}", network_id);
        Ok(genesis)
    }

    /// Initialize blockchain storage
    pub async fn initialize(&self, network_id: &str) -> BeaconResult<()> {
        // Check if we already have a genesis block
        if self.get_block_by_index(0).await?.is_none() {
            self.store_genesis_block(network_id).await?;
        }
        Ok(())
    }
}

/// Blockchain statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub total_blocks: u64,
    pub latest_block_index: Option<BlockIndex>,
    pub latest_block_hash: Option<String>,
    pub genesis_hash: Option<String>,
}

impl BlockchainStorage {
    /// Get blockchain statistics
    pub async fn get_stats(&self) -> BeaconResult<BlockchainStats> {
        let total_blocks = self.get_block_count().await?;
        let latest_block = self.get_latest_block().await?;
        let genesis_block = self.get_block_by_index(0).await?;

        Ok(BlockchainStats {
            total_blocks,
            latest_block_index: latest_block.as_ref().map(|b| b.header.index),
            latest_block_hash: latest_block.map(|b| b.hash),
            genesis_hash: genesis_block.map(|b| b.hash),
        })
    }
}
