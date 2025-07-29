use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// Simple rate limiter configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 1000, // Permissive default
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("RATE_LIMIT_ENABLED")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);

        let requests_per_minute = std::env::var("RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        Self {
            requests_per_minute,
            enabled,
        }
    }
}

/// Simple in-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    async fn check_rate_limit(&self, ip: IpAddr) -> Result<(u32, u32, Instant), StatusCode> {
        if !self.config.enabled {
            return Ok((
                self.config.requests_per_minute,
                self.config.requests_per_minute,
                Instant::now() + Duration::from_secs(60),
            ));
        }

        let mut windows = self.windows.lock().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);

        let (count, reset_at) = windows.entry(ip).or_insert((0, now + window_duration));

        // Reset window if expired
        if now >= *reset_at {
            *count = 0;
            *reset_at = now + window_duration;
        }

        if *count >= self.config.requests_per_minute {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        *count += 1;
        let remaining = self.config.requests_per_minute - *count;
        Ok((self.config.requests_per_minute, remaining, *reset_at))
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
    rate_limiter: RateLimiter,
) -> Response {
    // Extract IP from X-Forwarded-For or X-Real-IP headers
    let ip = extract_client_ip(&req);

    match rate_limiter.check_rate_limit(ip).await {
        Ok((limit, remaining, reset_at)) => {
            let mut response = next.run(req).await;
            let headers = response.headers_mut();

            // Add rate limit headers
            headers.insert("X-RateLimit-Limit", HeaderValue::from_str(&limit.to_string()).unwrap());
            headers.insert(
                "X-RateLimit-Remaining",
                HeaderValue::from_str(&remaining.to_string()).unwrap(),
            );

            let reset_seconds = reset_at.duration_since(Instant::now()).as_secs();
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from_str(&reset_seconds.to_string()).unwrap(),
            );

            response
        }
        Err(StatusCode::TOO_MANY_REQUESTS) => {
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": {
                        "code": "RATE_LIMIT_EXCEEDED",
                        "message": "Too many requests. Please try again later.",
                    }
                })),
            )
                .into_response();

            response
                .headers_mut()
                .insert("Retry-After", HeaderValue::from_static("60"));

            response
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Extract client IP from request headers
fn extract_client_ip(req: &Request) -> IpAddr {
    // Try X-Forwarded-For header first
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Default to localhost
    "127.0.0.1".parse().unwrap()
}
