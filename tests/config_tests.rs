use estuary::config::{secrets::*, Config};
use std::env;
use std::sync::Mutex;

// Global mutex to serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

// Helper to clean up environment variables
fn cleanup_env_vars() {
    env::remove_var("PORT");
    env::remove_var("DATABASE_TYPE");
    env::remove_var("CONVEX_DEPLOYMENT_URL");
    env::remove_var("RATE_LIMIT_MAX_REQUESTS");
    env::remove_var("RATE_LIMIT_WINDOW_SECONDS");
    env::remove_var("RATE_LIMIT_ENABLED");
    env::remove_var("AUTH_ENABLED");
    env::remove_var("AUTH_JWKS_URLS");
    env::remove_var("AUTH_AUDIENCE");
    env::remove_var("AUTH_ISSUER");
    env::remove_var("AUTH_JWKS_CACHE_SECONDS");
    env::remove_var("SECURITY_STRICT_MODE");
    env::remove_var("SECURITY_CSP");
    env::remove_var("CORS_ALLOWED_ORIGINS");
    env::remove_var("SHUTDOWN_TIMEOUT_SECONDS");
    env::remove_var("APP_PROFILE");
    env::remove_var("RUST_LOG");
}

#[test]
fn test_config_default_values() {
    let _guard = ENV_MUTEX.lock().unwrap();
    // Clear any existing env vars
    cleanup_env_vars();

    let config = Config::default();

    assert_eq!(config.server.port, 3000);
    assert_eq!(config.database.db_type, "memory");
    assert!(!config.auth.enabled);
    assert!(config.rate_limit.enabled);
    assert_eq!(config.rate_limit.max_requests, 1000);
    assert!(!config.security.strict_mode);

    cleanup_env_vars();
}

#[test]
fn test_config_production_profile() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    env::set_var("APP_PROFILE", "production");

    let config = Config::default();

    assert!(config.security.strict_mode);
    assert_eq!(config.rate_limit.max_requests, 100); // More restrictive in production

    cleanup_env_vars();
}

#[test]
fn test_config_from_env() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    env::set_var("PORT", "8080");
    env::set_var("DATABASE_TYPE", "convex");
    env::set_var("CONVEX_DEPLOYMENT_URL", "https://test.convex.cloud");
    env::set_var("RATE_LIMIT_MAX_REQUESTS", "500");
    env::set_var("AUTH_ENABLED", "true");
    env::set_var("AUTH_JWKS_URLS", "https://auth1.com/jwks,https://auth2.com/jwks");

    let config = Config::load().unwrap();

    assert_eq!(config.server.port, 8080);
    assert_eq!(config.database.db_type, "convex");
    assert_eq!(
        config.database.convex_deployment_url,
        Some("https://test.convex.cloud".to_string())
    );
    assert_eq!(config.rate_limit.max_requests, 500);
    assert!(config.auth.enabled);
    assert_eq!(config.auth.jwks_urls.len(), 2);

    cleanup_env_vars();
}

#[test]
fn test_config_validation_errors() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    // Test invalid port
    env::set_var("PORT", "99999");
    let result = Config::load();
    assert!(result.is_err());
    env::remove_var("PORT");

    // Test invalid rate limit
    env::set_var("RATE_LIMIT_MAX_REQUESTS", "not-a-number");
    let result = Config::load();
    assert!(result.is_err());
    env::remove_var("RATE_LIMIT_MAX_REQUESTS");

    // Test invalid shutdown timeout - 301 is over the max of 300
    env::set_var("SHUTDOWN_TIMEOUT_SECONDS", "301");
    let result = Config::load();
    assert!(result.is_err());
    env::remove_var("SHUTDOWN_TIMEOUT_SECONDS");

    cleanup_env_vars();
}

#[test]
fn test_runtime_dependency_validation() {
    let mut config = Config::default();

    // Convex database without URL should fail
    config.database.db_type = "convex".to_string();
    config.database.convex_deployment_url = None;
    assert!(config.validate_runtime_dependencies().is_err());

    // With URL should succeed
    config.database.convex_deployment_url = Some("https://test.convex.cloud".to_string());
    assert!(config.validate_runtime_dependencies().is_ok());

    // Auth enabled without JWKS URLs should fail
    config.auth.enabled = true;
    config.auth.jwks_urls = vec![];
    assert!(config.validate_runtime_dependencies().is_err());

    // With JWKS URLs should succeed
    config.auth.jwks_urls = vec!["https://auth.example.com/jwks".to_string()];
    assert!(config.validate_runtime_dependencies().is_ok());
}

