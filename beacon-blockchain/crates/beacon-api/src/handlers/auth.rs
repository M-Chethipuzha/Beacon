use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use crate::server::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub node_id: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
    pub user: UserInfo,
    pub permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (user identifier)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
    pub role: String,
    pub permissions: Vec<String>,
    pub node_id: Option<String>,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
    pub node_id: Option<String>,
    pub last_login: String,
}

// JWT secret - in production this should come from environment or secure storage
const JWT_SECRET: &[u8] = b"beacon_blockchain_jwt_secret_change_in_production";

/// User login endpoint
pub async fn login(
    State(_state): State<AppState>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Validate credentials (simplified - in production would validate against secure user store)
    let (role, permissions) = match validate_credentials(&login_request.username, &login_request.password) {
        Some((role, perms)) => (role, perms),
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Create JWT claims
    let now = Utc::now();
    let expires_at = now + Duration::hours(24); // Token expires in 24 hours
    
    let claims = Claims {
        sub: login_request.username.clone(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role.clone(),
        permissions: permissions.clone(),
        node_id: login_request.node_id.clone(),
    };
    
    // Generate JWT token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let user_info = UserInfo {
        username: login_request.username,
        role,
        node_id: login_request.node_id,
        last_login: now.to_rfc3339(),
    };
    
    let response = LoginResponse {
        token,
        expires_at: expires_at.to_rfc3339(),
        user: user_info,
        permissions,
    };
    
    Ok(Json(response))
}

/// Verify JWT token
pub async fn verify_token(token: &str) -> Result<Claims, StatusCode> {
    let validation = Validation::default();
    
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation,
    ) {
        Ok(token_data) => {
            // Check if token is expired
            let now = Utc::now().timestamp() as usize;
            if token_data.claims.exp < now {
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            Ok(token_data.claims)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Get current user info from token
pub async fn get_user_info(
    State(_state): State<AppState>,
) -> Result<Json<UserInfo>, StatusCode> {
    // In a real implementation, extract user from authenticated request
    let user_info = UserInfo {
        username: "authenticated_user".to_string(),
        role: "admin".to_string(),
        node_id: None,
        last_login: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(Json(user_info))
}

/// Logout endpoint (token blacklisting would be implemented here)
pub async fn logout() -> Result<Json<Value>, StatusCode> {
    // In a production system, you would add the token to a blacklist
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

/// Refresh token endpoint
pub async fn refresh_token(
    State(_state): State<AppState>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // In a real implementation, extract current claims from authenticated request
    let username = "authenticated_user".to_string();
    let role = "admin".to_string();
    let permissions = vec![
        "read:blockchain".to_string(),
        "write:transactions".to_string(),
        "admin:node".to_string(),
        "invoke:chaincode".to_string(),
        "read:state".to_string(),
        "write:state".to_string(),
    ];
    
    // Generate new token with extended expiration
    let now = Utc::now();
    let expires_at = now + Duration::hours(24);
    
    let new_claims = Claims {
        sub: username.clone(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role.clone(),
        permissions: permissions.clone(),
        node_id: None,
    };
    
    let token = encode(
        &Header::default(),
        &new_claims,
        &EncodingKey::from_secret(JWT_SECRET),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let user_info = UserInfo {
        username,
        role,
        node_id: None,
        last_login: now.to_rfc3339(),
    };
    
    let response = LoginResponse {
        token,
        expires_at: expires_at.to_rfc3339(),
        user: user_info,
        permissions,
    };
    
    Ok(Json(response))
}

/// Validate user credentials (simplified implementation)
fn validate_credentials(username: &str, password: &str) -> Option<(String, Vec<String>)> {
    // In production, this would validate against a secure user database
    match (username, password) {
        ("admin", "admin123") => Some((
            "admin".to_string(),
            vec![
                "read:blockchain".to_string(),
                "write:transactions".to_string(),
                "admin:node".to_string(),
                "invoke:chaincode".to_string(),
                "read:state".to_string(),
                "write:state".to_string(),
            ]
        )),
        ("operator", "operator123") => Some((
            "operator".to_string(),
            vec![
                "read:blockchain".to_string(),
                "write:transactions".to_string(),
                "invoke:chaincode".to_string(),
                "read:state".to_string(),
            ]
        )),
        ("viewer", "viewer123") => Some((
            "viewer".to_string(),
            vec![
                "read:blockchain".to_string(),
                "read:state".to_string(),
            ]
        )),
        ("gateway", "gateway123") => Some((
            "gateway".to_string(),
            vec![
                "read:blockchain".to_string(),
                "write:transactions".to_string(),
                "invoke:chaincode".to_string(),
                "read:state".to_string(),
                "gateway:heartbeat".to_string(),
            ]
        )),
        _ => None,
    }
}

/// Check if user has specific permission
pub fn has_permission(claims: &Claims, required_permission: &str) -> bool {
    // Admin role has all permissions
    if claims.role == "admin" {
        return true;
    }
    
    // Check specific permission
    claims.permissions.contains(&required_permission.to_string())
}

/// Get API key info (for API key authentication alternative to JWT)
pub async fn get_api_key_info(
    api_key: &str,
) -> Option<Claims> {
    // Simplified API key validation - in production would use secure storage
    match api_key {
        "beacon_admin_key_12345" => Some(Claims {
            sub: "api_admin".to_string(),
            exp: (Utc::now() + Duration::days(365)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            role: "admin".to_string(),
            permissions: vec![
                "read:blockchain".to_string(),
                "write:transactions".to_string(),
                "admin:node".to_string(),
                "invoke:chaincode".to_string(),
                "read:state".to_string(),
                "write:state".to_string(),
            ],
            node_id: None,
        }),
        "beacon_gateway_key_67890" => Some(Claims {
            sub: "api_gateway".to_string(),
            exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            role: "gateway".to_string(),
            permissions: vec![
                "read:blockchain".to_string(),
                "write:transactions".to_string(),
                "invoke:chaincode".to_string(),
                "read:state".to_string(),
                "gateway:heartbeat".to_string(),
            ],
            node_id: Some("gateway_001".to_string()),
        }),
        _ => None,
    }
}
