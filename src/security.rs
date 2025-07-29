use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Security headers configuration
#[derive(Clone)]
pub struct SecurityConfig {
    /// Whether to enable strict security headers
    pub strict_mode: bool,
    /// Content Security Policy
    pub csp: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            csp: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
        }
    }
}

impl SecurityConfig {
    pub fn from_env() -> Self {
        let strict_mode = std::env::var("SECURITY_STRICT_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let csp = if strict_mode {
            // Strict CSP for production
            "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
        } else {
            // Permissive CSP for development
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src * data:; font-src *; connect-src *"
        };

        Self {
            strict_mode,
            csp: std::env::var("SECURITY_CSP").unwrap_or_else(|_| csp.to_string()),
        }
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let config = SecurityConfig::from_env();
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Always set these security headers
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Content Security Policy
    if let Ok(csp_value) = HeaderValue::from_str(&config.csp) {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp_value);
    }

    // HSTS header (only in strict mode, assuming HTTPS)
    if config.strict_mode {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }

    response
}