#[test]
fn test_secrets_redaction() {
    assert_eq!(SecretsManager::redact_value("api_key", "abcdef123456"), "ab...56");
    assert_eq!(SecretsManager::redact_value("password", "mysecretpass"), "my...ss");
    assert_eq!(SecretsManager::redact_value("token", "tok_abc123"), "to...23");
    assert_eq!(SecretsManager::redact_value("secret", "xxx"), "***");
    assert_eq!(SecretsManager::redact_value("port", "3000"), "3000");
    assert_eq!(
        SecretsManager::redact_value("database_url", "postgres://localhost"),
        "postgres://localhost"
    );
}

#[test]
fn test_placeholder_secret_detection() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    env::set_var("API_SECRET", "changeme");
    env::set_var("AUTH_TOKEN", "xxx");
    env::set_var("DB_PASSWORD", "example123");

    // Should detect placeholder values but not error
    let result = SecretsManager::validate_no_secrets_in_code();
    assert!(result.is_ok());

    cleanup_env_vars();
}

#[tokio::test]
async fn test_rotatable_secret() {
    let secret = RotatableSecret::new("initial_value".to_string());

    // Check initial value
    assert_eq!(secret.current().await, "initial_value");
    assert!(secret.validate("initial_value").await);
    assert!(!secret.validate("wrong_value").await);

    // Rotate the secret
    secret.rotate("new_value".to_string()).await;

    // Check new value
    assert_eq!(secret.current().await, "new_value");
    assert!(secret.validate("new_value").await);

    // Old value should still validate during rotation period
    assert!(secret.validate("initial_value").await);
}

#[tokio::test]
async fn test_secret_store() {
    let store = SecretStore::new();

    // Add secrets
    store
        .add_secret("api_key".to_string(), "key123".to_string())
        .await;
    store
        .add_secret("db_password".to_string(), "pass456".to_string())
        .await;

    // Get secrets
    assert_eq!(store.get_secret("api_key").await, Some("key123".to_string()));
    assert_eq!(store.get_secret("db_password").await, Some("pass456".to_string()));
    assert_eq!(store.get_secret("nonexistent").await, None);

    // Validate secrets
    assert!(store.validate_secret("api_key", "key123").await);
    assert!(!store.validate_secret("api_key", "wrong_key").await);

    // Rotate a secret
    store
        .rotate_secret("api_key", "new_key789".to_string())
        .await
        .unwrap();
    assert_eq!(store.get_secret("api_key").await, Some("new_key789".to_string()));

    // Both old and new values should validate
    assert!(store.validate_secret("api_key", "new_key789").await);
    assert!(store.validate_secret("api_key", "key123").await);

    // Test rotation of non-existent secret
    assert!(store
        .rotate_secret("nonexistent", "value".to_string())
        .await
        .is_err());
}

#[test]
fn test_config_with_cors() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    env::set_var("CORS_ALLOWED_ORIGINS", "http://localhost:3000,https://app.example.com");

    let config = Config::load().unwrap();
    assert_eq!(config.cors.allowed_origins.len(), 2);
    assert_eq!(config.cors.allowed_origins[0], "http://localhost:3000");
    assert_eq!(config.cors.allowed_origins[1], "https://app.example.com");

    cleanup_env_vars();
}

#[test]
fn test_jwks_url_validation() {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env_vars();

    env::set_var("AUTH_ENABLED", "true");
    env::set_var("AUTH_JWKS_URLS", "not-a-url");

    let result = Config::load();
    assert!(result.is_err());

    env::set_var("AUTH_JWKS_URLS", "https://valid.url/jwks");
    let result = Config::load();
    assert!(result.is_ok());

    cleanup_env_vars();
}

#[test]
fn test_config_conversions() {
    let config = Config::default();

    // Test auth config conversion
    let auth_config: estuary::auth::AuthConfig = (&config.auth).into();
    assert_eq!(auth_config.enabled, config.auth.enabled);
    assert_eq!(auth_config.jwks_urls, config.auth.jwks_urls);

    // Test rate limit config conversion
    let rate_limit_config: estuary::rate_limit::RateLimitConfig = (&config.rate_limit).into();
    assert_eq!(rate_limit_config.enabled, config.rate_limit.enabled);
    assert_eq!(rate_limit_config.max_requests, config.rate_limit.max_requests as u32);

    // Test security config conversion
    let security_config: estuary::security::SecurityConfig = (&config.security).into();
    assert_eq!(security_config.strict_mode, config.security.strict_mode);
    assert_eq!(security_config.csp, config.security.csp);
}
