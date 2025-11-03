use thiserror::Error;

#[derive(Error, Debug)]
pub enum CurlGeneratorError {
    #[allow(dead_code)]
    #[error("Failed to load OpenAPI document: {0}")]
    OpenApiLoadError(String),

    #[error("Failed to parse OpenAPI document: {0}")]
    OpenApiParseError(String),

    #[allow(dead_code)]
    #[error("Invalid OpenAPI specification")]
    InvalidSpecification,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
