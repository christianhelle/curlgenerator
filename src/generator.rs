use crate::script::ScriptFile;
use anyhow::Result;
use openapiv3::{
    OpenAPI, Operation, Parameter, ParameterSchemaOrContent, ReferenceOr, Schema, SchemaKind, Type,
};

#[derive(Debug, Clone)]
pub struct GeneratorSettings {
    pub authorization_header: Option<String>,
    pub content_type: String,
    pub base_url: Option<String>,
    pub generate_bash_scripts: bool,
}

#[derive(Debug)]
pub struct GeneratorResult {
    pub files: Vec<ScriptFile>,
}

pub fn generate(document: &OpenAPI, settings: &GeneratorSettings) -> Result<GeneratorResult> {
    let mut files = Vec::new();

    let base_url = determine_base_url(document, settings);

    for (path, item) in &document.paths.paths {
        if let ReferenceOr::Item(path_item) = item {
            for (method, operation) in path_item.iter() {
                let verb = method.to_string().to_uppercase();
                let name = generate_operation_name(&verb, path, operation);

                let filename = if settings.generate_bash_scripts {
                    format!("{}.sh", capitalize_first(&name))
                } else {
                    format!("{}.ps1", capitalize_first(&name))
                };

                let content = if settings.generate_bash_scripts {
                    generate_bash_script(&verb, path, operation, &base_url, settings, document)
                } else {
                    generate_powershell_script(
                        &verb, path, operation, &base_url, settings, document,
                    )
                };

                files.push(ScriptFile::new(filename, content));
            }
        }
    }

    Ok(GeneratorResult { files })
}

fn determine_base_url(document: &OpenAPI, settings: &GeneratorSettings) -> String {
    if let Some(base_url) = &settings.base_url {
        return base_url.clone();
    }

    if !document.servers.is_empty() {
        if let Some(server) = document.servers.first() {
            return server.url.clone();
        }
    }

    "http://localhost".to_string()
}

fn generate_operation_name(verb: &str, path: &str, operation: &Operation) -> String {
    if let Some(operation_id) = &operation.operation_id {
        let cleaned = operation_id
            .replace("-", "_")
            .replace("/", "_")
            .replace(" ", "_");
        return format!("{}{}", verb, capitalize_first(&cleaned));
    }

    let cleaned = path
        .replace("/", "_")
        .replace("{", "")
        .replace("}", "")
        .replace("-", "_");

    format!("{}{}", verb, to_pascal_case(&cleaned))
}

fn generate_powershell_script(
    verb: &str,
    path: &str,
    operation: &Operation,
    base_url: &str,
    settings: &GeneratorSettings,
    document: &OpenAPI,
) -> String {
    let mut script = String::new();

    // Add comment header
    script.push_str("<#\n");
    script.push_str(&format!("  Request: {} {}\n", verb, path));
    if let Some(summary) = &operation.summary {
        script.push_str(&format!("  Summary: {}\n", summary));
    }
    if let Some(description) = &operation.description {
        script.push_str(&format!("  Description: {}\n", description));
    }
    script.push_str("#>\n\n");

    // Add parameters
    let path_params = extract_path_parameters(operation);
    let query_params = extract_query_parameters(operation);

    if !path_params.is_empty() || !query_params.is_empty() {
        script.push_str("param(\n");

        for param in path_params.iter().chain(query_params.iter()) {
            if let Some(desc) = &param.description {
                script.push_str(&format!("   <# {} #>\n", desc));
            }
            script.push_str("   [Parameter(Mandatory=$True)]\n");
            script.push_str(&format!("   [String] ${}\n", to_snake_case(&param.name)));
            script.push_str(",\n");
        }

        // Remove trailing comma
        if script.ends_with(",\n") {
            script.truncate(script.len() - 2);
            script.push('\n');
        }

        script.push_str(")\n\n");
    }

    // Build URL with path parameters
    let mut url = format!("{}{}", base_url, path);
    for param in &path_params {
        url = url.replace(
            &format!("{{{}}}", param.name),
            &format!("${}", to_snake_case(&param.name)),
        );
    }

    // Add query parameters
    if !query_params.is_empty() {
        url.push('?');
        for param in &query_params {
            url.push_str(&format!("{}=${}&", param.name, to_snake_case(&param.name)));
        }
        url.pop(); // Remove trailing &
    }

    // Generate curl command
    script.push_str(&format!("curl -X {} {} `\n", verb, url));
    script.push_str(&format!("  -H 'Accept: {}' `\n", settings.content_type));
    script.push_str(&format!(
        "  -H 'Content-Type: {}' `\n",
        settings.content_type
    ));

    if let Some(auth) = &settings.authorization_header {
        script.push_str(&format!("  -H 'Authorization: {}' `\n", auth));
    }

    // Add request body if present
    if let Some(ReferenceOr::Item(body)) = &operation.request_body {
        if let Some(content) = body.content.get(&settings.content_type) {
            if let Some(schema) = &content.schema {
                let sample_json = match schema {
                    ReferenceOr::Item(s) => generate_sample_from_schema(s, document),
                    ReferenceOr::Reference { reference } => {
                        resolve_schema_reference(document, reference)
                            .map(|s| generate_sample_from_schema(s, document))
                            .unwrap_or_else(|| "{}".to_string())
                    }
                };
                script.push_str(&format!("  -d '{}'\n", sample_json));
            }
        }
    }

    script
}

