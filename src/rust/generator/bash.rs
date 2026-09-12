//! Rendering of Bash `curl` scripts.

use crate::{
    generator::{NEWLINE, sample::sample_json, text::is_blank},
    model::GeneratorSettings,
    normalized::{Operation, ParameterLocation, Schema},
    string_extensions::convert_kebab_case_to_snake_case,
};

const FORM_URL_ENCODED: &str = "application/x-www-form-urlencoded";
const MULTIPART_FORM_DATA: &str = "multipart/form-data";
const OCTET_STREAM: &str = "application/octet-stream";
const DEFAULT_CONTENT_TYPE: &str = "application/json";

/// Renders the Bash script for a single operation.
pub fn render(
    settings: &GeneratorSettings,
    base_url: &str,
    path: &str,
    operation: &Operation,
) -> String {
    let mut script = String::new();

    append_summary(&mut script, path, operation);
    append_parameters(&mut script, operation);

    let route = route_for(path);
    let query = query_string(operation);
    let verb = &operation.method;

    line(
        &mut script,
        &format!(
            "curl -X {verb} \"{}{route}{query}\" \\",
            escape_double_quoted(base_url)
        ),
    );
    line(
        &mut script,
        &format!("  -H \"Accept: {DEFAULT_CONTENT_TYPE}\" \\"),
    );

    let content_type = request_content_type(operation);
    line(
        &mut script,
        &format!(
            "  -H \"Content-Type: {}\" \\",
            escape_double_quoted(&content_type)
        ),
    );

    if let Some(authorization) = settings
        .authorization_header
        .as_deref()
        .filter(|value| !is_blank(value))
    {
        line(
            &mut script,
            &format!(
                "  -H \"Authorization: {}\" \\",
                escape_double_quoted(authorization)
            ),
        );
    }

    match &operation.request_body {
        Some(body) => append_payload(&mut script, &content_type, body.schema_for(&content_type)),
        None => remove_trailing_continuation(&mut script),
    }

    script.push_str(NEWLINE);
    script
}

fn append_payload(script: &mut String, content_type: &str, schema: Option<&Schema>) {
    match content_type {
        FORM_URL_ENCODED | MULTIPART_FORM_DATA => {
            let Some(schema) = schema else {
                return;
            };

            let last = schema.properties.len().saturating_sub(1);
            for (index, (name, _)) in schema.properties.iter().enumerate() {
                let continuation = if index < last { " \\" } else { "" };
                line(script, &format!("-F \"{name}=${{{name}}}\"{continuation}"));
            }
        }
        OCTET_STREAM => line(script, "  --data-binary '@filename'"),
        _ => line(
            script,
            &format!("  -d '{}'", escape_single_quoted(&sample_json(schema))),
        ),
    }
}

/// Escapes a value for embedding inside a single-quoted Bash string, so a value taken from the
/// specification (a schema `example`, say) cannot end the string early.
fn escape_single_quoted(value: &str) -> String {
    value.replace('\'', r"'\''")
}

