//! Generates cURL request scripts (PowerShell or Bash) from a normalized
//! OpenAPI [`Document`].

use serde_json::Value;

use crate::http;
use crate::model::{self, Document, Operation, RequestBody};
use crate::operation_name::get_operation_name;
use crate::strings::{capitalize_first_character, convert_kebab_case_to_snake_case};

/// Settings that control script generation.
#[derive(Debug, Clone)]
pub struct GeneratorSettings {
    /// Path to the OpenAPI specification (local file or URL).
    pub open_api_path: String,
    /// Authorization header value applied to every request.
    pub authorization_header: Option<String>,
    /// Default `Content-Type`/`Accept` header (PowerShell scripts).
    pub content_type: String,
    /// Base URL applied in front of the server URL.
    pub base_url: Option<String>,
    /// Generate Bash scripts instead of PowerShell scripts.
    pub generate_bash_scripts: bool,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            open_api_path: String::new(),
            authorization_header: None,
            content_type: "application/json".to_string(),
            base_url: None,
            generate_bash_scripts: false,
        }
    }
}

/// A single generated script file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptFile {
    pub filename: String,
    pub content: String,
}

/// The result of a generation run.
#[derive(Debug, Clone)]
pub struct GeneratorResult {
    pub files: Vec<ScriptFile>,
}

/// Loads the specification referenced by `settings` and generates scripts.
pub fn generate(settings: &GeneratorSettings) -> Result<GeneratorResult, String> {
    let document = model::load(&settings.open_api_path)?;
    Ok(generate_from_document(settings, &document))
}

/// Generates scripts from an already-loaded [`Document`].
pub fn generate_from_document(settings: &GeneratorSettings, document: &Document) -> GeneratorResult {
    let base_url = resolve_base_url(settings, document);

    let mut files = Vec::new();
    for path_item in &document.paths {
        for operation in &path_item.operations {
            let verb = capitalize_first_character(&operation.method);
            let name = get_operation_name(
                &path_item.path,
                &verb,
                operation.operation_id.as_deref(),
            );
            let filename_base = capitalize_first_character(&name);
            let filename = if settings.generate_bash_scripts {
                format!("{filename_base}.sh")
            } else {
                format!("{filename_base}.ps1")
            };

            let body = if settings.generate_bash_scripts {
                generate_request_bash(settings, &base_url, &verb, &path_item.path, operation, &document.root)
            } else {
                generate_request_powershell(settings, &base_url, &verb, &path_item.path, operation, &document.root)
            };

            files.push(ScriptFile {
                filename,
                content: format!("{body}\n"),
            });
        }
    }

    GeneratorResult { files }
}

fn resolve_base_url(settings: &GeneratorSettings, document: &Document) -> String {
    let server = document.servers.first().cloned().unwrap_or_default();
    let mut base_url = format!(
        "{}{}",
        settings.base_url.clone().unwrap_or_default(),
        server
    );

    if !is_absolute_url(&base_url) && http::is_http(&settings.open_api_path) {
        base_url = format!("{}{}", http::authority(&settings.open_api_path), base_url);
    }

    base_url
}

fn is_absolute_url(value: &str) -> bool {
    match value.find("://") {
        Some(index) if index > 0 => value[..index]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.'),
        _ => false,
    }
}

fn nonempty(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|s| !s.trim().is_empty())
}

// ---------------------------------------------------------------------------
// PowerShell
// ---------------------------------------------------------------------------

fn generate_request_powershell(
    settings: &GeneratorSettings,
    base_url: &str,
    verb: &str,
    path: &str,
    operation: &Operation,
    root: &Value,
) -> String {
    let mut code = String::new();
    append_summary_powershell(verb, path, operation, &mut code);
    let parameter_map = append_parameters_powershell(operation, &mut code);

    let mut url = path.replace('{', "$").replace('}', "");
    if !parameter_map.is_empty() {
        url.push('?');
        for (key, snake) in &parameter_map {
            url.push_str(&format!("{key}=${snake}&"));
        }
        url.pop();
    }

    code.push_str(&format!("curl -X {} {}{} `\n", verb.to_uppercase(), base_url, url));
    code.push_str(&format!("  -H 'Accept: {}' `\n", settings.content_type));
    code.push_str(&format!("  -H 'Content-Type: {}' `\n", settings.content_type));

    if let Some(auth) = nonempty(&settings.authorization_header) {
        code.push_str(&format!("  -H 'Authorization: {auth}' `\n"));
    }

    if let Some(body) = &operation.request_body {
        if let Some(content_type) = body
            .content
            .iter()
            .map(|c| c.content_type.clone())
            .find(|c| c.contains(&settings.content_type))
        {
            let json = generate_sample_json(root, body.schema_for(&content_type));
            code.push_str(&format!("  -d '{json}'\n"));
        }
    }

    code
}