fn generate_bash_script(
    verb: &str,
    path: &str,
    operation: &Operation,
    base_url: &str,
    settings: &GeneratorSettings,
    document: &OpenAPI,
) -> String {
    let mut script = String::new();

    // Add comment header
    script.push_str("#\n");
    script.push_str(&format!("# Request: {} {}\n", verb, path));
    if let Some(summary) = &operation.summary {
        script.push_str(&format!("# Summary: {}\n", summary));
    }
    if let Some(description) = &operation.description {
        script.push_str(&format!("# Description: {}\n", description));
    }
    script.push_str("#\n\n");

    // Add parameter declarations
    let path_params = extract_path_parameters(operation);
    let query_params = extract_query_parameters(operation);

    for param in path_params.iter().chain(query_params.iter()) {
        if let Some(desc) = &param.description {
            script.push_str(&format!("# {}\n", desc));
        }
        script.push_str(&format!("{}=\"\"\n", to_snake_case(&param.name)));
    }

    if !path_params.is_empty() || !query_params.is_empty() {
        script.push('\n');
    }

    // Build URL with path parameters
    let mut url = format!("{}{}", base_url, path);
    for param in &path_params {
        url = url.replace(
            &format!("{{{}}}", param.name),
            &format!("${{{}}}", to_snake_case(&param.name)),
        );
    }

    // Add query parameters
    if !query_params.is_empty() {
        url.push('?');
        for param in &query_params {
            url.push_str(&format!(
                "{}=${{{}}}&",
                param.name,
                to_snake_case(&param.name)
            ));
        }
        url.pop(); // Remove trailing &
    }

    // Generate curl command
    script.push_str(&format!("curl -X {} \"{}\" \\\n", verb, url));
    script.push_str("  -H \"Accept: application/json\" \\\n");
    script.push_str("  -H \"Content-Type: application/json\" \\");

    if let Some(auth) = &settings.authorization_header {
        script.push('\n');
        script.push_str(&format!("  -H \"Authorization: {}\" \\", auth));
    }

    // Add request body if present
    if let Some(ReferenceOr::Item(body)) = &operation.request_body {
        if let Some(content) = body.content.get(&settings.content_type) {
            if let Some(schema) = &content.schema {
                let sample_json = match schema {
                    ReferenceOr::Item(s) => generate_sample_from_schema(s, document),
                    ReferenceOr::Reference { reference } => {
                        resolve_schema_reference(document, reference)
                            .map(|s| generate_sample_from_schema(s, document))
                            .unwrap_or_else(|| "{}".to_string())
                    }
                };
                script.push('\n');
                script.push_str(&format!("  -d '{}'", sample_json));
            }
        }
    }

    script.push('\n');
    script
}

#[derive(Debug, Clone)]
struct ParamInfo {
    name: String,
    description: Option<String>,
}

