use std::collections::HashMap;

use openapiv3::{OpenAPI, Parameter, SchemaKind};

use crate::models::{GeneratorResult, GeneratorSettings, ScriptFile};
use crate::names;
use crate::openapi;

pub async fn generate(settings: &GeneratorSettings) -> Result<GeneratorResult, String> {
    let doc = openapi::load_from_path(&settings.openapi_path).await.map_err(|e| format!("Failed to load OpenAPI spec: {}", e))?;

    let name_gen = names::OperationNameGenerator::new();

    let mut files = Vec::new();

    let base_url = determine_base_url(&settings, &doc);

    for (path, path_item_or_ref) in &doc.paths.paths {
        let path_item = match path_item_or_ref {
            openapiv3::ReferenceOr::Reference { .. } => continue,
            openapiv3::ReferenceOr::Item(item) => item,
        };

        let methods = [
            ("get", path_item.get.as_ref()),
            ("put", path_item.put.as_ref()),
            ("post", path_item.post.as_ref()),
            ("delete", path_item.delete.as_ref()),
            ("options", path_item.options.as_ref()),
            ("head", path_item.head.as_ref()),
            ("patch", path_item.patch.as_ref()),
            ("trace", path_item.trace.as_ref()),
        ];

        for (method, operation_or_ref) in methods {
            let op = match operation_or_ref {
                Some(op) => op,
                None => continue,
            };

            let verb = method.to_string();
            let name = name_gen.get_operation_name(path, &verb, op);

            let filename = if settings.generate_bash {
                format!("{}.sh", names::capitalize_first(&name))
            } else {
                format!("{}.ps1", names::capitalize_first(&name))
            };

            let content = if settings.generate_bash {
                generate_bash_script(&verb, path, op, &settings, &base_url)
            } else {
                generate_powershell_script(&verb, path, op, &settings, &base_url)
            };

            files.push(ScriptFile::new(filename, content));
        }
    }

    Ok(GeneratorResult::new(files))
}

fn determine_base_url(settings: &GeneratorSettings, doc: &OpenAPI) -> String {
    if let Some(ref base) = settings.base_url {
        return base.clone();
    }

    if let Some(first) = doc.servers.first() {
        return first.url.clone();
    }

    String::new()
}

fn extract_parameter_location(param: &Parameter) -> &str {
    match param {
        Parameter::Path { .. } => "path",
        Parameter::Query { .. } => "query",
        Parameter::Header { .. } => "header",
        Parameter::Cookie { .. } => "cookie",
    }
}

fn extract_parameter_data(param: &Parameter) -> &openapiv3::ParameterData {
    match param {
        Parameter::Path { parameter_data, .. } => parameter_data,
        Parameter::Query { parameter_data, .. } => parameter_data,
        Parameter::Header { parameter_data, .. } => parameter_data,
        Parameter::Cookie { parameter_data, .. } => parameter_data,
    }
}

fn generate_powershell_script(
    verb: &str,
    path: &str,
    operation: &openapiv3::Operation,
    settings: &GeneratorSettings,
    base_url: &str,
) -> String {
    let mut code = String::new();

    names::append_summary(verb, path, operation, &mut code, true);

    let parameter_map = append_parameters_powershell(verb, path, operation, &mut code);

    let mut url = path.replace("{", "$").replace("}", "");

    if !parameter_map.is_empty() {
        url.push('?');
        for (name, snake_name) in &parameter_map {
            url.push_str(&format!("{}=${}&", name, snake_name));
        }
        url.pop();
    }

    code.push_str(&format!("curl -X {} {}{} `", verb.to_uppercase(), base_url, url));
    code.push('\n');
    code.push_str(&format!("  -H 'Accept: {}' `", settings.content_type));
    code.push('\n');
    code.push_str(&format!("  -H 'Content-Type: {}' `", settings.content_type));
    code.push('\n');

    if let Some(ref auth) = settings.authorization_header {
        code.push_str(&format!("  -H 'Authorization: {}' `", auth));
        code.push('\n');
    }

    if let Some(openapiv3::ReferenceOr::Item(ref request_body)) = operation.request_body {
        for (ct, media_type) in &request_body.content {
            if ct.contains(&settings.content_type) {
                if let Some(openapiv3::ReferenceOr::Item(ref schema)) = &media_type.schema {
                    let json = generate_sample_json_from_schema(schema);
                    code.push_str(&format!("  -d '{}'", json));
                    code.push('\n');
                }
                break;
            }
        }
    }

    code
}

