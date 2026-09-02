//! Rendering of PowerShell `curl` scripts.

use crate::{
    generator::{NEWLINE, sample::sample_json, text::is_blank},
    model::GeneratorSettings,
    normalized::{Operation, ParameterLocation},
    string_extensions::convert_kebab_case_to_snake_case,
};

/// Renders the PowerShell script for a single operation.
pub fn render(
    settings: &GeneratorSettings,
    base_url: &str,
    path: &str,
    operation: &Operation,
) -> String {
    let mut script = String::new();

    append_summary(&mut script, path, operation);
    let parameters = append_parameters(&mut script, operation);

    let mut url = path.replace('{', "$").replace('}', "");
    if !parameters.is_empty() {
        url.push('?');
        for (name, variable) in &parameters {
            url.push_str(&format!("{name}=${variable}&"));
        }
        url.pop();
    }

    let verb = &operation.method;
    line(&mut script, &format!("curl -X {verb} {base_url}{url} `"));
    line(
        &mut script,
        &format!("  -H 'Accept: {}' `", settings.content_type),
    );
    line(
        &mut script,
        &format!("  -H 'Content-Type: {}' `", settings.content_type),
    );

    if let Some(authorization) = authorization_header(settings) {
        line(
            &mut script,
            &format!("  -H 'Authorization: {authorization}' `"),
        );
    }

    if let Some(body) = &operation.request_body
        && let Some(content_type) = body
            .content
            .iter()
            .map(|media_type| media_type.content_type.as_str())
            .find(|content_type| content_type.contains(&settings.content_type))
    {
        let payload = sample_json(body.schema_for(content_type));
        line(&mut script, &format!("  -d '{payload}'"));
    }

    script.push_str(NEWLINE);
    script
}

fn append_summary(script: &mut String, path: &str, operation: &Operation) {
    line(script, "<#");
    line(script, &format!("  Request: {} {path}", operation.method));

    if let Some(summary) = operation
        .summary
        .as_deref()
        .filter(|value| !is_blank(value))
    {
        line(script, &format!("  Summary: {summary}"));
    }

    if let Some(description) = operation
        .description
        .as_deref()
        .filter(|value| !is_blank(value))
    {
        line(script, &format!("  Description: {description}"));
    }

    line(script, "#>");
}

/// Appends the `param(...)` block and returns the parameter name to variable name pairs.
fn append_parameters(script: &mut String, operation: &Operation) -> Vec<(String, String)> {
    let Some(declared) = &operation.parameters else {
        return Vec::new();
    };

    let declared: Vec<_> = declared
        .iter()
        .filter(|parameter| {
            matches!(
                parameter.location,
                ParameterLocation::Path | ParameterLocation::Query
            )
        })
        .collect();

    if declared.is_empty() {
        script.push_str(NEWLINE);
        return Vec::new();
    }

    line(script, "param(");

    let mut names = Vec::new();
    for parameter in declared {
        let variable = convert_kebab_case_to_snake_case(&parameter.name);

        if let Some(description) = &parameter.description {
            line(script, &format!("   <# {description} #>"));
        }

        line(script, "   [Parameter(Mandatory=$True)]");
        line(script, &format!("   [String] ${variable},"));
        script.push_str(NEWLINE);

        names.push((parameter.name.clone(), variable));
    }

    remove_trailing_parameter_comma(script);
    line(script, ")");
    script.push_str(NEWLINE);

    names
}

/// Drops the comma and blank line the last parameter was rendered with.
fn remove_trailing_parameter_comma(script: &mut String) {
    let suffix = format!(",{NEWLINE}{NEWLINE}");
    if let Some(start) = script.len().checked_sub(suffix.len())
        && script[start..] == suffix
    {
        script.replace_range(start..start + 1 + NEWLINE.len(), "");
    }
}

fn authorization_header(settings: &GeneratorSettings) -> Option<&str> {
    settings
        .authorization_header
        .as_deref()
        .filter(|value| !is_blank(value))
}

fn line(script: &mut String, value: &str) {
    script.push_str(value);
    script.push_str(NEWLINE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalized::{MediaType, Parameter, RequestBody, Schema, SchemaType};

    fn settings() -> GeneratorSettings {
        GeneratorSettings::new("./openapi.json")
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

    fn lines(script: &str) -> Vec<&str> {
        script.split(NEWLINE).collect()
    }

    #[test]
    fn renders_a_request_without_parameters_or_a_body() {
        let script = render(&settings(), "/api/v3", "/pet", &operation());

        assert_eq!(
            lines(&script),
            vec![
                "<#",
                "  Request: GET /pet",
                "#>",
                "curl -X GET /api/v3/pet `",
                "  -H 'Accept: application/json' `",
                "  -H 'Content-Type: application/json' `",
                "",
                "",
            ]
        );
    }

    #[test]
    fn renders_the_summary_and_description() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                summary: Some("Finds pets".to_string()),
                description: Some("Longer text".to_string()),
                ..operation()
            },
        );

        assert!(script.contains("  Summary: Finds pets"));
        assert!(script.contains("  Description: Longer text"));
    }

    #[test]
    fn skips_blank_summaries_and_descriptions() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                summary: Some("   ".to_string()),
                description: Some(String::new()),
                ..operation()
            },
        );

        assert!(!script.contains("Summary:"));
        assert!(!script.contains("Description:"));
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
        assert_eq!(lines(&script)[4], "curl -X GET /pet `");
    }

    #[test]
    fn renders_path_and_query_parameters_into_a_param_block() {
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
                "<#",
                "  Request: GET /pet/{petId}",
                "#>",
                "param(",
                "   <# The pet id #>",
                "   [Parameter(Mandatory=$True)]",
                "   [String] $petid,",
                "",
                "   [Parameter(Mandatory=$True)]",
                "   [String] $api_key",
                ")",
                "",
                "curl -X GET /pet/$petId?petId=$petid&api-key=$api_key `",
                "  -H 'Accept: application/json' `",
                "  -H 'Content-Type: application/json' `",
                "",
                "",
            ]
        );
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

        assert!(script.contains("  -H 'Authorization: Bearer token' `"));
    }

    #[test]
    fn renders_a_request_body_for_the_configured_content_type() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
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

        assert!(script.contains("  -d '{\n  \"name\": \"string\"\n}'"));
    }

    #[test]
    fn omits_the_request_body_when_no_content_type_matches() {
        let script = render(
            &settings(),
            "",
            "/pet",
            &Operation {
                request_body: Some(RequestBody {
                    content: vec![MediaType {
                        content_type: "application/octet-stream".to_string(),
                        schema: None,
                    }],
                }),
                ..operation()
            },
        );

        assert!(!script.contains("-d "));
    }
}
