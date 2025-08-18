// API test module entry point
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment for API tests
pub fn init_api_test_env() {
    INIT.call_once(|| {
        env_logger::init();
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("DATABASE_URL", ":memory:");
        std::env::set_var("JWT_SECRET", "test_secret_key_for_api_testing");
        std::env::set_var("API_PORT", "3001");
    });
}

// Include API test modules
mod auth_test;
mod blockchain_endpoints_test;
mod transaction_endpoints_test;
