//! Reading and decoding OpenAPI specifications from disk or over HTTP.

use std::{fmt, fs, time::Duration};

use serde_json::Value;

use super::{OpenApiSource, SourceClassificationError, classify_source};

/// The serialization format a specification was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApiContentFormat {
    /// A JSON encoded document.
    Json,
    /// A YAML encoded document.
    Yaml,
}

impl fmt::Display for OpenApiContentFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(formatter, "JSON"),
            Self::Yaml => write!(formatter, "YAML"),
        }
    }
}

/// The specification family a document belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApiSpecificationVersion {
    /// A Swagger 2.0 document.
    Swagger2,
    /// An OpenAPI 3.0.x document.
    OpenApi30,
    /// An OpenAPI 3.1.x document.
    OpenApi31,
}

impl fmt::Display for OpenApiSpecificationVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Swagger2 => write!(formatter, "Swagger 2.0"),
            Self::OpenApi30 => write!(formatter, "OpenAPI 3.0.x"),
            Self::OpenApi31 => write!(formatter, "OpenAPI 3.1.x"),
        }
    }
}

/// Errors raised while loading or decoding a specification.
#[derive(Debug)]
pub enum OpenApiLoadError {
    /// The input could not be classified as a path or URL.
    SourceClassification(SourceClassificationError),
    /// Reading the local file failed.
    FileRead {
        /// The file that could not be read.
        path: String,
        /// The underlying failure description.
        reason: String,
    },
    /// Downloading the remote document failed.
    HttpRequest {
        /// The URL that could not be downloaded.
        url: String,
        /// The underlying failure description.
        reason: String,
    },
    /// The payload was neither valid JSON nor valid YAML.
    Decode {
        /// The parser failure description.
        reason: String,
    },
    /// The document did not declare a supported `openapi` or `swagger` version.
    UnsupportedVersion(String),
}

