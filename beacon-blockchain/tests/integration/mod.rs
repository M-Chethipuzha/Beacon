// Integration test module entry point
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment once per test run
pub fn init_test_env() {
    INIT.call_once(|| {
        env_logger::init();
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("DATABASE_URL", ":memory:");
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_only");
    });
}

// Include integration test modules
mod api_storage_integration;
mod chaincode_integration;
mod full_stack_test;
