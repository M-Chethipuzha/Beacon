use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::handlers::auth::{verify_token, get_api_key_info, has_permission, Claims};
use crate::server::AppState;

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

/// Authentication middleware that validates JWT tokens or API keys
pub async fn auth_middleware(
    State(_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    
    let claims = match auth_header {
        Some(auth_value) => {
            if auth_value.starts_with("Bearer ") {
                // JWT token authentication
                let token = &auth_value[7..]; // Remove "Bearer " prefix
                verify_token(token).await?
            } else if auth_value.starts_with("ApiKey ") {
                // API key authentication
                let api_key = &auth_value[7..]; // Remove "ApiKey " prefix
                get_api_key_info(api_key).await.ok_or(StatusCode::UNAUTHORIZED)?
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => {
            // Check for API key in query parameter (less secure, for testing)
            if let Some(api_key) = request.uri().query()
                .and_then(|q| {
                    q.split('&')
                        .find(|param| param.starts_with("api_key="))
                        .map(|param| &param[8..])
                }) {
                get_api_key_info(api_key).await.ok_or(StatusCode::UNAUTHORIZED)?
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    };
    
    // Add authenticated user to request extensions
    request.extensions_mut().insert(AuthenticatedUser { claims });
    
    Ok(next.run(request).await)
}

/// Permission checking middleware
pub fn require_permission(required_permission: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let permission = required_permission;
        Box::pin(async move {
            // Get authenticated user from request extensions
            let auth_user = request
                .extensions()
                .get::<AuthenticatedUser>()
                .ok_or(StatusCode::UNAUTHORIZED)?;
            
            // Check if user has required permission
            if !has_permission(&auth_user.claims, permission) {
                return Err(StatusCode::FORBIDDEN);
            }
            
            Ok(next.run(request).await)
        })
    }
}

/// Optional authentication middleware (allows unauthenticated requests)
pub async fn optional_auth_middleware(
    State(_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    
    if let Some(auth_value) = auth_header {
        if let Ok(claims) = if auth_value.starts_with("Bearer ") {
            let token = &auth_value[7..];
            verify_token(token).await
        } else if auth_value.starts_with("ApiKey ") {
            let api_key = &auth_value[7..];
            get_api_key_info(api_key).await.ok_or(StatusCode::UNAUTHORIZED)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        } {
            // Add authenticated user to request extensions if valid
            request.extensions_mut().insert(AuthenticatedUser { claims });
        }
    }
    
    Ok(next.run(request).await)
}

/// Rate limiting key extractor
pub fn get_rate_limit_key(request: &Request) -> String {
    // Try to get user ID from authenticated user
    if let Some(auth_user) = request.extensions().get::<AuthenticatedUser>() {
        format!("user:{}", auth_user.claims.sub)
    } else {
        // Fall back to IP address for unauthenticated requests
        request
            .headers()
            .get("x-forwarded-for")
            .and_then(|header| header.to_str().ok())
            .or_else(|| {
                request
                    .headers()
                    .get("x-real-ip")
                    .and_then(|header| header.to_str().ok())
            })
            .map(|ip| format!("ip:{}", ip))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Get rate limit based on user role
pub fn get_rate_limit_for_user(request: &Request) -> (u32, std::time::Duration) {
    if let Some(auth_user) = request.extensions().get::<AuthenticatedUser>() {
        match auth_user.claims.role.as_str() {
            "admin" => (1000, std::time::Duration::from_secs(60)), // 1000 requests per minute
            "operator" => (500, std::time::Duration::from_secs(60)), // 500 requests per minute
            "gateway" => (200, std::time::Duration::from_secs(60)), // 200 requests per minute
            _ => (100, std::time::Duration::from_secs(60)), // 100 requests per minute
        }
    } else {
        (50, std::time::Duration::from_secs(60)) // 50 requests per minute for unauthenticated
    }
}