fn append_summary_powershell(verb: &str, path: &str, operation: &Operation, code: &mut String) {
    code.push_str("<#\n");
    code.push_str(&format!("  Request: {} {}\n", verb.to_uppercase(), path));

    if let Some(summary) = nonempty(&operation.summary) {
        code.push_str(&format!("  Summary: {summary}\n"));
    }
    if let Some(description) = nonempty(&operation.description) {
        code.push_str(&format!("  Description: {description}\n"));
    }

    code.push_str("#>\n");
}

/// Appends a PowerShell `param(...)` block and returns the original-name to
/// snake-case-name mapping for the path and query parameters.
fn append_parameters_powershell(operation: &Operation, code: &mut String) -> Vec<(String, String)> {
    let parameters: Vec<_> = operation
        .parameters
        .iter()
        .filter(|p| matches!(p.location.as_str(), "path" | "query"))
        .collect();

    if parameters.is_empty() {
        code.push('\n');
        return Vec::new();
    }

    let mut map = Vec::new();
    let mut blocks = Vec::new();
    for parameter in &parameters {
        let snake = convert_kebab_case_to_snake_case(&parameter.name);
        let mut block = String::new();
        if let Some(description) = nonempty(&parameter.description) {
            block.push_str(&format!("   <# {description} #>\n"));
        }
        block.push_str("   [Parameter(Mandatory=$True)]\n");
        block.push_str(&format!("   [String] ${snake}"));
        blocks.push(block);
        map.push((parameter.name.clone(), snake));
    }

    code.push_str("param(\n");
    code.push_str(&blocks.join(",\n\n"));
    code.push_str("\n)\n\n");

    map
}

// ---------------------------------------------------------------------------
// Bash
// ---------------------------------------------------------------------------

fn generate_request_bash(
    settings: &GeneratorSettings,
    base_url: &str,
    verb: &str,
    path: &str,
    operation: &Operation,
    root: &Value,
) -> String {
    let mut code = String::new();
    append_summary_bash(verb, path, operation, &mut code);
    append_parameters_bash(operation, &mut code);

    let route = path.replace('{', "$").replace('}', "");

    let query_params: Vec<String> = operation
        .parameters
        .iter()
        .filter(|p| p.location == "query")
        .map(|p| format!("{}=${{{}}}", p.name, convert_kebab_case_to_snake_case(&p.name)))
        .collect();
    let query_string = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", query_params.join("&"))
    };

    code.push_str(&format!(
        "curl -X {} \"{}{}{}\" \\\n",
        verb.to_uppercase(),
        base_url,
        route,
        query_string
    ));
    code.push_str("  -H \"Accept: application/json\" \\\n");

    let content_type = operation
        .request_body
        .as_ref()
        .and_then(RequestBody::first_content_type)
        .unwrap_or("application/json")
        .to_string();
    code.push_str(&format!("  -H \"Content-Type: {content_type}\" \\\n"));

    if let Some(auth) = nonempty(&settings.authorization_header) {
        code.push_str(&format!("  -H \"Authorization: {auth}\" \\\n"));
    }

    match &operation.request_body {
        Some(body) => append_body_bash(&content_type, body, root, &mut code),
        None => {
            if let Some(stripped) = code.strip_suffix(" \\\n") {
                code = format!("{stripped}\n");
            }
        }
    }

    code
}

