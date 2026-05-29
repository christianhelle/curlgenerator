//! Lightweight OpenAPI validation and statistics gathering.
//!
//! Validation parses the specification and confirms it is a JSON/YAML object
//! that declares either `openapi` or `swagger`. Statistics are gathered by
//! walking the document structure.

use serde_json::Value;

use crate::http;
use crate::model;

const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// Counts of the major OpenAPI components in a document.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OpenApiStats {
    pub path_item_count: usize,
    pub operation_count: usize,
    pub parameter_count: usize,
    pub request_body_count: usize,
    pub response_count: usize,
    pub link_count: usize,
    pub callback_count: usize,
    pub schema_count: usize,
    pub header_count: usize,
}

/// The outcome of validating an OpenAPI specification.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub statistics: OpenApiStats,
}

impl ValidationResult {
    /// Returns `true` when there are no validation errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validates an OpenAPI specification located at a file path or URL.
///
/// Returns `Err` when the document cannot be loaded or parsed at all (e.g. a
/// bad URL or content that is not a recognizable OpenAPI document).
pub fn validate(open_api_path: &str) -> Result<ValidationResult, String> {
    let text = http::read_spec(open_api_path)?;
    let root = model::parse_value(&text)?;

    let is_object = root.is_object();
    let has_version = root.get("openapi").is_some() || root.get("swagger").is_some();
    if !is_object || !has_version {
        return Err("The document is not a valid OpenAPI specification".to_string());
    }

    Ok(ValidationResult {
        errors: Vec::new(),
        warnings: Vec::new(),
        statistics: compute_stats(&root),
    })
}

/// Computes [`OpenApiStats`] by walking the raw document structure.
pub fn compute_stats(root: &Value) -> OpenApiStats {
    let mut stats = OpenApiStats::default();

    if let Some(paths) = root.get("paths").and_then(Value::as_object) {
        for (_, path_item) in paths {
            stats.path_item_count += 1;
            count_parameters(path_item.get("parameters"), &mut stats);

            for method in HTTP_METHODS {
                let Some(operation) = path_item.get(method) else {
                    continue;
                };
                stats.operation_count += 1;
                count_parameters(operation.get("parameters"), &mut stats);

                if operation.get("requestBody").is_some() {
                    stats.request_body_count += 1;
                }

                if let Some(responses) = operation.get("responses").and_then(Value::as_object) {
                    stats.response_count += 1;
                    for (_, response) in responses {
                        count_map(response.get("headers"), &mut stats.header_count);
                        count_map(response.get("links"), &mut stats.link_count);
                    }
                }

                count_map(operation.get("callbacks"), &mut stats.callback_count);
            }
        }
    }

    count_map(
        root.pointer("/components/schemas")
            .or_else(|| root.get("definitions")),
        &mut stats.schema_count,
    );
    count_map(root.pointer("/components/links"), &mut stats.link_count);
    count_map(
        root.pointer("/components/callbacks"),
        &mut stats.callback_count,
    );
    count_map(root.pointer("/components/headers"), &mut stats.header_count);

    stats
}

fn count_parameters(value: Option<&Value>, stats: &mut OpenApiStats) {
    if let Some(array) = value.and_then(Value::as_array) {
        stats.parameter_count += array.len();
    }
}

fn count_map(value: Option<&Value>, counter: &mut usize) {
    if let Some(map) = value.and_then(Value::as_object) {
        *counter += map.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_all_elements_of_a_full_document() {
        let document = serde_json::json!({
            "openapi": "3.0.0",
            "paths": {
                "/test": {
                    "get": {
                        "parameters": [ { "in": "query", "name": "param1" } ],
                        "requestBody": {},
                        "responses": {
                            "200": { "headers": { "X-Rate-Limit": {} } }
                        },
                        "callbacks": { "onData": {} }
                    }
                }
            },
            "components": {
                "schemas": { "mySchema": {} },
                "links": { "myLink": {} }
            }
        });

        let stats = compute_stats(&document);
        assert_eq!(stats.path_item_count, 1);
        assert_eq!(stats.operation_count, 1);
        assert_eq!(stats.parameter_count, 1);
        assert_eq!(stats.request_body_count, 1);
        assert_eq!(stats.response_count, 1);
        assert_eq!(stats.link_count, 1);
        assert_eq!(stats.callback_count, 1);
        assert_eq!(stats.schema_count, 1);
        assert_eq!(stats.header_count, 1);
    }

    #[test]
    fn validate_rejects_non_specification_text() {
        let dir = std::env::temp_dir().join(format!("curlgen-{}", uuid_like()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("random.json");
        std::fs::write(&file, "this-is-not-a-spec").unwrap();

        let result = validate(file.to_str().unwrap());
        assert!(result.is_err());
    }

    fn uuid_like() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
