//! A lenient, normalized OpenAPI model built from a `serde_json::Value`.
//!
//! The model intentionally captures only the parts of an OpenAPI document that
//! are required to generate cURL requests: servers, paths, operations,
//! parameters and request bodies. It normalizes the differences between
//! OpenAPI 2.0 (Swagger) and 3.x so the generator can treat them uniformly.

use serde_json::Value;

use crate::http;

const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// A request/response parameter (path, query, header or cookie).
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    /// The parameter location: `path`, `query`, `header` or `cookie`.
    pub location: String,
    pub description: Option<String>,
}

/// A single media type entry of a request body.
#[derive(Debug, Clone)]
pub struct ContentEntry {
    pub content_type: String,
    pub schema: Option<Value>,
}

/// A normalized request body (v3 `requestBody` or synthesized from v2 params).
#[derive(Debug, Clone, Default)]
pub struct RequestBody {
    pub content: Vec<ContentEntry>,
}

impl RequestBody {
    /// Returns the first declared content type, if any.
    pub fn first_content_type(&self) -> Option<&str> {
        self.content.first().map(|c| c.content_type.as_str())
    }

    /// Returns the schema for the given content type, if present.
    pub fn schema_for(&self, content_type: &str) -> Option<&Value> {
        self.content
            .iter()
            .find(|c| c.content_type == content_type)
            .and_then(|c| c.schema.as_ref())
    }
}

/// A single operation (HTTP method) on a path.
#[derive(Debug, Clone)]
pub struct Operation {
    /// Lowercase HTTP method, e.g. `get`.
    pub method: String,
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    /// Whether the operation declared an operation-level `parameters` array
    /// (mirrors .NET's `operation.Parameters is null` check). A missing key or
    /// an explicit `null` is `false`; an empty array `[]` is `true`.
    pub declares_parameters: bool,
    pub request_body: Option<RequestBody>,
}

/// A path and the operations defined on it.
#[derive(Debug, Clone)]
pub struct PathItem {
    pub path: String,
    pub operations: Vec<Operation>,
}

/// A normalized OpenAPI document.
#[derive(Debug, Clone)]
pub struct Document {
    pub servers: Vec<String>,
    pub paths: Vec<PathItem>,
    pub is_v2: bool,
    /// The raw parsed document, used for `$ref` resolution.
    pub root: Value,
}

/// Parses raw JSON or YAML text into a `serde_json::Value`.
pub fn parse_value(text: &str) -> Result<Value, String> {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return Ok(value);
    }
    serde_yaml::from_str::<Value>(text).map_err(|e| format!("Failed to parse specification: {e}"))
}

/// Loads an OpenAPI specification from a file or URL and normalizes it.
pub fn load(open_api_path: &str) -> Result<Document, String> {
    let text = http::read_spec(open_api_path)?;
    let root = parse_value(&text)?;
    Ok(from_value(root))
}

/// Resolves a `$ref` JSON pointer (`#/a/b/c`) against the root document.
///
/// Only same-document references are supported. Returns `None` for external or
/// unresolvable references.
pub fn resolve_ref<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    let pointer = reference.strip_prefix("#/")?;
    let mut current = root;
    for raw_segment in pointer.split('/') {
        let segment = raw_segment.replace("~1", "/").replace("~0", "~");
        current = current.get(&segment)?;
    }
    Some(current)
}

/// If `value` is a `$ref`, resolves it against the root, otherwise returns it.
///
/// Follows chains of references up to a bounded depth to avoid infinite loops
/// on cyclic documents.
pub fn resolve<'a>(root: &'a Value, value: &'a Value) -> &'a Value {
    deref(root, value)
}

