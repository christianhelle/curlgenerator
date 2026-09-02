//! Conversion of decoded OpenAPI documents into the shared [`Document`] model.

use serde_json::Value;

use crate::normalized::{
    Document, MediaType, Operation, Parameter, ParameterLocation, PathItem, RequestBody, Schema,
    SchemaType,
};

use super::{OpenApiSpecificationVersion, RawOpenApiDocument};

/// HTTP methods recognized on a path item, matching the OpenAPI specification.
const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// Normalizes a decoded document into the version independent model.
pub fn normalize(document: &RawOpenApiDocument) -> Document {
    normalize_value(document.value(), document.version())
}

/// Normalizes a decoded document tree for an explicit specification version.
///
/// # Examples
///
/// ```
/// use curlgenerator_core::openapi::{OpenApiSpecificationVersion, normalize_value};
/// use serde_json::json;
///
/// let document = normalize_value(
///     &json!({
///         "openapi": "3.0.0",
///         "servers": [{ "url": "https://example.com" }],
///         "paths": { "/pets": { "get": { "operationId": "listPets" } } }
///     }),
///     OpenApiSpecificationVersion::OpenApi30,
/// );
///
/// assert_eq!(document.servers, vec!["https://example.com".to_string()]);
/// assert_eq!(document.paths[0].operations[0].method, "GET");
/// ```
pub fn normalize_value(value: &Value, version: OpenApiSpecificationVersion) -> Document {
    let is_swagger2 = version == OpenApiSpecificationVersion::Swagger2;
    let servers = if is_swagger2 {
        swagger2_servers(value)
    } else {
        openapi3_servers(value)
    };

    Document {
        servers,
        paths: normalize_paths(value, is_swagger2),
    }
}