impl fmt::Display for OpenApiLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceClassification(error) => write!(formatter, "{error}"),
            Self::FileRead { path, reason } => {
                write!(formatter, "could not open the file at {path}: {reason}")
            }
            Self::HttpRequest { url, reason } => {
                write!(formatter, "could not download the file at {url}: {reason}")
            }
            Self::Decode { reason } => {
                write!(formatter, "could not decode the OpenAPI document: {reason}")
            }
            Self::UnsupportedVersion(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for OpenApiLoadError {}

impl From<SourceClassificationError> for OpenApiLoadError {
    fn from(error: SourceClassificationError) -> Self {
        Self::SourceClassification(error)
    }
}

/// A decoded OpenAPI document together with the metadata describing where it came from.
#[derive(Debug, Clone)]
pub struct RawOpenApiDocument {
    source: OpenApiSource,
    format: OpenApiContentFormat,
    version: OpenApiSpecificationVersion,
    value: Value,
}

impl RawOpenApiDocument {
    /// Returns the path or URL the document was loaded from.
    pub fn source(&self) -> &OpenApiSource {
        &self.source
    }

    /// Returns the detected serialization format.
    pub fn format(&self) -> OpenApiContentFormat {
        self.format
    }

    /// Returns the detected specification version.
    pub fn version(&self) -> OpenApiSpecificationVersion {
        self.version
    }

    /// Returns the decoded document tree.
    pub fn value(&self) -> &Value {
        &self.value
    }
}

/// Loads and decodes an OpenAPI document from a path or URL.
pub fn load_document(input: &str) -> Result<RawOpenApiDocument, OpenApiLoadError> {
    let source = classify_source(input)?;
    let content = read_source(&source)?;

    decode_document(source, &content)
}

/// Decodes in-memory specification content into a [`RawOpenApiDocument`].
///
/// # Examples
///
/// ```
/// use curlgenerator::openapi::{OpenApiSource, OpenApiSpecificationVersion, decode_document};
/// use std::path::PathBuf;
///
/// let document = decode_document(
///     OpenApiSource::Path(PathBuf::from("openapi.yaml")),
///     "openapi: 3.0.0\npaths: {}\n",
/// )
/// .unwrap();
///
/// assert_eq!(document.version(), OpenApiSpecificationVersion::OpenApi30);
/// ```
pub fn decode_document(
    source: OpenApiSource,
    content: &str,
) -> Result<RawOpenApiDocument, OpenApiLoadError> {
    let (format, value) = decode_content(content)?;
    let version = detect_specification_version(&value)?;

    Ok(RawOpenApiDocument {
        source,
        format,
        version,
        value,
    })
}

/// Detects the specification version declared by a decoded document.
pub fn detect_specification_version(
    value: &Value,
) -> Result<OpenApiSpecificationVersion, OpenApiLoadError> {
    if let Some(version) = value.get("openapi").and_then(Value::as_str) {
        return match major_minor(version) {
            Some((3, 0)) => Ok(OpenApiSpecificationVersion::OpenApi30),
            Some((3, 1)) => Ok(OpenApiSpecificationVersion::OpenApi31),
            _ => Err(OpenApiLoadError::UnsupportedVersion(format!(
                "OpenAPI specification version '{version}' is not supported"
            ))),
        };
    }

    if let Some(version) = value.get("swagger").and_then(Value::as_str) {
        return match major_minor(version) {
            Some((2, 0)) => Ok(OpenApiSpecificationVersion::Swagger2),
            _ => Err(OpenApiLoadError::UnsupportedVersion(format!(
                "Swagger specification version '{version}' is not supported"
            ))),
        };
    }

    Err(OpenApiLoadError::UnsupportedVersion(
        "the document does not declare an 'openapi' or 'swagger' version".to_string(),
    ))
}

fn decode_content(content: &str) -> Result<(OpenApiContentFormat, Value), OpenApiLoadError> {
    if content.trim_start().starts_with(['{', '[']) {
        return serde_json::from_str(content)
            .map(|value| (OpenApiContentFormat::Json, value))
            .map_err(|error| OpenApiLoadError::Decode {
                reason: error.to_string(),
            });
    }

    yaml_serde::from_str(content)
        .map(|value| (OpenApiContentFormat::Yaml, value))
        .map_err(|error| OpenApiLoadError::Decode {
            reason: error.to_string(),
        })
}

fn read_source(source: &OpenApiSource) -> Result<String, OpenApiLoadError> {
    match source {
        OpenApiSource::Path(path) => {
            fs::read_to_string(path).map_err(|error| OpenApiLoadError::FileRead {
                path: path.display().to_string(),
                reason: error.to_string(),
            })
        }
        OpenApiSource::Url(url) => download(url.as_str()),
    }
}

fn download(url: &str) -> Result<String, OpenApiLoadError> {
    let http_error = |error: reqwest::Error| OpenApiLoadError::HttpRequest {
        url: url.to_string(),
        reason: error.to_string(),
    };

    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .gzip(true)
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(http_error)?;

    client
        .get(url)
        .send()
        .map_err(http_error)?
        .error_for_status()
        .map_err(http_error)?
        .text()
        .map_err(http_error)
}

fn major_minor(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.split('.');
    let major = parts.next()?.trim().parse().ok()?;
    let minor = parts.next()?.trim().parse().ok()?;

    Some((major, minor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path_source() -> OpenApiSource {
        OpenApiSource::Path(PathBuf::from("openapi.json"))
    }

    #[test]
    fn decode_document_reads_json_content() {
        let document = decode_document(
            path_source(),
            r#"{ "openapi": "3.0.1", "info": {}, "paths": {} }"#,
        )
        .unwrap();

        assert_eq!(document.format(), OpenApiContentFormat::Json);
        assert_eq!(document.version(), OpenApiSpecificationVersion::OpenApi30);
    }

    #[test]
    fn decode_document_reads_yaml_content() {
        let document = decode_document(path_source(), "swagger: \"2.0\"\npaths: {}\n").unwrap();

        assert_eq!(document.format(), OpenApiContentFormat::Yaml);
        assert_eq!(document.version(), OpenApiSpecificationVersion::Swagger2);
    }

    #[test]
    fn decode_document_recognizes_openapi_31() {
        let document = decode_document(path_source(), "openapi: 3.1.0\npaths: {}\n").unwrap();

        assert_eq!(document.version(), OpenApiSpecificationVersion::OpenApi31);
    }

    #[test]
    fn decode_document_rejects_documents_without_a_version() {
        let error = decode_document(path_source(), r#"{ "info": {} }"#).unwrap_err();

        assert!(matches!(error, OpenApiLoadError::UnsupportedVersion(_)));
    }

    #[test]
    fn decode_document_rejects_unsupported_versions() {
        let error = decode_document(path_source(), "openapi: 4.0.0\n").unwrap_err();

        assert!(matches!(error, OpenApiLoadError::UnsupportedVersion(_)));
    }

    #[test]
    fn decode_document_reports_malformed_payloads() {
        let error = decode_document(path_source(), "{ not json ").unwrap_err();

        assert!(matches!(error, OpenApiLoadError::Decode { .. }));
    }

    #[test]
    fn load_document_reports_missing_files() {
        let error = load_document("./does-not-exist.json").unwrap_err();

        assert!(matches!(error, OpenApiLoadError::FileRead { .. }));
    }

    #[test]
    fn load_errors_describe_what_went_wrong() {
        let message = |input: &str| load_document(input).unwrap_err().to_string();

        assert_eq!(message("   "), "the OpenAPI path is empty");
        assert_eq!(
            message("http://"),
            "could not parse 'http://' as a URL: empty host"
        );
        assert!(
            message("./does-not-exist.json")
                .starts_with("could not open the file at ./does-not-exist.json: ")
        );
        assert!(
            decode_document(path_source(), "{ not json ")
                .unwrap_err()
                .to_string()
                .starts_with("could not decode the OpenAPI document: ")
        );
        assert_eq!(
            decode_document(path_source(), "openapi: 4.0.0\n")
                .unwrap_err()
                .to_string(),
            "OpenAPI specification version '4.0.0' is not supported"
        );
    }

    #[test]
    fn specification_versions_render_friendly_names() {
        assert_eq!(
            OpenApiSpecificationVersion::Swagger2.to_string(),
            "Swagger 2.0"
        );
        assert_eq!(
            OpenApiSpecificationVersion::OpenApi30.to_string(),
            "OpenAPI 3.0.x"
        );
        assert_eq!(
            OpenApiSpecificationVersion::OpenApi31.to_string(),
            "OpenAPI 3.1.x"
        );
        assert_eq!(OpenApiContentFormat::Json.to_string(), "JSON");
        assert_eq!(OpenApiContentFormat::Yaml.to_string(), "YAML");
    }
}
