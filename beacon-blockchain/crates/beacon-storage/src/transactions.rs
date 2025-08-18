use crate::{Database, Keys, CF_TRANSACTIONS, CF_INDICES};
use beacon_core::{BeaconResult, Transaction, TransactionId, TransactionResult, BlockIndex};
use std::sync::Arc;

/// Transaction storage manager
pub struct TransactionStorage {
    db: Arc<Database>,
}

impl TransactionStorage {
    /// Create a new transaction storage instance
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Store a transaction
    pub async fn store_transaction(&self, transaction: &Transaction) -> BeaconResult<()> {
        let tx_data = bincode::serialize(transaction)?;
        let tx_key = Keys::transaction(transaction.id.as_str());

        self.db.put_cf(CF_TRANSACTIONS, &tx_key, &tx_data)?;
        tracing::debug!("Stored transaction: {}", transaction.id.as_str());

        Ok(())
    }

    /// Store a transaction with its result
    pub async fn store_transaction_with_result(
        &self,
        transaction: &Transaction,
        result: &TransactionResult,
        block_index: BlockIndex,
        tx_index: usize,
    ) -> BeaconResult<()> {
        let tx_data = bincode::serialize(transaction)?;
        let result_data = bincode::serialize(result)?;
        let tx_key = Keys::transaction(transaction.id.as_str());
        let result_key = format!("{}:result", transaction.id.as_str());
        let block_index_key = Keys::transaction_by_block(block_index, tx_index);

        let mut batch = self.db.create_batch();
        let tx_cf = self.db.cf_handle(CF_TRANSACTIONS)?;
        let idx_cf = self.db.cf_handle(CF_INDICES)?;

        // Store transaction
        batch.put_cf(tx_cf, &tx_key, &tx_data);
        
        // Store transaction result
        batch.put_cf(tx_cf, result_key.as_bytes(), &result_data);
        
        // Store block index reference
        batch.put_cf(idx_cf, &block_index_key, transaction.id.as_str().as_bytes());

        self.db.write_batch(batch)?;
        tracing::debug!(
            "Stored transaction {} with result in block {} at index {}",
            transaction.id.as_str(),
            block_index,
            tx_index
        );

        Ok(())
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, tx_id: &TransactionId) -> BeaconResult<Option<Transaction>> {
        let tx_key = Keys::transaction(tx_id.as_str());
        if let Some(data) = self.db.get_cf(CF_TRANSACTIONS, &tx_key)? {
            let transaction: Transaction = bincode::deserialize(&data)?;
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }

    /// Get a transaction result by transaction ID
    pub async fn get_transaction_result(&self, tx_id: &TransactionId) -> BeaconResult<Option<TransactionResult>> {
        let result_key = format!("{}:result", tx_id.as_str());
        if let Some(data) = self.db.get_cf(CF_TRANSACTIONS, result_key.as_bytes())? {
            let result: TransactionResult = bincode::deserialize(&data)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Get all transactions in a block
    pub async fn get_transactions_in_block(&self, block_index: BlockIndex) -> BeaconResult<Vec<Transaction>> {
        let mut transactions = Vec::new();
        let prefix = format!("tx_block:{:020}:", block_index);
        let prefix_bytes = prefix.as_bytes();

        let iter = self.db.iter_cf_mode(
            CF_INDICES,
            rocksdb::IteratorMode::From(prefix_bytes, rocksdb::Direction::Forward)
        )?;

        for item in iter {
            match item {
                Ok((key, value)) => {
                    if key.starts_with(prefix_bytes) {
                        let tx_id_str = String::from_utf8_lossy(&value);
                        let tx_id = TransactionId::from_string(tx_id_str.to_string());
                        if let Some(transaction) = self.get_transaction(&tx_id).await? {
                            transactions.push(transaction);
                        }
                    } else {
                        break; // We've gone past the prefix
                    }
                }
                Err(e) => {
                    tracing::warn!("Error iterating transactions in block {}: {}", block_index, e);
                    break;
                }
            }
        }

        // Sort by transaction index in block
        transactions.sort_by_key(|_tx| {
            // Extract tx_index from the database iteration order
            // This is a simplified approach - in production, you'd store the index explicitly
            0 // For now, maintain original order
        });

        Ok(transactions)
    }

    /// Check if a transaction exists
    pub async fn transaction_exists(&self, tx_id: &TransactionId) -> BeaconResult<bool> {
        let tx_key = Keys::transaction(tx_id.as_str());
        Ok(self.db.get_cf(CF_TRANSACTIONS, &tx_key)?.is_some())
    }

    /// Get transactions by sender address
    pub async fn get_transactions_by_sender(&self, sender: &str) -> BeaconResult<Vec<Transaction>> {
        let mut transactions = Vec::new();
        let iter = self.db.iter_cf(CF_TRANSACTIONS)?;

        for item in iter {
            match item {
                Ok((key, value)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    // Only process transaction keys (not result keys)
                    if key_str.starts_with("tx:") && !key_str.contains(":result") {
                        if let Ok(transaction) = bincode::deserialize::<Transaction>(&value) {
                            if transaction.from.as_str() == sender {
                                transactions.push(transaction);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error iterating transactions by sender: {}", e);
                    break;
                }
            }
        }

        // Sort by timestamp (newest first)
        transactions.sort_by(|a, b| b.timestamp.0.cmp(&a.timestamp.0));
        Ok(transactions)
    }

    /// Get recent transactions (last N transactions)
    pub async fn get_recent_transactions(&self, limit: usize) -> BeaconResult<Vec<Transaction>> {
        let mut transactions = Vec::new();
        let iter = self.db.iter_cf_mode(CF_TRANSACTIONS, rocksdb::IteratorMode::End)?;

        for item in iter {
            match item {
                Ok((key, value)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    // Only process transaction keys (not result keys)
                    if key_str.starts_with("tx:") && !key_str.contains(":result") {
                        if let Ok(transaction) = bincode::deserialize::<Transaction>(&value) {
                            transactions.push(transaction);
                            if transactions.len() >= limit {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error iterating recent transactions: {}", e);
                    break;
                }
            }
        }

        // Sort by timestamp (newest first)
        transactions.sort_by(|a, b| b.timestamp.0.cmp(&a.timestamp.0));
        Ok(transactions)
    }

    /// Get transaction count
    pub async fn get_transaction_count(&self) -> BeaconResult<u64> {
        let mut count = 0u64;
        let iter = self.db.iter_cf(CF_TRANSACTIONS)?;

        for item in iter {
            match item {
                Ok((key, _)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    // Only count transaction keys (not result keys)
                    if key_str.starts_with("tx:") && !key_str.contains(":result") {
                        count += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!("Error counting transactions: {}", e);
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Delete a transaction (use with caution)
    pub async fn delete_transaction(&self, tx_id: &TransactionId) -> BeaconResult<()> {
        let tx_key = Keys::transaction(tx_id.as_str());
        let result_key = format!("{}:result", tx_id.as_str());

        let mut batch = self.db.create_batch();
        let tx_cf = self.db.cf_handle(CF_TRANSACTIONS)?;

        batch.delete_cf(tx_cf, &tx_key);
        batch.delete_cf(tx_cf, result_key.as_bytes());

        self.db.write_batch(batch)?;
        tracing::debug!("Deleted transaction: {}", tx_id.as_str());

        Ok(())
    }

    /// Create indices for faster querying
    pub async fn create_indices(&self) -> BeaconResult<()> {
        // This would create additional indices for common queries
        // For now, we'll just log that indices are being created
        tracing::info!("Creating transaction indices");
        
        // In a real implementation, you might create indices for:
        // - Transactions by timestamp
        // - Transactions by type
        // - Transactions by chaincode
        // - etc.
        
        Ok(())
    }

    /// Rebuild indices (for maintenance)
    pub async fn rebuild_indices(&self) -> BeaconResult<()> {
        tracing::info!("Rebuilding transaction indices");
        
        // Clear existing indices
        // Rebuild from transaction data
        
        self.create_indices().await
    }
}