fn extract_path_parameters(operation: &Operation) -> Vec<ParamInfo> {
    let mut params = Vec::new();

    for param_ref in &operation.parameters {
        if let ReferenceOr::Item(param) = param_ref {
            if let ParameterSchemaOrContent::Schema { .. } = &param.parameter_data_ref().format {
                if let Parameter::Path { parameter_data, .. } = param {
                    params.push(ParamInfo {
                        name: parameter_data.name.clone(),
                        description: parameter_data.description.clone(),
                    });
                }
            }
        }
    }

    params
}

fn extract_query_parameters(operation: &Operation) -> Vec<ParamInfo> {
    let mut params = Vec::new();

    for param_ref in &operation.parameters {
        if let ReferenceOr::Item(Parameter::Query { parameter_data, .. }) = param_ref {
            params.push(ParamInfo {
                name: parameter_data.name.clone(),
                description: parameter_data.description.clone(),
            });
        }
    }

    params
}

fn resolve_schema_reference<'a>(document: &'a OpenAPI, reference: &str) -> Option<&'a Schema> {
    // References are in the format "#/components/schemas/SchemaName"
    if let Some(schema_name) = reference.strip_prefix("#/components/schemas/") {
        if let Some(components) = &document.components {
            if let Some(ReferenceOr::Item(schema)) = components.schemas.get(schema_name) {
                return Some(schema);
            }
        }
    }
    None
}

fn generate_sample_from_schema(schema: &Schema, document: &OpenAPI) -> String {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            let mut json_obj = serde_json::Map::new();

            for (key, prop_ref) in &obj.properties {
                let value = match prop_ref {
                    ReferenceOr::Item(prop_schema) => schema_to_json_value(prop_schema, document),
                    ReferenceOr::Reference { reference } => {
                        resolve_schema_reference(document, reference)
                            .map(|s| schema_to_json_value(s, document))
                            .unwrap_or_else(|| serde_json::Value::String("ref".to_string()))
                    }
                };
                json_obj.insert(key.clone(), value);
            }

            serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
        }
        SchemaKind::Type(Type::Array(arr)) => {
            if let Some(items) = &arr.items {
                let item_value = match items {
                    ReferenceOr::Item(item_schema) => schema_to_json_value(item_schema, document),
                    ReferenceOr::Reference { reference } => {
                        resolve_schema_reference(document, reference)
                            .map(|s| schema_to_json_value(s, document))
                            .unwrap_or_else(|| serde_json::Value::String("ref".to_string()))
                    }
                };
                serde_json::to_string_pretty(&vec![item_value]).unwrap_or_else(|_| "[]".to_string())
            } else {
                "[]".to_string()
            }
        }
        _ => schema_to_json_value(schema, document).to_string(),
    }
}

fn schema_to_json_value(schema: &Schema, document: &OpenAPI) -> serde_json::Value {
    match &schema.schema_kind {
        SchemaKind::Type(Type::String(_)) => serde_json::Value::String("string".to_string()),
        SchemaKind::Type(Type::Integer(_)) => serde_json::Value::Number(0.into()),
        SchemaKind::Type(Type::Number(_)) => {
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap())
        }
        SchemaKind::Type(Type::Boolean(_)) => serde_json::Value::Bool(false),
        SchemaKind::Type(Type::Object(obj)) => {
            let mut json_obj = serde_json::Map::new();
            for (key, prop_ref) in &obj.properties {
                let value = match prop_ref {
                    ReferenceOr::Item(prop_schema) => schema_to_json_value(prop_schema, document),
                    ReferenceOr::Reference { reference } => {
                        resolve_schema_reference(document, reference)
                            .map(|s| schema_to_json_value(s, document))
                            .unwrap_or_else(|| serde_json::Value::String("ref".to_string()))
                    }
                };
                json_obj.insert(key.clone(), value);
            }
            serde_json::Value::Object(json_obj)
        }
        SchemaKind::Type(Type::Array(arr)) => {
            if let Some(items) = &arr.items {
                let item_value = match items {
                    ReferenceOr::Item(item_schema) => schema_to_json_value(item_schema, document),
                    ReferenceOr::Reference { reference } => {
                        resolve_schema_reference(document, reference)
                            .map(|s| schema_to_json_value(s, document))
                            .unwrap_or_else(|| serde_json::Value::String("ref".to_string()))
                    }
                };
                serde_json::Value::Array(vec![item_value])
            } else {
                serde_json::Value::Array(vec![])
            }
        }
        _ => serde_json::Value::Null,
    }
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|s| !s.is_empty())
        .map(capitalize_first)
        .collect()
}

