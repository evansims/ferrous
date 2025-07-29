use serde::{Deserialize, Serialize};
use std::env;
use validator::Validate;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub shutdown: ShutdownConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: String,
    pub convex_deployment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub rust_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    pub timeout_seconds: u64,
}

// Simple error type
#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::default();

        // Load from environment variables
        if let Ok(port) = env::var("PORT") {
            config.server.port = port.parse().map_err(|_| ConfigError {
                message: "PORT must be a number between 1-65535".to_string(),
            })?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            if db_url.starts_with("memory://") {
                config.database.db_type = "memory".to_string();
            } else if db_url.starts_with("convex://") {
                config.database.db_type = "convex".to_string();
                config.database.convex_deployment_url =
                    Some(db_url.replace("convex://", "https://"));
            }
        } else if let Ok(db_type) = env::var("DATABASE_TYPE") {
            config.database.db_type = db_type;
            if config.database.db_type == "convex" {
                config.database.convex_deployment_url = env::var("CONVEX_DEPLOYMENT_URL").ok();
            }
        }

        if let Ok(rust_log) = env::var("RUST_LOG") {
            config.logging.rust_log = rust_log;
        }

        if let Ok(timeout) = env::var("SHUTDOWN_TIMEOUT_SECONDS") {
            config.shutdown.timeout_seconds = timeout.parse().unwrap_or(30);
        }

        // Validate
        config.validate().map_err(|e| ConfigError {
            message: format!("Validation failed: {e}"),
        })?;

        // Additional manual validations
        if config.server.port == 0 {
            return Err(ConfigError {
                message: "Port must be between 1-65535".to_string(),
            });
        }

        Ok(config)
    }

    pub fn validate_runtime_dependencies(&self) -> Result<(), ConfigError> {
        if self.database.db_type == "convex" && self.database.convex_deployment_url.is_none() {
            return Err(ConfigError {
                message: "Convex database requires CONVEX_DEPLOYMENT_URL".to_string(),
            });
        }
        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3000 }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: "memory".to_string(),
            convex_deployment_url: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            rust_log: "ferrous=debug,tower_http=debug".to_string(),
        }
    }
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
        }
    }
}

// Removed secrets module - use external tools for secrets management

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to prevent tests from interfering with each other
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.db_type, "memory");
        assert_eq!(config.shutdown.timeout_seconds, 30);
    }

    #[test]
    fn test_load_from_env() {
        let _guard = TEST_MUTEX.lock().unwrap();

        // Clean up first in case previous test left vars
        env::remove_var("PORT");
        env::remove_var("DATABASE_URL");

        env::set_var("PORT", "8080");
        env::set_var("DATABASE_URL", "convex://my-deployment");

        let config = Config::load().unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.db_type, "convex");
        assert_eq!(
            config.database.convex_deployment_url,
            Some("https://my-deployment".to_string())
        );

        env::remove_var("PORT");
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_validation() {
        let _guard = TEST_MUTEX.lock().unwrap();

        env::set_var("PORT", "0");
        let result = Config::load();
        assert!(result.is_err());
        env::remove_var("PORT");
    }

    #[test]
    fn test_runtime_validation() {
        let mut config = Config::default();
        config.database.db_type = "convex".to_string();
        assert!(config.validate_runtime_dependencies().is_err());

        config.database.convex_deployment_url = Some("https://example.convex.cloud".to_string());
        assert!(config.validate_runtime_dependencies().is_ok());
    }
}