fn openapi3_servers(root: &Value) -> Vec<String> {
    root.get("servers")
        .and_then(Value::as_array)
        .map(|servers| {
            servers
                .iter()
                .filter_map(|server| server.get("url").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn swagger2_servers(root: &Value) -> Vec<String> {
    let Some(host) = root.get("host").and_then(Value::as_str) else {
        return Vec::new();
    };

    let base_path = root
        .get("basePath")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let schemes: Vec<&str> = root
        .get("schemes")
        .and_then(Value::as_array)
        .map(|schemes| schemes.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    if schemes.is_empty() {
        return vec![format!("http://{host}{base_path}")];
    }

    schemes
        .into_iter()
        .map(|scheme| format!("{scheme}://{host}{base_path}"))
        .collect()
}

fn normalize_paths(root: &Value, is_swagger2: bool) -> Vec<PathItem> {
    let Some(paths) = root.get("paths").and_then(Value::as_object) else {
        return Vec::new();
    };

    paths
        .iter()
        .map(|(path, item)| PathItem {
            path: path.clone(),
            operations: normalize_operations(root, item, is_swagger2),
        })
        .collect()
}

fn normalize_operations(root: &Value, item: &Value, is_swagger2: bool) -> Vec<Operation> {
    let item = resolve_reference(root, item);
    let Some(item) = item.as_object() else {
        return Vec::new();
    };

    item.iter()
        .filter(|(key, _)| HTTP_METHODS.contains(&key.to_ascii_lowercase().as_str()))
        .map(|(method, operation)| {
            normalize_operation(root, &method.to_ascii_uppercase(), operation, is_swagger2)
        })
        .collect()
}

fn normalize_operation(
    root: &Value,
    method: &str,
    operation: &Value,
    is_swagger2: bool,
) -> Operation {
    let declared = operation
        .get("parameters")
        .and_then(Value::as_array)
        .map(|parameters| {
            parameters
                .iter()
                .map(|parameter| resolve_reference(root, parameter))
                .collect::<Vec<_>>()
        });

    let (parameters, request_body) = if is_swagger2 {
        swagger2_parameters(root, operation, declared)
    } else {
        (
            declared.map(|parameters| {
                parameters
                    .iter()
                    .filter_map(|parameter| normalize_parameter(parameter))
                    .collect()
            }),
            openapi3_request_body(root, operation),
        )
    };

    Operation {
        method: method.to_string(),
        operation_id: string_field(operation, "operationId"),
        summary: string_field(operation, "summary"),
        description: string_field(operation, "description"),
        parameters,
        request_body,
    }
}

fn normalize_parameter(parameter: &Value) -> Option<Parameter> {
    let name = parameter.get("name").and_then(Value::as_str)?;
    let location = match parameter.get("in").and_then(Value::as_str)? {
        "path" => ParameterLocation::Path,
        "query" => ParameterLocation::Query,
        "header" => ParameterLocation::Header,
        "cookie" => ParameterLocation::Cookie,
        _ => return None,
    };

    Some(Parameter {
        name: name.to_string(),
        location,
        description: string_field(parameter, "description"),
    })
}

fn openapi3_request_body(root: &Value, operation: &Value) -> Option<RequestBody> {
    let request_body = operation.get("requestBody")?;
    let request_body = resolve_reference(root, request_body);
    let content = request_body.get("content").and_then(Value::as_object)?;

    Some(RequestBody {
        content: content
            .iter()
            .map(|(content_type, media_type)| MediaType {
                content_type: content_type.clone(),
                schema: media_type
                    .get("schema")
                    .map(|schema| normalize_schema(root, schema, &mut Vec::new())),
            })
            .collect(),
    })
}

/// Splits Swagger 2.0 parameters into regular parameters and a converted request body.
fn swagger2_parameters(
    root: &Value,
    operation: &Value,
    declared: Option<Vec<&Value>>,
) -> (Option<Vec<Parameter>>, Option<RequestBody>) {
    let Some(declared) = declared else {
        return (None, None);
    };

    let mut parameters = Vec::new();
    let mut body_schema = None;
    let mut form_properties: Vec<(String, Schema)> = Vec::new();

    for parameter in &declared {
        match parameter.get("in").and_then(Value::as_str) {
            Some("body") => {
                body_schema = parameter
                    .get("schema")
                    .map(|schema| normalize_schema(root, schema, &mut Vec::new()));
            }
            Some("formData") => {
                if let Some(name) = parameter.get("name").and_then(Value::as_str) {
                    form_properties.push((
                        name.to_string(),
                        normalize_schema(root, parameter, &mut Vec::new()),
                    ));
                }
            }
            _ => {
                if let Some(normalized) = normalize_parameter(parameter) {
                    parameters.push(normalized);
                }
            }
        }
    }

    let schema = if form_properties.is_empty() {
        body_schema
    } else {
        Some(Schema {
            schema_type: Some(SchemaType::Object),
            properties: form_properties,
            ..Schema::default()
        })
    };

    let request_body = schema.map(|schema| RequestBody {
        content: consumed_content_types(root, operation)
            .into_iter()
            .map(|content_type| MediaType {
                content_type,
                schema: Some(schema.clone()),
            })
            .collect(),
    });

    (Some(parameters), request_body)
}

fn consumed_content_types(root: &Value, operation: &Value) -> Vec<String> {
    let consumes = operation
        .get("consumes")
        .or_else(|| root.get("consumes"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if consumes.is_empty() {
        vec!["application/json".to_string()]
    } else {
        consumes
    }
}

fn normalize_schema(root: &Value, schema: &Value, visited: &mut Vec<String>) -> Schema {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        if visited.iter().any(|seen| seen == reference) {
            return Schema::default();
        }

        let Some(target) = resolve_pointer(root, reference) else {
            return Schema::default();
        };

        visited.push(reference.to_string());
        let resolved = normalize_schema(root, target, visited);
        visited.pop();

        return resolved;
    }

    Schema {
        schema_type: schema_type_of(schema),
        format: string_field(schema, "format"),
        example: schema.get("example").cloned(),
        properties: schema
            .get("properties")
            .and_then(Value::as_object)
            .map(|properties| {
                properties
                    .iter()
                    .map(|(name, property)| {
                        (name.clone(), normalize_schema(root, property, visited))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        items: schema
            .get("items")
            .map(|items| Box::new(normalize_schema(root, items, visited))),
    }
}

fn schema_type_of(schema: &Value) -> Option<SchemaType> {
    match schema.get("type") {
        Some(Value::String(value)) => SchemaType::parse(value),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .find_map(SchemaType::parse),
        _ => None,
    }
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_string)
}

/// Follows a `$ref` when the value is a reference object, otherwise returns the value unchanged.
fn resolve_reference<'a>(root: &'a Value, value: &'a Value) -> &'a Value {
    value
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| resolve_pointer(root, reference))
        .unwrap_or(value)
}

/// Resolves a local JSON pointer such as `#/components/schemas/Pet`.
fn resolve_pointer<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    let pointer = reference.strip_prefix("#/")?;
    let mut current = root;

    for segment in pointer.split('/') {
        let segment = segment.replace("~1", "/").replace("~0", "~");
        current = match current {
            Value::Object(map) => map.get(&segment)?,
            Value::Array(values) => values.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn normalize_v3(value: &Value) -> Document {
        normalize_value(value, OpenApiSpecificationVersion::OpenApi30)
    }

    #[test]
    fn normalizes_servers_and_operations_in_document_order() {
        let document = normalize_v3(&json!({
            "openapi": "3.0.0",
            "servers": [{ "url": "https://one.example" }, { "url": "https://two.example" }],
            "paths": {
                "/pets": {
                    "post": { "operationId": "createPet" },
                    "get": { "operationId": "listPets" }
                }
            }
        }));

        assert_eq!(
            document.servers,
            vec!["https://one.example", "https://two.example"]
        );
        assert_eq!(document.paths.len(), 1);
        assert_eq!(document.paths[0].path, "/pets");
        assert_eq!(document.paths[0].operations[0].method, "POST");
        assert_eq!(document.paths[0].operations[1].method, "GET");
    }

    #[test]
    fn ignores_path_item_members_that_are_not_http_methods() {
        let document = normalize_v3(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/pets": {
                    "summary": "Pets",
                    "parameters": [{ "name": "row", "in": "path" }],
                    "get": { "operationId": "listPets" }
                }
            }
        }));

        assert_eq!(document.paths[0].operations.len(), 1);
        assert_eq!(document.paths[0].operations[0].method, "GET");
    }

    #[test]
    fn distinguishes_absent_parameters_from_an_empty_list() {
        let document = normalize_v3(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/a": { "get": { "operationId": "a" } },
                "/b": { "get": { "operationId": "b", "parameters": [] } }
            }
        }));

        assert!(document.paths[0].operations[0].parameters.is_none());
        assert_eq!(document.paths[1].operations[0].parameters, Some(Vec::new()));
    }

    #[test]
    fn resolves_referenced_parameters_and_schemas() {
        let document = normalize_v3(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/pets": {
                    "post": {
                        "operationId": "createPet",
                        "parameters": [{ "$ref": "#/components/parameters/Limit" }],
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Pet" }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "parameters": {
                    "Limit": { "name": "limit", "in": "query", "description": "How many" }
                },
                "schemas": {
                    "Pet": {
                        "type": "object",
                        "properties": { "name": { "type": "string" } }
                    }
                }
            }
        }));

        let operation = &document.paths[0].operations[0];
        let parameters = operation.parameters.as_ref().unwrap();

        assert_eq!(parameters[0].name, "limit");
        assert_eq!(parameters[0].location, ParameterLocation::Query);
        assert_eq!(parameters[0].description.as_deref(), Some("How many"));

        let schema = operation
            .request_body
            .as_ref()
            .unwrap()
            .schema_for("application/json")
            .unwrap();

        assert_eq!(schema.schema_type, Some(SchemaType::Object));
        assert_eq!(schema.properties[0].0, "name");
    }

    #[test]
    fn stops_at_self_referencing_schemas() {
        let document = normalize_v3(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/nodes": {
                    "post": {
                        "operationId": "createNode",
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Node" }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "Node": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "child": { "$ref": "#/components/schemas/Node" }
                        }
                    }
                }
            }
        }));

        let schema = document.paths[0].operations[0]
            .request_body
            .as_ref()
            .unwrap()
            .schema_for("application/json")
            .unwrap();

        assert_eq!(schema.properties[1].0, "child");
        assert_eq!(schema.properties[1].1, Schema::default());
    }

    #[test]
    fn reads_the_first_non_null_type_of_a_nullable_schema() {
        let document = normalize_value(
            &json!({
                "openapi": "3.1.0",
                "paths": {
                    "/pets": {
                        "post": {
                            "operationId": "createPet",
                            "requestBody": {
                                "content": {
                                    "application/json": {
                                        "schema": { "type": ["null", "string"] }
                                    }
                                }
                            }
                        }
                    }
                }
            }),
            OpenApiSpecificationVersion::OpenApi31,
        );

        let schema = document.paths[0].operations[0]
            .request_body
            .as_ref()
            .unwrap()
            .schema_for("application/json")
            .unwrap();

        assert_eq!(schema.schema_type, Some(SchemaType::String));
    }

    fn normalize_v2(value: &Value) -> Document {
        normalize_value(value, OpenApiSpecificationVersion::Swagger2)
    }

    #[test]
    fn builds_swagger2_servers_from_schemes_host_and_base_path() {
        let document = normalize_v2(&json!({
            "swagger": "2.0",
            "host": "petstore.swagger.io",
            "basePath": "/v2",
            "schemes": ["https", "http"]
        }));

        assert_eq!(
            document.servers,
            vec![
                "https://petstore.swagger.io/v2",
                "http://petstore.swagger.io/v2"
            ]
        );
    }

    #[test]
    fn defaults_swagger2_scheme_to_http_and_omits_servers_without_a_host() {
        assert_eq!(
            normalize_v2(&json!({ "swagger": "2.0", "host": "api.example", "basePath": "/v1" }))
                .servers,
            vec!["http://api.example/v1"]
        );
        assert!(
            normalize_v2(&json!({ "swagger": "2.0", "basePath": "/v1" }))
                .servers
                .is_empty()
        );
    }

    #[test]
    fn converts_swagger2_body_parameters_into_a_request_body() {
        let document = normalize_v2(&json!({
            "swagger": "2.0",
            "consumes": ["application/json", "application/xml"],
            "paths": {
                "/pet": {
                    "post": {
                        "operationId": "addPet",
                        "parameters": [
                            { "name": "petId", "in": "path", "description": "The id" },
                            { "name": "body", "in": "body", "schema": { "$ref": "#/definitions/Pet" } }
                        ]
                    }
                }
            },
            "definitions": {
                "Pet": { "type": "object", "properties": { "name": { "type": "string" } } }
            }
        }));

        let operation = &document.paths[0].operations[0];
        let parameters = operation.parameters.as_ref().unwrap();
        let request_body = operation.request_body.as_ref().unwrap();

        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "petId");
        assert_eq!(request_body.first_content_type(), Some("application/json"));
        assert_eq!(request_body.content.len(), 2);
        assert_eq!(
            request_body
                .schema_for("application/xml")
                .unwrap()
                .properties[0]
                .0,
            "name"
        );
    }

    #[test]
    fn converts_swagger2_form_parameters_into_a_request_body_schema() {
        let document = normalize_v2(&json!({
            "swagger": "2.0",
            "paths": {
                "/pet/{petId}": {
                    "post": {
                        "operationId": "updatePetWithForm",
                        "consumes": ["application/x-www-form-urlencoded"],
                        "parameters": [
                            { "name": "petId", "in": "path" },
                            { "name": "name", "in": "formData", "type": "string" },
                            { "name": "status", "in": "formData", "type": "string" }
                        ]
                    }
                }
            }
        }));

        let request_body = document.paths[0].operations[0]
            .request_body
            .as_ref()
            .unwrap();
        let schema = request_body
            .schema_for("application/x-www-form-urlencoded")
            .unwrap();

        assert_eq!(
            request_body.first_content_type(),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(schema.schema_type, Some(SchemaType::Object));
        assert_eq!(
            schema
                .properties
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            vec!["name", "status"]
        );
    }

    #[test]
    fn defaults_swagger2_body_content_type_to_json() {
        let document = normalize_v2(&json!({
            "swagger": "2.0",
            "paths": {
                "/pet": {
                    "post": {
                        "operationId": "addPet",
                        "parameters": [
                            { "name": "body", "in": "body", "schema": { "type": "object" } }
                        ]
                    }
                }
            }
        }));

        let operation = &document.paths[0].operations[0];

        assert_eq!(operation.parameters, Some(Vec::new()));
        assert_eq!(
            operation
                .request_body
                .as_ref()
                .unwrap()
                .first_content_type(),
            Some("application/json")
        );
    }

    #[test]
    fn returns_an_empty_document_when_paths_are_missing() {
        let document = normalize_v3(&json!({ "openapi": "3.0.0" }));

        assert!(document.paths.is_empty());
        assert!(document.servers.is_empty());
    }
}
