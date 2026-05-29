//! Helpers for loading the raw text of an OpenAPI specification from a local
//! file or an HTTP(S) URL.

use std::time::Duration;

/// Returns `true` when the path is an HTTP or HTTPS URL.
pub fn is_http(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// Reads the contents of an OpenAPI specification located either on disk or at
/// an HTTP(S) URL.
pub fn read_spec(path: &str) -> Result<String, String> {
    if is_http(path) {
        fetch_http(path)
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("Could not open the file at {path}: {e}"))
    }
}

/// Downloads the contents of an HTTP(S) URL, decompressing gzip/deflate and
/// accepting any server certificate (mirrors the original tool's behaviour).
pub fn fetch_http(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .gzip(true)
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Could not create HTTP client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Could not download the file at {url}: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Could not download the file at {url}: HTTP {}",
            response.status()
        ));
    }

    response
        .text()
        .map_err(|e| format!("Could not read the response from {url}: {e}"))
}

/// Returns the authority part (`scheme://host[:port]`) of an absolute URL.
pub fn authority(url: &str) -> String {
    let scheme_end = match url.find("://") {
        Some(index) => index + 3,
        None => return String::new(),
    };
    let rest = &url[scheme_end..];
    let path_start = rest.find('/').unwrap_or(rest.len());
    format!("{}{}", &url[..scheme_end], &rest[..path_start])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_http_detects_urls() {
        assert!(is_http("http://example.com"));
        assert!(is_http("HTTPS://example.com"));
        assert!(!is_http("./local/file.json"));
        assert!(!is_http("C:/path/file.yaml"));
    }

    #[test]
    fn authority_extracts_scheme_and_host() {
        assert_eq!(
            authority("https://petstore.swagger.io/v2/swagger.json"),
            "https://petstore.swagger.io"
        );
        assert_eq!(authority("http://host:8080/a/b"), "http://host:8080");
        assert_eq!(authority("not-a-url"), "");
    }
}
