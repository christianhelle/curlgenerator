//! Structural validation of an OpenAPI specification.
//!
//! The rules implemented here are the subset of the Microsoft.OpenApi validation rules that the
//! legacy .NET CLI surfaced for the specifications it ships tests for. Documents that fail any of
//! them are rejected unless `--skip-validation` is passed.

use std::collections::HashMap;

use curlgenerator_core::openapi::RawOpenApiDocument;
use serde_json::Value;

const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// A single validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The human readable failure description.
    pub message: String,
    /// The JSON pointer of the offending element.
    pub pointer: String,
}

impl Diagnostic {
    /// Renders the diagnostic the way the legacy CLI printed it.
    ///
    /// # Examples
    ///
    /// ```
    /// use curlgenerator_cli::validation::Diagnostic;
    ///
    /// let diagnostic = Diagnostic {
    ///     message: "Responses must contain at least one response".to_string(),
    ///     pointer: "#/paths/~1users/get/responses".to_string(),
    /// };
    ///
    /// assert_eq!(
    ///     diagnostic.to_string(),
    ///     "Responses must contain at least one response [#/paths/~1users/get/responses]"
    /// );
    /// ```
    pub fn new(message: impl Into<String>, pointer: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            pointer: pointer.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} [{}]", self.message, self.pointer)
    }
}

/// Validates a decoded document and returns every diagnostic it violates.
pub fn validate(document: &RawOpenApiDocument) -> Vec<Diagnostic> {
    validate_value(document.value())
}

/// Validates a decoded document tree.
pub fn validate_value(root: &Value) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let Some(paths) = root.get("paths").and_then(Value::as_object) else {
        return diagnostics;
    };

    let mut signatures: HashMap<String, Vec<&String>> = HashMap::new();
    for (path, item) in paths {
        signatures
            .entry(path_signature(path))
            .or_default()
            .push(path);

        let Some(members) = item.as_object() else {
            continue;
        };

        for (method, operation) in members {
            if !HTTP_METHODS.contains(&method.to_ascii_lowercase().as_str()) {
                continue;
            }

            let responses = operation.get("responses").and_then(Value::as_object);
            if responses.is_none_or(serde_json::Map::is_empty) {
                diagnostics.push(Diagnostic::new(
                    "Responses must contain at least one response",
                    format!("#/paths/{}/{method}/responses", escape_pointer(path)),
                ));
            }
        }
    }

    for (signature, duplicates) in signatures {
        if duplicates.len() < 2 {
            continue;
        }

        for path in duplicates {
            diagnostics.push(Diagnostic::new(
                format!("The path signature '{signature}' MUST be unique."),
                format!("#/paths/{}", escape_pointer(path)),
            ));
        }
    }

    diagnostics
}

/// Replaces every templated segment with an empty placeholder so paths that differ only in
/// parameter names collapse to the same signature.
///
/// # Examples
///
/// ```
/// use curlgenerator_cli::validation::path_signature;
///
/// assert_eq!(path_signature("/orders/{orderNumber}"), "/orders/{}");
/// ```
pub fn path_signature(path: &str) -> String {
    let mut signature = String::with_capacity(path.len());
    let mut depth = 0usize;

    for character in path.chars() {
        match character {
            '{' => {
                depth += 1;
                if depth == 1 {
                    signature.push('{');
                }
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        signature.push('}');
                    }
                }
            }
            _ if depth == 0 => signature.push(character),
            _ => {}
        }
    }

    signature
}

fn escape_pointer(path: &str) -> String {
    path.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_a_well_formed_document() {
        let diagnostics = validate_value(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/users": { "get": { "responses": { "200": { "description": "ok" } } } }
            }
        }));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn rejects_operations_without_responses() {
        let diagnostics = validate_value(&json!({
            "openapi": "3.0.0",
            "paths": { "/users": { "get": {} } }
        }));

        assert_eq!(
            diagnostics[0].to_string(),
            "Responses must contain at least one response [#/paths/~1users/get/responses]"
        );
    }

    #[test]
    fn rejects_operations_with_an_empty_responses_object() {
        let diagnostics = validate_value(&json!({
            "openapi": "3.0.0",
            "paths": { "/users": { "get": { "responses": {} } } }
        }));

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn rejects_duplicate_path_signatures() {
        let diagnostics = validate_value(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/orders/{orderNumber}": { "get": { "responses": { "200": { "description": "ok" } } } },
                "/orders/{ordernumber}": { "get": { "responses": { "200": { "description": "ok" } } } }
            }
        }));

        assert_eq!(diagnostics.len(), 2);
        assert!(
            diagnostics.iter().all(|diagnostic| diagnostic.message
                == "The path signature '/orders/{}' MUST be unique.")
        );
    }

    #[test]
    fn ignores_path_item_members_that_are_not_operations() {
        let diagnostics = validate_value(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/users": {
                    "summary": "Users",
                    "parameters": [],
                    "get": { "responses": { "200": { "description": "ok" } } }
                }
            }
        }));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn accepts_documents_without_paths() {
        assert!(validate_value(&json!({ "openapi": "3.1.0" })).is_empty());
    }

    #[test]
    fn collapses_templated_segments_into_a_signature() {
        assert_eq!(path_signature("/orders/{orderNumber}"), "/orders/{}");
        assert_eq!(path_signature("/a/{x}/b/{y}"), "/a/{}/b/{}");
        assert_eq!(path_signature("/plain"), "/plain");
    }

    #[test]
    fn escapes_json_pointer_segments() {
        assert_eq!(escape_pointer("/a/b"), "~1a~1b");
        assert_eq!(escape_pointer("/a~b"), "~1a~0b");
    }
}
