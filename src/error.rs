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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_parse_error() {
        let err = CurlGeneratorError::OpenApiParseError("test error".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to parse OpenAPI document: test error"
        );
    }

    #[test]
    fn test_invalid_specification() {
        let err = CurlGeneratorError::InvalidSpecification;
        assert_eq!(err.to_string(), "Invalid OpenAPI specification");
    }

    #[test]
    fn test_io_error_from_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = CurlGeneratorError::from(io_err);
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_json_error_from_conversion() {
        let json_str = "{invalid json}";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let err = CurlGeneratorError::from(json_err);
        assert!(err.to_string().contains("JSON parsing error"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CurlGeneratorError>();
    }
}
