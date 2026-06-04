use thiserror::Error;

use crate::openapi;
use crate::stats;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("OpenAPI validation failed")]
    ValidationFailed {
        source: OpenApiValidationFailed,
    },
    #[error("Unsupported OpenAPI spec version")]
    UnsupportedVersion,
}

#[derive(Error, Debug)]
#[error("OpenAPI validation failed")]
pub struct OpenApiValidationFailed {
    pub diagnostics: Vec<String>,
    pub statistics: stats::OpenApiStats,
}

pub async fn validate(openapi_path: &str) -> Result<ValidationResult, ValidationError> {
    let doc = openapi::load_from_path(openapi_path).await
        .map_err(|e| ValidationError::ValidationFailed {
            source: OpenApiValidationFailed {
                diagnostics: vec![e],
                statistics: stats::OpenApiStats::default(),
            },
        })?;

    let statistics = stats::compute_stats(&doc);

    let is_valid = true; // The doc is already validated by loading

    Ok(ValidationResult {
        is_valid,
        diagnostics: Vec::new(),
        statistics,
    })
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub diagnostics: Vec<String>,
    pub statistics: stats::OpenApiStats,
}
