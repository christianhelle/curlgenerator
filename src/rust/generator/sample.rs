//! Generation of sample request bodies from normalized schemas.

use serde_json::{Map, Value};

use crate::{
    generator::NEWLINE,
    normalized::{Schema, SchemaType},
    platform::{LocalTime, local_time},
};

/// Renders a sample JSON payload for a schema, formatted the way the generated scripts embed it.
///
/// # Examples
///
/// ```
/// use curlgenerator::generator::sample_json;
///
/// assert_eq!(sample_json(None), "{}");
/// ```
pub fn sample_json(schema: Option<&Schema>) -> String {
    let Some(schema) = schema else {
        return "{}".to_string();
    };

    serde_json::to_string_pretty(&sample_value(schema))
        .map(|payload| payload.replace('\n', NEWLINE))
        .unwrap_or_else(|_| "{}".to_string())
}

/// Builds the sample value a schema describes.
pub fn sample_value(schema: &Schema) -> Value {
    if let Some(example) = &schema.example {
        return example.clone();
    }

    match schema.schema_type {
        Some(SchemaType::Object) => Value::Object(
            schema
                .properties
                .iter()
                .map(|(name, property)| (name.clone(), sample_value(property)))
                .collect::<Map<_, _>>(),
        ),
        Some(SchemaType::Array) => match &schema.items {
            Some(items) => Value::Array(vec![sample_value(items)]),
            None => Value::Array(Vec::new()),
        },
        Some(SchemaType::String) => Value::String(sample_string(schema.format.as_deref())),
        Some(SchemaType::Integer) => Value::from(0),
        Some(SchemaType::Number) => Value::from(0.0),
        Some(SchemaType::Boolean) => Value::Bool(false),
        None => Value::String("value".to_string()),
    }
}

fn sample_string(format: Option<&str>) -> String {
    match format {
        Some("date") => {
            format_local_time(|now| format!("{:04}-{:02}-{:02}", now.year, now.month, now.day))
        }
        Some("date-time") => format_local_time(|now| {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                now.year, now.month, now.day, now.hour, now.minute, now.second
            )
        }),
        Some("email") => "user@example.com".to_string(),
        Some("uri") => "https://example.com".to_string(),
        _ => "string".to_string(),
    }
}

/// Formats the current local time, falling back to the plain string sample when the platform
/// cannot tell the time.
fn format_local_time(format: impl FnOnce(LocalTime) -> String) -> String {
    local_time().map_or_else(|| "string".to_string(), format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema_of(schema_type: SchemaType) -> Schema {
        Schema {
            schema_type: Some(schema_type),
            ..Schema::default()
        }
    }

    #[test]
    fn renders_an_empty_object_without_a_schema() {
        assert_eq!(sample_json(None), "{}");
    }

    #[test]
    fn renders_scalar_defaults_per_type() {
        assert_eq!(sample_value(&schema_of(SchemaType::Integer)), json!(0));
        assert_eq!(sample_value(&schema_of(SchemaType::Number)), json!(0.0));
        assert_eq!(sample_value(&schema_of(SchemaType::Boolean)), json!(false));
        assert_eq!(
            sample_value(&schema_of(SchemaType::String)),
            json!("string")
        );
    }

    #[test]
    fn renders_the_placeholder_value_for_untyped_schemas() {
        assert_eq!(sample_value(&Schema::default()), json!("value"));
    }

    #[test]
    fn renders_known_string_formats() {
        let formatted = |format: &str| {
            sample_value(&Schema {
                schema_type: Some(SchemaType::String),
                format: Some(format.to_string()),
                ..Schema::default()
            })
        };

        assert_eq!(formatted("email"), json!("user@example.com"));
        assert_eq!(formatted("uri"), json!("https://example.com"));
        assert_eq!(formatted("date").as_str().unwrap().len(), 10);
        assert!(formatted("date-time").as_str().unwrap().ends_with('Z'));
        assert_eq!(formatted("int64"), json!("string"));
    }

    #[test]
    fn renders_objects_and_arrays_recursively() {
        let schema = Schema {
            schema_type: Some(SchemaType::Object),
            properties: vec![
                ("name".to_string(), schema_of(SchemaType::String)),
                (
                    "tags".to_string(),
                    Schema {
                        schema_type: Some(SchemaType::Array),
                        items: Some(Box::new(schema_of(SchemaType::Integer))),
                        ..Schema::default()
                    },
                ),
            ],
            ..Schema::default()
        };

        assert_eq!(
            sample_value(&schema),
            json!({ "name": "string", "tags": [0] })
        );
    }

    #[test]
    fn renders_an_empty_array_when_the_item_schema_is_missing() {
        assert_eq!(sample_value(&schema_of(SchemaType::Array)), json!([]));
    }

    #[test]
    fn prefers_a_declared_example_over_the_generated_sample() {
        let schema = Schema {
            schema_type: Some(SchemaType::Object),
            example: Some(json!({ "id": 10, "name": "doggie" })),
            properties: vec![("ignored".to_string(), schema_of(SchemaType::String))],
            ..Schema::default()
        };

        assert_eq!(
            sample_json(Some(&schema)),
            format!("{{{NEWLINE}  \"id\": 10,{NEWLINE}  \"name\": \"doggie\"{NEWLINE}}}")
        );
    }

    #[test]
    fn formats_payloads_with_two_space_indentation() {
        let schema = Schema {
            schema_type: Some(SchemaType::Object),
            properties: vec![("name".to_string(), schema_of(SchemaType::String))],
            ..Schema::default()
        };

        assert_eq!(
            sample_json(Some(&schema)),
            format!("{{{NEWLINE}  \"name\": \"string\"{NEWLINE}}}")
        );
    }

    #[cfg(unix)]
    #[test]
    fn renders_dates_in_the_local_time_zone() {
        let local = |format: &str| {
            let output = std::process::Command::new("date")
                .arg(format)
                .output()
                .expect("the date command should run");

            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };
        let sample = |format: &str| {
            sample_value(&Schema {
                schema_type: Some(SchemaType::String),
                format: Some(format.to_string()),
                ..Schema::default()
            })
            .as_str()
            .expect("a string sample")
            .to_string()
        };

        let before = local("+%Y-%m-%dT%H:%M");
        let date_time = sample("date-time");
        let date = sample("date");
        let after = local("+%Y-%m-%dT%H:%M");

        assert!(
            date_time.starts_with(&before) || date_time.starts_with(&after),
            "{date_time} is not the local time between {before} and {after}"
        );
        assert_eq!(date_time.len(), "2026-01-01T00:00:00Z".len(), "{date_time}");
        assert!(
            before.starts_with(&date) || after.starts_with(&date),
            "{date} is not the local date between {before} and {after}"
        );
    }
}
