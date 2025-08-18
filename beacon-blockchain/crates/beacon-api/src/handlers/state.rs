use axum::{extract::{Query, State}, response::Json, http::StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use chrono::Utc;
use crate::server::AppState;

#[derive(Deserialize)]
pub struct StateQuery {
    pub key: String,
    pub chaincode_id: Option<String>,
    pub channel_id: Option<String>,
}

#[derive(Deserialize)]
pub struct StateRangeQuery {
    pub start_key: String,
    pub end_key: String,
    pub chaincode_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct StateHistoryQuery {
    pub key: String,
    pub chaincode_id: Option<String>,
    pub limit: Option<u32>,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
}

pub async fn get_state(
    State(_state): State<AppState>,
    Query(params): Query<StateQuery>,
) -> Result<Json<Value>, StatusCode> {
    // Mock state retrieval
    Ok(Json(json!({
        "key": params.key,
        "value": format!("mock_value_for_{}", params.key),
        "chaincode_id": params.chaincode_id.unwrap_or("default".to_string()),
        "channel_id": params.channel_id.unwrap_or("default".to_string()),
        "block_number": 12345,
        "transaction_id": "mock_tx_id",
        "timestamp": Utc::now().to_rfc3339(),
        "version": {
            "block_num": 12345,
            "tx_num": 1
        }
    })))
}

pub async fn query_state(
    State(_state): State<AppState>,
    Query(params): Query<StateRangeQuery>,
) -> Result<Json<Value>, StatusCode> {
    let limit = params.limit.unwrap_or(10).min(100);
    
    // Mock state range query
    let mut results = Vec::new();
    for i in 0..limit {
        let key = format!("{}_{}", params.start_key, i);
        results.push(json!({
            "key": key,
            "value": format!("mock_value_for_{}", key),
            "chaincode_id": params.chaincode_id.as_ref().unwrap_or(&"default".to_string()),
            "block_number": 12340 + i,
            "transaction_id": format!("mock_tx_{}", i),
            "timestamp": Utc::now().to_rfc3339()
        }));
    }
    
    Ok(Json(json!({
        "results": results,
        "range": {
            "start_key": params.start_key,
            "end_key": params.end_key,
            "limit": limit
        },
        "has_more": limit == 100
    })))
}

pub async fn get_state_history(
    State(_state): State<AppState>,
    Query(params): Query<StateHistoryQuery>,
) -> Result<Json<Value>, StatusCode> {
    let limit = params.limit.unwrap_or(10).min(100);
    
    // Mock state history
    let mut history = Vec::new();
    for i in 0..limit {
        let block_num = 12340 + i as u64;
        history.push(json!({
            "key": params.key,
            "value": format!("historical_value_{}_{}", params.key, i),
            "chaincode_id": params.chaincode_id.as_ref().unwrap_or(&"default".to_string()),
            "block_number": block_num,
            "transaction_id": format!("historical_tx_{}", i),
            "timestamp": Utc::now().to_rfc3339(),
            "is_delete": false,
            "version": {
                "block_num": block_num,
                "tx_num": 1
            }
        }));
    }
    
    Ok(Json(json!({
        "key": params.key,
        "history": history,
        "pagination": {
            "limit": limit,
            "from_block": params.from_block,
            "to_block": params.to_block,
            "has_more": false
        }
    })))
}
