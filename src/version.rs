use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use chrono::{DateTime, Utc};
use std::str::FromStr;

/// Current API version
pub const API_VERSION_CURRENT: &str = "v1";

/// Default API version when none specified
pub const API_VERSION_DEFAULT: &str = API_VERSION_CURRENT;

/// All supported API versions
pub const API_VERSION_SUPPORTED: &[&str] = &["v1"];

/// Deprecated API versions (empty for now)
pub const API_VERSION_DEPRECATED: &[&str] = &[];

/// Sunset dates for deprecated versions
pub const API_VERSION_SUNSET_DATES: &[(&str, &str)] = &[];

/// API Version enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ApiVersion {
    #[default]
    V1,
}

impl ApiVersion {
    /// Check if this version is deprecated
    pub fn is_deprecated(&self) -> bool {
        let version_str = self.as_str();
        API_VERSION_DEPRECATED.contains(&version_str)
    }

    /// Get sunset date for deprecated version
    pub fn sunset_date(&self) -> Option<DateTime<Utc>> {
        let version_str = self.as_str();
        API_VERSION_SUNSET_DATES
            .iter()
            .find(|(v, _)| v == &version_str)
            .and_then(|(_, date)| DateTime::parse_from_rfc3339(date).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "v1",
        }
    }
}

impl FromStr for ApiVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" | "1" => Ok(ApiVersion::V1),
            _ => Err(VersionError::Unsupported(s.to_string())),
        }
    }
}

/// Version parsing errors
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Unsupported API version: {0}")]
    Unsupported(String),

    #[error("Invalid version format: {0}")]
    InvalidFormat(String),
}

/// Extract API version from various sources
pub fn extract_version(path: &str, headers: &HeaderMap) -> ApiVersion {
    // 1. Check URL path for version
    if let Some(version) = extract_version_from_path(path) {
        if let Ok(v) = ApiVersion::from_str(&version) {
            return v;
        }
    }

    // 2. Check Accept header
    if let Some(accept) = headers.get("accept") {
        if let Ok(accept_str) = accept.to_str() {
            if let Some(version) = extract_version_from_accept(accept_str) {
                if let Ok(v) = ApiVersion::from_str(&version) {
                    return v;
                }
            }
        }
    }

    // 3. Check X-API-Version header
    if let Some(version_header) = headers.get("x-api-version") {
        if let Ok(version_str) = version_header.to_str() {
            if let Ok(v) = ApiVersion::from_str(version_str) {
                return v;
            }
        }
    }

    // 4. Default to current version
    ApiVersion::default()
}

/// Extract version from URL path
fn extract_version_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();

    // Look for /api/v1/... pattern
    if parts.len() >= 3 && parts[1] == "api" && parts[2].starts_with('v') {
        return Some(parts[2].to_string());
    }

    None
}

/// Extract version from Accept header
fn extract_version_from_accept(accept: &str) -> Option<String> {
    // Look for application/vnd.estuary.v1+json pattern
    if accept.contains("application/vnd.estuary.") {
        let parts: Vec<&str> = accept.split('.').collect();
        if parts.len() >= 3 {
            let version_part = parts[2];
            if let Some(version) = version_part.strip_suffix("+json") {
                return Some(version.to_string());
            }
        }
    }

    None
}

/// Version context extension for requests
#[derive(Debug, Clone)]
pub struct VersionContext {
    pub version: ApiVersion,
}

/// Middleware to extract and validate API version
pub async fn version_middleware(mut req: Request, next: Next) -> Result<Response, Response> {
    let path = req.uri().path();
    let headers = req.headers();

    let version = extract_version(path, headers);

    // Add version to request extensions
    req.extensions_mut().insert(VersionContext { version });

    let mut response = next.run(req).await;

    // Add deprecation headers if version is deprecated
    if version.is_deprecated() {
        let headers = response.headers_mut();
        headers.insert("Deprecation", "true".parse().unwrap());

        if let Some(sunset) = version.sunset_date() {
            headers.insert("Sunset", sunset.to_rfc2822().parse().unwrap());
            headers.insert(
                "Link",
                format!(
                    "<https://docs.estuary.com/migration/{}#to-{}>; rel=\"successor-version\"",
                    version.as_str(),
                    API_VERSION_CURRENT
                )
                .parse()
                .unwrap(),
            );
            headers.insert(
                "Warning",
                format!(
                    "299 - \"This API version is deprecated and will be removed on {}\"",
                    sunset.format("%Y-%m-%d")
                )
                .parse()
                .unwrap(),
            );
        }
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_version_from_str() {
        assert_eq!(ApiVersion::from_str("v1").unwrap(), ApiVersion::V1);
        assert_eq!(ApiVersion::from_str("1").unwrap(), ApiVersion::V1);
        assert!(ApiVersion::from_str("v2").is_err());
        assert!(ApiVersion::from_str("invalid").is_err());
    }

    #[test]
    fn test_extract_version_from_path() {
        assert_eq!(
            extract_version_from_path("/api/v1/items"),
            Some("v1".to_string())
        );
        assert_eq!(
            extract_version_from_path("/api/v2/items"),
            Some("v2".to_string())
        );
        assert_eq!(extract_version_from_path("/api/items"), None);
        assert_eq!(extract_version_from_path("/health"), None);
    }

    #[test]
    fn test_extract_version_from_accept() {
        assert_eq!(
            extract_version_from_accept("application/vnd.estuary.v1+json"),
            Some("v1".to_string())
        );
        assert_eq!(
            extract_version_from_accept("application/vnd.estuary.v2+json"),
            Some("v2".to_string())
        );
        assert_eq!(extract_version_from_accept("application/json"), None);
    }

    #[test]
    fn test_extract_version_precedence() {
        let mut headers = HeaderMap::new();

        // Test path takes precedence
        let version = extract_version("/api/v1/items", &headers);
        assert_eq!(version, ApiVersion::V1);

        // Test Accept header when no path version
        headers.insert(
            "accept",
            HeaderValue::from_static("application/vnd.estuary.v1+json"),
        );
        let version = extract_version("/items", &headers);
        assert_eq!(version, ApiVersion::V1);

        // Test X-API-Version when no path or accept version
        headers.clear();
        headers.insert("x-api-version", HeaderValue::from_static("v1"));
        let version = extract_version("/items", &headers);
        assert_eq!(version, ApiVersion::V1);

        // Test default when no version specified
        headers.clear();
        let version = extract_version("/items", &headers);
        assert_eq!(version, ApiVersion::V1);
    }
}
