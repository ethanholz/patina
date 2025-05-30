use axum::http::{HeaderMap, StatusCode};

/// Extract a required string header value
pub fn extract_header_string(headers: &HeaderMap, name: &str) -> Result<String, StatusCode> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(StatusCode::BAD_REQUEST)
}

/// Extract an optional string header value
pub fn extract_header_string_optional(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(|s| s.to_string())
}

/// Extract an optional numeric header value
pub fn extract_header_numeric<T: std::str::FromStr>(headers: &HeaderMap, name: &str) -> Option<T> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok()?.parse().ok())
}