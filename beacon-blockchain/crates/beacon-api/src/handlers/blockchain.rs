use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use crate::server::AppState;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct BlockQuery {
    pub include_transactions: Option<bool>,
    pub include_validators: Option<bool>,
}

#[derive(Deserialize)] 
pub struct LatestBlocksQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_transactions: Option<bool>,
}

/// Get a specific block by number
pub async fn get_block(
    State(_state): State<AppState>,
    Path(block_number): Path<u64>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<Value>, StatusCode> {
    let include_transactions = query.include_transactions.unwrap_or(false);
    
    // Mock block data
    let mut block = serde_json::json!({
        "number": block_number,
        "hash": format!("0x{:064x}", block_number * 1234567890),
        "parent_hash": if block_number > 0 { 
            Some(format!("0x{:064x}", (block_number - 1) * 1234567890))
        } else { 
            None 
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "validator": format!("validator_{}", block_number % 5),
        "size": 2048 + (block_number * 100) % 5000,
        "gas_limit": 10000000,
        "gas_used": 5000000 + (block_number * 12345) % 2000000,
        "transaction_count": 10 + (block_number % 20),
        "difficulty": 1000000,
        "extra_data": "0x424541434f4e"
    });
    
    if include_transactions {
        let transactions: Vec<Value> = (0..5).map(|i| {
            serde_json::json!({
                "hash": format!("0x{:064x}", block_number * 1000 + i),
                "from": format!("0x{:040x}", i * 12345),
                "to": format!("0x{:040x}", (i + 1) * 12345),
                "value": format!("{}", (i + 1) * 100),
                "gas": 21000,
                "gas_price": 20,
                "nonce": i,
                "status": "success"
            })
        }).collect();
        
        block["transactions"] = serde_json::json!(transactions);
    }
    
    Ok(Json(block))
}

/// Get a specific block by hash
pub async fn get_block_by_hash(
    State(_state): State<AppState>,
    Path(block_hash): Path<String>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<Value>, StatusCode> {
    // Extract block number from hash for mock purposes
    if let Ok(block_number) = u64::from_str_radix(&block_hash[2..], 16) {
        let derived_number = block_number / 1234567890;
        get_block(State(_state), Path(derived_number), Query(query)).await
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get the latest blocks
pub async fn get_latest_blocks(
    State(_state): State<AppState>,
    Query(query): Query<LatestBlocksQuery>,
) -> Result<Json<Value>, StatusCode> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0);
    let include_transactions = query.include_transactions.unwrap_or(false);
    
    // Mock latest block number
    let latest_block_number = 1000u64;
    
    let blocks: Vec<Value> = (0..limit).map(|i| {
        let block_number = latest_block_number - offset as u64 - i as u64;
        
        let mut block = serde_json::json!({
            "number": block_number,
            "hash": format!("0x{:064x}", block_number * 1234567890),
            "parent_hash": if block_number > 0 { 
                Some(format!("0x{:064x}", (block_number - 1) * 1234567890))
            } else { 
                None 
            },
            "timestamp": chrono::Utc::now() - chrono::Duration::seconds((i * 2) as i64),
            "validator": format!("validator_{}", block_number % 5),
            "size": 2048 + (block_number * 100) % 5000,
            "transaction_count": 5 + (block_number % 15),
            "gas_used": 3000000 + (block_number * 12345) % 2000000,
            "confirmations": i + 1
        });
        
        if include_transactions {
            let tx_count = 3 + (i % 5);
            let transactions: Vec<Value> = (0..tx_count).map(|j| {
                serde_json::json!({
                    "hash": format!("0x{:064x}", block_number * 1000 + j as u64),
                    "from": format!("0x{:040x}", j * 12345),
                    "to": format!("0x{:040x}", (j + 1) * 12345),
                    "value": format!("{}", (j + 1) * 100),
                    "status": "success"
                })
            }).collect();
            
            block["transactions"] = serde_json::json!(transactions);
        }
        
        block
    }).collect();
    
    let response = serde_json::json!({
        "blocks": blocks,
        "pagination": {
            "limit": limit,
            "offset": offset,
            "latest_block_number": latest_block_number,
            "has_more": offset + limit < latest_block_number as usize
        }
    });
    
    Ok(Json(response))
}

/// Get blockchain information and statistics
pub async fn get_blockchain_info(
    State(_state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let include_validators = query.get("include_validators")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    let mut info = serde_json::json!({
        "network": "BEACON Mainnet",
        "chain_id": "beacon-1",
        "genesis_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest_block": {
            "number": 1000,
            "hash": format!("0x{:064x}", 1000 * 1234567890),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "transaction_count": 15
        },
        "statistics": {
            "total_blocks": 1001,
            "total_transactions": 15000,
            "avg_block_time": 2.0,
            "avg_transaction_fee": "0.001",
            "active_validators": 5,
            "total_supply": "1000000",
            "circulating_supply": "800000"
        },
        "consensus": {
            "algorithm": "Proof of Stake",
            "block_time": "2 seconds",
            "finality": "2 blocks"
        },
        "network_health": {
            "status": "healthy",
            "peer_count": 25,
            "sync_status": "synced",
            "last_sync": chrono::Utc::now().to_rfc3339()
        }
    });
    
    if include_validators {
        let validators = vec![
            serde_json::json!({
                "address": "beacon1qyfe0mz2tx65j8fcp6k8y5d2nqr2kcpz50w9a7l",
                "voting_power": "200000",
                "commission": "5%",
                "status": "active",
                "uptime": "99.8%"
            }),
            serde_json::json!({
                "address": "beacon1qzx2vwy3nw8xv2ljr7d8y9k5m8n4r6t7u8v9w0x",
                "voting_power": "180000", 
                "commission": "3%",
                "status": "active",
                "uptime": "99.9%"
            }),
            serde_json::json!({
                "address": "beacon1qa1s2d3f4g5h6j7k8l9z0x1c2v3b4n5m6q7w8e9r",
                "voting_power": "150000",
                "commission": "7%", 
                "status": "active",
                "uptime": "98.5%"
            })
        ];
        
        info["validators"] = serde_json::json!(validators);
    }
    
    Ok(Json(info))
}
