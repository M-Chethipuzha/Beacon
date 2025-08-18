use axum::{
    extract::{Request, ConnectInfo},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn, error};
use std::{
    net::SocketAddr,
    time::Instant,
};

/// Logging middleware that logs all HTTP requests and responses
pub async fn logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    
    // Extract user agent and other relevant headers
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|header| header.to_str().ok())
        .unwrap_or("unknown");
    
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Log incoming request
    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        version = ?version,
        remote_addr = %addr,
        user_agent = %user_agent,
        "Incoming request"
    );
    
    // Process request
    let response = next.run(request).await;
    
    // Calculate response time
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Log response based on status code
    if status.is_success() {
        info!(
            request_id = %request_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed successfully"
        );
    } else if status.is_client_error() {
        warn!(
            request_id = %request_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with client error"
        );
    } else if status.is_server_error() {
        error!(
            request_id = %request_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with server error"
        );
    } else {
        info!(
            request_id = %request_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }
    
    response
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'".parse().unwrap()
    );
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap()
    );
    
    response
}

/// CORS middleware for API access
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add CORS headers (configure based on your requirements)
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods", 
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap()
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-Requested-With".parse().unwrap()
    );
    headers.insert("Access-Control-Max-Age", "3600".parse().unwrap());
    
    response
}

/// Request size limiting middleware
pub async fn request_size_limit_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Check content-length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024; // 10MB
                
                if length > MAX_REQUEST_SIZE {
                    warn!(
                        content_length = %length,
                        max_allowed = %MAX_REQUEST_SIZE,
                        "Request rejected due to size limit"
                    );
                    
                    return Response::builder()
                        .status(413) // Payload Too Large
                        .body("Request too large".into())
                        .unwrap();
                }
            }
        }
    }
    
    next.run(request).await
}

/// Timeout middleware
pub async fn timeout_middleware(
    request: Request,
    next: Next,
) -> Response {
    const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    
    match tokio::time::timeout(REQUEST_TIMEOUT, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            error!("Request timed out");
            Response::builder()
                .status(408) // Request Timeout
                .body("Request timeout".into())
                .unwrap()
        }
    }
}

/// Health check middleware (bypass other middleware for health endpoints)
pub async fn health_check_bypass_middleware(
    request: Request,
    next: Next,
) -> Response {
    // If this is a health check endpoint, skip heavy middleware
    if request.uri().path() == "/health" || request.uri().path() == "/api/v1/health" {
        // Simple health response without going through full middleware stack
        return Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"healthy","timestamp":"2024-01-01T00:00:00Z"}"#.into())
            .unwrap();
    }
    
    next.run(request).await
}
