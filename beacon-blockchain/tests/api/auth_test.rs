// Authentication API tests
use super::init_api_test_env;
use beacon_api::app::create_app;
use beacon_storage::SQLiteStorage;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use serde_json::json;

async fn create_test_app() -> axum::Router {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    create_app(storage).await
}

#[tokio::test]
async fn test_valid_login() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "admin",
                "password": "admin123"
            }).to_string()
        )
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(response_data["access_token"].is_string());
    assert!(response_data["token_type"].as_str().unwrap() == "Bearer");
    assert!(response_data["expires_in"].is_number());
}

#[tokio::test]
async fn test_invalid_credentials() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "invalid_user",
                "password": "wrong_password"
            }).to_string()
        )
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(response_data["error"].is_string());
    assert_eq!(response_data["error"], "Invalid credentials");
}

#[tokio::test]
async fn test_missing_credentials() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "admin"
                // Missing password
            }).to_string()
        )
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_json() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body("{ invalid json }")
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_logout_with_valid_token() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    // First, login to get a token
    let login_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "admin",
                "password": "admin123"
            }).to_string()
        )
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
    
    let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_data: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
    let token = login_data["access_token"].as_str().unwrap();
    
    // Now test logout
    let logout_request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header("authorization", format!("Bearer {}", token))
        .body("".to_string())
        .unwrap();
    
    let logout_response = app.oneshot(logout_request).await.unwrap();
    
    assert_eq!(logout_response.status(), StatusCode::OK);
    
    let logout_body = axum::body::to_bytes(logout_response.into_body(), usize::MAX).await.unwrap();
    let logout_data: serde_json::Value = serde_json::from_slice(&logout_body).unwrap();
    
    assert_eq!(logout_data["message"], "Successfully logged out");
}

#[tokio::test]
async fn test_logout_without_token() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .body("".to_string())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_with_invalid_token() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header("authorization", "Bearer invalid_token")
        .body("".to_string())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_token_refresh() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    // Login first
    let login_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "admin",
                "password": "admin123"
            }).to_string()
        )
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_data: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
    let token = login_data["access_token"].as_str().unwrap();
    
    // Test token refresh
    let refresh_request = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header("authorization", format!("Bearer {}", token))
        .body("".to_string())
        .unwrap();
    
    let refresh_response = app.oneshot(refresh_request).await.unwrap();
    
    assert_eq!(refresh_response.status(), StatusCode::OK);
    
    let refresh_body = axum::body::to_bytes(refresh_response.into_body(), usize::MAX).await.unwrap();
    let refresh_data: serde_json::Value = serde_json::from_slice(&refresh_body).unwrap();
    
    assert!(refresh_data["access_token"].is_string());
    assert_ne!(refresh_data["access_token"], token); // New token should be different
}

#[tokio::test]
async fn test_protected_endpoint_without_auth() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/blockchain/info")
        .body("".to_string())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_auth() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    // Login first
    let login_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "admin",
                "password": "admin123"
            }).to_string()
        )
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_data: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
    let token = login_data["access_token"].as_str().unwrap();
    
    // Access protected endpoint
    let protected_request = Request::builder()
        .method("GET")
        .uri("/api/v1/blockchain/info")
        .header("authorization", format!("Bearer {}", token))
        .body("".to_string())
        .unwrap();
    
    let protected_response = app.oneshot(protected_request).await.unwrap();
    
    assert_eq!(protected_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_malformed_authorization_header() {
    init_api_test_env();
    
    let app = create_test_app().await;
    
    let test_cases = vec![
        "Bearer",           // Missing token
        "bearer token",     // Wrong case
        "Basic token",      // Wrong auth type
        "token",           // Missing Bearer prefix
        "",                // Empty header
    ];
    
    for auth_header in test_cases {
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/blockchain/info")
            .header("authorization", auth_header)
            .body("".to_string())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, 
                  "Auth header '{}' should be unauthorized", auth_header);
    }
}
