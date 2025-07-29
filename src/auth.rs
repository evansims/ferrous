use axum::{
    extract::{FromRequestParts, Request},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub iss: Option<String>,
    pub aud: Option<String>,
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    pub enabled: bool,
    /// JWKS URLs to fetch public keys from
    pub jwks_urls: Vec<String>,
    /// Expected audience
    pub audience: Option<String>,
    /// Expected issuer
    pub issuer: Option<String>,
    /// Cache duration for JWKS
    pub jwks_cache_duration: Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for development
            jwks_urls: Vec::new(),
            audience: None,
            issuer: None,
            jwks_cache_duration: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("AUTH_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let jwks_urls = std::env::var("AUTH_JWKS_URLS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let audience = std::env::var("AUTH_AUDIENCE").ok();
        let issuer = std::env::var("AUTH_ISSUER").ok();

        let cache_duration_secs = std::env::var("AUTH_JWKS_CACHE_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);

        Self {
            enabled,
            jwks_urls,
            audience,
            issuer,
            jwks_cache_duration: Duration::from_secs(cache_duration_secs),
        }
    }
}

impl From<&crate::config::AuthConfig> for AuthConfig {
    fn from(config: &crate::config::AuthConfig) -> Self {
        Self {
            enabled: config.enabled,
            jwks_urls: config.jwks_urls.clone(),
            audience: config.audience.clone(),
            issuer: config.issuer.clone(),
            jwks_cache_duration: Duration::from_secs(config.jwks_cache_seconds),
        }
    }
}

/// JWKS cache entry
struct JwksCache {
    jwks: JwkSet,
    fetched_at: SystemTime,
}

/// JWT validator with JWKS support
#[derive(Clone)]
pub struct JwtValidator {
    config: AuthConfig,
    jwks_cache: Arc<RwLock<HashMap<String, JwksCache>>>,
    http_client: reqwest::Client,
}

impl JwtValidator {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            jwks_cache: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }

    /// Fetch JWKS from URL
    async fn fetch_jwks(&self, url: &str) -> Result<JwkSet, AuthError> {
        let response = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AuthError::JwksFetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::JwksFetchError(format!(
                "HTTP {} from JWKS endpoint",
                response.status()
            )));
        }

        response
            .json::<JwkSet>()
            .await
            .map_err(|e| AuthError::JwksFetchError(e.to_string()))
    }

    /// Get JWKS with caching
    async fn get_jwks(&self, url: &str) -> Result<JwkSet, AuthError> {
        // Check cache
        {
            let cache = self.jwks_cache.read().await;
            if let Some(cached) = cache.get(url) {
                if cached.fetched_at.elapsed().unwrap_or(Duration::MAX)
                    < self.config.jwks_cache_duration
                {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Fetch new JWKS
        let jwks = self.fetch_jwks(url).await?;

        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            cache.insert(
                url.to_string(),
                JwksCache {
                    jwks: jwks.clone(),
                    fetched_at: SystemTime::now(),
                },
            );
        }

        Ok(jwks)
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        // Decode header to get kid
        let header = decode_header(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("Missing kid in JWT header".to_string()))?;

        // Try each JWKS URL
        for jwks_url in &self.config.jwks_urls {
            let jwks = match self.get_jwks(jwks_url).await {
                Ok(jwks) => jwks,
                Err(_) => continue, // Try next URL
            };

            // Find key by kid
            if let Some(jwk) = jwks
                .keys
                .iter()
                .find(|k| k.common.key_id == Some(kid.clone()))
            {
                let key = DecodingKey::from_jwk(jwk)
                    .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

                // Set up validation
                let alg = header.alg;
                let mut validation = Validation::new(alg);

                if let Some(aud) = &self.config.audience {
                    validation.set_audience(&[aud]);
                }

                if let Some(iss) = &self.config.issuer {
                    validation.set_issuer(&[iss]);
                }

                // Decode and validate
                return decode::<Claims>(token, &key, &validation)
                    .map_err(|e| AuthError::InvalidToken(e.to_string()));
            }
        }

        Err(AuthError::InvalidToken("No matching key found in JWKS".to_string()))
    }
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization header format")]
    InvalidAuthHeader,

    #[error("Failed to fetch JWKS: {0}")]
    JwksFetchError(String),

    #[error("Authentication required")]
    AuthRequired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        use crate::error::{ErrorCode, ErrorDetails, ErrorResponse};
        use chrono::Utc;

        let (status, code, message, details) = match self {
            AuthError::InvalidToken(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Invalid authentication token".to_string(),
                Some(ErrorDetails {
                    validation_errors: None,
                    context: Some(msg),
                }),
            ),
            AuthError::MissingAuthHeader => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Missing authorization header".to_string(),
                None,
            ),
            AuthError::InvalidAuthHeader => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Invalid authorization header format".to_string(),
                None,
            ),
            AuthError::JwksFetchError(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::ServiceUnavailable,
                "Authentication service unavailable".to_string(),
                Some(ErrorDetails {
                    validation_errors: None,
                    context: Some(msg),
                }),
            ),
            AuthError::AuthRequired => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Authentication required".to_string(),
                None,
            ),
        };

        let error_response = ErrorResponse {
            error: code,
            message,
            details,
            timestamp: Utc::now(),
            request_id: None, // Will be injected by middleware
        };

        (status, Json(error_response)).into_response()
    }
}

/// Authenticated user extractor
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get claims from request extensions (set by middleware)
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::AuthRequired)?;

        Ok(AuthUser(claims))
    }
}

/// Optional authenticated user extractor
pub struct OptionalAuthUser(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<Claims>().cloned();
        Ok(OptionalAuthUser(claims))
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
    validator: JwtValidator,
) -> Result<Response, AuthError> {
    // Skip if auth is disabled
    if !validator.config.enabled {
        return Ok(next.run(req).await);
    }

    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Validate token
            match validator.validate_token(token).await {
                Ok(token_data) => {
                    // Add claims to request extensions
                    req.extensions_mut().insert(token_data.claims);
                }
                Err(_) => {
                    // Invalid token, but continue - let handlers decide if auth is required
                }
            }
        }
    }

    Ok(next.run(req).await)
}

/// Private key JWT client assertion validation (RFC 7523)
pub async fn validate_client_assertion(
    assertion: &str,
    client_id: &str,
    validator: &JwtValidator,
) -> Result<(), AuthError> {
    // Validate the JWT
    let token_data = validator.validate_token(assertion).await?;

    // Check that sub and iss match client_id
    if token_data.claims.sub != client_id {
        return Err(AuthError::InvalidToken("Client ID mismatch in sub claim".to_string()));
    }

    if let Some(iss) = &token_data.claims.iss {
        if iss != client_id {
            return Err(AuthError::InvalidToken("Client ID mismatch in iss claim".to_string()));
        }
    }

    Ok(())
}