/// If `value` is a `$ref`, resolves it against the root, otherwise returns it.
fn deref<'a>(root: &'a Value, value: &'a Value) -> &'a Value {
    let mut current = value;
    // Follow chains of references but bound the depth to avoid cycles.
    for _ in 0..32 {
        match current.get("$ref").and_then(Value::as_str) {
            Some(reference) => match resolve_ref(root, reference) {
                Some(resolved) => current = resolved,
                None => return current,
            },
            None => return current,
        }
    }
    current
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

/// Builds the normalized [`Document`] from a raw parsed value.
pub fn from_value(root: Value) -> Document {
    let is_v2 = root
        .get("swagger")
        .and_then(Value::as_str)
        .map(|v| v.starts_with('2'))
        .unwrap_or(false);

    let servers = if is_v2 {
        v2_servers(&root)
    } else {
        v3_servers(&root)
    };

    let doc_consumes = string_array(root.get("consumes"));

    let mut paths = Vec::new();
    if let Some(path_map) = root.get("paths").and_then(Value::as_object) {
        for (path, raw_item) in path_map {
            if path.starts_with("x-") {
                continue;
            }
            let item = deref(&root, raw_item);
            let path_level_params = parse_parameters(&root, item.get("parameters"));

            let mut operations = Vec::new();
            for method in HTTP_METHODS {
                let Some(raw_op) = item.get(method) else {
                    continue;
                };
                let op = deref(&root, raw_op);
                operations.push(parse_operation(
                    &root,
                    method,
                    op,
                    &path_level_params,
                    is_v2,
                    &doc_consumes,
                ));
            }

            paths.push(PathItem {
                path: path.clone(),
                operations,
            });
        }
    }

    Document {
        servers,
        paths,
        is_v2,
        root,
    }
}

fn v3_servers(root: &Value) -> Vec<String> {
    root.get("servers")
        .and_then(Value::as_array)
        .map(|servers| {
            servers
                .iter()
                .filter_map(|s| s.get("url").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn v2_servers(root: &Value) -> Vec<String> {
    let host = root.get("host").and_then(Value::as_str);
    let base_path = root.get("basePath").and_then(Value::as_str).unwrap_or("");
    let scheme = root
        .get("schemes")
        .and_then(Value::as_array)
        .and_then(|s| s.first())
        .and_then(Value::as_str)
        .unwrap_or("https");

    match host {
        Some(host) => vec![format!("{scheme}://{host}{base_path}")],
        None if !base_path.is_empty() => vec![base_path.to_string()],
        None => Vec::new(),
    }
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_operation(
    root: &Value,
    method: &str,
    op: &Value,
    path_level_params: &[ParsedParameter],
    is_v2: bool,
    doc_consumes: &[String],
) -> Operation {
    let mut all = path_level_params.to_vec();
    all.extend(parse_parameters(root, op.get("parameters")));

    let request_body = if is_v2 {
        synthesize_v2_request_body(root, &all, op, doc_consumes)
    } else {
        parse_v3_request_body(root, op.get("requestBody"))
    };

    // Only path/query/header/cookie parameters remain in the parameter list.
    let parameters = all
        .into_iter()
        .filter(|p| matches!(p.location.as_str(), "path" | "query" | "header" | "cookie"))
        .map(|p| Parameter {
            name: p.name,
            location: p.location,
            description: p.description,
        })
        .collect();

    Operation {
        method: method.to_string(),
        operation_id: string_field(op, "operationId"),
        summary: string_field(op, "summary"),
        description: string_field(op, "description"),
        parameters,
        declares_parameters: op.get("parameters").and_then(Value::as_array).is_some(),
        request_body,
    }
}

/// An intermediate parameter representation that also retains the v2 `body`
/// schema and `formData` type, which are dropped from the public [`Parameter`].
#[derive(Debug, Clone)]
struct ParsedParameter {
    name: String,
    location: String,
    description: Option<String>,
    schema: Option<Value>,
    is_file: bool,
}

fn parse_parameters(root: &Value, value: Option<&Value>) -> Vec<ParsedParameter> {
    let Some(array) = value.and_then(Value::as_array) else {
        return Vec::new();
    };

    array
        .iter()
        .map(|raw| deref(root, raw))
        .map(|param| {
            let name = string_field(param, "name").unwrap_or_default();
            let location = param
                .get("in")
                .and_then(Value::as_str)
                .unwrap_or("query")
                .to_string();
            let schema = param.get("schema").cloned();
            let is_file = param.get("type").and_then(Value::as_str) == Some("file");
            ParsedParameter {
                name,
                location,
                description: string_field(param, "description"),
                schema,
                is_file,
            }
        })
        .collect()
}

fn parse_v3_request_body(root: &Value, value: Option<&Value>) -> Option<RequestBody> {
    let body = deref(root, value?);
    let content = body.get("content").and_then(Value::as_object)?;

    let entries = content
        .iter()
        .map(|(content_type, media)| ContentEntry {
            content_type: content_type.clone(),
            schema: media.get("schema").cloned(),
        })
        .collect::<Vec<_>>();

    if entries.is_empty() {
        return Some(RequestBody::default());
    }
    Some(RequestBody { content: entries })
}

fn synthesize_v2_request_body(
    root: &Value,
    params: &[ParsedParameter],
    op: &Value,
    doc_consumes: &[String],
) -> Option<RequestBody> {
    let consumes = {
        let op_consumes = string_array(op.get("consumes"));
        if !op_consumes.is_empty() {
            op_consumes
        } else if !doc_consumes.is_empty() {
            doc_consumes.to_vec()
        } else {
            vec!["application/json".to_string()]
        }
    };

    if let Some(body) = params.iter().find(|p| p.location == "body") {
        let content = consumes
            .iter()
            .map(|content_type| ContentEntry {
                content_type: content_type.clone(),
                schema: body.schema.clone(),
            })
            .collect();
        return Some(RequestBody { content });
    }

    let form_params: Vec<&ParsedParameter> =
        params.iter().filter(|p| p.location == "formData").collect();
    if !form_params.is_empty() {
        let has_file = form_params.iter().any(|p| p.is_file);
        let content_type = if has_file {
            "multipart/form-data"
        } else {
            "application/x-www-form-urlencoded"
        };

        let mut properties = serde_json::Map::new();
        for param in &form_params {
            let _ = root; // root not needed here, kept for signature symmetry
            properties.insert(param.name.clone(), serde_json::json!({ "type": "string" }));
        }
        let schema = serde_json::json!({
            "type": "object",
            "properties": Value::Object(properties),
        });

        return Some(RequestBody {
            content: vec![ContentEntry {
                content_type: content_type.to_string(),
                schema: Some(schema),
            }],
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_ref_resolves_internal_pointer() {
        let root = serde_json::json!({
            "components": { "schemas": { "Pet": { "type": "object" } } }
        });
        let resolved = resolve_ref(&root, "#/components/schemas/Pet").unwrap();
        assert_eq!(resolved["type"], "object");
    }

    #[test]
    fn resolve_ref_returns_none_for_external() {
        let root = serde_json::json!({});
        assert!(resolve_ref(&root, "./other.yaml#/Pet").is_none());
    }

    #[test]
    fn detects_v2_documents() {
        let doc = from_value(serde_json::json!({
            "swagger": "2.0",
            "host": "petstore.swagger.io",
            "basePath": "/v2",
            "schemes": ["https", "http"],
            "paths": {}
        }));
        assert!(doc.is_v2);
        assert_eq!(doc.servers, vec!["https://petstore.swagger.io/v2"]);
    }

    #[test]
    fn detects_v3_servers() {
        let doc = from_value(serde_json::json!({
            "openapi": "3.0.0",
            "servers": [{ "url": "http://petstore.swagger.io/api" }],
            "paths": {}
        }));
        assert!(!doc.is_v2);
        assert_eq!(doc.servers, vec!["http://petstore.swagger.io/api"]);
    }

    #[test]
    fn parses_operations_and_parameters() {
        let doc = from_value(serde_json::json!({
            "openapi": "3.0.0",
            "paths": {
                "/pets/{id}": {
                    "get": {
                        "operationId": "getPet",
                        "parameters": [
                            { "name": "id", "in": "path", "description": "the id" }
                        ]
                    }
                }
            }
        }));
        assert_eq!(doc.paths.len(), 1);
        let op = &doc.paths[0].operations[0];
        assert_eq!(op.method, "get");
        assert_eq!(op.operation_id.as_deref(), Some("getPet"));
        assert_eq!(op.parameters.len(), 1);
        assert_eq!(op.parameters[0].name, "id");
        assert_eq!(op.parameters[0].location, "path");
    }

    #[test]
    fn synthesizes_v2_body_request_body() {
        let doc = from_value(serde_json::json!({
            "swagger": "2.0",
            "consumes": ["application/json"],
            "paths": {
                "/pets": {
                    "post": {
                        "operationId": "addPet",
                        "parameters": [
                            { "name": "body", "in": "body", "schema": { "type": "object" } }
                        ]
                    }
                }
            }
        }));
        let op = &doc.paths[0].operations[0];
        assert_eq!(op.parameters.len(), 0);
        let body = op.request_body.as_ref().unwrap();
        assert_eq!(body.first_content_type(), Some("application/json"));
    }

    #[test]
    fn synthesizes_v2_form_data_request_body() {
        let doc = from_value(serde_json::json!({
            "swagger": "2.0",
            "paths": {
                "/upload": {
                    "post": {
                        "operationId": "upload",
                        "parameters": [
                            { "name": "file", "in": "formData", "type": "file" }
                        ]
                    }
                }
            }
        }));
        let op = &doc.paths[0].operations[0];
        let body = op.request_body.as_ref().unwrap();
        assert_eq!(body.first_content_type(), Some("multipart/form-data"));
    }
}
