//! Classification of the CLI input into a local file path or a remote URL.

use std::path::PathBuf;

use url::Url;

/// Where an OpenAPI specification is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenApiSource {
    /// A specification stored on the local file system.
    Path(PathBuf),
    /// A specification served over HTTP or HTTPS.
    Url(Box<Url>),
}

/// Errors raised while classifying an OpenAPI input string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceClassificationError {
    /// The input was empty or only contained whitespace.
    EmptyInput,
    /// The input looked like a URL but could not be parsed.
    InvalidUrl {
        /// The original input.
        input: String,
        /// The parser failure description.
        reason: String,
    },
}

impl std::fmt::Display for SourceClassificationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(formatter, "the OpenAPI path is empty"),
            Self::InvalidUrl { input, reason } => {
                write!(formatter, "could not parse '{input}' as a URL: {reason}")
            }
        }
    }
}

impl std::error::Error for SourceClassificationError {}

/// Returns `true` when the path points at an HTTP or HTTPS URL.
///
/// The comparison is case insensitive, matching the legacy .NET `OpenApiDocumentFactory.IsHttp`.
///
/// # Examples
///
/// ```
/// use curlgenerator::openapi::is_http;
///
/// assert!(is_http("HTTPS://example.com/openapi.json"));
/// assert!(!is_http("./openapi.json"));
/// ```
pub fn is_http(path: &str) -> bool {
    let lowercase = path.to_ascii_lowercase();
    lowercase.starts_with("http://") || lowercase.starts_with("https://")
}

/// Classifies an OpenAPI input string as either a local path or a remote URL.
///
/// # Examples
///
/// ```
/// use curlgenerator::openapi::{OpenApiSource, classify_source};
/// use std::path::PathBuf;
///
/// assert_eq!(
///     classify_source("./openapi.json").unwrap(),
///     OpenApiSource::Path(PathBuf::from("./openapi.json"))
/// );
/// ```
pub fn classify_source(input: &str) -> Result<OpenApiSource, SourceClassificationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SourceClassificationError::EmptyInput);
    }

    if is_http(trimmed) {
        return Url::parse(trimmed)
            .map(|url| OpenApiSource::Url(Box::new(url)))
            .map_err(|error| SourceClassificationError::InvalidUrl {
                input: trimmed.to_string(),
                reason: error.to_string(),
            });
    }

    Ok(OpenApiSource::Path(PathBuf::from(trimmed)))
}

/// Returns the scheme and authority of a URL, matching .NET's `UriPartial.Authority`.
///
/// # Examples
///
/// ```
/// use curlgenerator::openapi::authority_of;
///
/// assert_eq!(
///     authority_of("https://petstore.swagger.io/v2/swagger.json").as_deref(),
///     Some("https://petstore.swagger.io")
/// );
/// ```
pub fn authority_of(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    let scheme = parsed.scheme();
    let authority = match parsed.port() {
        Some(port) => format!("{scheme}://{host}:{port}"),
        None => format!("{scheme}://{host}"),
    };

    Some(authority)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_http_detects_both_schemes_case_insensitively() {
        assert!(is_http("http://example.com"));
        assert!(is_http("https://example.com"));
        assert!(is_http("HtTpS://example.com"));
        assert!(!is_http("ftp://example.com"));
        assert!(!is_http("./openapi.json"));
        assert!(!is_http("C:/specs/openapi.json"));
    }

    #[test]
    fn classify_source_returns_a_path_for_local_inputs() {
        assert_eq!(
            classify_source("./openapi.json").unwrap(),
            OpenApiSource::Path(PathBuf::from("./openapi.json"))
        );
    }

    #[test]
    fn classify_source_returns_a_url_for_remote_inputs() {
        let source = classify_source("https://example.com/openapi.json").unwrap();

        assert!(matches!(source, OpenApiSource::Url(_)));
    }

    #[test]
    fn classify_source_rejects_empty_input() {
        assert_eq!(
            classify_source("   "),
            Err(SourceClassificationError::EmptyInput)
        );
    }

    #[test]
    fn authority_of_keeps_non_default_ports() {
        assert_eq!(
            authority_of("http://localhost:8080/spec.json").as_deref(),
            Some("http://localhost:8080")
        );
        assert_eq!(authority_of("./openapi.json"), None);
    }

    #[test]
    fn classify_source_rejects_malformed_urls() {
        for input in [
            "http://",
            "https://exa mple.com/openapi.json",
            "http://example.com:99999/openapi.json",
        ] {
            assert!(
                matches!(
                    classify_source(input),
                    Err(SourceClassificationError::InvalidUrl { .. })
                ),
                "{input} should be rejected"
            );
        }

        assert!(matches!(
            classify_source("http://[::1]:8080/openapi.json"),
            Ok(OpenApiSource::Url(_))
        ));
    }

    #[test]
    fn authority_of_normalizes_the_scheme_host_and_port() {
        for (url, authority) in [
            (
                "HTTPS://Example.COM:443/openapi.json",
                "https://example.com",
            ),
            ("http://example.com:80/openapi.json", "http://example.com"),
            ("http://example.com:/openapi.json", "http://example.com"),
            (
                "http://user:secret@example.com:8081/openapi.json",
                "http://example.com:8081",
            ),
            ("http://example.com?query", "http://example.com"),
            (
                "https://example.com\\specs\\openapi.json",
                "https://example.com",
            ),
            ("http://[::1]:8080/openapi.json", "http://[::1]:8080"),
        ] {
            assert_eq!(authority_of(url).as_deref(), Some(authority), "{url}");
        }
    }
}