fn append_body_bash(content_type: &str, body: &RequestBody, root: &Value, code: &mut String) {
    match content_type {
        "application/x-www-form-urlencoded" | "multipart/form-data" => {
            if let Some(schema) = body.schema_for(content_type) {
                let keys = schema_property_keys(root, schema);
                let form: Vec<String> = keys
                    .iter()
                    .map(|key| format!("-F \"{key}=${{{key}}}\""))
                    .collect();
                for (index, line) in form.iter().enumerate() {
                    if index < form.len() - 1 {
                        code.push_str(&format!("{line} \\\n"));
                    } else {
                        code.push_str(&format!("{line}\n"));
                    }
                }
            }
        }
        "application/octet-stream" => {
            code.push_str("  --data-binary '@filename'\n");
        }
        _ => {
            let json = generate_sample_json(root, body.schema_for(content_type));
            code.push_str(&format!("  -d '{json}'\n"));
        }
    }
}

fn append_summary_bash(verb: &str, path: &str, operation: &Operation, code: &mut String) {
    code.push_str("#\n");
    code.push_str(&format!("# Request: {} {}\n", verb.to_uppercase(), path));

    if let Some(summary) = nonempty(&operation.summary) {
        code.push_str(&format!("# Summary: {summary}\n"));
    }
    if let Some(description) = nonempty(&operation.description) {
        for line in description
            .split(['\r', '\n'])
            .filter(|l| !l.trim().is_empty())
        {
            code.push_str(&format!("# {}\n", line.trim()));
        }
    }

    code.push_str("#\n");
}

fn append_parameters_bash(operation: &Operation, code: &mut String) {
    if operation.parameters.is_empty() {
        code.push('\n');
        return;
    }

    code.push('\n');
    for parameter in &operation.parameters {
        let name = convert_kebab_case_to_snake_case(&parameter.name);
        match nonempty(&parameter.description) {
            Some(description) => code.push_str(&format!("# {description}\n")),
            None => code.push_str(&format!(
                "# {} parameter: {}\n",
                parameter.location.to_lowercase(),
                name
            )),
        }
        code.push_str(&format!("{name}=\"\"\n"));
    }

    // Form data and file upload fields.
    if let Some(body) = &operation.request_body {
        if let Some(content_type) = body.first_content_type() {
            if content_type == "application/x-www-form-urlencoded"
                || content_type == "multipart/form-data"
            {
                if let Some(schema) = body.schema_for(content_type) {
                    for key in schema_property_keys(&Value::Null, schema) {
                        code.push_str(&format!("{key}=\"\"\n"));
                    }
                }
            }
        }
    }

    code.push('\n');
}

// ---------------------------------------------------------------------------
// Sample JSON body generation
// ---------------------------------------------------------------------------

/// Returns the ordered property keys of a (possibly `$ref`'d) object schema.
fn schema_property_keys(root: &Value, schema: &Value) -> Vec<String> {
    let resolved = model::resolve(root, schema);
    resolved
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.keys().cloned().collect())
        .unwrap_or_default()
}

