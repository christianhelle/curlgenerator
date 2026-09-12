//! Resolution of the base URL that generated requests are sent to.

use crate::{
    normalized::Document,
    openapi::{authority_of, is_absolute_url, is_http},
};

/// Resolves the base URL for a document.
///
/// The configured base URL and the document's first server URL are concatenated, matching the
/// legacy .NET generator. When the result is not an absolute URL and the specification was loaded
/// over HTTP, the authority of the specification URL is prepended so relative server URLs still
/// resolve.
///
/// # Examples
///
/// ```
/// use curlgenerator::{base_url, normalized::Document};
///
/// let document = Document {
///     servers: vec!["/api/v3".to_string()],
///     paths: Vec::new(),
/// };
///
/// assert_eq!(base_url(&document, None, "./openapi.json"), "/api/v3");
/// assert_eq!(
///     base_url(&document, None, "https://petstore.swagger.io/openapi.json"),
///     "https://petstore.swagger.io/api/v3"
/// );
/// ```
pub fn base_url(document: &Document, configured: Option<&str>, open_api_path: &str) -> String {
    let server = document.servers.first().map(String::as_str).unwrap_or("");
    let base_url = format!("{}{server}", configured.unwrap_or(""));

    if is_absolute_url(&base_url) || !is_http(open_api_path) {
        return base_url;
    }

    match authority_of(open_api_path) {
        Some(authority) => format!("{authority}{base_url}"),
        None => base_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(servers: &[&str]) -> Document {
        Document {
            servers: servers.iter().map(|server| server.to_string()).collect(),
            paths: Vec::new(),
        }
    }

    #[test]
    fn uses_the_first_server_url_when_it_is_absolute() {
        assert_eq!(
            base_url(
                &document(&[
                    "https://petstore.swagger.io/v2",
                    "http://petstore.swagger.io/v2"
                ]),
                None,
                "./openapi.json"
            ),
            "https://petstore.swagger.io/v2"
        );
    }

    #[test]
    fn prepends_the_configured_base_url() {
        assert_eq!(
            base_url(
                &document(&["/api/v3"]),
                Some("https://my-host.example"),
                "./openapi.json"
            ),
            "https://my-host.example/api/v3"
        );
    }

    #[test]
    fn keeps_relative_server_urls_for_local_specifications() {
        assert_eq!(
            base_url(&document(&["/api/v3"]), None, "./openapi.json"),
            "/api/v3"
        );
    }

    #[test]
    fn prepends_the_specification_authority_for_relative_remote_server_urls() {
        assert_eq!(
            base_url(
                &document(&["/api/v3"]),
                None,
                "https://petstore3.swagger.io/api/v3/openapi.json"
            ),
            "https://petstore3.swagger.io/api/v3"
        );
    }

    #[test]
    fn falls_back_to_the_specification_authority_without_any_server() {
        assert_eq!(
            base_url(
                &document(&[]),
                None,
                "https://petstore3.swagger.io/openapi.json"
            ),
            "https://petstore3.swagger.io"
        );
        assert_eq!(base_url(&document(&[]), None, "./openapi.json"), "");
    }

    #[test]
    fn treats_a_base_url_with_any_scheme_as_absolute() {
        for configured in ["localhost:8080", "custom://api"] {
            assert_eq!(
                base_url(
                    &document(&["/api"]),
                    Some(configured),
                    "https://petstore3.swagger.io/openapi.json"
                ),
                format!("{configured}/api")
            );
        }
    }

    #[test]
    fn prepends_the_specification_authority_to_a_base_url_without_a_scheme() {
        assert_eq!(
            base_url(
                &document(&["/api"]),
                Some("/v1"),
                "https://petstore3.swagger.io/openapi.json"
            ),
            "https://petstore3.swagger.io/v1/api"
        );
    }
}