fn to_snake_case(s: &str) -> String {
    s.replace("-", "_").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("HELLO"), "HELLO");
        assert_eq!(capitalize_first("h"), "H");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("get_user_by_id"), "GetUserById");
        assert_eq!(to_pascal_case("_leading_underscore"), "LeadingUnderscore");
        assert_eq!(to_pascal_case("trailing_"), "Trailing");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Hello-World"), "hello_world");
        assert_eq!(to_snake_case("user-id"), "user_id");
        assert_eq!(to_snake_case("UPPER-CASE"), "upper_case");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_generator_settings_clone() {
        let settings = GeneratorSettings {
            authorization_header: Some("Bearer token".to_string()),
            content_type: "application/json".to_string(),
            base_url: Some("http://api.test".to_string()),
            generate_bash_scripts: true,
        };
        let cloned = settings.clone();
        assert_eq!(settings.content_type, cloned.content_type);
        assert_eq!(settings.generate_bash_scripts, cloned.generate_bash_scripts);
    }

    #[test]
    fn test_determine_base_url_with_settings() {
        let document = create_minimal_openapi();
        let settings = GeneratorSettings {
            authorization_header: None,
            content_type: "application/json".to_string(),
            base_url: Some("http://custom.com".to_string()),
            generate_bash_scripts: false,
        };
        assert_eq!(
            determine_base_url(&document, &settings),
            "http://custom.com"
        );
    }

    #[test]
    fn test_determine_base_url_from_document() {
        let mut document = create_minimal_openapi();
        document.servers = vec![openapiv3::Server {
            url: "http://api.example.com".to_string(),
            description: None,
            variables: Default::default(),
            extensions: Default::default(),
        }];
        let settings = GeneratorSettings {
            authorization_header: None,
            content_type: "application/json".to_string(),
            base_url: None,
            generate_bash_scripts: false,
        };
        assert_eq!(
            determine_base_url(&document, &settings),
            "http://api.example.com"
        );
    }

    #[test]
    fn test_determine_base_url_default() {
        let document = create_minimal_openapi();
        let settings = GeneratorSettings {
            authorization_header: None,
            content_type: "application/json".to_string(),
            base_url: None,
            generate_bash_scripts: false,
        };
        assert_eq!(determine_base_url(&document, &settings), "http://localhost");
    }

    #[test]
    fn test_generate_operation_name_with_operation_id() {
        let operation = Operation {
            operation_id: Some("get-user-by-id".to_string()),
            ..Default::default()
        };
        let name = generate_operation_name("GET", "/users/{id}", &operation);
        assert_eq!(name, "GETGet_user_by_id");
    }

    #[test]
    fn test_generate_operation_name_from_path() {
        let operation = Operation::default();
        let name = generate_operation_name("POST", "/users", &operation);
        assert_eq!(name, "POSTUsers");
    }

    #[test]
    fn test_generate_operation_name_with_path_params() {
        let operation = Operation::default();
        let name = generate_operation_name("DELETE", "/users/{id}/posts/{postId}", &operation);
        assert_eq!(name, "DELETEUsersIdPostsPostId");
    }

    #[test]
    fn test_extract_path_parameters() {
        let param_data = openapiv3::ParameterData {
            name: "userId".to_string(),
            description: Some("User ID".to_string()),
            required: true,
            deprecated: None,
            format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            })),
            example: None,
            examples: Default::default(),
            explode: None,
            extensions: Default::default(),
        };

        let operation = Operation {
            parameters: vec![ReferenceOr::Item(Parameter::Path {
                parameter_data: param_data.clone(),
                style: Default::default(),
            })],
            ..Default::default()
        };

        let params = extract_path_parameters(&operation);
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "userId");
    }

    #[test]
    fn test_extract_query_parameters() {
        let param_data = openapiv3::ParameterData {
            name: "limit".to_string(),
            description: Some("Limit results".to_string()),
            required: false,
            deprecated: None,
            format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Integer(Default::default())),
            })),
            example: None,
            examples: Default::default(),
            explode: None,
            extensions: Default::default(),
        };

        let operation = Operation {
            parameters: vec![ReferenceOr::Item(Parameter::Query {
                parameter_data: param_data.clone(),
                allow_reserved: false,
                style: Default::default(),
                allow_empty_value: None,
            })],
            ..Default::default()
        };

        let params = extract_query_parameters(&operation);
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "limit");
    }

    #[test]
    fn test_schema_to_json_value_string() {
        let document = create_minimal_openapi();
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::String(Default::default())),
        };
        let value = schema_to_json_value(&schema, &document);
        assert_eq!(value, serde_json::Value::String("string".to_string()));
    }

    #[test]
    fn test_schema_to_json_value_integer() {
        let document = create_minimal_openapi();
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Integer(Default::default())),
        };
        let value = schema_to_json_value(&schema, &document);
        assert_eq!(value, serde_json::Value::Number(0.into()));
    }

    #[test]
    fn test_schema_to_json_value_boolean() {
        let document = create_minimal_openapi();
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Boolean(Default::default())),
        };
        let value = schema_to_json_value(&schema, &document);
        assert_eq!(value, serde_json::Value::Bool(false));
    }

    #[test]
    fn test_schema_to_json_value_array() {
        let document = create_minimal_openapi();
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Array(openapiv3::ArrayType {
                items: Some(ReferenceOr::boxed_item(Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::String(Default::default())),
                })),
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        };
        let value = schema_to_json_value(&schema, &document);
        assert!(value.is_array());
    }

    #[test]
    fn test_generate_with_simple_operation() {
        let mut document = create_minimal_openapi();
        document.paths.paths.insert(
            "/test".to_string(),
            ReferenceOr::Item(openapiv3::PathItem {
                get: Some(Operation {
                    operation_id: Some("getTest".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );

        let settings = GeneratorSettings {
            authorization_header: None,
            content_type: "application/json".to_string(),
            base_url: Some("http://api.test".to_string()),
            generate_bash_scripts: false,
        };

        let result = generate(&document, &settings).unwrap();
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].filename.ends_with(".ps1"));
    }

    #[test]
    fn test_generate_bash_script() {
        let mut document = create_minimal_openapi();
        document.paths.paths.insert(
            "/users".to_string(),
            ReferenceOr::Item(openapiv3::PathItem {
                post: Some(Operation::default()),
                ..Default::default()
            }),
        );

        let settings = GeneratorSettings {
            authorization_header: Some("Bearer test".to_string()),
            content_type: "application/json".to_string(),
            base_url: None,
            generate_bash_scripts: true,
        };

        let result = generate(&document, &settings).unwrap();
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].filename.ends_with(".sh"));
        assert!(result.files[0].content.contains("curl"));
    }

    #[test]
    fn test_generate_with_authorization() {
        let mut document = create_minimal_openapi();
        document.paths.paths.insert(
            "/secure".to_string(),
            ReferenceOr::Item(openapiv3::PathItem {
                get: Some(Operation::default()),
                ..Default::default()
            }),
        );

        let settings = GeneratorSettings {
            authorization_header: Some("Bearer secret_token".to_string()),
            content_type: "application/json".to_string(),
            base_url: None,
            generate_bash_scripts: false,
        };

        let result = generate(&document, &settings).unwrap();
        assert!(result.files[0].content.contains("secret_token"));
    }

    #[test]
    fn test_generate_operation_name_with_slashes() {
        let mut operation = Operation::default();
        operation.operation_id = Some("post-/events/v3/send".to_string());
        
        let name = generate_operation_name("POST", "/test", &operation);
        
        // Should replace slashes with underscores to avoid path issues
        assert_eq!(name, "POSTPost__events_v3_send");
        assert!(!name.contains("/"));
    }

    fn create_minimal_openapi() -> OpenAPI {
        OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            servers: vec![],
            paths: openapiv3::Paths::default(),
            components: None,
            security: None,
            tags: vec![],
            external_docs: None,
            extensions: Default::default(),
        }
    }
}
