//! Generation of sample request bodies from normalized schemas.

use chrono::Local;
use serde_json::{Map, Value};

use crate::normalized::{Schema, SchemaType};

/// Renders a sample JSON payload for a schema, formatted the way the generated scripts embed it.
///
/// # Examples
///
/// ```
/// use curlgenerator_core::generator::sample_json;
///
/// assert_eq!(sample_json(None), "{}");
/// ```
pub fn sample_json(schema: Option<&Schema>) -> String {
    let Some(schema) = schema else {
        return "{}".to_string();
    };

    serde_json::to_string_pretty(&sample_value(schema)).unwrap_or_else(|_| "{}".to_string())
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
        Some("date") => Local::now().format("%Y-%m-%d").to_string(),
        Some("date-time") => Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        Some("email") => "user@example.com".to_string(),
        Some("uri") => "https://example.com".to_string(),
        _ => "string".to_string(),
    }
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
            "{\n  \"id\": 10,\n  \"name\": \"doggie\"\n}"
        );
    }

    #[test]
    fn formats_payloads_with_two_space_indentation() {
        let schema = Schema {
            schema_type: Some(SchemaType::Object),
            properties: vec![("name".to_string(), schema_of(SchemaType::String))],
            ..Schema::default()
        };

        assert_eq!(sample_json(Some(&schema)), "{\n  \"name\": \"string\"\n}");
    }
}
