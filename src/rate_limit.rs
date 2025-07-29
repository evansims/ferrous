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

/// Rate limit configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window_duration: Duration,
    /// Whether rate limiting is enabled
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 1000, // Very permissive default for development
            window_duration: Duration::from_secs(60), // 1 minute window
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("RATE_LIMIT_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let max_requests = std::env::var("RATE_LIMIT_MAX_REQUESTS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u32>()
            .unwrap_or(1000);

        let window_seconds = std::env::var("RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .unwrap_or(60);

        Self {
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
            enabled,
        }
    }
}

impl From<&crate::config::RateLimitConfig> for RateLimitConfig {
    fn from(config: &crate::config::RateLimitConfig) -> Self {
        Self {
            max_requests: config.max_requests as u32,
            window_duration: Duration::from_secs(config.window_seconds),
            enabled: config.enabled,
        }
    }
}

/// Simple in-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<IpAddr, Window>>>,
    config: RateLimitConfig,
}

#[derive(Debug)]
struct Window {
    count: u32,
    reset_at: Instant,
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
            return Ok((self.config.max_requests, 0, Instant::now() + self.config.window_duration));
        }

        let mut windows = self.windows.lock().await;
        let now = Instant::now();

        let window = windows.entry(ip).or_insert(Window {
            count: 0,
            reset_at: now + self.config.window_duration,
        });

        // Reset window if expired
        if now >= window.reset_at {
            window.count = 0;
            window.reset_at = now + self.config.window_duration;
        }

        if window.count >= self.config.max_requests {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        window.count += 1;
        let remaining = self.config.max_requests - window.count;

        Ok((self.config.max_requests, remaining, window.reset_at))
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
    rate_limiter: RateLimiter,
) -> Response {
    // Extract IP from X-Forwarded-For or socket address
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

            let reset_timestamp = reset_at.duration_since(Instant::now()).as_secs();
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from_str(&reset_timestamp.to_string()).unwrap(),
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

            let headers = response.headers_mut();
            headers.insert("Retry-After", HeaderValue::from_str("60").unwrap());

            response
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": "An error occurred while processing your request.",
                }
            })),
        )
            .into_response(),
    }
}

/// Extract client IP from request
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

    // Default to localhost if we can't determine the IP
    "127.0.0.1".parse().unwrap()
}
