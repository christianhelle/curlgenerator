//! Classification of the CLI input into a local file path or a remote URL.

use std::path::PathBuf;

/// Schemes whose URLs always have a host, together with their default ports.
const SPECIAL_SCHEMES: [(&str, u16); 5] = [
    ("http", 80),
    ("https", 443),
    ("ws", 80),
    ("wss", 443),
    ("ftp", 21),
];

/// Where an OpenAPI specification is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenApiSource {
    /// A specification stored on the local file system.
    Path(PathBuf),
    /// A specification served over HTTP or HTTPS.
    Url(String),
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
        return parse_authority(trimmed)
            .map(|_| OpenApiSource::Url(trimmed.to_string()))
            .map_err(|reason| SourceClassificationError::InvalidUrl {
                input: trimmed.to_string(),
                reason,
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
    let authority = parse_authority(url).ok()??;
    if authority.host.is_empty() {
        return None;
    }

    Some(match authority.port {
        Some(port) => format!("{}://{}:{port}", authority.scheme, authority.host),
        None => format!("{}://{}", authority.scheme, authority.host),
    })
}

/// Returns `true` when `value` is an absolute URL, one that starts with a scheme.
///
/// A URL whose scheme always has a host, such as `http`, must also have a valid one.
///
/// # Examples
///
/// ```
/// use curlgenerator::openapi::is_absolute_url;
///
/// assert!(is_absolute_url("https://example.com/api"));
/// assert!(!is_absolute_url("/api"));
/// assert!(!is_absolute_url("https://"));
/// ```
pub fn is_absolute_url(value: &str) -> bool {
    parse_authority(value).is_ok()
}

/// The scheme, host and port of an absolute URL, normalized the way browsers serialize them.
struct Authority {
    scheme: String,
    host: String,
    /// The port, or `None` when it is missing or the default port of the scheme.
    port: Option<u16>,
}

/// Parses the authority of an absolute URL.
///
/// Returns `Ok(None)` for an absolute URL without an authority, such as `mailto:someone`, and the
/// reason the URL is invalid otherwise. Hosts are lowercased but not converted to Punycode.
fn parse_authority(url: &str) -> Result<Option<Authority>, String> {
    let (scheme, rest) = split_scheme(url).ok_or("relative URL without a base")?;
    let scheme = scheme.to_ascii_lowercase();
    let default_port = SPECIAL_SCHEMES
        .iter()
        .find(|(name, _)| *name == scheme)
        .map(|(_, port)| *port);
    let special = default_port.is_some();

    // Special schemes treat backslashes as slashes and do not need the slashes at all.
    let (rest, delimiters): (&str, &[char]) = if special {
        (rest.trim_start_matches(['/', '\\']), &['/', '\\', '?', '#'])
    } else {
        match rest.strip_prefix("//") {
            Some(rest) => (rest, &['/', '?', '#']),
            None => return Ok(None),
        }
    };
    let authority = rest.split(delimiters).next().unwrap_or_default();
    let (credentials, host_and_port) = match authority.rsplit_once('@') {
        Some((credentials, host_and_port)) => (Some(credentials), host_and_port),
        None => (None, authority),
    };

    let (host, port) = match host_and_port.strip_prefix('[') {
        Some(bracketed) => {
            let (address, after) = bracketed.split_once(']').ok_or("invalid IPv6 address")?;
            let valid = !address.is_empty()
                && address.chars().all(|character| {
                    character.is_ascii_hexdigit() || matches!(character, ':' | '.')
                });
            if !valid {
                return Err("invalid IPv6 address".to_string());
            }

            let port = match after {
                "" => None,
                _ => Some(after.strip_prefix(':').ok_or("invalid IPv6 address")?),
            };

            (format!("[{}]", address.to_ascii_lowercase()), port)
        }
        None => {
            let (host, port) = match host_and_port.split_once(':') {
                Some((host, port)) => (host, Some(port)),
                None => (host_and_port, None),
            };
            if host.chars().any(|character| {
                character.is_control()
                    || matches!(character, ' ' | '<' | '>' | '[' | ']' | '\\' | '^' | '|')
            }) {
                return Err("invalid domain character".to_string());
            }

            (host.to_lowercase(), port)
        }
    };

    if host.is_empty() && (special || port.is_some() || credentials.is_some()) {
        return Err("empty host".to_string());
    }

    let port = match port {
        None | Some("") => None,
        Some(digits) => {
            let port = digits
                .bytes()
                .all(|byte| byte.is_ascii_digit())
                .then(|| digits.parse::<u16>().ok())
                .flatten()
                .ok_or("invalid port number")?;

            (Some(port) != default_port).then_some(port)
        }
    };

    Ok(Some(Authority { scheme, host, port }))
}

/// Splits a URL into its scheme and the rest, when it starts with a valid scheme.
fn split_scheme(url: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = url.split_once(':')?;
    let mut characters = scheme.chars();
    let valid = characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        });

    valid.then_some((scheme, rest))
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
