use axum::{
    extract::{FromRequestParts, Request},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Simple JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

/// Simple auth configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub enabled: bool,
    pub jwt_secret: Option<String>,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("AUTH_ENABLED")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);

        let jwt_secret = std::env::var("JWT_SECRET").ok();

        Self {
            enabled,
            jwt_secret,
        }
    }
}

/// Authenticated user extractor
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthUser)
            .ok_or(StatusCode::UNAUTHORIZED)
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
        Ok(OptionalAuthUser(parts.extensions.get::<Claims>().cloned()))
    }
}

/// Simple JWT authentication middleware
pub async fn auth_middleware(mut req: Request, next: Next, config: AuthConfig) -> Response {
    // Skip if auth is disabled
    if !config.enabled {
        return next.run(req).await;
    }

    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Simple JWT validation
                if let Some(secret) = &config.jwt_secret {
                    let key = DecodingKey::from_secret(secret.as_bytes());
                    let validation = Validation::default();

                    if let Ok(token_data) = decode::<Claims>(token, &key, &validation) {
                        // Add claims to request extensions
                        req.extensions_mut().insert(token_data.claims);
                    }
                }
            }
        }
    }

    next.run(req).await
}

/// Middleware to require authentication for specific routes
pub async fn require_auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Check if claims exist in extensions (set by auth middleware)
    if req.extensions().get::<Claims>().is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