fn generate_bash_script(
    verb: &str,
    path: &str,
    operation: &openapiv3::Operation,
    settings: &GeneratorSettings,
    base_url: &str,
) -> String {
    let mut code = String::new();

    names::append_summary(verb, path, operation, &mut code, false);
    append_parameters_bash(verb, path, operation, &mut code);

    let route = path.replace("{", "$").replace("}", "");

    let query_params: Vec<String> = operation
        .parameters
        .iter()
        .filter_map(|param_or_ref| {
            if let openapiv3::ReferenceOr::Item(param) = param_or_ref {
                if matches!(param, Parameter::Query { .. }) {
                    let data = extract_parameter_data(param);
                    Some(format!(
                        "{}=${}",
                        data.name,
                        data.name.replace('-', "_")
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let query_string = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", query_params.join("&"))
    };

    code.push_str(&format!("curl -X {} \"{}{}{}\" \\", verb.to_uppercase(), base_url, route, query_string));
    code.push('\n');
    code.push_str("  -H \"Accept: application/json\" \\");
    code.push('\n');

    let content_type = operation
        .request_body
        .as_ref()
        .and_then(|rb| {
            if let openapiv3::ReferenceOr::Item(ref req_body) = rb {
                req_body.content.keys().next().cloned()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "application/json".to_string());

    code.push_str(&format!("  -H \"Content-Type: {}\" \\", content_type));
    code.push('\n');

    if let Some(ref auth) = settings.authorization_header {
        code.push_str(&format!("  -H \"Authorization: {}\" \\", auth));
        code.push('\n');
    }

    if let Some(openapiv3::ReferenceOr::Item(ref request_body)) = operation.request_body {
        if let Some(media_type) = request_body.content.get(&content_type) {
            if content_type == "application/x-www-form-urlencoded"
                || content_type == "multipart/form-data"
            {
                // Handle form data
            } else if content_type == "application/octet-stream" {
                code.push_str("  --data-binary '@filename'\n");
            } else {
                if let Some(ref schema) = media_type.schema {
                    if let openapiv3::ReferenceOr::Item(ref s) = schema {
                        let json = generate_sample_json_from_schema(s);
                        code.push_str(&format!("  -d '{}'", json));
                        code.push('\n');
                    }
                }
            }
        }
    } else {
        if code.ends_with(" \\\n") {
            code.truncate(code.len() - 4);
            code.push('\n');
        }
    }

    code
}

fn append_parameters_powershell(
    _verb: &str,
    _path: &str,
    operation: &openapiv3::Operation,
    code: &mut String,
) -> HashMap<String, String> {
    let parameters: Vec<&Parameter> = operation
        .parameters
        .iter()
        .filter_map(|param_or_ref| {
            if let openapiv3::ReferenceOr::Item(param) = param_or_ref {
                match param {
                    Parameter::Path { .. } | Parameter::Query { .. } => Some(param),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect();

    if parameters.is_empty() {
        code.push('\n');
        return HashMap::new();
    }

    code.push_str("param(\n");

    let mut param_map = HashMap::new();
    for parameter in &parameters {
        let data = extract_parameter_data(parameter);
        let name = data.name.replace('-', "_").to_lowercase();
        if let Some(ref description) = data.description {
            code.push_str(&format!("                          <# {} #>\n", description));
        }
        code.push_str(&format!("                          [Parameter(Mandatory=$True)]\n"));
        code.push_str(&format!("                          [String] ${},\n", name));
        code.push('\n');
        param_map.insert(data.name.clone(), name);
    }

    let lines: Vec<&str> = code.lines().collect();
    if let Some(last_line) = lines.last() {
        if last_line.trim().ends_with(',') {
            let new_code: String = lines[..lines.len() - 1].join("\n");
            code.clear();
            code.push_str(&new_code);
        }
    }

    code.push_str(")\n\n");

    param_map
}

fn append_parameters_bash(
    _verb: &str,
    _path: &str,
    operation: &openapiv3::Operation,
    code: &mut String,
) {
    let parameters: Vec<&Parameter> = operation
        .parameters
        .iter()
        .filter_map(|param_or_ref| {
            if let openapiv3::ReferenceOr::Item(param) = param_or_ref {
                match param {
                    Parameter::Path { .. }
                | Parameter::Query { .. }
                | Parameter::Header { .. }
                | Parameter::Cookie { .. } => Some(param),
                }
            } else {
                None
            }
        })
        .collect();

    if parameters.is_empty() {
        code.push('\n');
        return;
    }

    code.push('\n');

    for parameter in &parameters {
        let data = extract_parameter_data(parameter);
        let name = data.name.replace('-', "_").to_lowercase();
        if let Some(ref description) = data.description {
            code.push_str(&format!("# {}\n", description));
        } else {
            code.push_str(&format!("# {} parameter: {}\n", extract_parameter_location(parameter), name));
        }
        code.push_str(&format!("{}=\"\"\n", name));
    }

    code.push('\n');
}

fn generate_sample_json_from_schema(schema: &openapiv3::Schema) -> String {
    if let Some(ref example) = schema.schema_data.example {
        return serde_json::to_string_pretty(example).unwrap_or_else(|_| "\"value\"".to_string());
    }

    match &schema.schema_kind {
        SchemaKind::Type(t) => match t {
            openapiv3::Type::Object(_) => "{}".to_string(),
            openapiv3::Type::Array(_) => "[]".to_string(),
            openapiv3::Type::String(_) => "\"string\"".to_string(),
            openapiv3::Type::Integer(_) => "0".to_string(),
            openapiv3::Type::Number(_) => "0.0".to_string(),
            openapiv3::Type::Boolean(_) => "false".to_string(),
            _ => "\"value\"".to_string(),
        },
        SchemaKind::OneOf { one_of, .. } => {
            if let Some(first) = one_of.first() {
                if let openapiv3::ReferenceOr::Item(ref s) = first {
                    generate_sample_json_from_schema(s)
                } else {
                    "{}".to_string()
                }
            } else {
                "{}".to_string()
            }
        }
        SchemaKind::AllOf { all_of, .. } => {
            if let Some(first) = all_of.first() {
                if let openapiv3::ReferenceOr::Item(ref s) = first {
                    generate_sample_json_from_schema(s)
                } else {
                    "{}".to_string()
                }
            } else {
                "{}".to_string()
            }
        }
        SchemaKind::AnyOf { any_of, .. } => {
            if let Some(first) = any_of.first() {
                if let openapiv3::ReferenceOr::Item(ref s) = first {
                    generate_sample_json_from_schema(s)
                } else {
                    "{}".to_string()
                }
            } else {
                "{}".to_string()
            }
        }
        SchemaKind::Not { not } => {
            if let openapiv3::ReferenceOr::Item(ref s) = **not {
                generate_sample_json_from_schema(&s)
            } else {
                "{}".to_string()
            }
        }
        SchemaKind::Any(_) => "{}".to_string(),
    }
}
