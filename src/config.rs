use serde::{Deserialize, Serialize};
use std::{env, fmt, time::Duration};
use tracing::info;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    #[serde(default)]
    pub security: SecurityConfig,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub cors: CorsConfig,

    #[serde(default)]
    pub metrics: MetricsConfig,

    #[serde(default)]
    pub shutdown: ShutdownConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type", default = "default_database_type")]
    pub db_type: String,

    pub convex_deployment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_rust_log")]
    pub rust_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateLimitConfig {
    pub enabled: bool,

    #[validate(range(min = 1))]
    pub max_requests: u64,

    #[validate(range(min = 1))]
    pub window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub strict_mode: bool,

    #[serde(default = "default_csp")]
    pub csp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthConfig {
    pub enabled: bool,

    pub jwks_urls: Vec<String>,

    pub audience: Option<String>,

    pub issuer: Option<String>,

    #[validate(range(min = 60, max = 86400))]
    pub jwks_cache_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ShutdownConfig {
    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: u64,
}

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error: {}", self.message)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let profile = env::var("APP_PROFILE").unwrap_or_else(|_| "development".to_string());
        info!("Loading configuration for profile: {}", profile);

        let mut config = Config::default();

        // Load from environment variables
        config.load_from_env()?;

        // Validate the configuration
        config.validate().map_err(|e| ConfigError {
            message: format!("Validation failed: {}", e),
        })?;

        // Additional manual validations
        if config.server.port == 0 {
            return Err(ConfigError {
                message: format!("Invalid port: {}. Must be between 1-65535", config.server.port),
            });
        }

        if config.rate_limit.max_requests == 0 {
            return Err(ConfigError {
                message: "Rate limit max_requests must be greater than 0".to_string(),
            });
        }

        if config.auth.jwks_cache_seconds < 60 || config.auth.jwks_cache_seconds > 86400 {
            return Err(ConfigError {
                message: format!(
                    "Invalid JWKS cache seconds: {}. Must be between 60-86400",
                    config.auth.jwks_cache_seconds
                ),
            });
        }

        // Additional custom validations
        if config.auth.enabled {
            for url in &config.auth.jwks_urls {
                if !url.starts_with("https://") && !url.starts_with("http://") {
                    return Err(ConfigError {
                        message: format!(
                            "Invalid JWKS URL: {}. URLs must start with http:// or https://",
                            url
                        ),
                    });
                }
            }
        }

        // Log configuration (with secrets redacted)
        info!("Configuration loaded successfully");
        config.log_configuration();

        Ok(config)
    }

    fn load_from_env(&mut self) -> Result<(), ConfigError> {
        // Server configuration
        if let Ok(port) = env::var("PORT") {
            self.server.port = port.parse().map_err(|_| ConfigError {
                message: "PORT must be a number between 1-65535".to_string(),
            })?;
        }

        // Database configuration
        if let Ok(db_type) = env::var("DATABASE_TYPE") {
            self.database.db_type = db_type;
        }
        self.database.convex_deployment_url = env::var("CONVEX_DEPLOYMENT_URL").ok();

        // Logging configuration
        if let Ok(rust_log) = env::var("RUST_LOG") {
            self.logging.rust_log = rust_log;
        }

        // Rate limit configuration
        if let Ok(enabled) = env::var("RATE_LIMIT_ENABLED") {
            self.rate_limit.enabled = enabled.parse().unwrap_or(true);
        }
        if let Ok(max_requests) = env::var("RATE_LIMIT_MAX_REQUESTS") {
            self.rate_limit.max_requests = max_requests.parse().map_err(|_| ConfigError {
                message: "RATE_LIMIT_MAX_REQUESTS must be a positive number".to_string(),
            })?;
        }
        if let Ok(window_seconds) = env::var("RATE_LIMIT_WINDOW_SECONDS") {
            self.rate_limit.window_seconds = window_seconds.parse().map_err(|_| ConfigError {
                message: "RATE_LIMIT_WINDOW_SECONDS must be a positive number".to_string(),
            })?;
        }

        // Security configuration
        if let Ok(strict_mode) = env::var("SECURITY_STRICT_MODE") {
            self.security.strict_mode = strict_mode.parse().unwrap_or(false);
        }
        if let Ok(csp) = env::var("SECURITY_CSP") {
            self.security.csp = csp;
        }

        // Auth configuration
        if let Ok(enabled) = env::var("AUTH_ENABLED") {
            self.auth.enabled = enabled.parse().unwrap_or(false);
        }
        if let Ok(jwks_urls) = env::var("AUTH_JWKS_URLS") {
            self.auth.jwks_urls = jwks_urls
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        self.auth.audience = env::var("AUTH_AUDIENCE").ok();
        self.auth.issuer = env::var("AUTH_ISSUER").ok();
        if let Ok(cache_seconds) = env::var("AUTH_JWKS_CACHE_SECONDS") {
            self.auth.jwks_cache_seconds = cache_seconds.parse().map_err(|_| ConfigError {
                message: "AUTH_JWKS_CACHE_SECONDS must be a number between 60-86400".to_string(),
            })?;
        }

        // CORS configuration
        if let Ok(origins) = env::var("CORS_ALLOWED_ORIGINS") {
            self.cors.allowed_origins = origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Shutdown configuration
        if let Ok(timeout) = env::var("SHUTDOWN_TIMEOUT_SECONDS") {
            let timeout_secs: u64 = timeout.parse().map_err(|_| ConfigError {
                message: "SHUTDOWN_TIMEOUT_SECONDS must be a number between 1-300".to_string(),
            })?;
            if !(1..=300).contains(&timeout_secs) {
                return Err(ConfigError {
                    message: format!(
                        "SHUTDOWN_TIMEOUT_SECONDS must be between 1-300, got {}",
                        timeout_secs
                    ),
                });
            }
            self.shutdown.timeout_seconds = timeout_secs;
        }

        Ok(())
    }

    pub fn validate_runtime_dependencies(&self) -> Result<(), ConfigError> {
        // Validate database-specific requirements
        match self.database.db_type.as_str() {
            "convex" => {
                if self.database.convex_deployment_url.is_none() {
                    return Err(ConfigError {
                        message: "CONVEX_DEPLOYMENT_URL is required when DATABASE_TYPE=convex"
                            .to_string(),
                    });
                }
            }
            "memory" => {}
            _ => {
                return Err(ConfigError {
                    message: format!("Unknown database type: {}", self.database.db_type),
                });
            }
        }

        // Validate auth configuration
        if self.auth.enabled && self.auth.jwks_urls.is_empty() {
            return Err(ConfigError {
                message: "AUTH_JWKS_URLS is required when AUTH_ENABLED=true".to_string(),
            });
        }

        Ok(())
    }

    fn log_configuration(&self) {
        info!("Server: {}:{}", self.server.host, self.server.port);
        info!("Database: {}", self.database.db_type);
        info!(
            "Rate limiting: {}",
            if self.rate_limit.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        info!(
            "Authentication: {}",
            if self.auth.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        info!(
            "Security mode: {}",
            if self.security.strict_mode {
                "strict"
            } else {
                "permissive"
            }
        );

        if self.auth.enabled {
            info!("JWKS URLs configured: {}", self.auth.jwks_urls.len());
        }

        if !self.cors.allowed_origins.is_empty() {
            info!("CORS origins configured: {}", self.cors.allowed_origins.len());
        }
    }

    pub fn shutdown_duration(&self) -> Duration {
        Duration::from_secs(self.shutdown.timeout_seconds)
    }
}

impl Default for Config {
    fn default() -> Self {
        let profile = env::var("APP_PROFILE").unwrap_or_else(|_| "development".to_string());
        let is_production = profile == "production";

        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            rate_limit: RateLimitConfig::default_for_profile(is_production),
            security: SecurityConfig::default_for_profile(is_production),
            auth: AuthConfig::default(),
            cors: CorsConfig::default(),
            metrics: MetricsConfig::default(),
            shutdown: ShutdownConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: default_host(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: default_database_type(),
            convex_deployment_url: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            rust_log: default_rust_log(),
        }
    }
}

impl RateLimitConfig {
    fn default_for_profile(is_production: bool) -> Self {
        if is_production {
            Self {
                enabled: true,
                max_requests: 100,
                window_seconds: 60,
            }
        } else {
            Self {
                enabled: true,
                max_requests: 1000,
                window_seconds: 60,
            }
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::default_for_profile(false)
    }
}

impl SecurityConfig {
    fn default_for_profile(is_production: bool) -> Self {
        Self {
            strict_mode: is_production,
            csp: default_csp(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self::default_for_profile(false)
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jwks_urls: Vec::new(),
            audience: None,
            issuer: None,
            jwks_cache_seconds: 3600,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_database_type() -> String {
    "memory".to_string()
}

fn default_rust_log() -> String {
    "estuary=debug,tower_http=debug".to_string()
}

fn default_csp() -> String {
    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
        .to_string()
}

pub mod secrets {
    use super::ConfigError;
    use std::{collections::HashMap, env, sync::Arc};
    use tokio::sync::RwLock;
    use tracing::{info, warn};

    pub struct SecretsManager;

    impl SecretsManager {
        pub fn validate_no_secrets_in_code() -> Result<(), ConfigError> {
            let sensitive_patterns = [
                "password",
                "secret",
                "api_key",
                "private_key",
                "token",
                "credential",
            ];

            for (key, value) in env::vars() {
                let key_lower = key.to_lowercase();

                // Check if this looks like a sensitive variable
                let is_sensitive = sensitive_patterns
                    .iter()
                    .any(|pattern| key_lower.contains(pattern));

                if is_sensitive && !value.is_empty() {
                    // Log that we found a secret (but not its value)
                    info!("Found configured secret: {} (length: {})", key, value.len());

                    // Warn if the value looks like a placeholder
                    if value.contains("example")
                        || value.contains("changeme")
                        || value.contains("xxx")
                    {
                        warn!(
                            "Secret {} appears to contain a placeholder value. Please update before production use.",
                            key
                        );
                    }
                }
            }

            Ok(())
        }

        pub fn redact_value(key: &str, value: &str) -> String {
            let key_lower = key.to_lowercase();

            if key_lower.contains("password")
                || key_lower.contains("secret")
                || key_lower.contains("token")
                || key_lower.contains("key")
            {
                if value.len() > 4 {
                    format!("{}...{}", &value[..2], &value[value.len() - 2..])
                } else {
                    "***".to_string()
                }
            } else {
                value.to_string()
            }
        }
    }

    /// Support for secret rotation
    #[derive(Clone)]
    pub struct RotatableSecret {
        current: Arc<RwLock<String>>,
        previous: Arc<RwLock<Option<String>>>,
    }

    impl RotatableSecret {
        pub fn new(value: String) -> Self {
            Self {
                current: Arc::new(RwLock::new(value)),
                previous: Arc::new(RwLock::new(None)),
            }
        }

        pub async fn rotate(&self, new_value: String) {
            let mut current = self.current.write().await;
            let mut previous = self.previous.write().await;

            // Move current to previous
            *previous = Some(current.clone());
            // Set new current
            *current = new_value;

            info!("Secret rotated successfully");
        }

        pub async fn current(&self) -> String {
            self.current.read().await.clone()
        }

        pub async fn validate(&self, value: &str) -> bool {
            let current = self.current.read().await;
            if *current == value {
                return true;
            }

            // Check previous value during rotation period
            if let Some(ref prev) = *self.previous.read().await {
                return prev == value;
            }

            false
        }
    }

    /// Manages multiple rotatable secrets
    pub struct SecretStore {
        secrets: Arc<RwLock<HashMap<String, RotatableSecret>>>,
    }

    impl SecretStore {
        pub fn new() -> Self {
            Self {
                secrets: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub async fn add_secret(&self, name: String, value: String) {
            let mut secrets = self.secrets.write().await;
            secrets.insert(name.clone(), RotatableSecret::new(value));
            info!("Added secret: {}", name);
        }

        pub async fn rotate_secret(
            &self,
            name: &str,
            new_value: String,
        ) -> Result<(), ConfigError> {
            let secrets = self.secrets.read().await;
            if let Some(secret) = secrets.get(name) {
                secret.rotate(new_value).await;
                Ok(())
            } else {
                Err(ConfigError {
                    message: format!("Secret '{}' not found", name),
                })
            }
        }

        pub async fn get_secret(&self, name: &str) -> Option<String> {
            let secrets = self.secrets.read().await;
            if let Some(secret) = secrets.get(name) {
                Some(secret.current().await)
            } else {
                None
            }
        }

        pub async fn validate_secret(&self, name: &str, value: &str) -> bool {
            let secrets = self.secrets.read().await;
            if let Some(secret) = secrets.get(name) {
                secret.validate(value).await
            } else {
                false
            }
        }
    }

    impl Default for SecretStore {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.db_type, "memory");
        assert!(!config.auth.enabled);
    }

    #[test]
    fn test_production_defaults() {
        env::set_var("APP_PROFILE", "production");
        let config = Config::default();
        assert!(config.security.strict_mode);
        assert_eq!(config.rate_limit.max_requests, 100);
        env::remove_var("APP_PROFILE");
    }

    #[test]
    fn test_config_validation() {
        // Test manual validation in Config::load()
        env::set_var("PORT", "0");
        let result = Config::load();
        assert!(result.is_err());
        env::remove_var("PORT");

        env::set_var("RATE_LIMIT_MAX_REQUESTS", "0");
        let result = Config::load();
        assert!(result.is_err());
        env::remove_var("RATE_LIMIT_MAX_REQUESTS");
    }

    #[test]
    fn test_runtime_validation() {
        let mut config = Config::default();
        config.database.db_type = "convex".to_string();
        assert!(config.validate_runtime_dependencies().is_err());

        config.database.convex_deployment_url = Some("https://example.convex.cloud".to_string());
        assert!(config.validate_runtime_dependencies().is_ok());
    }

    #[test]
    fn test_secrets_redaction() {
        use secrets::SecretsManager;

        assert_eq!(SecretsManager::redact_value("api_key", "abc123def456"), "ab...56");
        assert_eq!(SecretsManager::redact_value("port", "3000"), "3000");
        assert_eq!(SecretsManager::redact_value("password", "sho"), "***");
    }
}
