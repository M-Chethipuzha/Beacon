use axum::{http::StatusCode, response::Json};
use serde_json::Value;

/// Simple health check endpoint
pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    let response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "beacon-api",
        "version": "1.0.0"
    });
    
    Ok(Json(response))
}
