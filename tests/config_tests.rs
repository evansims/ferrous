use ferrous::config::Config;
use std::env;
use std::sync::Mutex;

// Global mutex to serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

// Helper to clean up environment variables
fn cleanup_env_vars() {
    env::remove_var("PORT");
    env::remove_var("DATABASE_URL");
    env::remove_var("DATABASE_TYPE");
    env::remove_var("CONVEX_DEPLOYMENT_URL");
    env::remove_var("RUST_LOG");
    env::remove_var("SHUTDOWN_TIMEOUT_SECONDS");
}

#[test]
fn test_config_default_values() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    let config = Config::default();
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.database.db_type, "memory");
    assert_eq!(config.logging.rust_log, "ferrous=debug,tower_http=debug");
    assert_eq!(config.shutdown.timeout_seconds, 30);

    cleanup_env_vars();
}

#[test]
fn test_config_from_env() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    env::set_var("PORT", "8080");
    env::set_var("DATABASE_URL", "convex://my-deployment.convex.cloud");
    env::set_var("RUST_LOG", "info");
    env::set_var("SHUTDOWN_TIMEOUT_SECONDS", "60");

    let config = Config::load().unwrap();
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.database.db_type, "convex");
    assert_eq!(
        config.database.convex_deployment_url,
        Some("https://my-deployment.convex.cloud".to_string())
    );
    assert_eq!(config.logging.rust_log, "info");
    assert_eq!(config.shutdown.timeout_seconds, 60);

    cleanup_env_vars();
}

#[test]
fn test_config_validation_errors() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    // Test invalid port
    env::set_var("PORT", "0");
    let result = Config::load();
    assert!(result.is_err());
    env::remove_var("PORT");

    // Test invalid port string
    env::set_var("PORT", "not-a-number");
    let result = Config::load();
    assert!(result.is_err());
    env::remove_var("PORT");

    cleanup_env_vars();
}

#[test]
fn test_runtime_dependency_validation() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    let mut config = Config::default();

    // Convex without URL should fail
    config.database.db_type = "convex".to_string();
    config.database.convex_deployment_url = None;
    assert!(config.validate_runtime_dependencies().is_err());

    // Convex with URL should succeed
    config.database.convex_deployment_url = Some("https://example.convex.cloud".to_string());
    assert!(config.validate_runtime_dependencies().is_ok());

    // Memory database should always succeed
    config.database.db_type = "memory".to_string();
    config.database.convex_deployment_url = None;
    assert!(config.validate_runtime_dependencies().is_ok());

    cleanup_env_vars();
}

#[test]
fn test_database_url_parsing() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    // Test memory URL
    env::set_var("DATABASE_URL", "memory://");
    let config = Config::load().unwrap();
    assert_eq!(config.database.db_type, "memory");
    assert!(config.database.convex_deployment_url.is_none());
    env::remove_var("DATABASE_URL");

    // Test convex URL
    env::set_var("DATABASE_URL", "convex://my-app.convex.cloud");
    let config = Config::load().unwrap();
    assert_eq!(config.database.db_type, "convex");
    assert_eq!(
        config.database.convex_deployment_url,
        Some("https://my-app.convex.cloud".to_string())
    );
    env::remove_var("DATABASE_URL");

    cleanup_env_vars();
}

#[test]
fn test_legacy_database_type() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    // Test using DATABASE_TYPE instead of DATABASE_URL
    env::set_var("DATABASE_TYPE", "convex");
    env::set_var("CONVEX_DEPLOYMENT_URL", "https://my-app.convex.cloud");

    let config = Config::load().unwrap();
    assert_eq!(config.database.db_type, "convex");
    assert_eq!(
        config.database.convex_deployment_url,
        Some("https://my-app.convex.cloud".to_string())
    );

    cleanup_env_vars();
}
