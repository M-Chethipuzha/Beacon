// API integration tests
use super::init_test_env;
use beacon_api::handlers;
use beacon_storage::SQLiteStorage;
use beacon_core::types::{Transaction, Block};
use std::sync::Arc;
use tokio;
use chrono::Utc;

#[tokio::test]
async fn test_api_storage_transaction_flow() {
    init_test_env();
    
    // Setup storage
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    
    // Create test transaction
    let test_tx = Transaction {
        id: "tx_integration_001".to_string(),
        chaincode_id: "test-contract".to_string(),
        function: "setValue".to_string(),
        args: vec!["key1".to_string(), "value1".to_string()],
        timestamp: Utc::now(),
        signature: "test_signature".to_string(),
    };
    
    // Test transaction submission through API handler
    let result = handlers::submit_transaction(storage.clone(), test_tx.clone()).await;
    assert!(result.is_ok(), "Transaction submission failed: {:?}", result);
    
    // Verify transaction was stored
    let stored_tx = storage.get_transaction(&test_tx.id).await.unwrap();
    assert_eq!(stored_tx.id, test_tx.id);
    assert_eq!(stored_tx.function, test_tx.function);
    assert_eq!(stored_tx.chaincode_id, test_tx.chaincode_id);
}

#[tokio::test]
async fn test_state_query_integration() {
    init_test_env();
    
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    
    // Store test state directly
    storage.put_state("test-contract", "integration_key", "integration_value", 1).await.unwrap();
    
    // Test API state query handler
    let result = handlers::query_state(
        storage.clone(),
        "test-contract".to_string(),
        "integration_key".to_string()
    ).await;
    
    assert!(result.is_ok(), "State query failed: {:?}", result);
    let state_value = result.unwrap();
    assert_eq!(state_value, "integration_value");
}

#[tokio::test]
async fn test_blockchain_info_with_blocks() {
    init_test_env();
    
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    
    // Create and store test blocks
    for i in 0..3 {
        let block = Block {
            number: i,
            hash: format!("hash_{:04}", i),
            previous_hash: if i == 0 { 
                "genesis".to_string() 
            } else { 
                format!("hash_{:04}", i-1) 
            },
            timestamp: Utc::now(),
            transactions: vec![],
        };
        storage.store_block(&block).await.unwrap();
    }
    
    // Test blockchain info handler
    let info = handlers::get_blockchain_info(storage.clone()).await.unwrap();
    assert_eq!(info.latest_block_number, 2);
    assert_eq!(info.total_blocks, 3);
    assert_eq!(info.network_id, "beacon-testnet");
}

#[tokio::test]
async fn test_range_query_integration() {
    init_test_env();
    
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    
    // Setup range test data
    let test_data = vec![
        ("product_001", "Laptop"),
        ("product_002", "Mouse"),
        ("product_003", "Keyboard"),
        ("product_999", "Monitor"),
    ];
    
    for (key, value) in &test_data {
        storage.put_state("test-contract", key, value, 1).await.unwrap();
    }
    
    // Test range query through API
    let results = handlers::query_state_range(
        storage.clone(),
        "test-contract".to_string(),
        "product_001".to_string(),
        "product_500".to_string(),
        10
    ).await.unwrap();
    
    assert_eq!(results.len(), 3); // Should get product_001, 002, 003 but not 999
    
    // Verify results are in order
    assert_eq!(results[0].key, "product_001");
    assert_eq!(results[1].key, "product_002");
    assert_eq!(results[2].key, "product_003");
}

#[tokio::test]
async fn test_transaction_history_integration() {
    init_test_env();
    
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    
    // Create multiple transactions for history
    let transactions = vec![
        Transaction {
            id: "tx_hist_001".to_string(),
            chaincode_id: "test-contract".to_string(),
            function: "createAsset".to_string(),
            args: vec!["asset1".to_string(), "value1".to_string()],
            timestamp: Utc::now(),
            signature: "sig1".to_string(),
        },
        Transaction {
            id: "tx_hist_002".to_string(),
            chaincode_id: "test-contract".to_string(),
            function: "updateAsset".to_string(),
            args: vec!["asset1".to_string(), "value2".to_string()],
            timestamp: Utc::now(),
            signature: "sig2".to_string(),
        },
    ];
    
    // Store transactions
    for tx in &transactions {
        storage.store_transaction(tx).await.unwrap();
    }
    
    // Test transaction history query
    let history = handlers::get_transaction_history(
        storage.clone(),
        "test-contract".to_string(),
        10
    ).await.unwrap();
    
    assert_eq!(history.len(), 2);
    assert!(history.iter().any(|tx| tx.id == "tx_hist_001"));
    assert!(history.iter().any(|tx| tx.id == "tx_hist_002"));
}

#[tokio::test]
async fn test_concurrent_api_storage_operations() {
    init_test_env();
    
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    let mut join_set = tokio::task::JoinSet::new();
    
    // Spawn concurrent operations
    for i in 0..10 {
        let storage_clone = storage.clone();
        join_set.spawn(async move {
            let tx = Transaction {
                id: format!("concurrent_tx_{:03}", i),
                chaincode_id: "test-contract".to_string(),
                function: "setValue".to_string(),
                args: vec![format!("key_{}", i), format!("value_{}", i)],
                timestamp: Utc::now(),
                signature: format!("sig_{}", i),
            };
            
            // Submit through API handler
            handlers::submit_transaction(storage_clone.clone(), tx.clone()).await.unwrap();
            
            // Query back immediately
            let stored = storage_clone.get_transaction(&tx.id).await.unwrap();
            assert_eq!(stored.id, tx.id);
            
            i
        });
    }
    
    // Wait for all operations to complete
    let mut completed = 0;
    while let Some(result) = join_set.join_next().await {
        assert!(result.is_ok());
        completed += 1;
    }
    
    assert_eq!(completed, 10);
    
    // Verify all transactions were stored correctly
    for i in 0..10 {
        let tx_id = format!("concurrent_tx_{:03}", i);
        let stored_tx = storage.get_transaction(&tx_id).await.unwrap();
        assert_eq!(stored_tx.id, tx_id);
    }
}