/// Generates a sample JSON document (pretty-printed) for a schema.
pub fn generate_sample_json(root: &Value, schema: Option<&Value>) -> String {
    match schema {
        None => "{}".to_string(),
        Some(schema) => {
            let sample = generate_sample_value(root, schema, 0);
            serde_json::to_string_pretty(&sample).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

fn generate_sample_value(root: &Value, schema: &Value, depth: usize) -> Value {
    if depth > 64 {
        return Value::Null;
    }

    let schema = model::resolve(root, schema);

    if let Some(example) = schema.get("example") {
        return example.clone();
    }

    match schema_type(schema) {
        Some("object") => {
            let mut object = serde_json::Map::new();
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                for (key, property) in properties {
                    object.insert(key.clone(), generate_sample_value(root, property, depth + 1));
                }
            }
            Value::Object(object)
        }
        Some("array") => match schema.get("items") {
            Some(items) => Value::Array(vec![generate_sample_value(root, items, depth + 1)]),
            None => Value::Array(Vec::new()),
        },
        Some("string") => Value::String(sample_string(schema)),
        Some("integer") => serde_json::json!(0),
        Some("number") => serde_json::json!(0.0),
        Some("boolean") => Value::Bool(false),
        _ => Value::String("value".to_string()),
    }
}

fn schema_type(schema: &Value) -> Option<&str> {
    match schema.get("type") {
        Some(Value::String(value)) => Some(value.as_str()),
        // OpenAPI 3.1 allows `type` to be an array of types.
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .find(|value| *value != "null"),
        _ => None,
    }
}

fn sample_string(schema: &Value) -> String {
    match schema.get("format").and_then(Value::as_str) {
        Some("date") => chrono::Local::now().format("%Y-%m-%d").to_string(),
        Some("date-time") => chrono::Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        Some("email") => "user@example.com".to_string(),
        Some("uri") => "https://example.com".to_string(),
        _ => "string".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::from_value;

    fn petstore_v3() -> Document {
        from_value(serde_json::json!({
            "openapi": "3.0.0",
            "servers": [{ "url": "http://petstore.swagger.io/api" }],
            "paths": {
                "/pets/{id}": {
                    "get": {
                        "operationId": "getPet",
                        "summary": "Find a pet",
                        "parameters": [
                            { "name": "id", "in": "path", "description": "the id" }
                        ]
                    }
                },
                "/pets": {
                    "post": {
                        "operationId": "addPet",
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" },
                                            "age": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
    }

    #[test]
    fn is_absolute_url_detects_absolute_urls() {
        assert!(is_absolute_url("http://my-custom-base-url.com"));
        assert!(is_absolute_url("https://example.com/api"));
        assert!(!is_absolute_url("/api"));
        assert!(!is_absolute_url(""));
    }

    #[test]
    fn generates_powershell_scripts_by_default() {
        let settings = GeneratorSettings::default();
        let result = generate_from_document(&settings, &petstore_v3());
        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().all(|f| f.filename.ends_with(".ps1")));
        assert!(result.files.iter().any(|f| f.filename == "GetPet.ps1"));
        assert!(result.files.iter().any(|f| f.filename == "PostAddPet.ps1"));
    }

    #[test]
    fn generates_bash_scripts_when_requested() {
        let settings = GeneratorSettings {
            generate_bash_scripts: true,
            ..Default::default()
        };
        let result = generate_from_document(&settings, &petstore_v3());
        assert!(result.files.iter().all(|f| f.filename.ends_with(".sh")));
    }

    #[test]
    fn includes_base_url_and_parameter_block() {
        let settings = GeneratorSettings::default();
        let result = generate_from_document(&settings, &petstore_v3());
        let get_pet = result
            .files
            .iter()
            .find(|f| f.filename == "GetPet.ps1")
            .unwrap();
        assert!(get_pet.content.contains("http://petstore.swagger.io/api/pets/$id"));
        assert!(get_pet.content.contains("[Parameter(Mandatory=$True)]"));
        assert!(get_pet.content.contains("$id"));
    }

    #[test]
    fn includes_request_body_sample() {
        let settings = GeneratorSettings::default();
        let result = generate_from_document(&settings, &petstore_v3());
        let add_pet = result
            .files
            .iter()
            .find(|f| f.filename == "PostAddPet.ps1")
            .unwrap();
        assert!(add_pet.content.contains("-d '"));
        assert!(add_pet.content.contains("\"name\": \"string\""));
        assert!(add_pet.content.contains("\"age\": 0"));
    }

    #[test]
    fn uses_custom_base_url() {
        let settings = GeneratorSettings {
            base_url: Some("http://my-custom-base-url.com".to_string()),
            ..Default::default()
        };
        let document = from_value(serde_json::json!({
            "openapi": "3.0.0",
            "paths": { "/ping": { "get": { "operationId": "ping" } } }
        }));
        let result = generate_from_document(&settings, &document);
        assert!(result
            .files
            .iter()
            .any(|f| f.content.contains("http://my-custom-base-url.com")));
    }

    #[test]
    fn generate_sample_json_handles_refs() {
        let root = serde_json::json!({
            "components": {
                "schemas": {
                    "Pet": {
                        "type": "object",
                        "properties": { "name": { "type": "string" } }
                    }
                }
            }
        });
        let schema = serde_json::json!({ "$ref": "#/components/schemas/Pet" });
        let json = generate_sample_json(&root, Some(&schema));
        assert!(json.contains("\"name\": \"string\""));
    }
}