/// Collapses embedded line breaks, so a value taken from the specification cannot break out of a
/// single-line `#` comment and run as script content.
fn sanitize_comment(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

/// Escapes a value for embedding inside a double-quoted Bash string, so a value taken from the
/// specification (a server URL or media type, say) cannot end the string or run as a command via
/// `$(...)` or backtick substitution.
fn escape_double_quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`")
}

/// Renders a path template as the route embedded in the double-quoted request URL: a `{name}`
/// placeholder becomes a `$name` variable reference, and the literal text around it is escaped.
fn route_for(path: &str) -> String {
    let mut route = String::new();
    let mut rest = path;

    while let Some(start) = rest.find('{') {
        route.push_str(&escape_double_quoted(&rest[..start]));
        rest = &rest[start + '{'.len_utf8()..];

        match rest.find('}') {
            Some(end) => {
                route.push('$');
                route.push_str(&rest[..end]);
                rest = &rest[end + '}'.len_utf8()..];
            }
            None => {
                // No closing brace: treat the rest of the path as literal text.
                route.push('{');
                break;
            }
        }
    }

    route.push_str(&escape_double_quoted(rest));
    route
}

fn request_content_type(operation: &Operation) -> String {
    operation
        .request_body
        .as_ref()
        .and_then(|body| body.first_content_type())
        .unwrap_or(DEFAULT_CONTENT_TYPE)
        .to_string()
}

fn query_string(operation: &Operation) -> String {
    let Some(parameters) = &operation.parameters else {
        return String::new();
    };

    let query: Vec<String> = parameters
        .iter()
        .filter(|parameter| parameter.location == ParameterLocation::Query)
        .map(|parameter| {
            let variable = convert_kebab_case_to_snake_case(&parameter.name);
            format!("{}=${{{variable}}}", escape_double_quoted(&parameter.name))
        })
        .collect();

    if query.is_empty() {
        String::new()
    } else {
        format!("?{}", query.join("&"))
    }
}

fn append_summary(script: &mut String, path: &str, operation: &Operation) {
    line(script, "#");
    line(
        script,
        &format!(
            "# Request: {} {}",
            sanitize_comment(&operation.method),
            sanitize_comment(path)
        ),
    );

    if let Some(summary) = operation
        .summary
        .as_deref()
        .filter(|value| !is_blank(value))
    {
        line(script, &format!("# Summary: {}", sanitize_comment(summary)));
    }

    if let Some(description) = operation
        .description
        .as_deref()
        .filter(|value| !is_blank(value))
    {
        for text in description
            .split(['\r', '\n'])
            .filter(|text| !text.is_empty())
        {
            line(script, &format!("# {}", text.trim()));
        }
    }

    line(script, "#");
}

fn append_parameters(script: &mut String, operation: &Operation) {
    let Some(declared) = &operation.parameters else {
        return;
    };

    if declared.is_empty() {
        script.push_str(NEWLINE);
        return;
    }

    script.push_str(NEWLINE);

    for parameter in declared {
        let variable = convert_kebab_case_to_snake_case(&parameter.name);
        let comment = match &parameter.description {
            Some(description) => format!("# {}", sanitize_comment(description)),
            None => format!(
                "# {} parameter: {variable}",
                location_name(parameter.location)
            ),
        };

        line(script, &comment);
        line(script, &format!("{variable}=\"\""));
    }

    if let Some(body) = &operation.request_body {
        let content_type = request_content_type(operation);
        if matches!(
            content_type.as_str(),
            FORM_URL_ENCODED | MULTIPART_FORM_DATA
        ) && let Some(schema) = body.schema_for(&content_type)
        {
            for (name, _) in &schema.properties {
                line(script, &format!("{name}=\"\""));
            }
        }
    }

    script.push_str(NEWLINE);
}

fn location_name(location: ParameterLocation) -> &'static str {
    match location {
        ParameterLocation::Path => "path",
        ParameterLocation::Query => "query",
        ParameterLocation::Header => "header",
        ParameterLocation::Cookie => "cookie",
    }
}

/// Drops the trailing line continuation left behind when a request has no payload.
fn remove_trailing_continuation(script: &mut String) {
    let suffix = format!(" \\{NEWLINE}");
    if script.ends_with(&suffix) {
        script.truncate(script.len() - suffix.len());
        script.push_str(NEWLINE);
    }
}

fn line(script: &mut String, value: &str) {
    script.push_str(value);
    script.push_str(NEWLINE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalized::{MediaType, Parameter, RequestBody, SchemaType};

    fn settings() -> GeneratorSettings {
        GeneratorSettings {
            generate_bash_scripts: true,
            ..GeneratorSettings::new("./openapi.json")
        }
    }

    fn operation() -> Operation {
        Operation {
            method: "GET".to_string(),
            operation_id: Some("getPet".to_string()),
            summary: None,
            description: None,
            parameters: None,
            request_body: None,
        }
    }

    fn parameter(name: &str, location: ParameterLocation, description: Option<&str>) -> Parameter {
        Parameter {
            name: name.to_string(),
            location,
            description: description.map(str::to_string),
        }
    }

    fn form_body(content_type: &str, properties: &[&str]) -> RequestBody {
        RequestBody {
            content: vec![MediaType {
                content_type: content_type.to_string(),
                schema: Some(Schema {
                    schema_type: Some(SchemaType::Object),
                    properties: properties
                        .iter()
                        .map(|name| (name.to_string(), Schema::default()))
                        .collect(),
                    ..Schema::default()
                }),
            }],
        }
    }

    fn lines(script: &str) -> Vec<&str> {
        script.split(NEWLINE).collect()
    }

    #[test]
    fn escapes_command_substitution_in_the_base_url() {
        let script = render(
            &settings(),
            "https://example.com/$(rm -rf ~)",
            "/pet",
            &operation(),
        );

        assert!(script.contains(r#"curl -X GET "https://example.com/\$(rm -rf ~)/pet""#));
    }

    #[test]
    fn escapes_command_substitution_in_the_content_type() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                request_body: Some(RequestBody {
                    content: vec![MediaType {
                        content_type: "application/json$(rm -rf ~)".to_string(),
                        schema: None,
                    }],
                }),
                ..operation()
            },
        );

        assert!(script.contains(r#"-H "Content-Type: application/json\$(rm -rf ~)""#));
    }

    #[test]
    fn escapes_command_substitution_in_the_authorization_header() {
        let script = render(
            &GeneratorSettings {
                authorization_header: Some("Bearer $(rm -rf ~)".to_string()),
                generate_bash_scripts: true,
                ..GeneratorSettings::new("./openapi.json")
            },
            "",
            "/pet",
            &operation(),
        );

        assert!(script.contains(r#"-H "Authorization: Bearer \$(rm -rf ~)""#));
    }

    #[test]
    fn escapes_literal_path_text_while_keeping_parameter_references_live() {
        let script = render(
            &settings(),
            "",
            "/pet/$(rm -rf ~)/{petId}",
            &Operation {
                parameters: Some(vec![parameter("petId", ParameterLocation::Path, None)]),
                ..operation()
            },
        );

        assert!(script.contains(r#"curl -X GET "/pet/\$(rm -rf ~)/$petId""#));
    }

    #[test]
    fn escapes_command_substitution_in_a_query_parameter_name() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                parameters: Some(vec![parameter(
                    "id$(rm -rf ~)",
                    ParameterLocation::Query,
                    None,
                )]),
                ..operation()
            },
        );

        // Only the literal key text before `=` is escaped here; the `${...}` variable reference
        // that follows is a separate, pre-existing concern (a raw specification-derived name used
        // as a Bash identifier) that this fix does not attempt to sanitize.
        assert!(script.contains(r"/pet?id\$(rm -rf ~)="));
        assert!(!script.contains("/pet?id$(rm -rf ~)="));
    }

    #[test]
    fn drops_the_line_continuation_when_there_is_no_payload() {
        let script = render(&settings(), "/api/v3", "/pet", &operation());

        assert_eq!(
            lines(&script),
            vec![
                "#",
                "# Request: GET /pet",
                "#",
                "curl -X GET \"/api/v3/pet\" \\",
                "  -H \"Accept: application/json\" \\",
                "  -H \"Content-Type: application/json\"",
                "",
                "",
            ]
        );
    }

    #[test]
    fn splits_multiline_descriptions_into_comment_lines() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                summary: Some("Finds pets".to_string()),
                description: Some("First line\r\n\r\n  Second line  ".to_string()),
                ..operation()
            },
        );

        assert_eq!(
            lines(&script)[1..5],
            [
                "# Request: GET /pet",
                "# Summary: Finds pets",
                "# First line",
                "# Second line",
            ]
        );
    }

    #[test]
    fn declares_every_parameter_and_queries_only_query_parameters() {
        let script = render(
            &settings(),
            "",
            "/pet/{petId}",
            &Operation {
                parameters: Some(vec![
                    parameter("petId", ParameterLocation::Path, Some("The pet id")),
                    parameter("api-key", ParameterLocation::Query, None),
                    parameter("X-Trace", ParameterLocation::Header, None),
                ]),
                ..operation()
            },
        );

        assert_eq!(
            lines(&script),
            vec![
                "#",
                "# Request: GET /pet/{petId}",
                "#",
                "",
                "# The pet id",
                "petid=\"\"",
                "# query parameter: api_key",
                "api_key=\"\"",
                "# header parameter: x_trace",
                "x_trace=\"\"",
                "",
                "curl -X GET \"/pet/$petId?api-key=${api_key}\" \\",
                "  -H \"Accept: application/json\" \\",
                "  -H \"Content-Type: application/json\"",
                "",
                "",
            ]
        );
    }

    #[test]
    fn renders_a_blank_line_for_an_empty_parameter_list() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                parameters: Some(Vec::new()),
                ..operation()
            },
        );

        assert_eq!(lines(&script)[3], "");
        assert_eq!(lines(&script)[4], "curl -X GET \"/pet\" \\");
    }

    #[test]
    fn renders_form_fields_for_url_encoded_bodies() {
        let script = render(
            &settings(),
            "",
            "/pet/{petId}",
            &Operation {
                method: "POST".to_string(),
                parameters: Some(vec![parameter("petId", ParameterLocation::Path, None)]),
                request_body: Some(form_body(FORM_URL_ENCODED, &["name", "status"])),
                ..operation()
            },
        );

        assert_eq!(
            lines(&script),
            vec![
                "#",
                "# Request: POST /pet/{petId}",
                "#",
                "",
                "# path parameter: petid",
                "petid=\"\"",
                "name=\"\"",
                "status=\"\"",
                "",
                "curl -X POST \"/pet/$petId\" \\",
                "  -H \"Accept: application/json\" \\",
                "  -H \"Content-Type: application/x-www-form-urlencoded\" \\",
                "-F \"name=${name}\" \\",
                "-F \"status=${status}\"",
                "",
                "",
            ]
        );
    }

    #[test]
    fn renders_a_binary_upload_for_octet_stream_bodies() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                method: "POST".to_string(),
                request_body: Some(RequestBody {
                    content: vec![MediaType {
                        content_type: OCTET_STREAM.to_string(),
                        schema: None,
                    }],
                }),
                ..operation()
            },
        );

        assert!(script.contains("  -H \"Content-Type: application/octet-stream\" \\"));
        assert!(script.contains("  --data-binary '@filename'"));
    }

    #[test]
    fn renders_a_json_payload_for_other_bodies() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                method: "POST".to_string(),
                request_body: Some(RequestBody {
                    content: vec![MediaType {
                        content_type: "application/json".to_string(),
                        schema: Some(Schema {
                            schema_type: Some(SchemaType::Object),
                            properties: vec![(
                                "name".to_string(),
                                Schema {
                                    schema_type: Some(SchemaType::String),
                                    ..Schema::default()
                                },
                            )],
                            ..Schema::default()
                        }),
                    }],
                }),
                ..operation()
            },
        );

        assert!(script.contains(&format!(
            "  -d '{{{NEWLINE}  \"name\": \"string\"{NEWLINE}}}'"
        )));
    }

    #[test]
    fn escapes_single_quotes_in_the_json_payload() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                method: "POST".to_string(),
                request_body: Some(RequestBody {
                    content: vec![MediaType {
                        content_type: "application/json".to_string(),
                        schema: Some(Schema {
                            example: Some(serde_json::json!("O'Reilly")),
                            ..Schema::default()
                        }),
                    }],
                }),
                ..operation()
            },
        );

        assert!(script.contains("-d '\"O'\\''Reilly\"'"));
    }

    #[test]
    fn sanitizes_embedded_line_breaks_in_comments() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                summary: Some("first line\nsecond line".to_string()),
                parameters: Some(vec![parameter(
                    "id",
                    ParameterLocation::Query,
                    Some("first\nsecond"),
                )]),
                ..operation()
            },
        );

        assert!(script.contains("# Summary: first line second line"));
        assert!(script.contains("# first second"));
        assert!(!script.lines().any(|line| line == "second line"));
        assert!(!script.lines().any(|line| line == "second"));
    }

    #[test]
    fn renders_the_authorization_header_when_configured() {
        let script = render(
            &GeneratorSettings {
                authorization_header: Some("Bearer token".to_string()),
                ..settings()
            },
            "",
            "/pet",
            &operation(),
        );

        assert!(script.contains("  -H \"Authorization: Bearer token\""));
    }
}
