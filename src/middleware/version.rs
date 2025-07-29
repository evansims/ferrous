use axum::{extract::Request, middleware::Next, response::Response};

/// Simple API versioning - just extract from URL path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApiVersion {
    #[default]
    V1,
}

/// Extract API version from request path
pub fn extract_version(path: &str) -> ApiVersion {
    if path.contains("/v1/") || path.contains("/api/v1") {
        ApiVersion::V1
    } else {
        ApiVersion::default()
    }
}

/// Version context for request extensions
#[derive(Debug, Clone)]
pub struct VersionContext {
    pub version: ApiVersion,
}

/// Simple version middleware - just extracts version from path
pub async fn version_middleware(mut req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let version = extract_version(path);

    // Add version to request extensions
    req.extensions_mut().insert(VersionContext { version });

    next.run(req).await
}
